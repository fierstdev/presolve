use serde::Serialize;

use ezc_parser::{
    ParsedClass, ParsedEventHandler, ParsedFile, ParsedJsxAttribute, ParsedJsxAttributeValue,
    ParsedJsxChild, ParsedSerializableValue, ParsedStateOperation, SourceSpan,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ComponentGraph {
    pub components: Vec<ComponentNode>,
    pub diagnostics: Vec<ComponentDiagnostic>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ComponentNode {
    pub class_name: String,
    pub element_name: Option<String>,
    pub route_path: Option<String>,
    pub state_fields: Vec<StateField>,
    pub methods: Vec<ComponentMethod>,
    pub actions: Vec<ComponentAction>,
    pub render: Option<RenderModel>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StateField {
    pub name: String,
    pub initial_value: Option<SerializableValue>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(untagged)]
pub enum SerializableValue {
    Null,
    Number(String),
    String(String),
    Boolean(bool),
}

impl SerializableValue {
    #[must_use]
    pub fn render_text(&self) -> String {
        match self {
            Self::Null => String::new(),
            Self::Number(value) | Self::String(value) => value.clone(),
            Self::Boolean(value) => value.to_string(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ComponentMethod {
    pub name: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ComponentAction {
    pub method: String,
    pub operation: StateOperation,
    pub field: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StateOperation {
    Increment,
    Decrement,
    AddAssign(SerializableValue),
    SubtractAssign(SerializableValue),
    Assign(SerializableValue),
    Toggle,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RenderChild {
    Text {
        value: String,
        span: SourceSpan,
    },
    Binding {
        expression: String,
        span: SourceSpan,
    },
    Element(RenderElement),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RenderElement {
    pub tag_name: String,
    pub span: SourceSpan,
    pub attributes: Vec<RenderAttribute>,
    pub event_handlers: Vec<RenderEventHandler>,
    pub children: Vec<RenderChild>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RenderAttribute {
    pub name: String,
    pub value: RenderAttributeValue,
    pub span: SourceSpan,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RenderAttributeValue {
    Boolean,
    Static(String),
    Expression(Option<String>),
    Spread(Option<String>),
    Unsupported,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RenderEventHandler {
    pub event: String,
    pub handler: String,
    pub span: SourceSpan,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RenderModel {
    pub root_element: Option<String>,
    pub root_span: Option<SourceSpan>,
    pub attributes: Vec<RenderAttribute>,
    pub bindings: Vec<String>,
    pub event_handlers: Vec<RenderEventHandler>,
    pub children: Vec<RenderChild>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ComponentDiagnostic {
    pub code: String,
    pub message: String,
}

#[must_use]
pub fn build_component_graph(parsed: &ParsedFile) -> ComponentGraph {
    let mut components = Vec::new();
    let mut diagnostics = Vec::new();

    for class in &parsed.classes {
        components.push(build_component_node(class, &mut diagnostics));
    }

    if parsed.classes.is_empty() && parsed.diagnostics.is_empty() {
        diagnostics.push(ComponentDiagnostic {
            code: "EZC1000".to_string(),
            message: "no component classes found".to_string(),
        });
    }

    ComponentGraph {
        components,
        diagnostics,
    }
}

fn build_component_node(
    class: &ParsedClass,
    diagnostics: &mut Vec<ComponentDiagnostic>,
) -> ComponentNode {
    let element_name = decorator_argument(class, "component");
    let route_path = decorator_argument(class, "route");

    if element_name.is_none() {
        diagnostics.push(ComponentDiagnostic {
            code: "EZC1001".to_string(),
            message: format!("class `{}` is missing @component(...)", class.name),
        });
    }

    let state_fields = class
        .properties
        .iter()
        .filter(|property| property.initializer.as_deref() == Some("state(...)"))
        .map(|property| StateField {
            name: property.name.clone(),
            initial_value: property
                .state_initial_value
                .as_ref()
                .map(serializable_value_from_parsed),
        })
        .collect::<Vec<_>>();

    let methods = class
        .methods
        .iter()
        .map(|method| ComponentMethod {
            name: method.name.clone(),
        })
        .collect::<Vec<_>>();

    let actions = class
        .methods
        .iter()
        .flat_map(|method| {
            method.state_updates.iter().map(|update| ComponentAction {
                method: method.name.clone(),
                operation: state_operation_from_parsed(&update.operation),
                field: update.field.clone(),
            })
        })
        .collect::<Vec<_>>();

    let render = class
        .methods
        .iter()
        .find(|method| method.name == "render")
        .map(|method| {
            let root = method.jsx_roots.first();

            RenderModel {
                root_element: root.map(|jsx| jsx.name.clone()),
                root_span: root.map(|jsx| jsx.span),
                attributes: root
                    .map(|jsx| {
                        jsx.attributes
                            .iter()
                            .map(render_attribute_from_parsed)
                            .collect::<Vec<_>>()
                    })
                    .unwrap_or_default(),
                event_handlers: root
                    .map(|jsx| {
                        jsx.event_handlers
                            .iter()
                            .map(render_event_handler_from_parsed)
                            .collect::<Vec<_>>()
                    })
                    .unwrap_or_default(),
                children: root
                    .map(|jsx| {
                        jsx.children
                            .iter()
                            .map(render_child_from_parsed)
                            .collect::<Vec<_>>()
                    })
                    .unwrap_or_default(),
                bindings: method.bindings.clone(),
            }
        });

    if render.is_none() {
        diagnostics.push(ComponentDiagnostic {
            code: "EZC1002".to_string(),
            message: format!("class `{}` is missing render()", class.name),
        });
    }

    if let Some(render) = &render {
        collect_render_binding_diagnostics(class, render, diagnostics);
        collect_render_event_diagnostics(class, render, diagnostics);
        collect_duplicate_event_diagnostics(render, &class.name, diagnostics);
        collect_render_attribute_diagnostics(render, &state_fields, &class.name, diagnostics);
    }

    ComponentNode {
        class_name: class.name.clone(),
        element_name,
        route_path,
        state_fields,
        methods,
        actions,
        render,
    }
}

fn collect_render_binding_diagnostics(
    class: &ParsedClass,
    render: &RenderModel,
    diagnostics: &mut Vec<ComponentDiagnostic>,
) {
    let property_names = class
        .properties
        .iter()
        .map(|property| property.name.as_str())
        .collect::<Vec<_>>();

    for binding in &render.bindings {
        if let Some(name) = this_member_name(binding) {
            if !property_names.contains(&name) {
                diagnostics.push(ComponentDiagnostic {
                    code: "EZC1003".to_string(),
                    message: format!(
                        "render binding `{binding}` references unknown field `{name}` in class `{}`",
                        class.name
                    ),
                });
            }
        }
    }
}

fn collect_render_event_diagnostics(
    class: &ParsedClass,
    render: &RenderModel,
    diagnostics: &mut Vec<ComponentDiagnostic>,
) {
    let method_names = class
        .methods
        .iter()
        .map(|method| method.name.as_str())
        .collect::<Vec<_>>();

    for event_handler in render_event_handlers(render) {
        if event_handler.event != "click" {
            diagnostics.push(ComponentDiagnostic {
                code: "EZC1005".to_string(),
                message: format!(
                    "event `{}` is not supported yet in class `{}`",
                    event_handler.event, class.name
                ),
            });
        }

        if let Some(name) = this_member_name(&event_handler.handler) {
            if !method_names.contains(&name) {
                diagnostics.push(ComponentDiagnostic {
                    code: "EZC1004".to_string(),
                    message: format!(
                        "event handler `{}` references unknown method `{name}` in class `{}`",
                        event_handler.handler, class.name
                    ),
                });
            }
        }
    }
}

fn state_operation_from_parsed(operation: &ParsedStateOperation) -> StateOperation {
    match operation {
        ParsedStateOperation::Increment => StateOperation::Increment,
        ParsedStateOperation::Decrement => StateOperation::Decrement,
        ParsedStateOperation::AddAssign(value) => {
            StateOperation::AddAssign(serializable_value_from_parsed(value))
        }
        ParsedStateOperation::SubtractAssign(value) => {
            StateOperation::SubtractAssign(serializable_value_from_parsed(value))
        }
        ParsedStateOperation::Assign(value) => {
            StateOperation::Assign(serializable_value_from_parsed(value))
        }
        ParsedStateOperation::Toggle => StateOperation::Toggle,
    }
}

fn serializable_value_from_parsed(value: &ParsedSerializableValue) -> SerializableValue {
    match value {
        ParsedSerializableValue::Null => SerializableValue::Null,
        ParsedSerializableValue::Number(value) => SerializableValue::Number(value.clone()),
        ParsedSerializableValue::String(value) => SerializableValue::String(value.clone()),
        ParsedSerializableValue::Boolean(value) => SerializableValue::Boolean(*value),
    }
}

fn decorator_argument(class: &ParsedClass, name: &str) -> Option<String> {
    class
        .decorators
        .iter()
        .find(|decorator| decorator.name == name)
        .and_then(|decorator| decorator.argument.clone())
}

fn render_child_from_parsed(child: &ParsedJsxChild) -> RenderChild {
    match child {
        ParsedJsxChild::Text { value, span } => RenderChild::Text {
            value: value.clone(),
            span: *span,
        },
        ParsedJsxChild::Binding { expression, span } => RenderChild::Binding {
            expression: expression.clone(),
            span: *span,
        },
        ParsedJsxChild::Element(element) => RenderChild::Element(RenderElement {
            tag_name: element.name.clone(),
            span: element.span,
            attributes: element
                .attributes
                .iter()
                .map(render_attribute_from_parsed)
                .collect(),
            event_handlers: element
                .event_handlers
                .iter()
                .map(render_event_handler_from_parsed)
                .collect(),
            children: element
                .children
                .iter()
                .map(render_child_from_parsed)
                .collect::<Vec<_>>(),
        }),
    }
}

fn render_attribute_from_parsed(attribute: &ParsedJsxAttribute) -> RenderAttribute {
    RenderAttribute {
        name: attribute.name.clone(),
        value: match &attribute.value {
            ParsedJsxAttributeValue::Boolean => RenderAttributeValue::Boolean,
            ParsedJsxAttributeValue::Static(value) => RenderAttributeValue::Static(value.clone()),
            ParsedJsxAttributeValue::Expression(expression) => {
                RenderAttributeValue::Expression(expression.clone())
            }
            ParsedJsxAttributeValue::Spread(expression) => {
                RenderAttributeValue::Spread(expression.clone())
            }
            ParsedJsxAttributeValue::Unsupported => RenderAttributeValue::Unsupported,
        },
        span: attribute.span,
    }
}

fn render_event_handler_from_parsed(event_handler: &ParsedEventHandler) -> RenderEventHandler {
    RenderEventHandler {
        event: event_handler.event.clone(),
        handler: event_handler.handler.clone(),
        span: event_handler.span,
    }
}

fn render_event_handlers(render: &RenderModel) -> Vec<&RenderEventHandler> {
    let mut event_handlers = render.event_handlers.iter().collect::<Vec<_>>();

    for child in &render.children {
        collect_child_event_handlers(child, &mut event_handlers);
    }

    event_handlers
}

fn collect_child_event_handlers<'a>(
    child: &'a RenderChild,
    event_handlers: &mut Vec<&'a RenderEventHandler>,
) {
    if let RenderChild::Element(element) = child {
        event_handlers.extend(element.event_handlers.iter());

        for child in &element.children {
            collect_child_event_handlers(child, event_handlers);
        }
    }
}

fn collect_duplicate_event_diagnostics(
    render: &RenderModel,
    class_name: &str,
    diagnostics: &mut Vec<ComponentDiagnostic>,
) {
    collect_duplicate_events_for_handlers(&render.event_handlers, class_name, diagnostics);

    for child in &render.children {
        collect_duplicate_child_event_diagnostics(child, class_name, diagnostics);
    }
}

fn collect_render_attribute_diagnostics(
    render: &RenderModel,
    state_fields: &[StateField],
    class_name: &str,
    diagnostics: &mut Vec<ComponentDiagnostic>,
) {
    collect_attribute_diagnostics_for_attributes(
        &render.attributes,
        state_fields,
        class_name,
        diagnostics,
    );

    for child in &render.children {
        collect_child_attribute_diagnostics(child, state_fields, class_name, diagnostics);
    }
}

fn collect_child_attribute_diagnostics(
    child: &RenderChild,
    state_fields: &[StateField],
    class_name: &str,
    diagnostics: &mut Vec<ComponentDiagnostic>,
) {
    if let RenderChild::Element(element) = child {
        collect_attribute_diagnostics_for_attributes(
            &element.attributes,
            state_fields,
            class_name,
            diagnostics,
        );

        for child in &element.children {
            collect_child_attribute_diagnostics(child, state_fields, class_name, diagnostics);
        }
    }
}

fn collect_attribute_diagnostics_for_attributes(
    attributes: &[RenderAttribute],
    state_fields: &[StateField],
    class_name: &str,
    diagnostics: &mut Vec<ComponentDiagnostic>,
) {
    let mut seen = Vec::<&str>::new();

    for attribute in attributes {
        if !attribute.name.starts_with("on") {
            if seen.contains(&attribute.name.as_str()) {
                diagnostics.push(ComponentDiagnostic {
                    code: "EZC1007".to_string(),
                    message: format!(
                        "attribute `{}` is declared more than once on the same element in class `{}`",
                        attribute.name, class_name
                    ),
                });
            } else if attribute.name != "{...}" {
                seen.push(&attribute.name);
            }
        }

        match &attribute.value {
            RenderAttributeValue::Expression(expression)
                if !is_event_attribute(&attribute.name) =>
            {
                match expression.as_deref().and_then(this_member_name) {
                    Some(field_name)
                        if state_fields.iter().any(|field| field.name == field_name) => {}
                    Some(field_name) => diagnostics.push(ComponentDiagnostic {
                        code: "EZC1003".to_string(),
                        message: format!(
                            "attribute binding `{}` references unknown state field `{field_name}` in class `{}`",
                            attribute.name, class_name
                        ),
                    }),
                    None => diagnostics.push(ComponentDiagnostic {
                        code: "EZC1008".to_string(),
                        message: format!(
                            "attribute `{}` uses an unsupported expression value in class `{}`",
                            attribute.name, class_name
                        ),
                    }),
                }
            }
            RenderAttributeValue::Spread(_) => {
                diagnostics.push(ComponentDiagnostic {
                    code: "EZC1009".to_string(),
                    message: format!(
                        "JSX spread attributes are not supported yet in class `{class_name}`"
                    ),
                });
            }
            RenderAttributeValue::Unsupported if !is_event_attribute(&attribute.name) => {
                diagnostics.push(ComponentDiagnostic {
                    code: "EZC1010".to_string(),
                    message: format!(
                        "attribute `{}` uses an unsupported JSX value in class `{}`",
                        attribute.name, class_name
                    ),
                });
            }
            _ => {}
        }
    }
}

fn is_event_attribute(name: &str) -> bool {
    name.strip_prefix("on")
        .and_then(|event| event.chars().next())
        .is_some_and(char::is_uppercase)
}

fn collect_duplicate_child_event_diagnostics(
    child: &RenderChild,
    class_name: &str,
    diagnostics: &mut Vec<ComponentDiagnostic>,
) {
    if let RenderChild::Element(element) = child {
        collect_duplicate_events_for_handlers(&element.event_handlers, class_name, diagnostics);

        for child in &element.children {
            collect_duplicate_child_event_diagnostics(child, class_name, diagnostics);
        }
    }
}

fn collect_duplicate_events_for_handlers(
    event_handlers: &[RenderEventHandler],
    class_name: &str,
    diagnostics: &mut Vec<ComponentDiagnostic>,
) {
    let mut seen = Vec::<&str>::new();

    for event_handler in event_handlers {
        if seen.contains(&event_handler.event.as_str()) {
            diagnostics.push(ComponentDiagnostic {
                code: "EZC1006".to_string(),
                message: format!(
                    "event `{}` is declared more than once on the same element in class `{}`",
                    event_handler.event, class_name
                ),
            });
        } else {
            seen.push(&event_handler.event);
        }
    }
}

fn this_member_name(reference: &str) -> Option<&str> {
    reference.strip_prefix("this.")
}
