use std::collections::BTreeSet;

use serde::Serialize;

use crate::{IrComputedEvaluationPlan, RuntimeComputedRegistry, SerializationCompatibility};

pub const RUNTIME_COMPUTED_ARTIFACT_SCHEMA_VERSION: u32 = 1;

/// Versioned runtime metadata emitted from canonical computed compiler products.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct RuntimeComputedArtifact {
    pub schema_version: u32,
    pub evaluations: Vec<RuntimeComputedArtifactEvaluation>,
    pub evaluation_order: Vec<String>,
    pub update_batches: Vec<Vec<String>>,
}

/// Runtime metadata for one compiler-lowered computed evaluation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct RuntimeComputedArtifactEvaluation {
    pub computed: String,
    pub cache_slot: String,
    pub dirty_flag: RuntimeComputedArtifactDirtyFlag,
    pub dependencies: Vec<String>,
    pub evaluation_function: String,
    pub serialization: RuntimeComputedArtifactSerialization,
}

/// Compiler-provided dirty state for one emitted computed evaluation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct RuntimeComputedArtifactDirtyFlag {
    pub id: String,
    pub initial_value: bool,
}

/// Runtime-facing spelling for the compiler's serialization contract.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum RuntimeComputedArtifactSerialization {
    Serializable,
    NotSerializable,
}

/// Emit deterministic runtime metadata from the E12 registry and E9 plan.
///
/// The artifact does not discover dependencies, compute values, or mutate
/// caches. It serializes only records that already have compiler-lowered
/// evaluation functions and filters the existing plan to those records.
#[must_use]
pub fn build_runtime_computed_artifact(
    registry: &RuntimeComputedRegistry,
    plan: &IrComputedEvaluationPlan,
) -> RuntimeComputedArtifact {
    let available = registry
        .records
        .keys()
        .map(|computed| computed.as_str().to_string())
        .collect::<BTreeSet<_>>();
    let evaluation_order = plan
        .evaluation_order
        .iter()
        .filter(|computed| available.contains(computed.as_str()))
        .cloned()
        .collect();
    let update_batches = plan
        .update_batches
        .iter()
        .map(|batch| {
            batch
                .iter()
                .filter(|computed| available.contains(computed.as_str()))
                .cloned()
                .collect::<Vec<_>>()
        })
        .filter(|batch| !batch.is_empty())
        .collect();
    let evaluations = registry
        .records
        .values()
        .map(|record| RuntimeComputedArtifactEvaluation {
            computed: record.computed.as_str().to_string(),
            cache_slot: record.cache_slot.as_str().to_string(),
            dirty_flag: RuntimeComputedArtifactDirtyFlag {
                id: record.dirty_flag.id.clone(),
                initial_value: record.dirty_flag.initial_value,
            },
            dependencies: record
                .dependencies
                .iter()
                .map(|dependency| dependency.as_str().to_string())
                .collect(),
            evaluation_function: record.evaluation_function.as_str().to_string(),
            serialization: serialization(record.serialization),
        })
        .collect();

    RuntimeComputedArtifact {
        schema_version: RUNTIME_COMPUTED_ARTIFACT_SCHEMA_VERSION,
        evaluations,
        evaluation_order,
        update_batches,
    }
}

/// Serialize emitted computed runtime metadata as deterministic, pretty JSON.
///
/// # Panics
///
/// Panics when the compiler-owned runtime metadata cannot serialize.
#[must_use]
pub fn runtime_computed_artifact_json(artifact: &RuntimeComputedArtifact) -> String {
    serde_json::to_string_pretty(artifact).expect("computed runtime artifact should serialize")
        + "\n"
}

const fn serialization(
    compatibility: SerializationCompatibility,
) -> RuntimeComputedArtifactSerialization {
    match compatibility {
        SerializationCompatibility::Serializable => {
            RuntimeComputedArtifactSerialization::Serializable
        }
        SerializationCompatibility::NotSerializable => {
            RuntimeComputedArtifactSerialization::NotSerializable
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        build_application_semantic_model, build_runtime_computed_artifact,
        build_runtime_computed_registry, lower_components_to_ir, runtime_computed_artifact_json,
        RuntimeComputedArtifactSerialization, RUNTIME_COMPUTED_ARTIFACT_SCHEMA_VERSION,
    };

    #[test]
    fn emits_deterministic_runtime_metadata_from_registry_and_plan() {
        let parsed = ezc_parser::parse_file(
            "src/RuntimeComputedArtifact.tsx",
            r#"
@component("x-runtime-computed-artifact")
class RuntimeComputedArtifact extends Component {
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
        let artifact = build_runtime_computed_artifact(&registry, &model.computed_evaluation_plan);

        assert_eq!(
            artifact.schema_version,
            RUNTIME_COMPUTED_ARTIFACT_SCHEMA_VERSION
        );
        assert_eq!(artifact.evaluations.len(), 2);
        assert_eq!(
            artifact.evaluation_order,
            vec![doubled.as_str().to_string(), label.as_str().to_string()]
        );
        assert_eq!(
            artifact.update_batches,
            vec![
                vec![doubled.as_str().to_string()],
                vec![label.as_str().to_string()]
            ]
        );
        assert_eq!(artifact.evaluations[0].computed, doubled.as_str());
        assert_eq!(
            artifact.evaluations[0].dependencies,
            vec![count.to_string()]
        );
        assert_eq!(artifact.evaluations[1].computed, label.as_str());
        assert_eq!(
            artifact.evaluations[1].dependencies,
            vec![doubled.to_string()]
        );
        assert!(artifact
            .evaluations
            .iter()
            .all(|evaluation| evaluation.dirty_flag.initial_value));
        assert!(artifact.evaluations.iter().all(|evaluation| {
            evaluation.evaluation_function == evaluation.computed
                && evaluation.serialization == RuntimeComputedArtifactSerialization::Serializable
        }));

        let first = runtime_computed_artifact_json(&artifact);
        let second = runtime_computed_artifact_json(&build_runtime_computed_artifact(
            &registry,
            &model.computed_evaluation_plan,
        ));
        assert_eq!(first, second);
        let json: serde_json::Value = serde_json::from_str(&first).expect("artifact JSON");
        assert_eq!(
            json["evaluations"][0]["cache_slot"],
            format!("{doubled}/runtime:cache")
        );
        assert_eq!(json["evaluations"][1]["dirty_flag"]["initial_value"], true);
        assert_eq!(json["evaluations"][1]["serialization"], "serializable");
    }
}
