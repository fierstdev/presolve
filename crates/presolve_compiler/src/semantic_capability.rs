use serde::Serialize;

pub const SEMANTIC_CAPABILITY_REGISTRY_SCHEMA_VERSION: u32 = 1;

/// Compiler-owned classification of an authoring capability.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum SemanticCapabilityClass {
    Native,
    Bounded,
    Opaque,
    Unsupported,
}

/// Whether the compiler currently admits the named capability.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum SemanticCapabilityStatus {
    Admitted,
    Deferred,
}

/// One public, compiler-owned capability admission record.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct SemanticCapability {
    pub id: &'static str,
    pub class: SemanticCapabilityClass,
    pub status: SemanticCapabilityStatus,
    pub source_form: &'static str,
    pub semantic_owner: &'static str,
    pub type_rule: &'static str,
    pub dependency_rule: &'static str,
    pub resume_policy: &'static str,
    pub artifact_impact: &'static str,
    pub proof_fixture: &'static str,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rejection_reason: Option<&'static str>,
}

/// Versioned public registry used to admit future compiler language families.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct SemanticCapabilityRegistry {
    pub schema_version: u32,
    pub capabilities: Vec<SemanticCapability>,
}

#[must_use]
pub fn build_semantic_capability_registry() -> SemanticCapabilityRegistry {
    SemanticCapabilityRegistry {
        schema_version: SEMANTIC_CAPABILITY_REGISTRY_SCHEMA_VERSION,
        capabilities: vec![
            admitted(
                "component",
                SemanticCapabilityClass::Native,
                "@component(\"tag\") class Name",
                "component definition and instance plan",
                "compiler-recognized component declaration",
                "compiler-owned instance identity and composition",
                "resumable through component artifacts",
                "HTML, component runtime, resume artifacts",
                "fixtures/0062-component-declarations/input/ValidComponents.tsx",
            ),
            admitted(
                "state",
                SemanticCapabilityClass::Bounded,
                "field = state(serializableInitializer)",
                "component-instance State storage",
                "compiler-supported serializable initializer",
                "compiler-derived reads and Action writes",
                "resumable when the value schema is supported",
                "runtime State slots and bindings",
                "examples/counter/src/Counter.tsx",
            ),
            admitted(
                "action",
                SemanticCapabilityClass::Bounded,
                "@action() method()",
                "completed Action batch",
                "compiler-supported parameters and body",
                "compiler-derived State writes and event activation",
                "lazy activation and resume through emitted action records",
                "runtime action chunks",
                "examples/counter/src/Counter.tsx",
            ),
            admitted(
                "computed",
                SemanticCapabilityClass::Bounded,
                "@computed() get value()",
                "compiler-owned Computed value",
                "getter-only supported expression subset",
                "compiler-derived dependency graph",
                "recomputed from resumed State; no authored cache",
                "Computed IR and runtime artifact",
                "fixtures/0047-computed-diamond/input/ComputedDiamond.tsx",
            ),
            admitted(
                "effect",
                SemanticCapabilityClass::Bounded,
                "@effect() method()",
                "terminal capability program",
                "compiler-known capability arguments only",
                "compiler-derived affected Action batches",
                "not independently resumed; runs by compiler policy",
                "effect IR and runtime capability artifact",
                "fixtures/0053-effect-initial-runtime/input/InitialEffectRuntime.tsx",
            ),
            admitted(
                "context",
                SemanticCapabilityClass::Bounded,
                "@context() static value; @provide(\"Owner.value\"); @consume(\"Owner.value\")",
                "Context declaration, provider, and consumer identities",
                "declared structural type boundary",
                "compiler-owned visibility and provider selection",
                "compiler-owned Context slots and resume records",
                "Context IR and runtime artifact",
                "fixtures/0059-context-runtime-matrix/input/ContextRuntimeMatrix.tsx",
            ),
            admitted(
                "slot",
                SemanticCapabilityClass::Bounded,
                "@slot() content; <slot />",
                "Slot declaration and caller-owned content",
                "compiler-owned SlotContent marker",
                "compiler-owned lexical owner and binding",
                "resumable through Slot-binding records",
                "component runtime Slot programs",
                "fixtures/0062-component-declarations/input/ValidComponents.tsx",
            ),
            admitted(
                "form",
                SemanticCapabilityClass::Bounded,
                "@form(); @field(\"form\"); @submit(\"form\")",
                "Form, Field, validation, and submission identities",
                "compiler-supported control and serializable field types",
                "compiler-owned Form plans and explicit host markers",
                "resumable through Form schema and records",
                "Forms runtime artifact",
                "framework/tests/forms-resume-types/src/ResumeForms.tsx",
            ),
            deferred(
                "module_types",
                SemanticCapabilityClass::Unsupported,
                "typed imports, exports, aliases, and generic utilities",
                "N1 module/type binding products",
                "no compiler-owned module/type capability contract yet",
                "cannot derive canonical bindings",
                "no resume policy before admission",
                "no artifact representation",
                "N1 must define module/type identities",
            ),
            deferred(
                "semantic_packages",
                SemanticCapabilityClass::Unsupported,
                "third-party import resolved through a semantic package contract",
                "N1-A package contract and explicit-resolution products",
                "no integrity-checked package contract schema yet",
                "cannot derive package export behavior",
                "no resume policy before admission",
                "no package artifact provenance",
                "N1-A must define package contracts",
            ),
            deferred(
                "resources",
                SemanticCapabilityClass::Unsupported,
                "compiler-owned Resource declaration",
                "N6 Resource identity and lifecycle plan",
                "no loading/success/error type contract yet",
                "cannot derive async dependencies or cancellation",
                "no Resource resume policy",
                "no Resource runtime artifact",
                "N6 must define Resource semantics",
            ),
            deferred(
                "opaque_typescript",
                SemanticCapabilityClass::Opaque,
                "explicit opaque boundary",
                "N9 compiler-recorded opaque activation boundary",
                "no opaque input/output contract yet",
                "opaque code cannot participate in inferred dependencies",
                "opaque resume is unavailable by default",
                "no opaque artifact contract",
                "N9 must define opaque isolation",
            ),
        ],
    }
}

