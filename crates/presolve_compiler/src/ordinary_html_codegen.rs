//! Compiler-owned J1-P materialization of ordinary template instance markers.

use std::collections::{BTreeMap, BTreeSet};

use crate::template_graph::{
    AttributeValue, ConditionalNode, ElementNode, FragmentNode, ListNode, TemplateAttribute,
};
use crate::{
    build_ordinary_template_instance_registry, build_resume_anchor_plan, ApplicationSemanticModel,
    ComponentInstanceId, ComponentInstanceStatus, ResumeAnchorPlacement, SerializableValue,
    SlotBindingStatus, TemplateChild, TemplateNode, TemplateSemanticKind,
};

#[derive(Debug, Default)]
struct ResumeHtmlMarkers {
    element_anchors: BTreeMap<String, String>,
    text_anchors: BTreeMap<String, String>,
    structural_starts: BTreeMap<String, String>,
    structural_ends: BTreeMap<String, String>,
    events: BTreeMap<String, String>,
}

/// Compiler-rendered branches for one compiler-addressed conditional host.
///
/// These fragments deliberately carry the same target, binding, and exact
/// structural-invocation markers as the ordinary initial renderer. They are
/// artifact data only: runtime materialization remains separately gated.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StructuralConditionalHostFragments {
    pub host_scope: StructuralConditionalHostScope,
    pub host_instance: ComponentInstanceId,
    pub when_true_html: String,
    pub when_false_html: String,
}

/// Compiler-rendered keyed item fragment for one exact component host scope.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StructuralKeyedHostFragment {
    pub host_scope: StructuralConditionalHostScope,
    pub host_instance: ComponentInstanceId,
    pub item_template_html: String,
}

/// The only host scopes the compiler currently renders exactly.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StructuralConditionalHostScope {
    StaticInstance,
    StructuralOccurrence,
}

impl StructuralConditionalHostScope {
    #[must_use]
    pub const fn artifact_text(self) -> &'static str {
        match self {
            Self::StaticInstance => "static-instance",
            Self::StructuralOccurrence => "structural-occurrence",
        }
    }
}

#[derive(Debug, Clone)]
enum SlotProjection<'a> {
    Authored {
        outlet_entity: crate::SemanticId,
        caller_instance: ComponentInstanceId,
        caller_template: &'a TemplateNode,
        invocation_element: &'a ElementNode,
        invocation_path: String,
        content_entities: BTreeSet<crate::SemanticId>,
        caller_slot_projections: &'a [SlotProjection<'a>],
    },
    DirectLayoutChild {
        outlet_entity: crate::SemanticId,
        child_instance: ComponentInstanceId,
    },
}

/// Render the planned initial component topology with compiler-precomputed
/// ordinary instance and J10 resume markers.
#[must_use]
pub fn generate_ordinary_instance_html(model: &ApplicationSemanticModel) -> String {
    generate_ordinary_instance_html_for_roots(model, None)
}

/// Renders only the planned ordinary instance tree rooted at one compiler
/// selected component. Application publication uses this entry-scoped form so
/// unrelated top-level components cannot leak into an entry page.
#[must_use]
pub fn generate_ordinary_instance_html_for_component(
    model: &ApplicationSemanticModel,
    root_component: &crate::SemanticId,
) -> String {
    generate_ordinary_instance_html_for_roots(model, Some(root_component))
}

/// Renders one compiler-planned structural template with the same exact
/// ordinary target and binding markers as an initial instance. The static
/// template-instance prefix is replaced by an opaque runtime-occurrence token;
/// only the later structural materializer may substitute that token.
#[must_use]
pub fn generate_structural_template_instance_html(
    model: &ApplicationSemanticModel,
    template_instance: &ComponentInstanceId,
) -> Option<String> {
    let instance = model
        .component_instance_plan
        .instances
        .get(template_instance)?;
    if instance.status != ComponentInstanceStatus::StructuralTemplate {
        return None;
    }
    let registry = build_ordinary_template_instance_registry(model);
    let targets = registry
        .targets
        .iter()
        .map(|record| {
            (
                (
                    record.component_instance_id.clone(),
                    record.template_entity_id.clone(),
                ),
                record.target_id.to_string(),
            )
        })
        .collect::<BTreeMap<_, _>>();
    let bindings = registry
        .bindings
        .iter()
        .map(|record| {
            (
                (
                    record.component_instance_id.clone(),
                    record.declaration_binding_id.clone(),
                ),
                record.instance_binding_id.to_string(),
            )
        })
        .collect::<BTreeMap<_, _>>();
    let children = model
        .component_instance_plan
        .instances
        .values()
        .filter_map(|child| {
            child.parent_instance.as_ref().and_then(|parent| {
                child
                    .invocation
                    .as_ref()
                    .map(|invocation| ((parent.clone(), invocation.clone()), child))
            })
        })
        .collect::<BTreeMap<_, _>>();
    let templates = model
        .templates
        .iter()
        .map(|template| (template.id.clone(), template))
        .collect::<BTreeMap<_, _>>();
    Some(
        render_instance(
            model,
            &templates,
            &children,
            &targets,
            &bindings,
            &ResumeHtmlMarkers::default(),
            &instance.id,
            &instance.component,
            &[],
        )
        .replace(instance.id.as_str(), "__PRESOLVE_STRUCTURAL_OCCURRENCE__"),
    )
}

