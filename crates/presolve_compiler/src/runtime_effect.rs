use std::collections::{BTreeMap, BTreeSet};

use crate::{
    ApplicationSemanticModel, CapabilityOperationId, EffectComputedPrerequisiteBatch,
    EffectExecutionBatch, EffectExecutionPolicy, EffectRenderBoundary, ExecutionBoundary,
    IntermediateRepresentation, SemanticId, SourceProvenance,
};

/// Compiler-owned runtime registry for F10-lowered effects.
///
/// The registry is metadata only: it projects existing F8 trigger, F9
/// prerequisite, and F10/F11 IR facts without executing or scheduling effects.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct RuntimeEffectRegistry {
    pub records: BTreeMap<SemanticId, RuntimeEffectRecord>,
}

/// Runtime metadata for one compiler-lowered effect.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeEffectRecord {
    pub effect: SemanticId,
    pub execution_function: SemanticId,
    pub initial_trigger_policy: EffectExecutionPolicy,
    pub initial_trigger: Option<RuntimeInitialEffectTrigger>,
    pub action_batch_triggers: Vec<RuntimeActionBatchEffectTrigger>,
    pub capability_operations: Vec<CapabilityOperationId>,
    pub execution_boundary: ExecutionBoundary,
    pub provenance: SourceProvenance,
}

/// Compiler-owned initial-render trigger metadata for one effect.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeInitialEffectTrigger {
    pub render_boundary: EffectRenderBoundary,
    pub required_computed: Vec<SemanticId>,
    pub prerequisite_batches: Vec<EffectComputedPrerequisiteBatch>,
    pub effect_batch_index: u32,
}

/// Compiler-owned completed-action-batch trigger metadata for one effect.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeActionBatchEffectTrigger {
    pub action_batch: SemanticId,
    pub matched_states: Vec<SemanticId>,
    pub required_computed: Vec<SemanticId>,
    pub prerequisite_batches: Vec<EffectComputedPrerequisiteBatch>,
    pub effect_batch_index: u32,
}

impl RuntimeEffectRegistry {
    #[must_use]
    pub fn record(&self, effect: &SemanticId) -> Option<&RuntimeEffectRecord> {
        self.records.get(effect)
    }
}

/// Build deterministic runtime effect records from canonical F8, F9, and F10/F11 products.
///
/// Registry membership is restricted to `IrEffectExecution` records. This
/// function never walks effect source/body expressions, re-infers dependencies,
/// creates scheduler positions, or invokes runtime capabilities.
#[must_use]
pub fn build_runtime_effect_registry(
    model: &ApplicationSemanticModel,
    ir: &IntermediateRepresentation,
) -> RuntimeEffectRegistry {
    let executions = ir
        .modules
        .iter()
        .flat_map(|module| &module.effect_executions)
        .map(|execution| (execution.effect.clone(), execution))
        .collect::<BTreeMap<_, _>>();
    let records = executions
        .into_iter()
        .filter_map(|(effect_id, execution)| {
            let effect = model.effects.get(&effect_id)?;
            let computed_dependencies = effect_computed_dependencies(model, &effect_id);
            Some((
                effect_id.clone(),
                RuntimeEffectRecord {
                    effect: effect_id.clone(),
                    execution_function: execution.function.clone(),
                    initial_trigger_policy: effect.execution_policy,
                    initial_trigger: initial_trigger(model, &effect_id, &computed_dependencies),
                    action_batch_triggers: action_batch_triggers(
                        model,
                        &effect_id,
                        &computed_dependencies,
                    ),
                    capability_operations: execution.capability_operations.clone(),
                    execution_boundary: effect.execution_boundary,
                    provenance: effect.provenance.clone(),
                },
            ))
        })
        .collect();

    RuntimeEffectRegistry { records }
}

fn effect_computed_dependencies(
    model: &ApplicationSemanticModel,
    effect: &SemanticId,
) -> BTreeSet<SemanticId> {
    model
        .effect_reactive_analysis
        .get(effect)
        .into_iter()
        .flat_map(|analysis| &analysis.dependencies)
        .filter(|dependency| model.computed_values.contains_key(*dependency))
        .cloned()
        .collect()
}

