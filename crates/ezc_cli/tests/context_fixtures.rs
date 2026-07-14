use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

use ezc_core::{
    build_application_semantic_model, build_application_semantic_model_for_unit,
    build_resume_manifest, build_resume_plan, build_runtime_context_artifact,
    build_runtime_context_registry, build_semantic_graph, build_template_manifest_from_asm,
    collect_context_diagnostics, collect_context_resolutions, generate_runtime_stub,
    lower_components_to_ir, optimize_context_ir, resume_manifest_json,
    runtime_context_artifact_json, semantic_graph_json, validate_application_semantic_model,
    validate_context_ir, validate_optimized_context_ir, validate_resume_manifest,
    validate_runtime_context_registry, CompatibilityStatus, CompilationUnit, ComponentScopeGraph,
    ConsumerId, ContextBindingCompatibility, ContextBindingLifetimeStatus,
    ContextDeclarationStatus, ContextDependencyNodeId, ContextResolutionResult,
    ContextSerializationCompatibility, ContextSlotResumeStatus, ContextSourceBlockReason,
    ContextSourcePlanStatus, ContextValueSourceId, IrInstructionKind, ProviderId,
    RuntimeContextSourceKind, RESUME_MANIFEST_SCHEMA_VERSION,
    RUNTIME_CONTEXT_ARTIFACT_SCHEMA_VERSION, SEMANTIC_GRAPH_SCHEMA_VERSION,
    TEMPLATE_MANIFEST_SCHEMA_VERSION,
};

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .expect("repository root")
}

fn ezc_cli_bin() -> &'static str {
    env!("CARGO_BIN_EXE_ezc_cli")
}

fn fixture_source(path: &str) -> String {
    fs::read_to_string(repo_root().join(path)).expect("fixture source")
}

fn fixture_model(path: &str) -> ezc_core::ApplicationSemanticModel {
    build_application_semantic_model(&ezc_parser::parse_file(path, &fixture_source(path)))
}

fn run_cli(args: &[&str]) -> Output {
    Command::new(ezc_cli_bin())
        .current_dir(repo_root())
        .args(args)
        .output()
        .expect("ezc_cli command")
}

fn context_codes(diagnostics: &[ezc_core::ComponentDiagnostic]) -> BTreeSet<String> {
    diagnostics
        .iter()
        .filter(|diagnostic| ("EZC1052"..="EZC1067").contains(&diagnostic.code.as_str()))
        .map(|diagnostic| diagnostic.code.clone())
        .collect()
}

#[test]
fn context_compiler_fixture_covers_declarations_ownership_dependencies_and_lifetime() {
    let path = "fixtures/0057-context-compiler-matrix/input/ContextCompilerMatrix.tsx";
    let model = fixture_model(path);
    assert!(model.diagnostics.is_empty());
    assert!(ezc_core::validate_application_semantic_model(&model).is_empty());
    assert_eq!(model.contexts.len(), 4);
    assert_eq!(model.providers.len(), 2);
    assert_eq!(model.consumers.len(), 4);
    assert_eq!(
        model
            .context_declaration_candidates()
            .candidates()
            .iter()
            .filter(|candidate| matches!(candidate.status, ContextDeclarationStatus::Invalid(_)))
            .count(),
        0
    );

    let component = &model.components[0].id;
    let locale = ezc_core::ContextId::for_component(component, "locale");
    let theme_provider = ProviderId::for_component(component, "providedTheme");
    let total_provider = ProviderId::for_component(component, "providedTotal");
    let first_total = ConsumerId::for_component(component, "firstTotal");
    let second_total = ConsumerId::for_component(component, "secondTotal");
    let locale_consumer = ConsumerId::for_component(component, "localeValue");

    let owned = model.context_ownership.context_owned_entities(component);
    assert_eq!(owned.contexts.len(), 4);
    assert_eq!(
        owned.providers,
        vec![theme_provider.clone(), total_provider.clone()]
    );
    assert_eq!(owned.consumers.len(), 4);
    assert_eq!(
        model
            .context_dependency
            .provider_value_dependencies(&theme_provider),
        &[ContextDependencyNodeId::State(
            component.state_field("selectedTheme")
        )]
    );
    assert_eq!(
        model
            .context_dependency
            .provider_value_dependencies(&total_provider),
        &[ContextDependencyNodeId::Computed(
            component.computed("doubled")
        )]
    );
    assert_eq!(
        model
            .context_dependency
            .consumer_binding_dependency(&first_total),
        Some(&ContextDependencyNodeId::Provider(total_provider.clone()))
    );
    assert_eq!(
        model
            .context_dependency
            .consumers_bound_to_provider(&total_provider),
        &[first_total.clone(), second_total.clone()]
    );
    assert_eq!(
        model.context_dependency.consumers_bound_to_default(&locale),
        &[locale_consumer]
    );
    assert!(model
        .context_lifetime
        .binding_lifetimes
        .values()
        .all(|record| record.compatibility == ContextBindingLifetimeStatus::Compatible));
}