/// Render both branches for a compiler-issued conditional structural region.
///
/// Instances with caller-owned Slot projection remain absent: publishing an
/// incomplete fragment would create a second renderer authority. A structural
/// host that has no such projection uses the already-authorized opaque
/// occurrence placeholder rather than a fabricated static instance ID.
#[must_use]
pub fn generate_structural_conditional_host_fragments(
    model: &ApplicationSemanticModel,
    region: &crate::ComponentStructuralRegionId,
) -> Vec<StructuralConditionalHostFragments> {
    let Some(entity) =
        crate::structural_template_entity_for_region(region, &model.template_entities)
    else {
        return Vec::new();
    };
    if entity.kind != TemplateSemanticKind::Conditional {
        return Vec::new();
    }
    let Some(template_id) = entity.owner.entity_id() else {
        return Vec::new();
    };
    let Some(template) = model
        .templates
        .iter()
        .find(|template| template.id == *template_id)
    else {
        return Vec::new();
    };
    let Some(component) = template.owner.entity_id() else {
        return Vec::new();
    };
    let Some((conditional, path)) = conditional_at_span(template, entity.provenance.span) else {
        return Vec::new();
    };

    let registry = build_ordinary_template_instance_registry(model);
    let targets = registry
        .targets
        .iter()
        .map(|record| {
            (
                (
                    record.component_instance_id.clone(),
                    record.template_entity_id.clone(),
                ),
                record.target_id.to_string(),
            )
        })
        .collect::<BTreeMap<_, _>>();
    let bindings = registry
        .bindings
        .iter()
        .map(|record| {
            (
                (
                    record.component_instance_id.clone(),
                    record.declaration_binding_id.clone(),
                ),
                record.instance_binding_id.to_string(),
            )
        })
        .collect::<BTreeMap<_, _>>();
    let children = model
        .component_instance_plan
        .instances
        .values()
        .filter_map(|child| {
            child.parent_instance.as_ref().and_then(|parent| {
                child
                    .invocation
                    .as_ref()
                    .map(|invocation| ((parent.clone(), invocation.clone()), child))
            })
        })
        .collect::<BTreeMap<_, _>>();
    let templates = model
        .templates
        .iter()
        .map(|candidate| (candidate.id.clone(), candidate))
        .collect::<BTreeMap<_, _>>();

    model
        .component_instance_plan
        .instances
        .values()
        .filter(|instance| {
            matches!(
                instance.status,
                ComponentInstanceStatus::Planned | ComponentInstanceStatus::StructuralTemplate
            ) && instance.component == *component
                && model.slot_bindings.for_callee(&instance.id).is_empty()
        })
        .map(|instance| {
            let host_scope = match instance.status {
                ComponentInstanceStatus::Planned => StructuralConditionalHostScope::StaticInstance,
                ComponentInstanceStatus::StructuralTemplate => {
                    StructuralConditionalHostScope::StructuralOccurrence
                }
            };
            let when_true_html = render_children(
                model,
                &templates,
                &children,
                &targets,
                &bindings,
                &ResumeHtmlMarkers::default(),
                &instance.id,
                template,
                &conditional.when_true,
                &format!("{path}.true"),
                &[],
            );
            let when_false_html = render_children(
                model,
                &templates,
                &children,
                &targets,
                &bindings,
                &ResumeHtmlMarkers::default(),
                &instance.id,
                template,
                &conditional.when_false,
                &format!("{path}.false"),
                &[],
            );
            let (when_true_html, when_false_html) = match host_scope {
                StructuralConditionalHostScope::StaticInstance => (when_true_html, when_false_html),
                StructuralConditionalHostScope::StructuralOccurrence => (
                    when_true_html
                        .replace(instance.id.as_str(), "__PRESOLVE_STRUCTURAL_OCCURRENCE__"),
                    when_false_html
                        .replace(instance.id.as_str(), "__PRESOLVE_STRUCTURAL_OCCURRENCE__"),
                ),
            };
            StructuralConditionalHostFragments {
                host_scope,
                host_instance: instance.id.clone(),
                when_true_html,
                when_false_html,
            }
        })
        .collect()
}

/// Render keyed item templates with compiler-issued structural invocation
/// anchors. The generic keyed tokens remain owned by `html_codegen`; this
/// function only joins exact template nodes to planned component invocations.
#[must_use]
pub fn generate_structural_keyed_host_fragments(
    model: &ApplicationSemanticModel,
    region: &crate::ComponentStructuralRegionId,
) -> Vec<StructuralKeyedHostFragment> {
    let Some(entity) =
        crate::structural_template_entity_for_region(region, &model.template_entities)
    else {
        return Vec::new();
    };
    if entity.kind != TemplateSemanticKind::List {
        return Vec::new();
    }
    let Some(template_id) = entity.owner.entity_id() else {
        return Vec::new();
    };
    let Some(template) = model
        .templates
        .iter()
        .find(|template| template.id == *template_id)
    else {
        return Vec::new();
    };
    let Some(component) = template.owner.entity_id() else {
        return Vec::new();
    };
    let Some((list, path)) = list_at_span(template, entity.provenance.span) else {
        return Vec::new();
    };
    let children = model
        .component_instance_plan
        .instances
        .values()
        .filter_map(|child| {
            child.parent_instance.as_ref().and_then(|parent| {
                child
                    .invocation
                    .as_ref()
                    .map(|invocation| ((parent.clone(), invocation.clone()), child))
            })
        })
        .collect::<BTreeMap<_, _>>();

    model
        .component_instance_plan
        .instances
        .values()
        .filter(|instance| {
            matches!(
                instance.status,
                ComponentInstanceStatus::Planned | ComponentInstanceStatus::StructuralTemplate
            ) && instance.component == *component
                && model.slot_bindings.for_callee(&instance.id).is_empty()
        })
        .map(|instance| {
            let host_scope = if instance.status == ComponentInstanceStatus::Planned {
                StructuralConditionalHostScope::StaticInstance
            } else {
                StructuralConditionalHostScope::StructuralOccurrence
            };
            let mut html = crate::html_codegen::generate_list_item_template_html(list);
            for (node, invocation) in structural_list_invocations(
                model,
                &children,
                instance,
                template,
                &list.item_template,
                &format!("{path}.item"),
            ) {
                let marker = format!("data-presolve-node=\"{node}:__ez_list_key__\"");
                let anchored = format!(
                    "{marker} data-presolve-structural-invocation=\"{}\"",
                    escape_attr(&invocation)
                );
                html = html.replacen(&marker, &anchored, 1);
            }
            StructuralKeyedHostFragment {
                host_scope,
                host_instance: instance.id.clone(),
                item_template_html: html,
            }
        })
        .collect()
}

