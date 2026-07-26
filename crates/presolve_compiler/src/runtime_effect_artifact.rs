use std::collections::BTreeMap;

use serde::Serialize;

use crate::runtime_computed_artifact::{
    RuntimeComputedArtifactInstruction, RuntimeComputedArtifactOperand,
};
use crate::{
    build_runtime_effect_instance_registry, build_runtime_effect_registry,
    ApplicationSemanticModel, EffectExecutionPolicy, EffectRenderBoundary, ExecutionBoundary,
    IntermediateRepresentation, IrInstruction, IrInstructionKind, IrValueId, RuntimeEffectRecord,
    EFFECT_CAPABILITY_REGISTRY,
};

pub const RUNTIME_EFFECT_ARTIFACT_SCHEMA_VERSION: u32 = 5;

/// Versioned compiler-generated runtime metadata and effect programs.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct RuntimeEffectArtifact {
    pub schema_version: u32,
    pub effects: Vec<RuntimeEffectArtifactEffect>,
    pub instances: Vec<RuntimeEffectArtifactInstance>,
}

/// One instance-qualified V2 effect ownership record. Programs remain on the
/// declaration record until instance execution context is available.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct RuntimeEffectArtifactInstance {
    pub effect_instance: String,
    pub effect: String,
    pub component_instance: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parent_instance: Option<String>,
    pub depth: usize,
    pub declaration_order: u32,
}

/// Runtime metadata and executable capability program for one lowered effect.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct RuntimeEffectArtifactEffect {
    pub effect: String,
    pub execution_function: String,
    pub initial_trigger_policy: RuntimeEffectArtifactExecutionPolicy,
    pub initial_trigger: Option<RuntimeEffectArtifactInitialTrigger>,
    pub action_batch_triggers: Vec<RuntimeEffectArtifactActionTrigger>,
    pub capability_operations: Vec<RuntimeEffectArtifactCapabilityOperation>,
    pub execution_boundary: RuntimeEffectArtifactExecutionBoundary,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub declaration_order: Option<u32>,
    /// Whether this V2 field effect is eligible for its single post-resume run.
    pub run_on_resume: bool,
    pub program: RuntimeEffectArtifactProgram,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cleanup_program: Option<RuntimeEffectArtifactProgram>,
}

/// Explicit initial-render trigger metadata for one emitted effect.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct RuntimeEffectArtifactInitialTrigger {
    pub render_boundary: RuntimeEffectArtifactRenderBoundary,
    pub required_computed: Vec<String>,
    pub prerequisite_batches: Vec<RuntimeEffectArtifactPrerequisiteBatch>,
    pub effect_batch_index: u32,
}

/// Explicit completed-action-batch trigger metadata for one emitted effect.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct RuntimeEffectArtifactActionTrigger {
    pub action_batch: String,
    pub matched_states: Vec<String>,
    pub required_computed: Vec<String>,
    pub prerequisite_batches: Vec<RuntimeEffectArtifactPrerequisiteBatch>,
    pub effect_batch_index: u32,
}

/// A filtered F9 computed-prerequisite batch available to an effect trigger.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct RuntimeEffectArtifactPrerequisiteBatch {
    pub source_batch_index: u32,
    pub computed: Vec<String>,
}

/// Stable compiler operation and runtime-lowering identities.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct RuntimeEffectArtifactCapabilityOperation {
    pub operation: String,
    pub runtime_lowering: String,
}

/// One complete, ordered F10/F11 effect function program.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct RuntimeEffectArtifactProgram {
    pub instructions: Vec<RuntimeEffectArtifactInstruction>,
}