#[test]
fn context_compiler_fixture_covers_planning_and_ir() {
    let path = "fixtures/0057-context-compiler-matrix/input/ContextCompilerMatrix.tsx";
    let model = fixture_model(path);
    let component = &model.components[0].id;
    let theme = ezc_core::ContextId::for_component(component, "theme");
    let locale = ezc_core::ContextId::for_component(component, "locale");
    let theme_provider = ProviderId::for_component(component, "providedTheme");
    let total_provider = ProviderId::for_component(component, "providedTotal");
    let first_total = ConsumerId::for_component(component, "firstTotal");
    let second_total = ConsumerId::for_component(component, "secondTotal");
    let theme_default = ContextValueSourceId::ContextDefault(theme);
    let locale_default = ContextValueSourceId::ContextDefault(locale);
    let total_source = ContextValueSourceId::Provider(total_provider.clone());
    assert_eq!(
        model
            .context_evaluation
            .context_source_plan(&theme_default)
            .unwrap()
            .status,
        ContextSourcePlanStatus::Unused
    );
    assert_eq!(
        model
            .context_evaluation
            .context_source_plan(&locale_default)
            .unwrap()
            .status,
        ContextSourcePlanStatus::Planned
    );
    assert_eq!(model.context_evaluation.planned_context_sources().len(), 3);

    let ir = lower_components_to_ir(&model);
    let optimized = optimize_context_ir(&ir);
    assert!(optimized
        .source_report
        .context_source_evaluation(&theme_default)
        .is_none());
    assert!(optimized
        .source_report
        .context_source_evaluation(&locale_default)
        .is_some());
    let theme_source = ContextValueSourceId::Provider(theme_provider.clone());
    let source_function_kinds = |source: &ContextValueSourceId| {
        let evaluation = ir.context_source_evaluation(source).unwrap();
        ir.modules
            .iter()
            .flat_map(|module| &module.functions)
            .find(|function| function.id == *evaluation.function.as_semantic_id())
            .unwrap()
            .blocks
            .iter()
            .flat_map(|block| &block.instructions)
            .map(|instruction| &instruction.kind)
            .collect::<Vec<_>>()
    };
    assert!(source_function_kinds(&theme_source)
        .iter()
        .any(|kind| matches!(kind, IrInstructionKind::LoadStorage { .. })));
    assert!(source_function_kinds(&total_source)
        .iter()
        .any(|kind| matches!(kind, IrInstructionKind::LoadComputed { .. })));
    assert!(source_function_kinds(&locale_default)
        .iter()
        .any(|kind| matches!(kind, IrInstructionKind::Constant { .. })));
    let first_binding = optimized
        .optimized_module
        .context_consumer_binding(&first_total)
        .unwrap();
    let second_binding = optimized
        .optimized_module
        .context_consumer_binding(&second_total)
        .unwrap();
    assert_eq!(first_binding.source, total_source);
    assert_eq!(first_binding.slot, second_binding.slot);
    assert_ne!(first_binding.load.id, second_binding.load.id);
}