fn initial_trigger(
    model: &ApplicationSemanticModel,
    effect: &SemanticId,
    computed_dependencies: &BTreeSet<SemanticId>,
) -> Option<RuntimeInitialEffectTrigger> {
    let initial = &model.effect_execution_plan.initial;
    let effect_batch_index = effect_batch_index(&initial.effect_batches, effect)?;
    let render_boundary = initial.render_boundary?;
    let required_computed =
        select_required_computed(&initial.required_computed, computed_dependencies);
    Some(RuntimeInitialEffectTrigger {
        render_boundary,
        prerequisite_batches: select_prerequisite_batches(
            &initial.prerequisite_batches,
            &required_computed,
        ),
        required_computed,
        effect_batch_index,
    })
}

fn action_batch_triggers(
    model: &ApplicationSemanticModel,
    effect: &SemanticId,
    computed_dependencies: &BTreeSet<SemanticId>,
) -> Vec<RuntimeActionBatchEffectTrigger> {
    let trigger_evidence = model
        .effect_trigger_plan
        .action_batch_triggers
        .iter()
        .filter_map(|trigger| {
            trigger
                .matched_states
                .get(effect)
                .map(|states| (trigger.action_batch.clone(), states.clone()))
        })
        .collect::<BTreeMap<_, _>>();
    let mut triggers = model
        .effect_execution_plan
        .actions
        .iter()
        .filter_map(|action| {
            let effect_batch_index = effect_batch_index(&action.effect_batches, effect)?;
            let matched_states = trigger_evidence.get(&action.action_batch)?.clone();
            let required_computed =
                select_required_computed(&action.required_computed, computed_dependencies);
            Some(RuntimeActionBatchEffectTrigger {
                action_batch: action.action_batch.clone(),
                matched_states,
                prerequisite_batches: select_prerequisite_batches(
                    &action.prerequisite_batches,
                    &required_computed,
                ),
                required_computed,
                effect_batch_index,
            })
        })
        .collect::<Vec<_>>();
    triggers.sort_by(|left, right| left.action_batch.cmp(&right.action_batch));
    triggers
}

fn effect_batch_index(batches: &[EffectExecutionBatch], effect: &SemanticId) -> Option<u32> {
    batches
        .iter()
        .find(|batch| batch.effects.contains(effect))
        .map(|batch| batch.index)
}

fn select_required_computed(
    scheduled: &[SemanticId],
    dependencies: &BTreeSet<SemanticId>,
) -> Vec<SemanticId> {
    scheduled
        .iter()
        .filter(|computed| dependencies.contains(*computed))
        .cloned()
        .collect()
}

