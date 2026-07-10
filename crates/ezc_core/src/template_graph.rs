use crate::component_graph::{
    ComponentGraph, RenderAttribute, RenderAttributeValue, RenderChild, RenderElement,
    RenderEventHandler, RenderModel, SerializableValue, StateField,
};
use ezc_parser::SourceSpan;

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
    pub span: SourceSpan,
    pub attributes: Vec<TemplateAttribute>,
    pub children: Vec<TemplateChild>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TemplateAttribute {
    pub name: String,
    pub value: AttributeValue,
    pub span: Option<SourceSpan>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AttributeValue {
    Boolean,
    Static(String),
    Binding {
        id: TemplateNodeId,
        expression: String,
        initial_value: Option<SerializableValue>,
    },
    EventHandler {
        event: String,
        handler: String,
    },
    BindingList(Vec<String>),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TemplateChild {
    Text {
        value: String,
        span: SourceSpan,
    },
    Binding {
        id: TemplateNodeId,
        expression: String,
        initial_value: Option<SerializableValue>,
        span: SourceSpan,
    },
    Element(ElementNode),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TemplateNodeId(pub String);

#[must_use]
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
    let span = render.root_span?;
    let id = ids.alloc();

    let direct_bindings = collect_direct_bindings_from_children(&render.children);

    let attributes = template_attributes(
        &render.attributes,
        &render.event_handlers,
        &direct_bindings,
        state_fields,
        ids,
    );

    let children = render
        .children
        .iter()
        .map(|child| template_child_from_render(child, state_fields, ids))
        .collect::<Vec<_>>();

    Some(ElementNode {
        id,
        tag_name,
        span,
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
        span: element.span,
        attributes: template_attributes(
            &element.attributes,
            &element.event_handlers,
            &collect_direct_bindings_from_children(&element.children),
            state_fields,
            ids,
        ),
        children: element
            .children
            .iter()
            .map(|child| template_child_from_render(child, state_fields, ids))
            .collect::<Vec<_>>(),
    }
}

fn template_attributes(
    static_attributes: &[RenderAttribute],
    event_handlers: &[RenderEventHandler],
    bindings: &[String],
    state_fields: &[StateField],
    ids: &mut TemplateIdAllocator,
) -> Vec<TemplateAttribute> {
    let mut attributes = Vec::new();

    for attribute in static_attributes {
        match &attribute.value {
            RenderAttributeValue::Boolean if !is_event_attribute(&attribute.name) => {
                attributes.push(TemplateAttribute {
                    name: attribute.name.clone(),
                    value: AttributeValue::Boolean,
                    span: Some(attribute.span),
                });
            }
            RenderAttributeValue::Static(value) if !is_event_attribute(&attribute.name) => {
                attributes.push(TemplateAttribute {
                    name: attribute.name.clone(),
                    value: AttributeValue::Static(value.clone()),
                    span: Some(attribute.span),
                });
            }
            RenderAttributeValue::Expression(Some(expression))
                if !is_event_attribute(&attribute.name)
                    && expression.strip_prefix("this.").is_some() =>
            {
                attributes.push(TemplateAttribute {
                    name: attribute.name.clone(),
                    value: AttributeValue::Binding {
                        id: ids.alloc(),
                        expression: expression.clone(),
                        initial_value: binding_initial_value(expression, state_fields),
                    },
                    span: Some(attribute.span),
                });
            }
            _ => {}
        }
    }

    for event_handler in event_handlers {
        attributes.push(TemplateAttribute {
            name: format!("data-ez-on-{}", event_handler.event),
            value: AttributeValue::EventHandler {
                event: event_handler.event.clone(),
                handler: event_handler.handler.clone(),
            },
            span: Some(event_handler.span),
        });
    }

    if !bindings.is_empty() {
        attributes.push(TemplateAttribute {
            name: "data-ez-bindings".to_string(),
            value: AttributeValue::BindingList(bindings.to_vec()),
            span: None,
        });
    }

    attributes
}

fn is_event_attribute(name: &str) -> bool {
    name.strip_prefix("on")
        .and_then(|event| event.chars().next())
        .is_some_and(char::is_uppercase)
}

fn collect_direct_bindings_from_children(children: &[RenderChild]) -> Vec<String> {
    let mut bindings = Vec::new();

    for child in children {
        if let RenderChild::Binding { expression, .. } = child {
            bindings.push(expression.clone());
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
        RenderChild::Text { value, span } => TemplateChild::Text {
            value: value.clone(),
            span: *span,
        },

        RenderChild::Binding { expression, span } => TemplateChild::Binding {
            id: ids.alloc(),
            expression: expression.clone(),
            initial_value: binding_initial_value(expression, state_fields),
            span: *span,
        },

        RenderChild::Element(element) => {
            TemplateChild::Element(element_from_render_element(element, state_fields, ids))
        }
    }
}

fn binding_initial_value(
    expression: &str,
    state_fields: &[StateField],
) -> Option<SerializableValue> {
    let field_name = expression.strip_prefix("this.")?;

    state_fields
        .iter()
        .find(|field| field.name == field_name)
        .and_then(|field| field.initial_value.clone())
}
