use std::collections::BTreeSet;

use crate::{
    ApplicationSemanticModel, ContextConsumerAvailabilityStatus, ContextConsumerLoadId,
    ContextEvaluationBatchId, ContextSerializationCompatibility, ContextSourceFunctionId,
    ContextSourcePlanStatus, ContextValueSlotId, ContextValueSourceId, ExecutionBoundary,
    OptimizedContextIrReport, SemanticId, SemanticTypeId, SourceProvenance,
};

/// Frozen G12 schema contract for compiler-owned Context runtime metadata.
pub const RUNTIME_CONTEXT_REGISTRY_SCHEMA_CONTRACT_VERSION: u32 = 1;

/// Immutable compiler-owned Context metadata. It is a projection of G9-G11
/// products and is not a runtime Provider, Context, or dependency registry.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeContextRegistry {
    pub schema_contract_version: u32,
    pub sources: Vec<RuntimeContextSourceRecord>,
    pub consumers: Vec<RuntimeContextConsumerRecord>,
    pub initial_batches: Vec<RuntimeContextEvaluationBatch>,
}

/// One executable Context source, keyed by its compiler-generated slot.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeContextSourceRecord {
    pub source: ContextValueSourceId,
    pub context: crate::ContextId,
    pub owner_component: SemanticId,
    pub function: ContextSourceFunctionId,
    pub slot: ContextValueSlotId,
    pub semantic_type: SemanticTypeId,
    pub source_kind: RuntimeContextSourceKind,
    pub required_state: Vec<SemanticId>,
    pub required_computed: Vec<SemanticId>,
    pub prerequisite_computed_batches: Vec<u32>,
    pub evaluation_batch: ContextEvaluationBatchId,
    pub boundary: ExecutionBoundary,
    pub serialization: ContextSerializationCompatibility,
    pub provenance: SourceProvenance,
}

/// The two authored Context source kinds remain distinct at runtime.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RuntimeContextSourceKind {
    Provider,
    ContextDefault,
}

/// One available Consumer's immutable, exact Context-slot binding.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeContextConsumerRecord {
    pub consumer: crate::ConsumerId,
    pub context: crate::ContextId,
    pub owner_component: SemanticId,
    pub selected_source: ContextValueSourceId,
    pub slot: ContextValueSlotId,
    pub load_identity: ContextConsumerLoadId,
    pub semantic_type: SemanticTypeId,
    pub source_batch: ContextEvaluationBatchId,
    pub provenance: SourceProvenance,
}

/// One G9-scheduled initial Context evaluation batch.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeContextEvaluationBatch {
    pub id: ContextEvaluationBatchId,
    pub sources: Vec<ContextValueSourceId>,
}

/// One compiler-side G12 registry validation failure.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeContextRegistryValidationDiagnostic {
    pub code: &'static str,
    pub message: String,
}

impl RuntimeContextRegistry {
    #[must_use]
    pub fn source(&self, source: &ContextValueSourceId) -> Option<&RuntimeContextSourceRecord> {
        self.sources.iter().find(|record| record.source == *source)
    }

    #[must_use]
    pub fn consumer(&self, consumer: &crate::ConsumerId) -> Option<&RuntimeContextConsumerRecord> {
        self.consumers
            .iter()
            .find(|record| record.consumer == *consumer)
    }
}