fn select_prerequisite_batches(
    batches: &[EffectComputedPrerequisiteBatch],
    required_computed: &[SemanticId],
) -> Vec<EffectComputedPrerequisiteBatch> {
    let required = required_computed.iter().collect::<BTreeSet<_>>();
    batches
        .iter()
        .filter_map(|batch| {
            let computed = batch
                .computed
                .iter()
                .filter(|computed| required.contains(computed))
                .cloned()
                .collect::<Vec<_>>();
            (!computed.is_empty()).then_some(EffectComputedPrerequisiteBatch {
                source_batch_index: batch.source_batch_index,
                computed,
            })
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use crate::{
        build_application_semantic_model, build_runtime_effect_registry, lower_components_to_ir,
        optimize_effect_ir, CapabilityOperationId, EffectExecutionPolicy, EffectRenderBoundary,
        ExecutionBoundary,
    };

    #[test]
    #[allow(clippy::too_many_lines)]
    fn builds_deterministic_runtime_records_from_effect_plans_and_ir() {
        let parsed = presolve_parser::parse_file(
            "src/RuntimeEffect.tsx",
            r#"
@component("x-runtime-effect")
class RuntimeEffect extends Component {
  price = state(1);
  locale = state("en-US");
  theme = state("light");

  @computed()
  get subtotal() { return this.price * 2; }

  @computed()
  get total() { return this.subtotal + 1; }

  @computed()
  get currentLocale() { return this.locale; }

  @action()
  increment() { this.price += 1; }

  @effect()
  report() { console.log(this.total, this.currentLocale); }

  @effect()
  audit() { console.log(this.price); }

  @effect()
  bootLog() { console.log("ready"); }

  @action()
  invalidAction() { this.price += 1; }

  @effect()
  invalid() { this.invalidAction(); }

  render() { return <p />; }
}
"#,
        );
        let model = build_application_semantic_model(&parsed);
        let component = &model.components[0];
        let price = component.id.state_field("price");
        let subtotal = component.id.computed("subtotal");
        let total = component.id.computed("total");
        let current_locale = component.id.computed("currentLocale");
        let report = component.id.effect("report");
        let audit = component.id.effect("audit");
        let boot_log = component.id.effect("bootLog");
        let invalid = component.id.effect("invalid");
        let increment = component.id.action_batch("increment");
        let registry = build_runtime_effect_registry(
            &model,
            &optimize_effect_ir(&lower_components_to_ir(&model)).output,
        );
        let report_record = registry.record(&report).expect("report record");
        let audit_record = registry.record(&audit).expect("audit record");
        let boot_log_record = registry.record(&boot_log).expect("boot log record");
        let initial = report_record
            .initial_trigger
            .as_ref()
            .expect("initial report trigger");
        let action = report_record
            .action_batch_triggers
            .iter()
            .find(|trigger| trigger.action_batch == increment)
            .expect("increment report trigger");

        assert_eq!(registry.records.len(), 3);
        assert_eq!(
            registry.records.keys().cloned().collect::<Vec<_>>(),
            vec![audit.clone(), boot_log, report.clone()]
        );
        assert!(registry.record(&invalid).is_none());
        assert_eq!(report_record.effect, report);
        assert_eq!(report_record.execution_function, report_record.effect);
        assert_eq!(
            report_record.initial_trigger_policy,
            EffectExecutionPolicy::AfterInitialRenderAndCompletedActionBatch
        );
        assert_eq!(report_record.execution_boundary, ExecutionBoundary::Client);
        assert_eq!(
            initial.render_boundary,
            EffectRenderBoundary::AfterInitialRender
        );
        assert_eq!(
            initial.required_computed,
            vec![current_locale, subtotal.clone(), total.clone()]
        );
        assert_eq!(
            initial
                .prerequisite_batches
                .iter()
                .map(|batch| batch.computed.clone())
                .collect::<Vec<_>>(),
            vec![
                vec![component.id.computed("currentLocale"), subtotal.clone()],
                vec![total.clone()]
            ]
        );
        assert_eq!(initial.effect_batch_index, 0);
        assert_eq!(action.matched_states, vec![price.clone()]);
        assert_eq!(action.required_computed, vec![subtotal, total]);
        assert_eq!(
            action
                .prerequisite_batches
                .iter()
                .map(|batch| batch.computed.clone())
                .collect::<Vec<_>>(),
            vec![
                vec![component.id.computed("subtotal")],
                vec![component.id.computed("total")]
            ]
        );
        assert_eq!(action.effect_batch_index, 0);
        assert_eq!(
            report_record.capability_operations,
            vec![CapabilityOperationId("builtin.browser.console.log")]
        );
        assert_eq!(
            audit_record
                .initial_trigger
                .as_ref()
                .expect("initial audit")
                .required_computed,
            Vec::new()
        );
        assert_eq!(
            audit_record.action_batch_triggers[0].matched_states,
            vec![price]
        );
        assert!(boot_log_record.action_batch_triggers.is_empty());
        assert_eq!(
            boot_log_record
                .initial_trigger
                .as_ref()
                .expect("initial boot log")
                .required_computed,
            Vec::new()
        );
        assert_eq!(
            report_record.provenance,
            model
                .effect(&report_record.effect)
                .expect("effect")
                .provenance
        );
    }
}
