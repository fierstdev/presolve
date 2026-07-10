use crate::component_graph::SerializableValue;
use crate::template_graph::{
    AttributeValue, ConditionalNode, ElementNode, FragmentNode, ListNode, TemplateAttribute,
    TemplateChild, TemplateGraph,
};

struct ListRenderScope<'a> {
    item_variable: &'a str,
    item: &'a SerializableValue,
    index_variable: Option<&'a str>,
    index: usize,
}

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
        if let Some(attribute_html) = generate_attribute_html(attribute) {
            html.push(' ');
            html.push_str(&attribute_html);
        }
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
    let Some(SerializableValue::Array(items)) = &list.initial_value else {
        return String::new();
    };

    items
        .iter()
        .enumerate()
        .map(|(index, item)| {
            let scope = ListRenderScope {
                item_variable: &list.item_variable,
                item,
                index_variable: list.index_variable.as_deref(),
                index,
            };
            generate_children_html_with_scope(&list.item_template, Some(&scope))
        })
        .collect()
}

fn node_id_for_scope(id: &str, scope: Option<&ListRenderScope<'_>>) -> String {
    scope.map_or_else(|| id.to_string(), |scope| format!("{id}:{}", scope.index))
}

fn binding_render_text(
    expression: &str,
    initial_value: Option<&SerializableValue>,
    scope: Option<&ListRenderScope<'_>>,
) -> String {
    if let Some(scope) = scope {
        if expression == scope.item_variable {
            return scope.item.render_text();
        }
        if scope.index_variable == Some(expression) {
            return scope.index.to_string();
        }
    }

    initial_value.map_or_else(String::new, SerializableValue::render_text)
}

fn generate_attribute_html(attribute: &TemplateAttribute) -> Option<String> {
    match &attribute.value {
        AttributeValue::Boolean => Some(attribute.name.clone()),
        AttributeValue::Binding { initial_value, .. }
            if is_boolean_attribute(&attribute.name)
                && initial_value
                    .as_ref()
                    .is_none_or(|value| value.render_text() != "true") =>
        {
            None
        }
        _ => {
            let mut html = String::new();
            html.push_str(&attribute.name);
            html.push_str("=\"");
            html.push_str(&escape_attr(&attribute_value_string(attribute)));
            html.push('"');
            Some(html)
        }
    }
}

fn attribute_value_string(attribute: &TemplateAttribute) -> String {
    match &attribute.value {
        AttributeValue::Boolean => String::new(),
        AttributeValue::Static(value) => value.clone(),
        AttributeValue::Binding { initial_value, .. } => initial_value
            .as_ref()
            .map(SerializableValue::render_text)
            .unwrap_or_default(),
        AttributeValue::EventHandler { handler, .. } => handler.clone(),
        AttributeValue::BindingList(bindings) => bindings.join(","),
    }
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
