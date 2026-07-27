//! Conservative purity and effect classification over function summaries.

use serde::Serialize;

use crate::{
    ControlFlowAccessKindV1, ControlFlowGraphV1, FunctionCallCoverageV1, FunctionSummaryGraphV1,
};

pub const PURITY_EFFECT_SCHEMA_VERSION: u32 = 1;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum FunctionPurityV1 {
    Pure,
    Impure,
    Unknown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum FunctionEffectKindV1 {
    StorageWrite,
    ContextSlotWrite,
    ObservableInstruction,
    ResourceRead,
    UnknownCall,
    UnavailableCallCoverage,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize)]
pub struct FunctionEffectFactV1 {
    pub kind: FunctionEffectKindV1,
    pub id: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct FunctionPurityEffectV1 {
    pub function: String,
    pub purity: FunctionPurityV1,
    pub effects: Vec<FunctionEffectFactV1>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct PurityEffectGraphV1 {
    pub schema_version: u32,
    pub functions: Vec<FunctionPurityEffectV1>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PurityEffectErrorV1 {
    MissingSummary(String),
}

impl std::fmt::Display for PurityEffectErrorV1 {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::MissingSummary(function) => {
                write!(formatter, "missing function summary for {function}")
            }
        }
    }
}

impl std::error::Error for PurityEffectErrorV1 {}

/// Classifies only effects directly encoded by canonical CFG and summary facts.
pub fn build_purity_effect_graph_v1(
    control_flow: &ControlFlowGraphV1,
    summaries: &FunctionSummaryGraphV1,
) -> Result<PurityEffectGraphV1, PurityEffectErrorV1> {
    let summaries = summaries
        .summaries
        .iter()
        .map(|summary| (summary.id.as_str(), summary))
        .collect::<std::collections::BTreeMap<_, _>>();
    let mut functions = Vec::new();
    for function in &control_flow.functions {
        let Some(summary) = summaries.get(function.id.as_str()) else {
            return Err(PurityEffectErrorV1::MissingSummary(function.id.clone()));
        };
        let mut effects = std::collections::BTreeSet::new();
        for access in &summary.direct_writes {
            let kind = match access.kind {
                ControlFlowAccessKindV1::Storage => Some(FunctionEffectKindV1::StorageWrite),
                ControlFlowAccessKindV1::ContextSlot => {
                    Some(FunctionEffectKindV1::ContextSlotWrite)
                }
                ControlFlowAccessKindV1::Value
                | ControlFlowAccessKindV1::Computed
                | ControlFlowAccessKindV1::Resource => None,
            };
            if let Some(kind) = kind {
                effects.insert(FunctionEffectFactV1 {
                    kind,
                    id: access.id.clone(),
                });
            }
        }
        for access in &summary.direct_reads {
            if access.kind == ControlFlowAccessKindV1::Resource {
                effects.insert(FunctionEffectFactV1 {
                    kind: FunctionEffectKindV1::ResourceRead,
                    id: access.id.clone(),
                });
            }
        }
        for block in &function.blocks {
            for instruction in &block.observable_instructions {
                effects.insert(FunctionEffectFactV1 {
                    kind: FunctionEffectKindV1::ObservableInstruction,
                    id: instruction.clone(),
                });
            }
        }
        if summary.has_direct_unknown_call {
            effects.insert(FunctionEffectFactV1 {
                kind: FunctionEffectKindV1::UnknownCall,
                id: function.id.clone(),
            });
        }
        if summary.call_coverage == FunctionCallCoverageV1::Unavailable {
            effects.insert(FunctionEffectFactV1 {
                kind: FunctionEffectKindV1::UnavailableCallCoverage,
                id: function.id.clone(),
            });
        }
        let effects = effects.into_iter().collect::<Vec<_>>();
        let impure = effects.iter().any(|effect| {
            matches!(
                effect.kind,
                FunctionEffectKindV1::StorageWrite
                    | FunctionEffectKindV1::ContextSlotWrite
                    | FunctionEffectKindV1::ObservableInstruction
            )
        });
        let unknown = effects.iter().any(|effect| {
            matches!(
                effect.kind,
                FunctionEffectKindV1::ResourceRead
                    | FunctionEffectKindV1::UnknownCall
                    | FunctionEffectKindV1::UnavailableCallCoverage
            )
        });
        functions.push(FunctionPurityEffectV1 {
            function: function.id.clone(),
            purity: if impure {
                FunctionPurityV1::Impure
            } else if unknown {
                FunctionPurityV1::Unknown
            } else {
                FunctionPurityV1::Pure
            },
            effects,
        });
    }
    functions.sort_by(|left, right| left.function.cmp(&right.function));
    Ok(PurityEffectGraphV1 {
        schema_version: PURITY_EFFECT_SCHEMA_VERSION,
        functions,
    })
}

#[cfg(test)]
mod tests {
    use crate::{
        ControlFlowAccessKindV1, ControlFlowAccessV1, ControlFlowBlockV1,
        ControlFlowCoverageStatusV1, ControlFlowCoverageV1, ControlFlowFunctionV1,
        ControlFlowGraphV1, ControlFlowProvenanceV1, FunctionCallCoverageV1,
        FunctionSummaryGraphV1, FunctionSummaryV1,
    };

