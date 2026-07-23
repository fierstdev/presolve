use std::collections::{BTreeMap, BTreeSet};

use crate::{
    ComponentInstanceId, ComponentInstancePlan, ComponentInvocationEntity, ComponentInvocationId,
    SemanticId, SlotBindingId, SlotContentFragment, SlotContentFragmentId,
    SlotContentFragmentStatus, SlotContentFragmentViolation, SlotEntity, SlotId, SlotOutlet,
    SlotOutletId, SlotOutletStatus, SlotOutletViolation, SourceProvenance,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum SlotBindingStatus {
    Bound,
    Empty,
    MissingOutlet,
    UnknownSlot,
    DuplicateContent,
    DuplicateOutlet,
    InvalidOwnership,
    BlockedInvocation,
}

/// One compiler-resolved relationship between caller content and callee placement.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SlotBinding {
    pub id: SlotBindingId,
    pub caller_instance: ComponentInstanceId,
    pub callee_instance: ComponentInstanceId,
    pub invocation: ComponentInvocationId,
    pub slot: Option<SlotId>,
    pub outlet: Option<SlotOutletId>,
    pub content_fragment: Option<SlotContentFragmentId>,
    /// Compiler-issued child instance for automatic file-route layout
    /// composition. Authored Slot content continues to use `content_fragment`.
    pub direct_child_instance: Option<ComponentInstanceId>,
    pub content_owner_instance: ComponentInstanceId,
    pub status: SlotBindingStatus,
    pub provenance: SourceProvenance,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct SlotBindingRegistry {
    pub bindings: BTreeMap<SlotBindingId, SlotBinding>,
}

impl SlotBindingRegistry {
    #[must_use]
    pub fn binding(&self, id: &SlotBindingId) -> Option<&SlotBinding> {
        self.bindings.get(id)
    }

    #[must_use]
    pub fn for_callee(&self, callee: &ComponentInstanceId) -> Vec<&SlotBinding> {
        self.bindings
            .values()
            .filter(|binding| &binding.callee_instance == callee)
            .collect()
    }

    #[must_use]
    pub fn for_invocation(&self, invocation: &ComponentInvocationId) -> Vec<&SlotBinding> {
        self.bindings
            .values()
            .filter(|binding| &binding.invocation == invocation)
            .collect()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
enum BindingKey {
    Slot(SlotId),
    Fragment(SlotContentFragmentId),
    Invocation,
}

impl BindingKey {
    fn identity(&self) -> String {
        match self {
            Self::Slot(slot) => format!("slot:{slot}"),
            Self::Fragment(fragment) => format!("fragment:{fragment}"),
            Self::Invocation => "invocation".to_string(),
        }
    }
}

struct BindingInputs<'a> {
    slots: &'a BTreeMap<SlotId, SlotEntity>,
    fragments: &'a BTreeMap<SlotContentFragmentId, SlotContentFragment>,
    outlets: &'a BTreeMap<SlotOutletId, SlotOutlet>,
}

/// Join H3 composition facts to exact H4 caller/callee instance relationships.
#[must_use]
pub fn collect_slot_bindings(
    plan: &ComponentInstancePlan,
    invocations: &BTreeMap<ComponentInvocationId, ComponentInvocationEntity>,
    slots: &BTreeMap<SlotId, SlotEntity>,
    fragments: &BTreeMap<SlotContentFragmentId, SlotContentFragment>,
    outlets: &BTreeMap<SlotOutletId, SlotOutlet>,
) -> SlotBindingRegistry {
    let inputs = BindingInputs {
        slots,
        fragments,
        outlets,
    };
    let mut registry = SlotBindingRegistry::default();

    for instance in plan.instances.values() {
        let (Some(invocation_id), Some(caller_instance)) =
            (&instance.invocation, &instance.parent_instance)
        else {
            continue;
        };
        let Some(invocation) = invocations.get(invocation_id) else {
            continue;
        };
        let caller_component = plan
            .instances
            .get(caller_instance)
            .map(|caller| &caller.component);
        collect_for_instance(
            &mut registry,
            &instance.id,
            caller_instance,
            caller_component,
            Some(&instance.component),
            invocation,
            false,
            &inputs,
        );
    }

    for blocked in plan.blocked.values() {
        let Some(invocation) = invocations.get(&blocked.invocation) else {
            continue;
        };
        let caller_component = plan
            .instances
            .get(&blocked.parent_instance)
            .map(|caller| &caller.component);
        collect_for_instance(
            &mut registry,
            &blocked.id,
            &blocked.parent_instance,
            caller_component,
            blocked.target_component.as_ref(),
            invocation,
            true,
            &inputs,
        );
    }

    registry
}

#[allow(clippy::similar_names, clippy::too_many_arguments)]
fn collect_for_instance(
    registry: &mut SlotBindingRegistry,
    callee_instance: &ComponentInstanceId,
    caller_instance: &ComponentInstanceId,
    caller_component: Option<&SemanticId>,
    callee_component: Option<&SemanticId>,
    invocation: &ComponentInvocationEntity,
    blocked: bool,
    inputs: &BindingInputs<'_>,
) {
    let mut keys = BTreeSet::new();
    if let Some(callee_component) = callee_component {
        keys.extend(
            inputs
                .slots
                .values()
                .filter(|slot| &slot.owner == callee_component)
                .map(|slot| BindingKey::Slot(slot.id.clone())),
        );
    }
    for fragment in inputs
        .fragments
        .values()
        .filter(|fragment| fragment.invocation == invocation.id)
    {
        keys.insert(fragment.slot.as_ref().map_or_else(
            || BindingKey::Fragment(fragment.id.clone()),
            |slot| BindingKey::Slot(slot.clone()),
        ));
    }
    if keys.is_empty() && blocked {
        keys.insert(BindingKey::Invocation);
    }

    for key in keys {
        let slot = match &key {
            BindingKey::Slot(slot) => Some(slot.clone()),
            BindingKey::Fragment(_) | BindingKey::Invocation => None,
        };
        let fragment = fragment_for_key(inputs.fragments, &invocation.id, &key);
        let outlet_candidates = slot.as_ref().map_or_else(Vec::new, |slot| {
            inputs
                .outlets
                .values()
                .filter(|outlet| outlet.slot.as_ref() == Some(slot))
                .collect()
        });
        let outlet = (outlet_candidates.len() == 1
            && outlet_candidates[0].status == SlotOutletStatus::Resolved)
            .then(|| outlet_candidates[0].id.clone());
        let status = classify_binding(
            blocked,
            caller_component,
            callee_component,
            fragment,
            &outlet_candidates,
            outlet.as_ref(),
            slot.as_ref(),
        );
        let id = SlotBindingId::for_instance(callee_instance, &key.identity());
        registry.bindings.insert(
            id.clone(),
            SlotBinding {
                id,
                caller_instance: caller_instance.clone(),
                callee_instance: callee_instance.clone(),
                invocation: invocation.id.clone(),
                slot,
                outlet,
                content_fragment: fragment.map(|fragment| fragment.id.clone()),
                direct_child_instance: None,
                content_owner_instance: caller_instance.clone(),
                status,
                provenance: invocation.provenance.clone(),
            },
        );
    }
}

fn fragment_for_key<'a>(
    fragments: &'a BTreeMap<SlotContentFragmentId, SlotContentFragment>,
    invocation: &ComponentInvocationId,
    key: &BindingKey,
) -> Option<&'a SlotContentFragment> {
    match key {
        BindingKey::Slot(slot) => fragments.values().find(|fragment| {
            &fragment.invocation == invocation && fragment.slot.as_ref() == Some(slot)
        }),
        BindingKey::Fragment(fragment) => fragments.get(fragment),
        BindingKey::Invocation => None,
    }
}

