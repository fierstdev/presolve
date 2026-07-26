#![allow(clippy::too_many_lines)]

use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use presolve_compiler::{
    build_application_semantic_model, build_application_semantic_model_for_unit,
    build_resume_activation_plan, build_resume_anchor_plan, build_resume_boundary_graph,
    build_resume_capture_plan, build_resume_chunk_graph, build_resume_liveness_plan,
    build_resume_manifest, build_resume_restore_plan, build_resume_schema_registry,
    build_runtime_component_artifact, build_runtime_component_registry,
    build_runtime_context_artifact, build_semantic_graph, build_template_manifest_from_asm,
    collect_component_diagnostics, generate_runtime_stub, lower_components_to_ir,
    optimize_context_ir, project_resume_diagnostics, resume_manifest_json,
    runtime_component_artifact_json, runtime_context_artifact_json, semantic_graph_json,
    template_manifest_json, validate_application_semantic_model, validate_resume_manifest,
    validate_runtime_component_artifact, BlockedComponentInstancePlan,
    BlockedComponentInstanceReason, CompilationUnit, ComponentInstanceStatus,
    ComponentInvocationResolutionStatus, CompositionCompatibility, InstanceContextResolutionStatus,
    ResumeManifestPhaseIComponentResumeRecord, SlotBindingStatus, COMPONENT_DIAGNOSTIC_CONTRACTS,
    RESUME_MANIFEST_SCHEMA_VERSION, RUNTIME_COMPONENT_ARTIFACT_SCHEMA_VERSION,
    RUNTIME_COMPONENT_REGISTRY_SCHEMA_CONTRACT_VERSION, RUNTIME_CONTEXT_ARTIFACT_SCHEMA_VERSION,
    SEMANTIC_GRAPH_SCHEMA_VERSION, TEMPLATE_MANIFEST_SCHEMA_VERSION,
};

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .expect("repository root")
}

fn fixture_source(path: &str) -> String {
    fs::read_to_string(repo_root().join(path)).expect("fixture source")
}

fn fixture_model(path: &str) -> presolve_compiler::ApplicationSemanticModel {
    build_application_semantic_model(&presolve_parser::parse_file(path, &fixture_source(path)))
}

fn fixture_unit(paths: &[&str]) -> presolve_compiler::ApplicationSemanticModel {
    let sources = paths
        .iter()
        .map(|path| (*path, fixture_source(path)))
        .collect::<Vec<_>>();
    build_application_semantic_model_for_unit(&CompilationUnit::parse_sources(sources))
}

fn cli_result(args: &[&str]) -> (Option<i32>, Vec<u8>, Vec<u8>) {
    let output = Command::new(env!("CARGO_BIN_EXE_presolve"))
        .current_dir(repo_root())
        .args(args)
        .output()
        .expect("CLI fixture command");
    (output.status.code(), output.stdout, output.stderr)
}

fn cli_output(args: &[&str]) -> Vec<u8> {
    let (status, stdout, stderr) = cli_result(args);
    assert!(
        status == Some(0),
        "command failed: {args:?}\nstatus: {status:?}\nstderr:\n{}",
        String::from_utf8_lossy(&stderr)
    );
    stdout
}

fn component_codes(model: &presolve_compiler::ApplicationSemanticModel) -> BTreeSet<String> {
    collect_component_diagnostics(model)
        .into_iter()
        .map(|diagnostic| diagnostic.code)
        .filter(|code| ("PSC1068"..="PSC1083").contains(&code.as_str()))
        .collect()
}

