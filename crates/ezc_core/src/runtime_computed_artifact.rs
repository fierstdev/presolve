use std::collections::{BTreeMap, BTreeSet};

use serde::Serialize;

use crate::{
    build_runtime_computed_registry, ApplicationSemanticModel, IntermediateRepresentation,
    IrBinaryOperation, IrConstant, IrInstructionKind, IrOperand, IrStorageId, IrUnaryOperation,
    SemanticId, SerializableValue, SerializationCompatibility,
};

pub const RUNTIME_COMPUTED_ARTIFACT_SCHEMA_VERSION: u32 = 3;

/// Versioned runtime metadata and executable programs emitted from canonical IR.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct RuntimeComputedArtifact {
    pub schema_version: u32,
    pub state: Vec<RuntimeComputedArtifactState>,
    pub invalidations: Vec<RuntimeComputedArtifactInvalidation>,
    pub evaluations: Vec<RuntimeComputedArtifactEvaluation>,
    pub evaluation_order: Vec<String>,
    pub update_batches: Vec<Vec<String>>,
}

/// Compiler-lowered storage initialization available to computed programs.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct RuntimeComputedArtifactState {
    pub component: String,
    pub field: String,
    pub storage: String,
    pub initial_value: Option<SerializableValue>,
}

/// Compiler-generated transitive computed invalidations for one state storage.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct RuntimeComputedArtifactInvalidation {
    pub storage: String,
    pub dependents: Vec<String>,
}

/// Runtime metadata and canonical instruction program for one computed value.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct RuntimeComputedArtifactEvaluation {
    pub computed: String,
    pub component: String,
    pub cache_slot: String,
    pub dirty_flag: RuntimeComputedArtifactDirtyFlag,
    pub dependencies: Vec<String>,
    pub evaluation_function: String,
    pub serialization: RuntimeComputedArtifactSerialization,
    pub program: RuntimeComputedArtifactProgram,
}

/// One compiler-lowered evaluation result and its authored-order instructions.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct RuntimeComputedArtifactProgram {
    pub result: String,
    pub instructions: Vec<RuntimeComputedArtifactInstruction>,
}

/// A runtime operand still tied to a canonical IR value or constant.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(tag = "kind", rename_all = "kebab-case")]
pub enum RuntimeComputedArtifactOperand {
    Value { value: String },
    Constant { value: SerializableValue },
    Storage { storage: String },
}

