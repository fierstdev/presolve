use std::collections::BTreeSet;

use crate::{
    ApplicationSemanticModel, CompatibilityStatus, ComponentDiagnostic,
    ComponentDiagnosticSeverity, ContextBindingLifetimeStatus, ContextDeclarationCandidateKind,
    ContextDeclarationStatus, ContextDeclarationViolation, ContextDependencyNodeId,
    ContextResolutionResult, ContextSerializationCompatibility, ContextSourceBlockReason,
    ContextSourcePlanEntry, ContextSourcePlanStatus, ContextValueSourceId,
    LifetimeCompatibilityStatus, SourceProvenance,
};

/// Projects only retained Context declaration candidates into the frozen G18
/// catalog.  This module deliberately has no parser or source-text dependency.
#[allow(clippy::too_many_lines)]
#[must_use]
pub fn collect_context_diagnostics(model: &ApplicationSemanticModel) -> Vec<ComponentDiagnostic> {
    let mut emitted_duplicate_groups = BTreeSet::new();
    let mut diagnostics = Vec::new();
    for candidate in model.context_declaration_candidates().invalid_candidates() {
        let ContextDeclarationStatus::Invalid(violations) = &candidate.status else {
            continue;
        };
        let primary = violations.first();
        let Some(violation) = primary else {
            continue;
        };
        let code = match violation {
            ContextDeclarationViolation::UnresolvedContextDesignator => "EZC1055",
            ContextDeclarationViolation::DuplicateProvider => "EZC1056",
            _ => match candidate.authored.kind {
                ContextDeclarationCandidateKind::Context => "EZC1052",
                ContextDeclarationCandidateKind::Provider => "EZC1053",
                ContextDeclarationCandidateKind::Consumer => "EZC1054",
            },
        };
        if code == "EZC1056" {
            let Some(designator) = &candidate.authored.context_designator else {
                continue;
            };
            let group = (
                candidate.authored.owner_component.clone(),
                designator.component_symbol.clone(),
                designator.context_member.clone(),
            );
            if !emitted_duplicate_groups.insert(group) {
                continue;
            }
        }
        let provenance = match violation {
            ContextDeclarationViolation::StaticDeclarationUnsupported => {
                candidate.authored.static_modifier_provenance.as_ref()
            }
            ContextDeclarationViolation::UnsupportedInitializer
            | ContextDeclarationViolation::ForbiddenInitializer
            | ContextDeclarationViolation::MissingInitializer => {
                candidate.authored.initializer_provenance.as_ref()
            }
            ContextDeclarationViolation::ContextDesignatorUnsupported
            | ContextDeclarationViolation::UnresolvedContextDesignator => candidate
                .authored
                .context_designator
                .as_ref()
                .map(|designator| &designator.provenance),
            _ => Some(&candidate.authored.decorator_provenance),
        }
        .or(Some(&candidate.authored.provenance));
        diagnostics.push(ComponentDiagnostic {
            code: code.to_string(),
            severity: ComponentDiagnosticSeverity::Error,
            message: message(code, candidate.authored.field_name.as_deref()),
            provenance: provenance.cloned(),
            effect_id: None,
            statement_id: None,
            context_declaration_candidate_id: Some(candidate.authored.id.clone()),
            context_id: None,
            provider_id: None,
            consumer_id: None,
            secondary_labels: Vec::new(),
        });
    }
    for resolution in model.context_resolutions.values() {
        match &resolution.result {
            ContextResolutionResult::Unresolved => push(
                &mut diagnostics,
                "EZC1057",
                "Consumer Context binding is unresolved.",
                &resolution.provenance,
            ),
            ContextResolutionResult::Ambiguous { .. } => push(
                &mut diagnostics,
                "EZC1058",
                "Consumer Context binding is ambiguous.",
                &resolution.provenance,
            ),
            _ => {}
        }
    }
    for record in model.provider_types.values() {
        let value_mismatch = record.value_to_declaration == CompatibilityStatus::Incompatible;
        let context_mismatch = record.declaration_to_context == CompatibilityStatus::Incompatible;
        if value_mismatch {
            let provenance = model
                .provider(&record.provider)
                .and_then(|provider| expression_provenance(model, &provider.value_expression))
                .unwrap_or(&record.provenance);
            push(
                &mut diagnostics,
                "EZC1059",
                "Provider value is incompatible with its declared type.",
                provenance,
            );
        }
        if context_mismatch {
            let provenance = model
                .provider(&record.provider)
                .map_or(&record.provenance, |provider| {
                    &provider.declared_type.provenance
                });
            push(
                &mut diagnostics,
                "EZC1060",
                "Provider declared type is incompatible with its Context.",
                provenance,
            );
        }
        if !value_mismatch
            && !context_mismatch
            && record.serialization == ContextSerializationCompatibility::NonSerializable
        {
            let provenance = model
                .provider(&record.provider)
                .and_then(|provider| expression_provenance(model, &provider.value_expression))
                .unwrap_or(&record.provenance);
            push(
                &mut diagnostics,
                "EZC1063",
                "Context Provider source is not serializable.",
                provenance,
            );
        } else if !value_mismatch
            && !context_mismatch
            && record.boundary_compatibility == CompatibilityStatus::Incompatible
        {
            let provenance = model
                .provider(&record.provider)
                .and_then(|provider| expression_provenance(model, &provider.value_expression))
                .unwrap_or(&record.provenance);
            push(
                &mut diagnostics,
                "EZC1064",
                "Context Provider source crosses an incompatible boundary.",
                provenance,
            );
        }
    }
    for record in model.context_types.values() {
        let default_mismatch =
            record.default_compatibility == Some(CompatibilityStatus::Incompatible);
        if default_mismatch {
            let provenance =
                context_default_provenance(model, &record.context).unwrap_or(&record.provenance);
            push(
                &mut diagnostics,
                "EZC1061",
                "Context default is incompatible with its declared type.",
                provenance,
            );
        }
        if !default_mismatch
            && record.serialization == ContextSerializationCompatibility::NonSerializable
        {
            let provenance = context_default_or_declared_type_provenance(model, &record.context)
                .unwrap_or(&record.provenance);
            push(
                &mut diagnostics,
                "EZC1063",
                "Context default is not serializable.",
                provenance,
            );
        } else if !default_mismatch
            && record.boundary_compatibility == CompatibilityStatus::Incompatible
        {
            let provenance = context_default_or_declared_type_provenance(model, &record.context)
                .unwrap_or(&record.provenance);
            push(
                &mut diagnostics,
                "EZC1064",
                "Context crosses an incompatible boundary.",
                provenance,
            );
        }
    }
    for record in model.consumer_types.values() {
        if record.context_to_consumer == CompatibilityStatus::Incompatible {
            let provenance = model
                .consumer(&record.consumer)
                .map_or(&record.provenance, |consumer| {
                    &consumer.requested_type.provenance
                });
            push(
                &mut diagnostics,
                "EZC1062",
                "Context is incompatible with the Consumer request.",
                provenance,
            );
        }
    }
    for record in &model.context_lifetime.dependency_lifetimes {
        if record.compatibility == LifetimeCompatibilityStatus::Incompatible {
            push(
                &mut diagnostics,
                "EZC1065",
                "Context dependency lifetime is incompatible.",
                &record.provenance,
            );
        }
    }
    for record in model.context_lifetime.binding_lifetimes.values() {
        if record.compatibility == ContextBindingLifetimeStatus::Incompatible {
            push(
                &mut diagnostics,
                "EZC1065",
                "Context Consumer binding lifetime is incompatible.",
                &record.provenance,
            );
        }
    }
    for entry in model.context_evaluation.source_entries.values() {
        let specific = matches!(
            entry.status,
            ContextSourcePlanStatus::BlockedType
                | ContextSourcePlanStatus::BlockedSerialization
                | ContextSourcePlanStatus::BlockedBoundary
                | ContextSourcePlanStatus::BlockedLifetime
        );
        if !specific
            && entry.reasons.iter().any(|reason| {
                matches!(
                    reason,
                    ContextSourceBlockReason::MissingStateDependency(_)
                        | ContextSourceBlockReason::UnavailableComputedDependency(_)
                )
            })
        {
            let provenance = context_source_failure_provenance(model, entry, "EZC1066");
            push(
                &mut diagnostics,
                "EZC1066",
                "Context source has an unavailable dependency.",
                provenance,
            );
        } else if !specific
            && entry
                .reasons
                .iter()
                .any(|reason| matches!(reason, ContextSourceBlockReason::UnsupportedExpression))
        {
            let provenance = context_source_failure_provenance(model, entry, "EZC1067");
            push(
                &mut diagnostics,
                "EZC1067",
                "Context source cannot be planned from an unsupported expression.",
                provenance,
            );
        }
    }
    populate_identities(model, &mut diagnostics);
    populate_secondary_labels(model, &mut diagnostics);
    diagnostics.sort_by(|left, right| {
        (
            left.provenance
                .as_ref()
                .map(|value| (&value.path, value.span.start)),
            &left.code,
        )
            .cmp(&(
                right
                    .provenance
                    .as_ref()
                    .map(|value| (&value.path, value.span.start)),
                &right.code,
            ))
    });
    diagnostics
}