#[test]
fn component_declaration_fixture_covers_slots_inheritance_imports_and_repeated_invocations() {
    let valid = "fixtures/0062-component-declarations/input/ValidComponents.tsx";
    let imported = "fixtures/0062-component-declarations/input/ImportedPage.tsx";
    let model = fixture_unit(&[imported, valid]);
    let repeated = fixture_unit(&[valid, imported]);
    let reversed = fixture_unit(&[imported, valid]);

    assert!(model.diagnostics.is_empty(), "{:#?}", model.diagnostics);
    assert!(validate_application_semantic_model(&model).is_empty());
    assert_eq!(model.slots.len(), 2);
    assert_eq!(model.component_invocations.len(), 3);
    assert!(model.component_invocations.values().all(|invocation| {
        invocation.status == ComponentInvocationResolutionStatus::Resolved
            && invocation.target_component.is_some()
    }));
    let invocation_ids = model
        .component_invocations
        .keys()
        .map(ToString::to_string)
        .collect::<BTreeSet<_>>();
    assert_eq!(invocation_ids.len(), 3);
    assert_eq!(
        model.component_instance_plan,
        repeated.component_instance_plan
    );
    assert_eq!(model.slot_bindings, repeated.slot_bindings);
    assert_eq!(model.component_invocations, reversed.component_invocations);
    assert_eq!(
        model.component_instance_plan,
        reversed.component_instance_plan
    );
    assert_eq!(model.slot_bindings, reversed.slot_bindings);

    let invalid = fixture_model("fixtures/0062-component-declarations/input/InvalidSlots.tsx");
    assert!(invalid.slots.is_empty());
    assert_eq!(
        component_codes(&invalid),
        BTreeSet::from(["PSC1068".to_string()])
    );
    let candidates = invalid
        .components
        .iter()
        .flat_map(|component| &component.slot_declaration_candidates)
        .collect::<Vec<_>>();
    assert_eq!(candidates.len(), 12);
    assert!(candidates
        .iter()
        .all(|candidate| !candidate.violations.is_empty()));

    let inheritance = fixture_model("fixtures/0062-component-declarations/input/Inheritance.tsx");
    assert_eq!(
        component_codes(&inheritance),
        BTreeSet::from(["PSC1072".to_string(), "PSC1073".to_string()])
    );
}

#[test]
fn component_composition_fixture_covers_topology_slots_caller_ownership_and_blocking() {
    let model = fixture_model("fixtures/0063-component-composition/input/Composition.tsx");
    assert!(
        component_codes(&model).is_empty(),
        "{:#?}",
        model.diagnostics
    );
    assert!(validate_application_semantic_model(&model).is_empty());
    assert!(model.component_composition.cycles.is_empty());
    assert!(model.component_instance_plan.blocked.is_empty());
    assert!(model
        .component_instance_plan
        .instances
        .values()
        .any(|instance| instance.depth >= 2));
    let root = model
        .component_instance_plan
        .instances
        .values()
        .find(|instance| instance.parent_instance.is_none())
        .unwrap();
    let root_children = model
        .component_instance_plan
        .instances
        .values()
        .filter(|instance| instance.parent_instance.as_ref() == Some(&root.id))
        .collect::<Vec<_>>();
    assert!(root_children.len() >= 3);
    assert!(
        root_children
            .iter()
            .filter(|instance| instance.component.as_str().ends_with("/component:x-card"))
            .count()
            >= 2
    );
    assert!(model.slot_bindings.bindings.values().any(|binding| {
        binding.status == SlotBindingStatus::Bound
            && binding.content_owner_instance == binding.caller_instance
            && binding.content_owner_instance != binding.callee_instance
    }));
    assert!(model
        .slot_bindings
        .bindings
        .values()
        .any(|binding| binding.status == SlotBindingStatus::Empty));
    assert!(model
        .slot_content_fragments
        .values()
        .any(|fragment| fragment.content_template_entities.len() > 1));
    assert!(model
        .composition_types
        .slot_bindings
        .values()
        .all(|record| { record.overall == CompositionCompatibility::Compatible }));

    let invalid = fixture_model("fixtures/0063-component-composition/input/InvalidComposition.tsx");
    let codes = component_codes(&invalid);
    for code in [
        "PSC1070", "PSC1071", "PSC1074", "PSC1075", "PSC1076", "PSC1077",
    ] {
        assert!(codes.contains(code), "missing {code}: {codes:#?}");
    }
    assert!(!invalid.component_composition.cycles.is_empty());
    assert!(!invalid.component_instance_plan.blocked.is_empty());
    assert!(invalid
        .component_instance_plan
        .blocked
        .values()
        .any(|blocked| blocked.reason == BlockedComponentInstanceReason::UnresolvedInvocation));
    assert!(invalid
        .component_instance_plan
        .blocked
        .values()
        .any(|blocked| {
            blocked.reason == BlockedComponentInstanceReason::CompositionCycleBoundary
        }));
    assert!(invalid.slot_bindings.bindings.values().any(|binding| {
        !matches!(
            binding.status,
            SlotBindingStatus::Bound | SlotBindingStatus::Empty
        )
    }));
}

