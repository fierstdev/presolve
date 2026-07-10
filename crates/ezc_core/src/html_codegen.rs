use crate::template_graph::{
    AttributeValue, ElementNode, TemplateAttribute, TemplateChild, TemplateGraph,
};

pub fn generate_static_html(template_graph: &TemplateGraph) -> String {
    let mut output = String::new();

    for template in &template_graph.templates {
        if let Some(root) = &template.root {
            output.push_str(&generate_element_html(root));
            output.push('\n');
        }
    }

    output
}

fn generate_element_html(element: &ElementNode) -> String {
    let mut html = String::new();

    html.push('<');
    html.push_str(&element.tag_name);

    html.push_str(" data-ez-node=\"");
    html.push_str(&escape_attr(&element.id.0));
    html.push('"');

    for attribute in &element.attributes {
        html.push(' ');
        html.push_str(&attribute.name);
        html.push_str("=\"");
        html.push_str(&escape_attr(&attribute_value_string(attribute)));
        html.push('"');
    }

    html.push('>');

    for child in &element.children {
        match child {
            TemplateChild::Text(text) => html.push_str(&escape_text(text)),
            TemplateChild::Binding {
                id,
                expression,
                initial_value,
            } => {
                html.push_str("<!-- ez-binding:");
                html.push_str(&escape_comment(&id.0));
                html.push(':');
                html.push_str(&escape_comment(expression));
                html.push_str(" -->");

                if let Some(initial_value) = initial_value {
                    html.push_str(&escape_text(initial_value));
                }
            }
            TemplateChild::Element(element) => {
                html.push_str(&generate_element_html(element));
            }
        }
    }

    html.push_str("</");
    html.push_str(&element.tag_name);
    html.push('>');

    html
}

fn attribute_value_string(attribute: &TemplateAttribute) -> String {
    match &attribute.value {
        AttributeValue::Static(value) => value.clone(),
        AttributeValue::EventHandler { handler, .. } => handler.clone(),
        AttributeValue::BindingList(bindings) => bindings.join(","),
    }
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
