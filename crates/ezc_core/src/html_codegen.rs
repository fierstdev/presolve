use crate::component_graph::SerializableValue;
use crate::template_graph::{
    AttributeValue, ConditionalNode, ElementNode, FragmentNode, ListNode, TemplateAttribute,
    TemplateChild, TemplateGraph,
};

struct ListRenderScope<'a> {
    item_variable: &'a str,
    item: Option<&'a SerializableValue>,
    index_variable: Option<&'a str>,
    index: Option<usize>,
    instance_key: &'a str,
}

const LIST_INDEX_TOKEN: &str = "__ez_list_index__";
const LIST_ITEM_TOKEN: &str = "__ez_list_item__";
const LIST_KEY_TOKEN: &str = "__ez_list_key__";

#[must_use]
pub fn generate_static_html(template_graph: &TemplateGraph) -> String {
    let mut output = String::new();

    for template in &template_graph.templates {
        if let Some(root) = &template.root {
            output.push_str(&generate_element_html(root, None));
            output.push('\n');
        } else if let Some(fragment) = &template.root_fragment {
            output.push_str(&generate_fragment_html(fragment, None));
            output.push('\n');
        }
    }

    output
}

fn generate_fragment_html(fragment: &FragmentNode, scope: Option<&ListRenderScope<'_>>) -> String {
    generate_children_html_with_scope(&fragment.children, scope)
}

pub(crate) fn generate_children_html(children: &[TemplateChild]) -> String {
    generate_children_html_with_scope(children, None)
}

fn generate_children_html_with_scope(
    children: &[TemplateChild],
    scope: Option<&ListRenderScope<'_>>,
) -> String {
    let mut html = String::new();

    for child in children {
        html.push_str(&generate_child_html(child, scope));
    }

    html
}

fn generate_element_html(element: &ElementNode, scope: Option<&ListRenderScope<'_>>) -> String {
    let mut html = String::new();

    html.push('<');
    html.push_str(&element.tag_name);

    html.push_str(" data-ez-node=\"");
    html.push_str(&escape_attr(&node_id_for_scope(&element.id.0, scope)));
    html.push('"');

    for attribute in &element.attributes {
        if let Some(attribute_html) = generate_attribute_html(attribute, scope) {
            html.push(' ');
            html.push_str(&attribute_html);
        }
    }

    if let Some(list_attribute_bindings) = list_attribute_bindings(&element.attributes, scope) {
        html.push_str(" data-ez-list-bindings=\"");
        html.push_str(&escape_attr(&list_attribute_bindings));
        html.push('"');
    }

    html.push('>');

    for child in &element.children {
        html.push_str(&generate_child_html(child, scope));
    }

    html.push_str("</");
    html.push_str(&element.tag_name);
    html.push('>');

    html
}

fn generate_child_html(child: &TemplateChild, scope: Option<&ListRenderScope<'_>>) -> String {
    match child {
        TemplateChild::Text { value, .. } => escape_text(value),
        TemplateChild::Binding {
            id,
            expression,
            initial_value,
            ..
        } => {
            let mut html = String::new();

            html.push_str("<!-- ez-binding:");
            html.push_str(&escape_comment(&node_id_for_scope(&id.0, scope)));
            html.push(':');
            html.push_str(&escape_comment(expression));
            html.push_str(" -->");

            html.push_str(&escape_text(&binding_render_text(
                expression,
                initial_value.as_ref(),
                scope,
            )));

            if scope.is_some() {
                html.push_str("<!-- ez-list-binding-end:");
                html.push_str(&escape_comment(&node_id_for_scope(&id.0, scope)));
                html.push_str(" -->");
            }

            html
        }
        TemplateChild::Element(element) => generate_element_html(element, scope),
        TemplateChild::Fragment(fragment) => generate_fragment_html(fragment, scope),
        TemplateChild::Conditional(conditional) => generate_conditional_html(conditional, scope),
        TemplateChild::List(list) => generate_list_html(list),
    }
}