/// An effect program instruction. Pure operand instructions reuse the existing
/// computed-artifact representation; capability operations are explicit roots.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(untagged)]
pub enum RuntimeEffectArtifactInstruction {
    Evaluate(RuntimeComputedArtifactInstruction),
    CapabilityCall {
        #[serde(rename = "kind")]
        kind: RuntimeEffectArtifactCapabilityInstructionKind,
        operation: String,
        runtime_lowering: String,
        arguments: Vec<RuntimeComputedArtifactOperand>,
    },
    CapabilityAssign {
        #[serde(rename = "kind")]
        kind: RuntimeEffectArtifactCapabilityInstructionKind,
        operation: String,
        runtime_lowering: String,
        value: RuntimeComputedArtifactOperand,
    },
}

/// Distinguishes the two observable capability instruction forms.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum RuntimeEffectArtifactCapabilityInstructionKind {
    CapabilityCall,
    CapabilityAssign,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum RuntimeEffectArtifactExecutionPolicy {
    AfterInitialRenderAndCompletedActionBatch,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum RuntimeEffectArtifactRenderBoundary {
    AfterInitialRender,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum RuntimeEffectArtifactExecutionBoundary {
    Client,
    Server,
}

/// Emit a deterministic effect artifact from F9 plans, F10/F11 IR, and F12 records.
///
/// The artifact contains canonical semantic and registry identities only. It
/// neither retains raw capability paths nor performs dependency discovery or
/// effect execution.
#[must_use]
pub fn build_runtime_effect_artifact(
    model: &ApplicationSemanticModel,
    ir: &IntermediateRepresentation,
) -> RuntimeEffectArtifact {
    let registry = build_runtime_effect_registry(model, ir);
    let programs = effect_programs(ir);
    let effects = registry
        .records
        .values()
        .filter_map(|record| {
            let programs = programs.get(&record.effect)?;
            runtime_effect(record, programs.main.clone(), programs.cleanup.clone())
        })
        .collect();
    RuntimeEffectArtifact {
        schema_version: RUNTIME_EFFECT_ARTIFACT_SCHEMA_VERSION,
        effects,
        instances: build_runtime_effect_instance_registry(model)
            .records
            .into_iter()
            .map(|record| RuntimeEffectArtifactInstance {
                effect_instance: record.id.as_str().to_owned(),
                effect: record.effect.to_string(),
                component_instance: record.component_instance.as_str().to_owned(),
                parent_instance: record
                    .parent_instance
                    .map(|parent| parent.as_str().to_owned()),
                depth: record.depth,
                declaration_order: record.declaration_order,
            })
            .collect(),
    }
}

/// Serialize emitted effect runtime metadata as deterministic, pretty JSON.
///
/// # Panics
///
/// Panics when compiler-owned effect runtime metadata cannot serialize.
#[must_use]
pub fn runtime_effect_artifact_json(artifact: &RuntimeEffectArtifact) -> String {
    serde_json::to_string_pretty(artifact).expect("effect runtime artifact should serialize") + "\n"
}

fn runtime_effect(
    record: &RuntimeEffectRecord,
    program: RuntimeEffectArtifactProgram,
    cleanup_program: Option<RuntimeEffectArtifactProgram>,
) -> Option<RuntimeEffectArtifactEffect> {
    Some(RuntimeEffectArtifactEffect {
        effect: record.effect.as_str().to_string(),
        execution_function: record.execution_function.as_str().to_string(),
        initial_trigger_policy: execution_policy(record.initial_trigger_policy),
        initial_trigger: record.initial_trigger.as_ref().map(|trigger| {
            RuntimeEffectArtifactInitialTrigger {
                render_boundary: render_boundary(trigger.render_boundary),
                required_computed: semantic_ids(&trigger.required_computed),
                prerequisite_batches: prerequisite_batches(&trigger.prerequisite_batches),
                effect_batch_index: trigger.effect_batch_index,
            }
        }),
        action_batch_triggers: record
            .action_batch_triggers
            .iter()
            .map(|trigger| RuntimeEffectArtifactActionTrigger {
                action_batch: trigger.action_batch.as_str().to_string(),
                matched_states: semantic_ids(&trigger.matched_states),
                required_computed: semantic_ids(&trigger.required_computed),
                prerequisite_batches: prerequisite_batches(&trigger.prerequisite_batches),
                effect_batch_index: trigger.effect_batch_index,
            })
            .collect(),
        capability_operations: capability_operations(&record.capability_operations)?,
        execution_boundary: execution_boundary(record.execution_boundary),
        declaration_order: record.declaration_order,
        run_on_resume: record.run_on_resume,
        program,
        cleanup_program,
    })
}

#[derive(Debug, Clone)]
struct EffectPrograms {
    main: RuntimeEffectArtifactProgram,
    cleanup: Option<RuntimeEffectArtifactProgram>,
}

fn effect_programs(ir: &IntermediateRepresentation) -> BTreeMap<crate::SemanticId, EffectPrograms> {
    ir.modules
        .iter()
        .flat_map(|module| {
            module.effect_executions.iter().filter_map(|execution| {
                let function = module
                    .functions
                    .iter()
                    .find(|function| function.id == execution.function)?;
                let program = runtime_effect_program(function)?;
                let cleanup = if let Some(cleanup_id) = &execution.cleanup_function {
                    let function = module
                        .functions
                        .iter()
                        .find(|function| function.id == *cleanup_id)?;
                    Some(runtime_effect_program(function)?)
                } else {
                    None
                };
                Some((
                    execution.effect.clone(),
                    EffectPrograms {
                        main: program,
                        cleanup,
                    },
                ))
            })
        })
        .collect()
}

fn runtime_effect_program(function: &crate::IrFunction) -> Option<RuntimeEffectArtifactProgram> {
    let instructions = function
        .blocks
        .iter()
        .flat_map(|block| block.instructions.iter())
        .map(runtime_instruction)
        .collect::<Option<Vec<_>>>()?;
    Some(RuntimeEffectArtifactProgram { instructions })
}

fn runtime_instruction(instruction: &IrInstruction) -> Option<RuntimeEffectArtifactInstruction> {
    match &instruction.kind {
        IrInstructionKind::CapabilityCall {
            operation,
            arguments,
        } => Some(RuntimeEffectArtifactInstruction::CapabilityCall {
            kind: RuntimeEffectArtifactCapabilityInstructionKind::CapabilityCall,
            operation: operation.0.to_string(),
            runtime_lowering: runtime_lowering(*operation)?,
            arguments: arguments.iter().map(runtime_value_operand).collect(),
        }),
        IrInstructionKind::CapabilityAssign { operation, value } => {
            Some(RuntimeEffectArtifactInstruction::CapabilityAssign {
                kind: RuntimeEffectArtifactCapabilityInstructionKind::CapabilityAssign,
                operation: operation.0.to_string(),
                runtime_lowering: runtime_lowering(*operation)?,
                value: runtime_value_operand(value),
            })
        }
        _ => crate::runtime_computed_artifact::runtime_instruction(instruction)
            .map(RuntimeEffectArtifactInstruction::Evaluate),
    }
}

fn capability_operations(
    operations: &[crate::CapabilityOperationId],
) -> Option<Vec<RuntimeEffectArtifactCapabilityOperation>> {
    operations
        .iter()
        .map(|operation| {
            Some(RuntimeEffectArtifactCapabilityOperation {
                operation: operation.0.to_string(),
                runtime_lowering: runtime_lowering(*operation)?,
            })
        })
        .collect()
}

fn runtime_lowering(operation: crate::CapabilityOperationId) -> Option<String> {
    EFFECT_CAPABILITY_REGISTRY
        .operation(operation)
        .map(|definition| definition.runtime_lowering.0.to_string())
}

fn runtime_value_operand(value: &IrValueId) -> RuntimeComputedArtifactOperand {
    RuntimeComputedArtifactOperand::Value {
        value: value.as_str().to_string(),
    }
}

fn semantic_ids(ids: &[crate::SemanticId]) -> Vec<String> {
    ids.iter().map(ToString::to_string).collect()
}

fn prerequisite_batches(
    batches: &[crate::EffectComputedPrerequisiteBatch],
) -> Vec<RuntimeEffectArtifactPrerequisiteBatch> {
    batches
        .iter()
        .map(|batch| RuntimeEffectArtifactPrerequisiteBatch {
            source_batch_index: batch.source_batch_index,
            computed: semantic_ids(&batch.computed),
        })
        .collect()
}

const fn execution_policy(policy: EffectExecutionPolicy) -> RuntimeEffectArtifactExecutionPolicy {
    match policy {
        EffectExecutionPolicy::AfterInitialRenderAndCompletedActionBatch => {
            RuntimeEffectArtifactExecutionPolicy::AfterInitialRenderAndCompletedActionBatch
        }
    }
}

const fn render_boundary(boundary: EffectRenderBoundary) -> RuntimeEffectArtifactRenderBoundary {
    match boundary {
        EffectRenderBoundary::AfterInitialRender => {
            RuntimeEffectArtifactRenderBoundary::AfterInitialRender
        }
    }
}

const fn execution_boundary(boundary: ExecutionBoundary) -> RuntimeEffectArtifactExecutionBoundary {
    match boundary {
        ExecutionBoundary::Client => RuntimeEffectArtifactExecutionBoundary::Client,
        ExecutionBoundary::Server => RuntimeEffectArtifactExecutionBoundary::Server,
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        build_application_semantic_model, build_runtime_effect_artifact, lower_components_to_ir,
        optimize_effect_ir, runtime_effect_artifact_json,
        RuntimeEffectArtifactCapabilityInstructionKind, RuntimeEffectArtifactInstruction,
        RUNTIME_EFFECT_ARTIFACT_SCHEMA_VERSION,
    };

    #[test]
    #[allow(clippy::too_many_lines)]
    fn emits_deterministic_effect_programs_from_registry_and_optimized_ir() {
        let parsed = presolve_parser::parse_file(
            "src/RuntimeEffectArtifact.tsx",
            r#"
@component("x-runtime-effect-artifact")
class RuntimeEffectArtifact extends Component {
  count = state(1);
  title = state("Presolve");

  @computed()
  get doubled() { return this.count * 2; }

  @action()
  increment() { this.count += 1; }

  @effect()
  report() {
    console.log(1 + 2, this.doubled);
    document.title = this.title;
    localStorage.setItem("count", "updated");
  }

  @action()
  invalidAction() { this.count += 1; }

  @effect()
  invalid() { this.invalidAction(); }

  render() { return <p />; }
}
"#,
        );
        let model = build_application_semantic_model(&parsed);
        let component = &model.components[0];
        let report = component.id.effect("report");
        let doubled = component.id.computed("doubled");
        let increment = component.id.action_batch("increment");
        let invalid_action = component.id.action_batch("invalidAction");
        let artifact = build_runtime_effect_artifact(
            &model,
            &optimize_effect_ir(&lower_components_to_ir(&model)).output,
        );
        let effect = artifact.effects.first().expect("report artifact");
        let operations = effect
            .program
            .instructions
            .iter()
            .filter_map(|instruction| match instruction {
                RuntimeEffectArtifactInstruction::CapabilityCall { operation, .. }
                | RuntimeEffectArtifactInstruction::CapabilityAssign { operation, .. } => {
                    Some(operation.as_str())
                }
                RuntimeEffectArtifactInstruction::Evaluate(_) => None,
            })
            .collect::<Vec<_>>();

        assert_eq!(
            artifact.schema_version,
            RUNTIME_EFFECT_ARTIFACT_SCHEMA_VERSION
        );
        assert_eq!(artifact.effects.len(), 1);
        assert!(artifact.instances.is_empty());
        assert!(effect.cleanup_program.is_none());
        assert!(!effect.run_on_resume);
        assert_eq!(effect.effect, report.as_str());
        assert_eq!(effect.execution_function, report.as_str());
        assert_eq!(
            effect
                .initial_trigger
                .as_ref()
                .expect("initial trigger")
                .required_computed,
            vec![doubled.to_string()]
        );
        assert_eq!(effect.action_batch_triggers.len(), 2);
        assert_eq!(
            effect.action_batch_triggers[0].action_batch,
            increment.as_str()
        );
        assert_eq!(
            effect.action_batch_triggers[0].required_computed,
            vec![doubled.to_string()]
        );
        assert_eq!(
            effect.action_batch_triggers[1].action_batch,
            invalid_action.as_str()
        );
        assert_eq!(
            effect.action_batch_triggers[1].required_computed,
            vec![doubled.to_string()]
        );
        assert_eq!(
            effect
                .capability_operations
                .iter()
                .map(|operation| operation.operation.as_str())
                .collect::<Vec<_>>(),
            vec![
                "builtin.browser.console.log",
                "builtin.browser.document.title.assign",
                "builtin.browser.local_storage.set_item",
            ]
        );
        assert!(effect
            .capability_operations
            .iter()
            .all(|operation| operation.operation == operation.runtime_lowering));
        assert_eq!(
            operations,
            vec![
                "builtin.browser.console.log",
                "builtin.browser.document.title.assign",
                "builtin.browser.local_storage.set_item",
            ]
        );
        assert!(effect.program.instructions.iter().any(|instruction| {
            matches!(
                instruction,
                RuntimeEffectArtifactInstruction::CapabilityCall {
                    kind: RuntimeEffectArtifactCapabilityInstructionKind::CapabilityCall,
                    runtime_lowering,
                    ..
                } if runtime_lowering == "builtin.browser.console.log"
            )
        }));

        let first = runtime_effect_artifact_json(&artifact);
        let second = runtime_effect_artifact_json(&build_runtime_effect_artifact(
            &model,
            &optimize_effect_ir(&lower_components_to_ir(&model)).output,
        ));
        assert_eq!(first, second);
        let json: serde_json::Value = serde_json::from_str(&first).expect("artifact JSON");
        assert_eq!(json["schema_version"], 5);
        assert_eq!(
            json["effects"][0]["program"]["instructions"][2]["kind"],
            "capability-call"
        );
        assert!(json["effects"][0].get("provenance").is_none());
        assert!(json["effects"][0]["program"]["instructions"]
            .as_array()
            .expect("instructions")
            .iter()
            .all(|instruction| instruction.get("static_path").is_none()));
    }

    #[test]
    fn emits_distinct_v2_effect_instances_for_repeated_component_instances() {
        let parsed = presolve_parser::parse_file(
            "src/RepeatedEffectArtifact.tsx",
            r#"
@component("x-card") class Card extends Component {
  @effect() report() { document.title = "card"; }
  render() { return <article />; }
}
@component("x-page") class Page extends Component {
  render() { return <main><Card /><Card /></main>; }
}
"#,
        );
        let mut model = build_application_semantic_model(&parsed);
        let effect_id = model.components[0].id.effect("report");
        let effect = model.effects.get_mut(&effect_id).expect("Card effect");
        effect.declaration = crate::EffectDeclaration::V2Field;
        effect.declaration_order = Some(0);
        let artifact = build_runtime_effect_artifact(
            &model,
            &optimize_effect_ir(&lower_components_to_ir(&model)).output,
        );

        assert_eq!(artifact.effects.len(), 1);
        assert_eq!(artifact.instances.len(), 2);
        assert!(artifact
            .instances
            .iter()
            .all(|record| record.effect == effect_id.as_str()));
        assert_ne!(
            artifact.instances[0].effect_instance,
            artifact.instances[1].effect_instance
        );
        assert!(artifact
            .instances
            .iter()
            .all(|record| record.parent_instance.is_some()));
        assert!(artifact.instances.iter().all(|record| record.depth == 1));
    }
}