fn context_default_provenance<'a>(
    model: &'a ApplicationSemanticModel,
    context: &crate::ContextId,
) -> Option<&'a SourceProvenance> {
    model
        .context(context)
        .and_then(|context| context.default_expression.as_ref())
        .and_then(|expression| expression_provenance(model, expression))
}

fn expression_provenance<'a>(
    model: &'a ApplicationSemanticModel,
    expression: &crate::SemanticId,
) -> Option<&'a SourceProvenance> {
    model
        .expression_graph
        .provenance_of(expression)
        .or_else(|| model.provenance(expression))
}

fn context_default_or_declared_type_provenance<'a>(
    model: &'a ApplicationSemanticModel,
    context: &crate::ContextId,
) -> Option<&'a SourceProvenance> {
    context_default_provenance(model, context).or_else(|| {
        model
            .context(context)
            .map(|context| &context.declared_type.provenance)
    })
}

fn context_source_failure_provenance<'a>(
    model: &'a ApplicationSemanticModel,
    entry: &'a ContextSourcePlanEntry,
    code: &str,
) -> &'a SourceProvenance {
    if code == "EZC1066" {
        let dependent = match &entry.source {
            ContextValueSourceId::Provider(provider) => {
                ContextDependencyNodeId::Provider(provider.clone())
            }
            ContextValueSourceId::ContextDefault(context) => {
                ContextDependencyNodeId::ContextDefault(context.clone())
            }
        };
        if let Some(provenance) = entry.reasons.iter().find_map(|reason| {
            let dependency = match reason {
                ContextSourceBlockReason::MissingStateDependency(id) => {
                    ContextDependencyNodeId::State(id.clone())
                }
                ContextSourceBlockReason::UnavailableComputedDependency(id) => {
                    ContextDependencyNodeId::Computed(id.clone())
                }
                _ => return None,
            };
            model
                .context_dependency
                .edges
                .iter()
                .find(|edge| edge.dependent == dependent && edge.dependency == dependency)
                .map(|edge| &edge.provenance)
        }) {
            return provenance;
        }
    }
    if code == "EZC1067" {
        if let Some(provenance) = expression_provenance(model, &entry.expression_root) {
            return provenance;
        }
    }
    &entry.provenance
}

#[allow(clippy::too_many_lines)]
fn populate_secondary_labels(
    model: &ApplicationSemanticModel,
    diagnostics: &mut [ComponentDiagnostic],
) {
    for diagnostic in diagnostics {
        if diagnostic.code == "EZC1058" {
            let mut labels = diagnostic
                .consumer_id
                .as_ref()
                .and_then(|consumer| {
                    let crate::ContextResolution {
                        result: ContextResolutionResult::Ambiguous { providers, .. },
                        ..
                    } = model.context_resolutions.get(consumer)?
                    else {
                        return None;
                    };
                    Some(
                        providers
                            .iter()
                            .filter_map(|id| model.provider(id))
                            .map(|provider| crate::DiagnosticSecondaryLabel {
                                provenance: provider.provenance.clone(),
                                message: format!("Candidate Provider `{}`.", provider.id),
                            })
                            .collect::<Vec<_>>(),
                    )
                })
                .unwrap_or_default();
            labels.sort_by(|left, right| left.message.cmp(&right.message));
            labels.dedup();
            diagnostic.secondary_labels = labels;
            continue;
        }
        let mut labels = Vec::new();
        let context = diagnostic
            .context_id
            .as_ref()
            .and_then(|id| model.context(id));
        let provider = diagnostic
            .provider_id
            .as_ref()
            .and_then(|id| model.provider(id));
        match diagnostic.code.as_str() {
            "EZC1057" => {
                if let Some(context) = context {
                    labels.push(crate::DiagnosticSecondaryLabel {
                        provenance: context.provenance.clone(),
                        message: "Requested Context declaration.".to_string(),
                    });
                }
            }
            "EZC1059" => {
                if let Some(provider) = provider {
                    labels.push(crate::DiagnosticSecondaryLabel {
                        provenance: provider.declared_type.provenance.clone(),
                        message: "Provider declared type.".to_string(),
                    });
                }
            }
            "EZC1060" | "EZC1061" | "EZC1062" | "EZC1063" | "EZC1064" => {
                if let Some(context) = context {
                    labels.push(crate::DiagnosticSecondaryLabel {
                        provenance: context.declared_type.provenance.clone(),
                        message: "Context declared type.".to_string(),
                    });
                }
            }
            "EZC1065" => populate_lifetime_labels(model, diagnostic, &mut labels),
            "EZC1067" => {
                if let Some(provider) = provider {
                    labels.push(crate::DiagnosticSecondaryLabel {
                        provenance: provider.provenance.clone(),
                        message: "Context source declaration.".to_string(),
                    });
                } else if let Some(context) = context {
                    labels.push(crate::DiagnosticSecondaryLabel {
                        provenance: context.provenance.clone(),
                        message: "Context source declaration.".to_string(),
                    });
                }
            }
            _ => {}
        }
        if diagnostic.code == "EZC1066" {
            if let Some(entry) = model
                .context_evaluation
                .source_entries
                .values()
                .find(|entry| {
                    diagnostic.provenance.as_ref()
                        == Some(context_source_failure_provenance(model, entry, "EZC1066"))
                })
            {
                for reason in &entry.reasons {
                    let dependency = match reason {
                        ContextSourceBlockReason::MissingStateDependency(id)
                        | ContextSourceBlockReason::UnavailableComputedDependency(id) => Some(id),
                        _ => None,
                    };
                    if let Some(provenance) = dependency.and_then(|id| model.provenance(id)) {
                        labels.push(crate::DiagnosticSecondaryLabel {
                            provenance: provenance.clone(),
                            message: "Unavailable Context dependency.".to_string(),
                        });
                    }
                }
            }
        }
        labels.retain(|label| diagnostic.provenance.as_ref() != Some(&label.provenance));
        labels.sort_by(|left, right| {
            (
                &left.provenance.path,
                left.provenance.span.start,
                &left.message,
            )
                .cmp(&(
                    &right.provenance.path,
                    right.provenance.span.start,
                    &right.message,
                ))
        });
        labels.dedup();
        diagnostic.secondary_labels = labels;
    }
}