#[allow(clippy::similar_names, clippy::too_many_arguments)]
fn classify_binding(
    blocked: bool,
    caller_component: Option<&SemanticId>,
    callee_component: Option<&SemanticId>,
    fragment: Option<&SlotContentFragment>,
    outlet_candidates: &[&SlotOutlet],
    outlet: Option<&SlotOutletId>,
    slot: Option<&SlotId>,
) -> SlotBindingStatus {
    if blocked {
        return SlotBindingStatus::BlockedInvocation;
    }
    let invalid_fragment_owner = fragment.is_some_and(|fragment| {
        caller_component.is_none_or(|component| &fragment.owner_component != component)
    });
    let invalid_outlet_owner = outlet_candidates.iter().any(|outlet| {
        callee_component.is_none_or(|component| &outlet.owner_component != component)
    });
    if invalid_fragment_owner || invalid_outlet_owner {
        return SlotBindingStatus::InvalidOwnership;
    }
    if fragment.is_some_and(|fragment| {
        fragment
            .violations
            .contains(&SlotContentFragmentViolation::DuplicateFragment)
    }) {
        return SlotBindingStatus::DuplicateContent;
    }
    if slot.is_none()
        || fragment.is_some_and(|fragment| {
            fragment.status == SlotContentFragmentStatus::Blocked
                && !fragment
                    .violations
                    .contains(&SlotContentFragmentViolation::DuplicateFragment)
        })
    {
        return SlotBindingStatus::UnknownSlot;
    }
    if outlet_candidates.len() > 1
        || outlet_candidates.iter().any(|outlet| {
            outlet
                .violations
                .contains(&SlotOutletViolation::DuplicateOutlet)
        })
    {
        return SlotBindingStatus::DuplicateOutlet;
    }
    match (fragment, outlet) {
        (Some(_), Some(_)) => SlotBindingStatus::Bound,
        (Some(_), None) => SlotBindingStatus::MissingOutlet,
        (None, _) => SlotBindingStatus::Empty,
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;

    use crate::{
        build_application_semantic_model, collect_slot_bindings,
        validate_application_semantic_model, SemanticOwner, SlotBindingStatus,
    };

    #[test]
    fn binds_repeated_nested_callees_and_preserves_caller_content_ownership() {
        let asm = build_application_semantic_model(&presolve_parser::parse_file(
            "src/Bindings.tsx",
            r#"
@component("x-card") class Card extends Component {
  @slot() children!: SlotContent;
  @slot() header!: SlotContent;
  render() { return <article><slot name="header" /><slot /></article>; }
}
@component("x-shell") class Shell extends Component {
  title = state("Title");
  render() { return <Card><template slot="header"><h1>{this.title}</h1></template><p>Body</p></Card>; }
}
@component("x-page") class Page extends Component {
  render() { return <main><Shell /><Shell /></main>; }
}
"#,
        ));
        let bound = asm
            .slot_bindings
            .bindings
            .values()
            .filter(|binding| binding.status == SlotBindingStatus::Bound)
            .collect::<Vec<_>>();

        assert_eq!(bound.len(), 4);
        assert_eq!(
            bound
                .iter()
                .map(|binding| &binding.callee_instance)
                .collect::<BTreeSet<_>>()
                .len(),
            2
        );
        assert_eq!(
            bound
                .iter()
                .map(|binding| &binding.caller_instance)
                .collect::<BTreeSet<_>>()
                .len(),
            2
        );
        assert!(bound.iter().all(|binding| {
            binding.content_owner_instance == binding.caller_instance
                && binding.outlet.is_some()
                && binding.content_fragment.is_some()
        }));
        for binding in bound {
            let fragment = asm
                .slot_content_fragment(binding.content_fragment.as_ref().unwrap())
                .unwrap();
            assert!(fragment.content_template_entities.iter().all(|entity| {
                asm.owner(entity).is_some_and(|owner| {
                    matches!(owner, SemanticOwner::Entity(template) if template.as_str().contains("component:x-shell/template:render"))
                })
            }));
        }
        assert!(validate_application_semantic_model(&asm).is_empty());
    }

    #[test]
    fn retains_empty_unknown_missing_and_duplicate_binding_states() {
        let asm = build_application_semantic_model(&presolve_parser::parse_file(
            "src/BindingStates.tsx",
            r#"
@component("x-empty") class Empty extends Component {
  @slot() children!: SlotContent;
  render() { return <div><slot /></div>; }
}
@component("x-missing") class Missing extends Component {
  @slot() header!: SlotContent;
  render() { return <div />; }
}
@component("x-unknown") class Unknown extends Component {
  @slot() children!: SlotContent;
  render() { return <div><slot /></div>; }
}
@component("x-dup-content") class DupContent extends Component {
  @slot() header!: SlotContent;
  render() { return <div><slot name="header" /></div>; }
}
@component("x-dup-outlet") class DupOutlet extends Component {
  @slot() children!: SlotContent;
  render() { return <div><slot /><slot /></div>; }
}
@component("x-page") class Page extends Component {
  render() { return <main><Empty /><Missing><template slot="header"><h1 /></template></Missing><Unknown><template slot="missing"><p /></template></Unknown><DupContent><template slot="header"><h1 /></template><template slot="header"><h2 /></template></DupContent><DupOutlet><p /></DupOutlet></main>; }
}
"#,
        ));
        let statuses = asm
            .slot_bindings
            .bindings
            .values()
            .map(|binding| binding.status)
            .collect::<BTreeSet<_>>();

        assert!(statuses.contains(&SlotBindingStatus::Empty));
        assert!(statuses.contains(&SlotBindingStatus::MissingOutlet));
        assert!(statuses.contains(&SlotBindingStatus::UnknownSlot));
        assert!(statuses.contains(&SlotBindingStatus::DuplicateContent));
        assert!(statuses.contains(&SlotBindingStatus::DuplicateOutlet));
    }

    #[test]
    fn retains_blocked_invocations_and_validates_canonical_ownership() {
        let mut asm = build_application_semantic_model(&presolve_parser::parse_file(
            "src/BlockedBinding.tsx",
            r#"
@component("x-card") class Card extends Component {
  @slot() children!: SlotContent;
  render() { return <slot />; }
}
@component("x-page") class Page extends Component {
  render() { return <main><Missing><p /></Missing><Card><p /></Card></main>; }
}
"#,
        ));
        assert!(asm
            .slot_bindings
            .bindings
            .values()
            .any(|binding| binding.status == SlotBindingStatus::BlockedInvocation));

        let fragment = asm
            .slot_content_fragments
            .values_mut()
            .find(|fragment| fragment.slot.is_some())
            .unwrap();
        fragment.owner_component = crate::SemanticId::component(Some("x-wrong"), "Wrong");
        asm.slot_bindings = collect_slot_bindings(
            &asm.component_instance_plan,
            &asm.component_invocations,
            &asm.slots,
            &asm.slot_content_fragments,
            &asm.slot_outlets,
        );
        assert!(asm
            .slot_bindings
            .bindings
            .values()
            .any(|binding| binding.status == SlotBindingStatus::InvalidOwnership));
        asm.slot_bindings
            .bindings
            .values_mut()
            .find(|binding| binding.status == SlotBindingStatus::InvalidOwnership)
            .unwrap()
            .status = SlotBindingStatus::Bound;
        assert!(validate_application_semantic_model(&asm)
            .iter()
            .any(|diagnostic| diagnostic.code == "PSASM1195"));
    }
}