fn structural_list_invocations(
    model: &ApplicationSemanticModel,
    children: &BTreeMap<
        (ComponentInstanceId, crate::ComponentInvocationId),
        &crate::ComponentInstance,
    >,
    instance: &crate::ComponentInstance,
    template: &TemplateNode,
    nodes: &[TemplateChild],
    parent_path: &str,
) -> Vec<(String, String)> {
    fn visit(
        model: &ApplicationSemanticModel,
        children: &BTreeMap<
            (ComponentInstanceId, crate::ComponentInvocationId),
            &crate::ComponentInstance,
        >,
        instance: &crate::ComponentInstance,
        template: &TemplateNode,
        nodes: &[TemplateChild],
        parent_path: &str,
        output: &mut Vec<(String, String)>,
    ) {
        for (index, node) in nodes.iter().enumerate() {
            let path = format!("{parent_path}.{index}");
            match node {
                TemplateChild::Element(element) => {
                    let entity = template.id.template_entity("element", &path);
                    if let Some(invocation) = model
                        .component_invocations
                        .values()
                        .find(|candidate| candidate.template_entity == entity)
                    {
                        if children
                            .get(&(instance.id.clone(), invocation.id.clone()))
                            .is_some_and(|child| {
                                child.status == ComponentInstanceStatus::StructuralTemplate
                            })
                        {
                            output.push((element.id.0.clone(), invocation.id.to_string()));
                        }
                    }
                    visit(
                        model,
                        children,
                        instance,
                        template,
                        &element.children,
                        &path,
                        output,
                    );
                }
                TemplateChild::Fragment(fragment) => {
                    visit(
                        model,
                        children,
                        instance,
                        template,
                        &fragment.children,
                        &path,
                        output,
                    );
                }
                TemplateChild::Conditional(conditional) => {
                    visit(
                        model,
                        children,
                        instance,
                        template,
                        &conditional.when_true,
                        &format!("{path}.true"),
                        output,
                    );
                    visit(
                        model,
                        children,
                        instance,
                        template,
                        &conditional.when_false,
                        &format!("{path}.false"),
                        output,
                    );
                }
                TemplateChild::List(list) => {
                    visit(
                        model,
                        children,
                        instance,
                        template,
                        &list.item_template,
                        &format!("{path}.item"),
                        output,
                    );
                }
                TemplateChild::Text { .. } | TemplateChild::Binding { .. } => {}
            }
        }
    }

    let mut output = Vec::new();
    visit(
        model,
        children,
        instance,
        template,
        nodes,
        parent_path,
        &mut output,
    );
    output
}

fn conditional_at_span(
    template: &TemplateNode,
    span: presolve_parser::SourceSpan,
) -> Option<(&ConditionalNode, String)> {
    fn visit<'a>(
        children: &'a [TemplateChild],
        parent_path: &str,
        span: presolve_parser::SourceSpan,
    ) -> Option<(&'a ConditionalNode, String)> {
        for (index, child) in children.iter().enumerate() {
            let path = format!("{parent_path}.{index}");
            match child {
                TemplateChild::Conditional(conditional) => {
                    if conditional.span == span {
                        return Some((conditional, path));
                    }
                    if let Some(found) =
                        visit(&conditional.when_true, &format!("{path}.true"), span).or_else(|| {
                            visit(&conditional.when_false, &format!("{path}.false"), span)
                        })
                    {
                        return Some(found);
                    }
                }
                TemplateChild::Element(element) => {
                    if let Some(found) = visit(&element.children, &path, span) {
                        return Some(found);
                    }
                }
                TemplateChild::Fragment(fragment) => {
                    if let Some(found) = visit(&fragment.children, &path, span) {
                        return Some(found);
                    }
                }
                TemplateChild::List(list) => {
                    if let Some(found) = visit(&list.item_template, &format!("{path}.item"), span) {
                        return Some(found);
                    }
                }
                TemplateChild::Text { .. } | TemplateChild::Binding { .. } => {}
            }
        }
        None
    }

    template
        .root
        .as_ref()
        .and_then(|root| visit(&root.children, "root", span))
        .or_else(|| {
            template
                .root_fragment
                .as_ref()
                .and_then(|fragment| visit(&fragment.children, "root", span))
        })
}

