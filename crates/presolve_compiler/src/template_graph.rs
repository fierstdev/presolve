use crate::component_graph::{
    ComponentGraph, MethodLocalVariable, RenderAttribute, RenderAttributeValue, RenderChild,
    RenderConditional, RenderElement, RenderEventHandler, RenderFragment, RenderList, RenderModel,
    SerializableValue, StateField,
};
use presolve_parser::SourceSpan;

use crate::semantic_id::{SemanticId, SemanticOwner};
use crate::semantic_provenance::SourceProvenance;

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
    pub id: SemanticId,
    pub owner: SemanticOwner,
    pub provenance: SourceProvenance,
    pub component_name: String,
    pub root: Option<ElementNode>,
    pub root_fragment: Option<FragmentNode>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ElementNode {
    pub id: TemplateNodeId,
    pub tag_name: String,
    pub tag_name_span: SourceSpan,
    pub span: SourceSpan,
    pub attributes: Vec<TemplateAttribute>,
    /// Complete normalized authored attributes retained for compiler analyses.
    /// Backend-facing `attributes` intentionally contains only executable
    /// template attributes.
    pub authored_attributes: Vec<RenderAttribute>,
    pub children: Vec<TemplateChild>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FragmentNode {
    pub id: TemplateNodeId,
    pub span: SourceSpan,
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
    Fragment(FragmentNode),
    Conditional(ConditionalNode),
    List(ListNode),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConditionalNode {
    pub id: TemplateNodeId,
    pub start_id: TemplateNodeId,
    pub end_id: TemplateNodeId,
    pub condition: String,
    pub initial_value: Option<SerializableValue>,
    pub span: SourceSpan,
    pub when_true: Vec<TemplateChild>,
    pub when_false: Vec<TemplateChild>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ListNode {
    pub id: TemplateNodeId,
    pub start_id: TemplateNodeId,
    pub end_id: TemplateNodeId,
    pub iterable: String,
    pub initial_value: Option<SerializableValue>,
    pub item_variable: String,
    pub index_variable: Option<String>,
    pub key_expression: String,
    pub span: SourceSpan,
    pub item_template: Vec<TemplateChild>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TemplateNodeId(pub String);

#[derive(Clone, Copy)]
struct ListItemTemplateScope<'a> {
    item_variable: &'a str,
    index_variable: Option<&'a str>,
}

#[must_use]
///
/// # Panics
///
/// Panics when a component has no semantic provenance entry. Component-graph
/// construction establishes this invariant before template construction.
pub fn build_template_graph(component_graph: &ComponentGraph) -> TemplateGraph {
    let mut ids = TemplateIdAllocator::default();

    let templates = component_graph
        .components
        .iter()
        .map(|component| {
            let local_variables = component
                .methods
                .iter()
                .find(|method| method.name == "render")
                .map_or(&[][..], |method| method.local_variables.as_slice());

            TemplateNode {
                id: component.id.template(),
                owner: SemanticOwner::entity(component.id.clone()),
                provenance: component_graph
                    .provenance
                    .get(&component.id.template())
                    .or_else(|| component_graph.provenance.get(&component.id))
                    .cloned()
                    .expect("component semantic provenance should exist"),
                component_name: component.class_name.clone(),
                root: component.render.as_ref().and_then(|render| {
                    element_from_render(render, &component.state_fields, local_variables, &mut ids)
                }),
                root_fragment: component.render.as_ref().and_then(|render| {
                    render.root_fragment.as_ref().map(|fragment| {
                        fragment_from_render_fragment(
                            fragment,
                            &component.state_fields,
                            local_variables,
                            &mut ids,
                            None,
                        )
                    })
                }),
            }
        })
        .collect::<Vec<_>>();

    TemplateGraph { templates }
}

fn element_from_render(
    render: &RenderModel,
    state_fields: &[StateField],
    local_variables: &[MethodLocalVariable],
    ids: &mut TemplateIdAllocator,
) -> Option<ElementNode> {
    let tag_name = render.root_element.clone()?;
    let tag_name_span = render.root_element_name_span?;
    let span = render.root_span?;
    let id = ids.alloc();

    let direct_bindings = collect_direct_bindings_from_children(&render.children);

    let attributes = template_attributes(
        &tag_name,
        &render.attributes,
        &render.event_handlers,
        &direct_bindings,
        state_fields,
        local_variables,
        ids,
        None,
    );

    let children = render
        .children
        .iter()
        .map(|child| template_child_from_render(child, state_fields, local_variables, ids, None))
        .collect::<Vec<_>>();

    Some(ElementNode {
        id,
        tag_name,
        tag_name_span,
        span,
        attributes,
        authored_attributes: render.attributes.clone(),
        children,
    })
}

fn element_from_render_element(
    element: &RenderElement,
    state_fields: &[StateField],
    local_variables: &[MethodLocalVariable],
    ids: &mut TemplateIdAllocator,
    list_scope: Option<ListItemTemplateScope<'_>>,
) -> ElementNode {
    let id = ids.alloc();

    ElementNode {
        id,
        tag_name: element.tag_name.clone(),
        tag_name_span: element.tag_name_span,
        span: element.span,
        attributes: template_attributes(
            &element.tag_name,
            &element.attributes,
            &element.event_handlers,
            &collect_direct_bindings_from_children(&element.children),
            state_fields,
            local_variables,
            ids,
            list_scope,
        ),
        authored_attributes: element.attributes.clone(),
        children: element
            .children
            .iter()
            .map(|child| {
                template_child_from_render(child, state_fields, local_variables, ids, list_scope)
            })
            .collect::<Vec<_>>(),
    }
}

fn fragment_from_render_fragment(
    fragment: &RenderFragment,
    state_fields: &[StateField],
    local_variables: &[MethodLocalVariable],
    ids: &mut TemplateIdAllocator,
    list_scope: Option<ListItemTemplateScope<'_>>,
) -> FragmentNode {
    FragmentNode {
        id: ids.alloc(),
        span: fragment.span,
        children: fragment
            .children
            .iter()
            .map(|child| {
                template_child_from_render(child, state_fields, local_variables, ids, list_scope)
            })
            .collect(),
    }
}

#[allow(clippy::too_many_arguments)]
fn template_attributes(
    tag_name: &str,
    static_attributes: &[RenderAttribute],
    event_handlers: &[RenderEventHandler],
    bindings: &[String],
    state_fields: &[StateField],
    local_variables: &[MethodLocalVariable],
    ids: &mut TemplateIdAllocator,
    list_scope: Option<ListItemTemplateScope<'_>>,
) -> Vec<TemplateAttribute> {
    let mut attributes = Vec::new();

    for attribute in static_attributes {
        // `field` and the explicit intrinsic-form host marker are compiler
        // facts, not ordinary HTML attributes.
        if attribute.name == "field" || (tag_name == "form" && attribute.name == "form") {
            continue;
        }
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
                    && attribute.name != "key"
                    && (expression.strip_prefix("this.").is_some()
                        || (list_scope.is_none()
                            && unique_local_variable(local_variables, expression).is_some())
                        || list_scope
                            .is_some_and(|scope| list_item_expression(expression, scope))) =>
            {
                attributes.push(TemplateAttribute {
                    name: attribute.name.clone(),
                    value: AttributeValue::Binding {
                        id: ids.alloc(),
                        expression: expression.clone(),
                        initial_value: binding_initial_value(
                            expression,
                            state_fields,
                            local_variables,
                            list_scope.is_none(),
                        ),
                    },
                    span: Some(attribute.span),
                });
            }
            _ => {}
        }
    }

    for event_handler in event_handlers {
        attributes.push(TemplateAttribute {
            name: format!("data-presolve-on-{}", event_handler.event),
            value: AttributeValue::EventHandler {
                event: event_handler.event.clone(),
                handler: event_handler.handler.clone(),
            },
            span: Some(event_handler.span),
        });
    }

    if !bindings.is_empty() {
        attributes.push(TemplateAttribute {
            name: "data-presolve-bindings".to_string(),
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

fn list_item_expression(expression: &str, scope: ListItemTemplateScope<'_>) -> bool {
    expression == scope.item_variable
        || scope.index_variable == Some(expression)
        || expression
            .strip_prefix(scope.item_variable)
            .and_then(|suffix| suffix.strip_prefix('.'))
            .is_some_and(|path| !path.is_empty() && !path.split('.').any(str::is_empty))
}

fn collect_direct_bindings_from_children(children: &[RenderChild]) -> Vec<String> {
    let mut bindings = Vec::new();

    for child in children {
        match child {
            RenderChild::Binding { expression, .. } => bindings.push(expression.clone()),
            RenderChild::Fragment(fragment) => {
                bindings.extend(collect_direct_bindings_from_children(&fragment.children));
            }
            RenderChild::Conditional(conditional) => {
                bindings.push(conditional.condition.clone());
            }
            RenderChild::List(list) => bindings.push(list.iterable.clone()),
            RenderChild::Element(_) | RenderChild::Text { .. } => {}
        }
    }

    bindings
}

fn template_child_from_render(
    child: &RenderChild,
    state_fields: &[StateField],
    local_variables: &[MethodLocalVariable],
    ids: &mut TemplateIdAllocator,
    list_scope: Option<ListItemTemplateScope<'_>>,
) -> TemplateChild {
    match child {
        RenderChild::Text { value, span } => TemplateChild::Text {
            value: value.clone(),
            span: *span,
        },

        RenderChild::Binding { expression, span } => TemplateChild::Binding {
            id: ids.alloc(),
            expression: expression.clone(),
            initial_value: binding_initial_value(
                expression,
                state_fields,
                local_variables,
                list_scope.is_none(),
            ),
            span: *span,
        },

        RenderChild::Element(element) => TemplateChild::Element(element_from_render_element(
            element,
            state_fields,
            local_variables,
            ids,
            list_scope,
        )),
        RenderChild::Fragment(fragment) => TemplateChild::Fragment(fragment_from_render_fragment(
            fragment,
            state_fields,
            local_variables,
            ids,
            list_scope,
        )),
        RenderChild::Conditional(conditional) => {
            TemplateChild::Conditional(conditional_from_render_conditional(
                conditional,
                state_fields,
                local_variables,
                ids,
                list_scope,
            ))
        }
        RenderChild::List(list) => TemplateChild::List(list_from_render_list(
            list,
            state_fields,
            local_variables,
            ids,
        )),
    }
}

fn list_from_render_list(
    list: &RenderList,
    state_fields: &[StateField],
    local_variables: &[MethodLocalVariable],
    ids: &mut TemplateIdAllocator,
) -> ListNode {
    ListNode {
        id: ids.alloc(),
        start_id: ids.alloc(),
        end_id: ids.alloc(),
        iterable: list.iterable.clone(),
        initial_value: binding_initial_value(&list.iterable, state_fields, local_variables, false),
        item_variable: list.item_variable.clone(),
        index_variable: list.index_variable.clone(),
        key_expression: list.key_expression.clone(),
        span: list.span,
        item_template: list
            .item_template
            .iter()
            .map(|child| {
                template_child_from_render(
                    child,
                    state_fields,
                    local_variables,
                    ids,
                    Some(ListItemTemplateScope {
                        item_variable: &list.item_variable,
                        index_variable: list.index_variable.as_deref(),
                    }),
                )
            })
            .collect(),
    }
}

fn conditional_from_render_conditional(
    conditional: &RenderConditional,
    state_fields: &[StateField],
    local_variables: &[MethodLocalVariable],
    ids: &mut TemplateIdAllocator,
    list_scope: Option<ListItemTemplateScope<'_>>,
) -> ConditionalNode {
    ConditionalNode {
        id: ids.alloc(),
        start_id: ids.alloc(),
        end_id: ids.alloc(),
        condition: conditional.condition.clone(),
        initial_value: binding_initial_value(
            &conditional.condition,
            state_fields,
            local_variables,
            false,
        ),
        span: conditional.span,
        when_true: conditional
            .when_true
            .iter()
            .map(|child| {
                template_child_from_render(child, state_fields, local_variables, ids, list_scope)
            })
            .collect(),
        when_false: conditional
            .when_false
            .iter()
            .map(|child| {
                template_child_from_render(child, state_fields, local_variables, ids, list_scope)
            })
            .collect(),
    }
}

fn binding_initial_value(
    expression: &str,
    state_fields: &[StateField],
    local_variables: &[MethodLocalVariable],
    allow_local: bool,
) -> Option<SerializableValue> {
    if let Some(field_name) = expression.strip_prefix("this.") {
        return state_fields
            .iter()
            .find(|field| field.name == field_name)
            .and_then(|field| field.initial_value.clone());
    }

    if allow_local {
        return unique_local_variable(local_variables, expression).map(|local| local.value.clone());
    }

    None
}

fn unique_local_variable<'a>(
    local_variables: &'a [MethodLocalVariable],
    name: &str,
) -> Option<&'a MethodLocalVariable> {
    let mut locals = local_variables.iter().filter(|local| local.name == name);
    let local = locals.next()?;
    locals.next().is_none().then_some(local)
}
