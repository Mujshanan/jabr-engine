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

/// The producer of a named worksheet export. Values remain in Jabr Kernel state.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ExportRef {
    pub name: String,
    pub producer: BlockId,
}

/// Engine-owned name-to-producer binding table. It never stores variable values or snapshots;
/// those remain Kernel-owned and are fetched when an adapter evaluation needs an input.
#[derive(Clone, Debug, Default)]
pub struct BindingTable {
    exports: BTreeMap<String, ExportRef>,
}

impl BindingTable {
    pub fn resolve(&self, name: &str) -> Option<&ExportRef> {
        self.exports.get(name)
    }

    /// Publish names declared by a successfully analyzed block. A later sequential assignment
    /// to an existing name updates its producer; duplicate names within one block are rejected.
    pub fn publish(
        &mut self,
        block_id: BlockId,
        definitions: impl IntoIterator<Item = Definition>,
    ) -> Result<(), Diagnostic> {
        let definitions: Vec<Definition> = definitions.into_iter().collect();
        let mut names = BTreeSet::new();
        for definition in &definitions {
            if !names.insert(definition.name.clone()) {
                return Err(Diagnostic {
                    code: "JBR-ENGINE-DUPLICATE-DEFINITION".to_owned(),
                    message: format!("Duplicate worksheet definition: {}", definition.name),
                    span: Some(definition.span),
                });
            }
        }
        for definition in definitions {
            self.exports.insert(
                definition.name.clone(),
                ExportRef {
                    name: definition.name,
                    producer: block_id.clone(),
                },
            );
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
    fn binding_table_resolves_export_producers_without_storing_values() {
        let mut bindings = BindingTable::default();
        let published = bindings.publish(
            BlockId("block-a".to_owned()),
            [Definition {
                name: "x".to_owned(),
                span: SourceSpan { start: 0, end: 1 },
            }],
        );
        assert!(published.is_ok());
        assert_eq!(
            bindings.resolve("x"),
            Some(&ExportRef {
                name: "x".to_owned(),
                producer: BlockId("block-a".to_owned()),
            })
        );
    }

    #[test]
    fn later_blocks_reassign_existing_names_in_sequential_order() {
        let mut bindings = BindingTable::default();
        let first = bindings.publish(
            BlockId("block-a".to_owned()),
            [Definition {
                name: "x".to_owned(),
                span: SourceSpan { start: 0, end: 1 },
            }],
        );
        assert!(first.is_ok());
        let second = bindings.publish(
            BlockId("block-b".to_owned()),
            [Definition {
                name: "x".to_owned(),
                span: SourceSpan { start: 0, end: 1 },
            }],
        );
        assert!(second.is_ok());
        assert_eq!(
            bindings.resolve("x"),
            Some(&ExportRef {
                name: "x".to_owned(),
                producer: BlockId("block-b".to_owned()),
            })
        );
    }

    #[test]
    fn duplicate_exports_are_rejected_without_partial_publish() {
        let mut bindings = BindingTable::default();
        let result = bindings.publish(
            BlockId("block-a".to_owned()),
            [
                Definition {
                    name: "x".to_owned(),
                    span: SourceSpan { start: 0, end: 1 },
                },
                Definition {
                    name: "x".to_owned(),
                    span: SourceSpan { start: 4, end: 5 },
                },
            ],
        );
        assert!(result.is_err());
        assert!(bindings.resolve("x").is_none());
    }}
