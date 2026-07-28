//! K20 static authority and refinement audit manifest.

use serde::{Deserialize, Serialize};

use crate::{OptimizationReportV1, ResumeBuildId, RuntimeCostReportV1};

pub const PRODUCTION_AUDIT_REPORT_SCHEMA_VERSION: u32 = 1;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ProductionRuntimeAuthority {
    pub responsibility: &'static str,
    pub authority: &'static str,
}

pub const PRODUCTION_RUNTIME_AUTHORITIES: [ProductionRuntimeAuthority; 8] = [
    ProductionRuntimeAuthority {
        responsibility: "optimization-policy",
        authority: "production_optimization::ProductionOptimizationPolicyV1",
    },
    ProductionRuntimeAuthority {
        responsibility: "program-fingerprint",
        authority:
            "production_optimization::ExecutableProgramFingerprint::for_canonical_opcode_stream",
    },
    ProductionRuntimeAuthority {
        responsibility: "reachability",
        authority: "production_reachability::build_production_reachability_graph",
    },
    ProductionRuntimeAuthority {
        responsibility: "ordinal-table",
        authority: "production_runtime_artifact::build_production_runtime_table",
    },
    ProductionRuntimeAuthority {
        responsibility: "emitter-minifier",
        authority: "production_module_emitter::emit_production_modules",
    },
    ProductionRuntimeAuthority {
        responsibility: "artifact-validator",
        authority: "production_validation::validate_production_runtime_pipeline",
    },
    ProductionRuntimeAuthority {
        responsibility: "cleanup-planner",
        authority: "production_cleanup::build_production_destroy_plan",
    },
    ProductionRuntimeAuthority {
        responsibility: "scheduler",
        authority: "production_scheduler::build_production_patch_schedule",
    },
];

pub const PRODUCTION_RUNTIME_REFINEMENT_INVARIANTS: [&str; 13] = [
    "one-authority-per-responsibility",
    "ordinal-hot-path-after-validation",
    "single-listener-and-registry-installation",
    "no-production-source-or-provenance-leakage",
    "no-dead-generated-helper",
    "bounded-instance-cache-growth",
    "production-hot-loop-specialization",
    "deterministic-content-addressed-modules",
    "validation-before-authored-execution",
    "reverse-order-exact-owner-cleanup",
    "static-budget-baselines",
    "earlier-schema-freeze",
    "development-production-observable-parity",
];

/// A versioned compiler-owned summary of the exact production reports that
/// passed the frozen production authority checks.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ProductionAuditReportV1 {
    pub schema_version: u32,
    pub build_id: ResumeBuildId,
    pub optimization_report_schema_version: u32,
    pub runtime_cost_report_schema_version: u32,
    pub runtime_table_count: u32,
    pub authority_count: u32,
    pub invariant_count: u32,
    pub checks: Vec<String>,
    pub status: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProductionAuditErrorV1 {
    pub code: &'static str,
    pub message: String,
}

impl std::fmt::Display for ProductionAuditErrorV1 {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(formatter, "{}: {}", self.code, self.message)
    }
}

impl std::error::Error for ProductionAuditErrorV1 {}

/// Validates the immutable production reports and produces the sole audit
/// artifact consumed by adapters and release tooling.
pub fn build_production_audit_report_v1(
    optimization: &OptimizationReportV1,
    runtime_cost: &RuntimeCostReportV1,
) -> Result<ProductionAuditReportV1, ProductionAuditErrorV1> {
    if optimization.schema_version != crate::OPTIMIZATION_REPORT_SCHEMA_VERSION
        || runtime_cost.schema_version != crate::RUNTIME_COST_REPORT_SCHEMA_VERSION
    {
        return Err(ProductionAuditErrorV1 {
            code: "PSV2A001_UNSUPPORTED_PRODUCTION_REPORT_SCHEMA",
            message: "production reports must use the frozen schema versions".into(),
        });
    }
    if optimization.build_id != runtime_cost.build_id {
        return Err(ProductionAuditErrorV1 {
            code: "PSV2A002_PRODUCTION_REPORT_BUILD_MISMATCH",
            message: "optimization and runtime-cost reports must describe one build".into(),
        });
    }
    if optimization.validation_status != "valid" {
        return Err(ProductionAuditErrorV1 {
            code: "PSV2A003_PRODUCTION_VALIDATION_FAILED",
            message: "the optimization report must record valid compiler validation".into(),
        });
    }
    if optimization.runtime_table_count != runtime_cost.runtime_table_count {
        return Err(ProductionAuditErrorV1 {
            code: "PSV2A004_RUNTIME_TABLE_COUNT_MISMATCH",
            message: "production reports disagree about runtime table count".into(),
        });
    }
    if runtime_cost.production_artifact_bytes > optimization.production_bytes {
        return Err(ProductionAuditErrorV1 {
            code: "PSV2A005_PRODUCTION_BYTE_ACCOUNTING_INVALID",
            message: "the production artifact cannot exceed the complete production output".into(),
        });
    }
    Ok(ProductionAuditReportV1 {
        schema_version: PRODUCTION_AUDIT_REPORT_SCHEMA_VERSION,
        build_id: optimization.build_id.clone(),
        optimization_report_schema_version: optimization.schema_version,
        runtime_cost_report_schema_version: runtime_cost.schema_version,
        runtime_table_count: optimization.runtime_table_count,
        authority_count: u32::try_from(PRODUCTION_RUNTIME_AUTHORITIES.len())
            .expect("production authority count exceeds u32"),
        invariant_count: u32::try_from(PRODUCTION_RUNTIME_REFINEMENT_INVARIANTS.len())
            .expect("production invariant count exceeds u32"),
        checks: vec![
            "report-schema".into(),
            "build-identity".into(),
            "compiler-validation".into(),
            "runtime-table-count".into(),
            "production-byte-accounting".into(),
        ],
        status: "passed".into(),
    })
}

