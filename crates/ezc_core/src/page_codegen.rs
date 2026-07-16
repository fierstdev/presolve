use crate::{
    runtime_component_artifact_json, runtime_computed_artifact_json, runtime_context_artifact_json,
    runtime_effect_artifact_json, runtime_forms_artifact_json, template_manifest_json,
    ResumeManifest, RuntimeComponentArtifact, RuntimeComputedArtifact, RuntimeContextArtifact,
    RuntimeEffectArtifact, RuntimeFormsArtifact, TemplateManifest,
};

#[must_use]
pub fn generate_standalone_page(
    title: &str,
    body_html: &str,
    manifest: &TemplateManifest,
) -> String {
    generate_page(
        title, body_html, manifest, None, None, None, None, None, None,
    )
}

/// Generate a standalone page with compiler-generated computed runtime data.
#[must_use]
pub fn generate_standalone_page_with_computed_runtime(
    title: &str,
    body_html: &str,
    manifest: &TemplateManifest,
    computed: &RuntimeComputedArtifact,
) -> String {
    generate_page(
        title,
        body_html,
        manifest,
        Some(computed),
        None,
        None,
        None,
        None,
        None,
    )
}

/// Generate a standalone page with compiler-generated computed and effect runtime data.
#[must_use]
pub fn generate_standalone_page_with_effect_runtime(
    title: &str,
    body_html: &str,
    manifest: &TemplateManifest,
    computed: &RuntimeComputedArtifact,
    effects: &RuntimeEffectArtifact,
) -> String {
    generate_page(
        title,
        body_html,
        manifest,
        Some(computed),
        None,
        Some(effects),
        None,
        None,
        None,
    )
}

/// Generate a standalone page with compiler-generated Context runtime data.
#[must_use]
pub fn generate_standalone_page_with_context_runtime(
    title: &str,
    body_html: &str,
    manifest: &TemplateManifest,
    computed: &RuntimeComputedArtifact,
    context: &RuntimeContextArtifact,
    effects: &RuntimeEffectArtifact,
) -> String {
    generate_page(
        title,
        body_html,
        manifest,
        Some(computed),
        Some(context),
        Some(effects),
        None,
        None,
        None,
    )
}

/// Generate a standalone page with all compiler-generated runtime artifacts.
#[must_use]
pub fn generate_standalone_page_with_component_runtime(
    title: &str,
    body_html: &str,
    manifest: &TemplateManifest,
    computed: &RuntimeComputedArtifact,
    context: &RuntimeContextArtifact,
    effects: &RuntimeEffectArtifact,
    components: &RuntimeComponentArtifact,
) -> String {
    generate_page(
        title,
        body_html,
        manifest,
        Some(computed),
        Some(context),
        Some(effects),
        Some(components),
        None,
        None,
    )
}

/// Generate a standalone page with all compiler-generated runtime artifacts,
/// including the I15 Forms artifact.
#[must_use]
#[allow(clippy::too_many_arguments)]
pub fn generate_standalone_page_with_component_runtime_and_forms(
    title: &str,
    body_html: &str,
    manifest: &TemplateManifest,
    computed: &RuntimeComputedArtifact,
    context: &RuntimeContextArtifact,
    effects: &RuntimeEffectArtifact,
    components: &RuntimeComponentArtifact,
    forms: &RuntimeFormsArtifact,
) -> String {
    generate_page(
        title,
        body_html,
        manifest,
        Some(computed),
        Some(context),
        Some(effects),
        Some(components),
        Some(forms),
        None,
    )
}

/// Generate a standalone page with every runtime artifact and the exact J9
/// resume-manifest bytes embedded for runtime consumption.
#[must_use]
#[allow(clippy::too_many_arguments)]
pub fn generate_standalone_page_with_resume_runtime(
    title: &str,
    body_html: &str,
    manifest: &TemplateManifest,
    computed: &RuntimeComputedArtifact,
    context: &RuntimeContextArtifact,
    effects: &RuntimeEffectArtifact,
    components: &RuntimeComponentArtifact,
    forms: &RuntimeFormsArtifact,
    resume: &ResumeManifest,
) -> String {
    generate_page(
        title,
        body_html,
        manifest,
        Some(computed),
        Some(context),
        Some(effects),
        Some(components),
        Some(forms),
        Some(resume),
    )
}

#[allow(clippy::too_many_arguments)]
fn generate_page(
    title: &str,
    body_html: &str,
    manifest: &TemplateManifest,
    computed: Option<&RuntimeComputedArtifact>,
    context: Option<&RuntimeContextArtifact>,
    effects: Option<&RuntimeEffectArtifact>,
    components: Option<&RuntimeComponentArtifact>,
    forms: Option<&RuntimeFormsArtifact>,
    resume: Option<&ResumeManifest>,
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
    if let Some(computed) = computed {
        output.push_str("    <script type=\"application/json\" id=\"ez-computed-runtime\">\n");
        for line in runtime_computed_artifact_json(computed).lines() {
            output.push_str("      ");
            output.push_str(&escape_script_json_line(line));
            output.push('\n');
        }
        output.push_str("    </script>\n");
    }
    if let Some(context) = context {
        output.push_str("    <script type=\"application/json\" id=\"ez-context-runtime\">\n");
        for line in runtime_context_artifact_json(context).lines() {
            output.push_str("      ");
            output.push_str(&escape_script_json_line(line));
            output.push('\n');
        }
        output.push_str("    </script>\n");
    }
    if let Some(effects) = effects {
        output.push_str("    <script type=\"application/json\" id=\"ez-effect-runtime\">\n");
        for line in runtime_effect_artifact_json(effects).lines() {
            output.push_str("      ");
            output.push_str(&escape_script_json_line(line));
            output.push('\n');
        }
        output.push_str("    </script>\n");
    }
    if let Some(components) = components {
        output.push_str("    <script type=\"application/json\" id=\"ez-component-runtime\">\n");
        for line in runtime_component_artifact_json(components).lines() {
            output.push_str("      ");
            output.push_str(&escape_script_json_line(line));
            output.push('\n');
        }
        output.push_str("    </script>\n");
    }
    if let Some(forms) = forms {
        output.push_str("    <script type=\"application/json\" id=\"ez-forms-runtime\">\n");
        for line in runtime_forms_artifact_json(forms).lines() {
            output.push_str("      ");
            output.push_str(&escape_script_json_line(line));
            output.push('\n');
        }
        output.push_str("    </script>\n");
    }
    if let Some(resume) = resume {
        output.push_str("    <script type=\"application/json\" id=\"ez-resume-runtime\">");
        output.push_str(&crate::resume_manifest_json(resume));
        output.push_str("    </script>\n");
    }
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
