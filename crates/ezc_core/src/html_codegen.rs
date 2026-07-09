use crate::component_graph::{ComponentGraph, ComponentNode, RenderChild, RenderModel};

pub fn generate_static_html(graph: &ComponentGraph) -> String {
    let mut output = String::new();

    for component in &graph.components {
        if let Some(html) = generate_component_html(component) {
            output.push_str(&html);
            output.push('\n');
        }
    }

    output
}

fn generate_component_html(component: &ComponentNode) -> Option<String> {
    let render = component.render.as_ref()?;
    generate_render_html(render)
}

fn generate_render_html(render: &RenderModel) -> Option<String> {
    let root = render.root_element.as_ref()?;

    let mut html = String::new();

    html.push('<');
    html.push_str(root);

    if !render.event_handler_refs.is_empty() {
        html.push_str(" data-ez-event-handlers=\"");
        html.push_str(&escape_attr(&render.event_handler_refs.join(",")));
        html.push('"');
    }

    if !render.bindings.is_empty() {
        html.push_str(" data-ez-bindings=\"");
        html.push_str(&escape_attr(&render.bindings.join(",")));
        html.push('"');
    }

    html.push('>');

    for child in &render.children {
        match child {
            RenderChild::Text(text) => html.push_str(&escape_text(text)),
            RenderChild::Binding(binding) => {
                html.push_str("<!-- binding:");
                html.push_str(&escape_comment(binding));
                html.push_str(" -->");
            }
        }
    }

    html.push_str("</");
    html.push_str(root);
    html.push('>');

    Some(html)
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
