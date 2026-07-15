use std::collections::BTreeMap;

use crate::{
    ComponentInstanceId, ComponentInstancePlan, CompositionCompatibility, CompositionTypeProducts,
    ContextSourceInstanceId, InstanceContextRegistry, IrReactiveEdge, IrReactiveEdgeKind,
    IrReactiveGraph, IrReactiveNode, IrReactiveNodeKind, IrUpdateScheduler, SlotBindingId,
    SlotBindingRegistry,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ComponentInstanceBatch {
    pub index: usize,
    pub instances: Vec<ComponentInstanceId>,
    pub context_sources: Vec<ContextSourceInstanceId>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SlotBindingBatch {
    pub index: usize,
    pub bindings: Vec<SlotBindingId>,
    pub prerequisite_instances: Vec<ComponentInstanceId>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ComponentInitializationPlan {
    pub root_instances: Vec<ComponentInstanceId>,
    pub instance_batches: Vec<ComponentInstanceBatch>,
    pub slot_binding_batches: Vec<SlotBindingBatch>,
    pub blocked_instances: Vec<ComponentInstanceId>,
}

#[must_use]
#[allow(clippy::too_many_lines)]
pub fn plan_component_initialization(
    instances: &ComponentInstancePlan,
    bindings: &SlotBindingRegistry,
    types: &CompositionTypeProducts,
    instance_context: &InstanceContextRegistry,
) -> ComponentInitializationPlan {
    let nodes = instances
        .instances
        .values()
        .map(|instance| {
            (
                instance.id.as_str().to_string(),
                IrReactiveNode {
                    id: instance.id.as_str().to_string(),
                    kind: IrReactiveNodeKind::Template,
                    provenance: instance.provenance.clone(),
                },
            )
        })
        .collect();
    let edges = instances
        .instances
        .values()
        .filter_map(|instance| {
            Some(IrReactiveEdge {
                source: instance.parent_instance.as_ref()?.as_str().to_string(),
                target: instance.id.as_str().to_string(),
                kind: IrReactiveEdgeKind::Invalidates,
                provenance: instance.provenance.clone(),
            })
        })
        .collect();
    let scheduler = IrUpdateScheduler::new(IrReactiveGraph { nodes, edges });
    let instance_batches = scheduler
        .update_batches()
        .into_iter()
        .enumerate()
        .map(|(index, ids)| {
            let instances = ids
                .into_iter()
                .filter_map(|id| {
                    instances
                        .instances
                        .keys()
                        .find(|instance| instance.as_str() == id)
                        .cloned()
                })
                .collect::<Vec<_>>();
            let context_sources = instance_context
                .resolutions
                .values()
                .filter(|resolution| {
                    instances.contains(&resolution.consumer_instance.component_instance)
                })
                .filter(|resolution| {
                    types
                        .instance_context_bindings
                        .get(&resolution.consumer_instance)
                        .is_some_and(|record| {
                            record.overall == CompositionCompatibility::Compatible
                        })
                })
                .filter_map(|resolution| resolution.selected_source.clone())
                .collect();
            ComponentInstanceBatch {
                index,
                instances,
                context_sources,
            }
        })
        .collect::<Vec<_>>();
    let batch_by_instance = instance_batches
        .iter()
        .flat_map(|batch| {
            batch
                .instances
                .iter()
                .cloned()
                .map(move |instance| (instance, batch.index))
        })
        .collect::<BTreeMap<_, _>>();
    let mut slot_batches = BTreeMap::<usize, Vec<SlotBindingId>>::new();
    let mut prerequisites = BTreeMap::<usize, Vec<ComponentInstanceId>>::new();
    for binding in bindings.bindings.values() {
        if types
            .slot_bindings
            .get(&binding.id)
            .is_none_or(|record| record.overall != CompositionCompatibility::Compatible)
            || binding.content_fragment.is_none()
        {
            continue;
        }
        let Some(index) = [
            batch_by_instance.get(&binding.caller_instance),
            batch_by_instance.get(&binding.callee_instance),
        ]
        .into_iter()
        .flatten()
        .max()
        .map(|index| index + 1) else {
            continue;
        };
        slot_batches
            .entry(index)
            .or_default()
            .push(binding.id.clone());
        prerequisites.entry(index).or_default().extend([
            binding.caller_instance.clone(),
            binding.callee_instance.clone(),
        ]);
    }
    let slot_binding_batches = slot_batches
        .into_iter()
        .map(|(index, mut bindings)| {
            bindings.sort();
            let mut prerequisite_instances = prerequisites.remove(&index).unwrap_or_default();
            prerequisite_instances.sort();
            prerequisite_instances.dedup();
            SlotBindingBatch {
                index,
                bindings,
                prerequisite_instances,
            }
        })
        .collect();

    ComponentInitializationPlan {
        root_instances: instances
            .instances
            .values()
            .filter(|instance| instance.parent_instance.is_none())
            .map(|instance| instance.id.clone())
            .collect(),
        instance_batches,
        slot_binding_batches,
        blocked_instances: instances.blocked.keys().cloned().collect(),
    }
}

#[cfg(test)]
mod tests {
    use crate::{build_application_semantic_model, validate_application_semantic_model};

    #[test]
    fn schedules_parents_before_children_and_groups_independent_siblings() {
        let asm = build_application_semantic_model(&ezc_parser::parse_file(
            "src/Initialization.tsx",
            r#"
@component("x-card") class Card extends Component {
  @slot() children!: SlotContent;
  render() { return <article><slot /></article>; }
}
@component("x-page") class Page extends Component {
  render() { return <main><Card><p>one</p></Card><Card><p>two</p></Card></main>; }
}
"#,
        ));
        let plan = &asm.component_initialization;
        assert_eq!(plan.root_instances.len(), 1);
        assert_eq!(plan.instance_batches.len(), 2);
        assert_eq!(plan.instance_batches[0].instances.len(), 1);
        assert_eq!(plan.instance_batches[1].instances.len(), 2);
        assert_eq!(plan.slot_binding_batches.len(), 1);
        assert_eq!(plan.slot_binding_batches[0].bindings.len(), 2);
        assert_eq!(plan.slot_binding_batches[0].prerequisite_instances.len(), 3);
        assert!(plan.blocked_instances.is_empty());
        assert!(validate_application_semantic_model(&asm).is_empty());
    }

    #[test]
    fn excludes_blocked_and_ineligible_bindings_and_keeps_context_ready_with_instance() {
        let asm = build_application_semantic_model(&ezc_parser::parse_file(
            "src/InitializationBlocked.tsx",
            r#"
@component("x-theme") class Theme extends Component { @context() color!: string; render() { return <div />; } }
@component("x-leaf") class Leaf extends Component { @consume(Theme.color) color!: string; render() { return <span />; } }
@component("x-card") class Card extends Component { @provide(Theme.color) color: string = "blue"; render() { return <Leaf />; } }
@component("x-page") class Page extends Component { render() { return <main><Card /><Missing><p /></Missing></main>; } }
"#,
        ));
        let plan = &asm.component_initialization;
        assert_eq!(plan.blocked_instances.len(), 1);
        assert!(plan.slot_binding_batches.is_empty());
        assert!(plan
            .instance_batches
            .iter()
            .any(|batch| !batch.context_sources.is_empty()));
    }

    #[test]
    fn validation_rejects_mutated_initialization_plan() {
        let mut asm = build_application_semantic_model(&ezc_parser::parse_file(
            "src/ValidateInitialization.tsx",
            r#"@component("x-page") class Page extends Component { render() { return <main />; } }"#,
        ));
        asm.component_initialization.root_instances.clear();
        assert!(validate_application_semantic_model(&asm)
            .iter()
            .any(|diagnostic| diagnostic.code == "EZASM1198"));
    }
}