fn populate_lifetime_labels(
    model: &ApplicationSemanticModel,
    diagnostic: &ComponentDiagnostic,
    labels: &mut Vec<crate::DiagnosticSecondaryLabel>,
) {
    let Some(primary) = diagnostic.provenance.as_ref() else {
        return;
    };
    if let Some(record) = model
        .context_lifetime
        .dependency_lifetimes
        .iter()
        .find(|record| &record.provenance == primary)
    {
        if let Some(provenance) = context_dependency_node_provenance(model, &record.dependency) {
            labels.push(crate::DiagnosticSecondaryLabel {
                provenance: provenance.clone(),
                message: "Incompatible lifetime dependency.".to_string(),
            });
        }
        return;
    }
    let Some(record) = diagnostic
        .consumer_id
        .as_ref()
        .and_then(|consumer| model.context_lifetime.binding_lifetimes.get(consumer))
    else {
        return;
    };
    let provenance = match &record.source {
        Some(crate::ContextBindingLifetimeSource::Provider(provider)) => {
            model.provider(provider).map(|item| &item.provenance)
        }
        Some(crate::ContextBindingLifetimeSource::ContextDefault(context)) => {
            model.context(context).map(|item| &item.provenance)
        }
        None => None,
    };
    if let Some(provenance) = provenance {
        labels.push(crate::DiagnosticSecondaryLabel {
            provenance: provenance.clone(),
            message: "Selected Context source declaration.".to_string(),
        });
    }
}

fn context_dependency_node_provenance<'a>(
    model: &'a ApplicationSemanticModel,
    node: &ContextDependencyNodeId,
) -> Option<&'a SourceProvenance> {
    match node {
        ContextDependencyNodeId::State(id) | ContextDependencyNodeId::Computed(id) => {
            model.provenance(id)
        }
        ContextDependencyNodeId::Context(id) | ContextDependencyNodeId::ContextDefault(id) => {
            model.context(id).map(|item| &item.provenance)
        }
        ContextDependencyNodeId::Provider(id) => model.provider(id).map(|item| &item.provenance),
        ContextDependencyNodeId::Consumer(id) => model.consumer(id).map(|item| &item.provenance),
    }
}

#[allow(clippy::too_many_lines)]
fn populate_identities(model: &ApplicationSemanticModel, diagnostics: &mut [ComponentDiagnostic]) {
    for diagnostic in diagnostics {
        let Some(provenance) = diagnostic.provenance.as_ref() else {
            continue;
        };
        match diagnostic.code.as_str() {
            "EZC1057" | "EZC1058" => {
                if let Some(resolution) = model
                    .context_resolutions
                    .values()
                    .find(|record| record.provenance == *provenance)
                {
                    diagnostic.consumer_id = Some(resolution.consumer.clone());
                    diagnostic.context_id = resolution.context.clone();
                }
            }
            "EZC1059" | "EZC1060" => {
                if let Some(record) = model.provider_types.values().find(|record| {
                    let Some(provider) = model.provider(&record.provider) else {
                        return false;
                    };
                    let expected = if diagnostic.code == "EZC1059" {
                        expression_provenance(model, &provider.value_expression)
                            .unwrap_or(&record.provenance)
                    } else {
                        &provider.declared_type.provenance
                    };
                    expected == provenance
                }) {
                    diagnostic.provider_id = Some(record.provider.clone());
                    diagnostic.context_id = record.context.clone();
                }
            }
            "EZC1061" => {
                if let Some(record) = model.context_types.values().find(|record| {
                    context_default_provenance(model, &record.context).unwrap_or(&record.provenance)
                        == provenance
                }) {
                    diagnostic.context_id = Some(record.context.clone());
                }
            }
            "EZC1062" => {
                if let Some(record) = model.consumer_types.values().find(|record| {
                    model
                        .consumer(&record.consumer)
                        .map_or(&record.provenance, |consumer| {
                            &consumer.requested_type.provenance
                        })
                        == provenance
                }) {
                    diagnostic.consumer_id = Some(record.consumer.clone());
                    diagnostic.context_id = record.context.clone();
                }
            }
            "EZC1063" | "EZC1064" => {
                if let Some(record) = model.provider_types.values().find(|record| {
                    model
                        .provider(&record.provider)
                        .and_then(|provider| {
                            expression_provenance(model, &provider.value_expression)
                        })
                        .unwrap_or(&record.provenance)
                        == provenance
                }) {
                    diagnostic.provider_id = Some(record.provider.clone());
                    diagnostic.context_id = record.context.clone();
                } else if let Some(record) = model.context_types.values().find(|record| {
                    context_default_or_declared_type_provenance(model, &record.context)
                        .unwrap_or(&record.provenance)
                        == provenance
                }) {
                    diagnostic.context_id = Some(record.context.clone());
                }
            }
            "EZC1065" => {
                if let Some(record) = model
                    .context_lifetime
                    .binding_lifetimes
                    .values()
                    .find(|record| record.provenance == *provenance)
                {
                    diagnostic.consumer_id = Some(record.consumer.clone());
                    diagnostic.context_id = model
                        .context_resolutions
                        .get(&record.consumer)
                        .and_then(|resolution| resolution.context.clone());
                } else if let Some(record) = model
                    .context_lifetime
                    .dependency_lifetimes
                    .iter()
                    .find(|record| record.provenance == *provenance)
                {
                    match &record.dependent {
                        ContextDependencyNodeId::Provider(provider) => {
                            diagnostic.provider_id = Some(provider.clone());
                            diagnostic.context_id =
                                model.provider(provider).map(|item| item.context.clone());
                        }
                        ContextDependencyNodeId::ContextDefault(context)
                        | ContextDependencyNodeId::Context(context) => {
                            diagnostic.context_id = Some(context.clone());
                        }
                        ContextDependencyNodeId::Consumer(consumer) => {
                            diagnostic.consumer_id = Some(consumer.clone());
                            diagnostic.context_id = model
                                .consumer(consumer)
                                .and_then(|item| item.context().cloned());
                        }
                        ContextDependencyNodeId::State(_)
                        | ContextDependencyNodeId::Computed(_) => {}
                    }
                }
            }
            "EZC1066" | "EZC1067" => {
                if let Some(entry) =
                    model
                        .context_evaluation
                        .source_entries
                        .values()
                        .find(|entry| {
                            context_source_failure_provenance(model, entry, &diagnostic.code)
                                == provenance
                        })
                {
                    diagnostic.context_id = Some(entry.context.clone());
                    if let crate::ContextValueSourceId::Provider(provider) = &entry.source {
                        diagnostic.provider_id = Some(provider.clone());
                    }
                }
            }
            _ => {}
        }
    }
}

