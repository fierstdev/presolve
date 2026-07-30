//! Compiler-owned application document template projection.

use std::fmt;

/// Exact placeholders required by an `app/index.html` document template.
pub const DOCUMENT_TEMPLATE_PLACEHOLDERS_V1: [&str; 3] =
    ["{{ head }}", "{{ app }}", "{{ runtime }}"];

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DocumentTemplateErrorV1 {
    pub code: &'static str,
    pub message: String,
}

impl fmt::Display for DocumentTemplateErrorV1 {
    fn fmt(&self, output: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(output, "{}: {}", self.code, self.message)
    }
}

impl std::error::Error for DocumentTemplateErrorV1 {}

/// Applies a validated `app/index.html` template to a compiler-generated page.
///
/// The template controls document framing only. Presolve extracts the existing
/// compiler-owned head, rendered application, and runtime payload from the
/// generated page and injects each into exactly one required placeholder.
pub fn apply_document_template_v1(
    template: &str,
    generated_page: &str,
    additional_head: &str,
) -> Result<String, DocumentTemplateErrorV1> {
    validate_template(template)?;
    let (head, app, runtime) = split_generated_page(generated_page)?;
    let head = format!("{head}{additional_head}");
    let mut replacements = [
        (
            template
                .find("{{ head }}")
                .expect("validated template has one head placeholder"),
            "{{ head }}",
            head.as_str(),
        ),
        (
            template
                .find("{{ app }}")
                .expect("validated template has one app placeholder"),
            "{{ app }}",
            app,
        ),
        (
            template
                .find("{{ runtime }}")
                .expect("validated template has one runtime placeholder"),
            "{{ runtime }}",
            runtime,
        ),
    ];
    replacements.sort_by_key(|(offset, _, _)| *offset);

    let mut output = String::with_capacity(template.len() + head.len() + app.len() + runtime.len());
    let mut cursor = 0;
    for (offset, placeholder, value) in replacements {
        output.push_str(&template[cursor..offset]);
        output.push_str(value);
        cursor = offset + placeholder.len();
    }
    output.push_str(&template[cursor..]);
    Ok(output)
}

fn validate_template(template: &str) -> Result<(), DocumentTemplateErrorV1> {
    if !template.trim_start().starts_with("<!doctype html>") || !template.contains("<html") {
        return Err(DocumentTemplateErrorV1 {
            code: "PSDOC1001_TEMPLATE_DOCUMENT_INVALID",
            message: "app/index.html must declare an HTML document".into(),
        });
    }
    for placeholder in DOCUMENT_TEMPLATE_PLACEHOLDERS_V1 {
        if template.matches(placeholder).count() != 1 {
            return Err(DocumentTemplateErrorV1 {
                code: "PSDOC1002_TEMPLATE_PLACEHOLDER_COUNT",
                message: format!("{placeholder} must occur exactly once"),
            });
        }
    }
    let mut remaining = template;
    while let Some(start) = remaining.find("{{") {
        let suffix = &remaining[start..];
        let Some(end) = suffix.find("}}") else {
            return Err(DocumentTemplateErrorV1 {
                code: "PSDOC1003_TEMPLATE_PLACEHOLDER_INVALID",
                message: "unclosed document-template placeholder".into(),
            });
        };
        let placeholder = &suffix[..end + 2];
        if !DOCUMENT_TEMPLATE_PLACEHOLDERS_V1.contains(&placeholder) {
            return Err(DocumentTemplateErrorV1 {
                code: "PSDOC1003_TEMPLATE_PLACEHOLDER_INVALID",
                message: format!("unsupported document-template placeholder {placeholder}"),
            });
        }
        remaining = &suffix[end + 2..];
    }
    Ok(())
}

