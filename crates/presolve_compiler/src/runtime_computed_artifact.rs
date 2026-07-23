use std::collections::{BTreeMap, BTreeSet};

use serde::Serialize;

use crate::{
    build_runtime_computed_registry, ApplicationSemanticModel, IntermediateRepresentation,
    IrBinaryOperation, IrConstant, IrInstructionKind, IrOperand, IrStorageId, IrUnaryOperation,
    SemanticId, SerializableValue, SerializationCompatibility,
};

pub const RUNTIME_COMPUTED_ARTIFACT_SCHEMA_VERSION: u32 = 11;

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
        optional: bool,
    },
    GetIndex {
        result: String,
        object: RuntimeComputedArtifactOperand,
        index: RuntimeComputedArtifactOperand,
    },
    Select {
        result: String,
        condition: RuntimeComputedArtifactOperand,
        when_true: RuntimeComputedArtifactOperand,
        when_false: RuntimeComputedArtifactOperand,
    },
    Template {
        result: String,
        quasis: Vec<String>,
        expressions: Vec<String>,
    },
    PurePackageCall {
        result: String,
        package: String,
        version: String,
        integrity: String,
        export: String,
        runtime_module: String,
        resume_policy: String,
        operation: crate::semantic_package::SemanticPackagePureOperation,
        arguments: Vec<String>,
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
    Min,
    Max,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum RuntimeComputedArtifactUnaryOperation {
    Not,
    Identity,
    Negate,
    Abs,
    Floor,
    Ceil,
    Round,
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
                    id: record.dirty_flag.id.as_str().to_string(),
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

pub(crate) fn runtime_instruction(
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
        IrInstructionKind::GetMember {
            object,
            property,
            optional,
        } => Some(RuntimeComputedArtifactInstruction::GetMember {
            result,
            object: runtime_operand(object),
            property: property.clone(),
            optional: *optional,
        }),
        IrInstructionKind::GetIndex { object, index } => {
            Some(RuntimeComputedArtifactInstruction::GetIndex {
                result,
                object: runtime_operand(object),
                index: runtime_operand(index),
            })
        }
        IrInstructionKind::Select {
            condition,
            when_true,
            when_false,
        } => Some(RuntimeComputedArtifactInstruction::Select {
            result,
            condition: runtime_operand(condition),
            when_true: runtime_operand(when_true),
            when_false: runtime_operand(when_false),
        }),
        IrInstructionKind::Template {
            quasis,
            expressions,
        } => Some(RuntimeComputedArtifactInstruction::Template {
            result,
            quasis: quasis.clone(),
            expressions: expressions
                .iter()
                .map(|expression| expression.as_str().to_string())
                .collect(),
        }),
        IrInstructionKind::PurePackageCall {
            package,
            version,
            integrity,
            export,
            runtime_module,
            resume_policy,
            operation,
            arguments,
        } => Some(RuntimeComputedArtifactInstruction::PurePackageCall {
            result,
            package: package.clone(),
            version: version.clone(),
            integrity: integrity.clone(),
            export: export.clone(),
            runtime_module: runtime_module.clone(),
            resume_policy: resume_policy.clone(),
            operation: *operation,
            arguments: arguments
                .iter()
                .map(|argument| argument.as_str().to_string())
                .collect(),
        }),
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
        | IrInstructionKind::InitializeContextSlot { .. }
        | IrInstructionKind::StoreStorage { .. }
        | IrInstructionKind::LoadContextSlot { .. }
        | IrInstructionKind::CapabilityCall { .. }
        | IrInstructionKind::CapabilityAssign { .. } => None,
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
        IrBinaryOperation::Min => RuntimeComputedArtifactBinaryOperation::Min,
        IrBinaryOperation::Max => RuntimeComputedArtifactBinaryOperation::Max,
    }
}

const fn unary_operation(operation: IrUnaryOperation) -> RuntimeComputedArtifactUnaryOperation {
    match operation {
        IrUnaryOperation::Not => RuntimeComputedArtifactUnaryOperation::Not,
        IrUnaryOperation::Identity => RuntimeComputedArtifactUnaryOperation::Identity,
        IrUnaryOperation::Negate => RuntimeComputedArtifactUnaryOperation::Negate,
        IrUnaryOperation::Abs => RuntimeComputedArtifactUnaryOperation::Abs,
        IrUnaryOperation::Floor => RuntimeComputedArtifactUnaryOperation::Floor,
        IrUnaryOperation::Ceil => RuntimeComputedArtifactUnaryOperation::Ceil,
        IrUnaryOperation::Round => RuntimeComputedArtifactUnaryOperation::Round,
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
    use super::RuntimeComputedArtifactInstruction;
    use crate::{
        build_application_semantic_model, build_runtime_computed_artifact, lower_components_to_ir,
        runtime_computed_artifact_json, RUNTIME_COMPUTED_ARTIFACT_SCHEMA_VERSION,
    };

    #[test]
    fn emits_deterministic_runtime_programs_from_canonical_ir() {
        let parsed = presolve_parser::parse_file(
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
        assert_eq!(json["schema_version"], 11);
        assert_eq!(
            json["evaluations"][0]["program"]["instructions"][0]["kind"],
            "load-state"
        );
        assert_eq!(
            json["evaluations"][1]["program"]["instructions"][0]["kind"],
            "load-computed"
        );
    }

    #[test]
    fn emits_compiler_lowered_identity_package_call_with_contract_provenance() {
        let unit = crate::CompilationUnit::parse_sources([(
            "src/PackageComputed.tsx",
            r#"
import { identity as visible } from "value-kit";

@component("x-package-computed")
class PackageComputed extends Component {
  count = state(1);

  @computed()
  get visibleCount() { return visible(this.count); }

  render() { return <output>Visible count</output>; }
}
"#,
        )]);
        let contract = crate::semantic_package::parse_semantic_package_contract(
            r#"{"schema_version":1,"package":"value-kit","version":"1.0.0","integrity":"sha256:bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb","exports":{"identity":{"kind":"pure","type_signature":"<T>(T)->T","runtime_module":"dist/identity.js","resume_policy":"input_only","pure_operation":"identity"}}}"#,
        )
        .expect("valid package contract");
        let mut packages = crate::semantic_package::SemanticPackageResolutionTable::default();
        packages.insert("value-kit".into(), contract).unwrap();

        let model = crate::application_semantic_model::build_application_semantic_model_for_unit_with_packages(&unit, &packages);
        assert!(
            model.diagnostics.is_empty(),
            "package model diagnostics: {:?}",
            model.diagnostics
        );
        assert!(model
            .diagnostics
            .iter()
            .all(|diagnostic| diagnostic.code != "PSC3015"));
        let ir = lower_components_to_ir(&model);
        let artifact = build_runtime_computed_artifact(&model, &ir);
        let instruction = artifact.evaluations[0]
            .program
            .instructions
            .iter()
            .find(|instruction| {
                matches!(
                    instruction,
                    RuntimeComputedArtifactInstruction::PurePackageCall { .. }
                )
            })
            .expect("lowered pure package call");
        let RuntimeComputedArtifactInstruction::PurePackageCall {
            package,
            version,
            integrity,
            export,
            operation,
            arguments,
            ..
        } = instruction
        else {
            unreachable!();
        };
        assert_eq!(package, "value-kit");
        assert_eq!(version, "1.0.0");
        assert_eq!(
            integrity,
            "sha256:bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb"
        );
        assert_eq!(export, "identity");
        assert_eq!(
            *operation,
            crate::semantic_package::SemanticPackagePureOperation::Identity
        );
        assert_eq!(arguments.len(), 1);
    }

    #[test]
    fn emits_template_interpolation_program_with_cooked_segments() {
        let parsed = presolve_parser::parse_file(
            "src/TemplateComputed.tsx",
            r#"
@component("x-template-computed")
class TemplateComputed extends Component {
  count = state(2);
  @computed()
  get label() { return `Count: ${this.count}!`; }
  render() { return <output>Count</output>; }
}
"#,
        );
        let model = build_application_semantic_model(&parsed);
        assert!(model.diagnostics.is_empty(), "{:?}", model.diagnostics);
        let artifact = build_runtime_computed_artifact(&model, &lower_components_to_ir(&model));
        let instruction = artifact.evaluations[0]
            .program
            .instructions
            .iter()
            .find(|instruction| {
                matches!(
                    instruction,
                    RuntimeComputedArtifactInstruction::Template { .. }
                )
            })
            .expect("lowered template instruction");
        let RuntimeComputedArtifactInstruction::Template {
            quasis,
            expressions,
            ..
        } = instruction
        else {
            unreachable!();
        };
        assert_eq!(quasis, &["Count: ".to_string(), "!".to_string()]);
        assert_eq!(expressions.len(), 1);
    }

    #[test]
    fn emits_static_index_access_program_for_tuple_state() {
        let parsed = presolve_parser::parse_file(
            "src/IndexedComputed.tsx",
            r#"
@component("x-indexed-computed")
class IndexedComputed extends Component {
  labels = state(["zero", "one"]);
  record = state({ label: "object" });
  @computed()
  get selected() { return this.labels[1]; }
  @computed()
  get title() { return this.record["label"]; }
  render() { return <output>Indexed</output>; }
}
"#,
        );
        let model = build_application_semantic_model(&parsed);
        assert!(model.diagnostics.is_empty(), "{:?}", model.diagnostics);
        let artifact = build_runtime_computed_artifact(&model, &lower_components_to_ir(&model));
        let instructions = artifact
            .evaluations
            .iter()
            .flat_map(|evaluation| evaluation.program.instructions.iter())
            .filter(|instruction| {
                matches!(
                    instruction,
                    RuntimeComputedArtifactInstruction::GetIndex { .. }
                )
            })
            .collect::<Vec<_>>();
        assert_eq!(instructions.len(), 2);
        let instruction = instructions[0];
        let RuntimeComputedArtifactInstruction::GetIndex { index, .. } = instruction else {
            unreachable!();
        };
        assert!(matches!(
            index,
            super::RuntimeComputedArtifactOperand::Value { .. }
        ));
    }

    #[test]
    fn emits_boolean_conditional_select_program() {
        let parsed = presolve_parser::parse_file(
            "src/ConditionalComputed.tsx",
            r#"
@component("x-conditional-computed")
class ConditionalComputed extends Component {
  enabled: boolean = state(true);
  @computed()
  get label() { return this.enabled ? "enabled" : "disabled"; }
  render() { return <output>Conditional</output>; }
}
"#,
        );
        let model = build_application_semantic_model(&parsed);
        assert!(model.diagnostics.is_empty(), "{:?}", model.diagnostics);
        let artifact = build_runtime_computed_artifact(&model, &lower_components_to_ir(&model));
        assert!(artifact.evaluations[0]
            .program
            .instructions
            .iter()
            .any(|instruction| {
                matches!(
                    instruction,
                    RuntimeComputedArtifactInstruction::Select { .. }
                )
            }));
    }

    #[test]
    fn rejects_non_boolean_computed_conditional() {
        let parsed = presolve_parser::parse_file(
            "src/InvalidConditionalComputed.tsx",
            r#"
@component("x-invalid-conditional-computed")
class InvalidConditionalComputed extends Component {
  count: number = state(1);
  @computed()
  get label() { return this.count ? "one" : "zero"; }
  render() { return <output>Conditional</output>; }
}
"#,
        );
        let model = build_application_semantic_model(&parsed);
        assert!(
            model
                .diagnostics
                .iter()
                .any(|diagnostic| diagnostic.code == "PSC1029"),
            "{:?}",
            model.diagnostics
        );
    }

    #[test]
    fn emits_optional_member_read_with_compiler_retained_optionality() {
        let parsed = presolve_parser::parse_file(
            "src/OptionalComputed.tsx",
            r#"
@component("x-optional-computed")
class OptionalComputed extends Component {
  profile = state({ label: "Presolve" });
  @computed()
  get label() { return this.profile?.label; }
  render() { return <output>Optional</output>; }
}
"#,
        );
        let model = build_application_semantic_model(&parsed);
        assert!(model.diagnostics.is_empty(), "{:?}", model.diagnostics);
        let artifact = build_runtime_computed_artifact(&model, &lower_components_to_ir(&model));
        assert!(artifact.evaluations[0]
            .program
            .instructions
            .iter()
            .any(|instruction| {
                matches!(
                    instruction,
                    RuntimeComputedArtifactInstruction::GetMember { optional: true, .. }
                )
            }));
    }

    #[test]
    fn emits_compiler_registered_math_abs_as_unary_program() {
        let parsed = presolve_parser::parse_file(
            "src/AbsComputed.tsx",
            r#"
@component("x-abs-computed")
class AbsComputed extends Component {
  count = state(-2);
  @computed()
  get magnitude() { return Math.abs(this.count); }
  render() { return <output>Abs</output>; }
}
"#,
        );
        let model = build_application_semantic_model(&parsed);
        assert!(model.diagnostics.is_empty(), "{:?}", model.diagnostics);
        let artifact = build_runtime_computed_artifact(&model, &lower_components_to_ir(&model));
        assert!(artifact.evaluations[0]
            .program
            .instructions
            .iter()
            .any(|instruction| {
                matches!(
                    instruction,
                    RuntimeComputedArtifactInstruction::Unary {
                        operation: super::RuntimeComputedArtifactUnaryOperation::Abs,
                        ..
                    }
                )
            }));
    }

    #[test]
    fn emits_compiler_registered_math_rounding_as_unary_programs() {
        let parsed = presolve_parser::parse_file(
            "src/RoundingComputed.tsx",
            r#"
@component("x-rounding-computed")
class RoundingComputed extends Component {
  value = state(-1.5);
  @computed() get floorValue() { return Math.floor(this.value); }
  @computed() get ceilValue() { return Math.ceil(this.value); }
  @computed() get roundValue() { return Math.round(this.value); }
  render() { return <output>Rounding</output>; }
}
"#,
        );
        let model = build_application_semantic_model(&parsed);
        assert!(model.diagnostics.is_empty(), "{:?}", model.diagnostics);
        let artifact = build_runtime_computed_artifact(&model, &lower_components_to_ir(&model));
        let operations = artifact
            .evaluations
            .iter()
            .flat_map(|evaluation| evaluation.program.instructions.iter())
            .filter_map(|instruction| match instruction {
                RuntimeComputedArtifactInstruction::Unary { operation, .. } => Some(*operation),
                _ => None,
            })
            .collect::<Vec<_>>();
        assert!(operations.contains(&super::RuntimeComputedArtifactUnaryOperation::Floor));
        assert!(operations.contains(&super::RuntimeComputedArtifactUnaryOperation::Ceil));
        assert!(operations.contains(&super::RuntimeComputedArtifactUnaryOperation::Round));
    }

    #[test]
    fn emits_compiler_registered_math_min_and_max_as_binary_programs() {
        let parsed = presolve_parser::parse_file(
            "src/MinMaxComputed.tsx",
            r#"
@component("x-min-max-computed")
class MinMaxComputed extends Component {
  left = state(-2);
  right = state(5);
  @computed()
  get minimum() { return Math.min(this.left, this.right); }
  @computed()
  get maximum() { return Math.max(this.left, this.right); }
  render() { return <output>MinMax</output>; }
}
"#,
        );
        let model = build_application_semantic_model(&parsed);
        assert!(model.diagnostics.is_empty(), "{:?}", model.diagnostics);
        let artifact = build_runtime_computed_artifact(&model, &lower_components_to_ir(&model));
        let operations = artifact
            .evaluations
            .iter()
            .flat_map(|evaluation| evaluation.program.instructions.iter())
            .filter_map(|instruction| match instruction {
                RuntimeComputedArtifactInstruction::Binary { operation, .. } => Some(*operation),
                _ => None,
            })
            .collect::<Vec<_>>();
        assert!(operations.contains(&super::RuntimeComputedArtifactBinaryOperation::Min));
        assert!(operations.contains(&super::RuntimeComputedArtifactBinaryOperation::Max));
    }
}
