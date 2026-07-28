use std::fmt::Write as _;

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
                "export class Name extends Component",
                "component definition and instance plan",
                "compiler-recognized component declaration",
                "compiler-owned instance identity and composition",
                "resumable through component artifacts",
                "HTML, component runtime, resume artifacts",
                "fixtures/0062-component-declarations/input/ValidComponents.tsx",
            ),
            admitted(
                "component_invocation",
                SemanticCapabilityClass::Bounded,
                "resolved JSX component invocation with compiler-supported inputs and Slots",
                "component invocation, composition, instance, ownership, and lifecycle products",
                "statically resolved component target with compiler-supported composition grammar",
                "compiler-owned invocation topology, caller/callee ownership, and instance identity",
                "component instances, Slots, and Context bindings use existing resume identities",
                "component runtime and resume artifacts",
                "component_fixtures::component_composition_fixture_covers_topology_slots_caller_ownership_and_blocking",
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
                "method = action((value: primitive) => { this.field = value; }); onClick={() => this.method(serializableLiteral)}",
                "Action parameter, State write, event binding, ordinary-instance plan, and runtime batch",
                "one or more explicitly typed primitive parameters supplied by exact serializable callback literals",
                "the compiler carries static arguments from the event declaration into the one completed Action batch; no DOM payload or closure capture is read",
                "arguments are static manifest data; the resulting State replacement uses the existing State resume codec",
                "template-manifest schema v5 and component-runtime schema v4 ordinary-event argument records with assign_parameter action operation",
                "runtime_browser::static_callback_argument_updates_state_through_compiler_action_parameter",
            ),
            admitted(
                "action_parameter_state_types",
                SemanticCapabilityClass::Bounded,
                "method = action((value: primitive) => { this.stateField = value; })",
                "Action parameter and component-instance State type boundary",
                "parameter and State must have the same compiler-known primitive kind, declared or inferred from the State initializer",
                "type compatibility is checked before Action/runtime lowering; it adds no dynamic dependency",
                "the existing compatible State slot and resume codec remain authoritative",
                "canonical PSC1044 diagnostic; no new runtime or artifact field",
                "tests::component_graph_rejects_action_parameter_state_type_mismatch",
            ),
            admitted(
                "serializable_action_locals",
                SemanticCapabilityClass::Bounded,
                "method = action(() => { const local = serializableLiteral; this.stateField = local; })",
                "Action-local declaration, complete State write, and existing Action batch",
                "one exact serializable local initializer with primitive compatibility to the target State field",
                "the local is compile-time data only; it introduces no runtime dependency, capture, or alias",
                "the resulting complete-State replacement uses the existing State resume codec",
                "existing assign Action operand and runtime State write; no new artifact field",
                "runtime_browser::serializable_action_local_updates_state_from_compiler_generated_runtime",
            ),
            admitted(
                "structured_serializable_action_locals",
                SemanticCapabilityClass::Bounded,
                "method = action(() => { const local = serializableRecordOrArray; this.stateField = local; })",
                "Action-local declaration, complete structured State write, and existing Action batch",
                "recursive literal shape compatible with the State initializer: exact object keys and homogeneous array element shape",
                "the local is resolved to one compiler-issued literal operand with no runtime alias, capture, or dependency",
                "the existing complete-State resume codec owns the structured value",
                "existing assign Action operand; no new runtime evaluator or artifact field",
                "runtime_browser::structured_serializable_action_local_executes_from_compiler_generated_runtime",
            ),
            admitted(
                "action",
                SemanticCapabilityClass::Bounded,
                "method = action(() => { /* compiler-supported State writes */ })",
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
                "get value()",
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
                "effectField = effect(() => { /* compiler-known capability program */ })",
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
                "legacy decorators: @context() static value; @provide(\"Owner.value\"); @consume(\"Owner.value\")",
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
                "content: SlotContent = slot(); <slot />",
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
                "typed_aria_bindings",
                SemanticCapabilityClass::Bounded,
                "dynamic role plus aria-label/aria-describedby/aria-errormessage/aria-controls/aria-current/aria-live string or aria-invalid/aria-busy/aria-expanded/aria-pressed/aria-hidden boolean JSX attribute",
                "template attribute binding, semantic type assignment, ordinary runtime binding, and DOM attribute update",
                "exact compiler-known boolean or string contract for the admitted ARIA name",
                "the attribute expression uses the existing compiler-derived State/Computed dependency path",
                "no independent resume value; the existing binding source resumes through its owner",
                "existing template/component runtime attribute binding records",
                "runtime_browser::typed_aria_attribute_updates_in_a_real_browser",
            ),
            admitted(
                "keyboard_action_event",
                SemanticCapabilityClass::Bounded,
                "onKeydown={this.actionMethod} on a compiler-supported element",
                "template event, Action batch, ordinary-instance event record, and delegated runtime listener",
                "exact keydown event with an existing zero-parameter compiler Action",
                "event activates the compiler-owned Action batch only; keyboard payload fields are not projected",
                "uses the existing Action activation and State resume path",
                "existing template and ordinary event artifacts with event_type keydown",
                "runtime_browser::structured_serializable_action_local_executes_from_compiler_generated_runtime",
            ),
            admitted(
                "form",
                SemanticCapabilityClass::Bounded,
                "legacy decorators: @form(); @field(\"form\"); @submit(\"form\")",
                "Form, Field, validation, and submission identities",
                "compiler-supported control and serializable field types",
                "compiler-owned Form plans and explicit host markers",
                "resumable through Form schema and records",
                "Forms runtime artifact",
                "fixtures/framework/resume-forms.tsx",
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
            admitted(
                "builtin_math_rounding",
                SemanticCapabilityClass::Bounded,
                "Math.floor(value), Math.ceil(value), or Math.round(value) in a supported @computed() getter",
                "compiler-registered BuiltinPureOperation, canonical unary IR, and runtime-computed artifact",
                "exactly one compiler-supported numeric operand",
                "inherits the operand's compiler-derived dependency set",
                "serializable numeric result; no package or independent resume record",
                "runtime-computed schema v12, including unary floor/ceil/round operations",
                "runtime_browser::registered_math_rounding_executes_from_compiler_generated_runtime_programs",
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
            admitted(
                "resources",
                SemanticCapabilityClass::Bounded,
                "legacy decorator: @resource(\"importedEndpoint\") field!: Resource<Data, Error>",
                "integrity-checked semantic-package endpoint, Resource declaration and activation identity, browser runtime artifact, and generated runtime",
                "one exactly imported semantic-package resource endpoint, Resource<Data, Error> field type, exact host-supplied runtime module location, and client/shared execution boundary; server/shared endpoints are admitted only through a route-loader handoff",
                "one compiler-owned cold activation per planned component instance; direct same-owner @computed() .data/.error/.state reads and terminal invalidation are admitted, while inputs and retry remain deferred",
                "snapshot policy restores validated state/data/error triples before dependent Computeds without importing the endpoint; reload has no snapshot slot and starts one post-restore generation; malformed, pending, or cancelled snapshots cold-fallback",
                "runtime resource schema v3 plus runtime-computed schema v12, embedded page artifacts, exact resume slots/codecs, and exact runtime module coordinate",
                "runtime_browser::host_bound_resource_endpoint_activates_in_a_real_browser",
            ),
            admitted(
                "route_loaders",
                SemanticCapabilityClass::Bounded,
                "legacy decorator: @loader(\"importedEndpoint\") field!: Resource<Data, Error> on a conventional route page",
                "compiler route graph, binding table, semantic-package contract, and RouteLoaderPlanV1 publication",
                "one integrity-bound server/shared resource export with route_parameters input, valid cache policy, and typed failure",
                "route parameters and cache policy are closed package facts; no browser reactive dependency or package source is evaluated",
                "server handoffs have no browser resume record or executor; a route carrying one is classified node-only",
                "route-loaders.plan.json schema v1 and the compiler-issued Node release inventory",
                "ergonomic_project::default_check_and_build_publish_a_compiler_route_loader_handoff",
            ),
            admitted(
                "server_actions",
                SemanticCapabilityClass::Bounded,
                "empty synchronous @serverAction(\"importedEndpoint\") method on a conventional route page",
                "compiler route graph, binding table, semantic-package contract, and RouteServerActionPlanV1 publication",
                "one integrity-bound server_action export with FormData -> ServerActionResult, cold_fallback, form_data input, json or redirect response, and typed failure",
                "the declaration has no parameters or body effects; the compiler records the closed server capability without reading package or application method source",
                "server actions are cold-fallback handoffs with no browser resume record or executor; a route carrying one is classified node-only",
                "route-server-actions.plan.json schema v1 and the compiler-issued Node release inventory",
                "ergonomic_project::default_check_and_build_publish_a_compiler_route_server_action_handoff",
            ),
            admitted(
                "opaque_typescript",
                SemanticCapabilityClass::Opaque,
                "legacy decorators: @action() @opaque(\"package\", \"export\") method(): void {}",
                "compiler-issued opaque terminal activation attached to one Action method",
                "empty synchronous zero-parameter Action; exact imported opaque semantic-package export with () -> void client terminal contract",
                "terminal after the compiler-owned Action batch; no State, Form, Context, Resource, or render dependency access",
                "opaque terminals force cold resume fallback; no opaque snapshot codec exists",
                "opaque.runtime.json schema v1, embedded page metadata, and exact integrity-bound runtime module coordinate",
                "runtime_browser::integrity_bound_opaque_terminal_runs_only_from_a_compiler_action_in_a_real_browser",
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

/// Deterministic human projection of the same registry used for JSON
/// inspection. It is intentionally a view, not a second capability list or a
/// schema-bearing product.
#[must_use]
pub fn semantic_capability_matrix_text() -> String {
    let registry = build_semantic_capability_registry();
    let mut output = format!(
        "Presolve semantic capability matrix (schema v{})\n\n",
        registry.schema_version
    );
    output.push_str("id | class | status | source form | proof fixture\n");
    output.push_str("--- | --- | --- | --- | ---\n");
    for capability in registry.capabilities {
        let proof_fixture = if capability.proof_fixture.is_empty() {
            "-"
        } else {
            capability.proof_fixture
        };
        writeln!(
            output,
            "{} | {} | {} | {} | {}",
            capability.id,
            capability_class_text(capability.class),
            capability_status_text(capability.status),
            capability.source_form,
            proof_fixture
        )
        .expect("writing capability matrix cannot fail");
        if let Some(reason) = capability.rejection_reason {
            writeln!(output, "  rejection: {reason}")
                .expect("writing capability matrix cannot fail");
        }
    }
    output
}

/// Deterministic compatibility and migration guidance for the current
/// registry. Deferred records are explained, never translated or relaxed.
#[must_use]
pub fn semantic_capability_migration_text() -> String {
    let registry = build_semantic_capability_registry();
    let deferred = registry
        .capabilities
        .iter()
        .filter(|capability| capability.status == SemanticCapabilityStatus::Deferred)
        .collect::<Vec<_>>();
    let mut output = format!(
        "Presolve semantic compatibility guide (registry schema v{})\n\n",
        registry.schema_version
    );
    output.push_str("Compatibility policy\n\n");
    output.push_str("- The registry is the exact compiler capability boundary.\n");
    output.push_str("- Admitted records are supported only under their listed compiler rules.\n");
    output.push_str("- Deferred records have no compatibility fallback, source rewrite, or opaque escape hatch.\n\n");
    output.push_str("Migration guide\n\n");
    for capability in &deferred {
        writeln!(
            output,
            "- {}: {}\n  reason: {}",
            capability.id,
            capability.source_form,
            capability
                .rejection_reason
                .expect("deferred capability has a rejection reason")
        )
        .expect("writing capability migration guide cannot fail");
    }
    output.push_str("\nRejected syntax catalog\n\n");
    for capability in deferred {
        writeln!(
            output,
            "- {} | {} | {}",
            capability.id,
            capability_class_text(capability.class),
            capability
                .rejection_reason
                .expect("deferred capability has a rejection reason")
        )
        .expect("writing rejected syntax catalog cannot fail");
    }
    output
}

fn capability_class_text(class: SemanticCapabilityClass) -> &'static str {
    match class {
        SemanticCapabilityClass::Native => "native",
        SemanticCapabilityClass::Bounded => "bounded",
        SemanticCapabilityClass::Opaque => "opaque",
        SemanticCapabilityClass::Unsupported => "unsupported",
    }
}

fn capability_status_text(status: SemanticCapabilityStatus) -> &'static str {
    match status {
        SemanticCapabilityStatus::Admitted => "admitted",
        SemanticCapabilityStatus::Deferred => "deferred",
    }
}

#[allow(clippy::too_many_arguments)]
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

#[allow(clippy::too_many_arguments)]
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
                "component_invocation",
                "state",
                "serializable_state_replacement",
                "static_action_parameters",
                "action_parameter_state_types",
                "serializable_action_locals",
                "structured_serializable_action_locals",
                "action",
                "computed",
                "effect",
                "context",
                "slot",
                "keyed_structural_list",
                "jsx_html_attribute_aliases",
                "typed_aria_bindings",
                "keyboard_action_event",
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
                "builtin_math_rounding",
                "semantic_package_exports",
                "resources",
                "route_loaders",
                "server_actions",
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
        let matrix = semantic_capability_matrix_text();
        assert_eq!(matrix, semantic_capability_matrix_text());
        assert!(matrix.starts_with("Presolve semantic capability matrix (schema v1)\n\n"));
        assert_eq!(
            matrix.matches("\ncomponent | native | admitted |").count(),
            1
        );
        assert!(matrix.contains("advanced_types | unsupported | deferred |"));
        assert!(matrix.contains("  rejection: N1-B must define advanced type semantics\n"));
        let migration = semantic_capability_migration_text();
        assert_eq!(migration, semantic_capability_migration_text());
        assert!(
            migration.starts_with("Presolve semantic compatibility guide (registry schema v1)\n\n")
        );
        assert!(!migration.contains("- opaque_typescript:"));
        assert!(!migration.contains("- resources:"));
        assert!(registry.capabilities.iter().any(|capability| {
            capability.id == "resources"
                && capability.status == SemanticCapabilityStatus::Admitted
                && capability.class == SemanticCapabilityClass::Bounded
        }));
        assert!(registry.capabilities.iter().any(|capability| {
            capability.id == "opaque_typescript"
                && capability.status == SemanticCapabilityStatus::Admitted
                && capability.class == SemanticCapabilityClass::Opaque
        }));
    }
}