#[test]
fn context_compiler_fixture_covers_runtime_registry_and_update_artifact() {
    let path = "fixtures/0057-context-compiler-matrix/input/ContextCompilerMatrix.tsx";
    let model = fixture_model(path);
    let component = &model.components[0].id;
    let locale = ezc_core::ContextId::for_component(component, "locale");
    let theme_provider = ProviderId::for_component(component, "providedTheme");
    let locale_default = ContextValueSourceId::ContextDefault(locale);
    let optimized = optimize_context_ir(&lower_components_to_ir(&model));
    let registry = build_runtime_context_registry(&model, &optimized);
    assert_eq!(registry.sources.len(), 3);
    assert_eq!(registry.consumers.len(), 4);
    assert_eq!(
        registry
            .initial_batches
            .iter()
            .flat_map(|batch| batch.sources.iter())
            .collect::<Vec<_>>(),
        model
            .context_evaluation
            .evaluation_batches
            .iter()
            .flat_map(|batch| batch.sources.iter())
            .collect::<Vec<_>>()
    );
    assert_eq!(
        registry
            .source(&ContextValueSourceId::Provider(theme_provider))
            .unwrap()
            .source_kind,
        RuntimeContextSourceKind::Provider
    );
    assert_eq!(
        registry.source(&locale_default).unwrap().source_kind,
        RuntimeContextSourceKind::ContextDefault
    );

    let artifact = build_runtime_context_artifact(&model, &optimized);
    assert_eq!(artifact.schema_version, 2);
    let increment = artifact
        .action_updates
        .iter()
        .find(|update| {
            update
                .action_batch
                .ends_with("/action-batch:incrementTwice")
        })
        .unwrap();
    let ignore = artifact
        .action_updates
        .iter()
        .find(|update| update.action_batch.ends_with("/action-batch:ignore"))
        .unwrap();
    assert_eq!(increment.invalidated_sources.len(), 1);
    assert_eq!(increment.affected_consumers.len(), 2);
    assert!(ignore.invalidated_sources.is_empty());
}

#[test]
fn context_compiler_fixture_covers_resume_and_candidate_exclusion() {
    let path = "fixtures/0057-context-compiler-matrix/input/ContextCompilerMatrix.tsx";
    let model = fixture_model(path);
    let component = &model.components[0].id;
    let total_provider = ProviderId::for_component(component, "providedTotal");
    let resume = build_resume_plan(&model);
    let manifest = build_resume_manifest(&resume);
    assert_eq!(manifest.schema_version, 3);
    assert_eq!(manifest.context_slots.len(), 3);
    assert!(manifest
        .context_slots
        .iter()
        .all(|record| record.initial_status == ContextSlotResumeStatus::Uninitialized));
    let total_resume = manifest
        .context_slots
        .iter()
        .find(|record| record.source == total_provider.as_str())
        .unwrap();
    assert_ne!(
        total_resume.resume_slot_id.as_str(),
        total_resume.runtime_slot_id
    );
    assert_eq!(total_resume.consumer_ids.len(), 2);

    let semantic_graph = build_semantic_graph(&model);
    assert!(model
        .context_declaration_candidates()
        .candidates()
        .iter()
        .all(|candidate| semantic_graph
            .nodes
            .iter()
            .all(|node| node.id.as_str() != candidate.authored.id.as_str())));
}

#[test]
fn context_resolution_fixture_covers_frozen_edge_cases() {
    let path = "fixtures/0058-context-resolution-typing/input/ContextResolutionTyping.tsx";
    let model = fixture_model(path);
    let contracts = model
        .components
        .iter()
        .find(|component| component.class_name == "ContextContracts")
        .unwrap();
    let boundary = model
        .components
        .iter()
        .find(|component| component.class_name == "ContextBoundary")
        .unwrap();
    let toolbar = model
        .components
        .iter()
        .find(|component| component.class_name == "ContextToolbar")
        .unwrap();

    let exact_provider = ProviderId::for_component(&contracts.id, "exactProvider");
    let exact_consumer = ConsumerId::for_component(&contracts.id, "exactConsumer");
    assert_eq!(
        model.resolved_provider(&exact_consumer),
        Some(&exact_provider)
    );
    let fallback_consumer = ConsumerId::for_component(&contracts.id, "fallbackConsumer");
    assert!(matches!(
        model.context_resolution(&fallback_consumer).unwrap().result,
        ContextResolutionResult::ContextDefault { .. }
    ));
    let boundary_provider = ProviderId::for_component(&boundary.id, "boundaryProvider");
    let boundary_consumer = ConsumerId::for_component(&boundary.id, "boundaryConsumer");
    assert_eq!(
        model.resolved_provider(&boundary_consumer),
        Some(&boundary_provider)
    );
    let scoped_consumer = ConsumerId::for_component(&toolbar.id, "scopedExact");
    assert!(matches!(
        model.context_resolution(&scoped_consumer).unwrap().result,
        ContextResolutionResult::Unresolved
    ));

    let scope = ComponentScopeGraph::with_parent_relations(
        &model.components,
        BTreeMap::from([
            (toolbar.id.clone(), boundary.id.clone()),
            (boundary.id.clone(), contracts.id.clone()),
        ]),
    );
    let scoped = collect_context_resolutions(
        &model.consumers,
        &model.contexts,
        &model.providers,
        &model.expression_graph,
        &scope,
    );
    assert!(matches!(
        scoped[&scoped_consumer].result,
        ContextResolutionResult::Provider {
            ref provider,
            distance: 1,
            ..
        } if provider == &boundary_provider
    ));
}