#[test]
fn instance_context_fixture_selects_exact_nearest_sources_without_leakage() {
    let model = fixture_model("fixtures/0064-component-instance-context/input/InstanceContext.tsx");
    assert!(
        component_codes(&model).is_empty(),
        "{:#?}",
        model.diagnostics
    );
    assert!(validate_application_semantic_model(&model).is_empty());
    assert_eq!(model.instance_context.resolutions.len(), 3);
    assert_eq!(
        model
            .instance_context
            .resolutions
            .values()
            .filter(
                |resolution| resolution.status == InstanceContextResolutionStatus::ProviderSelected
            )
            .count(),
        2
    );
    assert_eq!(
        model
            .instance_context
            .resolutions
            .values()
            .filter(|resolution| {
                resolution.status == InstanceContextResolutionStatus::ContextDefaultSelected
            })
            .count(),
        1
    );
    assert!(model
        .instance_context
        .resolutions
        .values()
        .all(|resolution| {
            resolution.selected_source.is_some()
                && resolution.value_slot.is_some()
                && resolution.candidate_provider_instances.len() <= 1
        }));
    let sources = model
        .instance_context
        .resolutions
        .values()
        .map(|resolution| resolution.selected_source.as_ref().unwrap().to_string())
        .collect::<BTreeSet<_>>();
    assert_eq!(sources.len(), 3);

    let ambiguous =
        fixture_model("fixtures/0064-component-instance-context/input/AmbiguousContext.tsx");
    let resolution = ambiguous
        .instance_context
        .resolutions
        .values()
        .next()
        .unwrap();
    assert_eq!(
        resolution.status,
        InstanceContextResolutionStatus::Unresolved
    );
    assert!(resolution.candidate_provider_instances.is_empty());
    assert!(resolution.selected_source.is_none());
    assert!(resolution.value_slot.is_none());
    assert!(ambiguous.providers.is_empty());
    assert_eq!(ambiguous.duplicate_provider_declarations.len(), 2);
    assert_eq!(
        ambiguous
            .diagnostics
            .iter()
            .filter(|diagnostic| diagnostic.code == "PSC1056")
            .count(),
        1
    );
}

#[test]
fn phase_j_fixture_products_remain_byte_identical_under_multifile_reversal() {
    let forward = fixture_unit(&[
        "fixtures/0062-component-declarations/input/ImportedPage.tsx",
        "fixtures/0062-component-declarations/input/ValidComponents.tsx",
    ]);
    let reversed = fixture_unit(&[
        "fixtures/0062-component-declarations/input/ValidComponents.tsx",
        "fixtures/0062-component-declarations/input/ImportedPage.tsx",
    ]);
    assert!(forward.diagnostics.is_empty(), "{:#?}", forward.diagnostics);
    assert!(
        reversed.diagnostics.is_empty(),
        "{:#?}",
        reversed.diagnostics
    );

    assert_eq!(
        resume_manifest_json(&build_resume_manifest(&forward)),
        resume_manifest_json(&build_resume_manifest(&reversed))
    );
    assert_eq!(
        build_resume_liveness_plan(&forward),
        build_resume_liveness_plan(&reversed)
    );
    assert_eq!(
        build_resume_boundary_graph(&forward),
        build_resume_boundary_graph(&reversed)
    );
    assert_eq!(
        build_resume_activation_plan(&forward),
        build_resume_activation_plan(&reversed)
    );
    assert_eq!(
        build_resume_chunk_graph(&forward),
        build_resume_chunk_graph(&reversed)
    );
    assert_eq!(
        build_resume_schema_registry(&forward),
        build_resume_schema_registry(&reversed)
    );
    assert_eq!(
        build_resume_capture_plan(&forward),
        build_resume_capture_plan(&reversed)
    );
    assert_eq!(
        build_resume_restore_plan(&forward),
        build_resume_restore_plan(&reversed)
    );
    assert_eq!(
        build_resume_anchor_plan(&forward),
        build_resume_anchor_plan(&reversed)
    );
    assert_eq!(
        project_resume_diagnostics(&forward),
        project_resume_diagnostics(&reversed)
    );
}

