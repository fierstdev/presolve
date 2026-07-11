use std::collections::BTreeMap;

use serde::Serialize;

use crate::semantic_id::{SemanticId, SemanticOwner};

use ezc_parser::{
    ParsedClass, ParsedEventHandler, ParsedFile, ParsedJsxAttribute, ParsedJsxAttributeValue,
    ParsedJsxChild, ParsedJsxConditional, ParsedJsxFragment, ParsedJsxList, ParsedJsxNode,
    ParsedSerializableValue, ParsedStateOperation, SourceSpan,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ComponentGraph {
    pub components: Vec<ComponentNode>,
    pub diagnostics: Vec<ComponentDiagnostic>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ComponentNode {
    pub id: SemanticId,
    pub owner: SemanticOwner,
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
    pub id: SemanticId,
    pub owner: SemanticOwner,
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
    Array(Vec<SerializableValue>),
    Object(BTreeMap<String, SerializableValue>),
}

impl SerializableValue {
    #[must_use]
    pub fn render_text(&self) -> String {
        match self {
            Self::Number(value) | Self::String(value) => value.clone(),
            Self::Boolean(value) => value.to_string(),
            Self::Null | Self::Array(_) | Self::Object(_) => String::new(),
        }
    }

    #[must_use]
    pub fn member_path_value(&self, path: &str) -> Option<&Self> {
        if path.is_empty() {
            return None;
        }

        path.split('.').try_fold(self, |value, member| match value {
            Self::Object(values) if !member.is_empty() => values.get(member),
            _ => None,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ComponentMethod {
    pub id: SemanticId,
    pub owner: SemanticOwner,
    pub name: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ComponentAction {
    pub id: SemanticId,
    pub owner: SemanticOwner,
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
    Fragment(RenderFragment),
    Conditional(RenderConditional),
    List(RenderList),
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
pub struct RenderFragment {
    pub span: SourceSpan,
    pub children: Vec<RenderChild>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RenderConditional {
    pub condition: String,
    pub span: SourceSpan,
    pub when_true: Vec<RenderChild>,
    pub when_false: Vec<RenderChild>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RenderList {
    pub iterable: String,
    pub item_variable: String,
    pub index_variable: Option<String>,
    pub key_expression: String,
    pub span: SourceSpan,
    pub item_template: Vec<RenderChild>,
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
    pub root_fragment: Option<RenderFragment>,
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
    let id = SemanticId::component(element_name.as_deref(), &class.name);

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
            id: id.state_field(&property.name),
            owner: SemanticOwner::entity(id.clone()),
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
            id: id.method(&method.name),
            owner: SemanticOwner::entity(id.clone()),
            name: method.name.clone(),
        })
        .collect::<Vec<_>>();

    let actions = class
        .methods
        .iter()
        .flat_map(|method| {
            method
                .state_updates
                .iter()
                .enumerate()
                .map(|(index, update)| ComponentAction {
                    id: id.action(&method.name, index),
                    owner: SemanticOwner::entity(id.method(&method.name)),
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
        .map(render_model_from_parsed_method);

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
        collect_render_list_diagnostics(render, &state_fields, &class.name, diagnostics);
    }

    ComponentNode {
        id,
        owner: SemanticOwner::Application,
        class_name: class.name.clone(),
        element_name,
        route_path,
        state_fields,
        methods,
        actions,
        render,
    }
}

fn render_model_from_parsed_method(method: &ezc_parser::ParsedMethod) -> RenderModel {
    let root = method.jsx_roots.first();
    let root_element = root.and_then(parsed_root_element);
    let root_fragment = root.and_then(parsed_root_fragment);

    RenderModel {
        root_element: root_element.map(|element| element.name.clone()),
        root_span: root_element.map(|element| element.span),
        root_fragment: root_fragment.map(render_fragment_from_parsed),
        attributes: root_element.map_or_else(Vec::new, |element| {
            element
                .attributes
                .iter()
                .map(render_attribute_from_parsed)
                .collect()
        }),
        event_handlers: root_element.map_or_else(Vec::new, |element| {
            element
                .event_handlers
                .iter()
                .map(render_event_handler_from_parsed)
                .collect()
        }),
        children: root_element.map_or_else(Vec::new, |element| {
            element
                .children
                .iter()
                .map(render_child_from_parsed)
                .collect()
        }),
        bindings: method.bindings.clone(),
    }
}

fn parsed_root_element(root: &ParsedJsxNode) -> Option<&ezc_parser::ParsedJsxElement> {
    match root {
        ParsedJsxNode::Element(element) => Some(element),
        ParsedJsxNode::Fragment(_) => None,
    }
}

fn parsed_root_fragment(root: &ParsedJsxNode) -> Option<&ParsedJsxFragment> {
    match root {
        ParsedJsxNode::Element(_) => None,
        ParsedJsxNode::Fragment(fragment) => Some(fragment),
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
        ParsedSerializableValue::Array(values) => {
            SerializableValue::Array(values.iter().map(serializable_value_from_parsed).collect())
        }
        ParsedSerializableValue::Object(values) => SerializableValue::Object(
            values
                .iter()
                .map(|(key, value)| (key.clone(), serializable_value_from_parsed(value)))
                .collect(),
        ),
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
        ParsedJsxChild::Fragment(fragment) => {
            RenderChild::Fragment(render_fragment_from_parsed(fragment))
        }
        ParsedJsxChild::Conditional(conditional) => {
            RenderChild::Conditional(render_conditional_from_parsed(conditional))
        }
        ParsedJsxChild::List(list) => RenderChild::List(render_list_from_parsed(list)),
    }
}

fn render_list_from_parsed(list: &ParsedJsxList) -> RenderList {
    RenderList {
        iterable: list.iterable.clone(),
        item_variable: list.item_variable.clone(),
        index_variable: list.index_variable.clone(),
        key_expression: list.key_expression.clone(),
        span: list.span,
        item_template: render_children_from_parsed_node(&list.item_template),
    }
}

fn render_conditional_from_parsed(conditional: &ParsedJsxConditional) -> RenderConditional {
    RenderConditional {
        condition: conditional.condition.clone(),
        span: conditional.span,
        when_true: render_children_from_parsed_node(&conditional.when_true),
        when_false: conditional
            .when_false
            .as_ref()
            .map(render_children_from_parsed_node)
            .unwrap_or_default(),
    }
}

fn render_children_from_parsed_node(node: &ParsedJsxNode) -> Vec<RenderChild> {
    match node {
        ParsedJsxNode::Element(element) => vec![RenderChild::Element(RenderElement {
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
        })],
        ParsedJsxNode::Fragment(fragment) => fragment
            .children
            .iter()
            .map(render_child_from_parsed)
            .collect(),
    }
}

fn render_fragment_from_parsed(fragment: &ParsedJsxFragment) -> RenderFragment {
    RenderFragment {
        span: fragment.span,
        children: fragment
            .children
            .iter()
            .map(render_child_from_parsed)
            .collect(),
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
    if let Some(fragment) = &render.root_fragment {
        for child in &fragment.children {
            collect_child_event_handlers(child, &mut event_handlers);
        }
    }

    event_handlers
}

fn collect_child_event_handlers<'a>(
    child: &'a RenderChild,
    event_handlers: &mut Vec<&'a RenderEventHandler>,
) {
    match child {
        RenderChild::Element(element) => {
            event_handlers.extend(element.event_handlers.iter());

            for child in &element.children {
                collect_child_event_handlers(child, event_handlers);
            }
        }
        RenderChild::Fragment(fragment) => {
            for child in &fragment.children {
                collect_child_event_handlers(child, event_handlers);
            }
        }
        RenderChild::Conditional(conditional) => {
            for child in &conditional.when_true {
                collect_child_event_handlers(child, event_handlers);
            }
            for child in &conditional.when_false {
                collect_child_event_handlers(child, event_handlers);
            }
        }
        RenderChild::List(list) => {
            for child in &list.item_template {
                collect_child_event_handlers(child, event_handlers);
            }
        }
        RenderChild::Text { .. } | RenderChild::Binding { .. } => {}
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
    if let Some(fragment) = &render.root_fragment {
        for child in &fragment.children {
            collect_duplicate_child_event_diagnostics(child, class_name, diagnostics);
        }
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
        None,
    );

    for child in &render.children {
        collect_child_attribute_diagnostics(child, state_fields, class_name, diagnostics);
    }
    if let Some(fragment) = &render.root_fragment {
        for child in &fragment.children {
            collect_child_attribute_diagnostics(child, state_fields, class_name, diagnostics);
        }
    }
}

fn collect_render_list_diagnostics(
    render: &RenderModel,
    state_fields: &[StateField],
    class_name: &str,
    diagnostics: &mut Vec<ComponentDiagnostic>,
) {
    for child in &render.children {
        collect_child_list_diagnostics(child, state_fields, class_name, diagnostics);
    }
    if let Some(fragment) = &render.root_fragment {
        for child in &fragment.children {
            collect_child_list_diagnostics(child, state_fields, class_name, diagnostics);
        }
    }
}

fn collect_child_list_diagnostics(
    child: &RenderChild,
    state_fields: &[StateField],
    class_name: &str,
    diagnostics: &mut Vec<ComponentDiagnostic>,
) {
    match child {
        RenderChild::Element(element) => {
            for child in &element.children {
                collect_child_list_diagnostics(child, state_fields, class_name, diagnostics);
            }
        }
        RenderChild::Fragment(fragment) => {
            for child in &fragment.children {
                collect_child_list_diagnostics(child, state_fields, class_name, diagnostics);
            }
        }
        RenderChild::Conditional(conditional) => {
            for child in &conditional.when_true {
                collect_child_list_diagnostics(child, state_fields, class_name, diagnostics);
            }
            for child in &conditional.when_false {
                collect_child_list_diagnostics(child, state_fields, class_name, diagnostics);
            }
        }
        RenderChild::List(list) => {
            collect_list_diagnostics(list, state_fields, class_name, diagnostics);

            for child in &list.item_template {
                collect_child_list_diagnostics(child, state_fields, class_name, diagnostics);
            }
        }
        RenderChild::Text { .. } | RenderChild::Binding { .. } => {}
    }
}

fn collect_list_diagnostics(
    list: &RenderList,
    state_fields: &[StateField],
    class_name: &str,
    diagnostics: &mut Vec<ComponentDiagnostic>,
) {
    if list.key_expression.is_empty() {
        diagnostics.push(ComponentDiagnostic {
            code: "EZC1011".to_string(),
            message: format!(
                "list over `{}` in class `{class_name}` is missing a `key={{...}}` attribute; stable keys are required for retained-node reconciliation",
                list.iterable
            ),
        });
        return;
    }

    if list.index_variable.as_deref() == Some(list.key_expression.as_str()) {
        diagnostics.push(ComponentDiagnostic {
            code: "EZC1012".to_string(),
            message: format!(
                "list key `{}` in class `{class_name}` uses the iteration index; index keys are unstable when items move",
                list.key_expression
            ),
        });
        return;
    }

    let member_path = list_member_key_path(list);
    if list.key_expression != list.item_variable && member_path.is_none() {
        diagnostics.push(ComponentDiagnostic {
            code: "EZC1013".to_string(),
            message: format!(
                "list key `{}` in class `{class_name}` is not supported yet; use the item variable `{}` or one of its object members",
                list.key_expression, list.item_variable
            ),
        });
        return;
    }

    let Some(field_name) = this_member_name(&list.iterable) else {
        return;
    };
    let Some(SerializableValue::Array(values)) = state_fields
        .iter()
        .find(|field| field.name == field_name)
        .and_then(|field| field.initial_value.as_ref())
    else {
        return;
    };

    let mut keys = Vec::new();
    for (index, value) in values.iter().enumerate() {
        let key_value = member_path.map_or(Some(value), |path| value.member_path_value(path));
        let Some(key) = key_value.and_then(list_key_from_static_value) else {
            diagnostics.push(ComponentDiagnostic {
                code: "EZC1015".to_string(),
                message: member_path.map_or_else(
                    || format!(
                        "list key `{}` resolves to a non-primitive initial item at index {index} in class `{class_name}`; keyed reconciliation requires primitive keys",
                        list.key_expression
                    ),
                    |_| format!(
                        "list key `{}` cannot resolve a primitive member value for initial item at index {index} in class `{class_name}`; every item must provide that member",
                        list.key_expression
                    ),
                ),
            });
            return;
        };

        if keys.contains(&key) {
            diagnostics.push(ComponentDiagnostic {
                code: "EZC1014".to_string(),
                message: format!(
                    "list key `{}` resolves to duplicate initial value `{key}` in class `{class_name}`; keyed reconciliation requires unique keys",
                    list.key_expression
                ),
            });
            return;
        }

        keys.push(key);
    }
}

fn list_member_key_path(list: &RenderList) -> Option<&str> {
    list.key_expression
        .strip_prefix(&list.item_variable)
        .and_then(|suffix| suffix.strip_prefix('.'))
        .filter(|path| !path.is_empty() && !path.split('.').any(str::is_empty))
}

fn list_key_from_static_value(value: &SerializableValue) -> Option<String> {
    match value {
        SerializableValue::Null => Some("null".to_string()),
        SerializableValue::Number(value) | SerializableValue::String(value) => Some(value.clone()),
        SerializableValue::Boolean(value) => Some(value.to_string()),
        SerializableValue::Array(_) | SerializableValue::Object(_) => None,
    }
}

fn collect_child_attribute_diagnostics(
    child: &RenderChild,
    state_fields: &[StateField],
    class_name: &str,
    diagnostics: &mut Vec<ComponentDiagnostic>,
) {
    match child {
        RenderChild::Element(element) => {
            collect_attribute_diagnostics_for_attributes(
                &element.attributes,
                state_fields,
                class_name,
                diagnostics,
                None,
            );

            for child in &element.children {
                collect_child_attribute_diagnostics(child, state_fields, class_name, diagnostics);
            }
        }
        RenderChild::Fragment(fragment) => {
            for child in &fragment.children {
                collect_child_attribute_diagnostics(child, state_fields, class_name, diagnostics);
            }
        }
        RenderChild::Conditional(conditional) => {
            for child in &conditional.when_true {
                collect_child_attribute_diagnostics(child, state_fields, class_name, diagnostics);
            }
            for child in &conditional.when_false {
                collect_child_attribute_diagnostics(child, state_fields, class_name, diagnostics);
            }
        }
        RenderChild::List(list) => {
            for child in &list.item_template {
                collect_list_item_attribute_diagnostics(
                    child,
                    state_fields,
                    class_name,
                    diagnostics,
                    &list.item_variable,
                    list.index_variable.as_deref(),
                );
            }
        }
        RenderChild::Text { .. } | RenderChild::Binding { .. } => {}
    }
}

fn collect_attribute_diagnostics_for_attributes(
    attributes: &[RenderAttribute],
    state_fields: &[StateField],
    class_name: &str,
    diagnostics: &mut Vec<ComponentDiagnostic>,
    list_scope: Option<(&str, Option<&str>)>,
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
            RenderAttributeValue::Expression(_) if attribute.name == "key" => {}
            RenderAttributeValue::Expression(expression)
                if !is_event_attribute(&attribute.name) =>
            {
                if expression.as_deref().is_some_and(|expression| {
                    list_scope
                        .is_some_and(|scope| list_item_attribute_expression(expression, scope))
                }) {
                    continue;
                }

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

fn collect_list_item_attribute_diagnostics(
    child: &RenderChild,
    state_fields: &[StateField],
    class_name: &str,
    diagnostics: &mut Vec<ComponentDiagnostic>,
    item_variable: &str,
    index_variable: Option<&str>,
) {
    match child {
        RenderChild::Element(element) => {
            collect_attribute_diagnostics_for_attributes(
                &element.attributes,
                state_fields,
                class_name,
                diagnostics,
                Some((item_variable, index_variable)),
            );

            for child in &element.children {
                collect_list_item_attribute_diagnostics(
                    child,
                    state_fields,
                    class_name,
                    diagnostics,
                    item_variable,
                    index_variable,
                );
            }
        }
        RenderChild::Fragment(fragment) => {
            for child in &fragment.children {
                collect_list_item_attribute_diagnostics(
                    child,
                    state_fields,
                    class_name,
                    diagnostics,
                    item_variable,
                    index_variable,
                );
            }
        }
        RenderChild::Conditional(conditional) => {
            for child in &conditional.when_true {
                collect_list_item_attribute_diagnostics(
                    child,
                    state_fields,
                    class_name,
                    diagnostics,
                    item_variable,
                    index_variable,
                );
            }
            for child in &conditional.when_false {
                collect_list_item_attribute_diagnostics(
                    child,
                    state_fields,
                    class_name,
                    diagnostics,
                    item_variable,
                    index_variable,
                );
            }
        }
        RenderChild::List(list) => {
            for child in &list.item_template {
                collect_list_item_attribute_diagnostics(
                    child,
                    state_fields,
                    class_name,
                    diagnostics,
                    &list.item_variable,
                    list.index_variable.as_deref(),
                );
            }
        }
        RenderChild::Text { .. } | RenderChild::Binding { .. } => {}
    }
}

fn list_item_attribute_expression(expression: &str, scope: (&str, Option<&str>)) -> bool {
    expression == scope.0
        || scope.1 == Some(expression)
        || expression
            .strip_prefix(scope.0)
            .and_then(|suffix| suffix.strip_prefix('.'))
            .is_some_and(|path| !path.is_empty() && !path.split('.').any(str::is_empty))
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
    match child {
        RenderChild::Element(element) => {
            collect_duplicate_events_for_handlers(&element.event_handlers, class_name, diagnostics);

            for child in &element.children {
                collect_duplicate_child_event_diagnostics(child, class_name, diagnostics);
            }
        }
        RenderChild::Fragment(fragment) => {
            for child in &fragment.children {
                collect_duplicate_child_event_diagnostics(child, class_name, diagnostics);
            }
        }
        RenderChild::Conditional(conditional) => {
            for child in &conditional.when_true {
                collect_duplicate_child_event_diagnostics(child, class_name, diagnostics);
            }
            for child in &conditional.when_false {
                collect_duplicate_child_event_diagnostics(child, class_name, diagnostics);
            }
        }
        RenderChild::List(list) => {
            for child in &list.item_template {
                collect_duplicate_child_event_diagnostics(child, class_name, diagnostics);
            }
        }
        RenderChild::Text { .. } | RenderChild::Binding { .. } => {}
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
