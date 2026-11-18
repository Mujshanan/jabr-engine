//! Math-domain adapter scaffolding.
//!
//! Calcpad parser integration is isolated behind this crate. The vendored source import is a
//! prerequisite; no external Git dependency is used here.

use jabr_engine_core::{BlockInput, Diagnostic, SourceSpan, StructuredValue};

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
        StructuredValue::Real(value) if value.is_finite() => value.to_string(),
        StructuredValue::Quantity { magnitude, unit }
            if magnitude.is_finite() && valid_unit(unit) =>
        {
            format!("{magnitude} {unit}")
        }
        StructuredValue::Boolean(value) => value.to_string(),
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
            let mapped_start = original_start + (start - generated_start);
            let mapped_end = mapped_start + (end - start);
            (mapped_end <= *original_end).then_some(SourceSpan {
                start: mapped_start as u32,
                end: mapped_end as u32,
            })
        }
        _ => None,
    })
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
        let result = format_binding(&input(
            "x; #include 'bad'",
            StructuredValue::Real(1.0),
        ));
        assert!(result.is_err());
    }
}