#[must_use]
pub fn semantic_capability_registry_json() -> String {
    serde_json::to_string_pretty(&build_semantic_capability_registry())
        .expect("semantic capability registry should serialize")
        + "\n"
}

fn admitted(
    id: &'static str,
    class: SemanticCapabilityClass,
    source_form: &'static str,
    semantic_owner: &'static str,
    type_rule: &'static str,
    dependency_rule: &'static str,
    resume_policy: &'static str,
    artifact_impact: &'static str,
    proof_fixture: &'static str,
) -> SemanticCapability {
    SemanticCapability {
        id,
        class,
        status: SemanticCapabilityStatus::Admitted,
        source_form,
        semantic_owner,
        type_rule,
        dependency_rule,
        resume_policy,
        artifact_impact,
        proof_fixture,
        rejection_reason: None,
    }
}

fn deferred(
    id: &'static str,
    class: SemanticCapabilityClass,
    source_form: &'static str,
    semantic_owner: &'static str,
    type_rule: &'static str,
    dependency_rule: &'static str,
    resume_policy: &'static str,
    artifact_impact: &'static str,
    rejection_reason: &'static str,
) -> SemanticCapability {
    SemanticCapability {
        id,
        class,
        status: SemanticCapabilityStatus::Deferred,
        source_form,
        semantic_owner,
        type_rule,
        dependency_rule,
        resume_policy,
        artifact_impact,
        proof_fixture: "",
        rejection_reason: Some(rejection_reason),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn registry_is_versioned_stable_and_explains_deferred_families() {
        let registry = build_semantic_capability_registry();
        assert_eq!(registry.schema_version, 1);
        assert_eq!(
            registry
                .capabilities
                .iter()
                .map(|capability| capability.id)
                .collect::<Vec<_>>(),
            vec![
                "component",
                "state",
                "action",
                "computed",
                "effect",
                "context",
                "slot",
                "form",
                "module_types",
                "semantic_packages",
                "resources",
                "opaque_typescript"
            ]
        );
        assert!(registry
            .capabilities
            .iter()
            .filter(|capability| capability.status == SemanticCapabilityStatus::Deferred)
            .all(|capability| capability.rejection_reason.is_some()));
        assert!(semantic_capability_registry_json().contains("\"semantic_packages\""));
    }
}
