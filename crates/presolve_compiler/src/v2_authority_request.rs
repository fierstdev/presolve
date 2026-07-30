//! Compiler-owned request construction for the installed V2 authority bridge.

use std::{
    collections::BTreeSet,
    path::{Component, Path, PathBuf},
};

use presolve_parser::{ParsedFile, SourceSpan};
use serde::Serialize;

use crate::{
    action_field_sites_v1, component_inheritance_sites_v1, effect_field_sites_v1,
    form_definition_sites_v1, form_field_definition_sites_v1, form_validation_definition_sites_v1,
    slot_field_sites_v1, AuthoredSourceRangeV1, CanonicalAuthoredSemanticModelV1,
};

pub const V2_AUTHORITY_REQUEST_SCHEMA_VERSION: u32 = 11;
const V2_VALIDATION_RULE_NAMES: &[&str] = &[
    "required",
    "min",
    "max",
    "minLength",
    "maxLength",
    "pattern",
    "email",
    "equals",
    "notEquals",
];

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct V2AuthorityPositionV1 {
    pub file: PathBuf,
    /// Zero-based UTF-16 code-unit offset used by the TypeScript API.
    pub position: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct V2AuthorityNamedPositionV1 {
    pub name: String,
    pub file: PathBuf,
    pub position: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct V2AuthoritySiteV1 {
    pub id: String,
    pub file: PathBuf,
    pub position: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct V2AuthorityFormFieldSiteV1 {
    pub id: String,
    pub file: PathBuf,
    pub position: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub initial_position: Option<usize>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct V2AuthorityStandardValidationSiteV1 {
    pub id: String,
    pub file: PathBuf,
    pub position: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub import_position: Option<usize>,
    pub module_specifier: String,
    pub export_name: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum V2AuthorityPackageInvocationCompletionV1 {
    Synchronous,
    Promise,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct V2AuthorityPackageInvocationSiteV1 {
    pub id: String,
    pub file: PathBuf,
    pub position: usize,
    pub import_position: usize,
    pub module_specifier: String,
    pub export_name: String,
    pub argument_types: Vec<String>,
    pub completion: V2AuthorityPackageInvocationCompletionV1,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub signal_position: Option<usize>,
}

/// A syntactic member-call candidate.  Rust deliberately records only the
/// structural object and property positions; the TypeScript authority decides
/// whether either resolved symbol has framework meaning.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct V2AuthorityMemberSiteV1 {
    pub id: String,
    pub file: PathBuf,
    pub object_position: usize,
    pub property_position: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct V2AuthorityCanonicalV1 {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub component: Option<V2AuthorityPositionV1>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub state: Option<V2AuthorityPositionV1>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub action: Option<V2AuthorityPositionV1>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub effect: Option<V2AuthorityPositionV1>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub slot: Option<V2AuthorityPositionV1>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub define_form: Option<V2AuthorityPositionV1>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub field: Option<V2AuthorityPositionV1>,
    pub validation_rules: Vec<V2AuthorityNamedPositionV1>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub environment: Option<V2AuthorityPositionV1>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct V2AuthorityRequestV1 {
    pub schema_version: u32,
    pub config_file: PathBuf,
    pub canonical: V2AuthorityCanonicalV1,
    pub components: Vec<V2AuthoritySiteV1>,
    pub states: Vec<V2AuthoritySiteV1>,
    pub actions: Vec<V2AuthoritySiteV1>,
    pub effects: Vec<V2AuthoritySiteV1>,
    pub slots: Vec<V2AuthoritySiteV1>,
    pub forms: Vec<V2AuthoritySiteV1>,
    pub form_fields: Vec<V2AuthorityFormFieldSiteV1>,
    pub validations: Vec<V2AuthoritySiteV1>,
    pub standard_validations: Vec<V2AuthorityStandardValidationSiteV1>,
    pub package_invocations: Vec<V2AuthorityPackageInvocationSiteV1>,
    pub environment_public: Vec<V2AuthorityMemberSiteV1>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum V2AuthorityRequestErrorV1 {
    MissingCanonicalExport(&'static str),
    InvalidSourceOffset(usize),
    FieldSiteSelection(String),
}

impl std::fmt::Display for V2AuthorityRequestErrorV1 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::MissingCanonicalExport(name) => write!(
                f,
                "missing `{name}` import from presolve needed for V2 authority query"
            ),
            Self::InvalidSourceOffset(offset) => write!(
                f,
                "V2 authority query offset {offset} is not a UTF-8 source boundary"
            ),
            Self::FieldSiteSelection(message) => write!(
                f,
                "unable to select V2 State, Action, or Effect authority sites: {message}"
            ),
        }
    }
}
impl std::error::Error for V2AuthorityRequestErrorV1 {}

/// Select bridge query positions; import text only nominates a site. The bridge
/// must still resolve and registry-classify every resulting symbol.
pub fn build_v2_authority_request_v1(
    parsed: &ParsedFile,
    config_file: PathBuf,
    component_model: &CanonicalAuthoredSemanticModelV1,
) -> Result<V2AuthorityRequestV1, V2AuthorityRequestErrorV1> {
    let component = Some(canonical_import(parsed, "Component")?.ok_or(
        V2AuthorityRequestErrorV1::MissingCanonicalExport("Component"),
    )?);
    let state = canonical_import(parsed, "state")?;
    let action = canonical_import(parsed, "action")?;
    let effect = canonical_import(parsed, "effect")?;
    let slot = canonical_import(parsed, "slot")?;
    let define_form = canonical_import(parsed, "defineForm")?;
    let field = canonical_import(parsed, "field")?;
    let validation_rules = canonical_validation_imports(parsed)?;
    let environment = canonical_import(parsed, "environment")?;
    let components = component_inheritance_sites_v1(parsed)
        .into_iter()
        .map(|site| {
            site_for(
                "component",
                site.heritage_source,
                &parsed.path,
                &parsed.syntax.source,
            )
        })
        .collect::<Result<Vec<_>, _>>()?;
    let states = state
        .is_some()
        .then(|| crate::state_initializer_sites_v1(parsed, component_model))
        .transpose()
        .map_err(|error| V2AuthorityRequestErrorV1::FieldSiteSelection(error.to_string()))?
        .unwrap_or_default()
        .into_iter()
        .map(|site| {
            site_for(
                "state",
                site.callee_source,
                &parsed.path,
                &parsed.syntax.source,
            )
        })
        .collect::<Result<Vec<_>, _>>()?;
    let actions = action
        .is_some()
        .then(|| action_field_sites_v1(parsed, component_model))
        .transpose()
        .map_err(|error| V2AuthorityRequestErrorV1::FieldSiteSelection(error.to_string()))?
        .unwrap_or_default()
        .into_iter()
        .map(|site| {
            site_for(
                "action",
                site.callee_source,
                &parsed.path,
                &parsed.syntax.source,
            )
        })
        .collect::<Result<Vec<_>, _>>()?;
    let effects = effect
        .is_some()
        .then(|| effect_field_sites_v1(parsed, component_model))
        .transpose()
        .map_err(|error| V2AuthorityRequestErrorV1::FieldSiteSelection(error.to_string()))?
        .unwrap_or_default()
        .into_iter()
        .map(|site| {
            site_for(
                "effect",
                site.callee_source,
                &parsed.path,
                &parsed.syntax.source,
            )
        })
        .collect::<Result<Vec<_>, _>>()?;
    let slots = slot
        .is_some()
        .then(|| slot_field_sites_v1(parsed, component_model))
        .transpose()
        .map_err(|error| V2AuthorityRequestErrorV1::FieldSiteSelection(error.to_string()))?
        .unwrap_or_default()
        .into_iter()
        .map(|site| {
            site_for(
                "slot",
                site.callee_source,
                &parsed.path,
                &parsed.syntax.source,
            )
        })
        .collect::<Result<Vec<_>, _>>()?;
    let forms = define_form
        .is_some()
        .then(|| form_definition_sites_v1(parsed, component_model))
        .transpose()
        .map_err(|error| V2AuthorityRequestErrorV1::FieldSiteSelection(error.to_string()))?
        .unwrap_or_default()
        .into_iter()
        .map(|site| {
            site_for(
                "form",
                site.callee_source,
                &parsed.path,
                &parsed.syntax.source,
            )
        })
        .collect::<Result<Vec<_>, _>>()?;
    let form_fields = field
        .is_some()
        .then(|| form_field_definition_sites_v1(parsed, component_model))
        .transpose()
        .map_err(|error| V2AuthorityRequestErrorV1::FieldSiteSelection(error.to_string()))?
        .unwrap_or_default()
        .into_iter()
        .map(|site| {
            let selected = site_for(
                "form-field",
                site.callee_source,
                &parsed.path,
                &parsed.syntax.source,
            )?;
            Ok(V2AuthorityFormFieldSiteV1 {
                id: selected.id,
                file: selected.file,
                position: selected.position,
                initial_position: site
                    .initial_source
                    .map(|range| utf16_position(&parsed.syntax.source, range.start))
                    .transpose()?,
            })
        })
        .collect::<Result<Vec<_>, _>>()?;
    let validation_sites = form_validation_definition_sites_v1(parsed, component_model)
        .map_err(|error| V2AuthorityRequestErrorV1::FieldSiteSelection(error.to_string()))?;
    let validations = if validation_rules.is_empty() {
        Vec::new()
    } else {
        let local_names = parsed
            .imports
            .iter()
            .filter(|import| import.source == "presolve")
            .flat_map(|import| &import.specifiers)
            .filter(|specifier| V2_VALIDATION_RULE_NAMES.contains(&specifier.imported.as_str()))
            .map(|specifier| specifier.local.as_str())
            .collect::<BTreeSet<_>>();
        validation_sites
            .iter()
            .filter(|site| {
                parsed
                    .syntax
                    .source
                    .get(site.callee_source.start..site.callee_source.end)
                    .is_some_and(|callee| local_names.contains(callee))
            })
            .map(|site| {
                site_for(
                    "validation",
                    site.callee_source,
                    &parsed.path,
                    &parsed.syntax.source,
                )
            })
            .collect::<Result<Vec<_>, _>>()?
    };
    let standard_validations = validation_sites
        .into_iter()
        .filter_map(|site| {
            let local = parsed
                .syntax
                .source
                .get(site.callee_source.start..site.callee_source.end)?;
            let import = parsed
                .imports
                .iter()
                .filter(|import| {
                    import.source != "presolve"
                        && !relative_import_targets_server(&parsed.path, &import.source)
                })
                .flat_map(|import| {
                    import
                        .specifiers
                        .iter()
                        .map(move |specifier| (import.source.as_str(), specifier))
                })
                .find(|(_, specifier)| {
                    specifier.local == local && specifier.imported != "default"
                })?;
            Some((site, import.0.to_owned(), import.1.imported.clone()))
        })
        .map(|(site, module_specifier, export_name)| {
            let selected = site_for(
                "standard-validation",
                site.callee_source,
                &parsed.path,
                &parsed.syntax.source,
            )?;
            Ok(V2AuthorityStandardValidationSiteV1 {
                id: selected.id,
                file: selected.file,
                position: selected.position,
                import_position: None,
                module_specifier,
                export_name,
            })
        })
        .collect::<Result<Vec<_>, V2AuthorityRequestErrorV1>>()?;
    let package_invocations = terminal_package_invocation_sites(parsed)?;
    let environment_public = environment_public_member_sites(parsed, environment.is_some())?;
    Ok(V2AuthorityRequestV1 {
        schema_version: V2_AUTHORITY_REQUEST_SCHEMA_VERSION,
        config_file,
        canonical: V2AuthorityCanonicalV1 {
            component,
            state,
            action,
            effect,
            slot,
            define_form,
            field,
            validation_rules,
            environment,
        },
        components,
        states,
        actions,
        effects,
        slots,
        forms,
        form_fields,
        validations,
        standard_validations,
        package_invocations,
        environment_public,
    })
}

/// Builds the first authority query for a source file.  It asks only for
/// component heritage; State and Action candidates cannot be selected until
/// canonical component ownership has been proven by this response.
pub fn build_v2_authority_component_request_v1(
    parsed: &ParsedFile,
    config_file: PathBuf,
) -> Result<V2AuthorityRequestV1, V2AuthorityRequestErrorV1> {
    let component = Some(canonical_import(parsed, "Component")?.ok_or(
        V2AuthorityRequestErrorV1::MissingCanonicalExport("Component"),
    )?);
    let components = component_inheritance_sites_v1(parsed)
        .into_iter()
        .map(|site| {
            site_for(
                "component",
                site.heritage_source,
                &parsed.path,
                &parsed.syntax.source,
            )
        })
        .collect::<Result<Vec<_>, _>>()?;
    Ok(V2AuthorityRequestV1 {
        schema_version: V2_AUTHORITY_REQUEST_SCHEMA_VERSION,
        config_file,
        canonical: V2AuthorityCanonicalV1 {
            component,
            state: canonical_import(parsed, "state")?,
            action: canonical_import(parsed, "action")?,
            effect: canonical_import(parsed, "effect")?,
            slot: canonical_import(parsed, "slot")?,
            define_form: canonical_import(parsed, "defineForm")?,
            field: canonical_import(parsed, "field")?,
            validation_rules: canonical_validation_imports(parsed)?,
            environment: canonical_import(parsed, "environment")?,
        },
        components,
        states: Vec::new(),
        actions: Vec::new(),
        effects: Vec::new(),
        slots: Vec::new(),
        forms: Vec::new(),
        form_fields: Vec::new(),
        validations: Vec::new(),
        standard_validations: Vec::new(),
        package_invocations: Vec::new(),
        environment_public: Vec::new(),
    })
}

/// Builds an authority request for a plain V2 module that imports the
/// environment intrinsic but does not declare a Component. The generic parser
/// candidates retain no framework meaning; this request merely makes them
/// available to TypeScript resolution.
pub fn build_v2_environment_authority_request_v1(
    parsed: &ParsedFile,
    config_file: PathBuf,
) -> Result<Option<V2AuthorityRequestV1>, V2AuthorityRequestErrorV1> {
    let environment = canonical_import(parsed, "environment")?;
    let Some(environment) = environment else {
        return Ok(None);
    };
    Ok(Some(V2AuthorityRequestV1 {
        schema_version: V2_AUTHORITY_REQUEST_SCHEMA_VERSION,
        config_file,
        canonical: V2AuthorityCanonicalV1 {
            component: None,
            state: None,
            action: None,
            effect: None,
            slot: None,
            define_form: None,
            field: None,
            validation_rules: Vec::new(),
            environment: Some(environment),
        },
        components: Vec::new(),
        states: Vec::new(),
        actions: Vec::new(),
        effects: Vec::new(),
        slots: Vec::new(),
        forms: Vec::new(),
        form_fields: Vec::new(),
        validations: Vec::new(),
        standard_validations: Vec::new(),
        package_invocations: Vec::new(),
        environment_public: environment_public_member_sites(parsed, true)?,
    }))
}

/// Select complete zero-argument direct calls whose callee is a named external
/// import. This is syntax-only nomination; TypeScript must prove the symbol.
fn terminal_package_invocation_sites(
    parsed: &ParsedFile,
) -> Result<Vec<V2AuthorityPackageInvocationSiteV1>, V2AuthorityRequestErrorV1> {
    let mut sites = parsed
        .classes
        .iter()
        .flat_map(|class| &class.properties)
        .filter_map(|property| {
            let handler = property
                .initializer_call
                .as_ref()?
                .inline_handler
                .as_ref()?;
            let invocation = handler.direct_call.as_ref()?;
            let (completion, forwarded_parameters, signal_parameter) =
                if handler.is_async && invocation.awaited {
                    let (signal, forwarded) = handler.parameters.split_last()?;
                    let annotation = signal.type_annotation.as_ref()?;
                    if signal.name != "signal" || annotation.text.trim() != "AbortSignal" {
                        return None;
                    }
                    (
                        V2AuthorityPackageInvocationCompletionV1::Promise,
                        forwarded,
                        Some(signal),
                    )
                } else if !handler.is_async && !invocation.awaited {
                    (
                        V2AuthorityPackageInvocationCompletionV1::Synchronous,
                        handler.parameters.as_slice(),
                        None,
                    )
                } else {
                    return None;
                };
            let argument_types = forwarded_parameters
                .iter()
                .map(|parameter| {
                    let annotation = parameter.type_annotation.as_ref()?.text.trim();
                    matches!(annotation, "string" | "number" | "boolean" | "null")
                        .then(|| annotation.to_owned())
                })
                .collect::<Option<Vec<_>>>()?;
            let expected_argument_names = forwarded_parameters
                .iter()
                .map(|parameter| parameter.name.as_str())
                .chain(signal_parameter.map(|parameter| parameter.name.as_str()))
                .collect::<Vec<_>>();
            if invocation
                .arguments
                .iter()
                .map(|argument| argument.name.as_str())
                .ne(expected_argument_names)
            {
                return None;
            }
            let (module_specifier, export_name, import_span) = parsed
                .imports
                .iter()
                .filter(|import| {
                    import.source != "presolve"
                        && !relative_import_targets_server(&parsed.path, &import.source)
                })
                .flat_map(|import| {
                    import
                        .specifiers
                        .iter()
                        .map(move |specifier| (import.source.as_str(), specifier))
                })
                .find(|(_, specifier)| {
                    specifier.local == invocation.callee && specifier.imported != "default"
                })
                .map(|(source, specifier)| {
                    (
                        source.to_owned(),
                        specifier.imported.clone(),
                        specifier.local_span,
                    )
                })?;
            Some((
                invocation.callee_span,
                module_specifier,
                export_name,
                import_span,
                argument_types,
                completion,
                signal_parameter
                    .and_then(|parameter| parameter.type_annotation.as_ref())
                    .map(|annotation| annotation.span),
            ))
        })
        .map(
            |(
                callee_source,
                module_specifier,
                export_name,
                import_span,
                argument_types,
                completion,
                signal_span,
            )| {
                let selected = site_for(
                    "package-invocation",
                    AuthoredSourceRangeV1 {
                        start: callee_source.start,
                        end: callee_source.end,
                        line: callee_source.line,
                        column: callee_source.column,
                    },
                    &parsed.path,
                    &parsed.syntax.source,
                )?;
                Ok(V2AuthorityPackageInvocationSiteV1 {
                    id: selected.id,
                    file: selected.file,
                    position: selected.position,
                    import_position: utf16_position(&parsed.syntax.source, import_span.start)?,
                    module_specifier,
                    export_name,
                    argument_types,
                    completion,
                    signal_position: signal_span
                        .map(|span| utf16_position(&parsed.syntax.source, span.start))
                        .transpose()?,
                })
            },
        )
        .collect::<Result<Vec<_>, V2AuthorityRequestErrorV1>>()?;
    sites.sort_by(|left, right| left.id.cmp(&right.id));
    Ok(sites)
}

fn environment_public_member_sites(
    parsed: &ParsedFile,
    enabled: bool,
) -> Result<Vec<V2AuthorityMemberSiteV1>, V2AuthorityRequestErrorV1> {
    if !enabled {
        return Ok(Vec::new());
    }
    parsed
        .call_expressions
        .iter()
        .filter_map(|call| {
            Some((
                call.member_object_span?,
                call.member_property_span?,
                call.span,
            ))
        })
        .map(|(object, property, call)| {
            member_site_for(
                "environment-public",
                object,
                property,
                call,
                &parsed.path,
                &parsed.syntax.source,
            )
        })
        .collect()
}

fn relative_import_targets_server(source_path: &Path, module_specifier: &str) -> bool {
    if !module_specifier.starts_with('.') {
        return false;
    }
    let mut normalized = PathBuf::new();
    for component in source_path
        .parent()
        .unwrap_or_else(|| Path::new(""))
        .join(module_specifier)
        .components()
    {
        match component {
            Component::Normal(value) => normalized.push(value),
            Component::CurDir => {}
            Component::ParentDir => {
                if !normalized.pop() {
                    return true;
                }
            }
            Component::RootDir | Component::Prefix(_) => return true,
        }
    }
    normalized
        .components()
        .next()
        .is_some_and(|component| component.as_os_str() == "server")
}

fn canonical_import(
    parsed: &ParsedFile,
    name: &str,
) -> Result<Option<V2AuthorityPositionV1>, V2AuthorityRequestErrorV1> {
    parsed
        .imports
        .iter()
        .filter(|import| import.source == "presolve")
        .flat_map(|import| &import.specifiers)
        .find(|specifier| specifier.imported == name)
        .map(|specifier| {
            utf16_position(&parsed.syntax.source, specifier.local_span.start).map(|position| {
                V2AuthorityPositionV1 {
                    file: parsed.path.clone(),
                    position,
                }
            })
        })
        .transpose()
}

fn canonical_validation_imports(
    parsed: &ParsedFile,
) -> Result<Vec<V2AuthorityNamedPositionV1>, V2AuthorityRequestErrorV1> {
    V2_VALIDATION_RULE_NAMES
        .iter()
        .filter_map(|name| {
            canonical_import(parsed, name)
                .transpose()
                .map(|result| result.map(|position| ((*name).to_owned(), position)))
        })
        .map(|result| {
            result.map(|(name, position)| V2AuthorityNamedPositionV1 {
                name,
                file: position.file,
                position: position.position,
            })
        })
        .collect()
}

fn site_for(
    kind: &str,
    range: AuthoredSourceRangeV1,
    path: &std::path::Path,
    source: &str,
) -> Result<V2AuthoritySiteV1, V2AuthorityRequestErrorV1> {
    Ok(V2AuthoritySiteV1 {
        id: format!("{kind}:{}:{}", range.start, range.end),
        file: path.to_path_buf(),
        position: utf16_position(source, range.start)?,
    })
}

fn member_site_for(
    kind: &str,
    object: SourceSpan,
    property: SourceSpan,
    call: SourceSpan,
    path: &std::path::Path,
    source: &str,
) -> Result<V2AuthorityMemberSiteV1, V2AuthorityRequestErrorV1> {
    Ok(V2AuthorityMemberSiteV1 {
        id: format!("{kind}:{}:{}", call.start, call.end),
        file: path.to_path_buf(),
        object_position: utf16_position(source, object.start)?,
        property_position: utf16_position(source, property.start)?,
    })
}

fn utf16_position(source: &str, byte_offset: usize) -> Result<usize, V2AuthorityRequestErrorV1> {
    let Some(prefix) = source.get(..byte_offset) else {
        return Err(V2AuthorityRequestErrorV1::InvalidSourceOffset(byte_offset));
    };
    Ok(prefix.encode_utf16().count())
}

#[cfg(test)]
mod tests {
    use std::path::{Path, PathBuf};

    use presolve_parser::parse_file;

    use crate::{
        component_inheritance_sites_v1, lower_component_inheritance_v1,
        ResolvedComponentInheritanceV1, ResolvedIntrinsicIdentityV1,
    };

    use super::{
        build_v2_authority_component_request_v1, build_v2_authority_request_v1,
        build_v2_environment_authority_request_v1, relative_import_targets_server,
        V2AuthorityRequestErrorV1,
    };

    #[test]
    fn builds_source_faithful_queries_for_aliases_and_canonical_fields() {
        let source = r#"
import { Component as FrameworkBase, state as reactiveCell, action as activate, effect as observe, slot } from "presolve";
class Counter extends FrameworkBase { children: SlotContent = slot(); count = reactiveCell(0); increment = activate(() => {}); sync = observe(() => {}); }
"#;
        let parsed = parse_file("src/Counter.tsx", source);
        let heritage = component_inheritance_sites_v1(&parsed).pop().unwrap();
        let components = lower_component_inheritance_v1(
            &parsed,
            [ResolvedComponentInheritanceV1 {
                heritage_source: heritage.heritage_source,
                component_identity: ResolvedIntrinsicIdentityV1 {
                    name: "Component".into(),
                    flags: 32,
                    declaration_modules: vec!["presolve".into()],
                },
            }],
        )
        .unwrap()
        .model;
        let request =
            build_v2_authority_request_v1(&parsed, PathBuf::from("tsconfig.json"), &components)
                .unwrap();
        assert_eq!(
            &source[request.canonical.component.as_ref().unwrap().position..][..13],
            "FrameworkBase"
        );
        assert_eq!(
            &source[request.canonical.state.as_ref().unwrap().position..][..12],
            "reactiveCell"
        );
        assert_eq!(
            &source[request.canonical.action.as_ref().unwrap().position..][..8],
            "activate"
        );
        assert_eq!(
            &source[request.canonical.effect.as_ref().unwrap().position..][..7],
            "observe"
        );
        assert_eq!(
            &source[request.canonical.slot.as_ref().unwrap().position..][..4],
            "slot"
        );
        assert_eq!(request.components.len(), 1);
        assert_eq!(request.states.len(), 4);
        assert_eq!(request.actions.len(), 4);
        assert_eq!(request.effects.len(), 4);
        assert_eq!(request.slots.len(), 1);
        assert!(request.forms.is_empty());
    }

    #[test]
    fn selects_form_definition_sites_only_when_define_form_is_imported() {
        let source = r#"
import { Component, defineForm as declareForm, field as declareField, required as mustExist } from "presolve";
import { profileSchema } from "./schema.js";
class Profile extends Component {
  profile = declareForm({ fields: { name: declareField({ initial: "", validate: [mustExist(), profileSchema] }) } });
  lookalike = helper({ fields: {} });
}
"#;
        let parsed = parse_file("src/Profile.tsx", source);
        let heritage = component_inheritance_sites_v1(&parsed).pop().unwrap();
        let components = lower_component_inheritance_v1(
            &parsed,
            [ResolvedComponentInheritanceV1 {
                heritage_source: heritage.heritage_source,
                component_identity: ResolvedIntrinsicIdentityV1 {
                    name: "Component".into(),
                    flags: 32,
                    declaration_modules: vec!["presolve".into()],
                },
            }],
        )
        .unwrap()
        .model;
        let request =
            build_v2_authority_request_v1(&parsed, PathBuf::from("tsconfig.json"), &components)
                .unwrap();
        assert!(request.canonical.define_form.is_some());
        assert!(request.canonical.field.is_some());
        assert_eq!(request.canonical.validation_rules.len(), 1);
        assert_eq!(request.forms.len(), 2);
        assert_eq!(request.form_fields.len(), 1);
        assert_eq!(request.validations.len(), 1);
        assert_eq!(request.standard_validations.len(), 1);
        assert_eq!(
            request.standard_validations[0].module_specifier,
            "./schema.js"
        );
        assert_eq!(request.standard_validations[0].export_name, "profileSchema");
        assert_eq!(
            &source[request.forms[0].position..][.."declareForm".len()],
            "declareForm"
        );
        assert_eq!(
            &source[request.form_fields[0].position..][.."declareField".len()],
            "declareField"
        );
        assert!(request.form_fields[0].initial_position.is_some());
        assert_eq!(
            &source[request.validations[0].position..][.."mustExist".len()],
            "mustExist"
        );
    }

    #[test]
    fn rejects_missing_canonical_imports() {
        let parsed = parse_file("src/Counter.tsx", "class Counter extends Base {}");
        let error = build_v2_authority_request_v1(
            &parsed,
            PathBuf::from("tsconfig.json"),
            &crate::CanonicalAuthoredSemanticModelV1 {
                schema_version: 1,
                source_path: parsed.path.clone(),
                declarations: Vec::new(),
            },
        )
        .unwrap_err();
        assert!(matches!(
            error,
            V2AuthorityRequestErrorV1::MissingCanonicalExport("Component")
        ));
    }

    #[test]
    fn converts_parser_byte_offsets_to_typescript_utf16_positions() {
        let source = r#"
// 🚀
import { Component as FrameworkBase, state, action } from "presolve";
class Counter extends FrameworkBase { count = state(0); increment = action(() => {}); }
"#;
        let parsed = parse_file("src/Counter.tsx", source);
        let heritage = component_inheritance_sites_v1(&parsed).pop().unwrap();
        let components = lower_component_inheritance_v1(
            &parsed,
            [ResolvedComponentInheritanceV1 {
                heritage_source: heritage.heritage_source,
                component_identity: ResolvedIntrinsicIdentityV1 {
                    name: "Component".into(),
                    flags: 32,
                    declaration_modules: vec!["presolve".into()],
                },
            }],
        )
        .unwrap()
        .model;
        let request =
            build_v2_authority_request_v1(&parsed, PathBuf::from("tsconfig.json"), &components)
                .unwrap();
        let byte_position = source.find("FrameworkBase").unwrap();
        assert_eq!(
            request.canonical.component.as_ref().unwrap().position,
            source[..byte_position].encode_utf16().count()
        );
        assert_ne!(
            request.canonical.component.as_ref().unwrap().position,
            byte_position
        );
    }

    #[test]
    fn component_phase_needs_only_component_and_selects_no_fields() {
        let parsed = parse_file(
            "src/Counter.tsx",
            "import { Component } from \"presolve\"; class Counter extends Component {}",
        );
        let request =
            build_v2_authority_component_request_v1(&parsed, PathBuf::from("tsconfig.json"))
                .unwrap();
        assert_eq!(request.components.len(), 1);
        assert!(request.canonical.state.is_none());
        assert!(request.canonical.action.is_none());
        assert!(request.states.is_empty());
        assert!(request.actions.is_empty());
    }

    #[test]
    fn builds_environment_authority_for_plain_modules_without_component_imports() {
        let source = r#"
import { environment as runtimeEnvironment } from "presolve";
const appName = runtimeEnvironment.public("PRESOLVE_PUBLIC_APP_NAME");
const local = lookalike.public("PRESOLVE_PUBLIC_LOOKALIKE");
"#;
        let parsed = parse_file("src/environment.ts", source);
        let request =
            build_v2_environment_authority_request_v1(&parsed, PathBuf::from("tsconfig.json"))
                .unwrap()
                .unwrap();
        assert!(request.canonical.component.is_none());
        assert_eq!(request.environment_public.len(), 2);
    }

    #[test]
    fn forwards_all_static_member_calls_as_environment_authority_candidates() {
        let source = r#"
import { Component, environment as runtimeEnvironment } from "presolve";
class Counter extends Component {}
const canonical = runtimeEnvironment.public("PRESOLVE_PUBLIC_APP_NAME");
const lookalike = localConfig.public("PRESOLVE_PUBLIC_LOOKALIKE");
"#;
        let parsed = parse_file("src/Counter.tsx", source);
        let heritage = component_inheritance_sites_v1(&parsed).pop().unwrap();
        let components = lower_component_inheritance_v1(
            &parsed,
            [ResolvedComponentInheritanceV1 {
                heritage_source: heritage.heritage_source,
                component_identity: ResolvedIntrinsicIdentityV1 {
                    name: "Component".into(),
                    flags: 32,
                    declaration_modules: vec!["presolve".into()],
                },
            }],
        )
        .unwrap()
        .model;
        let request =
            build_v2_authority_request_v1(&parsed, PathBuf::from("tsconfig.json"), &components)
                .unwrap();

        assert_eq!(request.environment_public.len(), 2);
        assert_eq!(
            &source[request.canonical.environment.unwrap().position..][..18],
            "runtimeEnvironment"
        );
        assert_eq!(
            request
                .environment_public
                .iter()
                .map(|site| &source[site.object_position..])
                .map(|suffix| &suffix[..suffix.find('.').unwrap()])
                .collect::<Vec<_>>(),
            vec!["runtimeEnvironment", "localConfig"]
        );
    }

    #[test]
    fn package_invocation_authority_excludes_server_owned_relative_modules() {
        assert!(relative_import_targets_server(
            Path::new("app/routes/docs/index.tsx"),
            "../../../server/analytics.js"
        ));
        assert!(!relative_import_targets_server(
            Path::new("app/routes/docs/index.tsx"),
            "../../components/analytics.js"
        ));
        assert!(!relative_import_targets_server(
            Path::new("app/routes/index.tsx"),
            "analytics-kit"
        ));
    }

    #[test]
    fn nominates_only_exact_primitive_and_abortable_package_action_shapes() {
        let source = r#"
import { action, Component } from "presolve";
import { recordMetric, recordMetricAsync } from "analytics-kit";
class Metrics extends Component {
  record = action((category: string, value: number, enabled: boolean, empty: null) => {
    recordMetric(category, value, enabled, empty);
  });
  recordAsync = action(async (category: string, signal: AbortSignal) => {
    await recordMetricAsync(category, signal);
  });
  reordered = action((category: string, value: number) => {
    recordMetric(value, category);
  });
}
"#;
        let parsed = parse_file("app/routes/index.tsx", source);
        let heritage = component_inheritance_sites_v1(&parsed).pop().unwrap();
        let components = lower_component_inheritance_v1(
            &parsed,
            [ResolvedComponentInheritanceV1 {
                heritage_source: heritage.heritage_source,
                component_identity: ResolvedIntrinsicIdentityV1 {
                    name: "Component".into(),
                    flags: 32,
                    declaration_modules: vec!["presolve".into()],
                },
            }],
        )
        .unwrap()
        .model;
        let request =
            build_v2_authority_request_v1(&parsed, PathBuf::from("tsconfig.json"), &components)
                .unwrap();
        assert_eq!(request.package_invocations.len(), 2);
        assert_eq!(
            request.package_invocations[0].argument_types,
            ["string", "number", "boolean", "null"]
        );
        assert_eq!(
            request.package_invocations[0].completion,
            super::V2AuthorityPackageInvocationCompletionV1::Synchronous
        );
        assert!(request.package_invocations[0].signal_position.is_none());
        assert_eq!(request.package_invocations[1].argument_types, ["string"]);
        assert_eq!(
            request.package_invocations[1].completion,
            super::V2AuthorityPackageInvocationCompletionV1::Promise
        );
        assert!(request.package_invocations[1].signal_position.is_some());
    }
}
