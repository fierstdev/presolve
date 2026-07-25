//! Stable function summaries derived from control-flow and explicit call facts.

use std::collections::{BTreeMap, BTreeSet};

use serde::Serialize;

use crate::{ControlFlowAccessV1, ControlFlowGraphV1, ControlFlowProvenanceV1};

pub const FUNCTION_SUMMARY_SCHEMA_VERSION: u32 = 1;

/// Call evidence supplied by a canonical lowering stage.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct FunctionCallFactV1 {
    pub caller: String,
    pub callee: Option<String>,
    pub provenance: ControlFlowProvenanceV1,
}

/// Whether the supplied call facts account for every call in the selected scope.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum FunctionCallCoverageV1 {
    Complete,
    Unavailable,
}

/// The closed call-fact input to the function-summary projection.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct FunctionCallFactsV1 {
    pub coverage: FunctionCallCoverageV1,
    pub calls: Vec<FunctionCallFactV1>,
}

/// Stable direct and transitive facts for one function.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct FunctionSummaryV1 {
    pub id: String,
    pub module_path: String,
    pub direct_reads: Vec<ControlFlowAccessV1>,
    pub direct_writes: Vec<ControlFlowAccessV1>,
    pub direct_callees: Vec<String>,
    pub has_direct_unknown_call: bool,
    pub call_coverage: FunctionCallCoverageV1,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub transitive_reads: Option<Vec<ControlFlowAccessV1>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub transitive_writes: Option<Vec<ControlFlowAccessV1>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub transitive_callees: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub has_transitive_unknown_call: Option<bool>,
}

/// A versioned function-summary projection.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct FunctionSummaryGraphV1 {
    pub schema_version: u32,
    pub summaries: Vec<FunctionSummaryV1>,
}

/// Invalid call evidence cannot silently alter a function summary.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FunctionSummaryErrorV1 {
    UnknownCaller(String),
    UnknownCallee { caller: String, callee: String },
}

impl std::fmt::Display for FunctionSummaryErrorV1 {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::UnknownCaller(caller) => {
                write!(formatter, "call fact references unknown caller {caller}")
            }
            Self::UnknownCallee { caller, callee } => {
                write!(
                    formatter,
                    "call fact from {caller} references unknown callee {callee}"
                )
            }
        }
    }
}

impl std::error::Error for FunctionSummaryErrorV1 {}