#[test]
fn component_runtime_and_resume_fixtures_preserve_order_isolation_structure_and_failures() {
    let path = "fixtures/0065-component-runtime/input/RuntimeComponents.tsx";
    let model = fixture_model(path);
    assert!(
        component_codes(&model).is_empty(),
        "{:#?}",
        model.diagnostics
    );
    let artifact = build_runtime_component_artifact(&model, &model.component_ir_optimization);
    assert_eq!(
        artifact.schema_version,
        RUNTIME_COMPONENT_ARTIFACT_SCHEMA_VERSION
    );
    assert!(validate_runtime_component_artifact(&artifact).is_ok());
    assert_eq!(
        runtime_component_artifact_json(&artifact),
        runtime_component_artifact_json(&build_runtime_component_artifact(
            &fixture_model(path),
            &fixture_model(path).component_ir_optimization,
        ))
    );
    assert!(artifact.instances.len() >= 6);
    let state_slots = artifact
        .instances
        .iter()
        .flat_map(|instance| instance.state_slots.iter())
        .map(|slot| slot.slot_id.as_str())
        .collect::<BTreeSet<_>>();
    assert_eq!(
        state_slots.len(),
        artifact
            .instances
            .iter()
            .map(|instance| instance.state_slots.len())
            .sum::<usize>()
    );
    assert!(artifact.instances.iter().all(|instance| {
        instance.state_slots.iter().all(|slot| {
            slot.slot_id
                .starts_with(&format!("{}/state-slot:", instance.instance))
        })
    }));
    assert!(artifact.instances.iter().all(|instance| {
        instance.parent.as_ref().is_none_or(|parent| {
            artifact
                .instances
                .iter()
                .find(|candidate| &candidate.instance == parent)
                .is_some_and(|parent| {
                    parent.depth < instance.depth
                        && parent.initialization_batch < instance.initialization_batch
                })
        })
    }));
    assert_eq!(artifact.slot_binding_programs.len(), 2);
    assert_eq!(artifact.instance_context_bindings.len(), 3);
    assert!(artifact
        .slot_binding_programs
        .iter()
        .all(|binding| binding.content_owner_instance == binding.caller_instance));

    let resume = build_resume_manifest(&model);
    assert_eq!(resume.schema_version, RESUME_MANIFEST_SCHEMA_VERSION);
    assert!(validate_resume_manifest(&resume).is_empty());
    let component_instances = resume
        .phase_i_component_resume_records
        .iter()
        .filter_map(|record| match record {
            ResumeManifestPhaseIComponentResumeRecord::ComponentInstance { component_instance } => {
                Some(component_instance)
            }
            _ => None,
        })
        .collect::<Vec<_>>();
    let resume_ids = component_instances
        .iter()
        .map(|instance| instance.resume_id.as_str())
        .collect::<BTreeSet<_>>();
    assert_eq!(resume_ids.len(), component_instances.len());
    assert_eq!(
        resume_manifest_json(&resume),
        resume_manifest_json(&build_resume_manifest(&fixture_model(path)))
    );
    assert!(component_instances
        .iter()
        .all(|instance| instance.active_status == "active"));
    assert!(
        component_instances
            .iter()
            .filter(|instance| instance.component.ends_with("/component:x-runtime-card"))
            .map(|instance| instance.resume_id.as_str())
            .collect::<BTreeSet<_>>()
            .len()
            >= 2
    );
    assert_eq!(
        resume
            .phase_i_component_resume_records
            .iter()
            .filter(|record| matches!(
                record,
                ResumeManifestPhaseIComponentResumeRecord::SlotBinding { .. }
            ))
            .count(),
        artifact.slot_binding_programs.len()
    );

    let structural =
        fixture_model("fixtures/0065-component-runtime/input/StructuralComponents.tsx");
    let structural_artifact =
        build_runtime_component_artifact(&structural, &structural.component_ir_optimization);
    assert!(!structural_artifact.structural_programs.is_empty());
    assert!(structural_artifact
        .structural_programs
        .iter()
        .all(|program| {
            program.create_order == program.template_instances
                && program
                    .destroy_order
                    .iter()
                    .rev()
                    .eq(program.template_instances.iter())
        }));
    let structural_resume = build_resume_manifest(&structural);
    assert!(structural_resume
        .phase_i_component_resume_records
        .iter()
        .any(|record| matches!(
            record,
            ResumeManifestPhaseIComponentResumeRecord::StructuralRegion { .. }
        )));
    assert!(structural_resume
        .phase_i_component_resume_records
        .iter()
        .filter_map(|record| match record {
            ResumeManifestPhaseIComponentResumeRecord::ComponentInstance { component_instance } =>
                Some(component_instance),
            _ => None,
        })
        .any(|instance| instance.active_status == "inactive"));
    assert!(structural_resume
        .phase_i_component_resume_records
        .iter()
        .filter_map(|record| match record {
            ResumeManifestPhaseIComponentResumeRecord::StructuralRegion { structural_region } => {
                Some(structural_region)
            }
            _ => None,
        })
        .all(|region| region.active_status == "inactive"));
    assert_eq!(
        resume_manifest_json(&structural_resume),
        resume_manifest_json(&build_resume_manifest(&fixture_model(
            "fixtures/0065-component-runtime/input/StructuralComponents.tsx"
        )))
    );

    let failure = fixture_model("fixtures/0065-component-runtime/input/FailureIsolation.tsx");
    assert_eq!(
        component_codes(&failure),
        BTreeSet::from(["PSC1070".to_string()])
    );
    assert!(failure
        .component_instance_plan
        .blocked
        .values()
        .all(|blocked| {
            blocked.target_component.is_none()
                && blocked.reason == BlockedComponentInstanceReason::UnresolvedInvocation
        }));
    let failure_artifact =
        build_runtime_component_artifact(&failure, &failure.component_ir_optimization);
    assert!(failure_artifact
        .instances
        .iter()
        .any(|instance| instance.component.ends_with("/component:x-safe-leaf")));
    let failure_resume = build_resume_manifest(&failure);
    assert!(failure_resume
        .phase_i_component_resume_records
        .iter()
        .filter_map(|record| match record {
            ResumeManifestPhaseIComponentResumeRecord::ComponentInstance { component_instance } =>
                Some(component_instance),
            _ => None,
        })
        .all(|instance| !instance.component.contains("Missing")));
}

