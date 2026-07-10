use serde::Serialize;

use crate::component_graph::{ComponentGraph, ComponentNode, SerializableValue, StateOperation};
use crate::template_graph::{
    AttributeValue, ElementNode, TemplateChild, TemplateGraph, TemplateNode,
};

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct TemplateManifest {
    pub components: Vec<ManifestComponent>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ManifestComponent {
    pub name: String,
    pub template: ManifestTemplate,
    pub actions: Vec<ManifestAction>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ManifestTemplate {
    pub nodes: Vec<ManifestNode>,
    pub events: Vec<ManifestEvent>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(tag = "kind")]
pub enum ManifestNode {
    #[serde(rename = "element")]
    Element { id: String, tag: String },

    #[serde(rename = "binding")]
    Binding {
        id: String,
        expression: String,
        initial_value: Option<SerializableValue>,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ManifestEvent {
    pub node: String,
    pub event: String,
    pub handler: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ManifestAction {
    pub method: String,
    pub operation: ManifestOperation,
    pub field: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub operand: Option<SerializableValue>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub enum ManifestOperation {
    #[serde(rename = "increment")]
    Increment,

    #[serde(rename = "decrement")]
    Decrement,

    #[serde(rename = "add_assign")]
    AddAssign,

    #[serde(rename = "subtract_assign")]
    SubtractAssign,
}

#[must_use]
pub fn build_template_manifest(
    component_graph: &ComponentGraph,
    template_graph: &TemplateGraph,
) -> TemplateManifest {
    TemplateManifest {
        components: template_graph
            .templates
            .iter()
            .map(|template| manifest_component(component_graph, template))
            .collect::<Vec<_>>(),
    }
}

/// Serialize a template manifest as pretty JSON.
///
/// # Panics
///
/// Panics if serde cannot serialize the compiler-owned manifest model.
#[must_use]
pub fn template_manifest_json(manifest: &TemplateManifest) -> String {
    serde_json::to_string_pretty(manifest).expect("template manifest should serialize")
}

fn manifest_component(
    component_graph: &ComponentGraph,
    template: &TemplateNode,
) -> ManifestComponent {
    let mut nodes = Vec::new();
    let mut events = Vec::new();

    if let Some(root) = &template.root {
        collect_element(root, &mut nodes, &mut events);
    }

    let actions = component_graph
        .components
        .iter()
        .find(|component| component.class_name == template.component_name)
        .map(manifest_actions)
        .unwrap_or_default();

    ManifestComponent {
        name: template.component_name.clone(),
        template: ManifestTemplate { nodes, events },
        actions,
    }
}

fn manifest_actions(component: &ComponentNode) -> Vec<ManifestAction> {
    component
        .actions
        .iter()
        .map(|action| ManifestAction {
            method: action.method.clone(),
            operation: manifest_operation(&action.operation),
            field: action.field.clone(),
            operand: manifest_operand(&action.operation),
        })
        .collect()
}

fn manifest_operation(operation: &StateOperation) -> ManifestOperation {
    match operation {
        StateOperation::Increment => ManifestOperation::Increment,
        StateOperation::Decrement => ManifestOperation::Decrement,
        StateOperation::AddAssign(_) => ManifestOperation::AddAssign,
        StateOperation::SubtractAssign(_) => ManifestOperation::SubtractAssign,
    }
}

fn manifest_operand(operation: &StateOperation) -> Option<SerializableValue> {
    match operation {
        StateOperation::Increment | StateOperation::Decrement => None,
        StateOperation::AddAssign(value) | StateOperation::SubtractAssign(value) => {
            Some(value.clone())
        }
    }
}

fn collect_element(
    element: &ElementNode,
    nodes: &mut Vec<ManifestNode>,
    events: &mut Vec<ManifestEvent>,
) {
    nodes.push(ManifestNode::Element {
        id: element.id.0.clone(),
        tag: element.tag_name.clone(),
    });

    for attribute in &element.attributes {
        if let AttributeValue::EventHandler { event, handler } = &attribute.value {
            events.push(ManifestEvent {
                node: element.id.0.clone(),
                event: event.clone(),
                handler: handler.clone(),
            });
        }
    }

    for child in &element.children {
        match child {
            TemplateChild::Text(_) => {}
            TemplateChild::Binding {
                id,
                expression,
                initial_value,
            } => {
                nodes.push(ManifestNode::Binding {
                    id: id.0.clone(),
                    expression: expression.clone(),
                    initial_value: initial_value.clone(),
                });
            }
            TemplateChild::Element(element) => collect_element(element, nodes, events),
        }
    }
}