/// Builds direct summaries and, only for complete call coverage, transitive facts.
pub fn build_function_summary_graph_v1(
    control_flow: &ControlFlowGraphV1,
    call_facts: &FunctionCallFactsV1,
) -> Result<FunctionSummaryGraphV1, FunctionSummaryErrorV1> {
    let functions = control_flow
        .functions
        .iter()
        .map(|function| (function.id.clone(), function))
        .collect::<BTreeMap<_, _>>();
    let mut calls = functions
        .keys()
        .cloned()
        .map(|id| (id, Vec::new()))
        .collect::<BTreeMap<_, Vec<&FunctionCallFactV1>>>();
    for fact in &call_facts.calls {
        if !functions.contains_key(&fact.caller) {
            return Err(FunctionSummaryErrorV1::UnknownCaller(fact.caller.clone()));
        }
        if let Some(callee) = &fact.callee {
            if !functions.contains_key(callee) {
                return Err(FunctionSummaryErrorV1::UnknownCallee {
                    caller: fact.caller.clone(),
                    callee: callee.clone(),
                });
            }
        }
        calls.entry(fact.caller.clone()).or_default().push(fact);
    }

    let direct = functions
        .iter()
        .map(|(id, function)| {
            let reads = function
                .blocks
                .iter()
                .flat_map(|block| block.reads.iter().cloned())
                .collect::<BTreeSet<_>>();
            let writes = function
                .blocks
                .iter()
                .flat_map(|block| block.writes.iter().cloned())
                .collect::<BTreeSet<_>>();
            let callees = calls[id]
                .iter()
                .filter_map(|fact| fact.callee.clone())
                .collect::<BTreeSet<_>>();
            let unknown = calls[id].iter().any(|fact| fact.callee.is_none());
            (
                id.clone(),
                SummaryAggregate {
                    reads,
                    writes,
                    callees,
                    unknown,
                },
            )
        })
        .collect::<BTreeMap<_, _>>();
    let transitive = match call_facts.coverage {
        FunctionCallCoverageV1::Complete => Some(derive_transitive_summaries(&direct)),
        FunctionCallCoverageV1::Unavailable => None,
    };
    let summaries = functions
        .iter()
        .map(|(id, function)| {
            let direct = &direct[id];
            let transitive = transitive.as_ref().map(|summaries| &summaries[id]);
            FunctionSummaryV1 {
                id: id.clone(),
                module_path: function.module_path.clone(),
                direct_reads: direct.reads.iter().cloned().collect(),
                direct_writes: direct.writes.iter().cloned().collect(),
                direct_callees: direct.callees.iter().cloned().collect(),
                has_direct_unknown_call: direct.unknown,
                call_coverage: call_facts.coverage,
                transitive_reads: transitive.map(|summary| summary.reads.iter().cloned().collect()),
                transitive_writes: transitive
                    .map(|summary| summary.writes.iter().cloned().collect()),
                transitive_callees: transitive
                    .map(|summary| summary.callees.iter().cloned().collect()),
                has_transitive_unknown_call: transitive.map(|summary| summary.unknown),
            }
        })
        .collect();
    Ok(FunctionSummaryGraphV1 {
        schema_version: FUNCTION_SUMMARY_SCHEMA_VERSION,
        summaries,
    })
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct SummaryAggregate {
    reads: BTreeSet<ControlFlowAccessV1>,
    writes: BTreeSet<ControlFlowAccessV1>,
    callees: BTreeSet<String>,
    unknown: bool,
}

fn derive_transitive_summaries(
    direct: &BTreeMap<String, SummaryAggregate>,
) -> BTreeMap<String, SummaryAggregate> {
    let mut summaries = direct.clone();
    let mut changed = true;
    while changed {
        changed = false;
        for (id, direct_summary) in direct {
            let mut next = direct_summary.clone();
            for callee in &direct_summary.callees {
                let callee_summary = &summaries[callee];
                next.reads.extend(callee_summary.reads.iter().cloned());
                next.writes.extend(callee_summary.writes.iter().cloned());
                next.callees.extend(callee_summary.callees.iter().cloned());
                next.unknown |= callee_summary.unknown;
            }
            if summaries.get(id) != Some(&next) {
                summaries.insert(id.clone(), next);
                changed = true;
            }
        }
    }
    summaries
}

#[cfg(test)]
mod tests {
    use crate::{
        ControlFlowAccessKindV1, ControlFlowAccessV1, ControlFlowBlockV1,
        ControlFlowCoverageStatusV1, ControlFlowCoverageV1, ControlFlowFunctionV1,
        ControlFlowGraphV1, ControlFlowProvenanceV1,
    };

    use super::{
        build_function_summary_graph_v1, FunctionCallCoverageV1, FunctionCallFactV1,
        FunctionCallFactsV1,
    };

    fn provenance() -> ControlFlowProvenanceV1 {
        ControlFlowProvenanceV1 {
            path: "src/App.tsx".into(),
            start: 0,
            end: 1,
            line: 1,
            column: 1,
        }
    }

    fn function(id: &str, module_path: &str, read: &str, write: &str) -> ControlFlowFunctionV1 {
        ControlFlowFunctionV1 {
            module_path: module_path.into(),
            id: id.into(),
            name: id.into(),
            provenance: provenance(),
            entry_block: format!("{id}/entry"),
            blocks: vec![ControlFlowBlockV1 {
                id: format!("{id}/entry"),
                provenance: provenance(),
                observable_instructions: Vec::new(),
                reads: vec![ControlFlowAccessV1 {
                    kind: ControlFlowAccessKindV1::Storage,
                    id: read.into(),
                }],
                writes: vec![ControlFlowAccessV1 {
                    kind: ControlFlowAccessKindV1::Storage,
                    id: write.into(),
                }],
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
        }
    }

    #[test]
    fn summarizes_cross_module_calls_transitively_and_keeps_unknown_calls_conservative() {
        let graph = ControlFlowGraphV1 {
            schema_version: 1,
            functions: vec![
                function("app", "src/App.tsx", "app-read", "app-write"),
                function("helper", "src/helper.ts", "helper-read", "helper-write"),
            ],
        };
        let facts = FunctionCallFactsV1 {
            coverage: FunctionCallCoverageV1::Complete,
            calls: vec![
                FunctionCallFactV1 {
                    caller: "app".into(),
                    callee: Some("helper".into()),
                    provenance: provenance(),
                },
                FunctionCallFactV1 {
                    caller: "helper".into(),
                    callee: None,
                    provenance: provenance(),
                },
            ],
        };
        let summary = build_function_summary_graph_v1(&graph, &facts).expect("valid call facts");
        let app = summary
            .summaries
            .iter()
            .find(|summary| summary.id == "app")
            .unwrap();
        assert_eq!(app.direct_callees, vec!["helper"]);
        assert!(app
            .transitive_reads
            .as_ref()
            .unwrap()
            .iter()
            .any(|access| access.id == "helper-read"));
        assert!(app.has_transitive_unknown_call.unwrap());
    }

    #[test]
    fn leaves_transitive_facts_unavailable_when_call_coverage_is_unavailable() {
        let graph = ControlFlowGraphV1 {
            schema_version: 1,
            functions: vec![function("app", "src/App.tsx", "read", "write")],
        };
        let facts = FunctionCallFactsV1 {
            coverage: FunctionCallCoverageV1::Unavailable,
            calls: Vec::new(),
        };
        let summary = build_function_summary_graph_v1(&graph, &facts).expect("valid empty facts");
        assert!(summary.summaries[0].transitive_reads.is_none());
        assert!(summary.summaries[0].has_transitive_unknown_call.is_none());
    }
}