#[test]
fn component_outputs_are_byte_deterministic_across_compiler_and_cli_surfaces() {
    let path = "fixtures/0065-component-runtime/input/RuntimeComponents.tsx";
    let model = fixture_model(path);
    let repeated = fixture_model(path);

    assert_eq!(
        semantic_graph_json(&build_semantic_graph(&model)),
        semantic_graph_json(&build_semantic_graph(&repeated))
    );
    assert_eq!(
        template_manifest_json(&build_template_manifest_from_asm(&model)),
        template_manifest_json(&build_template_manifest_from_asm(&repeated))
    );
    let context_ir = optimize_context_ir(&lower_components_to_ir(&model));
    let repeated_context_ir = optimize_context_ir(&lower_components_to_ir(&repeated));
    assert_eq!(
        runtime_context_artifact_json(&build_runtime_context_artifact(&model, &context_ir)),
        runtime_context_artifact_json(&build_runtime_context_artifact(
            &repeated,
            &repeated_context_ir
        ))
    );

    let component =
        "module:fixtures/0065-component-runtime/input/RuntimeComponents.tsx/component:x-runtime-page";
    for args in [vec!["check", path], vec!["check", path, "--format", "json"]] {
        assert_eq!(cli_result(&args), cli_result(&args), "{args:?}");
    }
    for args in [
        vec!["explain", "--inspect", path],
        vec!["explain", "--inspect", path, "--format", "json"],
        vec!["explain", "--inspect", path, "--format", "graph"],
        vec!["explain", "--inspect", path, "--entity", component],
        vec![
            "explain",
            "--inspect",
            path,
            "--entity",
            component,
            "--format",
            "json",
        ],
    ] {
        assert_eq!(cli_output(&args), cli_output(&args), "{args:?}");
    }
    for format in ["text", "json"] {
        let args = [path, "--entity", component, "--format", format];
        let asm = cli_output(&[
            "explain",
            "--inspect",
            args[0],
            args[1],
            args[2],
            args[3],
            args[4],
        ]);
        let explain = cli_output(&["explain", args[0], args[1], args[2], args[3], args[4]]);
        assert_eq!(asm, explain, "ASM/explain {format} parity");
    }
}

