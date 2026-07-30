//! Phase P explicit multi-source application-publication request validation.
//!
//! This module deliberately stops before artifact lowering or filesystem
//! publication. It establishes one compiler-owned request/entry authority for
//! the later publication product.

#![allow(clippy::too_many_lines)]

use std::collections::{BTreeMap, BTreeSet};
use std::fmt;
use std::fmt::Write as _;
use std::path::{Component, PathBuf};

use serde::{Deserialize, Serialize};

use crate::platform::{
    validate_workspace_configuration_v1, workspace_configuration_fingerprint_v1, Digest,
    WorkspaceConfiguration,
};
use crate::semantic_package::SemanticPackageResolutionTable;
use crate::semantic_package_runtime::SemanticPackageRuntimeModuleTable;
use crate::{
    build_application_semantic_model_for_unit_with_packages, build_production_audit_report_v1,
    build_production_reports, build_production_runtime_artifact, build_resume_chunk_graph,
    build_resume_manifest, build_runtime_component_artifact, build_runtime_computed_artifact,
    build_runtime_context_artifact, build_runtime_effect_artifact, build_runtime_forms_artifact,
    build_runtime_opaque_artifact_with_modules, build_runtime_package_invocation_artifact,
    build_runtime_resource_artifact_with_modules, build_template_manifest_from_asm,
    embed_opaque_runtime_artifact, embed_package_invocation_runtime_artifact,
    emit_production_modules, extract_production_chunk_graph,
    generate_ordinary_instance_html_for_component, generate_runtime_stub,
    generate_standalone_page_with_resume_runtime,
    generate_standalone_page_with_resume_runtime_and_resources, lower_components_to_ir,
    optimization_report_json, optimize_context_ir, optimize_effect_ir,
    production_audit_report_json_v1, production_runtime_artifact_json, resume_manifest_json,
    runtime_component_artifact_json, runtime_computed_artifact_json, runtime_context_artifact_json,
    runtime_cost_report_json, runtime_effect_artifact_json, runtime_forms_artifact_json,
    runtime_opaque_artifact_json, runtime_package_invocation_artifact_json,
    runtime_resource_artifact_json, template_manifest_json, validate_runtime_opaque_artifact,
    validate_runtime_package_invocation_artifact, validate_runtime_resource_artifact,
    CompilationUnit, ConstantFoldingPass, ExecutableProgramFingerprint, ImmutableAsmPass,
    ProductionReportInputs, ProductionRootChunkInput, SemanticId, SharedChunkCandidatePlan,
};

pub const APPLICATION_PUBLICATION_MANIFEST_SCHEMA_VERSION: u32 = 1;
pub const APPLICATION_PUBLICATION_COMPILER_CONTRACT_V1: &str = "presolve-application-publication:1";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ApplicationPublicationProfileV1 {
    Development,
    Production,
}