fn push(
    diagnostics: &mut Vec<ComponentDiagnostic>,
    code: &str,
    message: &str,
    provenance: &crate::SourceProvenance,
) {
    diagnostics.push(ComponentDiagnostic {
        code: code.to_string(),
        severity: ComponentDiagnosticSeverity::Error,
        message: message.to_string(),
        provenance: Some(provenance.clone()),
        effect_id: None,
        statement_id: None,
        context_declaration_candidate_id: None,
        context_id: None,
        provider_id: None,
        consumer_id: None,
        secondary_labels: Vec::new(),
    });
}

fn message(code: &str, field: Option<&str>) -> String {
    let subject = field.map_or("declaration".to_string(), |field| {
        format!("declaration `{field}`")
    });
    match code {
        "EZC1052" => format!("Invalid Context {subject}."),
        "EZC1053" => format!("Invalid Provider {subject}."),
        "EZC1054" => format!("Invalid Consumer {subject}."),
        "EZC1055" => format!("Unresolved Context designator for {subject}."),
        "EZC1056" => "Duplicate Provider declarations target the same Context.".to_string(),
        _ => unreachable!("frozen Context diagnostic code"),
    }
}

#[cfg(test)]
mod tests {
    use crate::{build_application_semantic_model, ComponentDiagnosticSeverity};

    fn codes(source: &str) -> Vec<crate::ComponentDiagnostic> {
        let model = build_application_semantic_model(&ezc_parser::parse_file(
            "src/context-diagnostics.tsx",
            source,
        ));
        assert_catalog_shapes(&model, &model.diagnostics);
        model.diagnostics
    }

