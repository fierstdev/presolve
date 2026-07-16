use std::collections::{BTreeMap, BTreeSet};

use crate::{
    ApplicationSemanticModel, IntermediateRepresentation, SemanticId, SerializationCompatibility,
    SourceProvenance,
};

/// Compiler-owned runtime registry for canonical computed values.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct RuntimeComputedRegistry {
    pub records: BTreeMap<SemanticId, RuntimeComputedRecord>,
}

/// Runtime metadata for one compiler-lowered computed evaluation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeComputedRecord {
    pub computed: SemanticId,
    pub cache_slot: ComputedCacheSlotId,
    pub dirty_flag: RuntimeComputedDirtyFlag,
    pub dependencies: Vec<SemanticId>,
    pub evaluation_function: SemanticId,
    pub serialization: SerializationCompatibility,
    pub provenance: SourceProvenance,
}

/// Stable runtime cache-slot identity for one computed value.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ComputedCacheSlotId(String);

impl ComputedCacheSlotId {
    #[must_use]
    pub fn for_computed(computed: &SemanticId) -> Self {
        Self(format!("{computed}/runtime:cache"))
    }

    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// Compatibility spelling for the declaration-level E12 cache identity.
pub type RuntimeComputedCacheSlot = ComputedCacheSlotId;

/// Stable declaration-level dirty-flag identity for one computed value.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ComputedDirtyFlagId(String);

impl ComputedDirtyFlagId {
    #[must_use]
    pub fn for_computed(computed: &SemanticId) -> Self {
        Self(format!("{computed}/runtime:dirty"))
    }

    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// Stable runtime dirty-flag identity and compiler-provided initial state.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeComputedDirtyFlag {
    pub id: ComputedDirtyFlagId,
    pub initial_value: bool,
}

impl RuntimeComputedDirtyFlag {
    #[must_use]
    pub fn for_computed(computed: &SemanticId) -> Self {
        Self {
            id: ComputedDirtyFlagId::for_computed(computed),
            initial_value: true,
        }
    }
}

impl RuntimeComputedRegistry {
    #[must_use]
    pub fn record(&self, computed: &SemanticId) -> Option<&RuntimeComputedRecord> {
        self.records.get(computed)
    }
}

/// Build deterministic runtime computed records from canonical ASM and IR
/// products.
#[must_use]
pub fn build_runtime_computed_registry(
    model: &ApplicationSemanticModel,
    ir: &IntermediateRepresentation,
) -> RuntimeComputedRegistry {
    let evaluations = ir
        .modules
        .iter()
        .flat_map(|module| module.computed_evaluations.iter())
        .map(|evaluation| (evaluation.computed.clone(), evaluation.function.clone()))
        .collect::<BTreeMap<_, _>>();
    let records = evaluations
        .into_iter()
        .filter_map(|(computed, evaluation_function)| {
            let value = model.computed_values.get(&computed)?;
            let computed_type = model.semantic_types.computed_values.get(&computed)?;
            Some((
                computed.clone(),
                RuntimeComputedRecord {
                    computed: computed.clone(),
                    cache_slot: ComputedCacheSlotId::for_computed(&computed),
                    dirty_flag: RuntimeComputedDirtyFlag::for_computed(&computed),
                    dependencies: computed_dependencies(model, &computed),
                    evaluation_function,
                    serialization: computed_type.serialization,
                    provenance: value.provenance.clone(),
                },
            ))
        })
        .collect();

    RuntimeComputedRegistry { records }
}

fn computed_dependencies(
    model: &ApplicationSemanticModel,
    computed: &SemanticId,
) -> Vec<SemanticId> {
    model
        .references_from(computed)
        .into_iter()
        .filter(|reference| {
            matches!(
                reference.kind,
                crate::SemanticReferenceKind::ComputedState
                    | crate::SemanticReferenceKind::ComputedComputed
            )
        })
        .map(|reference| reference.target.clone())
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect()
}

#[cfg(test)]
mod tests {
    use crate::{
        build_application_semantic_model, build_runtime_computed_registry, lower_components_to_ir,
        SerializationCompatibility,
    };

    #[test]
    fn builds_deterministic_runtime_records_from_computed_ir() {
        let parsed = ezc_parser::parse_file(
            "src/RuntimeComputed.tsx",
            r#"
@component("x-runtime-computed")
class RuntimeComputed extends Component {
  count = state(1);

  @computed()
  get doubled() { return this.count * 2; }

  @computed()
  get label() { return this.doubled + 1; }
}
"#,
        );
        let model = build_application_semantic_model(&parsed);
        let component = &model.components[0];
        let count = component.id.state_field("count");
        let doubled = component.id.computed("doubled");
        let label = component.id.computed("label");
        let registry = build_runtime_computed_registry(&model, &lower_components_to_ir(&model));
        let doubled_record = registry.record(&doubled).expect("doubled record");
        let label_record = registry.record(&label).expect("label record");

        assert_eq!(registry.records.len(), 2);
        assert_eq!(doubled_record.dependencies, vec![count]);
        assert_eq!(label_record.dependencies, vec![doubled.clone()]);
        assert_eq!(doubled_record.evaluation_function, doubled);
        assert_eq!(label_record.evaluation_function, label);
        assert_eq!(
            doubled_record.cache_slot.as_str(),
            format!("{}/runtime:cache", doubled_record.computed)
        );
        assert!(doubled_record.dirty_flag.initial_value);
        assert_eq!(
            doubled_record.dirty_flag.id.as_str(),
            format!("{}/runtime:dirty", doubled_record.computed)
        );
        assert_eq!(
            label_record.serialization,
            SerializationCompatibility::Serializable
        );
        assert_eq!(
            doubled_record.provenance,
            model
                .computed_value(&doubled)
                .expect("doubled value")
                .provenance
        );
    }
}