#[must_use]
pub fn production_audit_report_json_v1(report: &ProductionAuditReportV1) -> String {
    serde_json::to_string(report).expect("production audit report should serialize") + "\n"
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        emit_production_modules, extract_production_chunk_graph, ProductionRootChunkInput,
        SharedChunkCandidatePlan, OPTIMIZATION_REPORT_SCHEMA_VERSION,
        PRODUCTION_RUNTIME_ARTIFACT_SCHEMA_VERSION, RESUME_MANIFEST_SCHEMA_VERSION,
        RUNTIME_COMPONENT_ARTIFACT_SCHEMA_VERSION, RUNTIME_CONTEXT_ARTIFACT_SCHEMA_VERSION,
        RUNTIME_COST_REPORT_SCHEMA_VERSION, RUNTIME_EFFECT_ARTIFACT_SCHEMA_VERSION,
        RUNTIME_FORM_ARTIFACT_SCHEMA_VERSION, SEMANTIC_GRAPH_SCHEMA_VERSION,
        TEMPLATE_MANIFEST_SCHEMA_VERSION,
    };
    use std::collections::BTreeSet;

    #[test]
    fn k20_has_one_named_authority_for_every_production_responsibility() {
        let responsibilities = PRODUCTION_RUNTIME_AUTHORITIES
            .iter()
            .map(|record| record.responsibility)
            .collect::<BTreeSet<_>>();
        let authorities = PRODUCTION_RUNTIME_AUTHORITIES
            .iter()
            .map(|record| record.authority)
            .collect::<BTreeSet<_>>();
        assert_eq!(responsibilities.len(), PRODUCTION_RUNTIME_AUTHORITIES.len());
        assert_eq!(authorities.len(), PRODUCTION_RUNTIME_AUTHORITIES.len());
        assert_eq!(PRODUCTION_RUNTIME_REFINEMENT_INVARIANTS.len(), 13);
    }

    #[test]
    fn k20_runtime_installers_and_production_authorities_are_single_definition_paths() {
        let runtime = include_str!("runtime_codegen.rs");
        for installer in [
            "function productionOrdinalIndexes",
            "function installResumeActivationListeners",
            "function installDelegatedEventListeners",
            "function installOrdinaryInstanceEventListeners",
            "function initializeFormsRuntime",
        ] {
            assert_eq!(runtime.matches(installer).count(), 1, "{installer}");
        }
        let sources = [
            include_str!("production_optimization.rs"),
            include_str!("production_reachability.rs"),
            include_str!("production_runtime_artifact.rs"),
            include_str!("production_module_emitter.rs"),
            include_str!("production_validation.rs"),
            include_str!("production_cleanup.rs"),
            include_str!("production_scheduler.rs"),
        ]
        .join("\n");
        for symbol in [
            "struct ProductionOptimizationPolicyV1",
            "fn for_canonical_opcode_stream",
            "fn build_production_reachability_graph",
            "fn build_production_runtime_table",
            "fn emit_production_modules",
            "fn validate_production_runtime_pipeline",
            "fn build_production_destroy_plan",
            "fn build_production_patch_schedule",
        ] {
            assert_eq!(sources.matches(symbol).count(), 1, "{symbol}");
        }
    }

    #[test]
    fn k20_generated_helpers_are_referenced_and_modules_leak_no_development_evidence() {
        let runtime = include_str!("runtime_codegen.rs");
        for line in runtime
            .lines()
            .filter(|line| line.starts_with("  function "))
        {
            let name = line
                .trim_start()
                .strip_prefix("function ")
                .and_then(|value| value.split('(').next())
                .expect("generated helper name");
            assert!(
                runtime.match_indices(name).nth(1).is_some(),
                "unreferenced generated helper {name}"
            );
        }
        let (graph, _) = extract_production_chunk_graph(
            &SharedChunkCandidatePlan {
                candidates: Vec::new(),
                rejections: Vec::new(),
            },
            &[ProductionRootChunkInput {
                activation_root_id: "audit-root".to_string(),
                root_kind: "interaction".to_string(),
                programs: Vec::new(),
            }],
        )
        .expect("audit graph");
        let layout = emit_production_modules(&graph);
        for module in std::iter::once(&layout.eager)
            .chain(layout.shared.iter())
            .chain(layout.roots.iter())
        {
            assert!(!module.source.contains("source"));
            assert!(!module.source.contains("provenance"));
            assert!(!module.source.contains("development"));
            assert!(!module.source.contains("eval("));
            assert!(!module.source.contains("Function("));
        }
    }

    #[test]
    fn k20_preserves_every_frozen_schema_and_phase_k_v1_product() {
        assert_eq!(SEMANTIC_GRAPH_SCHEMA_VERSION, 6);
        assert_eq!(TEMPLATE_MANIFEST_SCHEMA_VERSION, 5);
        assert_eq!(RUNTIME_COMPONENT_ARTIFACT_SCHEMA_VERSION, 20);
        assert_eq!(RUNTIME_CONTEXT_ARTIFACT_SCHEMA_VERSION, 2);
        assert_eq!(RUNTIME_FORM_ARTIFACT_SCHEMA_VERSION, 6);
        assert_eq!(RUNTIME_EFFECT_ARTIFACT_SCHEMA_VERSION, 7);
        assert_eq!(RESUME_MANIFEST_SCHEMA_VERSION, 7);
        assert_eq!(PRODUCTION_RUNTIME_ARTIFACT_SCHEMA_VERSION, 1);
        assert_eq!(OPTIMIZATION_REPORT_SCHEMA_VERSION, 1);
        assert_eq!(RUNTIME_COST_REPORT_SCHEMA_VERSION, 1);
    }

    #[test]
    fn v2_audit_report_is_deterministic_and_rejects_cross_build_reports() {
        let optimization = OptimizationReportV1 {
            schema_version: 1,
            build_id: ResumeBuildId::for_public_inputs("audit"),
            optimization_policy: crate::OptimizationPolicyId::production_v1(),
            dead_products_removed: 0,
            constants_pooled: 0,
            programs_deduplicated: 0,
            shared_chunks_extracted: 0,
            shared_candidates_rejected: 0,
            binding_writes_coalesced: 0,
            runtime_table_count: 1,
            development_bytes: 10,
            production_bytes: 9,
            retained_exclusions: Vec::new(),
            validation_status: "valid".into(),
        };
        let runtime_cost = RuntimeCostReportV1 {
            schema_version: 1,
            build_id: optimization.build_id.clone(),
            bootstrap_module_bytes: 1,
            production_artifact_bytes: 2,
            eager_program_count: 0,
            lazy_root_chunk_count: 0,
            shared_chunk_count: 0,
            max_lazy_dependency_depth: 0,
            runtime_table_count: 1,
            runtime_record_count: 0,
            estimated_boot_decode_units: 0,
            estimated_boot_validation_units: 0,
            estimated_cold_init_operation_count: 0,
            estimated_resume_restore_operation_count: 0,
            max_action_batch_operation_count: 0,
            max_scheduler_batch_width: 0,
            max_dom_patch_count_per_action: 0,
            retained_slot_count: 0,
        };
        let first = build_production_audit_report_v1(&optimization, &runtime_cost).unwrap();
        assert_eq!(first.status, "passed");
        assert_eq!(
            production_audit_report_json_v1(&first),
            production_audit_report_json_v1(&first)
        );
        let mut different_build = runtime_cost;
        different_build.build_id = ResumeBuildId::for_public_inputs("other");
        assert_eq!(
            build_production_audit_report_v1(&optimization, &different_build)
                .unwrap_err()
                .code,
            "PSV2A002_PRODUCTION_REPORT_BUILD_MISMATCH"
        );
    }
}