    #[allow(clippy::too_many_lines)]
    fn assert_catalog_shapes(
        model: &crate::ApplicationSemanticModel,
        diagnostics: &[crate::ComponentDiagnostic],
    ) {
        let context_diagnostics = diagnostics
            .iter()
            .filter(|diagnostic| ("EZC1052"..="EZC1067").contains(&diagnostic.code.as_str()))
            .collect::<Vec<_>>();
        assert!(context_diagnostics.windows(2).all(|pair| {
            let key = |diagnostic: &crate::ComponentDiagnostic| {
                (
                    diagnostic
                        .provenance
                        .as_ref()
                        .map(|value| (value.path.clone(), value.span.start)),
                    diagnostic.code.clone(),
                )
            };
            key(pair[0]) <= key(pair[1])
        }));
        for diagnostic in context_diagnostics {
            assert_eq!(diagnostic.severity, ComponentDiagnosticSeverity::Error);
            assert!(diagnostic.effect_id.is_none() && diagnostic.statement_id.is_none());
            let primary = diagnostic.provenance.as_ref().expect("Context primary");
            let mut expected_labels = Vec::new();
            match diagnostic.code.as_str() {
                "EZC1052" | "EZC1053" | "EZC1054" | "EZC1055" | "EZC1056" => {
                    let candidate_id = diagnostic
                        .context_declaration_candidate_id
                        .as_ref()
                        .expect("declaration candidate identity");
                    let candidate = model
                        .context_declaration_candidates()
                        .candidate(candidate_id)
                        .expect("retained declaration candidate");
                    let crate::ContextDeclarationStatus::Invalid(violations) = &candidate.status
                    else {
                        panic!("diagnostic candidate must be invalid");
                    };
                    let expected = match violations.first().unwrap() {
                        crate::ContextDeclarationViolation::StaticDeclarationUnsupported => {
                            candidate.authored.static_modifier_provenance.as_ref()
                        }
                        crate::ContextDeclarationViolation::UnsupportedInitializer
                        | crate::ContextDeclarationViolation::ForbiddenInitializer
                        | crate::ContextDeclarationViolation::MissingInitializer => {
                            candidate.authored.initializer_provenance.as_ref()
                        }
                        crate::ContextDeclarationViolation::ContextDesignatorUnsupported
                        | crate::ContextDeclarationViolation::UnresolvedContextDesignator => {
                            candidate
                                .authored
                                .context_designator
                                .as_ref()
                                .map(|designator| &designator.provenance)
                        }
                        _ => Some(&candidate.authored.decorator_provenance),
                    }
                    .unwrap_or(&candidate.authored.provenance);
                    assert_eq!(primary, expected);
                    assert!(
                        diagnostic.context_id.is_none()
                            && diagnostic.provider_id.is_none()
                            && diagnostic.consumer_id.is_none()
                    );
                }
                "EZC1057" | "EZC1058" => {
                    assert!(diagnostic.context_declaration_candidate_id.is_none());
                    assert!(diagnostic.provider_id.is_none());
                    let consumer = diagnostic.consumer_id.as_ref().expect("Consumer identity");
                    let resolution = model
                        .context_resolutions
                        .get(consumer)
                        .expect("G4 resolution");
                    assert_eq!(primary, &resolution.provenance);
                    assert_eq!(diagnostic.context_id, resolution.context);
                    if diagnostic.code == "EZC1057" {
                        if let Some(context) = diagnostic
                            .context_id
                            .as_ref()
                            .and_then(|id| model.context(id))
                        {
                            expected_labels.push(crate::DiagnosticSecondaryLabel {
                                provenance: context.provenance.clone(),
                                message: "Requested Context declaration.".to_string(),
                            });
                        }
                    } else {
                        let crate::ContextResolutionResult::Ambiguous { providers, .. } =
                            &resolution.result
                        else {
                            panic!("EZC1058 requires G4 ambiguity");
                        };
                        expected_labels.extend(providers.iter().filter_map(|id| {
                            model
                                .provider(id)
                                .map(|provider| crate::DiagnosticSecondaryLabel {
                                    provenance: provider.provenance.clone(),
                                    message: format!("Candidate Provider `{}`.", provider.id),
                                })
                        }));
                        expected_labels.sort_by(|left, right| left.message.cmp(&right.message));
                        expected_labels.dedup();
                    }
                }
                "EZC1059" | "EZC1060" => {
                    assert!(diagnostic.context_declaration_candidate_id.is_none());
                    assert!(diagnostic.consumer_id.is_none());
                    let provider_id = diagnostic.provider_id.as_ref().expect("Provider identity");
                    let provider = model.provider(provider_id).expect("Provider entity");
                    assert_eq!(diagnostic.context_id.as_ref(), Some(&provider.context));
                    if diagnostic.code == "EZC1059" {
                        assert_eq!(
                            primary,
                            super::expression_provenance(model, &provider.value_expression)
                                .unwrap()
                        );
                        expected_labels.push(crate::DiagnosticSecondaryLabel {
                            provenance: provider.declared_type.provenance.clone(),
                            message: "Provider declared type.".to_string(),
                        });
                    } else {
                        assert_eq!(primary, &provider.declared_type.provenance);
                        let context = model.context(&provider.context).unwrap();
                        expected_labels.push(crate::DiagnosticSecondaryLabel {
                            provenance: context.declared_type.provenance.clone(),
                            message: "Context declared type.".to_string(),
                        });
                    }
                }
                "EZC1061" => {
                    assert!(diagnostic.context_declaration_candidate_id.is_none());
                    assert!(diagnostic.provider_id.is_none() && diagnostic.consumer_id.is_none());
                    let context = model
                        .context(diagnostic.context_id.as_ref().expect("Context identity"))
                        .unwrap();
                    assert_eq!(
                        primary,
                        super::expression_provenance(
                            model,
                            context.default_expression.as_ref().unwrap(),
                        )
                        .unwrap()
                    );
                    expected_labels.push(crate::DiagnosticSecondaryLabel {
                        provenance: context.declared_type.provenance.clone(),
                        message: "Context declared type.".to_string(),
                    });
                }
                "EZC1062" => {
                    let consumer = model
                        .consumer(diagnostic.consumer_id.as_ref().expect("Consumer identity"))
                        .unwrap();
                    assert_eq!(primary, &consumer.requested_type.provenance);
                    assert_eq!(diagnostic.context_id.as_ref(), consumer.context());
                    let context = model.context(consumer.context().unwrap()).unwrap();
                    expected_labels.push(crate::DiagnosticSecondaryLabel {
                        provenance: context.declared_type.provenance.clone(),
                        message: "Context declared type.".to_string(),
                    });
                }
                "EZC1063" | "EZC1064" => {
                    let context = model
                        .context(diagnostic.context_id.as_ref().expect("Context identity"))
                        .unwrap();
                    if let Some(provider) = diagnostic
                        .provider_id
                        .as_ref()
                        .and_then(|id| model.provider(id))
                    {
                        assert_eq!(
                            primary,
                            super::expression_provenance(model, &provider.value_expression)
                                .unwrap()
                        );
                    } else if let Some(default) = &context.default_expression {
                        assert_eq!(
                            primary,
                            super::expression_provenance(model, default).unwrap()
                        );
                    } else {
                        assert_eq!(primary, &context.declared_type.provenance);
                    }
                    expected_labels.push(crate::DiagnosticSecondaryLabel {
                        provenance: context.declared_type.provenance.clone(),
                        message: "Context declared type.".to_string(),
                    });
                }
                "EZC1065" => {
                    let consumer = diagnostic.consumer_id.as_ref().expect("Consumer identity");
                    let record = model
                        .context_lifetime
                        .binding_lifetimes
                        .get(consumer)
                        .expect("G8 binding lifetime");
                    assert_eq!(primary, &record.provenance);
                    assert_eq!(
                        diagnostic.context_id,
                        model
                            .context_resolutions
                            .get(consumer)
                            .and_then(|resolution| resolution.context.clone())
                    );
                    if let Some(crate::ContextBindingLifetimeSource::Provider(provider)) =
                        &record.source
                    {
                        expected_labels.push(crate::DiagnosticSecondaryLabel {
                            provenance: model.provider(provider).unwrap().provenance.clone(),
                            message: "Selected Context source declaration.".to_string(),
                        });
                    }
                }
                "EZC1066" | "EZC1067" => {
                    let entry = model
                        .context_evaluation
                        .source_entries
                        .values()
                        .find(|entry| {
                            super::context_source_failure_provenance(model, entry, &diagnostic.code)
                                == primary
                        })
                        .expect("G9 source failure");
                    assert_eq!(diagnostic.context_id.as_ref(), Some(&entry.context));
                    match &entry.source {
                        crate::ContextValueSourceId::Provider(provider) => {
                            assert_eq!(diagnostic.provider_id.as_ref(), Some(provider));
                            if diagnostic.code == "EZC1067" {
                                expected_labels.push(crate::DiagnosticSecondaryLabel {
                                    provenance: model
                                        .provider(provider)
                                        .unwrap()
                                        .provenance
                                        .clone(),
                                    message: "Context source declaration.".to_string(),
                                });
                            }
                        }
                        crate::ContextValueSourceId::ContextDefault(_) => {
                            assert!(diagnostic.provider_id.is_none());
                            if diagnostic.code == "EZC1067" {
                                expected_labels.push(crate::DiagnosticSecondaryLabel {
                                    provenance: model
                                        .context(&entry.context)
                                        .unwrap()
                                        .provenance
                                        .clone(),
                                    message: "Context source declaration.".to_string(),
                                });
                            }
                        }
                    }
                }
                _ => unreachable!("stable Context diagnostic code"),
            }
            expected_labels.retain(|label| &label.provenance != primary);
            expected_labels.sort_by(|left, right| {
                (
                    &left.provenance.path,
                    left.provenance.span.start,
                    &left.message,
                )
                    .cmp(&(
                        &right.provenance.path,
                        right.provenance.span.start,
                        &right.message,
                    ))
            });
            expected_labels.dedup();
            assert_eq!(diagnostic.secondary_labels, expected_labels);
        }
    }

