//! Explicit capture and escape facts for resumability admission.
//!
//! Canonical IR does not yet represent closure capture, suspension, or value
//! escape. This product therefore accepts only explicit facts from a future
//! lowering stage and rejects resume admission when any required coverage is
//! unavailable.

use std::collections::{BTreeMap, BTreeSet};

use serde::Serialize;

use crate::{
    ControlFlowAccessV1, ControlFlowCoverageStatusV1, ControlFlowGraphV1, ControlFlowProvenanceV1,
    FunctionCallCoverageV1, FunctionSummaryGraphV1,
};

pub const CAPTURE_ESCAPE_SCHEMA_VERSION: u32 = 1;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum CaptureEscapeCoverageV1 {
    Complete,
    Unavailable,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum CaptureEscapeKindV1 {
    Capture,
    Escape,
}

/// Evidence supplied by a closure/async lowering that owns capture semantics.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct CaptureEscapeFactV1 {
    pub function: String,
    pub access: ControlFlowAccessV1,
    pub kind: CaptureEscapeKindV1,
    pub provenance: ControlFlowProvenanceV1,
}

/// The closed fact input for capture and escape analysis.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct CaptureEscapeFactsV1 {
    pub coverage: CaptureEscapeCoverageV1,
    pub facts: Vec<CaptureEscapeFactV1>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ResumeCaptureAdmissionV1 {
    Admissible,
    RejectedUnavailableCoverage,
    RejectedUnknownCall,
}

/// Deterministic capture and escape facts for one function.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct FunctionCaptureEscapeV1 {
    pub function: String,
    pub captures: Vec<ControlFlowAccessV1>,
    pub escapes: Vec<ControlFlowAccessV1>,
    pub resume_admission: ResumeCaptureAdmissionV1,
}

/// A versioned capture/escape analysis product.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct CaptureEscapeGraphV1 {
    pub schema_version: u32,
    pub functions: Vec<FunctionCaptureEscapeV1>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CaptureEscapeErrorV1 {
    MissingSummary(String),
    UnknownFunction(String),
    UnknownAccess {
        function: String,
        access: ControlFlowAccessV1,
    },
}

impl std::fmt::Display for CaptureEscapeErrorV1 {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::MissingSummary(function) => {
                write!(formatter, "missing function summary for {function}")
            }
            Self::UnknownFunction(function) => write!(
                formatter,
                "capture fact references unknown function {function}"
            ),
            Self::UnknownAccess { function, access } => write!(
                formatter,
                "capture fact for {function} references non-CFG access {:?}:{}",
                access.kind, access.id
            ),
        }
    }
}

impl std::error::Error for CaptureEscapeErrorV1 {}

/// Builds stable capture/escape records and conservative resume admission.
pub fn build_capture_escape_graph_v1(
    control_flow: &ControlFlowGraphV1,
    summaries: &FunctionSummaryGraphV1,
    facts: &CaptureEscapeFactsV1,
) -> Result<CaptureEscapeGraphV1, CaptureEscapeErrorV1> {
    let summaries = summaries
        .summaries
        .iter()
        .map(|summary| (summary.id.as_str(), summary))
        .collect::<BTreeMap<_, _>>();
    let functions = control_flow
        .functions
        .iter()
        .map(|function| (function.id.as_str(), function))
        .collect::<BTreeMap<_, _>>();
    let mut grouped = functions
        .keys()
        .map(|id| (*id, BTreeSet::new()))
        .collect::<BTreeMap<_, BTreeSet<(CaptureEscapeKindV1, ControlFlowAccessV1)>>>();

    for fact in &facts.facts {
        let Some(function) = functions.get(fact.function.as_str()) else {
            return Err(CaptureEscapeErrorV1::UnknownFunction(fact.function.clone()));
        };
        let accesses = function
            .blocks
            .iter()
            .flat_map(|block| block.reads.iter().chain(&block.writes))
            .collect::<BTreeSet<_>>();
        if !accesses.contains(&fact.access) {
            return Err(CaptureEscapeErrorV1::UnknownAccess {
                function: fact.function.clone(),
                access: fact.access.clone(),
            });
        }
        grouped
            .entry(fact.function.as_str())
            .or_default()
            .insert((fact.kind, fact.access.clone()));
    }

    let mut results = Vec::new();
    for function in &control_flow.functions {
        let Some(summary) = summaries.get(function.id.as_str()) else {
            return Err(CaptureEscapeErrorV1::MissingSummary(function.id.clone()));
        };
        let facts_for_function = &grouped[function.id.as_str()];
        let captures = facts_for_function
            .iter()
            .filter(|(kind, _)| *kind == CaptureEscapeKindV1::Capture)
            .map(|(_, access)| access.clone())
            .collect();
        let escapes = facts_for_function
            .iter()
            .filter(|(kind, _)| *kind == CaptureEscapeKindV1::Escape)
            .map(|(_, access)| access.clone())
            .collect();
        let coverage_complete = facts.coverage == CaptureEscapeCoverageV1::Complete
            && function.coverage.capture_escape == ControlFlowCoverageStatusV1::Available
            && function.coverage.async_suspension == ControlFlowCoverageStatusV1::Available
            && function.coverage.unknown_calls == ControlFlowCoverageStatusV1::Available
            && function.coverage.resource_cancellation == ControlFlowCoverageStatusV1::Available
            && summary.call_coverage == FunctionCallCoverageV1::Complete;
        let resume_admission = if !coverage_complete {
            ResumeCaptureAdmissionV1::RejectedUnavailableCoverage
        } else if summary.has_transitive_unknown_call.unwrap_or(true) {
            ResumeCaptureAdmissionV1::RejectedUnknownCall
        } else {
            ResumeCaptureAdmissionV1::Admissible
        };
        results.push(FunctionCaptureEscapeV1 {
            function: function.id.clone(),
            captures,
            escapes,
            resume_admission,
        });
    }
    results.sort_by(|left, right| left.function.cmp(&right.function));
    Ok(CaptureEscapeGraphV1 {
        schema_version: CAPTURE_ESCAPE_SCHEMA_VERSION,
        functions: results,
    })
}

