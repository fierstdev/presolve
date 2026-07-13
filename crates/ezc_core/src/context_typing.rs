use std::collections::BTreeMap;

use crate::{
    boundary_compatibility, is_assignable, serialization_compatibility, ConsumerEntity, ConsumerId,
    ContextEntity, ContextId, ContextResolution, ContextResolutionResult, ExecutionBoundary,
    ExpressionGraph, ProviderEntity, ProviderId, SemanticType, SemanticTypeId, SemanticTypeModel,
    SerializationCompatibility, SourceProvenance,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CompatibilityStatus {
    Compatible,
    Incompatible,
    Unknown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ContextSerializationCompatibility {
    Serializable,
    NonSerializable,
    Unknown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ContextBindingCompatibility {
    Compatible,
    Incompatible,
    Unknown,
    Unresolved,
    Ambiguous,
    InvalidContextReference,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ContextTypeRecord {
    pub context: ContextId,
    pub declared_type: SemanticTypeId,
    pub normalized_type: SemanticTypeId,
    pub default_type: Option<SemanticTypeId>,
    pub default_compatibility: Option<CompatibilityStatus>,
    pub serialization: ContextSerializationCompatibility,
    pub boundary: ExecutionBoundary,
    pub boundary_compatibility: CompatibilityStatus,
    pub provenance: SourceProvenance,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProviderTypeRecord {
    pub provider: ProviderId,
    pub context: Option<ContextId>,
    pub inferred_value_type: SemanticTypeId,
    pub declared_type: SemanticTypeId,
    pub value_to_declaration: CompatibilityStatus,
    pub declaration_to_context: CompatibilityStatus,
    pub serialization: ContextSerializationCompatibility,
    pub boundary: ExecutionBoundary,
    pub boundary_compatibility: CompatibilityStatus,
    pub provenance: SourceProvenance,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConsumerTypeRecord {
    pub consumer: ConsumerId,
    pub context: Option<ContextId>,
    pub requested_type: SemanticTypeId,
    pub context_to_consumer: CompatibilityStatus,
    pub boundary: ExecutionBoundary,
    pub boundary_compatibility: CompatibilityStatus,
    pub provenance: SourceProvenance,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ContextBindingTypeRecord {
    pub consumer: ConsumerId,
    pub resolution: ContextResolutionResult,
    pub provider: Option<ProviderId>,
    pub context: Option<ContextId>,
    pub source_type: Option<SemanticTypeId>,
    pub context_type: Option<SemanticTypeId>,
    pub consumer_type: SemanticTypeId,
    pub source_to_context: CompatibilityStatus,
    pub context_to_consumer: CompatibilityStatus,
    pub overall: ContextBindingCompatibility,
    pub serialization: ContextSerializationCompatibility,
    pub boundary_compatibility: CompatibilityStatus,
    pub provenance: SourceProvenance,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ContextTypeProducts {
    pub contexts: BTreeMap<ContextId, ContextTypeRecord>,
    pub providers: BTreeMap<ProviderId, ProviderTypeRecord>,
    pub consumers: BTreeMap<ConsumerId, ConsumerTypeRecord>,
    pub bindings: BTreeMap<ConsumerId, ContextBindingTypeRecord>,
}

#[allow(clippy::too_many_lines)]
#[must_use]
pub fn collect_context_type_products(
    contexts: &BTreeMap<ContextId, ContextEntity>,
    providers: &BTreeMap<ProviderId, ProviderEntity>,
    consumers: &BTreeMap<ConsumerId, ConsumerEntity>,
    resolutions: &BTreeMap<ConsumerId, ContextResolution>,
    expression_graph: &ExpressionGraph,
    semantic_types: &SemanticTypeModel,
) -> ContextTypeProducts {
    let context_records = contexts
        .values()
        .map(|context| {
            let default_root = context
                .default_expression
                .as_ref()
                .and_then(|_| expression_graph.root_for(context.id.as_semantic_id()))
                .cloned();
            let default_type = default_root.as_ref().map(SemanticTypeId::for_subject);
            let default_compatibility = default_root.as_ref().map(|default| {
                compatibility_for_ids(semantic_types, default, context.id.as_semantic_id())
            });
            let mut serial_types = vec![context.id.as_semantic_id()];
            if let Some(default) = &default_root {
                serial_types.push(default);
            }
            let serialization = serialization_for_ids(semantic_types, &serial_types);
            (
                context.id.clone(),
                ContextTypeRecord {
                    context: context.id.clone(),
                    declared_type: context.declared_type_id.clone(),
                    normalized_type: context.declared_type_id.clone(),
                    default_type,
                    default_compatibility,
                    serialization,
                    boundary: context.execution_boundary,
                    boundary_compatibility: boundary_for_id(
                        semantic_types,
                        context.id.as_semantic_id(),
                        context.execution_boundary,
                        context.execution_boundary,
                    ),
                    provenance: context.provenance.clone(),
                },
            )
        })
        .collect::<BTreeMap<_, _>>();
    let provider_records = providers
        .values()
        .map(|provider| {
            let value_root = expression_graph
                .root_for(provider.id.as_semantic_id())
                .cloned();
            let inferred_value_type = value_root.as_ref().map_or_else(
                || SemanticTypeId::for_subject(provider.id.as_semantic_id()),
                SemanticTypeId::for_subject,
            );
            let value_to_declaration = compatibility_for_ids(
                semantic_types,
                value_root.as_ref().unwrap_or(provider.id.as_semantic_id()),
                provider.id.as_semantic_id(),
            );
            let declaration_to_context = compatibility_for_ids(
                semantic_types,
                provider.id.as_semantic_id(),
                provider.context.as_semantic_id(),
            );
            (
                provider.id.clone(),
                ProviderTypeRecord {
                    provider: provider.id.clone(),
                    context: Some(provider.context.clone()),
                    inferred_value_type: inferred_value_type.clone(),
                    declared_type: provider.declared_type_id.clone(),
                    value_to_declaration,
                    declaration_to_context,
                    serialization: serialization_for_ids(
                        semantic_types,
                        &[
                            value_root.as_ref().unwrap_or(provider.id.as_semantic_id()),
                            provider.id.as_semantic_id(),
                        ],
                    ),
                    boundary: provider.execution_boundary,
                    boundary_compatibility: boundary_for_id(
                        semantic_types,
                        value_root.as_ref().unwrap_or(provider.id.as_semantic_id()),
                        provider.execution_boundary,
                        provider.execution_boundary,
                    ),
                    provenance: provider.provenance.clone(),
                },
            )
        })
        .collect::<BTreeMap<_, _>>();
    let consumer_records = consumers
        .values()
        .map(|consumer| {
            let context_to_consumer =
                consumer
                    .context()
                    .map_or(CompatibilityStatus::Unknown, |context| {
                        compatibility_for_ids(
                            semantic_types,
                            context.as_semantic_id(),
                            consumer.id.as_semantic_id(),
                        )
                    });
            (
                consumer.id.clone(),
                ConsumerTypeRecord {
                    consumer: consumer.id.clone(),
                    context: consumer.context().cloned(),
                    requested_type: consumer.requested_type_id.clone(),
                    context_to_consumer,
                    boundary: consumer.execution_boundary,
                    boundary_compatibility: boundary_for_id(
                        semantic_types,
                        consumer.id.as_semantic_id(),
                        consumer.execution_boundary,
                        consumer.execution_boundary,
                    ),
                    provenance: consumer.provenance.clone(),
                },
            )
        })
        .collect::<BTreeMap<_, _>>();
    let bindings = consumers
        .values()
        .filter_map(|consumer| {
            let resolution = resolutions.get(&consumer.id)?;
            let consumer_record = consumer_records.get(&consumer.id)?;
            let context_record = consumer
                .context()
                .and_then(|context| context_records.get(context));
            let (provider, source_type, source_to_context, serialization, boundary) =
                match &resolution.result {
                    ContextResolutionResult::Provider { provider, .. } => {
                        let record = provider_records.get(provider)?;
                        (
                            Some(provider.clone()),
                            Some(record.inferred_value_type.clone()),
                            combine_compatibility(
                                record.value_to_declaration,
                                record.declaration_to_context,
                            ),
                            record.serialization,
                            record.boundary_compatibility,
                        )
                    }
                    ContextResolutionResult::ContextDefault {
                        context,
                        expression,
                    } => (
                        None,
                        Some(SemanticTypeId::for_subject(expression)),
                        context_records
                            .get(context)
                            .and_then(|record| record.default_compatibility)
                            .unwrap_or(CompatibilityStatus::Unknown),
                        serialization_for_ids(semantic_types, &[expression]),
                        boundary_for_id(
                            semantic_types,
                            expression,
                            ExecutionBoundary::Client,
                            ExecutionBoundary::Client,
                        ),
                    ),
                    _ => (
                        None,
                        None,
                        CompatibilityStatus::Unknown,
                        ContextSerializationCompatibility::Unknown,
                        CompatibilityStatus::Unknown,
                    ),
                };
            let context_to_consumer = consumer_record.context_to_consumer;
            let overall = binding_overall(
                &resolution.result,
                source_to_context,
                context_to_consumer,
                serialization,
                boundary,
            );
            Some((
                consumer.id.clone(),
                ContextBindingTypeRecord {
                    consumer: consumer.id.clone(),
                    resolution: resolution.result.clone(),
                    provider,
                    context: consumer.context().cloned(),
                    source_type,
                    context_type: context_record.map(|record| record.declared_type.clone()),
                    consumer_type: consumer.requested_type_id.clone(),
                    source_to_context,
                    context_to_consumer,
                    overall,
                    serialization,
                    boundary_compatibility: boundary,
                    provenance: resolution.provenance.clone(),
                },
            ))
        })
        .collect::<BTreeMap<_, _>>();
    ContextTypeProducts {
        contexts: context_records,
        providers: provider_records,
        consumers: consumer_records,
        bindings,
    }
}

fn compatibility_for_ids(
    types: &SemanticTypeModel,
    source: &crate::SemanticId,
    target: &crate::SemanticId,
) -> CompatibilityStatus {
    let Some(source) = types
        .assignments
        .get(source)
        .map(|record| &record.semantic_type)
    else {
        return CompatibilityStatus::Unknown;
    };
    let Some(target) = types
        .assignments
        .get(target)
        .map(|record| &record.semantic_type)
    else {
        return CompatibilityStatus::Unknown;
    };
    if matches!(source, SemanticType::Unknown) || matches!(target, SemanticType::Unknown) {
        CompatibilityStatus::Unknown
    } else if is_assignable(source, target) {
        CompatibilityStatus::Compatible
    } else {
        CompatibilityStatus::Incompatible
    }
}

fn serialization_for_ids(
    types: &SemanticTypeModel,
    ids: &[&crate::SemanticId],
) -> ContextSerializationCompatibility {
    let mut result = ContextSerializationCompatibility::Serializable;
    for id in ids {
        let Some(semantic_type) = types
            .assignments
            .get(*id)
            .map(|record| &record.semantic_type)
        else {
            return ContextSerializationCompatibility::Unknown;
        };
        if matches!(semantic_type, SemanticType::Unknown) {
            return ContextSerializationCompatibility::Unknown;
        }
        if serialization_compatibility(semantic_type) == SerializationCompatibility::NotSerializable
        {
            result = ContextSerializationCompatibility::NonSerializable;
        }
    }
    result
}

fn boundary_for_id(
    types: &SemanticTypeModel,
    id: &crate::SemanticId,
    source: ExecutionBoundary,
    target: ExecutionBoundary,
) -> CompatibilityStatus {
    let Some(semantic_type) = types
        .assignments
        .get(id)
        .map(|record| &record.semantic_type)
    else {
        return CompatibilityStatus::Unknown;
    };
    if matches!(semantic_type, SemanticType::Unknown) {
        return CompatibilityStatus::Unknown;
    }
    match boundary_compatibility(semantic_type, source, target) {
        crate::BoundaryCompatibility::Compatible => CompatibilityStatus::Compatible,
        crate::BoundaryCompatibility::Incompatible => CompatibilityStatus::Incompatible,
    }
}

fn combine_compatibility(
    left: CompatibilityStatus,
    right: CompatibilityStatus,
) -> CompatibilityStatus {
    if left == CompatibilityStatus::Incompatible || right == CompatibilityStatus::Incompatible {
        CompatibilityStatus::Incompatible
    } else if left == CompatibilityStatus::Unknown || right == CompatibilityStatus::Unknown {
        CompatibilityStatus::Unknown
    } else {
        CompatibilityStatus::Compatible
    }
}

fn binding_overall(
    resolution: &ContextResolutionResult,
    source_to_context: CompatibilityStatus,
    context_to_consumer: CompatibilityStatus,
    serialization: ContextSerializationCompatibility,
    boundary: CompatibilityStatus,
) -> ContextBindingCompatibility {
    match resolution {
        ContextResolutionResult::Unresolved => ContextBindingCompatibility::Unresolved,
        ContextResolutionResult::Ambiguous { .. } => ContextBindingCompatibility::Ambiguous,
        ContextResolutionResult::InvalidContextReference => {
            ContextBindingCompatibility::InvalidContextReference
        }
        ContextResolutionResult::Provider { .. }
        | ContextResolutionResult::ContextDefault { .. } => {
            if source_to_context == CompatibilityStatus::Incompatible
                || context_to_consumer == CompatibilityStatus::Incompatible
                || boundary == CompatibilityStatus::Incompatible
                || serialization == ContextSerializationCompatibility::NonSerializable
            {
                ContextBindingCompatibility::Incompatible
            } else if source_to_context == CompatibilityStatus::Unknown
                || context_to_consumer == CompatibilityStatus::Unknown
                || boundary == CompatibilityStatus::Unknown
                || serialization == ContextSerializationCompatibility::Unknown
            {
                ContextBindingCompatibility::Unknown
            } else {
                ContextBindingCompatibility::Compatible
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        build_application_semantic_model, validate_application_semantic_model, CompatibilityStatus,
        ConsumerId, ContextBindingCompatibility, ContextBindingCompatibility::*, ProviderId,
    };

    #[test]
    fn retains_the_directed_compatible_provider_chain() {
        let asm = build_application_semantic_model(&ezc_parser::parse_file(
            "src/components.tsx",
            r#"
@component("x-app")
class App extends Component {
  @context()
  theme!: string;
  @provide(App.theme)
  providedTheme: string = "dark";
  @consume(App.theme)
  toolbarTheme!: string;
  render() { return <main />; }
}
"#,
        ));
        let component = &asm.components[0].id;
        let provider = ProviderId::for_component(component, "providedTheme");
        let consumer = ConsumerId::for_component(component, "toolbarTheme");
        let provider_type = asm.provider_type(&provider).unwrap();
        let binding = asm.context_binding_type(&consumer).unwrap();

        assert_eq!(
            provider_type.value_to_declaration,
            CompatibilityStatus::Compatible
        );
        assert_eq!(
            provider_type.declaration_to_context,
            CompatibilityStatus::Compatible
        );
        assert_eq!(binding.context_to_consumer, CompatibilityStatus::Compatible);
        assert_eq!(binding.overall, Compatible, "{binding:#?}");
        assert!(asm.runtime_eligible_context_binding(&consumer));
        assert!(validate_application_semantic_model(&asm).is_empty());
    }

    #[test]
    fn retains_incompatible_selected_provider_without_reselecting() {
        let asm = build_application_semantic_model(&ezc_parser::parse_file(
            "src/components.tsx",
            r#"
@component("x-app")
class App extends Component {
  @context()
  theme!: number;
  @provide(App.theme)
  providedTheme: string = "dark";
  @consume(App.theme)
  toolbarTheme!: number;
  render() { return <main />; }
}
"#,
        ));
        let component = &asm.components[0].id;
        let provider = ProviderId::for_component(component, "providedTheme");
        let consumer = ConsumerId::for_component(component, "toolbarTheme");
        let binding = asm.context_binding_type(&consumer).unwrap();

        assert_eq!(asm.resolved_provider(&consumer), Some(&provider));
        assert_eq!(
            asm.provider_type(&provider).unwrap().declaration_to_context,
            CompatibilityStatus::Incompatible
        );
        assert_eq!(binding.overall, Incompatible);
        assert!(!asm.runtime_eligible_context_binding(&consumer));
    }

    #[test]
    fn types_context_defaults_as_distinct_fallback_sources() {
        let asm = build_application_semantic_model(&ezc_parser::parse_file(
            "src/components.tsx",
            r#"
@component("x-app")
class App extends Component {
  @context()
  locale: string = "en";
  @consume(App.locale)
  toolbarLocale!: string;
  render() { return <main />; }
}
"#,
        ));
        let consumer = ConsumerId::for_component(&asm.components[0].id, "toolbarLocale");
        let binding = asm.context_binding_type(&consumer).unwrap();

        assert!(binding.provider.is_none());
        assert!(binding.source_type.is_some());
        assert_eq!(binding.overall, Compatible, "{binding:#?}");
    }

    #[test]
    fn preserves_unresolved_and_invalid_binding_states() {
        let unresolved = build_application_semantic_model(&ezc_parser::parse_file(
            "src/unresolved.tsx",
            r#"
@component("x-app")
class App extends Component {
  @context()
  locale!: string;
  @consume(App.locale)
  toolbarLocale!: string;
  render() { return <main />; }
}
"#,
        ));
        let unresolved_consumer =
            ConsumerId::for_component(&unresolved.components[0].id, "toolbarLocale");
        assert_eq!(
            unresolved
                .context_binding_type(&unresolved_consumer)
                .unwrap()
                .overall,
            Unresolved
        );

        let invalid = build_application_semantic_model(&ezc_parser::parse_file(
            "src/invalid.tsx",
            r#"
@component("x-app")
class App extends Component {
  @consume(Missing.locale)
  toolbarLocale!: string;
  render() { return <main />; }
}
"#,
        ));
        let invalid_consumer =
            ConsumerId::for_component(&invalid.components[0].id, "toolbarLocale");
        assert_eq!(
            invalid
                .context_binding_type(&invalid_consumer)
                .unwrap()
                .overall,
            ContextBindingCompatibility::InvalidContextReference
        );
    }
}
