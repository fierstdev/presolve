use crate::component_graph::{ComponentGraph, RenderChild, RenderElement, RenderModel};

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
    Element(ElementNode),
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

    let direct_bindings = collect_direct_bindings_from_children(&render.children);

    let attributes = template_attributes(
        &render.attributes,
        &render.event_handler_refs,
        &direct_bindings,
    );

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

fn element_from_render_element(element: &RenderElement) -> ElementNode {
    ElementNode {
        tag_name: element.tag_name.clone(),
        attributes: template_attributes(
            &element.attributes,
            &element.event_handler_refs,
            &collect_direct_bindings_from_children(&element.children),
        ),
        children: element
            .children
            .iter()
            .map(template_child_from_render)
            .collect::<Vec<_>>(),
    }
}

fn template_attributes(
    _attributes: &[String],
    event_handler_refs: &[String],
    bindings: &[String],
) -> Vec<TemplateAttribute> {
    let mut attributes = Vec::new();

    for event_handler in event_handler_refs {
        attributes.push(TemplateAttribute {
            name: "data-ez-event-handler".to_string(),
            value: AttributeValue::EventHandler(event_handler.clone()),
        });
    }

    if !bindings.is_empty() {
        attributes.push(TemplateAttribute {
            name: "data-ez-bindings".to_string(),
            value: AttributeValue::BindingList(bindings.to_vec()),
        });
    }

    attributes
}

fn collect_direct_bindings_from_children(children: &[RenderChild]) -> Vec<String> {
    let mut bindings = Vec::new();

    for child in children {
        if let RenderChild::Binding(binding) = child {
            bindings.push(binding.clone());
        }
    }

    bindings
}

fn template_child_from_render(child: &RenderChild) -> TemplateChild {
    match child {
        RenderChild::Text(text) => TemplateChild::Text(text.clone()),
        RenderChild::Binding(binding) => TemplateChild::Binding(binding.clone()),
        RenderChild::Element(element) => {
            TemplateChild::Element(element_from_render_element(element))
        }
    }
}
