use crate::component_graph::{
    ComponentGraph, RenderChild, RenderElement, RenderEventHandler, RenderModel, StateField,
    StateInitialValue,
};

#[derive(Debug, Default)]
struct TemplateIdAllocator {
    next: usize,
}

impl TemplateIdAllocator {
    fn alloc(&mut self) -> TemplateNodeId {
        let id = TemplateNodeId(format!("n{}", self.next));
        self.next += 1;
        id
    }
}

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
    pub id: TemplateNodeId,
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
    EventHandler { event: String, handler: String },
    BindingList(Vec<String>),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TemplateChild {
    Text(String),
    Binding {
        id: TemplateNodeId,
        expression: String,
        initial_value: Option<StateInitialValue>,
    },
    Element(ElementNode),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TemplateNodeId(pub String);

pub fn build_template_graph(component_graph: &ComponentGraph) -> TemplateGraph {
    let mut ids = TemplateIdAllocator::default();

    let templates = component_graph
        .components
        .iter()
        .map(|component| TemplateNode {
            component_name: component.class_name.clone(),
            root: component
                .render
                .as_ref()
                .and_then(|render| element_from_render(render, &component.state_fields, &mut ids)),
        })
        .collect::<Vec<_>>();

    TemplateGraph { templates }
}

fn element_from_render(
    render: &RenderModel,
    state_fields: &[StateField],
    ids: &mut TemplateIdAllocator,
) -> Option<ElementNode> {
    let tag_name = render.root_element.clone()?;
    let id = ids.alloc();

    let direct_bindings = collect_direct_bindings_from_children(&render.children);

    let attributes =
        template_attributes(&render.attributes, &render.event_handlers, &direct_bindings);

    let children = render
        .children
        .iter()
        .map(|child| template_child_from_render(child, state_fields, ids))
        .collect::<Vec<_>>();

    Some(ElementNode {
        id,
        tag_name,
        attributes,
        children,
    })
}

fn element_from_render_element(
    element: &RenderElement,
    state_fields: &[StateField],
    ids: &mut TemplateIdAllocator,
) -> ElementNode {
    let id = ids.alloc();

    ElementNode {
        id,
        tag_name: element.tag_name.clone(),
        attributes: template_attributes(
            &element.attributes,
            &element.event_handlers,
            &collect_direct_bindings_from_children(&element.children),
        ),
        children: element
            .children
            .iter()
            .map(|child| template_child_from_render(child, state_fields, ids))
            .collect::<Vec<_>>(),
    }
}

fn template_attributes(
    _attributes: &[String],
    event_handlers: &[RenderEventHandler],
    bindings: &[String],
) -> Vec<TemplateAttribute> {
    let mut attributes = Vec::new();

    for event_handler in event_handlers {
        attributes.push(TemplateAttribute {
            name: format!("data-ez-on-{}", event_handler.event),
            value: AttributeValue::EventHandler {
                event: event_handler.event.clone(),
                handler: event_handler.handler.clone(),
            },
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

fn template_child_from_render(
    child: &RenderChild,
    state_fields: &[StateField],
    ids: &mut TemplateIdAllocator,
) -> TemplateChild {
    match child {
        RenderChild::Text(text) => TemplateChild::Text(text.clone()),

        RenderChild::Binding(binding) => TemplateChild::Binding {
            id: ids.alloc(),
            expression: binding.clone(),
            initial_value: binding_initial_value(binding, state_fields),
        },

        RenderChild::Element(element) => {
            TemplateChild::Element(element_from_render_element(element, state_fields, ids))
        }
    }
}

fn binding_initial_value(
    expression: &str,
    state_fields: &[StateField],
) -> Option<StateInitialValue> {
    let field_name = expression.strip_prefix("this.")?;

    state_fields
        .iter()
        .find(|field| field.name == field_name)
        .and_then(|field| field.initial_value.clone())
}
