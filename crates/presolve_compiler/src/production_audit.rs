//! K20 static authority and refinement audit manifest.

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
        assert_eq!(RUNTIME_COMPONENT_ARTIFACT_SCHEMA_VERSION, 4);
        assert_eq!(RUNTIME_CONTEXT_ARTIFACT_SCHEMA_VERSION, 2);
        assert_eq!(RUNTIME_FORM_ARTIFACT_SCHEMA_VERSION, 1);
        assert_eq!(RUNTIME_EFFECT_ARTIFACT_SCHEMA_VERSION, 1);
        assert_eq!(RESUME_MANIFEST_SCHEMA_VERSION, 6);
        assert_eq!(PRODUCTION_RUNTIME_ARTIFACT_SCHEMA_VERSION, 1);
        assert_eq!(OPTIMIZATION_REPORT_SCHEMA_VERSION, 1);
        assert_eq!(RUNTIME_COST_REPORT_SCHEMA_VERSION, 1);
    }
}
