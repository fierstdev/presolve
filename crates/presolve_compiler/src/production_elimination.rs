//! K3 immutable production-only dead executable-record elimination.

use std::collections::BTreeSet;

use crate::{OptimizationDecisionId, ProductionReachabilityGraph, ProductionUnreachableRecord};

/// One exact compiler-owned executable record eligible for a production
/// projection. The record is supplied by an existing canonical product; K3
/// never derives it from source, DOM, or runtime registration order.
#[derive(Clone, Debug, Eq, PartialEq)]
#[allow(clippy::struct_excessive_bools)]
pub struct ProductionExecutionRecord {
    pub subject_id: String,
    pub program_id: String,
    pub referenced_program_ids: Vec<String>,
    pub required_for_validation: bool,
    pub required_for_inspection: bool,
    pub required_for_failure: bool,
    pub required_for_destruction: bool,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DeadProductEliminationReason {
    UnreachableExecutableRecord,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct OptimizationDecision {
    pub id: OptimizationDecisionId,
    pub subject_id: String,
    pub reason: DeadProductEliminationReason,
    pub root_closure_evidence: Vec<String>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProductionArtifactProjection {
    pub retained_records: Vec<ProductionExecutionRecord>,
    pub eliminated_records: Vec<ProductionExecutionRecord>,
    pub validation_blocks: Vec<String>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DeadProductEliminationReport {
    pub decisions: Vec<OptimizationDecision>,
    pub unreachable: Vec<ProductionUnreachableRecord>,
}

/// Projects immutable canonical execution records for production only.
/// Development products are borrowed as input and are not mutated.
///
/// # Panics
///
/// Panics only when a caller supplies an empty canonical execution subject.
#[must_use]
pub fn eliminate_unreachable_production_records(
    records: &[ProductionExecutionRecord],
    reachability: &ProductionReachabilityGraph,
) -> (ProductionArtifactProjection, DeadProductEliminationReport) {
    let reachable = reachability
        .reachable_programs
        .iter()
        .cloned()
        .collect::<BTreeSet<_>>();
    let mut retained_records = Vec::new();
    let mut eliminated_records = Vec::new();
    let mut decisions = Vec::new();
    let mut unreachable = Vec::new();

    for record in records {
        let protected = record.required_for_validation
            || record.required_for_inspection
            || record.required_for_failure
            || record.required_for_destruction;
        if protected || reachable.contains(&record.program_id) {
            retained_records.push(record.clone());
        } else {
            let evidence = reachability
                .roots
                .iter()
                .map(|root| root.subject_id.clone())
                .collect::<Vec<_>>();
            decisions.push(OptimizationDecision {
                id: OptimizationDecisionId::for_canonical_subject(
                    "dead-product-elimination",
                    &record.subject_id,
                )
                .expect("canonical execution subject"),
                subject_id: record.subject_id.clone(),
                reason: DeadProductEliminationReason::UnreachableExecutableRecord,
                root_closure_evidence: evidence,
            });
            unreachable.push(ProductionUnreachableRecord {
                subject_id: record.subject_id.clone(),
                reason: "absent from the closed production root set".to_string(),
            });
            eliminated_records.push(record.clone());
        }
    }

    retained_records.sort_by(|left, right| left.subject_id.cmp(&right.subject_id));
    eliminated_records.sort_by(|left, right| left.subject_id.cmp(&right.subject_id));
    decisions.sort_by(|left, right| left.id.cmp(&right.id));
    unreachable.sort_by(|left, right| left.subject_id.cmp(&right.subject_id));

    let retained_programs = retained_records
        .iter()
        .map(|record| record.program_id.as_str())
        .collect::<BTreeSet<_>>();
    let mut validation_blocks = retained_records
        .iter()
        .flat_map(|record| {
            record
                .referenced_program_ids
                .iter()
                .filter(|reference| !retained_programs.contains(reference.as_str()))
                .map(|reference| {
                    format!(
                        "{} references removed program {reference}",
                        record.subject_id
                    )
                })
        })
        .collect::<Vec<_>>();
    validation_blocks.sort();
    validation_blocks.dedup();

    (
        ProductionArtifactProjection {
            retained_records,
            eliminated_records,
            validation_blocks,
        },
        DeadProductEliminationReport {
            decisions,
            unreachable,
        },
    )
}

#[cfg(test)]
mod tests {
    use super::{eliminate_unreachable_production_records, ProductionExecutionRecord};
    use crate::{
        ProductionExecutableRoot, ProductionReachabilityGraph, ProductionReachabilityReason,
    };

    #[test]
    fn k3_eliminates_only_unreachable_unprotected_records_deterministically() {
        let graph = ProductionReachabilityGraph {
            roots: vec![ProductionExecutableRoot {
                subject_id: "root".to_string(),
                reason: ProductionReachabilityReason::ColdBoot,
            }],
            edges: Vec::new(),
            reachable_programs: vec!["reachable".to_string()],
            unreachable: Vec::new(),
            blocks: Vec::new(),
        };
        let records = vec![
            record("dead", "dead-program", false),
            record("reachable", "reachable", false),
            record("reset", "reset-program", true),
        ];
        let (projection, report) = eliminate_unreachable_production_records(&records, &graph);
        assert_eq!(records[0].program_id, "dead-program");
        assert_eq!(projection.eliminated_records.len(), 1);
        assert_eq!(projection.eliminated_records[0].subject_id, "dead");
        assert_eq!(projection.retained_records.len(), 2);
        assert_eq!(report.decisions.len(), 1);
        assert_eq!(report.unreachable[0].subject_id, "dead");
    }

    #[test]
    fn k3_retains_validation_records_and_reports_dangling_projection_references() {
        let graph = ProductionReachabilityGraph {
            roots: Vec::new(),
            edges: Vec::new(),
            reachable_programs: Vec::new(),
            unreachable: Vec::new(),
            blocks: Vec::new(),
        };
        let mut retained = record("validator", "validator-program", true);
        retained
            .referenced_program_ids
            .push("removed-program".to_string());
        let (projection, report) = eliminate_unreachable_production_records(
            &[retained, record("dead", "removed-program", false)],
            &graph,
        );
        assert_eq!(projection.retained_records.len(), 1);
        assert_eq!(projection.eliminated_records.len(), 1);
        assert_eq!(
            projection.validation_blocks,
            vec!["validator references removed program removed-program"]
        );
        assert_eq!(report.decisions.len(), 1);
    }

    fn record(
        subject_id: &str,
        program_id: &str,
        required_for_validation: bool,
    ) -> ProductionExecutionRecord {
        ProductionExecutionRecord {
            subject_id: subject_id.to_string(),
            program_id: program_id.to_string(),
            referenced_program_ids: Vec::new(),
            required_for_validation,
            required_for_inspection: false,
            required_for_failure: false,
            required_for_destruction: false,
        }
    }
}