/// Build immutable G12 runtime Context metadata from existing G9-G11 products.
/// This never resolves a Provider, reads a Context name, or reconstructs a
/// dependency/ownership graph.
#[must_use]
pub fn build_runtime_context_registry(
    model: &ApplicationSemanticModel,
    optimized: &OptimizedContextIrReport,
) -> RuntimeContextRegistry {
    let mut sources = optimized
        .source_evaluations
        .iter()
        .filter_map(|evaluation| source_record(model, optimized, evaluation))
        .collect::<Vec<_>>();
    sources.sort_by(|left, right| left.source.cmp(&right.source));
    let source_ids = sources
        .iter()
        .map(|record| record.source.clone())
        .collect::<BTreeSet<_>>();

    let mut consumers = optimized
        .optimized_module
        .context_ir
        .consumer_bindings
        .iter()
        .filter_map(|binding| consumer_record(model, binding, &sources))
        .collect::<Vec<_>>();
    consumers.sort_by(|left, right| left.consumer.cmp(&right.consumer));

    let initial_batches = model
        .context_evaluation
        .evaluation_batches
        .iter()
        .filter(|batch| {
            batch
                .sources
                .iter()
                .all(|source| source_ids.contains(source))
        })
        .map(|batch| RuntimeContextEvaluationBatch {
            id: batch.id.clone(),
            sources: batch.sources.clone(),
        })
        .collect();

    RuntimeContextRegistry {
        schema_contract_version: RUNTIME_CONTEXT_REGISTRY_SCHEMA_CONTRACT_VERSION,
        sources,
        consumers,
        initial_batches,
    }
}

fn source_record(
    model: &ApplicationSemanticModel,
    optimized: &OptimizedContextIrReport,
    evaluation: &crate::OptimizedIrContextSourceEvaluation,
) -> Option<RuntimeContextSourceRecord> {
    let plan = model
        .context_evaluation
        .context_source_plan(&evaluation.source)?;
    if plan.status != ContextSourcePlanStatus::Planned {
        return None;
    }
    let function_count = optimized
        .optimized_module
        .modules
        .iter()
        .flat_map(|module| &module.functions)
        .filter(|function| function.id == *evaluation.function.as_semantic_id())
        .count();
    if function_count != 1 {
        return None;
    }
    let (semantic_type, source_kind, boundary, serialization) = match &evaluation.source {
        ContextValueSourceId::Provider(provider) => {
            let types = model.provider_types.get(provider)?;
            (
                types.inferred_value_type.clone(),
                RuntimeContextSourceKind::Provider,
                types.boundary,
                types.serialization,
            )
        }
        ContextValueSourceId::ContextDefault(context) => {
            let types = model.context_types.get(context)?;
            (
                types.default_type.clone()?,
                RuntimeContextSourceKind::ContextDefault,
                types.boundary,
                types.serialization,
            )
        }
    };
    (serialization == ContextSerializationCompatibility::Serializable).then_some(
        RuntimeContextSourceRecord {
            source: evaluation.source.clone(),
            context: evaluation.context.clone(),
            owner_component: plan.owner_component.clone(),
            function: evaluation.function.clone(),
            slot: evaluation.slot.clone(),
            semantic_type,
            source_kind,
            required_state: plan.required_state.clone(),
            required_computed: plan.required_computed.clone(),
            prerequisite_computed_batches: evaluation.prerequisite_computed_batches.clone(),
            evaluation_batch: evaluation.evaluation_batch.clone(),
            boundary,
            serialization,
            provenance: evaluation.provenance.clone(),
        },
    )
}

fn consumer_record(
    model: &ApplicationSemanticModel,
    binding: &crate::IrContextConsumerBinding,
    sources: &[RuntimeContextSourceRecord],
) -> Option<RuntimeContextConsumerRecord> {
    let availability = model
        .context_evaluation
        .context_consumer_availability(&binding.consumer)?;
    if availability.status != ContextConsumerAvailabilityStatus::Available
        || availability.selected_source.as_ref() != Some(&binding.source)
    {
        return None;
    }
    let consumer = model.consumers.get(&binding.consumer)?;
    let source = sources
        .iter()
        .find(|source| source.source == binding.source)?;
    Some(RuntimeContextConsumerRecord {
        consumer: binding.consumer.clone(),
        context: binding.context.clone(),
        owner_component: consumer.owner.entity_id()?.clone(),
        selected_source: binding.source.clone(),
        slot: binding.slot.clone(),
        load_identity: binding.load.id.clone(),
        semantic_type: binding.semantic_type.clone(),
        source_batch: source.evaluation_batch.clone(),
        provenance: binding.provenance.clone(),
    })
}

