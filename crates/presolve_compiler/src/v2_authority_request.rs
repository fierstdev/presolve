//! Compiler-owned request construction for the installed V2 authority bridge.

use std::path::PathBuf;

use presolve_parser::{ParsedFile, SourceSpan};
use serde::Serialize;

use crate::{
    action_field_sites_v1, component_inheritance_sites_v1, effect_field_sites_v1,
    slot_field_sites_v1, AuthoredSourceRangeV1, CanonicalAuthoredSemanticModelV1,
};

pub const V2_AUTHORITY_REQUEST_SCHEMA_VERSION: u32 = 4;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct V2AuthorityPositionV1 {
    pub file: PathBuf,
    /// Zero-based UTF-16 code-unit offset used by the TypeScript API.
    pub position: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct V2AuthoritySiteV1 {
    pub id: String,
    pub file: PathBuf,
    pub position: usize,
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
            environment,
        },
        components,
        states,
        actions,
        effects,
        slots,
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
            environment: canonical_import(parsed, "environment")?,
        },
        components,
        states: Vec::new(),
        actions: Vec::new(),
        effects: Vec::new(),
        slots: Vec::new(),
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
            environment: Some(environment),
        },
        components: Vec::new(),
        states: Vec::new(),
        actions: Vec::new(),
        effects: Vec::new(),
        slots: Vec::new(),
        environment_public: environment_public_member_sites(parsed, true)?,
    }))
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
    use std::path::PathBuf;

    use presolve_parser::parse_file;

    use crate::{
        component_inheritance_sites_v1, lower_component_inheritance_v1,
        ResolvedComponentInheritanceV1, ResolvedIntrinsicIdentityV1,
    };

    use super::{
        build_v2_authority_component_request_v1, build_v2_authority_request_v1,
        build_v2_environment_authority_request_v1, V2AuthorityRequestErrorV1,
    };

    #[test]
    fn builds_source_faithful_queries_for_aliases_and_canonical_fields() {
        let source = r#"
import { Component as FrameworkBase, state as reactiveCell, action as activate, effect as observe } from "presolve";
class Counter extends FrameworkBase { count = reactiveCell(0); increment = activate(() => {}); sync = observe(() => {}); }
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
        assert_eq!(request.components.len(), 1);
        assert_eq!(request.states.len(), 3);
        assert_eq!(request.actions.len(), 3);
        assert_eq!(request.effects.len(), 3);
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
}