    #[test]
    fn ezc1052_invalid_context_declaration_has_candidate_identity_and_no_semantic_identity() {
        let diagnostics = codes(
            r#"
@component("x-app") class App extends Component {
  @context("invalid") theme!: string;
  render() { return <main />; }
}"#,
        );
        let diagnostic = diagnostics
            .iter()
            .find(|item| item.code == "EZC1052")
            .unwrap();
        assert_eq!(diagnostic.severity, ComponentDiagnosticSeverity::Error);
        assert!(diagnostic.provenance.is_some());
        assert!(diagnostic.context_declaration_candidate_id.is_some());
        assert!(
            diagnostic.context_id.is_none()
                && diagnostic.provider_id.is_none()
                && diagnostic.consumer_id.is_none()
        );
        assert!(diagnostic.secondary_labels.is_empty());
        assert!(!diagnostics.iter().any(|item| item.code == "EZC1055"));
    }

    #[test]
    fn ezc1053_and_ezc1055_keep_declaration_and_designator_failures_distinct() {
        let diagnostics = codes(
            r#"
@component("x-app") class App extends Component {
  @provide(Missing.theme) value: string = "x";
  render() { return <main />; }
}"#,
        );
        let diagnostic = diagnostics
            .iter()
            .find(|item| item.code == "EZC1055")
            .unwrap();
        assert_eq!(diagnostic.severity, ComponentDiagnosticSeverity::Error);
        assert!(diagnostic.context_declaration_candidate_id.is_some());
        assert!(diagnostic.provenance.is_some());
        assert!(!diagnostics.iter().any(|item| item.code == "EZC1053"));
    }

    #[test]
    fn ezc1053_invalid_provider_declaration_has_only_candidate_identity() {
        let diagnostics = codes(
            r#"
@component("x-app") class App extends Component { @provide() value: string = "x"; render() { return <main />; } }
"#,
        );
        let diagnostic = diagnostics
            .iter()
            .find(|item| item.code == "EZC1053")
            .unwrap();
        assert_eq!(diagnostic.severity, ComponentDiagnosticSeverity::Error);
        assert!(diagnostic.context_declaration_candidate_id.is_some());
        assert!(
            diagnostic.context_id.is_none()
                && diagnostic.provider_id.is_none()
                && diagnostic.consumer_id.is_none()
        );
        assert!(!diagnostics.iter().any(|item| item.code == "EZC1055"));
    }

    #[test]
    fn ezc1056_duplicate_provider_group_has_one_deterministic_diagnostic() {
        let diagnostics = codes(
            r#"
@component("x-app") class App extends Component {
  @context() theme: string;
  @provide(App.theme) second: string = "b";
  @provide(App.theme) first: string = "a";
  render() { return <main />; }
}"#,
        );
        let matches = diagnostics
            .iter()
            .filter(|item| item.code == "EZC1056")
            .collect::<Vec<_>>();
        assert_eq!(matches.len(), 1);
        assert!(matches[0].context_declaration_candidate_id.is_some());
        assert!(matches[0].provider_id.is_none());
        assert!(!diagnostics.iter().any(|item| item.code == "EZC1058"));
    }

    #[test]
    fn ezc1054_invalid_consumer_declaration_suppresses_binding_diagnostics() {
        let diagnostics = codes(
            r#"
@component("x-app") class App extends Component {
  @consume(App.theme) theme: string = "x";
  render() { return <main />; }
}"#,
        );
        let diagnostic = diagnostics
            .iter()
            .find(|item| item.code == "EZC1054")
            .unwrap();
        assert!(diagnostic.context_declaration_candidate_id.is_some());
        assert!(!diagnostics.iter().any(|item| matches!(
            item.code.as_str(),
            "EZC1057" | "EZC1058" | "EZC1062" | "EZC1065" | "EZC1066" | "EZC1067"
        )));
    }

    #[test]
    fn ezc1059_and_ezc1060_coexist_for_independent_provider_contract_failures() {
        let diagnostics = codes(
            r#"
@component("x-app") class App extends Component {
  @context() theme: string;
  @provide(App.theme) provided: boolean = "x";
  render() { return <main />; }
}"#,
        );
        for code in ["EZC1059", "EZC1060"] {
            let diagnostic = diagnostics.iter().find(|item| item.code == code).unwrap();
            assert_eq!(diagnostic.severity, ComponentDiagnosticSeverity::Error);
            assert!(diagnostic.provider_id.is_some() && diagnostic.context_id.is_some());
            assert!(diagnostic.provenance.is_some());
        }
        assert!(!diagnostics
            .iter()
            .any(|item| matches!(item.code.as_str(), "EZC1066" | "EZC1067")));
    }

    #[test]
    fn ezc1059_value_mismatch_has_provider_and_context_identities() {
        let diagnostics = codes(
            r#"
@component("x-app") class App extends Component { @context() theme: boolean; @provide(App.theme) provided: boolean = "x"; render() { return <main />; } }
"#,
        );
        let diagnostic = diagnostics
            .iter()
            .find(|item| item.code == "EZC1059")
            .unwrap();
        assert!(diagnostic.provider_id.is_some() && diagnostic.context_id.is_some());
        assert!(!diagnostics.iter().any(|item| item.code == "EZC1060"));
    }

    #[test]
    fn ezc1060_declaration_context_mismatch_has_provider_and_context_identities() {
        let diagnostics = codes(
            r#"
@component("x-app") class App extends Component { @context() theme: string; @provide(App.theme) provided: boolean = true; render() { return <main />; } }
"#,
        );
        let diagnostic = diagnostics
            .iter()
            .find(|item| item.code == "EZC1060")
            .unwrap();
        assert!(diagnostic.provider_id.is_some() && diagnostic.context_id.is_some());
        assert!(!diagnostics.iter().any(|item| item.code == "EZC1059"));
    }

    #[test]
    fn ezc1061_context_default_mismatch_has_context_identity() {
        let diagnostics = codes(
            r#"
@component("x-app") class App extends Component {
  @context() theme: number = "x";
  render() { return <main />; }
}"#,
        );
        let diagnostic = diagnostics
            .iter()
            .find(|item| item.code == "EZC1061")
            .unwrap();
        assert_eq!(diagnostic.severity, ComponentDiagnosticSeverity::Error);
        assert!(diagnostic.context_id.is_some() && diagnostic.provenance.is_some());
        assert!(!diagnostics.iter().any(|item| matches!(
            item.code.as_str(),
            "EZC1063" | "EZC1064" | "EZC1066" | "EZC1067"
        )));
    }

    #[test]
    fn ezc1062_context_consumer_mismatch_has_both_canonical_identities() {
        let diagnostics = codes(
            r#"
@component("x-app") class App extends Component {
  @context() theme: string;
  @consume(App.theme) selected!: number;
  render() { return <main />; }
}"#,
        );
        let diagnostic = diagnostics
            .iter()
            .find(|item| item.code == "EZC1062")
            .unwrap();
        assert_eq!(diagnostic.severity, ComponentDiagnosticSeverity::Error);
        assert!(diagnostic.context_id.is_some() && diagnostic.consumer_id.is_some());
        assert!(diagnostic.provenance.is_some());
    }

