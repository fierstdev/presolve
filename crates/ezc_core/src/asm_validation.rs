use crate::application_semantic_model::ApplicationSemanticModel;
use crate::semantic_id::SemanticOwner;
use crate::{
    build_ordinary_template_instance_registry, build_runtime_component_artifact,
    build_template_manifest_from_asm, validate_ordinary_template_instance_registry,
    validate_runtime_component_artifact, validate_template_manifest, EffectOperationClassification,
    EffectRenderBoundary, EffectValidation, ManifestEventKind, OrdinaryTemplateIntegrityCode,
    SemanticTypeId, EFFECT_CAPABILITY_REGISTRY, TEMPLATE_MANIFEST_SCHEMA_VERSION,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AsmValidationDiagnostic {
    pub code: String,
    pub message: String,
}

#[must_use]
#[allow(clippy::too_many_lines)]
pub fn validate_application_semantic_model(
    model: &ApplicationSemanticModel,
) -> Vec<AsmValidationDiagnostic> {
    let mut diagnostics = Vec::new();

    for (id, owner) in &model.ownership {
        if model.entity(id).is_none() {
            diagnostics.push(AsmValidationDiagnostic {
                code: "EZASM1001".to_string(),
                message: format!("ownership references missing semantic entity `{id}`"),
            });
        }
        if !model.provenance.contains_key(id) {
            diagnostics.push(AsmValidationDiagnostic {
                code: "EZASM1002".to_string(),
                message: format!("semantic entity `{id}` is missing source provenance"),
            });
        }
        if let SemanticOwner::Entity(owner_id) = owner {
            if model.entity(owner_id).is_none() {
                diagnostics.push(AsmValidationDiagnostic {
                    code: "EZASM1003".to_string(),
                    message: format!("semantic entity `{id}` has missing owner `{owner_id}`"),
                });
            }
        }
    }

    for id in model.provenance.keys() {
        if !model.ownership.contains_key(id) {
            diagnostics.push(AsmValidationDiagnostic {
                code: "EZASM1004".to_string(),
                message: format!("provenance references unowned semantic entity `{id}`"),
            });
        }
    }

    for reference in &model.references {
        if model.entity(&reference.source).is_none() || model.entity(&reference.target).is_none() {
            diagnostics.push(AsmValidationDiagnostic {
                code: "EZASM1005".to_string(),
                message: format!(
                    "reference from `{}` to `{}` has a missing endpoint",
                    reference.source, reference.target
                ),
            });
        }
        let source_provenance_matches = model.provenance(&reference.source)
            == Some(&reference.provenance)
            || model.form_field_bindings.values().any(|binding| {
                binding.id.as_semantic_id() == &reference.source
                    && binding.expression_provenance == reference.provenance
            })
            || model.validation_rules.values().any(|rule| {
                rule.id.as_semantic_id() == &reference.source
                    && rule.argument_provenance.as_ref() == Some(&reference.provenance)
            })
            || model
                .consumer_for_semantic_id(&reference.source)
                .is_some_and(|consumer| {
                    consumer.context_designator.provenance == reference.provenance
                })
            || model
                .expression_graph
                .nodes_for(&reference.source)
                .iter()
                .any(|node| node.provenance == reference.provenance);
        if !source_provenance_matches {
            diagnostics.push(AsmValidationDiagnostic {
                code: "EZASM1006".to_string(),
                message: format!(
                    "reference source `{}` has mismatched provenance",
                    reference.source
                ),
            });
        }
    }

    validate_semantic_types(model, &mut diagnostics);
    validate_form_field_bindings(model, &mut diagnostics);
    validate_form_ownership(model, &mut diagnostics);
    validate_form_validation(model, &mut diagnostics);
    validate_form_tracking(model, &mut diagnostics);
    validate_form_submissions(model, &mut diagnostics);
    validate_form_serialization(model, &mut diagnostics);
    validate_form_reset(model, &mut diagnostics);
    validate_contexts(model, &mut diagnostics);
    validate_providers(model, &mut diagnostics);
    validate_consumers(model, &mut diagnostics);
    validate_context_resolution(model, &mut diagnostics);
    validate_context_typing(model, &mut diagnostics);
    validate_context_ownership(model, &mut diagnostics);
    validate_context_dependency(model, &mut diagnostics);
    validate_context_lifetime(model, &mut diagnostics);
    validate_context_evaluation(model, &mut diagnostics);
    validate_component_instance_scope(model, &mut diagnostics);
    validate_component_composition(model, &mut diagnostics);
    validate_component_initialization(model, &mut diagnostics);
    validate_component_ir(model, &mut diagnostics);
    validate_optimized_component_ir(model, &mut diagnostics);
    validate_instance_context(model, &mut diagnostics);
    validate_slot_bindings(model, &mut diagnostics);
    validate_composition_types(model, &mut diagnostics);
    validate_effect_statement_types(model, &mut diagnostics);
    validate_effect_execution_plan(model, &mut diagnostics);
    validate_component_diagnostic_metadata(model, &mut diagnostics);
    validate_template_action_bindings(model, &mut diagnostics);
    validate_ordinary_template_instance_projection(model, &mut diagnostics);

    diagnostics
}

fn validate_ordinary_template_instance_projection(
    model: &ApplicationSemanticModel,
    diagnostics: &mut Vec<AsmValidationDiagnostic>,
) {
    let registry = build_ordinary_template_instance_registry(model);
    let artifact = build_runtime_component_artifact(model, &model.component_ir_optimization);
    let manifest = build_template_manifest_from_asm(model);
    if validate_ordinary_template_instance_registry(model, &registry).is_err() {
        diagnostics.push(AsmValidationDiagnostic {
            code: OrdinaryTemplateIntegrityCode::StaleRegistry
                .as_str()
                .to_string(),
            message: "ordinary template registry drifted from canonical products".to_string(),
        });
    }
    if validate_runtime_component_artifact(&artifact).is_err()
        || validate_template_manifest(&manifest).is_err()
    {
        diagnostics.push(AsmValidationDiagnostic {
            code: OrdinaryTemplateIntegrityCode::ArtifactManifestDrift
                .as_str()
                .to_string(),
            message: "ordinary template artifact or manifest projection is invalid".to_string(),
        });
    }
}

fn validate_form_validation(
    model: &ApplicationSemanticModel,
    diagnostics: &mut Vec<AsmValidationDiagnostic>,
) {
    let expected_products =
        crate::collect_validation_products(&model.components, &model.forms, &model.form_fields);
    if model.validation_rule_candidates != expected_products.candidates
        || model.validation_rules != expected_products.rules
    {
        diagnostics.push(AsmValidationDiagnostic {
            code: "EZASM1241".to_string(),
            message: "validation products do not match canonical I6 lowering".to_string(),
        });
    }
    let validation = crate::validate_validation_graph(
        &model.validation_graph,
        &model.component_instance_plan.roots,
        &model.form_ownership,
        &model.forms,
        &model.form_fields,
        &model.validation_rules,
        &model.validation_rule_candidates,
    );
    diagnostics.extend(
        validation
            .diagnostics
            .iter()
            .map(|diagnostic| AsmValidationDiagnostic {
                code: diagnostic.code.clone(),
                message: diagnostic.message.clone(),
            }),
    );
    if model.validation_graph.validation != validation {
        diagnostics.push(AsmValidationDiagnostic {
            code: "EZASM1240".to_string(),
            message: "validation graph retained stale validation facts".to_string(),
        });
    }
    let expected_plans = crate::collect_validation_dependency_plans(
        &model.forms,
        &model.form_fields,
        &model.validation_rules,
        &model.form_ownership,
        &model.validation_graph,
    );
    if model.validation_dependency_plans != expected_plans {
        diagnostics.push(AsmValidationDiagnostic {
            code: "EZASM1272".to_string(),
            message: "validation dependency plans do not match canonical I7 planning".to_string(),
        });
    }
    let planning = crate::validate_validation_dependency_plans(
        &model.validation_dependency_plans,
        &model.forms,
        &model.form_fields,
        &model.validation_rules,
        &model.form_ownership,
        &model.validation_graph,
    );
    diagnostics.extend(
        planning
            .diagnostics
            .iter()
            .map(|diagnostic| AsmValidationDiagnostic {
                code: diagnostic.code.clone(),
                message: diagnostic.message.clone(),
            }),
    );
    if model.validation_dependency_plans.validation != planning {
        diagnostics.push(AsmValidationDiagnostic {
            code: "EZASM1271".to_string(),
            message: "validation dependency plans retained stale validation facts".to_string(),
        });
    }
}

fn validate_form_tracking(
    model: &ApplicationSemanticModel,
    diagnostics: &mut Vec<AsmValidationDiagnostic>,
) {
    let expected = crate::collect_form_tracking_products(
        &model.forms,
        &model.form_fields,
        &model.form_field_bindings,
        &model.form_ownership,
    );
    if model.form_tracking != expected {
        diagnostics.push(AsmValidationDiagnostic {
            code: "EZASM1285".to_string(),
            message: "I8 form tracking products do not match canonical declaration planning"
                .to_string(),
        });
    }
    let dirty = crate::validate_dirty_tracking_graph(
        &model.form_tracking.dirty,
        &model.forms,
        &model.form_fields,
        &model.form_field_bindings,
        &model.form_ownership,
    );
    let touched = crate::validate_touched_tracking_graph(
        &model.form_tracking.touched,
        &model.forms,
        &model.form_fields,
        &model.form_field_bindings,
        &model.form_ownership,
    );
    diagnostics.extend(
        dirty
            .diagnostics
            .iter()
            .chain(touched.diagnostics.iter())
            .map(|diagnostic| AsmValidationDiagnostic {
                code: diagnostic.code.clone(),
                message: diagnostic.message.clone(),
            }),
    );
}

fn validate_form_submissions(
    model: &ApplicationSemanticModel,
    diagnostics: &mut Vec<AsmValidationDiagnostic>,
) {
    let expected = crate::collect_submission_products(
        &model.components,
        &model.forms,
        &model.form_fields,
        &model.validation_rules,
        &model.effect_trigger_plan,
    );
    if model.submissions != expected {
        diagnostics.push(AsmValidationDiagnostic {
            code: "EZASM1286".to_string(),
            message: "I9 submission products do not match canonical declaration planning"
                .to_string(),
        });
    }
}

fn validate_form_serialization(
    model: &ApplicationSemanticModel,
    diagnostics: &mut Vec<AsmValidationDiagnostic>,
) {
    let expected = crate::collect_serialization_products(
        &model.components,
        &model.forms,
        &model.form_fields,
        &model.submissions.plans,
    );
    if model.serialization != expected {
        diagnostics.push(AsmValidationDiagnostic {
            code: "EZASM1287".to_string(),
            message: "I10 serialization products do not match canonical declaration planning"
                .to_string(),
        });
    }
}

fn validate_form_reset(
    model: &ApplicationSemanticModel,
    diagnostics: &mut Vec<AsmValidationDiagnostic>,
) {
    let expected = crate::collect_reset_products(
        &model.forms,
        &model.form_fields,
        &model.form_field_bindings,
        &model.form_tracking,
    );
    if model.reset != expected {
        diagnostics.push(AsmValidationDiagnostic {
            code: "EZASM1288".to_string(),
            message: "I11 reset products do not match canonical declaration planning".to_string(),
        });
    }
}

fn validate_form_ownership(
    model: &ApplicationSemanticModel,
    diagnostics: &mut Vec<AsmValidationDiagnostic>,
) {
    let validation = crate::validate_form_ownership_graph(&model.form_ownership, model);
    diagnostics.extend(
        validation
            .diagnostics
            .iter()
            .map(|diagnostic| AsmValidationDiagnostic {
                code: diagnostic.code.clone(),
                message: diagnostic.message.clone(),
            }),
    );
    if model.form_ownership.validation != validation {
        diagnostics.push(AsmValidationDiagnostic {
            code: "EZASM1220".to_string(),
            message: "Form ownership graph retained stale validation facts".to_string(),
        });
    }
}

fn validate_form_field_bindings(
    model: &ApplicationSemanticModel,
    diagnostics: &mut Vec<AsmValidationDiagnostic>,
) {
    let expected = crate::collect_form_field_binding_products(
        &model.components,
        &model.templates,
        &model.forms,
        &model.form_fields,
        &model.form_field_declaration_candidates,
    );
    if model.form_field_binding_candidates != expected.candidates
        || model.form_field_bindings != expected.bindings
    {
        diagnostics.push(AsmValidationDiagnostic {
            code: "EZASM1202".to_string(),
            message: "Form Field bindings do not match canonical I3/template lowering".to_string(),
        });
    }
}

fn validate_component_ir(
    model: &ApplicationSemanticModel,
    diagnostics: &mut Vec<AsmValidationDiagnostic>,
) {
    if !crate::validate_component_ir(model, &model.component_ir).is_empty() {
        diagnostics.push(AsmValidationDiagnostic {
            code: "EZASM1199".to_string(),
            message: "component IR does not match canonical H10 operations".to_string(),
        });
    }
}

fn validate_optimized_component_ir(
    model: &ApplicationSemanticModel,
    diagnostics: &mut Vec<AsmValidationDiagnostic>,
) {
    if !crate::validate_optimized_component_ir(&model.component_ir_optimization).is_empty() {
        diagnostics.push(AsmValidationDiagnostic {
            code: "EZASM1200".to_string(),
            message: "optimized component IR does not match the canonical H12 projection"
                .to_string(),
        });
    }
}

fn validate_component_initialization(
    model: &ApplicationSemanticModel,
    diagnostics: &mut Vec<AsmValidationDiagnostic>,
) {
    let expected = crate::plan_component_initialization(
        &model.component_instance_plan,
        &model.slot_bindings,
        &model.composition_types,
        &model.instance_context,
    );
    if model.component_initialization != expected {
        diagnostics.push(AsmValidationDiagnostic {
            code: "EZASM1198".to_string(),
            message: "component initialization plan does not match canonical H4/H6/H7/H8 products"
                .to_string(),
        });
    }
}

fn validate_component_composition(
    model: &ApplicationSemanticModel,
    diagnostics: &mut Vec<AsmValidationDiagnostic>,
) {
    let components = model
        .components
        .iter()
        .map(|component| component.id.clone())
        .collect();
    let expected = crate::analyze_component_composition(
        &components,
        &model.component_invocations,
        &model.component_instance_plan,
    );
    if model.component_composition != expected {
        diagnostics.push(AsmValidationDiagnostic {
            code: "EZASM1197".to_string(),
            message: "component composition cycles do not match canonical resolved invocations"
                .to_string(),
        });
    }
}

fn validate_composition_types(
    model: &ApplicationSemanticModel,
    diagnostics: &mut Vec<AsmValidationDiagnostic>,
) {
    let component_ids = model
        .components
        .iter()
        .map(|component| component.id.clone())
        .collect();
    let expected = crate::collect_composition_type_products(
        &component_ids,
        &model.component_invocations,
        &model.slot_bindings,
        &model.slots,
        &model.slot_content_fragments,
        &model.slot_outlets,
        &model.instance_context,
        &model.context_binding_types,
        &model.context_types,
        &model.provider_types,
        &model.context_lifetime,
        &model.ownership,
        &model.references,
    );
    if model.composition_types != expected {
        diagnostics.push(AsmValidationDiagnostic {
            code: "EZASM1196".to_string(),
            message: "composition typing does not match canonical H2/H6/H7 products".to_string(),
        });
    }
}

fn validate_slot_bindings(
    model: &ApplicationSemanticModel,
    diagnostics: &mut Vec<AsmValidationDiagnostic>,
) {
    let expected = crate::collect_slot_bindings(
        &model.component_instance_plan,
        &model.component_invocations,
        &model.slots,
        &model.slot_content_fragments,
        &model.slot_outlets,
    );
    if model.slot_bindings != expected {
        diagnostics.push(AsmValidationDiagnostic {
            code: "EZASM1195".to_string(),
            message: "Slot binding registry does not match canonical H3 facts and H4 instances"
                .to_string(),
        });
    }
}

fn validate_instance_context(
    model: &ApplicationSemanticModel,
    diagnostics: &mut Vec<AsmValidationDiagnostic>,
) {
    let expected = crate::collect_instance_context_registry(
        &model.component_instance_scope,
        &model.contexts,
        &model.providers,
        &model.consumers,
    );
    if model.instance_context != expected {
        diagnostics.push(AsmValidationDiagnostic {
            code: "EZASM1194".to_string(),
            message:
                "instance Context registry does not match canonical declarations and H5 ancestry"
                    .to_string(),
        });
    }
}

fn validate_component_instance_scope(
    model: &ApplicationSemanticModel,
    diagnostics: &mut Vec<AsmValidationDiagnostic>,
) {
    let expected = crate::build_component_instance_scope_graph(&model.component_instance_plan);
    if model.component_instance_scope != expected {
        diagnostics.push(AsmValidationDiagnostic {
            code: "EZASM1192".to_string(),
            message: "component instance scope graph does not match the canonical H4 plan"
                .to_string(),
        });
    }
    diagnostics.extend(
        crate::validate_component_instance_scope_graph(&model.component_instance_scope)
            .into_iter()
            .map(|diagnostic| AsmValidationDiagnostic {
                code: "EZASM1193".to_string(),
                message: format!(
                    "component instance scope {:?} at `{}`{}",
                    diagnostic.violation,
                    diagnostic.instance,
                    diagnostic.related.map_or_else(String::new, |related| {
                        format!(" related to `{related}`")
                    })
                ),
            }),
    );
}

fn validate_context_dependency(
    model: &ApplicationSemanticModel,
    diagnostics: &mut Vec<AsmValidationDiagnostic>,
) {
    let expected = crate::collect_context_dependency_graph(
        &model.components,
        &model.contexts,
        &model.providers,
        &model.consumers,
        &model.context_resolutions,
        &model.context_types,
        &model.provider_types,
        &model.context_binding_types,
        &model.computed_values,
        &model.expression_graph,
    );
    if model.context_dependency != expected {
        diagnostics.push(AsmValidationDiagnostic {
            code: "EZASM1189".to_string(),
            message: "Context dependency graph does not match canonical ASM products".to_string(),
        });
    }
}

fn validate_context_lifetime(
    model: &ApplicationSemanticModel,
    diagnostics: &mut Vec<AsmValidationDiagnostic>,
) {
    let expected = crate::collect_context_lifetime_analysis(
        &model.components,
        &model.contexts,
        &model.providers,
        &model.consumers,
        &model.computed_values,
        &model.context_ownership,
        &model.component_scope,
        &model.context_resolutions,
        &model.context_dependency,
        &model.provenance,
    );
    if model.context_lifetime != expected {
        diagnostics.push(AsmValidationDiagnostic {
            code: "EZASM1190".to_string(),
            message: "Context lifetime analysis does not match canonical ASM products".to_string(),
        });
    }
}

fn validate_context_evaluation(
    model: &ApplicationSemanticModel,
    diagnostics: &mut Vec<AsmValidationDiagnostic>,
) {
    let expected = crate::collect_context_evaluation_plan(
        &model.contexts,
        &model.providers,
        &model.context_resolutions,
        &model.context_types,
        &model.provider_types,
        &model.context_binding_types,
        &model.context_lifetime,
        &model.context_dependency,
        &model.computed_evaluation_plan,
        &model.component_scope,
    );
    if model.context_evaluation != expected {
        diagnostics.push(AsmValidationDiagnostic {
            code: "EZASM1191".to_string(),
            message: "Context evaluation plan does not match canonical ASM products".to_string(),
        });
    }
}

#[allow(clippy::too_many_lines)]
fn validate_context_ownership(
    model: &ApplicationSemanticModel,
    diagnostics: &mut Vec<AsmValidationDiagnostic>,
) {
    let graph = &model.context_ownership;
    if graph
        .nodes
        .iter()
        .any(|node| matches!(node.id, crate::ContextOwnershipNodeId::Component(ref id) if model.component(id).is_none()))
    {
        diagnostics.push(AsmValidationDiagnostic {
            code: "EZASM1182".to_string(),
            message: "Context ownership graph references a missing component".to_string(),
        });
    }
    if graph.nodes.len()
        != model.contexts.len()
            + model.providers.len()
            + model.consumers.len()
            + graph
                .nodes
                .iter()
                .filter(|node| matches!(node.kind, crate::ContextOwnershipNodeKind::Component))
                .count()
    {
        diagnostics.push(AsmValidationDiagnostic {
            code: "EZASM1183".to_string(),
            message: "Context ownership graph has an invalid node domain".to_string(),
        });
    }
    for context in model.contexts.values() {
        let expected_owner = context.owner.entity_id();
        let default = context.default_expression.as_ref();
        if graph.owner_of_context(&context.id) != expected_owner
            || graph.context_default_expression(&context.id) != default
            || !expected_owner.is_some_and(|owner| {
                exactly_one_ownership_edge(
                    graph,
                    &crate::ContextOwnershipOwnerId::Component(owner.clone()),
                    &crate::ContextOwnershipTargetId::Context(context.id.clone()),
                    crate::ContextOwnershipEdgeKind::ComponentOwnsContext,
                )
            })
        {
            diagnostics.push(AsmValidationDiagnostic {
                code: "EZASM1184".to_string(),
                message: format!("Context `{}` has an invalid ownership record", context.id),
            });
        }
        if let Some(default) = default {
            let default_provenance = model
                .expression_graph
                .node(default)
                .map(|node| &node.provenance);
            let default_edge = graph.edges.iter().find(|edge| {
                edge.owner == crate::ContextOwnershipOwnerId::Context(context.id.clone())
                    && edge.owned == crate::ContextOwnershipTargetId::Expression(default.clone())
                    && edge.kind == crate::ContextOwnershipEdgeKind::ContextOwnsDefaultExpression
            });
            if default_edge.is_none_or(|edge| Some(&edge.provenance) != default_provenance) {
                diagnostics.push(AsmValidationDiagnostic {
                    code: "EZASM1188".to_string(),
                    message: format!(
                        "Context `{}` has an invalid default ownership edge",
                        context.id
                    ),
                });
            }
        }
    }
    for provider in model.providers.values() {
        if graph.owner_of_provider(&provider.id) != provider.owner.entity_id()
            || !provider.owner.entity_id().is_some_and(|owner| {
                exactly_one_ownership_edge(
                    graph,
                    &crate::ContextOwnershipOwnerId::Component(owner.clone()),
                    &crate::ContextOwnershipTargetId::Provider(provider.id.clone()),
                    crate::ContextOwnershipEdgeKind::ComponentOwnsProvider,
                )
            })
        {
            diagnostics.push(AsmValidationDiagnostic {
                code: "EZASM1185".to_string(),
                message: format!("Provider `{}` has an invalid ownership record", provider.id),
            });
        }
    }
    for consumer in model.consumers.values() {
        if graph.owner_of_consumer(&consumer.id) != consumer.owner.entity_id()
            || !consumer.owner.entity_id().is_some_and(|owner| {
                exactly_one_ownership_edge(
                    graph,
                    &crate::ContextOwnershipOwnerId::Component(owner.clone()),
                    &crate::ContextOwnershipTargetId::Consumer(consumer.id.clone()),
                    crate::ContextOwnershipEdgeKind::ComponentOwnsConsumer,
                )
            })
        {
            diagnostics.push(AsmValidationDiagnostic {
                code: "EZASM1186".to_string(),
                message: format!("Consumer `{}` has an invalid ownership record", consumer.id),
            });
        }
    }
    if graph.edges.windows(2).any(|pair| {
        (&pair[0].owner, pair[0].kind, &pair[0].owned)
            > (&pair[1].owner, pair[1].kind, &pair[1].owned)
    }) || graph.edges.iter().any(|edge| {
        !matches!(
            (&edge.owner, &edge.owned, edge.kind),
            (
                crate::ContextOwnershipOwnerId::Component(_),
                crate::ContextOwnershipTargetId::Context(_),
                crate::ContextOwnershipEdgeKind::ComponentOwnsContext
            ) | (
                crate::ContextOwnershipOwnerId::Component(_),
                crate::ContextOwnershipTargetId::Provider(_),
                crate::ContextOwnershipEdgeKind::ComponentOwnsProvider
            ) | (
                crate::ContextOwnershipOwnerId::Component(_),
                crate::ContextOwnershipTargetId::Consumer(_),
                crate::ContextOwnershipEdgeKind::ComponentOwnsConsumer
            ) | (
                crate::ContextOwnershipOwnerId::Context(_),
                crate::ContextOwnershipTargetId::Expression(_),
                crate::ContextOwnershipEdgeKind::ContextOwnsDefaultExpression
            )
        )
    }) {
        diagnostics.push(AsmValidationDiagnostic {
            code: "EZASM1187".to_string(),
            message: "Context ownership graph has an invalid edge domain or order".to_string(),
        });
    }
}

fn exactly_one_ownership_edge(
    graph: &crate::ContextOwnershipGraph,
    source: &crate::ContextOwnershipOwnerId,
    target: &crate::ContextOwnershipTargetId,
    kind: crate::ContextOwnershipEdgeKind,
) -> bool {
    graph
        .edges
        .iter()
        .filter(|edge| &edge.owner == source && &edge.owned == target && edge.kind == kind)
        .count()
        == 1
}

fn validate_context_typing(
    model: &ApplicationSemanticModel,
    diagnostics: &mut Vec<AsmValidationDiagnostic>,
) {
    if model.context_types.len() != model.contexts.len()
        || model.provider_types.len() != model.providers.len()
        || model.consumer_types.len() != model.consumers.len()
        || model.context_binding_types.len() != model.consumers.len()
    {
        diagnostics.push(AsmValidationDiagnostic {
            code: "EZASM1176".to_string(),
            message: "Context typing records do not exactly cover canonical entities".to_string(),
        });
    }
    for context in model.contexts.values() {
        if model.context_type(&context.id).is_none_or(|record| {
            record.declared_type != context.declared_type_id
                || record.normalized_type != context.declared_type_id
        }) {
            diagnostics.push(AsmValidationDiagnostic {
                code: "EZASM1177".to_string(),
                message: format!("Context `{}` has an invalid type record", context.id),
            });
        }
    }
    for provider in model.providers.values() {
        if model.provider_type(&provider.id).is_none_or(|record| {
            record.declared_type != provider.declared_type_id
                || record.inferred_value_type == provider.declared_type_id
                || record.context != Some(provider.context.clone())
        }) {
            diagnostics.push(AsmValidationDiagnostic {
                code: "EZASM1178".to_string(),
                message: format!("Provider `{}` has an invalid type record", provider.id),
            });
        }
    }
    for consumer in model.consumers.values() {
        let Some(binding) = model.context_binding_type(&consumer.id) else {
            continue;
        };
        if model.consumer_type(&consumer.id).is_none_or(|record| {
            record.requested_type != consumer.requested_type_id
                || record.context != consumer.context().cloned()
        }) || model
            .context_resolution(&consumer.id)
            .is_none_or(|resolution| binding.resolution != resolution.result)
        {
            diagnostics.push(AsmValidationDiagnostic {
                code: "EZASM1179".to_string(),
                message: format!("Consumer `{}` has an invalid type record", consumer.id),
            });
        }
        if let crate::ContextResolutionResult::Provider { provider, .. } = &binding.resolution {
            if binding.provider.as_ref() != Some(provider) {
                diagnostics.push(AsmValidationDiagnostic {
                    code: "EZASM1180".to_string(),
                    message: format!("Consumer `{}` does not retain its G4 Provider", consumer.id),
                });
            }
        }
        if binding.overall == crate::ContextBindingCompatibility::Compatible
            && (binding.source_to_context != crate::CompatibilityStatus::Compatible
                || binding.context_to_consumer != crate::CompatibilityStatus::Compatible
                || binding.boundary_compatibility != crate::CompatibilityStatus::Compatible
                || binding.serialization != crate::ContextSerializationCompatibility::Serializable)
        {
            diagnostics.push(AsmValidationDiagnostic {
                code: "EZASM1181".to_string(),
                message: format!(
                    "Consumer `{}` has a permissively compatible binding",
                    consumer.id
                ),
            });
        }
    }
}

#[allow(clippy::too_many_lines)]
fn validate_context_resolution(
    model: &ApplicationSemanticModel,
    diagnostics: &mut Vec<AsmValidationDiagnostic>,
) {
    for scope_diagnostic in model.component_scope.diagnostics() {
        diagnostics.push(AsmValidationDiagnostic {
            code: "EZASM1165".to_string(),
            message: scope_diagnostic.message,
        });
    }
    if model.context_resolutions.len() != model.consumers.len()
        || model
            .context_resolutions
            .keys()
            .any(|consumer| !model.consumers.contains_key(consumer))
    {
        diagnostics.push(AsmValidationDiagnostic {
            code: "EZASM1166".to_string(),
            message: "Context resolution records do not exactly cover canonical Consumers"
                .to_string(),
        });
    }

    for (consumer_id, consumer) in &model.consumers {
        let Some(resolution) = model.context_resolution(consumer_id) else {
            continue;
        };
        if resolution.consumer != *consumer_id
            || resolution.provenance != consumer.context_designator.provenance
            || resolution.context != consumer.context().cloned()
        {
            diagnostics.push(AsmValidationDiagnostic {
                code: "EZASM1167".to_string(),
                message: format!("consumer `{consumer_id}` has a non-canonical resolution record"),
            });
        }
        let expected_scopes = consumer.owner.entity_id().map_or_else(Vec::new, |owner| {
            model.component_scope.ancestor_chain(owner)
        });
        if resolution.searched_scopes != expected_scopes {
            diagnostics.push(AsmValidationDiagnostic {
                code: "EZASM1168".to_string(),
                message: format!(
                    "consumer `{consumer_id}` searched non-canonical component scopes"
                ),
            });
        }

        match &resolution.result {
            crate::ContextResolutionResult::Provider {
                provider,
                provider_owner,
                distance,
            } => {
                let provider_entity = model.provider(provider);
                let context_matches = resolution.context.as_ref().is_some_and(|context| {
                    provider_entity.is_some_and(|provider| &provider.context == context)
                });
                let owner_matches = provider_entity.and_then(|provider| provider.owner.entity_id())
                    == Some(provider_owner);
                let expected_owner = resolution.searched_scopes.get(*distance as usize);
                if !context_matches || !owner_matches || expected_owner != Some(provider_owner) {
                    diagnostics.push(AsmValidationDiagnostic {
                        code: "EZASM1169".to_string(),
                        message: format!(
                            "consumer `{consumer_id}` has an invalid Provider resolution"
                        ),
                    });
                }
                if (*distance as usize) > resolution.searched_scopes.len()
                    || resolution.searched_scopes[..*distance as usize]
                        .iter()
                        .any(|scope| {
                            !resolution_candidates(model, scope, resolution.context.as_ref())
                                .is_empty()
                        })
                    || resolution_candidates(model, provider_owner, resolution.context.as_ref())
                        .len()
                        != 1
                {
                    diagnostics.push(AsmValidationDiagnostic {
                        code: "EZASM1170".to_string(),
                        message: format!(
                            "consumer `{consumer_id}` does not select the nearest Provider"
                        ),
                    });
                }
            }
            crate::ContextResolutionResult::ContextDefault {
                context,
                expression,
            } => {
                let default_is_canonical = model.context(context).is_some_and(|context_entity| {
                    context_entity.default_expression.is_some()
                        && model.expression_root(context.as_semantic_id()) == Some(expression)
                });
                if resolution.context.as_ref() != Some(context)
                    || !default_is_canonical
                    || resolution
                        .searched_scopes
                        .iter()
                        .any(|scope| !resolution_candidates(model, scope, Some(context)).is_empty())
                {
                    diagnostics.push(AsmValidationDiagnostic {
                        code: "EZASM1171".to_string(),
                        message: format!(
                            "consumer `{consumer_id}` has an invalid Context default fallback"
                        ),
                    });
                }
            }
            crate::ContextResolutionResult::Unresolved => {
                let has_visible_provider = resolution.searched_scopes.iter().any(|scope| {
                    !resolution_candidates(model, scope, resolution.context.as_ref()).is_empty()
                });
                let has_default = resolution.context.as_ref().is_some_and(|context| {
                    model
                        .context(context)
                        .is_some_and(|entity| entity.default_expression.is_some())
                });
                if resolution.context.is_none() || has_visible_provider || has_default {
                    diagnostics.push(AsmValidationDiagnostic {
                        code: "EZASM1172".to_string(),
                        message: format!(
                            "consumer `{consumer_id}` has an invalid unresolved result"
                        ),
                    });
                }
            }
            crate::ContextResolutionResult::Ambiguous {
                providers,
                distance,
            } => {
                let Some(scope) = resolution.searched_scopes.get(*distance as usize) else {
                    diagnostics.push(AsmValidationDiagnostic {
                        code: "EZASM1173".to_string(),
                        message: format!(
                            "consumer `{consumer_id}` has an invalid ambiguity distance"
                        ),
                    });
                    continue;
                };
                let candidates = resolution_candidates(model, scope, resolution.context.as_ref());
                if providers.len() < 2
                    || providers.windows(2).any(|pair| pair[0] >= pair[1])
                    || *providers != candidates
                    || resolution.searched_scopes[..*distance as usize]
                        .iter()
                        .any(|scope| {
                            !resolution_candidates(model, scope, resolution.context.as_ref())
                                .is_empty()
                        })
                {
                    diagnostics.push(AsmValidationDiagnostic {
                        code: "EZASM1174".to_string(),
                        message: format!(
                            "consumer `{consumer_id}` has an invalid ambiguity result"
                        ),
                    });
                }
            }
            crate::ContextResolutionResult::InvalidContextReference => {
                if resolution.context.is_some() || consumer.context().is_some() {
                    diagnostics.push(AsmValidationDiagnostic {
                        code: "EZASM1175".to_string(),
                        message: format!(
                            "consumer `{consumer_id}` has an invalid Context-reference result"
                        ),
                    });
                }
            }
        }
    }
}

fn resolution_candidates(
    model: &ApplicationSemanticModel,
    scope: &crate::SemanticId,
    context: Option<&crate::ContextId>,
) -> Vec<crate::ProviderId> {
    let Some(context) = context else {
        return Vec::new();
    };
    model
        .providers
        .values()
        .filter(|provider| {
            provider.owner.entity_id() == Some(scope) && provider.context == *context
        })
        .map(|provider| provider.id.clone())
        .collect()
}

fn validate_contexts(
    model: &ApplicationSemanticModel,
    diagnostics: &mut Vec<AsmValidationDiagnostic>,
) {
    for context in model.contexts.values() {
        let Some(component_id) = context.owner.entity_id() else {
            diagnostics.push(AsmValidationDiagnostic {
                code: "EZASM1135".to_string(),
                message: format!("context `{}` is not component-owned", context.id),
            });
            continue;
        };
        let Some(component) = model.component(component_id) else {
            diagnostics.push(AsmValidationDiagnostic {
                code: "EZASM1136".to_string(),
                message: format!("context `{}` has a missing component owner", context.id),
            });
            continue;
        };
        if context.id != crate::ContextId::for_component(component_id, &context.name) {
            diagnostics.push(AsmValidationDiagnostic {
                code: "EZASM1137".to_string(),
                message: format!("context `{}` has a non-canonical identity", context.id),
            });
        }
        if context.authored_field != component.id.context_field(&context.name) {
            diagnostics.push(AsmValidationDiagnostic {
                code: "EZASM1138".to_string(),
                message: format!(
                    "context `{}` has a non-canonical authored field",
                    context.id
                ),
            });
        }
        if context.declared_type.text.is_empty() {
            diagnostics.push(AsmValidationDiagnostic {
                code: "EZASM1139".to_string(),
                message: format!(
                    "context `{}` is missing an explicit declared type",
                    context.id
                ),
            });
        }
        if context.execution_boundary != crate::ExecutionBoundary::Client {
            diagnostics.push(AsmValidationDiagnostic {
                code: "EZASM1140".to_string(),
                message: format!(
                    "context `{}` has a non-client execution boundary",
                    context.id
                ),
            });
        }
        if component
            .state_fields
            .iter()
            .any(|field| field.name == context.name)
        {
            diagnostics.push(AsmValidationDiagnostic {
                code: "EZASM1141".to_string(),
                message: format!("context `{}` also lowered as state", context.id),
            });
        }
        let declaration = component
            .context_declarations
            .iter()
            .find(|declaration| declaration.authored_field == context.authored_field);
        if declaration.is_none_or(|declaration| declaration.provenance != context.provenance) {
            diagnostics.push(AsmValidationDiagnostic {
                code: "EZASM1142".to_string(),
                message: format!(
                    "context `{}` has non-canonical field provenance",
                    context.id
                ),
            });
        }
        if context.default_expression != model.expression_root(context.id.as_semantic_id()).cloned()
        {
            diagnostics.push(AsmValidationDiagnostic {
                code: "EZASM1143".to_string(),
                message: format!("context `{}` has an invalid default expression", context.id),
            });
        }
    }
}

#[allow(clippy::too_many_lines)]
fn validate_providers(
    model: &ApplicationSemanticModel,
    diagnostics: &mut Vec<AsmValidationDiagnostic>,
) {
    for provider in model.providers.values() {
        let Some(component_id) = provider.owner.entity_id() else {
            diagnostics.push(AsmValidationDiagnostic {
                code: "EZASM1144".to_string(),
                message: format!("provider `{}` is not component-owned", provider.id),
            });
            continue;
        };
        let Some(component) = model.component(component_id) else {
            diagnostics.push(AsmValidationDiagnostic {
                code: "EZASM1145".to_string(),
                message: format!("provider `{}` has a missing component owner", provider.id),
            });
            continue;
        };
        if provider.id != crate::ProviderId::for_component(component_id, &provider.name) {
            diagnostics.push(AsmValidationDiagnostic {
                code: "EZASM1146".to_string(),
                message: format!("provider `{}` has a non-canonical identity", provider.id),
            });
        }
        if provider.authored_field != component.id.provider_field(&provider.name) {
            diagnostics.push(AsmValidationDiagnostic {
                code: "EZASM1147".to_string(),
                message: format!(
                    "provider `{}` has a non-canonical authored field",
                    provider.id
                ),
            });
        }
        let context = model.context(&provider.context);
        if context.is_none() {
            diagnostics.push(AsmValidationDiagnostic {
                code: "EZASM1148".to_string(),
                message: format!("provider `{}` targets a missing Context", provider.id),
            });
        }
        if context.is_some_and(|context| {
            context.name != provider.context_designator.context_member
                || context
                    .owner
                    .entity_id()
                    .and_then(|owner| model.component(owner))
                    .is_none_or(|owner| {
                        owner.class_name != provider.context_designator.component_symbol
                    })
        }) {
            diagnostics.push(AsmValidationDiagnostic {
                code: "EZASM1149".to_string(),
                message: format!(
                    "provider `{}` has a mismatched Context designator",
                    provider.id
                ),
            });
        }
        if provider.declared_type.text.is_empty() {
            diagnostics.push(AsmValidationDiagnostic {
                code: "EZASM1150".to_string(),
                message: format!(
                    "provider `{}` is missing an explicit declared type",
                    provider.id
                ),
            });
        }
        if model.expression_root(provider.id.as_semantic_id()) != Some(&provider.value_expression) {
            diagnostics.push(AsmValidationDiagnostic {
                code: "EZASM1151".to_string(),
                message: format!("provider `{}` has an invalid value expression", provider.id),
            });
        }
        if provider.execution_boundary != crate::ExecutionBoundary::Client {
            diagnostics.push(AsmValidationDiagnostic {
                code: "EZASM1152".to_string(),
                message: format!(
                    "provider `{}` has a non-client execution boundary",
                    provider.id
                ),
            });
        }
        if component
            .state_fields
            .iter()
            .any(|field| field.name == provider.name)
            || component
                .context_declarations
                .iter()
                .any(|context| context.name == provider.name)
        {
            diagnostics.push(AsmValidationDiagnostic {
                code: "EZASM1153".to_string(),
                message: format!(
                    "provider `{}` has a conflicting semantic primitive",
                    provider.id
                ),
            });
        }
        let declaration = component
            .provider_declarations
            .iter()
            .find(|declaration| declaration.authored_field == provider.authored_field);
        if declaration.is_none_or(|declaration| declaration.provenance != provider.provenance) {
            diagnostics.push(AsmValidationDiagnostic {
                code: "EZASM1154".to_string(),
                message: format!(
                    "provider `{}` has non-canonical field provenance",
                    provider.id
                ),
            });
        }
    }
}

#[allow(clippy::too_many_lines)]
fn validate_consumers(
    model: &ApplicationSemanticModel,
    diagnostics: &mut Vec<AsmValidationDiagnostic>,
) {
    for consumer in model.consumers.values() {
        let Some(component_id) = consumer.owner.entity_id() else {
            diagnostics.push(AsmValidationDiagnostic {
                code: "EZASM1155".to_string(),
                message: format!("consumer `{}` is not component-owned", consumer.id),
            });
            continue;
        };
        let Some(component) = model.component(component_id) else {
            diagnostics.push(AsmValidationDiagnostic {
                code: "EZASM1156".to_string(),
                message: format!("consumer `{}` has a missing component owner", consumer.id),
            });
            continue;
        };
        if consumer.id != crate::ConsumerId::for_component(component_id, &consumer.name) {
            diagnostics.push(AsmValidationDiagnostic {
                code: "EZASM1157".to_string(),
                message: format!("consumer `{}` has a non-canonical identity", consumer.id),
            });
        }
        if consumer.authored_field != component.id.consumer_field(&consumer.name) {
            diagnostics.push(AsmValidationDiagnostic {
                code: "EZASM1158".to_string(),
                message: format!(
                    "consumer `{}` has a non-canonical authored field",
                    consumer.id
                ),
            });
        }
        if let crate::ContextResolutionState::Resolved(context_id) = &consumer.context_resolution {
            let context = model.context(context_id);
            if context.is_none() {
                diagnostics.push(AsmValidationDiagnostic {
                    code: "EZASM1159".to_string(),
                    message: format!("consumer `{}` targets a missing Context", consumer.id),
                });
            }
            if context.is_some_and(|context| {
                context.name != consumer.context_designator.context_member
                    || context
                        .owner
                        .entity_id()
                        .and_then(|owner| model.component(owner))
                        .is_none_or(|owner| {
                            owner.class_name != consumer.context_designator.component_symbol
                        })
            }) {
                diagnostics.push(AsmValidationDiagnostic {
                    code: "EZASM1160".to_string(),
                    message: format!(
                        "consumer `{}` has a mismatched Context designator",
                        consumer.id
                    ),
                });
            }
        }
        if consumer.requested_type.text.is_empty() {
            diagnostics.push(AsmValidationDiagnostic {
                code: "EZASM1161".to_string(),
                message: format!(
                    "consumer `{}` is missing an explicit requested type",
                    consumer.id
                ),
            });
        }
        if consumer.execution_boundary != crate::ExecutionBoundary::Client {
            diagnostics.push(AsmValidationDiagnostic {
                code: "EZASM1162".to_string(),
                message: format!(
                    "consumer `{}` has a non-client execution boundary",
                    consumer.id
                ),
            });
        }
        if component
            .state_fields
            .iter()
            .any(|field| field.name == consumer.name)
            || component
                .context_declarations
                .iter()
                .any(|context| context.name == consumer.name)
            || component
                .provider_declarations
                .iter()
                .any(|provider| provider.name == consumer.name)
        {
            diagnostics.push(AsmValidationDiagnostic {
                code: "EZASM1163".to_string(),
                message: format!(
                    "consumer `{}` has a conflicting semantic primitive",
                    consumer.id
                ),
            });
        }
        let declaration = component
            .consumer_declarations
            .iter()
            .find(|declaration| declaration.authored_field == consumer.authored_field);
        if declaration.is_none_or(|declaration| declaration.provenance != consumer.provenance) {
            diagnostics.push(AsmValidationDiagnostic {
                code: "EZASM1164".to_string(),
                message: format!(
                    "consumer `{}` has non-canonical field provenance",
                    consumer.id
                ),
            });
        }
    }
}

#[allow(clippy::too_many_lines)]
fn validate_component_diagnostic_metadata(
    model: &ApplicationSemanticModel,
    diagnostics: &mut Vec<AsmValidationDiagnostic>,
) {
    let actual_component_diagnostics = model
        .diagnostics
        .iter()
        .filter(|item| ("EZC1068"..="EZC1083").contains(&item.code.as_str()))
        .cloned()
        .collect::<Vec<_>>();
    let expected_component_diagnostics = crate::collect_component_diagnostics(model);
    if actual_component_diagnostics != expected_component_diagnostics {
        diagnostics.push(AsmValidationDiagnostic {
            code: "EZASM1201".to_string(),
            message: "component diagnostics do not match the canonical H19 projection".to_string(),
        });
    }
    for diagnostic in &model.diagnostics {
        let valid_context = diagnostic
            .context_id
            .as_ref()
            .is_none_or(|id| model.context(id).is_some());
        let valid_provider = diagnostic
            .provider_id
            .as_ref()
            .is_none_or(|id| model.provider(id).is_some());
        let valid_consumer = diagnostic
            .consumer_id
            .as_ref()
            .is_none_or(|id| model.consumer(id).is_some());
        let valid_candidate = diagnostic
            .context_declaration_candidate_id
            .as_ref()
            .is_none_or(|id| {
                model
                    .context_declaration_candidates()
                    .candidate(id)
                    .is_some()
            });
        if !(valid_context && valid_provider && valid_consumer && valid_candidate) {
            diagnostics.push(AsmValidationDiagnostic {
                code: "EZASM1135".to_string(),
                message: format!(
                    "compiler diagnostic `{}` references an unknown Context diagnostic subject",
                    diagnostic.code
                ),
            });
        }
        if let (Some(provider), Some(context)) = (
            diagnostic
                .provider_id
                .as_ref()
                .and_then(|id| model.provider(id)),
            diagnostic.context_id.as_ref(),
        ) {
            if &provider.context != context {
                diagnostics.push(AsmValidationDiagnostic {
                    code: "EZASM1138".to_string(),
                    message: format!(
                        "compiler diagnostic `{}` references a Provider for the wrong Context",
                        diagnostic.code
                    ),
                });
            }
        }
        if let (Some(consumer), Some(context)) = (
            diagnostic
                .consumer_id
                .as_ref()
                .and_then(|id| model.consumer(id)),
            diagnostic.context_id.as_ref(),
        ) {
            if consumer.context() != Some(context) {
                diagnostics.push(AsmValidationDiagnostic {
                    code: "EZASM1139".to_string(),
                    message: format!(
                        "compiler diagnostic `{}` references a Consumer for the wrong Context",
                        diagnostic.code
                    ),
                });
            }
        }
        if diagnostic.context_declaration_candidate_id.is_some()
            && (diagnostic.context_id.is_some()
                || diagnostic.provider_id.is_some()
                || diagnostic.consumer_id.is_some())
        {
            diagnostics.push(AsmValidationDiagnostic { code: "EZASM1136".to_string(), message: format!("invalid Context declaration diagnostic `{}` carries a semantic entity identity", diagnostic.code) });
        }
        if ("EZC1052"..="EZC1067").contains(&diagnostic.code.as_str()) {
            if let Some(primary) = diagnostic.provenance.as_ref() {
                if !context_diagnostic_primary_is_canonical(model, diagnostic, primary) {
                    diagnostics.push(AsmValidationDiagnostic {
                        code: "EZASM1140".to_string(),
                        message: format!(
                            "Context diagnostic `{}` has non-canonical primary provenance",
                            diagnostic.code
                        ),
                    });
                }
            }
            if diagnostic.code == "EZC1058" {
                let mut expected = diagnostic
                    .consumer_id
                    .as_ref()
                    .and_then(|consumer| model.context_resolutions.get(consumer))
                    .and_then(|resolution| match &resolution.result {
                        crate::ContextResolutionResult::Ambiguous { providers, .. } => {
                            Some(providers.clone())
                        }
                        _ => None,
                    })
                    .unwrap_or_default();
                expected.sort();
                expected.dedup();
                let expected = expected
                    .iter()
                    .filter_map(|id| model.provider(id))
                    .map(|provider| crate::DiagnosticSecondaryLabel {
                        provenance: provider.provenance.clone(),
                        message: format!("Candidate Provider `{}`.", provider.id),
                    })
                    .collect::<Vec<_>>();
                if diagnostic.secondary_labels != expected {
                    diagnostics.push(AsmValidationDiagnostic {
                        code: "EZASM1134".to_string(),
                        message: "Context ambiguity diagnostic has non-canonical Provider evidence"
                            .to_string(),
                    });
                }
            }
        }
        let effect = diagnostic.effect_id.as_ref().and_then(|effect_id| {
            model
                .effects
                .values()
                .find(|effect| effect.id.as_str() == effect_id.as_str())
        });
        if diagnostic.effect_id.is_some() && effect.is_none() {
            diagnostics.push(AsmValidationDiagnostic {
                code: "EZASM1129".to_string(),
                message: format!(
                    "compiler diagnostic `{}` references a missing effect subject",
                    diagnostic.code
                ),
            });
        }

        let statement = diagnostic.statement_id.as_ref().and_then(|statement_id| {
            model
                .effect_statements
                .values()
                .find(|statement| statement.id.as_str() == statement_id.as_str())
        });
        if diagnostic.statement_id.is_some() && diagnostic.effect_id.is_none() {
            diagnostics.push(AsmValidationDiagnostic {
                code: "EZASM1130".to_string(),
                message: format!(
                    "compiler diagnostic `{}` has an effect statement without an effect subject",
                    diagnostic.code
                ),
            });
        }
        if diagnostic.statement_id.is_some()
            && statement
                .is_none_or(|statement| effect.is_none_or(|effect| statement.owner != effect.id))
        {
            diagnostics.push(AsmValidationDiagnostic {
                code: "EZASM1131".to_string(),
                message: format!(
                    "compiler diagnostic `{}` has a statement that does not belong to its effect",
                    diagnostic.code
                ),
            });
        }

        let primary_subject = statement
            .map(|statement| &statement.provenance)
            .or_else(|| effect.map(|effect| &effect.provenance));
        if let (Some(primary), Some(subject)) = (&diagnostic.provenance, primary_subject) {
            if !provenance_contains(subject, primary) {
                diagnostics.push(AsmValidationDiagnostic {
                    code: "EZASM1132".to_string(),
                    message: format!(
                        "compiler diagnostic `{}` has non-canonical primary provenance",
                        diagnostic.code
                    ),
                });
            }
        }

        let mut sorted = diagnostic.secondary_labels.clone();
        sorted.sort_by(secondary_label_order);
        sorted.dedup();
        if sorted != diagnostic.secondary_labels {
            diagnostics.push(AsmValidationDiagnostic {
                code: "EZASM1133".to_string(),
                message: format!(
                    "compiler diagnostic `{}` has unordered or duplicate secondary labels",
                    diagnostic.code
                ),
            });
        }
        for label in &diagnostic.secondary_labels {
            if diagnostic.provenance.as_ref() == Some(&label.provenance) {
                diagnostics.push(AsmValidationDiagnostic {
                    code: "EZASM1137".to_string(),
                    message: format!(
                        "compiler diagnostic `{}` repeats primary provenance as a secondary label",
                        diagnostic.code
                    ),
                });
            }
            let canonical = model
                .provenance
                .values()
                .any(|provenance| provenance == &label.provenance)
                || model
                    .expression_graph
                    .nodes
                    .values()
                    .any(|expression| expression.provenance == label.provenance)
                || context_diagnostic_secondary_is_canonical(model, diagnostic, &label.provenance);
            if !canonical {
                diagnostics.push(AsmValidationDiagnostic {
                    code: "EZASM1134".to_string(),
                    message: format!(
                        "compiler diagnostic `{}` has non-canonical secondary-label provenance",
                        diagnostic.code
                    ),
                });
            }
        }
    }
}

fn context_diagnostic_secondary_is_canonical(
    model: &ApplicationSemanticModel,
    diagnostic: &crate::ComponentDiagnostic,
    provenance: &crate::SourceProvenance,
) -> bool {
    match diagnostic.code.as_str() {
        "EZC1059" => diagnostic
            .provider_id
            .as_ref()
            .and_then(|id| model.provider(id))
            .is_some_and(|provider| &provider.declared_type.provenance == provenance),
        "EZC1060" | "EZC1061" | "EZC1062" | "EZC1063" | "EZC1064" => diagnostic
            .context_id
            .as_ref()
            .and_then(|id| model.context(id))
            .is_some_and(|context| &context.declared_type.provenance == provenance),
        _ => false,
    }
}

#[allow(clippy::too_many_lines)]
fn context_diagnostic_primary_is_canonical(
    model: &ApplicationSemanticModel,
    diagnostic: &crate::ComponentDiagnostic,
    primary: &crate::SourceProvenance,
) -> bool {
    match diagnostic.code.as_str() {
        "EZC1052" | "EZC1053" | "EZC1054" | "EZC1055" | "EZC1056" => diagnostic
            .context_declaration_candidate_id
            .as_ref()
            .and_then(|id| model.context_declaration_candidates().candidate(id))
            .is_some_and(|candidate| {
                let crate::ContextDeclarationStatus::Invalid(violations) = &candidate.status else {
                    return false;
                };
                let expected = match violations.first() {
                    Some(crate::ContextDeclarationViolation::StaticDeclarationUnsupported) => {
                        candidate.authored.static_modifier_provenance.as_ref()
                    }
                    Some(
                        crate::ContextDeclarationViolation::UnsupportedInitializer
                        | crate::ContextDeclarationViolation::ForbiddenInitializer
                        | crate::ContextDeclarationViolation::MissingInitializer,
                    ) => candidate.authored.initializer_provenance.as_ref(),
                    Some(
                        crate::ContextDeclarationViolation::ContextDesignatorUnsupported
                        | crate::ContextDeclarationViolation::UnresolvedContextDesignator,
                    ) => candidate
                        .authored
                        .context_designator
                        .as_ref()
                        .map(|designator| &designator.provenance),
                    Some(_) => Some(&candidate.authored.decorator_provenance),
                    None => None,
                }
                .unwrap_or(&candidate.authored.provenance);
                expected == primary
            }),
        "EZC1057" | "EZC1058" => diagnostic
            .consumer_id
            .as_ref()
            .and_then(|id| model.context_resolutions.get(id))
            .is_some_and(|record| &record.provenance == primary),
        "EZC1059" => {
            diagnostic
                .provider_id
                .as_ref()
                .and_then(|id| model.provider(id))
                .and_then(|provider| {
                    model
                        .expression_graph
                        .provenance_of(&provider.value_expression)
                })
                == Some(primary)
        }
        "EZC1060" => diagnostic
            .provider_id
            .as_ref()
            .and_then(|id| model.provider(id))
            .is_some_and(|provider| &provider.declared_type.provenance == primary),
        "EZC1061" => {
            diagnostic
                .context_id
                .as_ref()
                .and_then(|id| model.context(id))
                .and_then(|context| context.default_expression.as_ref())
                .and_then(|expression| model.expression_graph.provenance_of(expression))
                == Some(primary)
        }
        "EZC1062" => diagnostic
            .consumer_id
            .as_ref()
            .and_then(|id| model.consumer(id))
            .is_some_and(|consumer| &consumer.requested_type.provenance == primary),
        "EZC1063" | "EZC1064" => {
            if let Some(provider) = diagnostic
                .provider_id
                .as_ref()
                .and_then(|id| model.provider(id))
            {
                return model
                    .expression_graph
                    .provenance_of(&provider.value_expression)
                    == Some(primary);
            }
            diagnostic
                .context_id
                .as_ref()
                .and_then(|id| model.context(id))
                .is_some_and(|context| {
                    context
                        .default_expression
                        .as_ref()
                        .and_then(|expression| model.expression_graph.provenance_of(expression))
                        .unwrap_or(&context.declared_type.provenance)
                        == primary
                })
        }
        "EZC1065" => {
            diagnostic
                .consumer_id
                .as_ref()
                .and_then(|id| model.context_lifetime.binding_lifetimes.get(id))
                .is_some_and(|record| &record.provenance == primary)
                || model
                    .context_lifetime
                    .dependency_lifetimes
                    .iter()
                    .any(|record| &record.provenance == primary)
        }
        "EZC1066" | "EZC1067" => model
            .context_evaluation
            .source_entries
            .values()
            .filter(|entry| diagnostic.context_id.as_ref() == Some(&entry.context))
            .filter(|entry| match &entry.source {
                crate::ContextValueSourceId::Provider(provider) => {
                    diagnostic.provider_id.as_ref() == Some(provider)
                }
                crate::ContextValueSourceId::ContextDefault(_) => diagnostic.provider_id.is_none(),
            })
            .any(|entry| {
                if diagnostic.code == "EZC1067" {
                    return model
                        .expression_graph
                        .provenance_of(&entry.expression_root)
                        .unwrap_or(&entry.provenance)
                        == primary;
                }
                let dependent = match &entry.source {
                    crate::ContextValueSourceId::Provider(provider) => {
                        crate::ContextDependencyNodeId::Provider(provider.clone())
                    }
                    crate::ContextValueSourceId::ContextDefault(context) => {
                        crate::ContextDependencyNodeId::ContextDefault(context.clone())
                    }
                };
                entry.reasons.iter().any(|reason| {
                    let dependency = match reason {
                        crate::ContextSourceBlockReason::MissingStateDependency(id) => {
                            crate::ContextDependencyNodeId::State(id.clone())
                        }
                        crate::ContextSourceBlockReason::UnavailableComputedDependency(id) => {
                            crate::ContextDependencyNodeId::Computed(id.clone())
                        }
                        _ => return false,
                    };
                    model.context_dependency.edges.iter().any(|edge| {
                        edge.dependent == dependent
                            && edge.dependency == dependency
                            && &edge.provenance == primary
                    })
                }) || &entry.provenance == primary
            }),
        _ => false,
    }
}

fn provenance_contains(
    subject: &crate::SourceProvenance,
    primary: &crate::SourceProvenance,
) -> bool {
    subject.path == primary.path
        && subject.span.start <= primary.span.start
        && primary.span.end <= subject.span.end
}

fn secondary_label_order(
    left: &crate::DiagnosticSecondaryLabel,
    right: &crate::DiagnosticSecondaryLabel,
) -> std::cmp::Ordering {
    (
        left.provenance.path.as_path(),
        left.provenance.span.start,
        left.provenance.span.end,
        left.message.as_str(),
    )
        .cmp(&(
            right.provenance.path.as_path(),
            right.provenance.span.start,
            right.provenance.span.end,
            right.message.as_str(),
        ))
}

fn validate_template_action_bindings(
    model: &ApplicationSemanticModel,
    diagnostics: &mut Vec<AsmValidationDiagnostic>,
) {
    let manifest = build_template_manifest_from_asm(model);
    if manifest.schema_version < 2 || manifest.schema_version > TEMPLATE_MANIFEST_SCHEMA_VERSION {
        return;
    }
    if manifest.schema_version == TEMPLATE_MANIFEST_SCHEMA_VERSION {
        for event in &manifest.ordinary_events {
            let component = model
                .components
                .iter()
                .find(|component| component.id.as_str() == event.component_id);
            // The v4 execution table is the authority. Declaration-template
            // events remain inspection context and deliberately need not be
            // executable for every materialized instance.
            let Some(component) = component else {
                diagnostics.push(AsmValidationDiagnostic {
                    code: "EZASM1126".to_string(),
                    message: "ordinary event references a missing component".to_string(),
                });
                continue;
            };
            let method = component
                .methods
                .iter()
                .find(|method| method.id.as_str() == event.handler_method_id);
            let batch = method.and_then(|method| {
                model
                    .effect_trigger_plan
                    .action_batches
                    .values()
                    .find(|batch| batch.authored_action_method == method.id)
            });
            if batch.is_none_or(|batch| Some(batch.id.as_str()) != event.action_batch_id.as_deref())
            {
                diagnostics.push(AsmValidationDiagnostic {
                    code: "EZASM1128".to_string(),
                    message: "ordinary event does not resolve to its canonical F8 action batch"
                        .to_string(),
                });
            }
        }
        return;
    }
    for component_manifest in &manifest.components {
        let Some(component) = model
            .components
            .iter()
            .find(|component| component.class_name == component_manifest.name)
        else {
            diagnostics.push(AsmValidationDiagnostic {
                code: "EZASM1126".to_string(),
                message: format!(
                    "template manifest references missing component `{}`",
                    component_manifest.name
                ),
            });
            continue;
        };
        for event in &component_manifest.template.events {
            if event.kind != Some(ManifestEventKind::Action) {
                diagnostics.push(AsmValidationDiagnostic {
                    code: "EZASM1127".to_string(),
                    message: format!(
                        "template event `{}` is missing canonical action binding metadata",
                        event.node
                    ),
                });
                continue;
            }
            let method = component
                .methods
                .iter()
                .find(|method| Some(method.id.as_str()) == event.method_id.as_deref());
            let batch = method.and_then(|method| {
                model
                    .effect_trigger_plan
                    .action_batches
                    .values()
                    .find(|batch| batch.authored_action_method == method.id)
            });
            if batch.is_none_or(|batch| Some(batch.id.as_str()) != event.action_batch_id.as_deref())
            {
                diagnostics.push(AsmValidationDiagnostic {
                    code: "EZASM1128".to_string(),
                    message: format!(
                        "template event `{}` does not resolve to its canonical F8 action batch",
                        event.node
                    ),
                });
            }
        }
    }
}

fn validate_effect_execution_plan(
    model: &ApplicationSemanticModel,
    diagnostics: &mut Vec<AsmValidationDiagnostic>,
) {
    let plan = model.effect_execution_plan();
    if plan.initial.render_boundary != Some(EffectRenderBoundary::AfterInitialRender) {
        diagnostics.push(AsmValidationDiagnostic {
            code: "EZASM1120".to_string(),
            message: "initial effect execution plan is missing the after-initial-render boundary"
                .to_string(),
        });
    }
    validate_effect_execution_entry(
        model,
        &model.effect_trigger_plan.initial_effects,
        &plan.initial.required_computed,
        &plan.initial.prerequisite_batches,
        &plan.initial.effect_batches,
        &plan.initial.unplanned_effects,
        "initial",
        diagnostics,
    );
    for action in &plan.actions {
        let Some(trigger) = model
            .effect_trigger_plan
            .action_batch_triggers
            .iter()
            .find(|trigger| trigger.action_batch == action.action_batch)
        else {
            diagnostics.push(AsmValidationDiagnostic {
                code: "EZASM1121".to_string(),
                message: format!(
                    "effect execution plan references untriggered action batch `{}`",
                    action.action_batch
                ),
            });
            continue;
        };
        validate_effect_execution_entry(
            model,
            &trigger.effects,
            &action.required_computed,
            &action.prerequisite_batches,
            &action.effect_batches,
            &action.unplanned_effects,
            action.action_batch.as_str(),
            diagnostics,
        );
    }
}

#[allow(clippy::too_many_arguments)]
fn validate_effect_execution_entry(
    model: &ApplicationSemanticModel,
    eligible_effects: &[crate::SemanticId],
    required_computed: &[crate::SemanticId],
    prerequisite_batches: &[crate::EffectComputedPrerequisiteBatch],
    effect_batches: &[crate::EffectExecutionBatch],
    unplanned_effects: &[crate::UnplannedEffect],
    context: &str,
    diagnostics: &mut Vec<AsmValidationDiagnostic>,
) {
    let required = required_computed
        .iter()
        .collect::<std::collections::BTreeSet<_>>();
    if required.len() != required_computed.len()
        || required_computed
            .iter()
            .any(|computed| !model.computed_values.contains_key(computed))
    {
        diagnostics.push(AsmValidationDiagnostic {
            code: "EZASM1122".to_string(),
            message: format!("effect plan `{context}` has invalid required computed membership"),
        });
    }
    let mut batch_membership = std::collections::BTreeSet::new();
    let mut prior_source_index = None;
    for batch in prerequisite_batches {
        let expected = model
            .computed_evaluation_plan
            .update_batches
            .get(batch.source_batch_index as usize)
            .map(|source| {
                source
                    .iter()
                    .filter_map(|id| {
                        model
                            .computed_values
                            .keys()
                            .find(|computed| computed.as_str() == id)
                    })
                    .filter(|computed| required.contains(computed))
                    .cloned()
                    .collect::<Vec<_>>()
            });
        if expected.as_ref() != Some(&batch.computed)
            || prior_source_index.is_some_and(|prior| prior >= batch.source_batch_index)
        {
            diagnostics.push(AsmValidationDiagnostic {
                code: "EZASM1123".to_string(),
                message: format!(
                    "effect plan `{context}` does not preserve canonical computed batch membership"
                ),
            });
        }
        prior_source_index = Some(batch.source_batch_index);
        batch_membership.extend(batch.computed.iter());
    }
    if batch_membership != required {
        diagnostics.push(AsmValidationDiagnostic {
            code: "EZASM1124".to_string(),
            message: format!(
                "effect plan `{context}` required computed values do not match prerequisite batches"
            ),
        });
    }

    let scheduled = effect_batches
        .iter()
        .flat_map(|batch| &batch.effects)
        .collect::<std::collections::BTreeSet<_>>();
    let unplanned = unplanned_effects
        .iter()
        .map(|record| &record.effect)
        .collect::<std::collections::BTreeSet<_>>();
    let eligible = eligible_effects
        .iter()
        .collect::<std::collections::BTreeSet<_>>();
    let covered = scheduled
        .union(&unplanned)
        .copied()
        .collect::<std::collections::BTreeSet<_>>();
    if covered != eligible
        || scheduled.intersection(&unplanned).next().is_some()
        || scheduled.iter().any(|effect| {
            model
                .effects
                .get(*effect)
                .is_none_or(|effect| effect.validation != EffectValidation::Valid)
        })
    {
        diagnostics.push(AsmValidationDiagnostic {
            code: "EZASM1125".to_string(),
            message: format!("effect plan `{context}` has invalid effect eligibility membership"),
        });
    }
    if effect_batches.iter().enumerate().any(|(index, batch)| {
        batch.index != u32::try_from(index).expect("effect scheduler batch index should fit u32")
    }) {
        diagnostics.push(AsmValidationDiagnostic {
            code: "EZASM1126".to_string(),
            message: format!("effect plan `{context}` has non-contiguous terminal batch indexes"),
        });
    }
}

fn validate_effect_statement_types(
    model: &ApplicationSemanticModel,
    diagnostics: &mut Vec<AsmValidationDiagnostic>,
) {
    for (statement, record) in &model.semantic_types.effect_statements {
        if statement != &record.statement {
            diagnostics.push(AsmValidationDiagnostic {
                code: "EZASM1110".to_string(),
                message: format!(
                    "effect statement type record key `{statement}` does not match statement `{}`",
                    record.statement
                ),
            });
        }
        let Some(canonical_statement) = model.effect_statement(statement) else {
            diagnostics.push(AsmValidationDiagnostic {
                code: "EZASM1111".to_string(),
                message: format!(
                    "effect statement type record references missing statement `{statement}`"
                ),
            });
            continue;
        };
        if canonical_statement.provenance != record.provenance {
            diagnostics.push(AsmValidationDiagnostic {
                code: "EZASM1112".to_string(),
                message: format!(
                    "effect statement type record for `{statement}` has inconsistent provenance"
                ),
            });
        }
        let operation_exists = record.capability_operation.is_some_and(|operation_id| {
            EFFECT_CAPABILITY_REGISTRY
                .definitions()
                .iter()
                .flat_map(|definition| definition.operations)
                .any(|operation| operation.id == operation_id)
        });
        if record.operation_classification == EffectOperationClassification::RecognizedCapability
            && !operation_exists
        {
            diagnostics.push(AsmValidationDiagnostic {
                code: "EZASM1113".to_string(),
                message: format!(
                    "recognized effect statement `{statement}` has no registry capability operation"
                ),
            });
        }
        if record.operation_classification != EffectOperationClassification::RecognizedCapability
            && record.capability_operation.is_some()
        {
            diagnostics.push(AsmValidationDiagnostic {
                code: "EZASM1114".to_string(),
                message: format!(
                    "non-capability effect statement `{statement}` unexpectedly names a registry operation"
                ),
            });
        }
    }
}

fn validate_semantic_types(
    model: &ApplicationSemanticModel,
    diagnostics: &mut Vec<AsmValidationDiagnostic>,
) {
    for (subject, assignment) in &model.semantic_types.assignments {
        if subject != &assignment.subject {
            diagnostics.push(AsmValidationDiagnostic {
                code: "EZASM1101".to_string(),
                message: format!(
                    "semantic type assignment map key `{subject}` does not match subject `{}`",
                    assignment.subject
                ),
            });
        }
        if assignment.id != SemanticTypeId::for_subject(&assignment.subject) {
            diagnostics.push(AsmValidationDiagnostic {
                code: "EZASM1102".to_string(),
                message: format!(
                    "semantic type assignment for `{subject}` has an invalid canonical type ID `{}`",
                    assignment.id
                ),
            });
        }
        let subject_provenance = model
            .provenance(&assignment.subject)
            .or_else(|| model.expression_provenance(&assignment.subject));
        if subject_provenance.is_none() {
            diagnostics.push(AsmValidationDiagnostic {
                code: "EZASM1103".to_string(),
                message: format!(
                    "semantic type assignment references missing subject `{}`",
                    assignment.subject
                ),
            });
        }
        if subject_provenance != Some(&assignment.provenance) {
            diagnostics.push(AsmValidationDiagnostic {
                code: "EZASM1104".to_string(),
                message: format!(
                    "semantic type assignment for `{subject}` has inconsistent provenance"
                ),
            });
        }
        if model.entity(&assignment.origin).is_none()
            && !model
                .semantic_types
                .aliases
                .contains_key(&assignment.origin)
        {
            diagnostics.push(AsmValidationDiagnostic {
                code: "EZASM1105".to_string(),
                message: format!(
                    "semantic type assignment for `{subject}` has unresolved origin `{}`",
                    assignment.origin
                ),
            });
        }
    }

    for alias in model.semantic_types.aliases.values() {
        let expected = crate::SemanticId::type_alias_in_module(&alias.provenance.path, &alias.name);
        if alias.id != expected {
            diagnostics.push(AsmValidationDiagnostic {
                code: "EZASM1106".to_string(),
                message: format!(
                    "type alias `{}` has invalid canonical identity `{}`",
                    alias.name, alias.id
                ),
            });
        }
    }
}