#[test]
fn phase_h_freezes_authorities_schemas_and_no_discovery_contract() {
    let path = "fixtures/0065-component-runtime/input/RuntimeComponents.tsx";
    let model = fixture_model(path);
    assert!(validate_application_semantic_model(&model).is_empty());

    let definition = model
        .components
        .iter()
        .find(|component| component.element_name.as_deref() == Some("x-runtime-page"))
        .unwrap();
    let invocation = model.component_invocations.values().next().unwrap();
    let instance = model
        .component_instance_plan
        .instances
        .values()
        .next()
        .unwrap();
    assert!(definition.id.as_str().contains("/component:"));
    assert!(invocation.id.as_str().contains("/component-invocation:"));
    assert!(instance.id.as_str().starts_with("root:"));
    assert_ne!(definition.id.as_str(), invocation.id.as_str());
    assert_ne!(definition.id.as_str(), instance.id.as_str());

    let registry = build_runtime_component_registry(&model, &model.component_ir_optimization);
    assert_eq!(
        registry.schema_contract_version,
        RUNTIME_COMPONENT_REGISTRY_SCHEMA_CONTRACT_VERSION
    );
    assert!(registry.instances.len() >= 6);
    assert_eq!(
        registry
            .instances
            .iter()
            .map(|record| &record.instance)
            .collect::<BTreeSet<_>>()
            .len(),
        registry.instances.len()
    );
    assert!(registry
        .slot_bindings
        .iter()
        .all(|binding| binding.content_owner_instance == binding.caller_instance));
    assert!(registry
        .instance_context_bindings
        .iter()
        .all(
            |binding| binding.selected_source.to_string() != binding.consumer_instance.to_string()
        ));

    let component_artifact =
        build_runtime_component_artifact(&model, &model.component_ir_optimization);
    assert_eq!(SEMANTIC_GRAPH_SCHEMA_VERSION, 6);
    assert_eq!(RUNTIME_COMPONENT_ARTIFACT_SCHEMA_VERSION, 10);
    assert_eq!(RUNTIME_CONTEXT_ARTIFACT_SCHEMA_VERSION, 2);
    assert_eq!(RESUME_MANIFEST_SCHEMA_VERSION, 6);
    assert_eq!(TEMPLATE_MANIFEST_SCHEMA_VERSION, 5);
    assert_eq!(component_artifact.schema_version, 10);
    assert!(validate_runtime_component_artifact(&component_artifact).is_ok());
    assert_eq!(build_semantic_graph(&model).schema_version, 6);
    assert_eq!(build_resume_manifest(&model).schema_version, 6);
    assert_eq!(build_template_manifest_from_asm(&model).schema_version, 5);

    for (args, expected_status, expected_schema) in [
        (vec!["check", path, "--format", "json"], Some(1), 6),
        (
            vec!["explain", "--inspect", path, "--format", "json"],
            Some(0),
            12,
        ),
    ] {
        let (status, stdout, stderr) = cli_result(&args);
        assert_eq!(
            status,
            expected_status,
            "{}",
            String::from_utf8_lossy(&stderr)
        );
        let document: serde_json::Value = serde_json::from_slice(&stdout).unwrap();
        assert_eq!(document["schema_version"], expected_schema);
    }

    let runtime = generate_runtime_stub();
    assert!(runtime.contains("SUPPORTED_COMPONENT_ARTIFACT_SCHEMA_VERSION = 10"));
    assert!(!runtime.contains("__EZ_COMPONENT_SCHEMA_VERSION__"));
    for forbidden in [
        "resolveComponent",
        "componentByTag",
        "componentTag",
        "resolveSlot",
        "slotByName",
        "findProvider",
        "providerSearch",
        "componentAncestors",
        "reconstructComponent",
        "virtualDom",
        "virtualDOM",
    ] {
        assert!(!runtime.contains(forbidden), "runtime contains {forbidden}");
    }
    let component_tables = runtime.find("store.componentInstances = new Map").unwrap();
    let context_execution = runtime.rfind("executeInitialContext(store);").unwrap();
    let effect_execution = runtime.rfind("executeInitialEffects(store);").unwrap();
    assert!(component_tables < context_execution && context_execution < effect_execution);
    assert!(runtime.contains("store.componentRegions = new Map"));

    let core = repo_root().join("crates/presolve_compiler/src");
    for (needle, authorities) in [
        (
            "ComponentInstanceId::for_",
            &[
                "component_instance.rs",
                "resource.rs",
                "resume_identity.rs",
                "resume_liveness.rs",
                "slot_projection.rs",
            ][..],
        ),
        (
            "ComponentStructuralRegionId::for_",
            &["component_instance.rs", "resume_restore.rs"][..],
        ),
        (
            "SlotBindingId::for_instance",
            &["slot_binding.rs", "slot_projection.rs"][..],
        ),
        ("ProviderInstanceId::new", &["instance_context.rs"][..]),
        ("ConsumerInstanceId::new", &["instance_context.rs"][..]),
    ] {
        let owners = fs::read_dir(&core)
            .unwrap()
            .filter_map(Result::ok)
            .filter(|entry| {
                entry
                    .path()
                    .extension()
                    .is_some_and(|extension| extension == "rs")
            })
            .filter(|entry| fs::read_to_string(entry.path()).unwrap().contains(needle))
            .map(|entry| entry.file_name().to_string_lossy().into_owned())
            .collect::<BTreeSet<_>>();
        assert_eq!(
            owners,
            authorities
                .iter()
                .map(|authority| (*authority).to_string())
                .collect(),
            "{needle}"
        );
    }

    for file in [
        "runtime_component.rs",
        "runtime_component_artifact.rs",
        "runtime_codegen.rs",
        "resume_plan.rs",
    ] {
        let source = fs::read_to_string(core.join(file)).unwrap();
        for forbidden in [
            "resolve_invocation_target",
            "authored_symbol",
            "requested_slot_name",
            "tag_name",
            "element_name",
        ] {
            assert!(!source.contains(forbidden), "{file} contains {forbidden}");
        }
    }
    let diagnostics_source = fs::read_to_string(core.join("component_diagnostics.rs")).unwrap();
    let diagnostics = diagnostics_source.split("#[cfg(test)]").next().unwrap();
    assert!(!diagnostics.contains("presolve_parser"));
    assert!(!diagnostics.contains("parse_file"));
}