/// Validate that the G12 registry is one exact, deterministic projection of
/// G9-G11 facts. Validation does not recover or synthesize any missing record.
#[must_use]
pub fn validate_runtime_context_registry(
    model: &ApplicationSemanticModel,
    optimized: &OptimizedContextIrReport,
    registry: &RuntimeContextRegistry,
) -> Vec<RuntimeContextRegistryValidationDiagnostic> {
    let expected = build_runtime_context_registry(model, optimized);
    let mut diagnostics = Vec::new();
    if registry.schema_contract_version != RUNTIME_CONTEXT_REGISTRY_SCHEMA_CONTRACT_VERSION {
        diagnostics.push(RuntimeContextRegistryValidationDiagnostic {
            code: "PSCTX1200",
            message: "Context runtime registry has an unsupported schema contract version"
                .to_string(),
        });
    }
    if registry.sources != expected.sources {
        diagnostics.push(RuntimeContextRegistryValidationDiagnostic {
            code: "PSCTX1201",
            message: "Context runtime registry sources do not exactly join planned G9 and optimized G11 identities".to_string(),
        });
    }
    if registry.consumers != expected.consumers {
        diagnostics.push(RuntimeContextRegistryValidationDiagnostic {
            code: "PSCTX1202",
            message:
                "Context runtime registry Consumers do not retain exact available G10 bindings"
                    .to_string(),
        });
    }
    if registry.initial_batches != expected.initial_batches {
        diagnostics.push(RuntimeContextRegistryValidationDiagnostic {
            code: "PSCTX1203",
            message: "Context runtime registry batches do not retain G9 scheduler order"
                .to_string(),
        });
    }
    diagnostics
}

#[cfg(test)]
mod tests {
    use crate::{
        build_application_semantic_model, build_runtime_context_registry, lower_components_to_ir,
        optimize_context_ir, validate_runtime_context_registry, ContextValueSourceId, ProviderId,
        RuntimeContextSourceKind, RUNTIME_CONTEXT_REGISTRY_SCHEMA_CONTRACT_VERSION,
    };

    #[test]
    fn projects_only_planned_sources_and_available_exact_slot_bindings() {
        let model = build_application_semantic_model(&presolve_parser::parse_file(
            "src/App.tsx",
            r#"
@component("x-app")
class App extends Component {
  count = state(1);
  @context()
  total!: number;
  @provide(App.total)
  providedTotal: number = this.count + 2;
  @consume(App.total)
  total!: number;
  @context()
  locale: string = "en";
  @consume(App.locale)
  locale!: string;
  @context()
  unused!: string;
  @provide(App.unused)
  unusedProvider: string = "unused";
  @consume(App.missing)
  missing!: string;
  render() { return <main />; }
}
"#,
        ));
        let component = &model.components[0].id;
        let total =
            ContextValueSourceId::Provider(ProviderId::for_component(component, "providedTotal"));
        let optimized = optimize_context_ir(&lower_components_to_ir(&model));
        let registry = build_runtime_context_registry(&model, &optimized);

        assert_eq!(
            registry.schema_contract_version,
            RUNTIME_CONTEXT_REGISTRY_SCHEMA_CONTRACT_VERSION
        );
        assert_eq!(registry.sources.len(), 2);
        assert_eq!(registry.consumers.len(), 2);
        assert!(registry.source(&total).is_some_and(|record| {
            record.source_kind == RuntimeContextSourceKind::Provider
                && record.required_state == vec![component.state_field("count")]
        }));
        assert!(registry.sources.iter().all(|record| record.source
            != ContextValueSourceId::Provider(ProviderId::for_component(
                component,
                "unusedProvider"
            ))));
        assert!(registry.consumers.iter().all(|record| {
            registry
                .source(&record.selected_source)
                .is_some_and(|source| {
                    source.slot == record.slot && source.evaluation_batch == record.source_batch
                })
        }));
        assert!(validate_runtime_context_registry(&model, &optimized, &registry).is_empty());
    }
}
