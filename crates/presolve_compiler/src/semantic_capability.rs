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
                "serializable_state_replacement",
                SemanticCapabilityClass::Bounded,
                "field = state(recordOrArray); @action() { this.field = serializableRecordOrArray; }",
                "component-instance State storage, Action plan, runtime field write, and resume codec",
                "recursively serializable record or array literal at the whole-field boundary",
                "Action-owned field replacement; no nested mutable alias or untracked write",
                "existing State resume codec carries the complete replacement value",
                "existing action manifest assign operand and State runtime slot",
                "runtime_browser::serializable_record_state_replacement_executes_from_compiler_generated_runtime",
            ),
            admitted(
                "static_action_parameters",
                SemanticCapabilityClass::Bounded,
                "@action() method(value: primitive) { this.field = value; }; onClick={() => this.method(serializableLiteral)}",
                "Action parameter, State write, event binding, ordinary-instance plan, and runtime batch",
                "one or more explicitly typed primitive parameters supplied by exact serializable callback literals",
                "the compiler carries static arguments from the event declaration into the one completed Action batch; no DOM payload or closure capture is read",
                "arguments are static manifest data; the resulting State replacement uses the existing State resume codec",
                "template-manifest schema v5 and component-runtime schema v4 ordinary-event argument records with assign_parameter action operation",
                "runtime_browser::static_callback_argument_updates_state_through_compiler_action_parameter",
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
                "keyed_structural_list",
                SemanticCapabilityClass::Bounded,
                "compiler-supported iterable.map((item) => <element key={item.id} />)",
                "template graph, compiler-issued keyed item identity, and structural runtime plan",
                "explicit unique primitive key and supported item template grammar",
                "compiler-owned iterable dependency and keyed instance lifecycle",
                "keyed structural identity is carried by compiler artifacts and resume anchors",
                "template/component runtime structural list records",
                "runtime_browser::keyed_lists_reconcile_in_a_real_browser",
            ),
            admitted(
                "jsx_html_attribute_aliases",
                SemanticCapabilityClass::Bounded,
                "className and htmlFor on compiler-supported intrinsic elements",
                "template lowering before semantic analysis, manifests, static HTML, and runtime bindings",
                "exact className or htmlFor spelling with a static or compiler-supported string binding",
                "compiler-derived binding dependency after normalization to class or for",
                "uses the existing attribute-binding State resume path",
                "canonical HTML class/for template and runtime attribute records",
                "template_graph::tests::normalizes_jsx_html_attribute_aliases",
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
            admitted(
                "module_bindings",
                SemanticCapabilityClass::Bounded,
                "local/relative import, export, re-export, namespace import, and local type alias",
                "CompilationUnit, ModuleGraph, BindingTable, and local type bindings",
                "local and relative binding forms retained by the parser",
                "compiler-resolved module edges and named bindings",
                "module bindings carry no independent runtime state",
                "canonical binding-table product",
                "binding_table::tests::resolves_relative_named_default_and_namespace_imports",
            ),
            deferred(
                "advanced_types",
                SemanticCapabilityClass::Unsupported,
                "generic utilities, structural imported types, and checker-dependent TypeScript semantics",
                "N1 advanced type products",
                "no versioned compiler-owned TypeScript front-end contract yet",
                "cannot derive canonical structural type bindings",
                "no resume policy before admission",
                "no artifact representation",
                "N1-B must define advanced type semantics",
            ),
            admitted(
                "semantic_package_bindings",
                SemanticCapabilityClass::Bounded,
                "named or default external import with a caller-supplied semantic package contract",
                "SemanticPackageContract and BindingTable",
                "validated schema, SHA-256 integrity, declared named/default export",
                "declared binding only; executable use awaits its semantic-kind admission",
                "binding itself owns no independently resumable state",
                "canonical binding-table product records package/version/integrity/export",
                "binding_table::tests::resolves_external_imports_only_through_semantic_package_contracts",
            ),
            admitted(
                "semantic_package_pure_identity",
                SemanticCapabilityClass::Bounded,
                "direct call to a declared pure package export with pure_operation: identity in @computed()",
                "SemanticPackageContract, ExpressionGraph, canonical IR, and runtime-computed artifact",
                "one declared argument; result inherits the argument semantic type and serialization boundary",
                "inherits the argument's compiler-derived reactive dependencies; no package code executes",
                "input-only; no package-owned state or activation record",
                "runtime-computed schema v4 pure-package-call instruction with package provenance",
                "runtime_browser::pure_package_contracts_execute_only_the_compiler_lowered_operation_in_a_real_browser",
            ),
            admitted(
                "template_interpolation",
                SemanticCapabilityClass::Bounded,
                "untagged template literal in a supported @computed() getter",
                "parser, ExpressionGraph, canonical IR, and runtime-computed artifact",
                "cooked static segments and compiler-supported interpolation expressions",
                "compiler-derived union of every interpolation dependency",
                "serializable string result; no independent resume record",
                "runtime-computed schema v5 template instruction, retained in schema v6",
                "runtime_browser::template_interpolations_execute_from_compiler_generated_runtime_programs",
            ),
            admitted(
                "static_index_access",
                SemanticCapabilityClass::Bounded,
                "this.value[non-negative integer literal] or this.value[\"string literal\"] in a supported @computed() getter",
                "parser, ExpressionGraph, canonical IR, and runtime-computed artifact",
                "tuple, array, or object read with a literal string or non-negative integer index",
                "compiler-derived dependency on the indexed object only; the literal index has no reactive dependency",
                "serializable result when the selected value is serializable; no independent resume record",
                "runtime-computed schema v6 get-index instruction, retained in schema v8",
                "runtime_browser::static_index_accesses_execute_from_compiler_generated_runtime_programs",
            ),
            admitted(
                "boolean_computed_conditional",
                SemanticCapabilityClass::Bounded,
                "boolean condition ? compiler-supported consequent : compiler-supported alternate in @computed()",
                "parser, ExpressionGraph, canonical IR, and runtime-computed artifact",
                "boolean condition with serializable compiler-supported branch values",
                "compiler-derived union of condition and both branch dependencies",
                "serializable selected result; no independent resume record",
                "runtime-computed schema v7 select instruction, retained in schema v8",
                "runtime_browser::boolean_conditional_computed_values_execute_from_compiler_generated_runtime_programs",
            ),
            admitted(
                "builtin_math_abs",
                SemanticCapabilityClass::Bounded,
                "Math.abs(value) in a supported @computed() getter",
                "compiler-registered BuiltinPureOperation, canonical IR, and runtime-computed artifact",
                "exactly one compiler-supported numeric operand",
                "inherits the operand's compiler-derived dependency set",
                "serializable numeric result; no package or independent resume record",
                "runtime-computed schema v9 unary abs operation",
                "runtime_browser::registered_math_abs_executes_from_compiler_generated_runtime_programs",
            ),
            admitted(
                "builtin_math_min_max",
                SemanticCapabilityClass::Bounded,
                "Math.min(left, right) or Math.max(left, right) in a supported @computed() getter",
                "compiler-registered BuiltinPureOperation, canonical binary IR, and runtime-computed artifact",
                "exactly two compiler-supported numeric operands",
                "union of the compiler-derived dependencies of both operands",
                "serializable numeric result; no package or independent resume record",
                "runtime-computed schema v10 binary min/max operations",
                "runtime_browser::registered_math_min_max_execute_from_compiler_generated_runtime_programs",
            ),
            deferred(
                "semantic_package_exports",
                SemanticCapabilityClass::Unsupported,
                "all other pure operations and capability, resource, codec, or component package exports",
                "N1-A2 semantic-kind lowering products",
                "declared export metadata is not yet an executable compiler semantic",
                "cannot derive reactive, capability, resource, codec, or component behavior from a binding alone",
                "no export-kind resume policy has been admitted",
                "no package-kind IR or artifact provenance has been admitted",
                "N1-A2 must admit each package kind through full-path lowering",
            ),
            deferred(
                "resources",
                SemanticCapabilityClass::Unsupported,
                "compiler-owned Resource declaration",
                "N6-A declaration/lifecycle products and N6-B registered endpoint contract",
                "no compiler-recognized Resource source form or endpoint selection yet",
                "no executable async dependency or cancellation plan yet",
                "no Resource resume policy",
                "no Resource runtime artifact",
                "N6-B deliberately stops before source lowering and runtime admission",
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
                "serializable_state_replacement",
                "static_action_parameters",
                "action",
                "computed",
                "effect",
                "context",
                "slot",
                "keyed_structural_list",
                "jsx_html_attribute_aliases",
                "form",
                "module_bindings",
                "advanced_types",
                "semantic_package_bindings",
                "semantic_package_pure_identity",
                "template_interpolation",
                "static_index_access",
                "boolean_computed_conditional",
                "builtin_math_abs",
                "builtin_math_min_max",
                "semantic_package_exports",
                "resources",
                "opaque_typescript"
            ]
        );
        assert!(registry
            .capabilities
            .iter()
            .filter(|capability| capability.status == SemanticCapabilityStatus::Deferred)
            .all(|capability| capability.rejection_reason.is_some()));
        assert!(semantic_capability_registry_json().contains("\"semantic_package_bindings\""));
        assert!(semantic_capability_registry_json().contains("\"static_index_access\""));
    }
}