fn list_at_span(
    template: &TemplateNode,
    span: presolve_parser::SourceSpan,
) -> Option<(&ListNode, String)> {
    fn visit<'a>(
        children: &'a [TemplateChild],
        parent_path: &str,
        span: presolve_parser::SourceSpan,
    ) -> Option<(&'a ListNode, String)> {
        for (index, child) in children.iter().enumerate() {
            let path = format!("{parent_path}.{index}");
            match child {
                TemplateChild::List(list) => {
                    if list.span == span {
                        return Some((list, path));
                    }
                    if let Some(found) = visit(&list.item_template, &format!("{path}.item"), span) {
                        return Some(found);
                    }
                }
                TemplateChild::Element(element) => {
                    if let Some(found) = visit(&element.children, &path, span) {
                        return Some(found);
                    }
                }
                TemplateChild::Fragment(fragment) => {
                    if let Some(found) = visit(&fragment.children, &path, span) {
                        return Some(found);
                    }
                }
                TemplateChild::Conditional(conditional) => {
                    if let Some(found) =
                        visit(&conditional.when_true, &format!("{path}.true"), span).or_else(|| {
                            visit(&conditional.when_false, &format!("{path}.false"), span)
                        })
                    {
                        return Some(found);
                    }
                }
                TemplateChild::Text { .. } | TemplateChild::Binding { .. } => {}
            }
        }
        None
    }

    template
        .root
        .as_ref()
        .and_then(|root| visit(&root.children, "root", span))
        .or_else(|| {
            template
                .root_fragment
                .as_ref()
                .and_then(|fragment| visit(&fragment.children, "root", span))
        })
}

fn generate_ordinary_instance_html_for_roots(
    model: &ApplicationSemanticModel,
    root_component: Option<&crate::SemanticId>,
) -> String {
    let registry = build_ordinary_template_instance_registry(model);
    let resume = build_resume_anchor_plan(model);
    let mut resume_markers = ResumeHtmlMarkers::default();
    for anchor in &resume.anchors {
        let target = anchor.marker_target_id.to_string();
        let index = match anchor.placement {
            ResumeAnchorPlacement::ElementAttribute => &mut resume_markers.element_anchors,
            ResumeAnchorPlacement::TextTemplate => &mut resume_markers.text_anchors,
            ResumeAnchorPlacement::StructuralStartComment => &mut resume_markers.structural_starts,
            ResumeAnchorPlacement::StructuralEndComment => &mut resume_markers.structural_ends,
        };
        index.insert(target, anchor.anchor_id.to_string());
    }
    for event in &resume.events {
        resume_markers.events.insert(
            event.target_id.to_string(),
            event.resume_event_id.to_string(),
        );
    }
    let targets = registry
        .targets
        .iter()
        .map(|record| {
            (
                (
                    record.component_instance_id.clone(),
                    record.template_entity_id.clone(),
                ),
                record.target_id.to_string(),
            )
        })
        .collect::<BTreeMap<_, _>>();
    let bindings = registry
        .bindings
        .iter()
        .map(|record| {
            (
                (
                    record.component_instance_id.clone(),
                    record.declaration_binding_id.clone(),
                ),
                record.instance_binding_id.to_string(),
            )
        })
        .collect::<BTreeMap<_, _>>();
    let children = model
        .component_instance_plan
        .instances
        .values()
        .filter_map(|instance| {
            instance.parent_instance.as_ref().and_then(|parent| {
                instance
                    .invocation
                    .as_ref()
                    .map(|invocation| ((parent.clone(), invocation.clone()), instance))
            })
        })
        .collect::<BTreeMap<_, _>>();
    let templates = model
        .templates
        .iter()
        .map(|template| (template.id.clone(), template))
        .collect::<BTreeMap<_, _>>();
    model
        .component_instance_plan
        .instances
        .values()
        .filter(|instance| {
            instance.parent_instance.is_none()
                && instance.status == ComponentInstanceStatus::Planned
                && root_component.is_none_or(|component| instance.component == *component)
        })
        .map(|instance| {
            render_instance(
                model,
                &templates,
                &children,
                &targets,
                &bindings,
                &resume_markers,
                &instance.id,
                &instance.component,
                &[],
            )
        })
        .collect::<Vec<_>>()
        .join("\n")
}

