//! K18 canonical public diagnostics for immutable production products.

use std::collections::{BTreeMap, BTreeSet};

use serde::Serialize;

use crate::SourceProvenance;

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum ProductionDiagnosticKind {
    InvalidOptimizationRoot,
    InvalidProgramFingerprint,
    UnsafeProgramDeduplication,
    InvalidConstantPoolEntry,
    InvalidSharedChunkCandidate,
    ProductionChunkCycle,
    InvalidRuntimeOrdinalTable,
    ProductionArtifactMismatch,
    UnsafeBindingWriteCoalescing,
    MissingRuntimeCleanup,
    InvalidRuntimeCleanupOrder,
    DetachedActivationTarget,
    InvalidProductionFailureRecord,
    OptimizationReportMismatch,
    ProductionBudgetRegression,
    ProductionDeterminismFailure,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ProductionDiagnosticContract {
    pub kind: ProductionDiagnosticKind,
    pub code: &'static str,
    pub name: &'static str,
    pub message: &'static str,
}

pub const PRODUCTION_DIAGNOSTIC_CATALOG: [ProductionDiagnosticContract; 16] = [
    contract(ProductionDiagnosticKind::InvalidOptimizationRoot, "PSC1112", "InvalidOptimizationRoot", "A production executable root references no exact valid canonical program or owner."),
    contract(ProductionDiagnosticKind::InvalidProgramFingerprint, "PSC1113", "InvalidProgramFingerprint", "A fingerprint disagrees with its canonical opcode stream or collides with non-identical bytes."),
    contract(ProductionDiagnosticKind::UnsafeProgramDeduplication, "PSC1114", "UnsafeProgramDeduplication", "A program merge crosses an identity, side-effect, schedule, protocol, or instance-ownership boundary."),
    contract(ProductionDiagnosticKind::InvalidConstantPoolEntry, "PSC1115", "InvalidConstantPoolEntry", "A constant pool entry has incompatible bytes, observable identity, an invalid consumer, or noncanonical order."),
    contract(ProductionDiagnosticKind::InvalidSharedChunkCandidate, "PSC1116", "InvalidSharedChunkCandidate", "A shared candidate violates root-count, byte, savings, root-specificity, or registration-only requirements."),
    contract(ProductionDiagnosticKind::ProductionChunkCycle, "PSC1117", "ProductionChunkCycle", "Production chunk topology contains a cycle or dependency depth greater than one."),
    contract(ProductionDiagnosticKind::InvalidRuntimeOrdinalTable, "PSC1118", "InvalidRuntimeOrdinalTable", "A runtime table has invalid ordinals, width, identities, ordering, or checksum."),
    contract(ProductionDiagnosticKind::ProductionArtifactMismatch, "PSC1119", "ProductionArtifactMismatch", "Packed production records disagree with frozen canonical artifacts, versions, build identity, ownership, or ordering."),
    contract(ProductionDiagnosticKind::UnsafeBindingWriteCoalescing, "PSC1120", "UnsafeBindingWriteCoalescing", "A removed DOM write lacks proof that no observable operation, read, or boundary depends on it."),
    contract(ProductionDiagnosticKind::MissingRuntimeCleanup, "PSC1121", "MissingRuntimeCleanup", "A destroyable runtime owner lacks exact cleanup coverage for compiler-owned registrations or slots."),
    contract(ProductionDiagnosticKind::InvalidRuntimeCleanupOrder, "PSC1122", "InvalidRuntimeCleanupOrder", "Cleanup can expose stale dispatch, remove required state early, or violate structural ordering."),
    contract(ProductionDiagnosticKind::DetachedActivationTarget, "PSC1123", "DetachedActivationTarget", "An imported or queued activation can execute against a destroyed or detached target."),
    contract(ProductionDiagnosticKind::InvalidProductionFailureRecord, "PSC1124", "InvalidProductionFailureRecord", "A compact production failure cannot be resolved safely or leaks forbidden provenance."),
    contract(ProductionDiagnosticKind::OptimizationReportMismatch, "PSC1125", "OptimizationReportMismatch", "Reported decisions, sizes, counts, or savings disagree with immutable optimized products."),
    contract(ProductionDiagnosticKind::ProductionBudgetRegression, "PSC1126", "ProductionBudgetRegression", "A normative static size, count, or operation budget was exceeded without an authorized revision."),
    contract(ProductionDiagnosticKind::ProductionDeterminismFailure, "PSC1127", "ProductionDeterminismFailure", "Equivalent builds produced different optimized products, artifacts, reports, ordinals, modules, names, or hashes."),
];

const fn contract(
    kind: ProductionDiagnosticKind,
    code: &'static str,
    name: &'static str,
    message: &'static str,
) -> ProductionDiagnosticContract {
    ProductionDiagnosticContract {
        kind,
        code,
        name,
        message,
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProductionDiagnosticFact {
    pub kind: ProductionDiagnosticKind,
    pub actionable: bool,
    pub primary_identity: Option<String>,
    pub primary_provenance: Option<SourceProvenance>,
    pub secondary_evidence: Vec<String>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct ProductionDiagnosticProvenance {
    pub path: String,
    pub line: usize,
    pub column: usize,
    pub start: usize,
    pub end: usize,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct ProductionProjectedDiagnostic {
    pub code: &'static str,
    pub name: &'static str,
    pub message: &'static str,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub primary_identity: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub primary_provenance: Option<ProductionDiagnosticProvenance>,
    pub secondary_evidence: Vec<String>,
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
struct DiagnosticKey {
    kind: ProductionDiagnosticKind,
    identity: Option<String>,
    path: Option<String>,
    start: Option<usize>,
    end: Option<usize>,
}

/// Projects only explicit immutable failures. Missing identities/provenance stay
/// absent rather than being reconstructed from names or source text.
#[must_use]
pub fn project_production_diagnostics(
    facts: &[ProductionDiagnosticFact],
) -> Vec<ProductionProjectedDiagnostic> {
    let mut grouped =
        BTreeMap::<DiagnosticKey, (Option<ProductionDiagnosticProvenance>, BTreeSet<String>)>::new(
        );
    for fact in facts.iter().filter(|fact| fact.actionable) {
        let identity = fact
            .primary_identity
            .as_ref()
            .filter(|identity| !identity.trim().is_empty())
            .cloned();
        let provenance = fact
            .primary_provenance
            .as_ref()
            .and_then(project_provenance);
        let key = DiagnosticKey {
            kind: fact.kind,
            identity,
            path: provenance.as_ref().map(|value| value.path.clone()),
            start: provenance.as_ref().map(|value| value.start),
            end: provenance.as_ref().map(|value| value.end),
        };
        grouped
            .entry(key)
            .or_insert_with(|| (provenance, BTreeSet::new()))
            .1
            .extend(
                fact.secondary_evidence
                    .iter()
                    .filter(|value| !value.is_empty())
                    .cloned(),
            );
    }
    grouped
        .into_iter()
        .map(|(key, (provenance, evidence))| {
            let contract = &PRODUCTION_DIAGNOSTIC_CATALOG[key.kind as usize];
            ProductionProjectedDiagnostic {
                code: contract.code,
                name: contract.name,
                message: contract.message,
                primary_identity: key.identity,
                primary_provenance: provenance,
                secondary_evidence: evidence.into_iter().collect(),
            }
        })
        .collect()
}

fn project_provenance(value: &SourceProvenance) -> Option<ProductionDiagnosticProvenance> {
    let path = value.path.to_string_lossy().into_owned();
    (!path.is_empty()
        && value.span.line > 0
        && value.span.column > 0
        && value.span.start <= value.span.end)
        .then_some(ProductionDiagnosticProvenance {
            path,
            line: value.span.line,
            column: value.span.column,
            start: value.span.start,
            end: value.span.end,
        })
}

#[cfg(test)]
mod tests {
    use super::*;
    use ezc_parser::SourceSpan;
    use std::path::Path;

    fn fact(contract: ProductionDiagnosticContract, actionable: bool) -> ProductionDiagnosticFact {
        ProductionDiagnosticFact {
            kind: contract.kind,
            actionable,
            primary_identity: Some(format!("subject:{}", contract.code)),
            primary_provenance: Some(SourceProvenance::new(
                Path::new("src/App.tsx"),
                SourceSpan {
                    start: contract.kind as usize,
                    end: contract.kind as usize + 1,
                    line: 1,
                    column: contract.kind as usize + 1,
                },
            )),
            secondary_evidence: vec!["z".to_string(), "a".to_string(), "a".to_string()],
        }
    }

    #[test]
    fn k18_projects_one_positive_and_one_negative_case_per_code_in_catalog_order() {
        let mut facts = PRODUCTION_DIAGNOSTIC_CATALOG
            .iter()
            .rev()
            .flat_map(|contract| [fact(*contract, true), fact(*contract, false)])
            .collect::<Vec<_>>();
        facts.reverse();
        let diagnostics = project_production_diagnostics(&facts);
        assert_eq!(diagnostics.len(), 16);
        assert_eq!(diagnostics.first().map(|value| value.code), Some("PSC1112"));
        assert_eq!(diagnostics.last().map(|value| value.code), Some("PSC1127"));
        assert!(diagnostics
            .iter()
            .all(|diagnostic| diagnostic.secondary_evidence == ["a", "z"]));
    }

    #[test]
    fn k18_malformed_identity_and_span_cases_per_code_are_not_fabricated_and_deduplicate() {
        let malformed = PRODUCTION_DIAGNOSTIC_CATALOG
            .iter()
            .flat_map(|contract| {
                let mut value = fact(*contract, true);
                value.primary_identity = Some(" ".to_string());
                value.primary_provenance = Some(SourceProvenance::new(
                    Path::new("src/App.tsx"),
                    SourceSpan {
                        start: 9,
                        end: 2,
                        line: 1,
                        column: 1,
                    },
                ));
                [value.clone(), value]
            })
            .collect::<Vec<_>>();
        let diagnostics = project_production_diagnostics(&malformed);
        assert_eq!(diagnostics.len(), 16);
        assert!(diagnostics.iter().all(|diagnostic| {
            diagnostic.primary_identity.is_none() && diagnostic.primary_provenance.is_none()
        }));
    }
}
