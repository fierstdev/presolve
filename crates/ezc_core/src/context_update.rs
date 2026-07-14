use std::collections::BTreeSet;

use crate::{
    ApplicationSemanticModel, ContextSourcePlanEntry, ContextSourcePlanStatus,
    ContextValueSourceId, OptimizedContextIrReport, SemanticId,
};

/// Compiler-owned re-evaluation work for one completed authored action batch.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ContextActionUpdatePlan {
    pub action_batch: SemanticId,
    pub invalidated_sources: Vec<ContextValueSourceId>,
    pub prerequisite_computed_batches: Vec<u32>,
    pub source_evaluation_batches: Vec<crate::ContextEvaluationBatchId>,
    pub affected_consumers: Vec<crate::ConsumerId>,
}

/// Immutable G15 update plan. Empty action records are retained so the runtime
/// never infers that an absent Context update is an accidental omission.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ContextUpdatePlan {
    pub actions: Vec<ContextActionUpdatePlan>,
}

/// Build Context updates solely from G7/G9 dependencies, existing computed
/// transitive facts, authored action-batch identities, and G10 bindings.
#[must_use]
pub fn build_context_update_plan(
    model: &ApplicationSemanticModel,
    optimized: &OptimizedContextIrReport,
) -> ContextUpdatePlan {
    let planned = optimized
        .source_evaluations
        .iter()
        .filter_map(|evaluation| {
            model
                .context_evaluation
                .context_source_plan(&evaluation.source)
                .filter(|entry| entry.status == ContextSourcePlanStatus::Planned)
                .map(|entry| (evaluation, entry))
        })
        .collect::<Vec<_>>();
    let actions = model
        .effect_trigger_plan
        .action_batches
        .values()
        .map(|action| {
            let written = action.written_states.iter().collect::<BTreeSet<_>>();
            let invalidated_sources = planned
                .iter()
                .filter(|(_, entry)| matches!(entry.source, ContextValueSourceId::Provider(_)))
                .filter(|(_, entry)| source_depends_on_written_state(model, entry, &written))
                .map(|(evaluation, _)| evaluation.source.clone())
                .collect::<Vec<_>>();
            let invalidated = invalidated_sources.iter().collect::<BTreeSet<_>>();
            let prerequisite_computed_batches = planned
                .iter()
                .filter(|(evaluation, _)| invalidated.contains(&evaluation.source))
                .flat_map(|(evaluation, _)| {
                    evaluation.prerequisite_computed_batches.iter().copied()
                })
                .collect::<BTreeSet<_>>()
                .into_iter()
                .collect();
            let source_evaluation_batches = model
                .context_evaluation
                .evaluation_batches
                .iter()
                .filter(|batch| {
                    batch
                        .sources
                        .iter()
                        .any(|source| invalidated.contains(source))
                })
                .map(|batch| batch.id.clone())
                .collect();
            let affected_consumers = optimized
                .optimized_module
                .context_ir
                .consumer_bindings
                .iter()
                .filter(|binding| invalidated.contains(&binding.source))
                .map(|binding| binding.consumer.clone())
                .collect();
            ContextActionUpdatePlan {
                action_batch: action.id.clone(),
                invalidated_sources,
                prerequisite_computed_batches,
                source_evaluation_batches,
                affected_consumers,
            }
        })
        .collect();
    ContextUpdatePlan { actions }
}

fn source_depends_on_written_state(
    model: &ApplicationSemanticModel,
    entry: &ContextSourcePlanEntry,
    written: &BTreeSet<&SemanticId>,
) -> bool {
    entry
        .required_state
        .iter()
        .any(|state| written.contains(state))
        || entry.required_computed.iter().any(|computed| {
            model
                .reactive_transitive_analysis
                .dependencies_of(computed.as_str())
                .iter()
                .any(|dependency| written.iter().any(|state| state.as_str() == dependency))
        })
}

#[cfg(test)]
mod tests {
    use crate::{
        build_application_semantic_model, build_context_update_plan, lower_components_to_ir,
        optimize_context_ir, ContextValueSourceId, ProviderId,
    };

    #[test]
    fn plans_each_reactive_provider_once_per_matching_action_batch() {
        let model = build_application_semantic_model(&ezc_parser::parse_file(
            "src/App.tsx",
            r#"
@component("x-app")
class App extends Component {
  count = state(1);
  other = state(0);
  @computed() get doubled(): number { return this.count * 2; }
  @context() total!: number;
  @provide(App.total) providedTotal: number = this.doubled;
  @consume(App.total) total!: number;
  @action() increment() { this.count += 1; this.count += 1; }
  @action() ignore() { this.other += 1; }
  render() { return <main />; }
}
"#,
        ));
        let component = &model.components[0].id;
        let source =
            ContextValueSourceId::Provider(ProviderId::for_component(component, "providedTotal"));
        let plan = build_context_update_plan(
            &model,
            &optimize_context_ir(&lower_components_to_ir(&model)),
        );
        let increment = plan
            .actions
            .iter()
            .find(|action| action.action_batch == component.action_batch("increment"))
            .expect("increment plan");
        let ignore = plan
            .actions
            .iter()
            .find(|action| action.action_batch == component.action_batch("ignore"))
            .expect("ignore plan");
        assert_eq!(increment.invalidated_sources, vec![source]);
        assert!(increment.affected_consumers.len() == 1);
        assert!(ignore.invalidated_sources.is_empty());
    }
}
