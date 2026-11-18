//! Jabr-owned worksheet execution contracts and deterministic orchestration primitives.
//!
//! This crate deliberately does not parse mathematical expressions. Domain adapters provide
//! analysis and evaluation while Engine owns block ordering, bindings, and result lifecycle.

use std::collections::{BTreeMap, BTreeSet};

/// Stable identifier supplied by the authored document/UI.
#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct BlockId(pub String);

/// Byte offsets into the original authored block source (half-open range).
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SourceSpan {
    pub start: u32,
    pub end: u32,
}

impl SourceSpan {
    pub fn new(start: u32, end: u32) -> Option<Self> {
        (start <= end).then_some(Self { start, end })
    }
}

/// Starter transport value. The stable mathematical value model belongs to Jabr-Math.
#[derive(Clone, Debug, PartialEq)]
pub enum StructuredValue {
    Real(f64),
    ApproximateReal {
        value: f64,
        backend: String,
    },
    Complex {
        re: f64,
        im: f64,
    },
    ComplexQuantity {
        re: f64,
        im: f64,
        unit: String,
    },
    Quantity {
        magnitude: f64,
        unit: String,
    },
    ApproximateQuantity {
        magnitude: f64,
        unit: String,
        backend: String,
    },
    Vector(Vec<StructuredValue>),
    Matrix(Vec<Vec<StructuredValue>>),
    Boolean(bool),
    Text(String),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Definition {
    pub name: String,
    pub span: SourceSpan,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Reference {
    pub name: String,
    pub span: SourceSpan,
}

#[derive(Clone, Debug, PartialEq)]
pub enum AnalysisOutcome {
    Available(BlockAnalysis),
    Failed(Vec<Diagnostic>),
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct BlockAnalysis {
    pub definitions: Vec<Definition>,
    pub references: Vec<Reference>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Diagnostic {
    pub code: String,
    pub message: String,
    pub span: Option<SourceSpan>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct BlockInput {
    pub name: String,
    pub value: StructuredValue,
    pub producer: Option<BlockId>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct BlockOutput {
    pub name: String,
    pub value: StructuredValue,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct BlockEvaluation {
    pub outputs: Vec<BlockOutput>,
    pub diagnostics: Vec<Diagnostic>,
}

/// Domain-specific adapter contract. Analysis uses authored-source offsets.
/// Evaluation receives only Engine-resolved inputs and returns typed values.
pub trait DomainAdapter {
    fn analyze(&self, source: &str) -> AnalysisOutcome;

    fn evaluate(
        &self,
        source: &str,
        analysis: &BlockAnalysis,
        inputs: &[BlockInput],
    ) -> Result<BlockEvaluation, Vec<Diagnostic>>;
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorksheetBlock {
    pub id: BlockId,
    pub kind: String,
    pub source: String,
    /// Normalized row key from the document layout model, not a CSS pixel coordinate.
    pub row: u32,
    /// Normalized column key from the document layout model.
    pub column: u32,
}

/// Stable visual reading order: top-to-bottom, then left-to-right within a row.
pub fn visual_reading_order(blocks: &mut [WorksheetBlock]) {
    blocks.sort_by(|a, b| {
        a.row
            .cmp(&b.row)
            .then_with(|| a.column.cmp(&b.column))
            .then_with(|| a.id.cmp(&b.id))
    });
}

/// Transient environment containing exports from already executed blocks.
#[derive(Clone, Debug, Default)]
pub struct ExecutionEnvironment {
    values: BTreeMap<String, (StructuredValue, BlockId)>,
    definitions: BTreeMap<String, BlockId>,
}

impl ExecutionEnvironment {
    pub fn get(&self, name: &str) -> Option<BlockInput> {
        self.values.get(name).map(|(value, producer)| BlockInput {
            name: name.to_owned(),
            value: value.clone(),
            producer: Some(producer.clone()),
        })
    }

    pub fn contains(&self, name: &str) -> bool {
        self.values.contains_key(name)
    }

    pub fn publish(
        &mut self,
        block_id: BlockId,
        outputs: impl IntoIterator<Item = BlockOutput>,
    ) -> Result<(), Diagnostic> {
        let outputs: Vec<BlockOutput> = outputs.into_iter().collect();
        let mut names = BTreeSet::new();
        for output in &outputs {
            if !names.insert(output.name.clone()) {
                return Err(Diagnostic {
                    code: "JBR-ENGINE-DUPLICATE-DEFINITION".to_owned(),
                    message: format!("Duplicate worksheet definition: {}", output.name),
                    span: None,
                });
            }
        }
        for output in outputs {
            self.definitions
                .insert(output.name.clone(), block_id.clone());
            self.values
                .insert(output.name, (output.value, block_id.clone()));
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn block(id: &str, row: u32, column: u32) -> WorksheetBlock {
        WorksheetBlock {
            id: BlockId(id.to_owned()),
            kind: "math".to_owned(),
            source: String::new(),
            row,
            column,
        }
    }

    #[test]
    fn reading_order_is_row_major_not_column_major() {
        let mut blocks = vec![
            block("lower-left", 1, 0),
            block("upper-right", 0, 1),
            block("upper-left", 0, 0),
        ];
        visual_reading_order(&mut blocks);
        let ids: Vec<&str> = blocks.iter().map(|b| b.id.0.as_str()).collect();
        assert_eq!(ids, ["upper-left", "upper-right", "lower-left"]);
    }

    #[test]
    fn environment_publishes_and_resolves_structured_values() {
        let mut env = ExecutionEnvironment::default();
        let published = env.publish(
            BlockId("block-a".to_owned()),
            [BlockOutput {
                name: "x".to_owned(),
                value: StructuredValue::Quantity {
                    magnitude: 5.0,
                    unit: "m".to_owned(),
                },
            }],
        );
        assert!(published.is_ok());
        assert_eq!(
            env.get("x").map(|input| input.value),
            Some(StructuredValue::Quantity {
                magnitude: 5.0,
                unit: "m".to_owned()
            })
        );
    }


    #[test]
    fn later_blocks_reassign_existing_names_in_sequential_order() {
        let mut env = ExecutionEnvironment::default();
        let first = env.publish(
            BlockId("block-a".to_owned()),
            [BlockOutput {
                name: "x".to_owned(),
                value: StructuredValue::Real(5.0),
            }],
        );
        assert!(first.is_ok());
        let second = env.publish(
            BlockId("block-b".to_owned()),
            [BlockOutput {
                name: "x".to_owned(),
                value: StructuredValue::Real(20.0),
            }],
        );
        assert!(second.is_ok());
        let current = env.get("x");
        assert_eq!(current.as_ref().map(|input| input.value.clone()), Some(StructuredValue::Real(20.0)));
        assert_eq!(current.and_then(|input| input.producer), Some(BlockId("block-b".to_owned())));
    }

    #[test]
    fn duplicate_exports_are_rejected_without_partial_publish() {
        let mut env = ExecutionEnvironment::default();
        let result = env.publish(
            BlockId("block-a".to_owned()),
            [
                BlockOutput {
                    name: "x".to_owned(),
                    value: StructuredValue::Real(1.0),
                },
                BlockOutput {
                    name: "x".to_owned(),
                    value: StructuredValue::Real(2.0),
                },
            ],
        );
        assert!(result.is_err());
        assert!(!env.contains("x"));
    }
}
