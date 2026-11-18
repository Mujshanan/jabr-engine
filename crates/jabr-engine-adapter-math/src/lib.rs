//! Math-domain adapter for the vendored CivilKit Calcpad calculation core.
//!
//! Jabr-Math remains the source of block analysis and stable authored spans. Calcpad receives
//! an isolated source string with resolved inputs injected as synthetic assignments.

use std::rc::Rc;

use calcpad_core::base_types::real_value::RealValue;
use calcpad_core::base_types::value::Value;
use calcpad_core::data_files::NoFiles;
use calcpad_core::parsers::expression_parser::ExpressionParser;
use jabr_engine_core::{
    AnalysisOutcome, BlockAnalysis, BlockEvaluation, BlockInput, BlockOutput, Diagnostic,
    DomainAdapter, SourceSpan, StructuredValue,
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum SourceSegment {
    SyntheticBinding {
        name: String,
        generated_start: usize,
        generated_end: usize,
    },
    Authored {
        original_start: usize,
        original_end: usize,
        generated_start: usize,
        generated_end: usize,
    },
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct GeneratedSource {
    pub text: String,
    pub segments: Vec<SourceSegment>,
}

/// Analysis implementation supplied by the Jabr-Math facade.
pub trait MathAnalysisProvider {
    fn analyze_math_block(&self, source: &str) -> AnalysisOutcome;
}

/// The Calcpad backend is replaceable. The analyzer is supplied by Jabr-Math.
pub struct CalcpadMathAdapter<A> {
    analyzer: A,
}

impl<A> CalcpadMathAdapter<A> {
    pub fn new(analyzer: A) -> Self {
        Self { analyzer }
    }
}

impl<A: MathAnalysisProvider> DomainAdapter for CalcpadMathAdapter<A> {
    fn analyze(&self, source: &str) -> AnalysisOutcome {
        self.analyzer.analyze_math_block(source)
    }

    fn evaluate(
        &self,
        source: &str,
        analysis: &BlockAnalysis,
        inputs: &[BlockInput],
    ) -> Result<BlockEvaluation, Vec<Diagnostic>> {
        let generated = prepend_bindings(source, inputs)?;
        let mut parser = ExpressionParser::new();

        // A worksheet block is not allowed to read or write arbitrary host files.
        parser.data_files = Rc::new(NoFiles);
        parser.allow_data_write = false;
        parser.debug = true;
        parser.capture_values = true;

        // ExpressionParser owns a fresh MathParser for this evaluation. No mutable Calcpad
        // variable/function/unit state is shared between blocks or worker threads.
        parser.parse(&generated.text, true);

        let mut diagnostics = parser
            .errors_list
            .iter()
            .map(|error| {
                let generated_span = line_span(&generated.text, error.source_line);
                let authored_span =
                    generated_span.and_then(|span| map_authored_span(&generated, span));
                Diagnostic {
                    code: "JBR-MATH-CALCPAD-ERROR".to_owned(),
                    message: error.message.clone(),
                    span: authored_span,
                }
            })
            .collect::<Vec<_>>();

        let mut outputs = Vec::with_capacity(analysis.definitions.len());
        for definition in &analysis.definitions {
            match parser.variable(&definition.name) {
                Some(value) => match convert_value(&value) {
                    Ok(value) => outputs.push(BlockOutput {
                        name: definition.name.clone(),
                        value,
                    }),
                    Err(message) => diagnostics.push(Diagnostic {
                        code: "JBR-MATH-UNSUPPORTED-RESULT".to_owned(),
                        message: format!("{}: {}", definition.name, message),
                        span: Some(definition.span),
                    }),
                },
                None => diagnostics.push(Diagnostic {
                    code: "JBR-MATH-MISSING-OUTPUT".to_owned(),
                    message: format!(
                        "Calcpad did not produce the declared output '{}'",
                        definition.name
                    ),
                    span: Some(definition.span),
                }),
            }
        }

        Ok(BlockEvaluation {
            outputs,
            diagnostics,
        })
    }
}

/// Convert a structured Jabr input into a Calcpad assignment expression.
/// Unsupported values are errors; arbitrary values are never stringified.
pub fn format_binding(input: &BlockInput) -> Result<String, Diagnostic> {
    if !valid_identifier(&input.name) {
        return Err(Diagnostic {
            code: "JBR-MATH-INVALID-BINDING-NAME".to_owned(),
            message: format!("Invalid input name '{}'", input.name),
            span: None,
        });
    }

    let expression = match &input.value {
        StructuredValue::Real(value) | StructuredValue::ApproximateReal { value, .. }
            if value.is_finite() =>
        {
            value.to_string()
        }
        StructuredValue::Quantity { magnitude, unit }
        | StructuredValue::ApproximateQuantity {
            magnitude, unit, ..
        } if magnitude.is_finite() && valid_unit(unit) => {
            format!("{magnitude} {unit}")
        }
        _ => {
            return Err(Diagnostic {
                code: "JBR-MATH-UNSUPPORTED-BINDING-VALUE".to_owned(),
                message: format!(
                    "Input '{}' cannot yet be represented safely by the Calcpad bridge",
                    input.name
                ),
                span: None,
            });
        }
    };
    Ok(format!("{} = {}", input.name, expression))
}

/// Prefix synthetic assignments while preserving a map to authored source byte offsets.
pub fn prepend_bindings(
    authored_source: &str,
    inputs: &[BlockInput],
) -> Result<GeneratedSource, Vec<Diagnostic>> {
    let mut lines = Vec::with_capacity(inputs.len());
    let mut diagnostics = Vec::new();
    for input in inputs {
        match format_binding(input) {
            Ok(line) => lines.push((input.name.clone(), line)),
            Err(diagnostic) => diagnostics.push(diagnostic),
        }
    }
    if !diagnostics.is_empty() {
        return Err(diagnostics);
    }

    let mut text = String::new();
    let mut segments = Vec::with_capacity(lines.len() + 1);
    for (name, line) in lines {
        let start = text.len();
        text.push_str(&line);
        text.push('\n');
        segments.push(SourceSegment::SyntheticBinding {
            name,
            generated_start: start,
            generated_end: text.len(),
        });
    }

    let authored_start = text.len();
    text.push_str(authored_source);
    segments.push(SourceSegment::Authored {
        original_start: 0,
        original_end: authored_source.len(),
        generated_start: authored_start,
        generated_end: text.len(),
    });

    Ok(GeneratedSource { text, segments })
}

/// Map a generated backend span to authored source. Synthetic-prefix spans map to None.
pub fn map_authored_span(source: &GeneratedSource, generated: SourceSpan) -> Option<SourceSpan> {
    let start = generated.start as usize;
    let end = generated.end as usize;
    source.segments.iter().find_map(|segment| match segment {
        SourceSegment::Authored {
            original_start,
            original_end,
            generated_start,
            generated_end,
        } if start >= *generated_start && end <= *generated_end && start <= end => {
            let mapped_start = *original_start + (start - *generated_start);
            let mapped_end = mapped_start + (end - start);
            (mapped_end <= *original_end).then_some(SourceSpan {
                start: mapped_start as u32,
                end: mapped_end as u32,
            })
        }
        _ => None,
    })
}

fn line_span(source: &str, one_based_line: i32) -> Option<SourceSpan> {
    if one_based_line <= 0 {
        return None;
    }
    let mut offset = 0usize;
    for (index, line) in source.split_inclusive('\n').enumerate() {
        if index + 1 == one_based_line as usize {
            let end = offset + line.trim_end_matches('\n').len();
            return SourceSpan::new(offset as u32, end as u32);
        }
        offset += line.len();
    }
    None
}

fn convert_real(value: &RealValue) -> StructuredValue {
    match &value.units {
        Some(unit) => StructuredValue::ApproximateQuantity {
            magnitude: value.d,
            unit: unit.text(),
            backend: "civilkit-calcpad".to_owned(),
        },
        None => StructuredValue::ApproximateReal {
            value: value.d,
            backend: "civilkit-calcpad".to_owned(),
        },
    }
}

fn convert_value(value: &Value) -> Result<StructuredValue, String> {
    match value {
        Value::Real(real) => Ok(convert_real(real)),
        Value::Complex(complex) => match &complex.units {
            Some(unit) => Ok(StructuredValue::ComplexQuantity {
                re: complex.a,
                im: complex.b,
                unit: unit.text(),
            }),
            None => Ok(StructuredValue::Complex {
                re: complex.a,
                im: complex.b,
            }),
        },
        Value::Vector(vector) => Ok(StructuredValue::Vector(
            (0..vector.len())
                .map(|index| convert_real(&vector.get(index)))
                .collect(),
        )),
        Value::Matrix(matrix) => Ok(StructuredValue::Matrix(
            (0..matrix.row_count())
                .map(|row| {
                    (0..matrix.col_count())
                        .map(|column| convert_real(&matrix.get(row, column)))
                        .collect()
                })
                .collect(),
        )),
    }
}

fn valid_identifier(name: &str) -> bool {
    let mut chars = name.chars();
    matches!(chars.next(), Some(c) if c == '_' || c.is_alphabetic())
        && chars.all(|c| c == '_' || c.is_alphanumeric())
}

fn valid_unit(unit: &str) -> bool {
    !unit.is_empty()
        && unit.chars().all(|c| {
            c.is_alphanumeric()
                || matches!(
                    c,
                    '_' | '/' | '*' | '^' | '(' | ')' | '·' | 'μ' | 'µ' | '°' | '-'
                )
        })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn input(name: &str, value: StructuredValue) -> BlockInput {
        BlockInput {
            name: name.to_owned(),
            value,
            producer: None,
        }
    }

    #[test]
    fn prepends_bound_assignments_and_keeps_authored_offsets() {
        let generated = prepend_bindings(
            "y = x * 2",
            &[input(
                "x",
                StructuredValue::Quantity {
                    magnitude: 5.0,
                    unit: "m".to_owned(),
                },
            )],
        );
        let generated = match generated {
            Ok(value) => value,
            Err(_) => return,
        };
        assert_eq!(generated.text, "x = 5 m\ny = x * 2");
        let authored_start = generated.text.find("y =");
        let authored_start = match authored_start {
            Some(value) => value,
            None => return,
        };
        let span = SourceSpan::new(authored_start as u32, (authored_start + 1) as u32);
        let span = match span {
            Some(value) => value,
            None => return,
        };
        assert_eq!(map_authored_span(&generated, span), SourceSpan::new(0, 1));
    }

    #[test]
    fn rejects_unsupported_values_instead_of_stringifying() {
        let result = format_binding(&input(
            "A",
            StructuredValue::Matrix(vec![vec![StructuredValue::Real(1.0)]]),
        ));
        assert!(result.is_err());
    }

    #[test]
    fn rejects_invalid_names() {
        let result = format_binding(&input("x; #include 'bad'", StructuredValue::Real(1.0)));
        assert!(result.is_err());
    }

    #[test]
    fn preserves_approximation_provenance_on_input() {
        let result = format_binding(&input(
            "x",
            StructuredValue::ApproximateQuantity {
                magnitude: 5.0,
                unit: "m".to_owned(),
                backend: "civilkit-calcpad".to_owned(),
            },
        ));
        assert_eq!(result, Ok("x = 5 m".to_owned()));
    }

    struct NoopAnalyzer;

    impl MathAnalysisProvider for NoopAnalyzer {
        fn analyze_math_block(&self, _source: &str) -> AnalysisOutcome {
            AnalysisOutcome::Available(BlockAnalysis::default())
        }
    }

    #[test]
    fn evaluates_block_with_injected_quantity_and_extracts_declared_output() {
        let adapter = CalcpadMathAdapter::new(NoopAnalyzer);
        let analysis = BlockAnalysis {
            definitions: vec![jabr_engine_core::Definition {
                name: "y".to_owned(),
                span: SourceSpan { start: 0, end: 1 },
            }],
            references: vec![jabr_engine_core::Reference {
                name: "x".to_owned(),
                span: SourceSpan { start: 4, end: 5 },
            }],
        };
        let inputs = [input(
            "x",
            StructuredValue::Quantity {
                magnitude: 5.0,
                unit: "m".to_owned(),
            },
        )];
        let evaluated = adapter.evaluate("y = x * 2", &analysis, &inputs);
        assert!(evaluated.is_ok());
        if let Ok(evaluation) = evaluated {
            assert!(evaluation.diagnostics.is_empty());
            assert_eq!(evaluation.outputs.len(), 1);
            if let Some(output) = evaluation.outputs.first() {
                assert!(matches!(
                    &output.value,
                    StructuredValue::ApproximateQuantity { .. }
                ));
                if let StructuredValue::ApproximateQuantity {
                    magnitude,
                    unit,
                    backend,
                } = &output.value
                {
                    assert!((*magnitude - 10.0).abs() < 1e-9);
                    assert_eq!(unit, "m");
                    assert_eq!(backend, "civilkit-calcpad");
                }
            }
        }
    }
}
