use std::collections::BTreeMap;

use serde::Serialize;

use crate::runtime_computed_artifact::{
    runtime_instruction, RuntimeComputedArtifactInstruction, RuntimeComputedArtifactOperand,
};
use crate::{
    build_runtime_context_registry, ContextEvaluationBatchId, ExecutionBoundary,
    OptimizedContextIrReport, RuntimeContextConsumerRecord, RuntimeContextEvaluationBatch,
    RuntimeContextSourceKind, RuntimeContextSourceRecord,
};

pub const RUNTIME_CONTEXT_ARTIFACT_SCHEMA_VERSION: u32 = 1;

/// Separate schema-v1 artifact for compiler-owned Context runtime metadata.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct RuntimeContextArtifact {
    pub schema_version: u32,
    pub sources: Vec<SerializedContextSource>,
    pub consumers: Vec<SerializedContextConsumerBinding>,
    pub initial_batches: Vec<SerializedContextEvaluationBatch>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct SerializedContextSource {
    pub source: String,
    pub context: String,
    pub slot: String,
    pub source_function: String,
    pub source_kind: SerializedContextSourceKind,
    pub program: SerializedContextProgram,
    pub required_state: Vec<String>,
    pub required_computed: Vec<String>,
    pub prerequisite_computed_batches: Vec<u32>,
    pub evaluation_batch: SerializedContextBatchId,
    pub semantic_type: String,
    pub execution_boundary: SerializedContextExecutionBoundary,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct SerializedContextProgram {
    pub result: String,
    pub instructions: Vec<SerializedContextInstruction>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(untagged)]
pub enum SerializedContextInstruction {
    Evaluate(RuntimeComputedArtifactInstruction),
    InitializeContextSlot {
        #[serde(rename = "kind")]
        kind: SerializedContextInstructionKind,
        slot: String,
        value: RuntimeComputedArtifactOperand,
    },
    LoadContextSlot {
        #[serde(rename = "kind")]
        kind: SerializedContextInstructionKind,
        result: String,
        slot: String,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum SerializedContextInstructionKind {
    InitializeContextSlot,
    LoadContextSlot,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum SerializedContextSourceKind {
    Provider,
    ContextDefault,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct SerializedContextConsumerBinding {
    pub consumer: String,
    pub context: String,
    pub selected_source: String,
    pub slot: String,
    pub load_identity: String,
    pub semantic_type: String,
    pub source_batch: SerializedContextBatchId,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct SerializedContextEvaluationBatch {
    pub id: SerializedContextBatchId,
    pub sources: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct SerializedContextBatchId {
    pub plan: &'static str,
    pub index: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum SerializedContextExecutionBoundary {
    Client,
    Server,
}

/// Emit a deterministic Context artifact from G12 registry and optimized G11
/// programs. It serializes instructions only; it performs no Context execution.
#[must_use]
pub fn build_runtime_context_artifact(
    model: &crate::ApplicationSemanticModel,
    optimized: &OptimizedContextIrReport,
) -> RuntimeContextArtifact {
    let registry = build_runtime_context_registry(model, optimized);
    let programs = context_programs(optimized);
    RuntimeContextArtifact {
        schema_version: RUNTIME_CONTEXT_ARTIFACT_SCHEMA_VERSION,
        sources: registry
            .sources
            .iter()
            .filter_map(|record| {
                programs
                    .get(&record.source)
                    .cloned()
                    .map(|program| serialized_source(record, program))
            })
            .collect(),
        consumers: registry.consumers.iter().map(serialized_consumer).collect(),
        initial_batches: registry
            .initial_batches
            .iter()
            .map(serialized_batch)
            .collect(),
    }
}

/// Serialize the Context artifact as deterministic, pretty JSON.
///
/// # Panics
///
/// Panics when compiler-owned Context runtime metadata cannot serialize.
#[must_use]
pub fn runtime_context_artifact_json(artifact: &RuntimeContextArtifact) -> String {
    serde_json::to_string_pretty(artifact).expect("Context runtime artifact should serialize")
        + "\n"
}

fn context_programs(
    optimized: &OptimizedContextIrReport,
) -> BTreeMap<crate::ContextValueSourceId, SerializedContextProgram> {
    optimized
        .source_evaluations
        .iter()
        .filter_map(|evaluation| {
            let function = optimized
                .optimized_module
                .modules
                .iter()
                .flat_map(|module| &module.functions)
                .find(|function| function.id == *evaluation.function.as_semantic_id())?;
            let instructions = function
                .blocks
                .iter()
                .flat_map(|block| &block.instructions)
                .map(serialized_instruction)
                .collect::<Option<Vec<_>>>()?;
            Some((
                evaluation.source.clone(),
                SerializedContextProgram {
                    result: evaluation.result.as_str().to_string(),
                    instructions,
                },
            ))
        })
        .collect()
}

fn serialized_instruction(
    instruction: &crate::IrInstruction,
) -> Option<SerializedContextInstruction> {
    match &instruction.kind {
        crate::IrInstructionKind::InitializeContextSlot { slot, value } => {
            Some(SerializedContextInstruction::InitializeContextSlot {
                kind: SerializedContextInstructionKind::InitializeContextSlot,
                slot: slot.as_str().to_string(),
                value: RuntimeComputedArtifactOperand::Value {
                    value: value.as_str().to_string(),
                },
            })
        }
        crate::IrInstructionKind::LoadContextSlot { slot } => {
            Some(SerializedContextInstruction::LoadContextSlot {
                kind: SerializedContextInstructionKind::LoadContextSlot,
                result: instruction.result.as_ref()?.as_str().to_string(),
                slot: slot.as_str().to_string(),
            })
        }
        _ => runtime_instruction(instruction).map(SerializedContextInstruction::Evaluate),
    }
}

fn serialized_source(
    record: &RuntimeContextSourceRecord,
    program: SerializedContextProgram,
) -> SerializedContextSource {
    SerializedContextSource {
        source: source_id(&record.source),
        context: record.context.as_str().to_string(),
        slot: record.slot.as_str().to_string(),
        source_function: record.function.as_semantic_id().as_str().to_string(),
        source_kind: source_kind(record.source_kind),
        program,
        required_state: semantic_ids(&record.required_state),
        required_computed: semantic_ids(&record.required_computed),
        prerequisite_computed_batches: record.prerequisite_computed_batches.clone(),
        evaluation_batch: batch_id(&record.evaluation_batch),
        semantic_type: record.semantic_type.to_string(),
        execution_boundary: boundary(record.boundary),
    }
}

fn serialized_consumer(record: &RuntimeContextConsumerRecord) -> SerializedContextConsumerBinding {
    SerializedContextConsumerBinding {
        consumer: record.consumer.as_str().to_string(),
        context: record.context.as_str().to_string(),
        selected_source: source_id(&record.selected_source),
        slot: record.slot.as_str().to_string(),
        load_identity: record.load_identity.as_semantic_id().as_str().to_string(),
        semantic_type: record.semantic_type.to_string(),
        source_batch: batch_id(&record.source_batch),
    }
}

fn serialized_batch(batch: &RuntimeContextEvaluationBatch) -> SerializedContextEvaluationBatch {
    SerializedContextEvaluationBatch {
        id: batch_id(&batch.id),
        sources: batch.sources.iter().map(source_id).collect(),
    }
}

fn batch_id(batch: &ContextEvaluationBatchId) -> SerializedContextBatchId {
    SerializedContextBatchId {
        plan: "initial",
        index: batch.index,
    }
}

fn semantic_ids(values: &[crate::SemanticId]) -> Vec<String> {
    values
        .iter()
        .map(|value| value.as_str().to_string())
        .collect()
}

fn source_id(source: &crate::ContextValueSourceId) -> String {
    match source {
        crate::ContextValueSourceId::Provider(provider) => provider.as_str().to_string(),
        crate::ContextValueSourceId::ContextDefault(context) => {
            format!("{}/default", context.as_str())
        }
    }
}

const fn source_kind(kind: RuntimeContextSourceKind) -> SerializedContextSourceKind {
    match kind {
        RuntimeContextSourceKind::Provider => SerializedContextSourceKind::Provider,
        RuntimeContextSourceKind::ContextDefault => SerializedContextSourceKind::ContextDefault,
    }
}

const fn boundary(boundary: ExecutionBoundary) -> SerializedContextExecutionBoundary {
    match boundary {
        ExecutionBoundary::Client => SerializedContextExecutionBoundary::Client,
        ExecutionBoundary::Server => SerializedContextExecutionBoundary::Server,
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        build_application_semantic_model, build_runtime_context_artifact, lower_components_to_ir,
        optimize_context_ir, runtime_context_artifact_json,
        RUNTIME_CONTEXT_ARTIFACT_SCHEMA_VERSION,
    };

    #[test]
    fn emits_deterministic_context_slot_programs_without_runtime_lookup_operations() {
        let model = build_application_semantic_model(&ezc_parser::parse_file(
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
  render() { return <main />; }
}
"#,
        ));
        let optimized = optimize_context_ir(&lower_components_to_ir(&model));
        let artifact = build_runtime_context_artifact(&model, &optimized);
        let json = runtime_context_artifact_json(&artifact);

        assert_eq!(
            artifact.schema_version,
            RUNTIME_CONTEXT_ARTIFACT_SCHEMA_VERSION
        );
        assert_eq!(artifact.sources.len(), 1);
        assert_eq!(artifact.consumers.len(), 1);
        assert_eq!(artifact.sources[0].program.instructions.len(), 4);
        assert!(json.contains("initialize_context_slot"));
        assert!(json.contains("load-state"));
        assert!(!json.contains("find_provider"));
        assert!(!json.contains("resolve_context"));
        assert!(!json.contains("walk_parent"));
        assert!(!json.contains("lookup_by_name"));
        assert_eq!(
            json,
            runtime_context_artifact_json(&build_runtime_context_artifact(&model, &optimized))
        );
    }
}
