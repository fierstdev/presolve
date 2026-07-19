//! Compiler-owned J1-P materialization of ordinary template instance markers.

use std::collections::{BTreeMap, BTreeSet};

use crate::template_graph::{AttributeValue, ElementNode, FragmentNode, TemplateAttribute};
use crate::{
    build_ordinary_template_instance_registry, build_resume_anchor_plan, ApplicationSemanticModel,
    ComponentInstanceId, ComponentInstanceStatus, ResumeAnchorPlacement, SerializableValue,
    SlotBindingStatus, TemplateChild, TemplateNode,
};

#[derive(Debug, Default)]
struct ResumeHtmlMarkers {
    element_anchors: BTreeMap<String, String>,
    text_anchors: BTreeMap<String, String>,
    structural_starts: BTreeMap<String, String>,
    structural_ends: BTreeMap<String, String>,
    events: BTreeMap<String, String>,
}

#[derive(Debug)]
struct SlotProjection<'a> {
    outlet_entity: crate::SemanticId,
    caller_instance: ComponentInstanceId,
    caller_template: &'a TemplateNode,
    invocation_element: &'a ElementNode,
    invocation_path: String,
    content_entities: BTreeSet<crate::SemanticId>,
    caller_slot_projections: &'a [SlotProjection<'a>],
}

/// Render the planned initial component topology with compiler-precomputed
/// ordinary instance and J10 resume markers.
#[must_use]
pub fn generate_ordinary_instance_html(model: &ApplicationSemanticModel) -> String {
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
            slot_projections,
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
                slot_projections,
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
                        "<template data-ez-r=\"{}\"></template>",
                        escape_attr(anchor)
                    )
                });
            bindings.get(&(instance.clone(), entity)).map_or_else(
                || format!("{resume_marker}<!-- ez-binding:{expression} -->{value}"),
                |id| format!("{resume_marker}<!--ez-ti-binding-start:{id}-->{value}<!--ez-ti-binding-end:{id}-->"),
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
            let selected = if matches!(
                conditional.initial_value,
                Some(SerializableValue::Boolean(true))
            ) {
                &conditional.when_true
            } else {
                &conditional.when_false
            };
            format!(
                "<!--ez-r-start:{}--><!--ez-conditional-start:{}:ti:{}-->{}<!--ez-conditional-end:{}:ti:{}--><!--ez-r-end:{}-->",
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
                    path,
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
                "<!--ez-r-start:{}--><!--ez-ti-target-start:{marker}-->{}<!--ez-ti-target-end:{marker}--><!--ez-r-end:{}-->",
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
            .find(|projection| projection.outlet_entity == entity)
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
    let mut html = format!(
        "<{} data-ez-node=\"{}\"",
        element.tag_name,
        escape_attr(&element.id.0)
    );
    if let Some(target) = targets.get(&(instance.clone(), entity)) {
        html.push_str(" data-ez-ti=\"");
        html.push_str(&escape_attr(target));
        html.push('"');
        if let Some(anchor) = resume_markers.element_anchors.get(target) {
            html.push_str(" data-ez-r=\"");
            html.push_str(&escape_attr(anchor));
            html.push('"');
        }
        if let Some(event) = resume_markers.events.get(target) {
            html.push_str(" data-ez-e=\"");
            html.push_str(&escape_attr(event));
            html.push('"');
        }
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
            Some(SlotProjection {
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
    let mut html = String::new();
    for (index, child) in projection.invocation_element.children.iter().enumerate() {
        let path = format!("{}.{}", projection.invocation_path, index);
        let entity = template_child_entity(&projection.caller_template.id, child, &path);
        if projection.content_entities.contains(&entity) {
            html.push_str(&render_child(
                model,
                templates,
                children,
                targets,
                bindings,
                resume_markers,
                &projection.caller_instance,
                projection.caller_template,
                child,
                &path,
                projection.caller_slot_projections,
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
            let wrapped_entity =
                template_child_entity(&projection.caller_template.id, wrapped, &wrapped_path);
            if projection.content_entities.contains(&wrapped_entity) {
                html.push_str(&render_child(
                    model,
                    templates,
                    children,
                    targets,
                    bindings,
                    resume_markers,
                    &projection.caller_instance,
                    projection.caller_template,
                    wrapped,
                    &wrapped_path,
                    projection.caller_slot_projections,
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
    use super::generate_ordinary_instance_html;
    use crate::{
        build_application_semantic_model, build_resume_anchor_plan, validate_resume_marker_html,
    };

    #[test]
    fn emits_precomputed_repeated_instance_and_exact_resume_markers() {
        let model = build_application_semantic_model(&ezc_parser::parse_file(
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
        assert_eq!(html.matches("data-ez-ti=").count(), 2);
        assert_eq!(html.matches("ez-ti-binding-start:").count(), 2);
        assert_eq!(html.matches("data-ez-r=").count(), 4);
        assert_eq!(html.matches("data-ez-e=").count(), 2);
        assert_eq!(html, generate_ordinary_instance_html(&model));
    }

    #[test]
    fn places_caller_owned_slot_content_at_the_exact_compiler_outlet() {
        let model = build_application_semantic_model(&ezc_parser::parse_file(
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
        assert_eq!(html.matches("ez-ti-binding-start:").count(), 2);
        assert!(
            validate_resume_marker_html(&resume, &html).is_empty(),
            "caller-owned Slot content must carry every required resume marker"
        );
    }
}