#[allow(clippy::too_many_arguments)]
fn render_instance(
    model: &ApplicationSemanticModel,
    templates: &BTreeMap<crate::SemanticId, &TemplateNode>,
    children: &BTreeMap<
        (ComponentInstanceId, crate::ComponentInvocationId),
        &crate::ComponentInstance,
    >,
    targets: &BTreeMap<(ComponentInstanceId, crate::SemanticId), String>,
    bindings: &BTreeMap<(ComponentInstanceId, crate::SemanticId), String>,
    resume_markers: &ResumeHtmlMarkers,
    instance: &ComponentInstanceId,
    component: &crate::SemanticId,
    slot_projections: &[SlotProjection<'_>],
) -> String {
    let mut instance_slot_projections = slot_projections.to_vec();
    instance_slot_projections.extend(direct_layout_slot_projections(model, instance));
    let Some(template) = templates.get(&component.template()) else {
        return String::new();
    };
    if let Some(root) = &template.root {
        return render_element(
            model,
            templates,
            children,
            targets,
            bindings,
            resume_markers,
            instance,
            template,
            root,
            "root",
            &instance_slot_projections,
        );
    }
    template
        .root_fragment
        .as_ref()
        .map_or_else(String::new, |fragment| {
            render_fragment(
                model,
                templates,
                children,
                targets,
                bindings,
                resume_markers,
                instance,
                template,
                fragment,
                "root",
                &instance_slot_projections,
            )
        })
}

#[allow(clippy::too_many_arguments)]
fn render_fragment(
    model: &ApplicationSemanticModel,
    templates: &BTreeMap<crate::SemanticId, &TemplateNode>,
    children: &BTreeMap<
        (ComponentInstanceId, crate::ComponentInvocationId),
        &crate::ComponentInstance,
    >,
    targets: &BTreeMap<(ComponentInstanceId, crate::SemanticId), String>,
    bindings: &BTreeMap<(ComponentInstanceId, crate::SemanticId), String>,
    resume_markers: &ResumeHtmlMarkers,
    instance: &ComponentInstanceId,
    template: &TemplateNode,
    fragment: &FragmentNode,
    path: &str,
    slot_projections: &[SlotProjection<'_>],
) -> String {
    render_children(
        model,
        templates,
        children,
        targets,
        bindings,
        resume_markers,
        instance,
        template,
        &fragment.children,
        path,
        slot_projections,
    )
}

#[allow(clippy::too_many_arguments)]
fn render_children(
    model: &ApplicationSemanticModel,
    templates: &BTreeMap<crate::SemanticId, &TemplateNode>,
    children: &BTreeMap<
        (ComponentInstanceId, crate::ComponentInvocationId),
        &crate::ComponentInstance,
    >,
    targets: &BTreeMap<(ComponentInstanceId, crate::SemanticId), String>,
    bindings: &BTreeMap<(ComponentInstanceId, crate::SemanticId), String>,
    resume_markers: &ResumeHtmlMarkers,
    instance: &ComponentInstanceId,
    template: &TemplateNode,
    nodes: &[TemplateChild],
    parent_path: &str,
    slot_projections: &[SlotProjection<'_>],
) -> String {
    nodes
        .iter()
        .enumerate()
        .map(|(index, node)| {
            render_child(
                model,
                templates,
                children,
                targets,
                bindings,
                resume_markers,
                instance,
                template,
                node,
                &format!("{parent_path}.{index}"),
                slot_projections,
            )
        })
        .collect()
}

#[allow(clippy::too_many_arguments, clippy::too_many_lines)]
fn render_child(
    model: &ApplicationSemanticModel,
    templates: &BTreeMap<crate::SemanticId, &TemplateNode>,
    children: &BTreeMap<
        (ComponentInstanceId, crate::ComponentInvocationId),
        &crate::ComponentInstance,
    >,
    targets: &BTreeMap<(ComponentInstanceId, crate::SemanticId), String>,
    bindings: &BTreeMap<(ComponentInstanceId, crate::SemanticId), String>,
    resume_markers: &ResumeHtmlMarkers,
    instance: &ComponentInstanceId,
    template: &TemplateNode,
    node: &TemplateChild,
    path: &str,
    slot_projections: &[SlotProjection<'_>],
) -> String {
    match node {
        TemplateChild::Text { value, .. } => escape_text(value),
        TemplateChild::Binding {
            expression,
            initial_value,
            ..
        } => {
            let entity = template.id.template_entity("binding", path);
            let value = initial_value
                .as_ref()
                .map_or_else(String::new, SerializableValue::render_text);
            let marker_target = targets.get(&(instance.clone(), entity.clone()));
            let resume_marker = marker_target
                .and_then(|target| resume_markers.text_anchors.get(target))
                .map_or_else(String::new, |anchor| {
                    format!(
                        "<template data-presolve-r=\"{}\"></template>",
                        escape_attr(anchor)
                    )
                });
            bindings.get(&(instance.clone(), entity)).map_or_else(
                || format!("{resume_marker}<!-- presolve-binding:{expression} -->{value}"),
                |id| format!("{resume_marker}<!--presolve-ti-binding-start:{id}-->{value}<!--presolve-ti-binding-end:{id}-->"),
            )
        }
        TemplateChild::Element(element) => render_element(
            model,
            templates,
            children,
            targets,
            bindings,
            resume_markers,
            instance,
            template,
            element,
            path,
            slot_projections,
        ),
        TemplateChild::Fragment(fragment) => render_fragment(
            model,
            templates,
            children,
            targets,
            bindings,
            resume_markers,
            instance,
            template,
            fragment,
            path,
            slot_projections,
        ),
        TemplateChild::Conditional(conditional) => {
            let entity = template.id.template_entity("conditional", path);
            let marker = targets
                .get(&(instance.clone(), entity))
                .map_or("", String::as_str);
            let resume_start = resume_markers
                .structural_starts
                .get(marker)
                .map_or("", String::as_str);
            let resume_end = resume_markers
                .structural_ends
                .get(marker)
                .map_or("", String::as_str);
            let selected_true = matches!(
                conditional.initial_value,
                Some(SerializableValue::Boolean(true))
            );
            let selected = if selected_true {
                &conditional.when_true
            } else {
                &conditional.when_false
            };
            let selected_path = format!("{path}.{}", if selected_true { "true" } else { "false" });
            format!(
                "<!--presolve-r-start:{}--><!--presolve-conditional-start:{}:ti:{}-->{}<!--presolve-conditional-end:{}:ti:{}--><!--presolve-r-end:{}-->",
                escape_comment(resume_start),
                conditional.start_id.0,
                marker,
                render_children(
                    model,
                    templates,
                    children,
                    targets,
                    bindings,
                    resume_markers,
                    instance,
                    template,
                    selected,
                    &selected_path,
                    slot_projections,
                ),
                conditional.end_id.0,
                marker,
                escape_comment(resume_end)
            )
        }
        TemplateChild::List(list) => {
            let entity = template.id.template_entity("list", path);
            let marker = targets
                .get(&(instance.clone(), entity))
                .map_or("", String::as_str);
            let resume_start = resume_markers
                .structural_starts
                .get(marker)
                .map_or("", String::as_str);
            let resume_end = resume_markers
                .structural_ends
                .get(marker)
                .map_or("", String::as_str);
            format!(
                "<!--presolve-r-start:{}--><!--presolve-ti-target-start:{marker}-->{}<!--presolve-ti-target-end:{marker}--><!--presolve-r-end:{}-->",
                escape_comment(resume_start),
                crate::html_codegen::generate_list_html(list),
                escape_comment(resume_end),
            )
        }
    }
}

#[allow(clippy::too_many_arguments, clippy::too_many_lines)]
fn render_element(
    model: &ApplicationSemanticModel,
    templates: &BTreeMap<crate::SemanticId, &TemplateNode>,
    children: &BTreeMap<
        (ComponentInstanceId, crate::ComponentInvocationId),
        &crate::ComponentInstance,
    >,
    targets: &BTreeMap<(ComponentInstanceId, crate::SemanticId), String>,
    bindings: &BTreeMap<(ComponentInstanceId, crate::SemanticId), String>,
    resume_markers: &ResumeHtmlMarkers,
    instance: &ComponentInstanceId,
    template: &TemplateNode,
    element: &ElementNode,
    path: &str,
    slot_projections: &[SlotProjection<'_>],
) -> String {
    let entity = template.id.template_entity("element", path);
    if element.tag_name == "slot" {
        if let Some(projection) = slot_projections
            .iter()
            .find(|projection| projection.outlet_entity() == &entity)
        {
            return render_slot_projection(
                model,
                templates,
                children,
                targets,
                bindings,
                resume_markers,
                projection,
            );
        }
        if let Some(outlet) = model
            .slot_outlets
            .values()
            .find(|outlet| outlet.template_entity == entity)
        {
            let empty = model
                .slot_bindings
                .for_callee(instance)
                .into_iter()
                .any(|binding| {
                    binding.outlet.as_ref() == Some(&outlet.id)
                        && binding.status == SlotBindingStatus::Empty
                });
            if empty {
                return String::new();
            }
        }
    }
    if let Some(invocation) = model
        .component_invocations
        .values()
        .find(|candidate| candidate.template_entity == entity)
    {
        if let Some(child) = children.get(&(instance.clone(), invocation.id.clone())) {
            if child.status == ComponentInstanceStatus::Planned {
                let projections = slot_projections_for_invocation(
                    model,
                    &child.id,
                    instance,
                    template,
                    element,
                    path,
                    slot_projections,
                );
                return render_instance(
                    model,
                    templates,
                    children,
                    targets,
                    bindings,
                    resume_markers,
                    &child.id,
                    &child.component,
                    &projections,
                );
            }
        }
    }
    let structural_invocation = model
        .component_invocations
        .values()
        .find(|candidate| candidate.template_entity == entity)
        .and_then(|invocation| {
            children
                .get(&(instance.clone(), invocation.id.clone()))
                .filter(|child| child.status == ComponentInstanceStatus::StructuralTemplate)
                .map(|_| invocation.id.as_str())
        });
    let mut html = format!(
        "<{} data-presolve-node=\"{}\"",
        element.tag_name,
        escape_attr(&element.id.0)
    );
    if let Some(target) = targets.get(&(instance.clone(), entity)) {
        html.push_str(" data-presolve-ti=\"");
        html.push_str(&escape_attr(target));
        html.push('"');
        if let Some(anchor) = resume_markers.element_anchors.get(target) {
            html.push_str(" data-presolve-r=\"");
            html.push_str(&escape_attr(anchor));
            html.push('"');
        }
        if let Some(event) = resume_markers.events.get(target) {
            html.push_str(" data-presolve-e=\"");
            html.push_str(&escape_attr(event));
            html.push('"');
        }
    }
    if let Some(invocation) = structural_invocation {
        html.push_str(" data-presolve-structural-invocation=\"");
        html.push_str(&escape_attr(invocation));
        html.push('"');
    }
    for attribute in &element.attributes {
        html.push(' ');
        html.push_str(&attribute_html(attribute));
    }
    html.push('>');
    html.push_str(&render_children(
        model,
        templates,
        children,
        targets,
        bindings,
        resume_markers,
        instance,
        template,
        &element.children,
        path,
        slot_projections,
    ));
    html.push_str("</");
    html.push_str(&element.tag_name);
    html.push('>');
    html
}

#[allow(clippy::similar_names, clippy::too_many_arguments)]
fn slot_projections_for_invocation<'a>(
    model: &'a ApplicationSemanticModel,
    callee_instance: &ComponentInstanceId,
    caller_instance: &ComponentInstanceId,
    caller_template: &'a TemplateNode,
    invocation_element: &'a ElementNode,
    invocation_path: &str,
    caller_slot_projections: &'a [SlotProjection<'a>],
) -> Vec<SlotProjection<'a>> {
    model
        .slot_bindings
        .for_callee(callee_instance)
        .into_iter()
        .filter(|binding| binding.status == SlotBindingStatus::Bound)
        .filter_map(|binding| {
            let outlet = model.slot_outlets.get(binding.outlet.as_ref()?)?;
            let fragment = model
                .slot_content_fragments
                .get(binding.content_fragment.as_ref()?)?;
            Some(SlotProjection::Authored {
                outlet_entity: outlet.template_entity.clone(),
                caller_instance: caller_instance.clone(),
                caller_template,
                invocation_element,
                invocation_path: invocation_path.to_string(),
                content_entities: fragment.content_template_entities.iter().cloned().collect(),
                caller_slot_projections,
            })
        })
        .collect()
}

impl SlotProjection<'_> {
    fn outlet_entity(&self) -> &crate::SemanticId {
        match self {
            Self::Authored { outlet_entity, .. }
            | Self::DirectLayoutChild { outlet_entity, .. } => outlet_entity,
        }
    }
}

fn direct_layout_slot_projections(
    model: &ApplicationSemanticModel,
    instance: &ComponentInstanceId,
) -> Vec<SlotProjection<'static>> {
    model
        .slot_bindings
        .for_callee(instance)
        .into_iter()
        .filter(|binding| binding.status == SlotBindingStatus::Bound)
        .filter_map(|binding| {
            let child_instance = binding.direct_child_instance.clone()?;
            let outlet = model.slot_outlets.get(binding.outlet.as_ref()?)?;
            Some(SlotProjection::DirectLayoutChild {
                outlet_entity: outlet.template_entity.clone(),
                child_instance,
            })
        })
        .collect()
}

