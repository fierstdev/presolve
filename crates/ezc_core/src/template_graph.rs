use crate::component_graph::{ComponentGraph, RenderChild, RenderModel};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TemplateGraph {
    pub templates: Vec<TemplateNode>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TemplateNode {
    pub component_name: String,
    pub root: Option<ElementNode>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ElementNode {
    pub tag_name: String,
    pub attributes: Vec<TemplateAttribute>,
    pub children: Vec<TemplateChild>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TemplateAttribute {
    pub name: String,
    pub value: AttributeValue,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AttributeValue {
    Static(String),
    EventHandler(String),
    BindingList(Vec<String>),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TemplateChild {
    Text(String),
    Binding(String),
}

pub fn build_template_graph(component_graph: &ComponentGraph) -> TemplateGraph {
    let templates = component_graph
        .components
        .iter()
        .map(|component| TemplateNode {
            component_name: component.class_name.clone(),
            root: component.render.as_ref().and_then(element_from_render),
        })
        .collect::<Vec<_>>();

    TemplateGraph { templates }
}

fn element_from_render(render: &RenderModel) -> Option<ElementNode> {
    let tag_name = render.root_element.clone()?;

    let mut attributes = Vec::new();

    for event_handler in &render.event_handler_refs {
        attributes.push(TemplateAttribute {
            name: "data-ez-event-handler".to_string(),
            value: AttributeValue::EventHandler(event_handler.clone()),
        });
    }

    if !render.bindings.is_empty() {
        attributes.push(TemplateAttribute {
            name: "data-ez-bindings".to_string(),
            value: AttributeValue::BindingList(render.bindings.clone()),
        });
    }

    let children = render
        .children
        .iter()
        .map(template_child_from_render)
        .collect::<Vec<_>>();

    Some(ElementNode {
        tag_name,
        attributes,
        children,
    })
}

fn template_child_from_render(child: &RenderChild) -> TemplateChild {
    match child {
        RenderChild::Text(text) => TemplateChild::Text(text.clone()),
        RenderChild::Binding(binding) => TemplateChild::Binding(binding.clone()),
    }
}