    use super::{build_purity_effect_graph_v1, FunctionPurityV1};

    fn provenance() -> ControlFlowProvenanceV1 {
        ControlFlowProvenanceV1 {
            path: "src/App.tsx".into(),
            start: 0,
            end: 1,
            line: 1,
            column: 1,
        }
    }

    fn graph() -> ControlFlowGraphV1 {
        ControlFlowGraphV1 {
            schema_version: 1,
            functions: vec![ControlFlowFunctionV1 {
                module_path: "src/App.tsx".into(),
                id: "app".into(),
                name: "app".into(),
                provenance: provenance(),
                entry_block: "app/entry".into(),
                blocks: vec![ControlFlowBlockV1 {
                    id: "app/entry".into(),
                    provenance: provenance(),
                    reads: Vec::new(),
                    writes: Vec::new(),
                    observable_instructions: vec!["app/effect".into()],
                }],
                branch_edges: Vec::new(),
                loops: Vec::new(),
                coverage: ControlFlowCoverageV1 {
                    branch_topology: ControlFlowCoverageStatusV1::Available,
                    definite_dataflow: ControlFlowCoverageStatusV1::Available,
                    natural_loops: ControlFlowCoverageStatusV1::Available,
                    exception_paths: ControlFlowCoverageStatusV1::Unavailable,
                    async_suspension: ControlFlowCoverageStatusV1::Unavailable,
                    unknown_calls: ControlFlowCoverageStatusV1::Unavailable,
                    capture_escape: ControlFlowCoverageStatusV1::Unavailable,
                    resource_cancellation: ControlFlowCoverageStatusV1::Unavailable,
                },
            }],
        }
    }

    #[test]
    fn classifies_observable_writes_as_impure_and_incomplete_calls_as_unknown() {
        let summary = FunctionSummaryGraphV1 {
            schema_version: 1,
            summaries: vec![FunctionSummaryV1 {
                id: "app".into(),
                module_path: "src/App.tsx".into(),
                direct_reads: vec![ControlFlowAccessV1 {
                    kind: ControlFlowAccessKindV1::Resource,
                    id: "resource:profile".into(),
                }],
                direct_writes: Vec::new(),
                direct_callees: Vec::new(),
                has_direct_unknown_call: false,
                call_coverage: FunctionCallCoverageV1::Unavailable,
                transitive_reads: None,
                transitive_writes: None,
                transitive_callees: None,
                has_transitive_unknown_call: None,
            }],
        };
        let classified =
            build_purity_effect_graph_v1(&graph(), &summary).expect("matching summary");
        assert_eq!(classified.functions[0].purity, FunctionPurityV1::Impure);
        assert_eq!(classified.functions[0].effects.len(), 3);
    }
}