    #[test]
    fn ezc1057_projects_only_an_explicit_unresolved_g4_result() {
        let mut model = build_application_semantic_model(&ezc_parser::parse_file(
            "src/g4.tsx",
            r#"
@component("x-app") class App extends Component {
  @context() theme: string;
  @consume(App.theme) selected!: string;
  render() { return <main />; }
}"#,
        ));
        let consumer = model.consumers()[0].id.clone();
        let resolution = model.context_resolutions.get_mut(&consumer).unwrap();
        resolution.result = crate::ContextResolutionResult::Unresolved;
        let diagnostics = super::collect_context_diagnostics(&model);
        assert_catalog_shapes(&model, &diagnostics);
        let diagnostic = diagnostics
            .iter()
            .find(|item| item.code == "EZC1057")
            .unwrap();
        assert_eq!(diagnostic.severity, ComponentDiagnosticSeverity::Error);
        assert_eq!(diagnostic.consumer_id, Some(consumer));
        assert!(diagnostic.context_id.is_some());
        assert!(!diagnostics.iter().any(|item| matches!(
            item.code.as_str(),
            "EZC1062" | "EZC1065" | "EZC1066" | "EZC1067"
        )));
    }

    #[test]
    fn ezc1058_emits_one_ambiguity_diagnostic_with_sorted_provider_labels() {
        let mut model = build_application_semantic_model(&ezc_parser::parse_file(
            "src/g4-ambiguous.tsx",
            r#"
@component("x-app") class App extends Component {
  @context() theme: string;
  @provide(App.theme) first: string = "a";
  @consume(App.theme) selected!: string;
  render() { return <main />; }
}"#,
        ));
        let consumer = model.consumers()[0].id.clone();
        let first = model.providers()[0].id.clone();
        let mut second_entity = model.providers()[0].clone();
        let second = crate::ProviderId::for_component(&model.components[0].id, "second");
        second_entity.id = second.clone();
        second_entity.provenance.span.start += 1;
        second_entity.provenance.span.end += 1;
        model.provenance.insert(
            second.as_semantic_id().clone(),
            second_entity.provenance.clone(),
        );
        model.providers.insert(second.clone(), second_entity);
        model.context_resolutions.get_mut(&consumer).unwrap().result =
            crate::ContextResolutionResult::Ambiguous {
                providers: vec![second, first],
                distance: 0,
            };
        let diagnostics = super::collect_context_diagnostics(&model);
        assert_catalog_shapes(&model, &diagnostics);
        let matches = diagnostics
            .iter()
            .filter(|item| item.code == "EZC1058")
            .collect::<Vec<_>>();
        assert_eq!(matches.len(), 1);
        assert_eq!(matches[0].consumer_id, Some(consumer));
        assert!(matches[0].provider_id.is_none());
        assert_eq!(matches[0].secondary_labels.len(), 2);
        assert!(!diagnostics.iter().any(|item| matches!(
            item.code.as_str(),
            "EZC1062" | "EZC1065" | "EZC1066" | "EZC1067"
        )));
        let mut validation_model = model.clone();
        validation_model.diagnostics.clone_from(&diagnostics);
        assert!(
            !crate::validate_application_semantic_model(&validation_model)
                .iter()
                .any(|item| item.code == "EZASM1134")
        );
        validation_model
            .diagnostics
            .iter_mut()
            .find(|item| item.code == "EZC1058")
            .unwrap()
            .secondary_labels
            .reverse();
        assert!(
            crate::validate_application_semantic_model(&validation_model)
                .iter()
                .any(|item| item.code == "EZASM1134")
        );
    }

    #[test]
    fn ezc1063_serialization_suppresses_generic_planning_codes() {
        let mut model = build_application_semantic_model(&ezc_parser::parse_file(
            "src/g5-serialization.tsx",
            r#"@component("x-app") class App extends Component { @context() theme: string; render() { return <main />; } }"#,
        ));
        let context = model.contexts()[0].id.clone();
        model.context_types.get_mut(&context).unwrap().serialization =
            crate::ContextSerializationCompatibility::NonSerializable;
        let diagnostics = super::collect_context_diagnostics(&model);
        assert_catalog_shapes(&model, &diagnostics);
        assert_eq!(
            diagnostics
                .iter()
                .find(|item| item.code == "EZC1063")
                .unwrap()
                .context_id,
            Some(context)
        );
        assert!(!diagnostics
            .iter()
            .any(|item| matches!(item.code.as_str(), "EZC1066" | "EZC1067")));
    }

    #[test]
    fn ezc1064_boundary_suppresses_generic_planning_codes() {
        let mut model = build_application_semantic_model(&ezc_parser::parse_file(
            "src/g5-boundary.tsx",
            r#"@component("x-app") class App extends Component { @context() theme: string; render() { return <main />; } }"#,
        ));
        let context = model.contexts()[0].id.clone();
        model
            .context_types
            .get_mut(&context)
            .unwrap()
            .boundary_compatibility = crate::CompatibilityStatus::Incompatible;
        let diagnostics = super::collect_context_diagnostics(&model);
        assert_catalog_shapes(&model, &diagnostics);
        assert_eq!(
            diagnostics
                .iter()
                .find(|item| item.code == "EZC1064")
                .unwrap()
                .context_id,
            Some(context)
        );
        assert!(!diagnostics
            .iter()
            .any(|item| matches!(item.code.as_str(), "EZC1066" | "EZC1067")));
    }

    fn planned_provider_model() -> crate::ApplicationSemanticModel {
        build_application_semantic_model(&ezc_parser::parse_file(
            "src/g9.tsx",
            r#"
@component("x-app") class App extends Component {
  @context() theme: string;
  @provide(App.theme) provided: string = "x";
  @consume(App.theme) selected!: string;
  render() { return <main />; }
}"#,
        ))
    }

    #[test]
    fn ezc1065_projects_explicit_g8_binding_lifetime_failure() {
        let mut model = planned_provider_model();
        let consumer = model.consumers()[0].id.clone();
        model
            .context_lifetime
            .binding_lifetimes
            .get_mut(&consumer)
            .unwrap()
            .compatibility = crate::ContextBindingLifetimeStatus::Incompatible;
        let diagnostics = super::collect_context_diagnostics(&model);
        assert_catalog_shapes(&model, &diagnostics);
        let diagnostic = diagnostics
            .iter()
            .find(|item| item.code == "EZC1065")
            .unwrap();
        assert_eq!(diagnostic.consumer_id, Some(consumer));
        assert!(diagnostic.context_id.is_some());
        assert!(!diagnostics
            .iter()
            .any(|item| matches!(item.code.as_str(), "EZC1066" | "EZC1067")));
    }