fn generate_conditional_html(
    conditional: &ConditionalNode,
    scope: Option<&ListRenderScope<'_>>,
) -> String {
    let mut html = String::new();

    html.push_str("<!-- ez-conditional-start:");
    html.push_str(&escape_comment(&node_id_for_scope(
        &conditional.start_id.0,
        scope,
    )));
    html.push(':');
    html.push_str(&escape_comment(&conditional.condition));
    html.push_str(" -->");

    let children = match conditional.initial_value {
        Some(SerializableValue::Boolean(true)) => &conditional.when_true,
        _ => &conditional.when_false,
    };

    html.push_str(&generate_children_html_with_scope(children, scope));
    html.push_str("<!-- ez-conditional-end:");
    html.push_str(&escape_comment(&node_id_for_scope(
        &conditional.end_id.0,
        scope,
    )));
    html.push_str(" -->");

    html
}

fn generate_list_html(list: &ListNode) -> String {
    let mut html = String::new();
    html.push_str("<!-- ez-list-start:");
    html.push_str(&escape_comment(&list.start_id.0));
    html.push(':');
    html.push_str(&escape_comment(&list.iterable));
    html.push_str(" -->");

    if let Some(SerializableValue::Array(items)) = &list.initial_value {
        for (index, item) in items.iter().enumerate() {
            let instance_key = list_instance_key(list, item, index);
            let scope = ListRenderScope {
                item_variable: &list.item_variable,
                item: Some(item),
                index_variable: list.index_variable.as_deref(),
                index: Some(index),
                instance_key: &instance_key,
            };
            html.push_str(&generate_children_html_with_scope(
                &list.item_template,
                Some(&scope),
            ));
        }
    }

    html.push_str("<!-- ez-list-end:");
    html.push_str(&escape_comment(&list.end_id.0));
    html.push_str(" -->");
    html
}

pub(crate) fn generate_list_item_template_html(list: &ListNode) -> String {
    let scope = ListRenderScope {
        item_variable: &list.item_variable,
        item: None,
        index_variable: list.index_variable.as_deref(),
        index: None,
        instance_key: LIST_KEY_TOKEN,
    };

    generate_children_html_with_scope(&list.item_template, Some(&scope))
}

fn node_id_for_scope(id: &str, scope: Option<&ListRenderScope<'_>>) -> String {
    scope.map_or_else(
        || id.to_string(),
        |scope| format!("{id}:{}", scope.instance_key),
    )
}

fn binding_render_text(
    expression: &str,
    initial_value: Option<&SerializableValue>,
    scope: Option<&ListRenderScope<'_>>,
) -> String {
    binding_render_value(expression, initial_value, scope)
}

fn binding_render_value(
    expression: &str,
    initial_value: Option<&SerializableValue>,
    scope: Option<&ListRenderScope<'_>>,
) -> String {
    if let Some(scope) = scope {
        if expression == scope.item_variable {
            return scope.item.map_or_else(
                || LIST_ITEM_TOKEN.to_string(),
                SerializableValue::render_text,
            );
        }
        if let Some(value) = list_item_member_value(expression, scope) {
            return value.render_text();
        }
        if scope.index_variable == Some(expression) {
            return scope
                .index
                .map_or_else(|| LIST_INDEX_TOKEN.to_string(), |index| index.to_string());
        }
    }

    initial_value.map_or_else(String::new, SerializableValue::render_text)
}

fn list_instance_key(list: &ListNode, item: &SerializableValue, index: usize) -> String {
    if list.key_expression == list.item_variable {
        return serializable_list_key(item).unwrap_or_else(|| index.to_string());
    }

    if let Some(path) = list_member_key_path(list) {
        return item
            .member_path_value(path)
            .and_then(serializable_list_key)
            .unwrap_or_else(|| index.to_string());
    }

    if list.index_variable.as_deref() == Some(list.key_expression.as_str()) {
        return index.to_string();
    }

    index.to_string()
}

fn list_item_member_value<'a>(
    expression: &str,
    scope: &'a ListRenderScope<'a>,
) -> Option<&'a SerializableValue> {
    let path = expression
        .strip_prefix(scope.item_variable)?
        .strip_prefix('.')?;

    scope.item?.member_path_value(path)
}

fn list_member_key_path(list: &ListNode) -> Option<&str> {
    list.key_expression
        .strip_prefix(&list.item_variable)
        .and_then(|suffix| suffix.strip_prefix('.'))
        .filter(|path| !path.is_empty() && !path.split('.').any(str::is_empty))
}

