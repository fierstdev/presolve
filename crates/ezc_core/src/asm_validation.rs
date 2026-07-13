use crate::application_semantic_model::ApplicationSemanticModel;
use crate::semantic_id::SemanticOwner;
use crate::{
    build_template_manifest_from_asm, EffectOperationClassification, EffectRenderBoundary,
    EffectValidation, ManifestEventKind, SemanticTypeId, EFFECT_CAPABILITY_REGISTRY,
    TEMPLATE_MANIFEST_SCHEMA_VERSION,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AsmValidationDiagnostic {
    pub code: String,
    pub message: String,
}

#[must_use]
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
    validate_contexts(model, &mut diagnostics);
    validate_providers(model, &mut diagnostics);
    validate_consumers(model, &mut diagnostics);
    validate_context_resolution(model, &mut diagnostics);
    validate_effect_statement_types(model, &mut diagnostics);
    validate_effect_execution_plan(model, &mut diagnostics);
    validate_component_diagnostic_metadata(model, &mut diagnostics);
    validate_template_action_bindings(model, &mut diagnostics);

    diagnostics
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

fn validate_component_diagnostic_metadata(
    model: &ApplicationSemanticModel,
    diagnostics: &mut Vec<AsmValidationDiagnostic>,
) {
    for diagnostic in &model.diagnostics {
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
            let canonical = model
                .provenance
                .values()
                .any(|provenance| provenance == &label.provenance)
                || model
                    .expression_graph
                    .nodes
                    .values()
                    .any(|expression| expression.provenance == label.provenance);
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
    if manifest.schema_version != TEMPLATE_MANIFEST_SCHEMA_VERSION {
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