#[test]
fn context_typing_fixture_covers_frozen_edge_cases() {
    let path = "fixtures/0058-context-resolution-typing/input/ContextResolutionTyping.tsx";
    let model = fixture_model(path);
    let contracts = model
        .components
        .iter()
        .find(|component| component.class_name == "ContextContracts")
        .unwrap();
    let exact_provider = ProviderId::for_component(&contracts.id, "exactProvider");
    let value_mismatch = ProviderId::for_component(&contracts.id, "valueMismatchProvider");
    assert_eq!(
        model
            .provider_type(&value_mismatch)
            .unwrap()
            .value_to_declaration,
        CompatibilityStatus::Incompatible
    );
    let declaration_mismatch =
        ProviderId::for_component(&contracts.id, "declarationMismatchProvider");
    assert_eq!(
        model
            .provider_type(&declaration_mismatch)
            .unwrap()
            .declaration_to_context,
        CompatibilityStatus::Incompatible
    );
    let narrowed = ConsumerId::for_component(&contracts.id, "narrowedChoice");
    let widened = ConsumerId::for_component(&contracts.id, "widenedChoice");
    assert_eq!(
        model.consumer_type(&narrowed).unwrap().context_to_consumer,
        CompatibilityStatus::Incompatible
    );
    assert_eq!(
        model.consumer_type(&widened).unwrap().context_to_consumer,
        CompatibilityStatus::Compatible
    );
    let alias = ConsumerId::for_component(&contracts.id, "aliasConsumer");
    assert_eq!(
        model.context_binding_type(&alias).unwrap().overall,
        ContextBindingCompatibility::Compatible
    );
    let unknown = ConsumerId::for_component(&contracts.id, "unknownConsumer");
    assert_eq!(
        model.consumer_type(&unknown).unwrap().context_to_consumer,
        CompatibilityStatus::Unknown
    );

    let mut serialization = model.clone();
    serialization
        .provider_types
        .get_mut(&exact_provider)
        .unwrap()
        .serialization = ContextSerializationCompatibility::NonSerializable;
    assert!(collect_context_diagnostics(&serialization)
        .iter()
        .any(|diagnostic| diagnostic.code == "EZC1063"));
    let mut boundary_failure = model.clone();
    boundary_failure
        .provider_types
        .get_mut(&exact_provider)
        .unwrap()
        .boundary_compatibility = CompatibilityStatus::Incompatible;
    assert!(collect_context_diagnostics(&boundary_failure)
        .iter()
        .any(|diagnostic| diagnostic.code == "EZC1064"));
}