fn serializable_list_key(value: &SerializableValue) -> Option<String> {
    match value {
        SerializableValue::Null => Some("null".to_string()),
        SerializableValue::Number(value) | SerializableValue::String(value) => Some(value.clone()),
        SerializableValue::Boolean(value) => Some(value.to_string()),
        SerializableValue::Array(_) | SerializableValue::Object(_) => None,
    }
}

fn generate_attribute_html(
    attribute: &TemplateAttribute,
    scope: Option<&ListRenderScope<'_>>,
) -> Option<String> {
    match &attribute.value {
        AttributeValue::Boolean => Some(attribute.name.clone()),
        AttributeValue::Binding {
            expression,
            initial_value,
            ..
        } if is_boolean_attribute(&attribute.name)
            && binding_render_value(expression, initial_value.as_ref(), scope) != "true" =>
        {
            None
        }
        _ => {
            let mut html = String::new();
            html.push_str(&attribute.name);
            html.push_str("=\"");
            html.push_str(&escape_attr(&attribute_value_string(attribute, scope)));
            html.push('"');
            Some(html)
        }
    }
}

fn attribute_value_string(
    attribute: &TemplateAttribute,
    scope: Option<&ListRenderScope<'_>>,
) -> String {
    match &attribute.value {
        AttributeValue::Boolean => String::new(),
        AttributeValue::Static(value) => value.clone(),
        AttributeValue::Binding {
            expression,
            initial_value,
            ..
        } => binding_render_value(expression, initial_value.as_ref(), scope),
        AttributeValue::EventHandler { handler, .. } => handler.clone(),
        AttributeValue::BindingList(bindings) => bindings.join(","),
    }
}

fn list_attribute_bindings(
    attributes: &[TemplateAttribute],
    scope: Option<&ListRenderScope<'_>>,
) -> Option<String> {
    let scope = scope?;
    let bindings = attributes
        .iter()
        .filter_map(|attribute| {
            let AttributeValue::Binding { expression, .. } = &attribute.value else {
                return None;
            };

            list_item_binding_expression(expression, scope)
                .then(|| format!("{}={expression}", attribute.name))
        })
        .collect::<Vec<_>>();

    (!bindings.is_empty()).then(|| bindings.join(";"))
}

fn list_item_binding_expression(expression: &str, scope: &ListRenderScope<'_>) -> bool {
    expression == scope.item_variable
        || scope.index_variable == Some(expression)
        || expression
            .strip_prefix(scope.item_variable)
            .and_then(|suffix| suffix.strip_prefix('.'))
            .is_some_and(|path| !path.is_empty() && !path.split('.').any(str::is_empty))
}

fn is_boolean_attribute(name: &str) -> bool {
    matches!(
        name,
        "allowfullscreen"
            | "async"
            | "autofocus"
            | "autoplay"
            | "checked"
            | "controls"
            | "default"
            | "defer"
            | "disabled"
            | "formnovalidate"
            | "hidden"
            | "inert"
            | "loop"
            | "multiple"
            | "muted"
            | "nomodule"
            | "novalidate"
            | "open"
            | "readonly"
            | "required"
            | "reversed"
            | "selected"
    )
}

fn escape_attr(value: &str) -> String {
    let mut output = String::new();

    for ch in value.chars() {
        match ch {
            '&' => output.push_str("&amp;"),
            '"' => output.push_str("&quot;"),
            '<' => output.push_str("&lt;"),
            '>' => output.push_str("&gt;"),
            ch => output.push(ch),
        }
    }

    output
}

fn escape_text(value: &str) -> String {
    let mut output = String::new();

    for ch in value.chars() {
        match ch {
            '&' => output.push_str("&amp;"),
            '<' => output.push_str("&lt;"),
            '>' => output.push_str("&gt;"),
            ch => output.push(ch),
        }
    }

    output
}

fn escape_comment(value: &str) -> String {
    value.replace("--", "—")
}

#[cfg(test)]
mod tests {
    use super::escape_attr;

    #[test]
    fn escapes_html_attributes() {
        assert_eq!(
            escape_attr(r#"this.value<&">"#),
            "this.value&lt;&amp;&quot;&gt;"
        );
    }
}