#[cfg(test)]
mod tests {
    use crate::{
        ControlFlowAccessKindV1, ControlFlowBlockV1, ControlFlowCoverageV1, ControlFlowFunctionV1,
        ControlFlowProvenanceV1, FunctionSummaryV1,
    };

    use super::*;

    fn provenance() -> ControlFlowProvenanceV1 {
        ControlFlowProvenanceV1 {
            path: "src/App.tsx".into(),
            start: 0,
            end: 1,
            line: 1,
            column: 1,
        }
    }

    fn graph(available: bool) -> ControlFlowGraphV1 {
        let coverage = if available {
            ControlFlowCoverageStatusV1::Available
        } else {
            ControlFlowCoverageStatusV1::Unavailable
        };
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
                    reads: vec![ControlFlowAccessV1 {
                        kind: ControlFlowAccessKindV1::Storage,
                        id: "count".into(),
                    }],
                    writes: Vec::new(),
                    observable_instructions: Vec::new(),
                }],
                branch_edges: Vec::new(),
                loops: Vec::new(),
                coverage: ControlFlowCoverageV1 {
                    branch_topology: coverage,
                    definite_dataflow: coverage,
                    natural_loops: coverage,
                    exception_paths: coverage,
                    async_suspension: coverage,
                    unknown_calls: coverage,
                    capture_escape: coverage,
                    resource_cancellation: coverage,
                },
            }],
        }
    }

    fn summaries(unknown_call: bool) -> FunctionSummaryGraphV1 {
        FunctionSummaryGraphV1 {
            schema_version: 1,
            summaries: vec![FunctionSummaryV1 {
                id: "app".into(),
                module_path: "src/App.tsx".into(),
                direct_reads: Vec::new(),
                direct_writes: Vec::new(),
                direct_callees: Vec::new(),
                has_direct_unknown_call: unknown_call,
                call_coverage: FunctionCallCoverageV1::Complete,
                transitive_reads: Some(Vec::new()),
                transitive_writes: Some(Vec::new()),
                transitive_callees: Some(Vec::new()),
                has_transitive_unknown_call: Some(unknown_call),
            }],
        }
    }

    fn facts() -> CaptureEscapeFactsV1 {
        CaptureEscapeFactsV1 {
            coverage: CaptureEscapeCoverageV1::Complete,
            facts: vec![CaptureEscapeFactV1 {
                function: "app".into(),
                access: ControlFlowAccessV1 {
                    kind: ControlFlowAccessKindV1::Storage,
                    id: "count".into(),
                },
                kind: CaptureEscapeKindV1::Capture,
                provenance: provenance(),
            }],
        }
    }

    #[test]
    fn retains_explicit_capture_facts_and_admits_only_complete_known_coverage() {
        let output = build_capture_escape_graph_v1(&graph(true), &summaries(false), &facts())
            .expect("valid capture evidence");
        assert_eq!(output.schema_version, CAPTURE_ESCAPE_SCHEMA_VERSION);
        assert_eq!(output.functions[0].captures[0].id, "count");
        assert_eq!(
            output.functions[0].resume_admission,
            ResumeCaptureAdmissionV1::Admissible
        );
    }

    #[test]
    fn rejects_resume_when_cfg_coverage_or_calls_are_incomplete() {
        let unavailable = build_capture_escape_graph_v1(&graph(false), &summaries(false), &facts())
            .expect("valid capture evidence");
        assert_eq!(
            unavailable.functions[0].resume_admission,
            ResumeCaptureAdmissionV1::RejectedUnavailableCoverage
        );
        let unknown = build_capture_escape_graph_v1(&graph(true), &summaries(true), &facts())
            .expect("valid capture evidence");
        assert_eq!(
            unknown.functions[0].resume_admission,
            ResumeCaptureAdmissionV1::RejectedUnknownCall
        );
    }
}