#[test]
fn every_component_diagnostic_fixture_matches_its_frozen_contract() {
    for contract in COMPONENT_DIAGNOSTIC_CONTRACTS {
        let path = format!(
            "fixtures/0066-component-diagnostics/input/{}.tsx",
            contract.code
        );
        let mut model = fixture_model(&path);
        mutate_authoritative_product(contract.code, &mut model);
        let first = collect_component_diagnostics(&model);
        let second = collect_component_diagnostics(&model);
        assert_eq!(first, second, "{} ordering", contract.code);
        let matches = first
            .iter()
            .filter(|diagnostic| diagnostic.code == contract.code)
            .collect::<Vec<_>>();
        assert_eq!(matches.len(), 1, "{} dedup: {first:#?}", contract.code);
        let diagnostic = matches[0];
        assert_eq!(diagnostic.message, contract.message);
        assert!(diagnostic
            .provenance
            .as_ref()
            .is_some_and(|provenance| provenance.path == Path::new(&path)));
        assert_contract_identities(diagnostic, contract.identities);
        assert!(diagnostic
            .secondary_labels
            .iter()
            .enumerate()
            .all(|(index, label)| diagnostic.secondary_labels[index + 1..]
                .iter()
                .all(|other| other != label)));
        assert!(diagnostic
            .secondary_labels
            .iter()
            .all(|label| { Some(&label.provenance) != diagnostic.provenance.as_ref() }));
    }
}

