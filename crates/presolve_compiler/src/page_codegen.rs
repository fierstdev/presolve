use serde::Serialize;

use crate::{
    ResumeManifest, RuntimeComponentArtifact, RuntimeComputedArtifact, RuntimeContextArtifact,
    RuntimeEffectArtifact, RuntimeFormsArtifact, RuntimeOpaqueArtifact,
    RuntimePackageInvocationArtifact, RuntimeResourceArtifact, TemplateManifest,
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

/// Generates the full runtime page with a Resource artifact embedded before
/// the runtime boot script. The caller must supply the host-bound artifact.
#[must_use]
#[allow(clippy::too_many_arguments)]
pub fn generate_standalone_page_with_resume_runtime_and_resources(
    title: &str,
    body_html: &str,
    manifest: &TemplateManifest,
    computed: &RuntimeComputedArtifact,
    context: &RuntimeContextArtifact,
    effects: &RuntimeEffectArtifact,
    components: &RuntimeComponentArtifact,
    forms: &RuntimeFormsArtifact,
    resume: &ResumeManifest,
    resources: &RuntimeResourceArtifact,
) -> String {
    let page = generate_standalone_page_with_resume_runtime(
        title, body_html, manifest, computed, context, effects, components, forms, resume,
    );
    let resource_script = embedded_runtime_json_script("presolve-resources-runtime", resources);
    page.replacen(
        "    <script src=\"./runtime.js\" defer></script>",
        &(resource_script + "    <script src=\"./runtime.js\" defer></script>"),
        1,
    )
}

/// Embeds a validated opaque-terminal artifact immediately before the runtime
/// boot script. The caller composes this with other compiler-owned page
/// products; the function never inspects application source.
#[must_use]
pub fn embed_opaque_runtime_artifact(page: String, opaque: &RuntimeOpaqueArtifact) -> String {
    let opaque_script = embedded_runtime_json_script("presolve-opaque-runtime", opaque);
    page.replacen(
        "    <script src=\"./runtime.js\" defer></script>",
        &(opaque_script + "    <script src=\"./runtime.js\" defer></script>"),
        1,
    )
}

/// Embeds the compiler-authorized decorator-free package invocation registry
/// metadata immediately before runtime boot.
#[must_use]
pub fn embed_package_invocation_runtime_artifact(
    page: String,
    package_invocations: &RuntimePackageInvocationArtifact,
) -> String {
    let artifact_script =
        embedded_runtime_json_script("presolve-package-invocations-runtime", package_invocations);
    page.replacen(
        "    <script src=\"./runtime.js\" defer></script>",
        &(artifact_script + "    <script src=\"./runtime.js\" defer></script>"),
        1,
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

    output.push_str(&embedded_runtime_json_script(
        "presolve-template-manifest",
        manifest,
    ));
    if let Some(computed) = computed {
        output.push_str(&embedded_runtime_json_script(
            "presolve-computed-runtime",
            computed,
        ));
    }
    if let Some(context) = context {
        output.push_str(&embedded_runtime_json_script(
            "presolve-context-runtime",
            context,
        ));
    }
    if let Some(effects) = effects {
        output.push_str(&embedded_runtime_json_script(
            "presolve-effect-runtime",
            effects,
        ));
    }
    if let Some(components) = components {
        output.push_str(&embedded_runtime_json_script(
            "presolve-component-runtime",
            components,
        ));
    }
    if let Some(forms) = forms {
        output.push_str(&embedded_runtime_json_script(
            "presolve-forms-runtime",
            forms,
        ));
    }
    if let Some(resume) = resume {
        output.push_str("    <script type=\"application/json\" id=\"presolve-resume-runtime\">");
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

/// The canonical JSON artifacts remain pretty-printed, digest-bound files in
/// the publication inventory. Their document copies are a browser transport,
/// so compact serialization avoids repeating presentation whitespace in every
/// route while retaining the exact schema and values the runtime validates.
fn embedded_runtime_json_script<T: Serialize>(id: &str, value: &T) -> String {
    let json = serde_json::to_string(value).expect("runtime artifact should serialize");
    format!(
        "    <script type=\"application/json\" id=\"{id}\">\n      {}\n    </script>\n",
        escape_script_json_line(&json)
    )
}

#[cfg(test)]
mod tests {
    use super::{
        embed_opaque_runtime_artifact, embedded_runtime_json_script, escape_script_json_line,
        escape_text,
    };

    #[test]
    fn escapes_title_text() {
        assert_eq!(escape_text("A < B & C > D"), "A &lt; B &amp; C &gt; D");
    }

    #[test]
    fn escapes_script_close_sequence() {
        assert_eq!(escape_script_json_line(r#""</script>""#), r#""<\/script>""#);
    }

    #[test]
    fn embeds_runtime_json_compactly_without_changing_values() {
        let value = serde_json::json!({
            "schema_version": 1,
            "records": [{ "id": "record:one", "value": "</script>" }]
        });
        let script = embedded_runtime_json_script("presolve-test-runtime", &value);
        let payload = script
            .split_once('>')
            .and_then(|(_, suffix)| suffix.rsplit_once("</script>"))
            .map(|(payload, _)| payload.trim())
            .expect("embedded JSON script payload");

        assert_eq!(payload.lines().count(), 1);
        assert!(payload.contains(r#"<\/script>"#));
        assert_eq!(
            serde_json::from_str::<serde_json::Value>(payload).unwrap(),
            value
        );
    }

    #[test]
    fn embeds_opaque_artifact_before_the_runtime_boot_script() {
        let artifact: crate::RuntimeOpaqueArtifact = serde_json::from_str(
            r#"{"schema_version":1,"activations":[{"id":"opaque:track","owner_component":"component:x","method":"component:x/method:track","package":"@acme/analytics","version":"1.2.3","integrity":"sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa","export":"trackPurchase","type_signature":"() -> void","runtime_module":"dist/track.js","execution_boundary":"client","resume_policy":"cold_fallback"}]}"#,
        )
        .unwrap();
        let page = embed_opaque_runtime_artifact(
            "    <script src=\"./runtime.js\" defer></script>".to_string(),
            &artifact,
        );
        assert!(page.contains("presolve-opaque-runtime"));
        assert!(page.contains("trackPurchase"));
        assert!(page.find("presolve-opaque-runtime") < page.find("./runtime.js"));
    }
}
