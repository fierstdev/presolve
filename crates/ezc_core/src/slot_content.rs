use std::collections::BTreeMap;

use crate::{
    AttributeValue, ComponentInvocationEntity, ComponentInvocationId,
    ComponentInvocationResolutionStatus, ElementNode, FragmentNode, SemanticId,
    SlotContentFragmentId, SlotEntity, SlotId, SlotOutletId, SourceProvenance, TemplateChild,
    TemplateNode,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SlotContentFragmentStatus {
    Resolved,
    Blocked,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum SlotContentFragmentViolation {
    UnresolvedInvocationTarget,
    MissingSlotDeclaration,
    DuplicateFragment,
    InvalidNestedWrapper,
    UnsupportedDynamicSlotName,
    InvalidWrapperForm,
}

/// One caller-owned content range supplied to one invocation Slot name.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SlotContentFragment {
    pub id: SlotContentFragmentId,
    pub owner_component: SemanticId,
    pub invocation: ComponentInvocationId,
    pub requested_slot_name: String,
    pub slot: Option<SlotId>,
    pub template_fragment_root: SemanticId,
    pub content_template_entities: Vec<SemanticId>,
    pub status: SlotContentFragmentStatus,
    pub violations: Vec<SlotContentFragmentViolation>,
    pub provenance: SourceProvenance,
    pub secondary_provenances: Vec<SourceProvenance>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SlotOutletStatus {
    Resolved,
    Blocked,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum SlotOutletViolation {
    MissingSlotDeclaration,
    DuplicateOutlet,
    UnsupportedDynamicSlotName,
    InvalidOutletAttributes,
    UnsupportedFallback,
}

/// One callee-authored compile-time placement directive for a declared Slot.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SlotOutlet {
    pub id: SlotOutletId,
    pub owner_component: SemanticId,
    pub slot: Option<SlotId>,
    pub requested_slot_name: String,
    pub template_entity: SemanticId,
    pub status: SlotOutletStatus,
    pub violations: Vec<SlotOutletViolation>,
    pub provenance: SourceProvenance,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct SlotCompositionRegistry {
    pub fragments: BTreeMap<SlotContentFragmentId, SlotContentFragment>,
    pub outlets: BTreeMap<SlotOutletId, SlotOutlet>,
}

#[derive(Debug, Clone)]
struct RawFragment {
    invocation: ComponentInvocationId,
    requested_slot_name: String,
    identity_name: String,
    content_template_entities: Vec<SemanticId>,
    violations: Vec<SlotContentFragmentViolation>,
    provenance: SourceProvenance,
}

/// Lower H3 content and outlet facts from immutable template and invocation products.
#[must_use]
pub fn collect_slot_composition(
    templates: &[TemplateNode],
    invocations: &BTreeMap<ComponentInvocationId, ComponentInvocationEntity>,
    slots: &BTreeMap<SlotId, SlotEntity>,
) -> SlotCompositionRegistry {
    let mut raw_fragments = Vec::new();
    let mut outlets = Vec::new();

    for template in templates {
        if let Some(root) = &template.root {
            collect_element_facts(
                template,
                root,
                "root",
                None,
                false,
                invocations,
                slots,
                &mut raw_fragments,
                &mut outlets,
            );
        }
        if let Some(root) = &template.root_fragment {
            collect_fragment_facts(
                template,
                root,
                "root",
                None,
                false,
                invocations,
                slots,
                &mut raw_fragments,
                &mut outlets,
            );
        }
    }

    let fragments = finalize_fragments(raw_fragments, invocations, slots);
    mark_duplicate_outlets(&mut outlets);
    SlotCompositionRegistry {
        fragments,
        outlets: outlets
            .into_iter()
            .map(|outlet| (outlet.id.clone(), outlet))
            .collect(),
    }
}

#[allow(clippy::too_many_arguments)]
fn collect_element_facts(
    template: &TemplateNode,
    element: &ElementNode,
    path: &str,
    nearest_invocation: Option<&ComponentInvocationEntity>,
    direct_child_of_invocation: bool,
    invocations: &BTreeMap<ComponentInvocationId, ComponentInvocationEntity>,
    slots: &BTreeMap<SlotId, SlotEntity>,
    raw_fragments: &mut Vec<RawFragment>,
    outlets: &mut Vec<SlotOutlet>,
) {
    let entity_id = template.id.template_entity("element", path);
    let invocation_here = invocations
        .values()
        .find(|invocation| invocation.template_entity == entity_id);
    let active_invocation = invocation_here.or(nearest_invocation);

    if let Some(invocation) = invocation_here {
        collect_direct_invocation_content(template, element, path, invocation, raw_fragments);
    } else if element.tag_name == "template"
        && has_attribute(element, "slot")
        && !direct_child_of_invocation
    {
        if let Some(nearest_invocation) = nearest_invocation {
            raw_fragments.push(raw_wrapper_fragment(
                template,
                element,
                path,
                nearest_invocation,
                true,
            ));
        }
    }

    if element.tag_name == "slot" {
        outlets.push(lower_outlet(template, element, path, slots));
    }

    for (index, child) in element.children.iter().enumerate() {
        let child_path = format!("{path}.{index}");
        collect_child_facts(
            template,
            child,
            &child_path,
            active_invocation,
            invocation_here.is_some(),
            invocations,
            slots,
            raw_fragments,
            outlets,
        );
    }
}

#[allow(clippy::too_many_arguments)]
fn collect_fragment_facts(
    template: &TemplateNode,
    fragment: &FragmentNode,
    path: &str,
    nearest_invocation: Option<&ComponentInvocationEntity>,
    direct_child_of_invocation: bool,
    invocations: &BTreeMap<ComponentInvocationId, ComponentInvocationEntity>,
    slots: &BTreeMap<SlotId, SlotEntity>,
    raw_fragments: &mut Vec<RawFragment>,
    outlets: &mut Vec<SlotOutlet>,
) {
    for (index, child) in fragment.children.iter().enumerate() {
        collect_child_facts(
            template,
            child,
            &format!("{path}.{index}"),
            nearest_invocation,
            direct_child_of_invocation,
            invocations,
            slots,
            raw_fragments,
            outlets,
        );
    }
}

#[allow(clippy::too_many_arguments)]
fn collect_child_facts(
    template: &TemplateNode,
    child: &TemplateChild,
    path: &str,
    nearest_invocation: Option<&ComponentInvocationEntity>,
    direct_child_of_invocation: bool,
    invocations: &BTreeMap<ComponentInvocationId, ComponentInvocationEntity>,
    slots: &BTreeMap<SlotId, SlotEntity>,
    raw_fragments: &mut Vec<RawFragment>,
    outlets: &mut Vec<SlotOutlet>,
) {
    match child {
        TemplateChild::Element(element) => collect_element_facts(
            template,
            element,
            path,
            nearest_invocation,
            direct_child_of_invocation,
            invocations,
            slots,
            raw_fragments,
            outlets,
        ),
        TemplateChild::Fragment(fragment) => collect_fragment_facts(
            template,
            fragment,
            path,
            nearest_invocation,
            direct_child_of_invocation,
            invocations,
            slots,
            raw_fragments,
            outlets,
        ),
        TemplateChild::Conditional(conditional) => {
            for (index, child) in conditional.when_true.iter().enumerate() {
                collect_child_facts(
                    template,
                    child,
                    &format!("{path}.true.{index}"),
                    nearest_invocation,
                    false,
                    invocations,
                    slots,
                    raw_fragments,
                    outlets,
                );
            }
            for (index, child) in conditional.when_false.iter().enumerate() {
                collect_child_facts(
                    template,
                    child,
                    &format!("{path}.false.{index}"),
                    nearest_invocation,
                    false,
                    invocations,
                    slots,
                    raw_fragments,
                    outlets,
                );
            }
        }
        TemplateChild::List(list) => {
            for (index, child) in list.item_template.iter().enumerate() {
                collect_child_facts(
                    template,
                    child,
                    &format!("{path}.item.{index}"),
                    nearest_invocation,
                    false,
                    invocations,
                    slots,
                    raw_fragments,
                    outlets,
                );
            }
        }
        TemplateChild::Text { .. } | TemplateChild::Binding { .. } => {}
    }
}

fn collect_direct_invocation_content(
    template: &TemplateNode,
    element: &ElementNode,
    path: &str,
    invocation: &ComponentInvocationEntity,
    raw_fragments: &mut Vec<RawFragment>,
) {
    let mut default_entities = Vec::new();
    let mut default_span = None;

    for (index, child) in element.children.iter().enumerate() {
        let child_path = format!("{path}.{index}");
        let exact_wrapper = match child {
            TemplateChild::Element(element)
                if element.tag_name == "template" && has_attribute(element, "slot") =>
            {
                let raw = raw_wrapper_fragment(template, element, &child_path, invocation, false);
                let is_exact = raw.violations.is_empty();
                raw_fragments.push(raw);
                is_exact
            }
            _ => false,
        };
        if exact_wrapper {
            continue;
        }
        default_entities.push(child_entity_id(&template.id, child, &child_path));
        default_span.get_or_insert_with(|| child_span(child));
    }

    if !default_entities.is_empty() {
        raw_fragments.push(RawFragment {
            invocation: invocation.id.clone(),
            requested_slot_name: "children".to_string(),
            identity_name: "children".to_string(),
            content_template_entities: default_entities,
            violations: Vec::new(),
            provenance: SourceProvenance::new(
                &template.provenance.path,
                default_span.expect("non-empty default content has a span"),
            ),
        });
    }
}

#[allow(clippy::single_match_else)]
fn raw_wrapper_fragment(
    template: &TemplateNode,
    element: &ElementNode,
    path: &str,
    invocation: &ComponentInvocationEntity,
    nested: bool,
) -> RawFragment {
    let slot_attributes = element
        .attributes
        .iter()
        .filter(|attribute| attribute.name == "slot")
        .collect::<Vec<_>>();
    let mut violations = Vec::new();
    let (requested_slot_name, identity_name) = match slot_attributes.as_slice() {
        [attribute] => match &attribute.value {
            AttributeValue::Static(name) if !name.is_empty() => (name.clone(), name.clone()),
            AttributeValue::Binding { expression, .. } => {
                violations.push(SlotContentFragmentViolation::UnsupportedDynamicSlotName);
                (
                    expression.clone(),
                    format!("dynamic:{}", template.id.template_entity("element", path)),
                )
            }
            _ => {
                violations.push(SlotContentFragmentViolation::InvalidWrapperForm);
                (
                    "<invalid>".to_string(),
                    format!("invalid:{}", template.id.template_entity("element", path)),
                )
            }
        },
        _ => {
            violations.push(SlotContentFragmentViolation::InvalidWrapperForm);
            (
                "<invalid>".to_string(),
                format!("invalid:{}", template.id.template_entity("element", path)),
            )
        }
    };
    if element.attributes.len() != 1 || nested {
        violations.push(if nested {
            SlotContentFragmentViolation::InvalidNestedWrapper
        } else {
            SlotContentFragmentViolation::InvalidWrapperForm
        });
    }
    violations.sort_unstable();
    violations.dedup();

    RawFragment {
        invocation: invocation.id.clone(),
        requested_slot_name,
        identity_name,
        content_template_entities: element
            .children
            .iter()
            .enumerate()
            .map(|(index, child)| child_entity_id(&template.id, child, &format!("{path}.{index}")))
            .collect(),
        violations,
        provenance: SourceProvenance::new(&template.provenance.path, element.tag_name_span),
    }
}

fn finalize_fragments(
    raw_fragments: Vec<RawFragment>,
    invocations: &BTreeMap<ComponentInvocationId, ComponentInvocationEntity>,
    slots: &BTreeMap<SlotId, SlotEntity>,
) -> BTreeMap<SlotContentFragmentId, SlotContentFragment> {
    let mut grouped = BTreeMap::<(ComponentInvocationId, String), Vec<RawFragment>>::new();
    for raw in raw_fragments {
        grouped
            .entry((raw.invocation.clone(), raw.identity_name.clone()))
            .or_default()
            .push(raw);
    }

    grouped
        .into_iter()
        .map(|((invocation_id, identity_name), mut candidates)| {
            candidates.sort_by(|left, right| {
                (
                    left.provenance.path.as_path(),
                    left.provenance.span.start,
                    &left.requested_slot_name,
                )
                    .cmp(&(
                        right.provenance.path.as_path(),
                        right.provenance.span.start,
                        &right.requested_slot_name,
                    ))
            });
            let first = candidates.remove(0);
            let invocation = invocations
                .get(&invocation_id)
                .expect("raw fragments originate from canonical invocations");
            let mut violations = first.violations.clone();
            if !candidates.is_empty() {
                violations.push(SlotContentFragmentViolation::DuplicateFragment);
            }
            let slot = invocation.target_component.as_ref().and_then(|target| {
                let id = SlotId::for_component(target, &first.requested_slot_name);
                slots.contains_key(&id).then_some(id)
            });
            if invocation.status != ComponentInvocationResolutionStatus::Resolved {
                violations.push(SlotContentFragmentViolation::UnresolvedInvocationTarget);
            } else if slot.is_none()
                && !violations.contains(&SlotContentFragmentViolation::UnsupportedDynamicSlotName)
                && !violations.contains(&SlotContentFragmentViolation::InvalidWrapperForm)
            {
                violations.push(SlotContentFragmentViolation::MissingSlotDeclaration);
            }
            violations.sort_unstable();
            violations.dedup();
            let id = SlotContentFragmentId::for_invocation(&invocation_id, &identity_name);
            let template_fragment_root = id
                .as_semantic_id()
                .template_entity("fragment-root", "content");
            let fragment = SlotContentFragment {
                id: id.clone(),
                owner_component: invocation.owner_component.clone(),
                invocation: invocation_id,
                requested_slot_name: first.requested_slot_name,
                slot,
                template_fragment_root,
                content_template_entities: first.content_template_entities,
                status: if violations.is_empty() {
                    SlotContentFragmentStatus::Resolved
                } else {
                    SlotContentFragmentStatus::Blocked
                },
                violations,
                provenance: first.provenance,
                secondary_provenances: candidates
                    .into_iter()
                    .map(|candidate| candidate.provenance)
                    .collect(),
            };
            (id, fragment)
        })
        .collect()
}

fn lower_outlet(
    template: &TemplateNode,
    element: &ElementNode,
    path: &str,
    slots: &BTreeMap<SlotId, SlotEntity>,
) -> SlotOutlet {
    let owner_component = template
        .owner
        .entity_id()
        .expect("component templates have component owners")
        .clone();
    let mut violations = Vec::new();
    let (requested_slot_name, identity_name) = match element.attributes.as_slice() {
        [] => ("children".to_string(), "children".to_string()),
        [attribute] if attribute.name == "name" => match &attribute.value {
            AttributeValue::Static(name) if !name.is_empty() => (name.clone(), name.clone()),
            AttributeValue::Binding { expression, .. } => {
                violations.push(SlotOutletViolation::UnsupportedDynamicSlotName);
                (expression.clone(), format!("dynamic:{path}"))
            }
            _ => {
                violations.push(SlotOutletViolation::InvalidOutletAttributes);
                ("<invalid>".to_string(), format!("invalid:{path}"))
            }
        },
        _ => {
            violations.push(SlotOutletViolation::InvalidOutletAttributes);
            ("<invalid>".to_string(), format!("invalid:{path}"))
        }
    };
    if !element.children.is_empty() {
        violations.push(SlotOutletViolation::UnsupportedFallback);
    }
    let slot_id = SlotId::for_component(&owner_component, &requested_slot_name);
    let slot = slots.contains_key(&slot_id).then_some(slot_id);
    if slot.is_none()
        && !violations.contains(&SlotOutletViolation::UnsupportedDynamicSlotName)
        && !violations.contains(&SlotOutletViolation::InvalidOutletAttributes)
    {
        violations.push(SlotOutletViolation::MissingSlotDeclaration);
    }
    violations.sort_unstable();
    violations.dedup();
    let template_entity = template.id.template_entity("element", path);
    let id = SlotOutletId::for_template_entity(&template_entity, &identity_name);

    SlotOutlet {
        id,
        owner_component,
        slot,
        requested_slot_name,
        template_entity,
        status: if violations.is_empty() {
            SlotOutletStatus::Resolved
        } else {
            SlotOutletStatus::Blocked
        },
        violations,
        provenance: SourceProvenance::new(&template.provenance.path, element.tag_name_span),
    }
}

fn mark_duplicate_outlets(outlets: &mut [SlotOutlet]) {
    let mut counts = BTreeMap::<(SemanticId, String), usize>::new();
    for outlet in outlets.iter() {
        *counts
            .entry((
                outlet.owner_component.clone(),
                outlet.requested_slot_name.clone(),
            ))
            .or_default() += 1;
    }
    for outlet in outlets {
        if counts[&(
            outlet.owner_component.clone(),
            outlet.requested_slot_name.clone(),
        )] > 1
        {
            outlet.violations.push(SlotOutletViolation::DuplicateOutlet);
            outlet.violations.sort_unstable();
            outlet.violations.dedup();
            outlet.status = SlotOutletStatus::Blocked;
        }
    }
}

fn has_attribute(element: &ElementNode, name: &str) -> bool {
    element
        .attributes
        .iter()
        .any(|attribute| attribute.name == name)
}

fn child_entity_id(template: &SemanticId, child: &TemplateChild, path: &str) -> SemanticId {
    template.template_entity(
        match child {
            TemplateChild::Text { .. } => "text",
            TemplateChild::Binding { .. } => "binding",
            TemplateChild::Element(_) => "element",
            TemplateChild::Fragment(_) => "fragment",
            TemplateChild::Conditional(_) => "conditional",
            TemplateChild::List(_) => "list",
        },
        path,
    )
}

const fn child_span(child: &TemplateChild) -> ezc_parser::SourceSpan {
    match child {
        TemplateChild::Text { span, .. } | TemplateChild::Binding { span, .. } => *span,
        TemplateChild::Element(element) => element.span,
        TemplateChild::Fragment(fragment) => fragment.span,
        TemplateChild::Conditional(conditional) => conditional.span,
        TemplateChild::List(list) => list.span,
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        build_application_semantic_model, build_semantic_graph,
        validate_application_semantic_model, ComponentInvocationResolutionStatus,
        SemanticEntityKind, SemanticOwner, SlotContentFragmentStatus, SlotContentFragmentViolation,
        SlotKind, SlotOutletStatus, SlotOutletViolation,
    };

    #[test]
    fn lowers_default_and_named_content_with_caller_ownership() {
        let asm = build_application_semantic_model(&ezc_parser::parse_file(
            "src/Composition.tsx",
            r#"
@component("x-card") class Card extends Component {
  @slot() children!: SlotContent;
  @slot() header!: SlotContent;
  render() { return <article><slot name="header" /><slot /></article>; }
}
@component("x-page") class Page extends Component {
  render() {
    return <Card>before<span>body</span><template slot="header"><h1>Title</h1></template>after</Card>;
  }
}
"#,
        ));
        let invocation = asm.component_invocations().into_iter().next().unwrap();
        assert_eq!(
            invocation.status,
            ComponentInvocationResolutionStatus::Resolved
        );
        let fragments = asm.slot_content_fragments_for(&invocation.id);
        assert_eq!(fragments.len(), 2);
        let children = fragments
            .iter()
            .find(|fragment| fragment.requested_slot_name == "children")
            .unwrap();
        let header = fragments
            .iter()
            .find(|fragment| fragment.requested_slot_name == "header")
            .unwrap();
        assert_eq!(children.status, SlotContentFragmentStatus::Resolved);
        assert_eq!(header.status, SlotContentFragmentStatus::Resolved);
        assert_eq!(children.content_template_entities.len(), 3);
        assert_eq!(header.content_template_entities.len(), 1);
        assert_eq!(children.owner_component, invocation.owner_component);
        assert_eq!(header.owner_component, invocation.owner_component);
        assert_eq!(
            asm.owner(&header.content_template_entities[0]),
            Some(&SemanticOwner::entity(
                invocation.owner_component.template()
            ))
        );
        assert_eq!(header.slot.as_ref().unwrap(), &asm.slots()[1].id);
        assert!(fragments.iter().all(|fragment| {
            fragment
                .slot
                .as_ref()
                .and_then(|slot| asm.slot(slot))
                .is_some_and(|slot| matches!(slot.kind, SlotKind::Default | SlotKind::Named))
        }));
        assert_eq!(asm.slot_outlets().len(), 2);
        assert!(asm
            .slot_outlets()
            .iter()
            .all(|outlet| outlet.status == SlotOutletStatus::Resolved));
        assert!(asm.slot_outlets().iter().all(|outlet| {
            asm.owner(outlet.id.as_semantic_id())
                == Some(&SemanticOwner::entity(outlet.owner_component.clone()))
                && asm
                    .entity(outlet.id.as_semantic_id())
                    .is_some_and(|entity| entity.kind() == SemanticEntityKind::SlotOutlet)
        }));
        assert!(
            validate_application_semantic_model(&asm).is_empty(),
            "canonical Slot composition should pass ASM validation"
        );
        let graph = build_semantic_graph(&asm);
        assert_eq!(graph.schema_version, 6);
        assert!(graph.nodes.iter().all(|node| {
            !asm.slot_content_fragments
                .keys()
                .any(|fragment| node.id == *fragment.as_semantic_id())
                && !asm
                    .slot_outlets
                    .keys()
                    .any(|outlet| node.id == *outlet.as_semantic_id())
        }));
    }

    #[test]
    fn retains_empty_missing_dynamic_nested_and_duplicate_fragment_facts() {
        let asm = build_application_semantic_model(&ezc_parser::parse_file(
            "src/InvalidContent.tsx",
            r#"
@component("x-card") class Card extends Component {
  @slot() children!: SlotContent;
  @slot() header!: SlotContent;
  render() { return <article />; }
}
@component("x-page") class Page extends Component {
  render() {
    return <><Card /><Card><template slot="header" /><template slot="header" /></Card><Card><template slot="missing" /></Card><Card><template slot={this.name} /></Card><Card><div><template slot="header" /></div></Card></>;
  }
}
"#,
        ));
        let invocations = asm.component_invocations();
        assert_eq!(invocations.len(), 5);
        assert!(asm
            .slot_content_fragments_for(&invocations[0].id)
            .is_empty());
        let all = asm.slot_content_fragments();
        assert!(all.iter().any(|fragment| fragment
            .violations
            .contains(&SlotContentFragmentViolation::DuplicateFragment)));
        assert!(all.iter().any(|fragment| fragment
            .violations
            .contains(&SlotContentFragmentViolation::MissingSlotDeclaration)));
        assert!(all.iter().any(|fragment| fragment
            .violations
            .contains(&SlotContentFragmentViolation::UnsupportedDynamicSlotName)));
        assert!(all.iter().any(|fragment| fragment
            .violations
            .contains(&SlotContentFragmentViolation::InvalidNestedWrapper)));
        assert!(all
            .iter()
            .filter(|fragment| fragment.requested_slot_name == "header")
            .any(|fragment| !fragment.secondary_provenances.is_empty()));
    }

    #[test]
    fn retains_missing_dynamic_duplicate_and_fallback_outlets() {
        let asm = build_application_semantic_model(&ezc_parser::parse_file(
            "src/InvalidOutlets.tsx",
            r#"
@component("x-panel") class Panel extends Component {
  @slot() children!: SlotContent;
  @slot() title!: SlotContent;
  render() { return <main><slot /><slot /><slot name="missing" /><slot name={this.name} /><slot name="title">fallback</slot></main>; }
}
"#,
        ));
        let outlets = asm.slot_outlets();
        assert_eq!(outlets.len(), 5);
        assert!(outlets
            .iter()
            .filter(|outlet| outlet.requested_slot_name == "children")
            .all(|outlet| outlet
                .violations
                .contains(&SlotOutletViolation::DuplicateOutlet)
                && outlet.status == SlotOutletStatus::Blocked));
        assert!(outlets.iter().any(|outlet| outlet
            .violations
            .contains(&SlotOutletViolation::MissingSlotDeclaration)));
        assert!(outlets.iter().any(|outlet| outlet
            .violations
            .contains(&SlotOutletViolation::UnsupportedDynamicSlotName)));
        assert!(outlets.iter().any(|outlet| outlet
            .violations
            .contains(&SlotOutletViolation::UnsupportedFallback)));
    }

    #[test]
    fn keeps_ordinary_template_elements_as_default_content() {
        let asm = build_application_semantic_model(&ezc_parser::parse_file(
            "src/Templates.tsx",
            r#"
@component("x-card") class Card extends Component {
  @slot() children!: SlotContent;
  render() { return <article><slot /></article>; }
}
@component("x-page") class Page extends Component {
  render() { return <Card><template data-kind="ordinary"><b>Body</b></template></Card>; }
}
"#,
        ));
        let invocation = asm.component_invocations().into_iter().next().unwrap();
        let fragments = asm.slot_content_fragments_for(&invocation.id);

        assert_eq!(fragments.len(), 1);
        assert_eq!(fragments[0].requested_slot_name, "children");
        assert_eq!(fragments[0].status, SlotContentFragmentStatus::Resolved);
        assert!(fragments[0].violations.is_empty());
        assert!(fragments[0].content_template_entities[0]
            .as_str()
            .contains("/element:"));
    }

    #[test]
    fn blocks_content_for_an_unresolved_invocation_without_runtime_lookup() {
        let asm = build_application_semantic_model(&ezc_parser::parse_file(
            "src/Unresolved.tsx",
            r#"
@component("x-page") class Page extends Component {
  render() { return <Missing>Body</Missing>; }
}
"#,
        ));
        let invocation = asm.component_invocations().into_iter().next().unwrap();
        let fragment = asm
            .slot_content_fragments_for(&invocation.id)
            .into_iter()
            .next()
            .unwrap();

        assert_eq!(fragment.status, SlotContentFragmentStatus::Blocked);
        assert!(fragment
            .violations
            .contains(&SlotContentFragmentViolation::UnresolvedInvocationTarget));
        assert!(fragment.slot.is_none());
    }
}
