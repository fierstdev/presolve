use std::collections::{BTreeMap, BTreeSet};

use serde::Serialize;

use crate::{
    build_effect_resume_plan, build_runtime_effect_registry, lower_components_to_ir,
    optimize_effect_ir, ApplicationSemanticModel, CapabilityOperationKind, Effect,
    EffectActivationStatus, EffectExecutionPolicy, EffectRenderBoundary,
    EffectSemanticViolationKind, EffectValidation, ExecutionBoundary, IrReactiveEdgeKind,
    RuntimeEffectRecord, SemanticId, SourceProvenance, EFFECT_CAPABILITY_REGISTRY,
};

/// Immutable F17 projection of existing effect compiler products for inspection.
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize)]
pub struct EffectInspectionRegistry {
    pub records: BTreeMap<SemanticId, EffectInspection>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EffectInspectionValidationDiagnostic {
    pub code: String,
    pub message: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct EffectInspection {
    pub validation: EffectInspectionValidation,
    pub direct_dependencies: EffectInspectionDependencies,
    pub transitive_dependencies: EffectInspectionDependencies,
    pub dependents: Vec<String>,
    pub initial_trigger: Option<EffectInspectionInitialTrigger>,
    pub action_triggers: Vec<EffectInspectionActionTrigger>,
    pub schedule: EffectInspectionSchedule,
    pub capabilities: Vec<EffectInspectionCapability>,
    pub ir: Option<EffectInspectionIr>,
    pub runtime: EffectInspectionRuntime,
    pub resumability: Option<EffectInspectionResumability>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct EffectInspectionValidation {
    pub status: &'static str,
    pub violations: Vec<EffectInspectionViolation>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct EffectInspectionViolation {
    pub category: &'static str,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub statement_id: Option<String>,
    pub provenance: EffectInspectionProvenance,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct EffectInspectionProvenance {
    pub path: String,
    pub line: usize,
    pub column: usize,
    pub start: usize,
    pub end: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize)]
pub struct EffectInspectionDependencies {
    pub state: Vec<String>,
    pub computed: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct EffectInspectionInitialTrigger {
    pub policy: &'static str,
    pub batch_index: u32,
    pub render_boundary: &'static str,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct EffectInspectionActionTrigger {
    pub action_batch_id: String,
    pub matched_states: Vec<String>,
    pub required_computed: Vec<String>,
    pub prerequisite_batches: Vec<EffectInspectionPrerequisiteBatch>,
    pub effect_batch_index: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct EffectInspectionPrerequisiteBatch {
    pub source_batch_index: u32,
    pub computed: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct EffectInspectionSchedule {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub initial_effect_batch_index: Option<u32>,
    pub action_batches: Vec<EffectInspectionScheduledAction>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub unplanned: Option<EffectInspectionUnplanned>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct EffectInspectionScheduledAction {
    pub action_batch_id: String,
    pub effect_batch_index: u32,
    pub prerequisite_computed_batch_refs: Vec<u32>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct EffectInspectionUnplanned {
    pub reason: &'static str,
    pub unavailable_computed_dependencies: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct EffectInspectionCapability {
    pub operation_id: String,
    pub runtime_lowering_id: String,
    pub kind: &'static str,
    pub boundary: &'static str,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct EffectInspectionIr {
    pub function_id: String,
    pub instruction_count: usize,
    pub capability_operation_count: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct EffectInspectionRuntime {
    pub registered: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub function_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub execution_policy: Option<&'static str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub boundary: Option<&'static str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub initial_membership: Option<EffectInspectionInitialTrigger>,
    pub action_batch_ids: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct EffectInspectionResumability {
    pub activation_slot_id: Option<String>,
    pub initial_status: Option<EffectActivationStatus>,
    pub render_boundary: Option<&'static str>,
    pub initial_batch_index: Option<u32>,
    pub action_batch_ids: Vec<String>,
    pub manifest_schema_version: u32,
}

/// Build F17 inspection records solely by projecting canonical F5--F16 products.
#[must_use]
pub fn build_effect_inspection_registry(
    model: &ApplicationSemanticModel,
) -> EffectInspectionRegistry {
    let ir = lower_components_to_ir(model);
    let optimized_ir = optimize_effect_ir(&ir).output;
    let runtime = build_runtime_effect_registry(model, &optimized_ir);
    let resumability = build_effect_resume_plan(model, &runtime);
    let executions = optimized_ir
        .modules
        .iter()
        .flat_map(|module| &module.effect_executions)
        .map(|execution| (execution.effect.clone(), execution))
        .collect::<BTreeMap<_, _>>();
    let functions = optimized_ir
        .modules
        .iter()
        .flat_map(|module| &module.functions)
        .map(|function| (function.id.clone(), function))
        .collect::<BTreeMap<_, _>>();

    EffectInspectionRegistry {
        records: model
            .effects
            .iter()
            .map(|(id, effect)| {
                (
                    id.clone(),
                    inspect_effect(
                        model,
                        effect,
                        runtime.record(id),
                        resumability.record(id),
                        executions.get(id),
                        &functions,
                    ),
                )
            })
            .collect(),
    }
}

/// Verify that an inspection registry remains an exact projection of the
/// canonical F5--F16 products. It does not inspect source or runtime state.
#[must_use]
pub fn validate_effect_inspection_registry(
    model: &ApplicationSemanticModel,
    registry: &EffectInspectionRegistry,
) -> Vec<EffectInspectionValidationDiagnostic> {
    let expected = build_effect_inspection_registry(model);
    if expected == *registry {
        return Vec::new();
    }

    let mut diagnostics = Vec::new();
    for effect in model.effects.keys() {
        match (registry.records.get(effect), expected.records.get(effect)) {
            (None, Some(_)) => diagnostics.push(validation_diagnostic(
                "EZINS1701",
                "canonical effect is missing an inspection record",
            )),
            (Some(actual), Some(expected)) if actual != expected => {
                diagnostics.push(validation_diagnostic(
                    "EZINS1702",
                    "effect inspection record diverges from canonical compiler products",
                ));
            }
            _ => {}
        }
    }
    if registry
        .records
        .keys()
        .any(|effect| !model.effects.contains_key(effect))
    {
        diagnostics.push(validation_diagnostic(
            "EZINS1703",
            "effect inspection record does not resolve to a canonical effect",
        ));
    }
    diagnostics
}

#[allow(clippy::too_many_arguments)]
fn inspect_effect(
    model: &ApplicationSemanticModel,
    effect: &Effect,
    runtime: Option<&RuntimeEffectRecord>,
    resume: Option<&crate::EffectResumeRecord>,
    execution: Option<&&crate::IrEffectExecution>,
    functions: &BTreeMap<SemanticId, &crate::IrFunction>,
) -> EffectInspection {
    let initial_trigger = runtime
        .and_then(|record| record.initial_trigger.as_ref())
        .map(initial_trigger);
    let action_triggers = runtime.map_or_else(Vec::new, |record| {
        record
            .action_batch_triggers
            .iter()
            .map(action_trigger)
            .collect()
    });
    EffectInspection {
        validation: validation(effect),
        direct_dependencies: direct_dependencies(model, &effect.id),
        transitive_dependencies: transitive_dependencies(model, &effect.id),
        dependents: model
            .effect_reactive_analysis(&effect.id)
            .map_or_else(Vec::new, |analysis| semantic_ids(&analysis.dependents)),
        initial_trigger: initial_trigger.clone(),
        action_triggers: action_triggers.clone(),
        schedule: schedule(
            model,
            &effect.id,
            initial_trigger.as_ref(),
            &action_triggers,
        ),
        capabilities: execution.map_or_else(Vec::new, |execution| capabilities(execution)),
        ir: execution.and_then(|execution| ir_inspection(execution, functions)),
        runtime: runtime_inspection(runtime, initial_trigger, &action_triggers),
        resumability: resumability_inspection(resume),
    }
}

fn validation(effect: &Effect) -> EffectInspectionValidation {
    EffectInspectionValidation {
        status: match effect.validation {
            EffectValidation::Valid => "valid",
            EffectValidation::Invalid => "invalid",
            EffectValidation::Unvalidated => "unvalidated",
        },
        violations: effect
            .semantic_violations
            .iter()
            .map(|violation| EffectInspectionViolation {
                category: violation_category(violation.kind),
                statement_id: violation.statement.as_ref().map(ToString::to_string),
                provenance: provenance(&violation.provenance),
            })
            .collect(),
    }
}

fn direct_dependencies(
    model: &ApplicationSemanticModel,
    effect: &SemanticId,
) -> EffectInspectionDependencies {
    let dependencies = model
        .reactive_graph
        .edges
        .iter()
        .filter(|edge| edge.source == effect.as_str() && edge.kind == IrReactiveEdgeKind::Reads)
        .map(|edge| edge.target.clone())
        .collect::<BTreeSet<_>>();
    split_dependencies(model, &dependencies)
}

fn transitive_dependencies(
    model: &ApplicationSemanticModel,
    effect: &SemanticId,
) -> EffectInspectionDependencies {
    let dependencies =
        model
            .effect_reactive_analysis(effect)
            .map_or_else(BTreeSet::new, |analysis| {
                analysis
                    .dependencies
                    .iter()
                    .map(ToString::to_string)
                    .collect()
            });
    split_dependencies(model, &dependencies)
}

fn split_dependencies(
    model: &ApplicationSemanticModel,
    dependencies: &BTreeSet<String>,
) -> EffectInspectionDependencies {
    EffectInspectionDependencies {
        state: dependencies
            .iter()
            .filter(|id| {
                model.components.iter().any(|component| {
                    component
                        .state_fields
                        .iter()
                        .any(|field| field.id.as_str() == *id)
                })
            })
            .cloned()
            .collect(),
        computed: dependencies
            .iter()
            .filter(|id| {
                model
                    .computed_values
                    .keys()
                    .any(|computed| computed.as_str() == *id)
            })
            .cloned()
            .collect(),
    }
}

fn initial_trigger(trigger: &crate::RuntimeInitialEffectTrigger) -> EffectInspectionInitialTrigger {
    EffectInspectionInitialTrigger {
        policy: "after_initial_render",
        batch_index: trigger.effect_batch_index,
        render_boundary: render_boundary(trigger.render_boundary),
    }
}

fn action_trigger(
    trigger: &crate::RuntimeActionBatchEffectTrigger,
) -> EffectInspectionActionTrigger {
    EffectInspectionActionTrigger {
        action_batch_id: trigger.action_batch.to_string(),
        matched_states: semantic_ids(&trigger.matched_states),
        required_computed: semantic_ids(&trigger.required_computed),
        prerequisite_batches: trigger
            .prerequisite_batches
            .iter()
            .map(|batch| EffectInspectionPrerequisiteBatch {
                source_batch_index: batch.source_batch_index,
                computed: semantic_ids(&batch.computed),
            })
            .collect(),
        effect_batch_index: trigger.effect_batch_index,
    }
}

fn schedule(
    model: &ApplicationSemanticModel,
    effect: &SemanticId,
    initial: Option<&EffectInspectionInitialTrigger>,
    actions: &[EffectInspectionActionTrigger],
) -> EffectInspectionSchedule {
    let unplanned = model
        .effect_execution_plan
        .initial
        .unplanned_effects
        .iter()
        .chain(
            model
                .effect_execution_plan
                .actions
                .iter()
                .flat_map(|action| &action.unplanned_effects),
        )
        .find(|unplanned| unplanned.effect == *effect)
        .map(|unplanned| EffectInspectionUnplanned {
            reason: "unavailable_computed_prerequisite",
            unavailable_computed_dependencies: semantic_ids(&unplanned.computed_dependencies),
        });
    EffectInspectionSchedule {
        initial_effect_batch_index: initial.map(|trigger| trigger.batch_index),
        action_batches: actions
            .iter()
            .map(|trigger| EffectInspectionScheduledAction {
                action_batch_id: trigger.action_batch_id.clone(),
                effect_batch_index: trigger.effect_batch_index,
                prerequisite_computed_batch_refs: trigger
                    .prerequisite_batches
                    .iter()
                    .map(|batch| batch.source_batch_index)
                    .collect(),
            })
            .collect(),
        unplanned,
    }
}

fn capabilities(execution: &crate::IrEffectExecution) -> Vec<EffectInspectionCapability> {
    execution
        .capability_operations
        .iter()
        .filter_map(|id| EFFECT_CAPABILITY_REGISTRY.operation(*id))
        .map(|operation| EffectInspectionCapability {
            operation_id: operation.id.0.to_string(),
            runtime_lowering_id: operation.runtime_lowering.0.to_string(),
            kind: match operation.kind {
                CapabilityOperationKind::MemberAssignment => "member_assignment",
                CapabilityOperationKind::MethodCall => "method_call",
            },
            boundary: execution_boundary(operation.boundary),
        })
        .collect()
}

fn ir_inspection(
    execution: &crate::IrEffectExecution,
    functions: &BTreeMap<SemanticId, &crate::IrFunction>,
) -> Option<EffectInspectionIr> {
    let function = functions.get(&execution.function)?;
    Some(EffectInspectionIr {
        function_id: execution.function.to_string(),
        instruction_count: function
            .blocks
            .iter()
            .map(|block| block.instructions.len())
            .sum(),
        capability_operation_count: execution.capability_operations.len(),
    })
}

fn runtime_inspection(
    runtime: Option<&RuntimeEffectRecord>,
    initial: Option<EffectInspectionInitialTrigger>,
    actions: &[EffectInspectionActionTrigger],
) -> EffectInspectionRuntime {
    let Some(runtime) = runtime else {
        return EffectInspectionRuntime {
            registered: false,
            function_id: None,
            execution_policy: None,
            boundary: None,
            initial_membership: None,
            action_batch_ids: Vec::new(),
        };
    };
    EffectInspectionRuntime {
        registered: true,
        function_id: Some(runtime.execution_function.to_string()),
        execution_policy: Some(execution_policy(runtime.initial_trigger_policy)),
        boundary: Some(execution_boundary(runtime.execution_boundary)),
        initial_membership: initial,
        action_batch_ids: actions
            .iter()
            .map(|trigger| trigger.action_batch_id.clone())
            .collect(),
    }
}

fn resumability_inspection(
    resume: Option<&crate::EffectResumeRecord>,
) -> Option<EffectInspectionResumability> {
    let record = resume?;
    Some(EffectInspectionResumability {
        activation_slot_id: record
            .activation_slot
            .as_ref()
            .map(|slot| slot.as_str().to_string()),
        initial_status: record.initial_status,
        render_boundary: record
            .initial_plan_membership
            .as_ref()
            .map(|membership| render_boundary(membership.render_boundary)),
        initial_batch_index: record
            .initial_plan_membership
            .as_ref()
            .map(|membership| membership.batch_index),
        action_batch_ids: semantic_ids(&record.action_batches),
        manifest_schema_version: crate::RESUME_MANIFEST_SCHEMA_VERSION,
    })
}

fn semantic_ids(ids: &[SemanticId]) -> Vec<String> {
    ids.iter().map(ToString::to_string).collect()
}

fn provenance(provenance: &SourceProvenance) -> EffectInspectionProvenance {
    EffectInspectionProvenance {
        path: provenance.path.display().to_string(),
        line: provenance.span.line,
        column: provenance.span.column,
        start: provenance.span.start,
        end: provenance.span.end,
    }
}

fn violation_category(kind: EffectSemanticViolationKind) -> &'static str {
    match kind {
        EffectSemanticViolationKind::Async => "async",
        EffectSemanticViolationKind::ReactiveStateMutation => "reactive_state_mutation",
        EffectSemanticViolationKind::ActionInvocation => "action_invocation",
        EffectSemanticViolationKind::EffectInvocation => "effect_invocation",
        EffectSemanticViolationKind::ComponentMethodInvocation => "component_method_invocation",
        EffectSemanticViolationKind::UnresolvedComponentCall => "unresolved_component_call",
        EffectSemanticViolationKind::UnresolvedComponentAssignment => {
            "unresolved_component_assignment"
        }
        EffectSemanticViolationKind::UnknownExternalCapability => "unknown_external_capability",
        EffectSemanticViolationKind::CapabilitySignature => "capability_signature",
        EffectSemanticViolationKind::CapabilityBoundary => "capability_boundary",
        EffectSemanticViolationKind::CapabilitySerialization => "capability_serialization",
        EffectSemanticViolationKind::ValueReturn => "value_return",
        EffectSemanticViolationKind::UnsupportedStatement => "unsupported_statement",
    }
}

fn execution_policy(policy: EffectExecutionPolicy) -> &'static str {
    match policy {
        EffectExecutionPolicy::AfterInitialRenderAndCompletedActionBatch => {
            "after_initial_render_and_completed_action_batch"
        }
    }
}

fn render_boundary(boundary: EffectRenderBoundary) -> &'static str {
    match boundary {
        EffectRenderBoundary::AfterInitialRender => "after_initial_render",
    }
}

fn execution_boundary(boundary: ExecutionBoundary) -> &'static str {
    match boundary {
        ExecutionBoundary::Client => "client",
        ExecutionBoundary::Server => "server",
    }
}

fn validation_diagnostic(code: &str, message: &str) -> EffectInspectionValidationDiagnostic {
    EffectInspectionValidationDiagnostic {
        code: code.to_string(),
        message: message.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        build_application_semantic_model, build_effect_inspection_registry,
        validate_effect_inspection_registry,
    };

    #[test]
    fn projects_valid_and_invalid_effects_without_creating_runtime_facts() {
        let parsed = ezc_parser::parse_file(
            "src/EffectInspection.tsx",
            r#"
@component("x-effect-inspection")
class EffectInspection extends Component {
  count = state(1);

  @computed()
  get doubled() { return this.count * 2; }

  @action()
  increment() { this.count += 1; }

  @effect()
  report() { console.log(this.doubled); console.info(this.count); }

  @effect()
  invalid() { this.count = 0; }
}
"#,
        );
        let model = build_application_semantic_model(&parsed);
        let registry = build_effect_inspection_registry(&model);
        let component = &model.components[0].id;
        let report = registry
            .records
            .get(&component.effect("report"))
            .expect("valid effect inspection");
        let invalid = registry
            .records
            .get(&component.effect("invalid"))
            .expect("invalid effect inspection");

        assert_eq!(report.validation.status, "valid");
        assert_eq!(report.direct_dependencies.computed.len(), 1);
        assert_eq!(report.direct_dependencies.state.len(), 1);
        assert_eq!(report.dependents, Vec::<String>::new());
        assert_eq!(report.action_triggers.len(), 1);
        assert_eq!(report.capabilities.len(), 2);
        assert!(report.ir.is_some());
        assert!(report.runtime.registered);
        assert_eq!(
            report
                .resumability
                .as_ref()
                .and_then(|record| record.initial_status),
            Some(crate::EffectActivationStatus::Pending)
        );

        assert_eq!(invalid.validation.status, "invalid");
        assert!(!invalid.validation.violations.is_empty());
        assert!(invalid.ir.is_none());
        assert!(!invalid.runtime.registered);
        assert!(invalid.resumability.is_none());
        assert!(validate_effect_inspection_registry(&model, &registry).is_empty());
    }
}
