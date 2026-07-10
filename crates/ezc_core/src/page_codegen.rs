use crate::template_manifest::{template_manifest_json, TemplateManifest};

#[must_use]
pub fn generate_standalone_page(
    title: &str,
    body_html: &str,
    manifest: &TemplateManifest,
) -> String {
    let manifest_json = template_manifest_json(manifest);

    let mut output = String::new();

    output.push_str("<!doctype html>\n");
    output.push_str("<html lang=\"en\">\n");
    output.push_str("  <head>\n");
    output.push_str("    <meta charset=\"utf-8\">\n");
    output.push_str("    <title>");
    output.push_str(&escape_text(title));
    output.push_str("</title>\n");
    output.push_str("  </head>\n");
    output.push_str("  <body>\n");

    for line in body_html.lines() {
        output.push_str("    ");
        output.push_str(line);
        output.push('\n');
    }

    output.push_str("    <script type=\"application/json\" id=\"ez-template-manifest\">\n");

    for line in manifest_json.lines() {
        output.push_str("      ");
        output.push_str(&escape_script_json_line(line));
        output.push('\n');
    }

    output.push_str("    </script>\n");
    output.push_str("    <script src=\"./runtime.js\" defer></script>\n");
    output.push_str("  </body>\n");
    output.push_str("</html>\n");

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

fn escape_script_json_line(value: &str) -> String {
    value.replace("</script", "<\\/script")
}

#[cfg(test)]
mod tests {
    use super::{escape_script_json_line, escape_text};

    #[test]
    fn escapes_title_text() {
        assert_eq!(escape_text("A < B & C > D"), "A &lt; B &amp; C &gt; D");
    }

    #[test]
    fn escapes_script_close_sequence() {
        assert_eq!(escape_script_json_line(r#""</script>""#), r#""<\/script>""#);
    }
}