#[allow(clippy::too_many_arguments)]
fn render_slot_projection(
    model: &ApplicationSemanticModel,
    templates: &BTreeMap<crate::SemanticId, &TemplateNode>,
    children: &BTreeMap<
        (ComponentInstanceId, crate::ComponentInvocationId),
        &crate::ComponentInstance,
    >,
    targets: &BTreeMap<(ComponentInstanceId, crate::SemanticId), String>,
    bindings: &BTreeMap<(ComponentInstanceId, crate::SemanticId), String>,
    resume_markers: &ResumeHtmlMarkers,
    projection: &SlotProjection<'_>,
) -> String {
    if let SlotProjection::DirectLayoutChild { child_instance, .. } = projection {
        let Some(child) = model.component_instance_plan.instances.get(child_instance) else {
            return String::new();
        };
        return render_instance(
            model,
            templates,
            children,
            targets,
            bindings,
            resume_markers,
            &child.id,
            &child.component,
            &[],
        );
    }
    let SlotProjection::Authored {
        caller_instance,
        caller_template,
        invocation_element,
        invocation_path,
        content_entities,
        caller_slot_projections,
        ..
    } = projection
    else {
        unreachable!("direct layout Slot projection returns above");
    };
    let mut html = String::new();
    for (index, child) in invocation_element.children.iter().enumerate() {
        let path = format!("{invocation_path}.{index}");
        let entity = template_child_entity(&caller_template.id, child, &path);
        if content_entities.contains(&entity) {
            html.push_str(&render_child(
                model,
                templates,
                children,
                targets,
                bindings,
                resume_markers,
                caller_instance,
                caller_template,
                child,
                &path,
                caller_slot_projections,
            ));
            continue;
        }
        let TemplateChild::Element(wrapper) = child else {
            continue;
        };
        if wrapper.tag_name != "template" {
            continue;
        }
        for (child_index, wrapped) in wrapper.children.iter().enumerate() {
            let wrapped_path = format!("{path}.{child_index}");
            let wrapped_entity = template_child_entity(&caller_template.id, wrapped, &wrapped_path);
            if content_entities.contains(&wrapped_entity) {
                html.push_str(&render_child(
                    model,
                    templates,
                    children,
                    targets,
                    bindings,
                    resume_markers,
                    caller_instance,
                    caller_template,
                    wrapped,
                    &wrapped_path,
                    caller_slot_projections,
                ));
            }
        }
    }
    html
}