#[test]
fn context_diagnostic_fixtures_cover_the_complete_stable_catalog() {
    let mut codes = BTreeSet::new();
    for path in [
        "fixtures/0061-context-declaration-diagnostics/input/InvalidContextDeclarations.tsx",
        "fixtures/0056-context-diagnostic-parity/input/ContextDiagnosticParity.tsx",
        "fixtures/0058-context-resolution-typing/input/ContextResolutionTyping.tsx",
    ] {
        codes.extend(context_codes(&fixture_model(path).diagnostics));
    }

    let path = "fixtures/0061-context-declaration-diagnostics/input/ContextDiagnosticProducts.tsx";
    let base = fixture_model(path);
    let consumer = base.consumers.values().next().unwrap().id.clone();
    let provider = base.providers.values().next().unwrap().id.clone();
    let context = base.contexts.values().next().unwrap().id.clone();

    let mut unresolved = base.clone();
    unresolved
        .context_resolutions
        .get_mut(&consumer)
        .unwrap()
        .result = ContextResolutionResult::Unresolved;
    codes.extend(context_codes(&collect_context_diagnostics(&unresolved)));

    let mut ambiguous = base.clone();
    let second_id = ProviderId::for_component(&ambiguous.components[0].id, "secondProvider");
    let mut second = ambiguous.provider(&provider).unwrap().clone();
    second.id = second_id.clone();
    second.provenance.span.start += 1;
    second.provenance.span.end += 1;
    ambiguous.providers.insert(second_id.clone(), second);
    ambiguous
        .context_resolutions
        .get_mut(&consumer)
        .unwrap()
        .result = ContextResolutionResult::Ambiguous {
        providers: vec![second_id, provider.clone()],
        distance: 0,
    };
    codes.extend(context_codes(&collect_context_diagnostics(&ambiguous)));

    let mut serialization = base.clone();
    serialization
        .provider_types
        .get_mut(&provider)
        .unwrap()
        .serialization = ContextSerializationCompatibility::NonSerializable;
    codes.extend(context_codes(&collect_context_diagnostics(&serialization)));

    let mut boundary = base.clone();
    boundary
        .provider_types
        .get_mut(&provider)
        .unwrap()
        .boundary_compatibility = CompatibilityStatus::Incompatible;
    codes.extend(context_codes(&collect_context_diagnostics(&boundary)));

    let mut lifetime = base.clone();
    lifetime
        .context_lifetime
        .binding_lifetimes
        .get_mut(&consumer)
        .unwrap()
        .compatibility = ContextBindingLifetimeStatus::Incompatible;
    codes.extend(context_codes(&collect_context_diagnostics(&lifetime)));

    let source = ContextValueSourceId::Provider(provider);
    let mut dependency = base.clone();
    let dependency_entry = dependency
        .context_evaluation
        .source_entries
        .get_mut(&source)
        .unwrap();
    dependency_entry.status = ContextSourcePlanStatus::BlockedDependency;
    dependency_entry.reasons = vec![ContextSourceBlockReason::MissingStateDependency(
        dependency.components[0].id.state_field("missing"),
    )];
    codes.extend(context_codes(&collect_context_diagnostics(&dependency)));

    let mut lowering = base.clone();
    let lowering_entry = lowering
        .context_evaluation
        .source_entries
        .get_mut(&source)
        .unwrap();
    lowering_entry.status = ContextSourcePlanStatus::BlockedUnsupportedExpression;
    lowering_entry.reasons = vec![ContextSourceBlockReason::UnsupportedExpression];
    codes.extend(context_codes(&collect_context_diagnostics(&lowering)));

    assert_eq!(
        codes,
        (1052..=1067).map(|code| format!("EZC{code}")).collect()
    );
    assert!(base
        .context_declaration_candidates()
        .candidates()
        .iter()
        .all(|candidate| matches!(candidate.status, ContextDeclarationStatus::Valid(_))));
    assert_eq!(context, base.providers[&source_provider(&source)].context);
}

#[test]
fn blocked_context_fixture_products_are_excluded_from_ir_runtime_and_resume() {
    let path = "fixtures/0061-context-declaration-diagnostics/input/ContextDiagnosticProducts.tsx";
    let base = fixture_model(path);
    let provider = base.providers.values().next().unwrap().id.clone();
    let consumer = base.consumers.values().next().unwrap().id.clone();
    let source = ContextValueSourceId::Provider(provider);

    for (status, reason) in [
        (
            ContextSourcePlanStatus::BlockedDependency,
            ContextSourceBlockReason::MissingStateDependency(
                base.components[0].id.state_field("missing"),
            ),
        ),
        (
            ContextSourcePlanStatus::BlockedUnsupportedExpression,
            ContextSourceBlockReason::UnsupportedExpression,
        ),
    ] {
        let mut model = base.clone();
        let entry = model
            .context_evaluation
            .source_entries
            .get_mut(&source)
            .unwrap();
        entry.status = status;
        entry.reasons = vec![reason];

        let ir = optimize_context_ir(&lower_components_to_ir(&model));
        assert!(ir
            .optimized_module
            .context_source_evaluation(&source)
            .is_none());
        assert!(ir
            .optimized_module
            .context_consumer_binding(&consumer)
            .is_none());
        assert!(build_runtime_context_registry(&model, &ir)
            .sources
            .is_empty());
        assert!(build_runtime_context_artifact(&model, &ir)
            .sources
            .is_empty());
        assert!(build_resume_manifest(&build_resume_plan(&model))
            .context_slots
            .is_empty());
    }
}