impl ApplicationPublicationProfileV1 {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Development => "development",
            Self::Production => "production",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ApplicationPublicationSourceV1 {
    pub logical_path: PathBuf,
    pub source: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ApplicationPublicationRequestV1 {
    pub configuration: WorkspaceConfiguration,
    /// The complete caller-authorized source set. The compiler parses this
    /// exact set; no source-root discovery is part of publication.
    pub sources: Vec<ApplicationPublicationSourceV1>,
    pub entry_path: PathBuf,
    pub package_contracts: SemanticPackageResolutionTable,
    pub package_runtime_modules: SemanticPackageRuntimeModuleTable,
    pub profile: ApplicationPublicationProfileV1,
    pub output_root: PathBuf,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ValidatedApplicationPublicationRequestV1 {
    pub request: ApplicationPublicationRequestV1,
    pub unit: CompilationUnit,
    /// Source-selected page component. This remains the published entry
    /// identity even when a compiler-owned file-route layout is the rendered
    /// root.
    pub entry_component: SemanticId,
    /// Compiler-selected materialization root. Explicit publication defaults
    /// to `entry_component`; conventional file routes may set an outer layout.
    pub render_root_component: SemanticId,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ApplicationPublicationRequestErrorV1 {
    pub code: &'static str,
    pub message: String,
}

impl fmt::Display for ApplicationPublicationRequestErrorV1 {
    fn fmt(&self, output: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(output, "{}: {}", self.code, self.message)
    }
}

impl std::error::Error for ApplicationPublicationRequestErrorV1 {}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ApplicationPublicationErrorV1 {
    pub code: &'static str,
    pub message: String,
}

impl fmt::Display for ApplicationPublicationErrorV1 {
    fn fmt(&self, output: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(output, "{}: {}", self.code, self.message)
    }
}

impl std::error::Error for ApplicationPublicationErrorV1 {}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub struct ApplicationPublicationArtifactV1 {
    pub path: String,
    pub digest: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ApplicationPublicationManifestV1 {
    pub schema_version: u32,
    pub compiler_contract: String,
    pub workspace_snapshot_id: String,
    pub entry_component_id: String,
    pub profile: String,
    pub artifacts: Vec<ApplicationPublicationArtifactV1>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ApplicationPublicationProductV1 {
    pub manifest: ApplicationPublicationManifestV1,
    /// Exact compiler-generated bytes keyed by normalized relative artifact
    /// path. Filesystem publication is deliberately a separate host concern.
    pub artifacts: BTreeMap<PathBuf, Vec<u8>>,
}

#[must_use]
/// Serializes the exact schema-v1 publication manifest.
///
/// # Panics
///
/// Panics only if the compiler-owned manifest model cannot serialize.
pub fn application_publication_manifest_json_v1(
    manifest: &ApplicationPublicationManifestV1,
) -> String {
    serde_json::to_string_pretty(manifest)
        .expect("application-publication manifest is serializable")
        + "\n"
}

/// Validates caller-owned request identity and explicit entry selection.
///
/// # Errors
///
/// Returns a stable `PSAPP100x` error when the workspace configuration, exact
/// source set, logical entry, or rendered application root is invalid.
pub fn validate_application_publication_request_v1(
    request: ApplicationPublicationRequestV1,
) -> Result<ValidatedApplicationPublicationRequestV1, ApplicationPublicationRequestErrorV1> {
    if let Err(error) = validate_workspace_configuration_v1(&request.configuration) {
        return Err(ApplicationPublicationRequestErrorV1 {
            code: "PSAPP1007_INVALID_WORKSPACE_CONFIGURATION",
            message: error.message,
        });
    }
    if request.sources.is_empty() {
        return Err(ApplicationPublicationRequestErrorV1 {
            code: "PSAPP1001_EMPTY_SOURCE_SET",
            message: "application publication requires at least one explicit source".into(),
        });
    }
    if request.entry_path.as_os_str().is_empty()
        || request.entry_path.is_absolute()
        || request.entry_path.components().any(|component| {
            matches!(
                component,
                Component::ParentDir | Component::RootDir | Component::Prefix(_)
            )
        })
    {
        return Err(ApplicationPublicationRequestErrorV1 {
            code: "PSAPP1002_INVALID_ENTRY_PATH",
            message: "application entry path must be a non-empty relative logical path".into(),
        });
    }
    let paths = request
        .sources
        .iter()
        .map(|source| source.logical_path.clone())
        .collect::<BTreeSet<_>>();
    if paths.len() != request.sources.len() {
        return Err(ApplicationPublicationRequestErrorV1 {
            code: "PSAPP1003_DUPLICATE_LOGICAL_SOURCE",
            message: "application publication source logical paths must be unique".into(),
        });
    }
    if !paths.contains(&request.entry_path) {
        return Err(ApplicationPublicationRequestErrorV1 {
            code: "PSAPP1004_ENTRY_NOT_IN_SOURCE_SET",
            message: "application entry path must name one explicit source".into(),
        });
    }
    let unit = CompilationUnit::parse_sources(
        request
            .sources
            .iter()
            .map(|source| (&source.logical_path, source.source.as_str())),
    );
    let model =
        build_application_semantic_model_for_unit_with_packages(&unit, &request.package_contracts);
    let entries = model
        .components
        .iter()
        .filter(|component| {
            component.module_path == request.entry_path
                && component.element_name.is_some()
                && component.render.is_some()
        })
        .map(|component| component.id.clone())
        .collect::<Vec<_>>();
    let [entry_component] = entries.as_slice() else {
        return Err(ApplicationPublicationRequestErrorV1 {
            code: if entries.is_empty() {
                "PSAPP1005_ENTRY_APPLICATION_ROOT_MISSING"
            } else {
                "PSAPP1006_ENTRY_APPLICATION_ROOT_AMBIGUOUS"
            },
            message:
                "application entry source must declare exactly one supported rendered component"
                    .into(),
        });
    };
    Ok(ValidatedApplicationPublicationRequestV1 {
        request,
        unit,
        entry_component: entry_component.clone(),
        render_root_component: entry_component.clone(),
    })
}

/// Lowers one validated, explicit complete workspace into its exact artifact
/// inventory. This is the only multi-source publication derivation authority;
/// command and framework layers may publish these bytes but may not alter them.
///
/// # Errors
///
/// Returns a stable `PSAPP200x` error when package mappings or compiler
/// generated runtime/production products cannot form a browser publication.
pub fn build_application_publication_product_v1(
    validated: ValidatedApplicationPublicationRequestV1,
) -> Result<ApplicationPublicationProductV1, ApplicationPublicationErrorV1> {
    let asm =
        ConstantFoldingPass.transform(&build_application_semantic_model_for_unit_with_packages(
            &validated.unit,
            &validated.request.package_contracts,
        ));
    build_application_publication_product_from_asm_v1(validated, asm)
}

/// Lowers one validated explicit publication request from an already-built
/// canonical application model. This is used by compiler-owned file-route
/// composition; callers cannot supply altered artifacts or a framework model.
///
/// # Errors
///
/// Returns the same publication diagnostics as the standard model assembly
/// path when compiler products cannot form a browser publication.
pub fn build_application_publication_product_from_asm_v1(
    validated: ValidatedApplicationPublicationRequestV1,
    asm: crate::ApplicationSemanticModel,
) -> Result<ApplicationPublicationProductV1, ApplicationPublicationErrorV1> {
    let request = validated.request;
    if let Some(diagnostic) = asm.diagnostics.iter().find(|diagnostic| {
        diagnostic.code.starts_with("PSBIND10")
            || matches!(diagnostic.code.as_str(), "PSC1128" | "PSC1130" | "PSC1131")
    }) {
        return Err(ApplicationPublicationErrorV1 {
            code: "PSAPP2001_PACKAGE_SEMANTICS_INVALID",
            message: format!("{}: {}", diagnostic.code, diagnostic.message),
        });
    }
    let resource_runtime_artifact = if asm.resource_declarations.is_empty() {
        None
    } else {
        let artifact =
            build_runtime_resource_artifact_with_modules(&asm, &request.package_runtime_modules)
                .map_err(|error| ApplicationPublicationErrorV1 {
                    code: "PSAPP2002_RESOURCE_RUNTIME_MAPPING_INCOMPLETE",
                    message: format!("{error:?}"),
                })?;
        if !validate_runtime_resource_artifact(&artifact).is_empty() {
            return Err(ApplicationPublicationErrorV1 {
                code: "PSAPP2003_RESOURCE_RUNTIME_ARTIFACT_INVALID",
                message: "compiler produced an invalid Resource runtime artifact".into(),
            });
        }
        if let Some(declaration) = artifact
            .declarations
            .iter()
            .find(|declaration| declaration.execution_boundary == "Server")
        {
            return Err(ApplicationPublicationErrorV1 {
                code: "PSAPP2004_SERVER_RESOURCE_IN_BROWSER_PUBLICATION",
                message: format!(
                    "resource `{}` uses a server endpoint and cannot be published for browser execution",
                    declaration.id
                ),
            });
        }
        Some(artifact)
    };
    let opaque_runtime_artifact = if asm.opaque_action_resolutions.is_empty() {
        None
    } else {
        let artifact =
            build_runtime_opaque_artifact_with_modules(&asm, &request.package_runtime_modules)
                .map_err(|error| ApplicationPublicationErrorV1 {
                    code: "PSAPP2005_OPAQUE_RUNTIME_MAPPING_INCOMPLETE",
                    message: format!("{error:?}"),
                })?;
        if !validate_runtime_opaque_artifact(&artifact).is_empty() {
            return Err(ApplicationPublicationErrorV1 {
                code: "PSAPP2006_OPAQUE_RUNTIME_ARTIFACT_INVALID",
                message: "compiler produced an invalid opaque-terminal runtime artifact".into(),
            });
        }
        Some(artifact)
    };
    let package_invocation_runtime_artifact = if asm.terminal_package_invocations.is_empty() {
        None
    } else {
        let artifact = build_runtime_package_invocation_artifact(&asm);
        if !validate_runtime_package_invocation_artifact(&artifact).is_empty() {
            return Err(ApplicationPublicationErrorV1 {
                code: "PSAPP2009_PACKAGE_INVOCATION_RUNTIME_ARTIFACT_INVALID",
                message: "compiler produced an invalid decorator-free package invocation artifact"
                    .into(),
            });
        }
        Some(artifact)
    };

    let ir = lower_components_to_ir(&asm);
    let computed_runtime_artifact = build_runtime_computed_artifact(&asm, &ir);
    let computed_runtime_json = runtime_computed_artifact_json(&computed_runtime_artifact);
    let effect_runtime_artifact =
        build_runtime_effect_artifact(&asm, &optimize_effect_ir(&ir).output);
    let effect_runtime_json = runtime_effect_artifact_json(&effect_runtime_artifact);
    let context_runtime_artifact = build_runtime_context_artifact(&asm, &optimize_context_ir(&ir));
    let context_runtime_json = runtime_context_artifact_json(&context_runtime_artifact);
    let component_runtime_artifact =
        build_runtime_component_artifact(&asm, &asm.component_ir_optimization);
    let component_runtime_json = runtime_component_artifact_json(&component_runtime_artifact);
    let forms_runtime_artifact = build_runtime_forms_artifact(&asm);
    let forms_runtime_json = runtime_forms_artifact_json(&forms_runtime_artifact);
    let resume_runtime_artifact = build_resume_manifest(&asm);
    let resume_runtime_json = resume_manifest_json(&resume_runtime_artifact);
    let resource_runtime_json = resource_runtime_artifact
        .as_ref()
        .map(runtime_resource_artifact_json);
    let opaque_runtime_json = opaque_runtime_artifact
        .as_ref()
        .map(runtime_opaque_artifact_json);
    let package_invocation_runtime_json = package_invocation_runtime_artifact
        .as_ref()
        .map(runtime_package_invocation_artifact_json);
    let (production_chunk_graph, _) = extract_production_chunk_graph(
        &SharedChunkCandidatePlan {
            candidates: Vec::new(),
            rejections: Vec::new(),
        },
        &production_root_chunk_inputs(&resume_runtime_artifact),
    )
    .map_err(|error| ApplicationPublicationErrorV1 {
        code: "PSAPP2007_PRODUCTION_GRAPH_INVALID",
        message: format!("{error:?}"),
    })?;
    let production_runtime_artifact =
        build_production_runtime_artifact(&resume_runtime_artifact, &production_chunk_graph)
            .map_err(|error| ApplicationPublicationErrorV1 {
                code: "PSAPP2008_PRODUCTION_RUNTIME_INVALID",
                message: format!("{error:?}"),
            })?;
    let production_runtime_json = production_runtime_artifact_json(&production_runtime_artifact);
    let resume_chunks = build_resume_chunk_graph(&asm);
    let html_fragment =
        generate_ordinary_instance_html_for_component(&asm, &validated.render_root_component);
    let template_manifest = build_template_manifest_from_asm(&asm);
    let template_manifest_json = template_manifest_json(&template_manifest);
    let page_title = asm
        .components
        .iter()
        .find(|component| component.id == validated.render_root_component)
        .map_or_else(
            || "Presolve App".to_string(),
            |component| component.class_name.clone(),
        );
    let page_html = generate_standalone_page_with_resume_runtime(
        &page_title,
        &html_fragment,
        &template_manifest,
        &computed_runtime_artifact,
        &context_runtime_artifact,
        &effect_runtime_artifact,
        &component_runtime_artifact,
        &forms_runtime_artifact,
        &resume_runtime_artifact,
    );
    let page_html = resource_runtime_artifact
        .as_ref()
        .map_or(page_html, |resources| {
            generate_standalone_page_with_resume_runtime_and_resources(
                &page_title,
                &html_fragment,
                &template_manifest,
                &computed_runtime_artifact,
                &context_runtime_artifact,
                &effect_runtime_artifact,
                &component_runtime_artifact,
                &forms_runtime_artifact,
                &resume_runtime_artifact,
                resources,
            )
        });
    let page_html = if let Some(opaque) = &opaque_runtime_artifact {
        embed_opaque_runtime_artifact(page_html, opaque)
    } else {
        page_html
    };
    let page_html = if let Some(package_invocations) = &package_invocation_runtime_artifact {
        embed_package_invocation_runtime_artifact(page_html, package_invocations)
    } else {
        page_html
    };
    let page_html = production_mode_page_html(
        page_html,
        request.profile == ApplicationPublicationProfileV1::Production,
        &production_runtime_json,
    );
    let runtime_js = generate_runtime_stub();
    let production_layout = emit_production_modules(&production_chunk_graph);
    let development_bytes = byte_count([
        &page_html,
        &template_manifest_json,
        &computed_runtime_json,
        &context_runtime_json,
        &effect_runtime_json,
        &component_runtime_json,
        &forms_runtime_json,
        &resume_runtime_json,
        &runtime_js,
    ]) + resource_runtime_json
        .as_ref()
        .map_or(0, |json| json.len() as u64)
        + opaque_runtime_json
            .as_ref()
            .map_or(0, |json| json.len() as u64)
        + package_invocation_runtime_json
            .as_ref()
            .map_or(0, |json| json.len() as u64);
    let production_bytes = byte_count(
        std::iter::once(&production_runtime_json).chain(
            std::iter::once(&production_layout.eager)
                .chain(production_layout.shared.iter())
                .chain(production_layout.roots.iter())
                .map(|module| &module.source),
        ),
    );
    let report_inputs = ProductionReportInputs {
        dead_products_removed: 0,
        constants_pooled: 0,
        programs_deduplicated: 0,
        shared_candidates_rejected: 0,
        binding_writes_coalesced: 0,
        development_bytes,
        production_bytes,
        cold_init_operation_count: report_count(
            resume_runtime_artifact
                .capture_programs
                .iter()
                .map(|program| program.instructions.len())
                .sum(),
        ),
        resume_restore_operation_count: report_count(
            resume_runtime_artifact
                .restore_programs
                .iter()
                .map(|program| program.instructions.len())
                .sum(),
        ),
        max_action_batch_operation_count: 0,
        max_scheduler_batch_width: 0,
        max_dom_patch_count_per_action: 0,
        retained_slot_count: report_count(resume_runtime_artifact.slot_schemas.len()),
    };
    let (optimization_report, runtime_cost_report) = build_production_reports(
        &production_runtime_artifact,
        &production_chunk_graph,
        &report_inputs,
    );
    let production_audit_report =
        build_production_audit_report_v1(&optimization_report, &runtime_cost_report).map_err(
            |error| ApplicationPublicationErrorV1 {
                code: error.code,
                message: error.message,
            },
        )?;

    let mut artifacts = BTreeMap::new();
    insert_artifact(&mut artifacts, "index.html", page_html.into_bytes());
    insert_artifact(
        &mut artifacts,
        "template.manifest.json",
        template_manifest_json.into_bytes(),
    );
    insert_artifact(
        &mut artifacts,
        "computed.runtime.json",
        computed_runtime_json.into_bytes(),
    );
    insert_artifact(
        &mut artifacts,
        "context.runtime.json",
        context_runtime_json.into_bytes(),
    );
    insert_artifact(
        &mut artifacts,
        "effect.runtime.json",
        effect_runtime_json.into_bytes(),
    );
    insert_artifact(
        &mut artifacts,
        "component.runtime.json",
        component_runtime_json.into_bytes(),
    );
    insert_artifact(
        &mut artifacts,
        "forms.runtime.json",
        forms_runtime_json.into_bytes(),
    );
    insert_artifact(
        &mut artifacts,
        "resume.runtime.json",
        resume_runtime_json.into_bytes(),
    );
    insert_artifact(
        &mut artifacts,
        "production.runtime.json",
        production_runtime_json.into_bytes(),
    );
    insert_artifact(
        &mut artifacts,
        "optimization-report.json",
        optimization_report_json(&optimization_report).into_bytes(),
    );
    insert_artifact(
        &mut artifacts,
        "runtime-cost-report.json",
        runtime_cost_report_json(&runtime_cost_report).into_bytes(),
    );
    insert_artifact(
        &mut artifacts,
        "production-audit.json",
        production_audit_report_json_v1(&production_audit_report).into_bytes(),
    );
    insert_artifact(&mut artifacts, "runtime.js", runtime_js.into_bytes());
    if let Some(json) = resource_runtime_json {
        insert_artifact(&mut artifacts, "resources.runtime.json", json.into_bytes());
    }
    if let Some(json) = opaque_runtime_json {
        insert_artifact(&mut artifacts, "opaque.runtime.json", json.into_bytes());
    }
    if let Some(json) = package_invocation_runtime_json {
        insert_artifact(
            &mut artifacts,
            "package-invocations.runtime.json",
            json.into_bytes(),
        );
    }
    for chunk in &resume_chunks.chunks {
        artifacts.insert(
            PathBuf::from(&chunk.module.module_path),
            chunk.module.canonical_module_bytes.as_bytes().to_vec(),
        );
    }
    if request.profile == ApplicationPublicationProfileV1::Production {
        for module in std::iter::once(&production_layout.eager)
            .chain(production_layout.shared.iter())
            .chain(production_layout.roots.iter())
        {
            artifacts.insert(
                PathBuf::from("production").join(&module.filename),
                module.source.as_bytes().to_vec(),
            );
        }
    }
    let manifest = ApplicationPublicationManifestV1 {
        schema_version: APPLICATION_PUBLICATION_MANIFEST_SCHEMA_VERSION,
        compiler_contract: APPLICATION_PUBLICATION_COMPILER_CONTRACT_V1.into(),
        workspace_snapshot_id: application_workspace_snapshot_id_v1(&request),
        entry_component_id: validated.entry_component.to_string(),
        profile: request.profile.as_str().into(),
        artifacts: artifacts
            .iter()
            .map(|(path, bytes)| ApplicationPublicationArtifactV1 {
                path: path.to_string_lossy().replace('\\', "/"),
                digest: Digest::sha256(bytes).to_string(),
            })
            .collect(),
    };
    let manifest_json = application_publication_manifest_json_v1(&manifest);
    insert_artifact(
        &mut artifacts,
        "application.manifest.json",
        manifest_json.into_bytes(),
    );
    Ok(ApplicationPublicationProductV1 {
        manifest,
        artifacts,
    })
}

fn insert_artifact(artifacts: &mut BTreeMap<PathBuf, Vec<u8>>, path: &str, bytes: Vec<u8>) {
    let previous = artifacts.insert(PathBuf::from(path), bytes);
    debug_assert!(
        previous.is_none(),
        "application artifact paths must be unique"
    );
}

fn application_workspace_snapshot_id_v1(request: &ApplicationPublicationRequestV1) -> String {
    let configuration = workspace_configuration_fingerprint_v1(&request.configuration)
        .expect("validated application publication requires a valid workspace configuration");
    let mut canonical = format!(
        "contract={APPLICATION_PUBLICATION_COMPILER_CONTRACT_V1}\nconfiguration={}\n",
        configuration.as_str()
    );
    let mut sources = request.sources.iter().collect::<Vec<_>>();
    sources.sort_by(|left, right| left.logical_path.cmp(&right.logical_path));
    for source in sources {
        write!(
            canonical,
            "path={}\nsource={}\n",
            source.logical_path.display(),
            source.source
        )
        .expect("writing a workspace snapshot into a String cannot fail");
    }
    format!("application-workspace:{}", Digest::sha256(canonical))
}

fn production_root_chunk_inputs(
    resume: &crate::ResumeManifest,
) -> Vec<crate::ProductionRootChunkInput> {
    let mut roots = resume
        .chunks
        .iter()
        .filter_map(|chunk| {
            let root_kind = match chunk.root_kind {
                crate::resume_manifest::ResumeManifestChunkRootKind::Eager => return None,
                crate::resume_manifest::ResumeManifestChunkRootKind::Interaction => "interaction",
                crate::resume_manifest::ResumeManifestChunkRootKind::Visible => "visible",
                crate::resume_manifest::ResumeManifestChunkRootKind::Manual => "manual",
            };
            Some(ProductionRootChunkInput {
                activation_root_id: chunk.root_id.clone(),
                root_kind: root_kind.into(),
                programs: chunk
                    .provided_program_ids
                    .iter()
                    .map(|program| {
                        ExecutableProgramFingerprint::for_canonical_opcode_stream(
                            program.as_bytes(),
                        )
                    })
                    .collect(),
            })
        })
        .collect::<Vec<_>>();
    roots.sort_by(|left, right| left.activation_root_id.cmp(&right.activation_root_id));
    roots
}

fn production_mode_page_html(
    page_html: String,
    production_mode: bool,
    artifact_json: &str,
) -> String {
    if !production_mode {
        return page_html;
    }
    let artifact = artifact_json.replace("</script", "<\\/script");
    page_html.replacen(
        "    <script src=\"./runtime.js\" defer></script>",
        &format!(
            "    <script type=\"application/json\" id=\"presolve-production-runtime\">{artifact}    </script>\n    <script src=\"./runtime.js\" defer></script>"
        ),
        1,
    )
}

fn byte_count<'a>(values: impl IntoIterator<Item = &'a String>) -> u64 {
    u64::try_from(values.into_iter().map(String::len).sum::<usize>())
        .expect("application publication byte count exceeds u64")
}

fn report_count(value: usize) -> u32 {
    u32::try_from(value).expect("application publication report count exceeds u32")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn request(sources: Vec<(&str, &str)>, entry: &str) -> ApplicationPublicationRequestV1 {
        ApplicationPublicationRequestV1 {
            configuration: WorkspaceConfiguration::default(),
            sources: sources
                .into_iter()
                .map(|(logical_path, source)| ApplicationPublicationSourceV1 {
                    logical_path: PathBuf::from(logical_path),
                    source: source.into(),
                })
                .collect(),
            entry_path: PathBuf::from(entry),
            package_contracts: SemanticPackageResolutionTable::default(),
            package_runtime_modules: SemanticPackageRuntimeModuleTable::default(),
            profile: ApplicationPublicationProfileV1::Development,
            output_root: PathBuf::from("dist"),
        }
    }

    #[test]
    fn validates_one_explicit_rendered_component_entry_independent_of_source_order() {
        let source =
            r#"@component("x-app") class App extends Component { render() { return <main />; } }"#;
        let first = validate_application_publication_request_v1(request(
            vec![
                ("src/Utility.ts", "export const value = 1;"),
                ("src/App.tsx", source),
            ],
            "src/App.tsx",
        ))
        .unwrap();
        let second = validate_application_publication_request_v1(request(
            vec![
                ("src/App.tsx", source),
                ("src/Utility.ts", "export const value = 1;"),
            ],
            "src/App.tsx",
        ))
        .unwrap();
        assert_eq!(first.entry_component, second.entry_component);
    }

    #[test]
    fn rejects_missing_ambiguous_and_non_member_entries() {
        let missing = validate_application_publication_request_v1(request(
            vec![("src/Entry.tsx", "export const value = 1;")],
            "src/Entry.tsx",
        ))
        .unwrap_err();
        assert_eq!(missing.code, "PSAPP1005_ENTRY_APPLICATION_ROOT_MISSING");
        let ambiguous = validate_application_publication_request_v1(request(
            vec![("src/Entry.tsx", r#"@component("x-a") class A extends Component { render() { return <main />; } } @component("x-b") class B extends Component { render() { return <main />; } }"#)],
            "src/Entry.tsx",
        ))
        .unwrap_err();
        assert_eq!(ambiguous.code, "PSAPP1006_ENTRY_APPLICATION_ROOT_AMBIGUOUS");
        let non_member = validate_application_publication_request_v1(request(
            vec![("src/App.tsx", r#"@component("x-app") class App extends Component { render() { return <main />; } }"#)],
            "src/Missing.tsx",
        ))
        .unwrap_err();
        assert_eq!(non_member.code, "PSAPP1004_ENTRY_NOT_IN_SOURCE_SET");
    }

    #[test]
    fn lowers_a_validated_complete_workspace_to_a_digest_bound_manifest() {
        let request = request(
            vec![
                ("src/Helper.ts", "const value = 1;"),
                (
                    "src/App.tsx",
                    r#"@component("x-app") class App extends Component { render() { return <main>App</main>; } }"#,
                ),
            ],
            "src/App.tsx",
        );
        let product = build_application_publication_product_v1(
            validate_application_publication_request_v1(request).unwrap(),
        )
        .unwrap();
        assert_eq!(product.manifest.schema_version, 1);
        assert_eq!(product.manifest.profile, "development");
        assert!(product.artifacts.contains_key(&PathBuf::from("index.html")));
        let audit: serde_json::Value =
            serde_json::from_slice(&product.artifacts[&PathBuf::from("production-audit.json")])
                .expect("production audit artifact JSON");
        assert_eq!(audit["schemaVersion"], 1);
        assert_eq!(audit["status"], "passed");
        assert!(product
            .artifacts
            .contains_key(&PathBuf::from("application.manifest.json")));
        assert_eq!(
            product.manifest.artifacts.len() + 1,
            product.artifacts.len(),
            "the manifest inventories every generated artifact except itself"
        );
        for artifact in &product.manifest.artifacts {
            let bytes = product
                .artifacts
                .get(&PathBuf::from(&artifact.path))
                .unwrap();
            assert_eq!(artifact.digest, Digest::sha256(bytes).to_string());
        }
    }

    #[test]
    fn materializes_only_the_selected_entry_tree_from_a_multi_component_workspace() {
        let request = request(
            vec![
                (
                    "src/Home.tsx",
                    r#"@component("x-home") class Home extends Component { render() { return <main>Home</main>; } }"#,
                ),
                (
                    "src/About.tsx",
                    r#"@component("x-about") class About extends Component { render() { return <main>About</main>; } }"#,
                ),
            ],
            "src/Home.tsx",
        );

        let product = build_application_publication_product_v1(
            validate_application_publication_request_v1(request).unwrap(),
        )
        .unwrap();
        let page =
            String::from_utf8(product.artifacts[&PathBuf::from("index.html")].clone()).unwrap();

        assert!(page.contains(">Home</main>"));
        assert!(!page.contains(">About</main>"));
    }
}