    #[test]
    fn ezc1066_unavailable_dependency_does_not_imply_ezc1067() {
        let mut model = planned_provider_model();
        let entry = model
            .context_evaluation
            .source_entries
            .values_mut()
            .next()
            .unwrap();
        entry.status = crate::ContextSourcePlanStatus::BlockedDependency;
        entry.reasons = vec![
            crate::ContextSourceBlockReason::UnavailableComputedDependency(
                model.components[0].id.computed("missing"),
            ),
        ];
        let diagnostics = super::collect_context_diagnostics(&model);
        assert_catalog_shapes(&model, &diagnostics);
        let diagnostic = diagnostics
            .iter()
            .find(|item| item.code == "EZC1066")
            .unwrap();
        assert!(diagnostic.context_id.is_some() && diagnostic.provider_id.is_some());
        assert!(!diagnostics.iter().any(|item| item.code == "EZC1067"));
    }

    #[test]
    fn ezc1067_requires_explicit_retained_unsupported_expression_reason() {
        let mut model = planned_provider_model();
        let entry = model
            .context_evaluation
            .source_entries
            .values_mut()
            .next()
            .unwrap();
        entry.status = crate::ContextSourcePlanStatus::BlockedUnsupportedExpression;
        entry.reasons = vec![crate::ContextSourceBlockReason::UnsupportedExpression];
        let diagnostics = super::collect_context_diagnostics(&model);
        assert_catalog_shapes(&model, &diagnostics);
        let diagnostic = diagnostics
            .iter()
            .find(|item| item.code == "EZC1067")
            .unwrap();
        assert!(diagnostic.context_id.is_some() && diagnostic.provider_id.is_some());
        let mut without_reason = model.clone();
        without_reason
            .context_evaluation
            .source_entries
            .values_mut()
            .next()
            .unwrap()
            .reasons
            .clear();
        assert!(!super::collect_context_diagnostics(&without_reason)
            .iter()
            .any(|item| item.code == "EZC1067"));
    }

    #[test]
    fn unused_context_source_emits_no_diagnostic() {
        let model = build_application_semantic_model(&ezc_parser::parse_file(
            "src/unused.tsx",
            r#"
@component("x-app") class App extends Component { @context() theme: string; @provide(App.theme) provided: string = "x"; render() { return <main />; } }
"#,
        ));
        assert!(!super::collect_context_diagnostics(&model)
            .iter()
            .any(|item| ("EZC1052"..="EZC1067").contains(&item.code.as_str())));
    }

    #[test]
    fn validation_rejects_unknown_candidate_and_fabricated_semantic_identity() {
        let mut model = build_application_semantic_model(&ezc_parser::parse_file(
            "src/invalid-validation.tsx",
            r#"
@component("x-app") class App extends Component { @context("bad") theme: string; render() { return <main />; } }
"#,
        ));
        let diagnostic = model
            .diagnostics
            .iter_mut()
            .find(|item| item.code == "EZC1052")
            .unwrap();
        diagnostic.context_declaration_candidate_id = Some(
            crate::ContextDeclarationCandidateId::for_component_position(
                &model.components[0].id,
                usize::MAX,
            ),
        );
        diagnostic.context_id = Some(crate::ContextId::for_component(
            &model.components[0].id,
            "fabricated",
        ));
        let validation = crate::validate_application_semantic_model(&model);
        assert!(validation.iter().any(|item| item.code == "EZASM1135"));
        assert!(validation.iter().any(|item| item.code == "EZASM1136"));
    }

    #[test]
    fn validation_rejects_wrong_provider_and_consumer_context_relationships() {
        let mut model = build_application_semantic_model(&ezc_parser::parse_file(
            "src/wrong-context.tsx",
            r#"
@component("x-app") class App extends Component {
  @context() first: string; @context() second: string;
  @provide(App.first) provided: boolean = true;
  @consume(App.first) selected!: number;
  render() { return <main />; }
}"#,
        ));
        let wrong = model
            .contexts()
            .iter()
            .find(|context| context.name == "second")
            .unwrap()
            .id
            .clone();
        model
            .diagnostics
            .iter_mut()
            .filter(|item| matches!(item.code.as_str(), "EZC1060" | "EZC1062"))
            .for_each(|item| item.context_id = Some(wrong.clone()));
        let validation = crate::validate_application_semantic_model(&model);
        assert!(validation.iter().any(|item| item.code == "EZASM1138"));
        assert!(validation.iter().any(|item| item.code == "EZASM1139"));
    }

    #[test]
    fn validation_rejects_unsorted_duplicate_and_primary_repeating_labels() {
        let mut model = build_application_semantic_model(&ezc_parser::parse_file(
            "src/label-validation.tsx",
            r#"
@component("x-app") class App extends Component { @context() theme: string; @provide(App.theme) provided: boolean = true; render() { return <main />; } }
"#,
        ));
        let diagnostic = model
            .diagnostics
            .iter_mut()
            .find(|item| item.code == "EZC1060")
            .unwrap();
        let primary = diagnostic.provenance.clone().unwrap();
        diagnostic.secondary_labels = vec![
            crate::DiagnosticSecondaryLabel {
                provenance: primary.clone(),
                message: "z".to_string(),
            },
            crate::DiagnosticSecondaryLabel {
                provenance: primary,
                message: "z".to_string(),
            },
        ];
        let validation = crate::validate_application_semantic_model(&model);
        assert!(validation.iter().any(|item| item.code == "EZASM1133"));
        assert!(validation.iter().any(|item| item.code == "EZASM1137"));
    }

    #[test]
    fn validation_rejects_noncanonical_context_primary_provenance() {
        let mut model = build_application_semantic_model(&ezc_parser::parse_file(
            "src/primary-validation.tsx",
            r#"
@component("x-app") class App extends Component { @context("bad") theme: string; render() { return <main />; } }
"#,
        ));
        let diagnostic = model
            .diagnostics
            .iter_mut()
            .find(|item| item.code == "EZC1052")
            .unwrap();
        diagnostic.provenance.as_mut().unwrap().span.start += 1;
        let validation = crate::validate_application_semantic_model(&model);
        assert!(validation.iter().any(|item| item.code == "EZASM1140"));
    }

    #[test]
    fn validation_rejects_noncanonical_context_secondary_label_provenance() {
        let mut model = build_application_semantic_model(&ezc_parser::parse_file(
            "src/secondary-validation.tsx",
            r#"
@component("x-app") class App extends Component { @context() theme: string; @provide(App.theme) provided: boolean = true; render() { return <main />; } }
"#,
        ));
        let diagnostic = model
            .diagnostics
            .iter_mut()
            .find(|item| item.code == "EZC1060")
            .unwrap();
        diagnostic.secondary_labels[0].provenance.span.start += 1;
        let validation = crate::validate_application_semantic_model(&model);
        assert!(validation.iter().any(|item| item.code == "EZASM1134"));
    }
}