fn source_provider(source: &ContextValueSourceId) -> ProviderId {
    match source {
        ContextValueSourceId::Provider(provider) => provider.clone(),
        ContextValueSourceId::ContextDefault(_) => unreachable!("fixture uses Provider source"),
    }
}

#[test]
fn context_multi_file_fixture_keeps_module_qualified_identities_distinct() {
    let alpha = "fixtures/0060-context-multi-file-identity/input/AlphaContext.tsx";
    let beta = "fixtures/0060-context-multi-file-identity/input/BetaContext.tsx";
    let owner = "fixtures/0060-context-multi-file-identity/input/ContextOwner.tsx";
    let users = "fixtures/0060-context-multi-file-identity/input/ContextUsers.tsx";
    let unit = CompilationUnit::parse_sources([
        (alpha, fixture_source(alpha)),
        (beta, fixture_source(beta)),
        (owner, fixture_source(owner)),
        (users, fixture_source(users)),
    ]);
    let model = build_application_semantic_model_for_unit(&unit);
    assert!(model.diagnostics.is_empty());
    assert!(ezc_core::validate_application_semantic_model(&model).is_empty());
    assert_eq!(model.contexts.len(), 3);
    assert_eq!(model.providers.len(), 3);
    assert_eq!(model.consumers.len(), 3);
    let context_ids = model
        .contexts
        .keys()
        .map(ezc_core::ContextId::as_str)
        .collect::<Vec<_>>();
    assert_ne!(context_ids[0], context_ids[1]);
    assert!(context_ids
        .iter()
        .filter(|id| id.contains("AlphaContext.tsx") || id.contains("BetaContext.tsx"))
        .all(|id| id.ends_with("/context:value")));
    assert!(context_ids.iter().any(|id| id.contains("AlphaContext.tsx")));
    assert!(context_ids.iter().any(|id| id.contains("BetaContext.tsx")));

    let imported_context = model
        .contexts
        .keys()
        .find(|id| id.as_str().contains("ContextOwner.tsx"))
        .unwrap();
    let imported_provider = model
        .providers
        .values()
        .find(|provider| provider.id.as_str().contains("ContextUsers.tsx"))
        .unwrap();
    let imported_consumer = model
        .consumers
        .values()
        .find(|consumer| consumer.id.as_str().contains("ContextUsers.tsx"))
        .unwrap();
    assert_eq!(&imported_provider.context, imported_context);
    assert_eq!(imported_consumer.context(), Some(imported_context));
}

#[test]
fn context_fixture_outputs_are_byte_deterministic_across_all_serialized_surfaces() {
    let path = "fixtures/0057-context-compiler-matrix/input/ContextCompilerMatrix.tsx";
    let context_id = concat!(
        "module:fixtures/0057-context-compiler-matrix/input/ContextCompilerMatrix.tsx/",
        "component:x-context-compiler-matrix/context:total"
    );
    for args in [
        vec!["check", path, "--format", "json"],
        vec!["asm", path, "--format", "json"],
        vec!["explain", path, "--entity", context_id, "--format", "json"],
    ] {
        let first = run_cli(&args);
        let second = run_cli(&args);
        assert_eq!(first.status.success(), second.status.success());
        assert_eq!(first.stdout, second.stdout);
        assert_eq!(first.stderr, second.stderr);
    }

    let model = fixture_model(path);
    let first_graph = semantic_graph_json(&build_semantic_graph(&model));
    let second_graph = semantic_graph_json(&build_semantic_graph(&model));
    assert_eq!(first_graph, second_graph);
    let first_ir = optimize_context_ir(&lower_components_to_ir(&model));
    let second_ir = optimize_context_ir(&lower_components_to_ir(&model));
    assert_eq!(first_ir, second_ir);
    let first_registry = build_runtime_context_registry(&model, &first_ir);
    let second_registry = build_runtime_context_registry(&model, &second_ir);
    assert_eq!(first_registry, second_registry);
    assert_eq!(
        runtime_context_artifact_json(&build_runtime_context_artifact(&model, &first_ir)),
        runtime_context_artifact_json(&build_runtime_context_artifact(&model, &second_ir))
    );
    assert_eq!(
        resume_manifest_json(&build_resume_manifest(&build_resume_plan(&model))),
        resume_manifest_json(&build_resume_manifest(&build_resume_plan(&model)))
    );

    let out_a = repo_root().join("target/ezc-test-output/context-fixture-determinism-a");
    let out_b = repo_root().join("target/ezc-test-output/context-fixture-determinism-b");
    for out in [&out_a, &out_b] {
        if out.exists() {
            fs::remove_dir_all(out).expect("clean deterministic build output");
        }
        let output = run_cli(&["build", path, "--out", out.to_str().unwrap()]);
        assert!(output.status.success());
    }
    for artifact in ["context.runtime.json", "template.manifest.json"] {
        assert_eq!(
            fs::read(out_a.join(artifact)).unwrap(),
            fs::read(out_b.join(artifact)).unwrap()
        );
    }
}