fn template_child_entity(
    template: &crate::SemanticId,
    child: &TemplateChild,
    path: &str,
) -> crate::SemanticId {
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

fn attribute_html(attribute: &TemplateAttribute) -> String {
    let value = match &attribute.value {
        AttributeValue::Boolean => return attribute.name.clone(),
        AttributeValue::Static(value) => value.clone(),
        AttributeValue::Binding { initial_value, .. } => initial_value
            .as_ref()
            .map_or_else(String::new, SerializableValue::render_text),
        AttributeValue::EventHandler { handler, .. } => handler.clone(),
        AttributeValue::BindingList(values) => values.join(","),
    };
    format!("{}=\"{}\"", attribute.name, escape_attr(&value))
}

fn escape_attr(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('"', "&quot;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}

fn escape_comment(value: &str) -> String {
    value.replace("--", "&#45;&#45;")
}
fn escape_text(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}

#[cfg(test)]
mod tests {
    use super::{
        generate_ordinary_instance_html, generate_ordinary_instance_html_for_component,
        generate_structural_conditional_host_fragments, generate_structural_keyed_host_fragments,
        StructuralConditionalHostScope,
    };
    use crate::{
        build_application_semantic_model, build_resume_anchor_plan, validate_resume_marker_html,
    };

    #[test]
    fn emits_precomputed_repeated_instance_and_exact_resume_markers() {
        let model = build_application_semantic_model(&presolve_parser::parse_file(
            "src/Markers.tsx",
            r#"
@component("x-child") class Child {
  count = state(0);
  @action() increment() { this.count++; }
  render() { return <button onClick={() => this.increment()}>{this.count}</button>; }
}
@component("x-parent") class Parent { render() { return <><Child /><Child /></>; } }
"#,
        ));
        let html = generate_ordinary_instance_html(&model);
        assert_eq!(html.matches("data-presolve-ti=").count(), 2);
        assert_eq!(html.matches("presolve-ti-binding-start:").count(), 2);
        assert_eq!(html.matches("data-presolve-r=").count(), 4);
        assert_eq!(html.matches("data-presolve-e=").count(), 2);
        assert_eq!(html, generate_ordinary_instance_html(&model));
    }

    #[test]
    fn places_caller_owned_slot_content_at_the_exact_compiler_outlet() {
        let model = build_application_semantic_model(&presolve_parser::parse_file(
            "src/Slots.tsx",
            r#"
@component("x-leaf") class Leaf {
  count = state(0);
  @action() increment() { this.count++; }
  render() { return <button onClick={() => this.increment()}>{this.count}</button>; }
}
@component("x-card") class Card {
  @slot() children!: SlotContent;
  render() { return <article><slot /><Leaf /></article>; }
}
@component("x-page") class Page { render() { return <main><Card><Leaf /></Card></main>; } }
"#,
        ));
        let html = generate_ordinary_instance_html(&model);
        let resume = build_resume_anchor_plan(&model);

        assert_eq!(html.matches("<button").count(), 2);
        assert!(!html.contains("<slot"));
        assert_eq!(html.matches("presolve-ti-binding-start:").count(), 2);
        assert!(
            validate_resume_marker_html(&resume, &html).is_empty(),
            "caller-owned Slot content must carry every required resume marker"
        );
    }

    #[test]
    fn renders_only_the_selected_entry_component_tree() {
        let model = build_application_semantic_model(&presolve_parser::parse_file(
            "src/Entries.tsx",
            r#"
@component("x-home") class Home { render() { return <main>Home</main>; } }
@component("x-about") class About { render() { return <main>About</main>; } }
"#,
        ));
        let home = model
            .components
            .iter()
            .find(|component| component.class_name == "Home")
            .unwrap();

        let html = generate_ordinary_instance_html_for_component(&model, &home.id);

        assert!(html.contains("Home"));
        assert!(!html.contains("About"));
    }

    #[test]
    fn preserves_conditional_template_paths_for_structural_invocations() {
        let model = build_application_semantic_model(&presolve_parser::parse_file(
            "src/StructuralMarker.tsx",
            r#"
@component("x-leaf") class Leaf { count = state(0); render() { return <strong>{this.count}</strong>; } }
@component("x-page") class Page {
  visible = state(true);
  render() { return <main>{this.visible ? <Leaf /> : <span>Hidden</span>}</main>; }
}
"#,
        ));
        let html = generate_ordinary_instance_html(&model);
        assert!(html.contains("data-presolve-node"));
        assert!(!html.contains("componentByTag"));
        assert!(html.contains("data-presolve-structural-invocation="));
    }

    #[test]
    fn renders_both_compiler_addressed_conditional_host_fragments() {
        let model = build_application_semantic_model(&presolve_parser::parse_file(
            "src/StructuralHostFragments.tsx",
            r#"
@component("x-leaf") class Leaf { count = state(0); render() { return <strong>{this.count}</strong>; } }
@component("x-page") class Page {
  visible = state(true);
  render() { return <main>{this.visible ? <Leaf /> : <span>Hidden</span>}</main>; }
}
"#,
        ));
        let region = model
            .component_instance_plan
            .instances
            .values()
            .find_map(|instance| instance.structural_region.as_ref())
            .expect("conditional leaf is a structural template");

        let fragments = generate_structural_conditional_host_fragments(&model, region);

        assert_eq!(fragments.len(), 1);
        assert!(fragments[0]
            .when_true_html
            .contains("data-presolve-structural-invocation="));
        assert!(fragments[0].when_false_html.contains("Hidden"));
        assert!(fragments[0].when_true_html.contains("data-presolve-node"));
    }

    #[test]
    fn renders_nested_conditional_hosts_against_the_opaque_parent_occurrence() {
        let model = build_application_semantic_model(&presolve_parser::parse_file(
            "src/NestedStructuralHostFragments.tsx",
            r#"
@component("x-leaf") class Leaf { render() { return <small />; } }
@component("x-card") class Card {
  expanded = state(true);
  render() { return <article>{this.expanded ? <section><Leaf />{this.expanded}</section> : <em>Collapsed</em>}</article>; }
}
@component("x-page") class Page {
  shown = state(true);
  render() { return <main>{this.shown ? <Card /> : <span>Hidden</span>}</main>; }
}
"#,
        ));

        let fragments = model
            .component_instance_plan
            .instances
            .values()
            .filter_map(|instance| instance.structural_region.clone())
            .flat_map(|region| generate_structural_conditional_host_fragments(&model, &region))
            .find(|fragments| {
                fragments.host_scope == StructuralConditionalHostScope::StructuralOccurrence
            })
            .expect("nested structural host fragments");

        assert!(fragments
            .when_true_html
            .contains("__PRESOLVE_STRUCTURAL_OCCURRENCE__"));
        assert!(fragments
            .when_true_html
            .contains("data-presolve-structural-invocation="));
        assert!(fragments.when_false_html.contains("Collapsed"));
    }

    #[test]
    fn renders_keyed_component_invocations_with_compiler_anchors() {
        let model = build_application_semantic_model(&presolve_parser::parse_file(
            "src/KeyedStructuralHostFragments.tsx",
            r#"
@component("x-leaf") class Leaf { render() { return <small />; } }
@component("x-page") class Page {
  items = state([{ id: "a" }]);
  render() { return <ul>{this.items.map(item => <li key={item.id}><Leaf /></li>)}</ul>; }
}
"#,
        ));
        let region = model
            .component_instance_plan
            .instances
            .values()
            .find_map(|instance| instance.structural_region.as_ref())
            .expect("keyed leaf is a structural template");
        let fragments = generate_structural_keyed_host_fragments(&model, region);

        assert_eq!(fragments.len(), 1);
        assert_eq!(
            fragments[0].host_scope,
            StructuralConditionalHostScope::StaticInstance
        );
        assert!(fragments[0].item_template_html.contains("__ez_list_key__"));
        assert!(fragments[0]
            .item_template_html
            .contains("data-presolve-structural-invocation="));
    }
}