/// One supported compiler-lowered computed instruction.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(tag = "kind", rename_all = "kebab-case")]
pub enum RuntimeComputedArtifactInstruction {
    Constant {
        result: String,
        value: SerializableValue,
    },
    LoadState {
        result: String,
        storage: String,
    },
    LoadComputed {
        result: String,
        computed: String,
    },
    GetMember {
        result: String,
        object: RuntimeComputedArtifactOperand,
        property: String,
    },
    Binary {
        result: String,
        operation: RuntimeComputedArtifactBinaryOperation,
        left: RuntimeComputedArtifactOperand,
        right: RuntimeComputedArtifactOperand,
    },
    Unary {
        result: String,
        operation: RuntimeComputedArtifactUnaryOperation,
        operand: RuntimeComputedArtifactOperand,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum RuntimeComputedArtifactBinaryOperation {
    Add,
    Subtract,
    Multiply,
    Divide,
    Remainder,
    Equal,
    NotEqual,
    LessThan,
    LessThanOrEqual,
    GreaterThan,
    GreaterThanOrEqual,
    And,
    Or,
    NullishCoalesce,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum RuntimeComputedArtifactUnaryOperation {
    Not,
    Identity,
    Negate,
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

/// Emit deterministic runtime metadata and E10 programs from canonical ASM and IR.
///
/// The artifact contains no source expressions and requires no runtime dependency
/// discovery. E15 executes its explicit E9 order once, populating compiler-owned
/// cache slots without invalidation or re-planning.
#[must_use]
pub fn build_runtime_computed_artifact(
    model: &ApplicationSemanticModel,
    ir: &IntermediateRepresentation,
) -> RuntimeComputedArtifact {
    let registry = build_runtime_computed_registry(model, ir);
    let available = registry_ids(&registry);
    let evaluations = runtime_evaluations(model, &registry, ir);
    let emitted = evaluations
        .iter()
        .map(|evaluation| evaluation.computed.clone())
        .collect::<BTreeSet<_>>();
    let state = runtime_state(model);

    RuntimeComputedArtifact {
        schema_version: RUNTIME_COMPUTED_ARTIFACT_SCHEMA_VERSION,
        invalidations: runtime_invalidations(model, &state, &emitted),
        state,
        evaluations,
        evaluation_order: planned_evaluations(model, &available, &emitted),
        update_batches: planned_batches(model, &available, &emitted),
    }
}

fn registry_ids(registry: &crate::RuntimeComputedRegistry) -> BTreeSet<String> {
    registry
        .records
        .keys()
        .map(|computed| computed.as_str().to_string())
        .collect()
}

fn runtime_evaluations(
    model: &ApplicationSemanticModel,
    registry: &crate::RuntimeComputedRegistry,
    ir: &IntermediateRepresentation,
) -> Vec<RuntimeComputedArtifactEvaluation> {
    let programs = computed_programs(ir);
    registry
        .records
        .values()
        .filter_map(|record| {
            let component = model
                .components
                .iter()
                .find(|component| record.computed.as_str().starts_with(component.id.as_str()))?;
            let program = programs.get(&record.computed)?.clone();
            Some(RuntimeComputedArtifactEvaluation {
                computed: record.computed.as_str().to_string(),
                component: component.class_name.clone(),
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
                program,
            })
        })
        .collect()
}

fn runtime_state(model: &ApplicationSemanticModel) -> Vec<RuntimeComputedArtifactState> {
    model
        .components
        .iter()
        .flat_map(|component| {
            component
                .state_fields
                .iter()
                .map(move |field| RuntimeComputedArtifactState {
                    component: component.class_name.clone(),
                    field: field.name.clone(),
                    storage: IrStorageId::for_semantic_origin(&field.id)
                        .as_str()
                        .to_string(),
                    initial_value: field.initial_value.clone(),
                })
        })
        .collect()
}

fn runtime_invalidations(
    model: &ApplicationSemanticModel,
    state: &[RuntimeComputedArtifactState],
    emitted: &BTreeSet<String>,
) -> Vec<RuntimeComputedArtifactInvalidation> {
    state
        .iter()
        .filter_map(|state| {
            let field = model.components.iter().find_map(|component| {
                (component.class_name == state.component)
                    .then(|| {
                        component
                            .state_fields
                            .iter()
                            .find(|field| field.name == state.field)
                    })
                    .flatten()
            })?;
            Some(RuntimeComputedArtifactInvalidation {
                storage: state.storage.clone(),
                dependents: model
                    .reactive_transitive_analysis
                    .dependents_of(field.id.as_str())
                    .iter()
                    .filter(|computed| emitted.contains(computed.as_str()))
                    .cloned()
                    .collect(),
            })
        })
        .collect()
}

fn planned_evaluations(
    model: &ApplicationSemanticModel,
    available: &BTreeSet<String>,
    emitted: &BTreeSet<String>,
) -> Vec<String> {
    model
        .computed_evaluation_plan
        .evaluation_order
        .iter()
        .filter(|computed| {
            available.contains(computed.as_str()) && emitted.contains(computed.as_str())
        })
        .cloned()
        .collect()
}

fn planned_batches(
    model: &ApplicationSemanticModel,
    available: &BTreeSet<String>,
    emitted: &BTreeSet<String>,
) -> Vec<Vec<String>> {
    model
        .computed_evaluation_plan
        .update_batches
        .iter()
        .map(|batch| {
            batch
                .iter()
                .filter(|computed| {
                    available.contains(computed.as_str()) && emitted.contains(computed.as_str())
                })
                .cloned()
                .collect::<Vec<_>>()
        })
        .filter(|batch| !batch.is_empty())
        .collect()
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

fn computed_programs(
    ir: &IntermediateRepresentation,
) -> BTreeMap<SemanticId, RuntimeComputedArtifactProgram> {
    ir.modules
        .iter()
        .flat_map(|module| {
            module.computed_evaluations.iter().filter_map(|evaluation| {
                let function = module
                    .functions
                    .iter()
                    .find(|function| function.id == evaluation.function)?;
                let instructions = function
                    .blocks
                    .iter()
                    .flat_map(|block| block.instructions.iter())
                    .map(runtime_instruction)
                    .collect::<Option<Vec<_>>>()?;
                Some((
                    evaluation.computed.clone(),
                    RuntimeComputedArtifactProgram {
                        result: evaluation.result.as_str().to_string(),
                        instructions,
                    },
                ))
            })
        })
        .collect()
}

fn runtime_instruction(
    instruction: &crate::IrInstruction,
) -> Option<RuntimeComputedArtifactInstruction> {
    let result = instruction.result.as_ref()?.as_str().to_string();
    match &instruction.kind {
        IrInstructionKind::Constant { value } => {
            Some(RuntimeComputedArtifactInstruction::Constant {
                result,
                value: runtime_constant(value),
            })
        }
        IrInstructionKind::LoadStorage { storage } => {
            Some(RuntimeComputedArtifactInstruction::LoadState {
                result,
                storage: storage.as_str().to_string(),
            })
        }
        IrInstructionKind::LoadComputed { computed } => {
            Some(RuntimeComputedArtifactInstruction::LoadComputed {
                result,
                computed: computed.as_str().to_string(),
            })
        }
        IrInstructionKind::GetMember { object, property } => {
            Some(RuntimeComputedArtifactInstruction::GetMember {
                result,
                object: runtime_operand(object),
                property: property.clone(),
            })
        }
        IrInstructionKind::Binary {
            operation,
            left,
            right,
        } => Some(RuntimeComputedArtifactInstruction::Binary {
            result,
            operation: binary_operation(*operation),
            left: runtime_operand(left),
            right: runtime_operand(right),
        }),
        IrInstructionKind::Unary { operation, operand } => {
            Some(RuntimeComputedArtifactInstruction::Unary {
                result,
                operation: unary_operation(*operation),
                operand: runtime_operand(operand),
            })
        }
        IrInstructionKind::Nop
        | IrInstructionKind::Copy { .. }
        | IrInstructionKind::InitializeStorage { .. }
        | IrInstructionKind::StoreStorage { .. } => None,
    }
}

fn runtime_operand(operand: &IrOperand) -> RuntimeComputedArtifactOperand {
    match operand {
        IrOperand::Value(value) => RuntimeComputedArtifactOperand::Value {
            value: value.as_str().to_string(),
        },
        IrOperand::Constant(value) => RuntimeComputedArtifactOperand::Constant {
            value: runtime_constant(value),
        },
        IrOperand::Storage(storage) => RuntimeComputedArtifactOperand::Storage {
            storage: storage.as_str().to_string(),
        },
    }
}

fn runtime_constant(constant: &IrConstant) -> SerializableValue {
    match constant {
        IrConstant::Null => SerializableValue::Null,
        IrConstant::Boolean(value) => SerializableValue::Boolean(*value),
        IrConstant::Number(value) => SerializableValue::Number(value.clone()),
        IrConstant::String(value) => SerializableValue::String(value.clone()),
        IrConstant::Array(value) => SerializableValue::Array(value.clone()),
        IrConstant::Object(value) => SerializableValue::Object(value.clone()),
    }
}

const fn binary_operation(operation: IrBinaryOperation) -> RuntimeComputedArtifactBinaryOperation {
    match operation {
        IrBinaryOperation::Add => RuntimeComputedArtifactBinaryOperation::Add,
        IrBinaryOperation::Subtract => RuntimeComputedArtifactBinaryOperation::Subtract,
        IrBinaryOperation::Multiply => RuntimeComputedArtifactBinaryOperation::Multiply,
        IrBinaryOperation::Divide => RuntimeComputedArtifactBinaryOperation::Divide,
        IrBinaryOperation::Remainder => RuntimeComputedArtifactBinaryOperation::Remainder,
        IrBinaryOperation::Equal => RuntimeComputedArtifactBinaryOperation::Equal,
        IrBinaryOperation::NotEqual => RuntimeComputedArtifactBinaryOperation::NotEqual,
        IrBinaryOperation::LessThan => RuntimeComputedArtifactBinaryOperation::LessThan,
        IrBinaryOperation::LessThanOrEqual => {
            RuntimeComputedArtifactBinaryOperation::LessThanOrEqual
        }
        IrBinaryOperation::GreaterThan => RuntimeComputedArtifactBinaryOperation::GreaterThan,
        IrBinaryOperation::GreaterThanOrEqual => {
            RuntimeComputedArtifactBinaryOperation::GreaterThanOrEqual
        }
        IrBinaryOperation::And => RuntimeComputedArtifactBinaryOperation::And,
        IrBinaryOperation::Or => RuntimeComputedArtifactBinaryOperation::Or,
        IrBinaryOperation::NullishCoalesce => {
            RuntimeComputedArtifactBinaryOperation::NullishCoalesce
        }
    }
}

const fn unary_operation(operation: IrUnaryOperation) -> RuntimeComputedArtifactUnaryOperation {
    match operation {
        IrUnaryOperation::Not => RuntimeComputedArtifactUnaryOperation::Not,
        IrUnaryOperation::Identity => RuntimeComputedArtifactUnaryOperation::Identity,
        IrUnaryOperation::Negate => RuntimeComputedArtifactUnaryOperation::Negate,
    }
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
        build_application_semantic_model, build_runtime_computed_artifact, lower_components_to_ir,
        runtime_computed_artifact_json, RUNTIME_COMPUTED_ARTIFACT_SCHEMA_VERSION,
    };

    #[test]
    fn emits_deterministic_runtime_programs_from_canonical_ir() {
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
        let ir = lower_components_to_ir(&model);
        let artifact = build_runtime_computed_artifact(&model, &ir);

        assert_eq!(
            artifact.schema_version,
            RUNTIME_COMPUTED_ARTIFACT_SCHEMA_VERSION
        );
        assert_eq!(artifact.state.len(), 1);
        assert_eq!(artifact.state[0].field, "count");
        assert_eq!(artifact.invalidations.len(), 1);
        assert_eq!(
            artifact.invalidations[0].dependents,
            vec![doubled.to_string(), label.to_string()]
        );
        assert_eq!(artifact.evaluations.len(), 2);
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
        assert_eq!(
            artifact.evaluation_order,
            vec![doubled.to_string(), label.to_string()]
        );
        assert!(artifact.evaluations.iter().all(|evaluation| {
            !evaluation.program.instructions.is_empty()
                && evaluation.program.result.contains("/value:")
                && evaluation.dirty_flag.initial_value
        }));

        let first = runtime_computed_artifact_json(&artifact);
        let second = runtime_computed_artifact_json(&build_runtime_computed_artifact(&model, &ir));
        assert_eq!(first, second);
        let json: serde_json::Value = serde_json::from_str(&first).expect("artifact JSON");
        assert_eq!(json["schema_version"], 3);
        assert_eq!(
            json["evaluations"][0]["program"]["instructions"][0]["kind"],
            "load-state"
        );
        assert_eq!(
            json["evaluations"][1]["program"]["instructions"][0]["kind"],
            "load-computed"
        );
    }
}