fn assert_contract_identities(
    diagnostic: &presolve_compiler::ComponentDiagnostic,
    identities: &str,
) {
    for identity in identities.split(", ") {
        let present = match identity {
            "component_id" => diagnostic.component_id.is_some(),
            "invocation_id" => diagnostic.invocation_id.is_some(),
            "slot_id" => diagnostic.slot_id.is_some(),
            "component_instance_id" => diagnostic.component_instance_id.is_some(),
            "slot_binding_id" => diagnostic.slot_binding_id.is_some(),
            "structural_region_id" => diagnostic.structural_region_id.is_some(),
            "provider_instance_id" => diagnostic.provider_instance_id.is_some(),
            "consumer_instance_id" => diagnostic.consumer_instance_id.is_some(),
            "none" => true,
            other => panic!("unknown contract identity {other}"),
        };
        assert!(present, "{} missing {identity}", diagnostic.code);
    }
}

fn mutate_authoritative_product(
    code: &str,
    model: &mut presolve_compiler::ApplicationSemanticModel,
) {
    match code {
        "PSC1078" => {
            model
                .slot_bindings
                .bindings
                .values_mut()
                .next()
                .unwrap()
                .status = SlotBindingStatus::InvalidOwnership;
        }
        "PSC1079" => {
            let record = model
                .composition_types
                .slot_bindings
                .values_mut()
                .next()
                .unwrap();
            record.type_compatibility = CompositionCompatibility::Incompatible;
            record.overall = CompositionCompatibility::Incompatible;
        }
        "PSC1080" => {
            let target = model
                .components
                .iter()
                .find(|component| component.element_name.as_deref() == Some("x-card"))
                .unwrap()
                .id
                .clone();
            let invocation_id = model
                .component_instance_plan
                .blocked
                .values()
                .next()
                .unwrap()
                .invocation
                .clone();
            let invocation = model.component_invocations.get_mut(&invocation_id).unwrap();
            invocation.status = ComponentInvocationResolutionStatus::Resolved;
            invocation.target_component = Some(target.clone());
            model
                .component_instance_plan
                .blocked
                .values_mut()
                .next()
                .unwrap()
                .target_component = Some(target);
        }
        "PSC1082" => {
            let id = model
                .component_instance_plan
                .instances
                .values()
                .find(|instance| instance.status == ComponentInstanceStatus::StructuralTemplate)
                .unwrap()
                .id
                .clone();
            let instance = model.component_instance_plan.instances.remove(&id).unwrap();
            model.component_instance_plan.blocked.insert(
                id.clone(),
                BlockedComponentInstancePlan {
                    id,
                    invocation: instance.invocation.unwrap(),
                    parent_instance: instance.parent_instance.unwrap(),
                    owner_root: instance.owner_root,
                    target_component: Some(instance.component),
                    structural_region: instance.structural_region,
                    depth: instance.depth,
                    reason: BlockedComponentInstanceReason::InvalidParentPlan,
                    provenance: instance.provenance,
                },
            );
        }
        "PSC1083" => {
            model
                .slot_bindings
                .bindings
                .values_mut()
                .next()
                .unwrap()
                .status = SlotBindingStatus::BlockedInvocation;
        }
        _ => {}
    }
}
