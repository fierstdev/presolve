use crate::component_graph::SerializableValue;
use crate::template_graph::{
    AttributeValue, ElementNode, FragmentNode, TemplateAttribute, TemplateChild, TemplateGraph,
};

#[must_use]
pub fn generate_static_html(template_graph: &TemplateGraph) -> String {
    let mut output = String::new();

    for template in &template_graph.templates {
        if let Some(root) = &template.root {
            output.push_str(&generate_element_html(root));
            output.push('\n');
        } else if let Some(fragment) = &template.root_fragment {
            output.push_str(&generate_fragment_html(fragment));
            output.push('\n');
        }
    }

    output
}

fn generate_fragment_html(fragment: &FragmentNode) -> String {
    let mut html = String::new();

    for child in &fragment.children {
        html.push_str(&generate_child_html(child));
    }

    html
}

fn generate_element_html(element: &ElementNode) -> String {
    let mut html = String::new();

    html.push('<');
    html.push_str(&element.tag_name);

    html.push_str(" data-ez-node=\"");
    html.push_str(&escape_attr(&element.id.0));
    html.push('"');

    for attribute in &element.attributes {
        if let Some(attribute_html) = generate_attribute_html(attribute) {
            html.push(' ');
            html.push_str(&attribute_html);
        }
    }

    html.push('>');

    for child in &element.children {
        html.push_str(&generate_child_html(child));
    }

    html.push_str("</");
    html.push_str(&element.tag_name);
    html.push('>');

    html
}

fn generate_child_html(child: &TemplateChild) -> String {
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
            html.push_str(&escape_comment(&id.0));
            html.push(':');
            html.push_str(&escape_comment(expression));
            html.push_str(" -->");

            if let Some(initial_value) = initial_value {
                html.push_str(&escape_text(&initial_value.render_text()));
            }

            html
        }
        TemplateChild::Element(element) => generate_element_html(element),
        TemplateChild::Fragment(fragment) => generate_fragment_html(fragment),
    }
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
