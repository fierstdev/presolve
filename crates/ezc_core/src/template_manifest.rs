use serde::Serialize;

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
        initial_value: Option<String>,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ManifestEvent {
    pub node: String,
    pub handler: String,
}

pub fn build_template_manifest(template_graph: &TemplateGraph) -> TemplateManifest {
    TemplateManifest {
        components: template_graph
            .templates
            .iter()
            .map(manifest_component)
            .collect::<Vec<_>>(),
    }
}

pub fn template_manifest_json(manifest: &TemplateManifest) -> String {
    serde_json::to_string_pretty(manifest).expect("template manifest should serialize")
}

fn manifest_component(template: &TemplateNode) -> ManifestComponent {
    let mut nodes = Vec::new();
    let mut events = Vec::new();

    if let Some(root) = &template.root {
        collect_element(root, &mut nodes, &mut events);
    }

    ManifestComponent {
        name: template.component_name.clone(),
        template: ManifestTemplate { nodes, events },
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
        if let AttributeValue::EventHandler(handler) = &attribute.value {
            events.push(ManifestEvent {
                node: element.id.0.clone(),
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