fn split_generated_page(page: &str) -> Result<(&str, &str, &str), DocumentTemplateErrorV1> {
    let head_start = "  <head>\n";
    let head_end = "  </head>\n";
    let body_start = "  <body>\n";
    let runtime_start =
        "    <script type=\"application/json\" id=\"presolve-template-manifest\">\n";
    let body_end = "  </body>\n</html>\n";
    let Some(head_after) = page.find(head_start).map(|index| index + head_start.len()) else {
        return generated_page_error();
    };
    let Some(head_end_index) = page[head_after..]
        .find(head_end)
        .map(|index| head_after + index)
    else {
        return generated_page_error();
    };
    let body_after = head_end_index + head_end.len();
    if !page[body_after..].starts_with(body_start) {
        return generated_page_error();
    }
    let app_after = body_after + body_start.len();
    let Some(runtime_index) = page[app_after..]
        .find(runtime_start)
        .map(|index| app_after + index)
    else {
        return generated_page_error();
    };
    let Some(body_end_index) = page[runtime_index..]
        .find(body_end)
        .map(|index| runtime_index + index)
    else {
        return generated_page_error();
    };
    Ok((
        &page[head_after..head_end_index],
        &page[app_after..runtime_index],
        &page[runtime_index..body_end_index],
    ))
}

fn generated_page_error<T>() -> Result<T, DocumentTemplateErrorV1> {
    Err(DocumentTemplateErrorV1 {
        code: "PSDOC1004_GENERATED_PAGE_INVALID",
        message: "compiler-generated page does not have the expected document structure".into(),
    })
}

#[cfg(test)]
mod tests {
    use super::apply_document_template_v1;

    const PAGE: &str = "<!doctype html>\n<html lang=\"en\">\n  <head>\n    <meta charset=\"utf-8\">\n    <title>Home</title>\n  </head>\n  <body>\n    <main>Home</main>\n    <script type=\"application/json\" id=\"presolve-template-manifest\">\n      {}\n    </script>\n    <script src=\"./runtime.js\" defer></script>\n  </body>\n</html>\n";

    #[test]
    fn projects_compiler_owned_parts_into_the_required_placeholders() {
        let output = apply_document_template_v1(
            "<!doctype html>\n<html lang=\"en\">\n<head>\n{{ head }}\n</head>\n<body>\n{{ app }}{{ runtime }}\n</body>\n</html>\n",
            PAGE,
            "    <link rel=\"stylesheet\" href=\"/app.css\">\n",
        )
        .unwrap();
        assert!(output.contains("<head>\n    <meta charset"));
        assert!(output.contains("href=\"/app.css\""));
        assert!(output.contains("<main>Home</main>"));
        assert!(output.contains("presolve-template-manifest"));
        assert!(!output.contains("{{ head }}"));
    }

    #[test]
    fn rejects_missing_or_unknown_placeholders() {
        assert_eq!(
            apply_document_template_v1(
                "<!doctype html><html>{{ head }}{{ app }}</html>",
                PAGE,
                "",
            )
            .unwrap_err()
            .code,
            "PSDOC1002_TEMPLATE_PLACEHOLDER_COUNT"
        );
        assert_eq!(
            apply_document_template_v1(
                "<!doctype html><html>{{ head }}{{ app }}{{ runtime }}{{ title }}</html>",
                PAGE,
                "",
            )
            .unwrap_err()
            .code,
            "PSDOC1003_TEMPLATE_PLACEHOLDER_INVALID"
        );
    }

    #[test]
    fn preserves_document_placeholder_spelling_inside_rendered_application_content() {
        let page = PAGE.replace(
            "<main>Home</main>",
            "<main><code>{{ head }} {{ app }} {{ runtime }}</code></main>",
        );
        let output = apply_document_template_v1(
            "<!doctype html>\n<html lang=\"en\">\n<head>{{ head }}</head>\n<body>{{ app }}{{ runtime }}</body>\n</html>\n",
            &page,
            "",
        )
        .unwrap();

        assert!(output.contains("<code>{{ head }} {{ app }} {{ runtime }}</code>"));
        assert_eq!(output.matches("presolve-template-manifest").count(), 1);
        assert_eq!(output.matches("{{ runtime }}").count(), 1);
    }
}