#[test]
fn phase_g_diagnostic_fixtures_are_canonically_validated() {
    for path in [
        "fixtures/0057-context-compiler-matrix/input/ContextCompilerMatrix.tsx",
        "fixtures/0058-context-resolution-typing/input/ContextResolutionTyping.tsx",
        "fixtures/0061-context-declaration-diagnostics/input/InvalidContextDeclarations.tsx",
        "fixtures/0061-context-declaration-diagnostics/input/ContextDiagnosticProducts.tsx",
    ] {
        let validation = validate_application_semantic_model(&fixture_model(path));
        assert!(validation.is_empty(), "{path}: {validation:#?}");
    }
}

#[test]
fn phase_g_freezes_schema_versions_runtime_order_and_no_discovery_contract() {
    assert_eq!(SEMANTIC_GRAPH_SCHEMA_VERSION, 5);
    assert_eq!(RUNTIME_CONTEXT_ARTIFACT_SCHEMA_VERSION, 2);
    assert_eq!(TEMPLATE_MANIFEST_SCHEMA_VERSION, 2);
    assert_eq!(RESUME_MANIFEST_SCHEMA_VERSION, 3);

    let path = "fixtures/0059-context-runtime-matrix/input/ContextRuntimeMatrix.tsx";
    let model = fixture_model(path);
    let ir = lower_components_to_ir(&model);
    assert!(validate_context_ir(&model, &ir).is_empty());
    let optimized = optimize_context_ir(&ir);
    assert!(validate_optimized_context_ir(&model, &ir, &optimized).is_empty());
    let registry = build_runtime_context_registry(&model, &optimized);
    assert!(validate_runtime_context_registry(&model, &optimized, &registry).is_empty());
    assert_eq!(build_semantic_graph(&model).schema_version, 5);
    assert_eq!(
        build_runtime_context_artifact(&model, &optimized).schema_version,
        2
    );
    let resume = build_resume_manifest(&build_resume_plan(&model));
    assert_eq!(resume.schema_version, 3);
    assert!(validate_resume_manifest(&resume).is_empty());
    assert_eq!(build_template_manifest_from_asm(&model).schema_version, 2);

    for (args, expected) in [
        (vec!["check", path, "--format", "json"], 3),
        (vec!["asm", path, "--format", "json"], 6),
    ] {
        let output = run_cli(&args);
        assert!(output.status.success());
        let document: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
        assert_eq!(document["schema_version"], expected);
    }

    let runtime = generate_runtime_stub();
    for forbidden in [
        "contextProviders",
        "resolveContext",
        "contextName",
        "contextKey",
        "contextAncestors",
        "reconstructContext",
    ] {
        assert!(!runtime.contains(forbidden), "runtime contains {forbidden}");
    }
    let cold_computed = runtime.find("executeComputedPlan(store);").unwrap();
    let cold_context = runtime.find("executeInitialContext(store);").unwrap();
    let cold_effects = runtime.find("executeInitialEffects(store);").unwrap();
    assert!(cold_computed < cold_context && cold_context < cold_effects);
    let update_computed = runtime
        .find("executeComputedUpdateBatches(store);")
        .unwrap();
    let update_context = runtime
        .find("executeContextUpdates(store, actionBatchId);")
        .unwrap();
    let update_effects = runtime
        .find("executeCompletedActionEffects(store, actionBatchId);")
        .unwrap();
    assert!(update_computed < update_context && update_context < update_effects);
}
