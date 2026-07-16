//! Compiler-owned J1-P materialization of ordinary template instance markers.

use std::collections::BTreeMap;

use crate::template_graph::{AttributeValue, ElementNode, FragmentNode, TemplateAttribute};
use crate::{
    build_ordinary_template_instance_registry, ApplicationSemanticModel, ComponentInstanceId,
    ComponentInstanceStatus, SerializableValue, TemplateChild, TemplateNode,
};

/// Render the planned initial component topology with only compiler-precomputed
/// ordinary instance markers. This intentionally does not introduce resume
/// anchors or runtime-generated component identities.
#[must_use]
pub fn generate_ordinary_instance_html(model: &ApplicationSemanticModel) -> String {
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
                &instance.id,
                &instance.component,
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
    instance: &ComponentInstanceId,
    component: &crate::SemanticId,
) -> String {
    let Some(template) = templates.get(&component.template()) else {
        return String::new();
    };
    if let Some(root) = &template.root {
        return render_element(
            model, templates, children, targets, bindings, instance, template, root, "root",
        );
    }
    template
        .root_fragment
        .as_ref()
        .map_or_else(String::new, |fragment| {
            render_fragment(
                model, templates, children, targets, bindings, instance, template, fragment, "root",
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
    instance: &ComponentInstanceId,
    template: &TemplateNode,
    fragment: &FragmentNode,
    path: &str,
) -> String {
    render_children(
        model,
        templates,
        children,
        targets,
        bindings,
        instance,
        template,
        &fragment.children,
        path,
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
    instance: &ComponentInstanceId,
    template: &TemplateNode,
    nodes: &[TemplateChild],
    parent_path: &str,
) -> String {
    nodes.iter().enumerate().map(|(index, node)| {
        let path = format!("{parent_path}.{index}");
        match node {
            TemplateChild::Text { value, .. } => escape_text(value),
            TemplateChild::Binding { expression, initial_value, .. } => {
                let entity = template.id.template_entity("binding", &path);
                let value = initial_value.as_ref().map_or_else(String::new, SerializableValue::render_text);
                bindings.get(&(instance.clone(), entity)).map_or_else(
                    || format!("<!-- ez-binding:{expression} -->{value}"),
                    |id| format!("<!--ez-ti-binding-start:{id}-->{value}<!--ez-ti-binding-end:{id}-->"),
                )
            }
            TemplateChild::Element(element) => render_element(model, templates, children, targets, bindings, instance, template, element, &path),
            TemplateChild::Fragment(fragment) => render_fragment(model, templates, children, targets, bindings, instance, template, fragment, &path),
            TemplateChild::Conditional(conditional) => {
                let entity = template.id.template_entity("conditional", &path);
                let marker = targets.get(&(instance.clone(), entity)).map_or("", |id| id.as_str());
                let selected = if matches!(conditional.initial_value, Some(SerializableValue::Boolean(true))) { &conditional.when_true } else { &conditional.when_false };
                format!("<!--ez-conditional-start:{}:ti:{}-->{}<!--ez-conditional-end:{}:ti:{}-->", conditional.start_id.0, marker, render_children(model, templates, children, targets, bindings, instance, template, selected, &path), conditional.end_id.0, marker)
            }
            TemplateChild::List(list) => {
                let entity = template.id.template_entity("list", &path);
                let marker = targets.get(&(instance.clone(), entity)).map_or("", |id| id.as_str());
                format!(
                    "<!--ez-ti-target-start:{marker}-->{}<!--ez-ti-target-end:{marker}-->",
                    crate::html_codegen::generate_list_html(list)
                )
            }
        }
    }).collect()
}

#[allow(clippy::too_many_arguments)]
fn render_element(
    model: &ApplicationSemanticModel,
    templates: &BTreeMap<crate::SemanticId, &TemplateNode>,
    children: &BTreeMap<
        (ComponentInstanceId, crate::ComponentInvocationId),
        &crate::ComponentInstance,
    >,
    targets: &BTreeMap<(ComponentInstanceId, crate::SemanticId), String>,
    bindings: &BTreeMap<(ComponentInstanceId, crate::SemanticId), String>,
    instance: &ComponentInstanceId,
    template: &TemplateNode,
    element: &ElementNode,
    path: &str,
) -> String {
    let entity = template.id.template_entity("element", path);
    if let Some(invocation) = model
        .component_invocations
        .values()
        .find(|candidate| candidate.template_entity == entity)
    {
        if let Some(child) = children.get(&(instance.clone(), invocation.id.clone())) {
            if child.status == ComponentInstanceStatus::Planned {
                return render_instance(
                    model,
                    templates,
                    children,
                    targets,
                    bindings,
                    &child.id,
                    &child.component,
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
        instance,
        template,
        &element.children,
        path,
    ));
    html.push_str("</");
    html.push_str(&element.tag_name);
    html.push('>');
    html
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
fn escape_text(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}

#[cfg(test)]
mod tests {
    use super::generate_ordinary_instance_html;
    use crate::build_application_semantic_model;

    #[test]
    fn emits_precomputed_repeated_instance_markers_without_resume_markers() {
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
        assert!(!html.contains("data-ez-r"));
        assert!(!html.contains("data-ez-e"));
        assert_eq!(html, generate_ordinary_instance_html(&model));
    }
}
