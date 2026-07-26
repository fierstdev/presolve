//! Compiler-owned request construction for the installed V2 authority bridge.

use std::path::PathBuf;

use presolve_parser::ParsedFile;
use serde::Serialize;

use crate::{
    action_field_sites_v1, component_inheritance_sites_v1, AuthoredSourceRangeV1,
    CanonicalAuthoredSemanticModelV1,
};

pub const V2_AUTHORITY_REQUEST_SCHEMA_VERSION: u32 = 1;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct V2AuthorityPositionV1 {
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
pub struct V2AuthorityCanonicalV1 {
    pub component: V2AuthorityPositionV1,
    pub state: V2AuthorityPositionV1,
    pub action: V2AuthorityPositionV1,
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
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum V2AuthorityRequestErrorV1 {
    MissingCanonicalExport(&'static str),
}

impl std::fmt::Display for V2AuthorityRequestErrorV1 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::MissingCanonicalExport(name) => write!(
                f,
                "missing `{name}` import from presolve needed for V2 authority query"
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
    let canonical = |name| {
        parsed
            .imports
            .iter()
            .filter(|import| import.source == "presolve")
            .flat_map(|import| &import.specifiers)
            .find(|specifier| specifier.imported == name)
            .map(|specifier| V2AuthorityPositionV1 {
                file: parsed.path.clone(),
                position: specifier.local_span.start,
            })
    };
    let component = canonical("Component").ok_or(
        V2AuthorityRequestErrorV1::MissingCanonicalExport("Component"),
    )?;
    let state =
        canonical("state").ok_or(V2AuthorityRequestErrorV1::MissingCanonicalExport("state"))?;
    let action =
        canonical("action").ok_or(V2AuthorityRequestErrorV1::MissingCanonicalExport("action"))?;
    let components = component_inheritance_sites_v1(parsed)
        .into_iter()
        .map(|site| site_for("component", site.heritage_source, &parsed.path))
        .collect();
    let states = crate::state_initializer_sites_v1(parsed, component_model)
        .unwrap_or_default()
        .into_iter()
        .map(|site| site_for("state", site.callee_source, &parsed.path))
        .collect();
    let actions = action_field_sites_v1(parsed, component_model)
        .unwrap_or_default()
        .into_iter()
        .map(|site| site_for("action", site.callee_source, &parsed.path))
        .collect();
    Ok(V2AuthorityRequestV1 {
        schema_version: V2_AUTHORITY_REQUEST_SCHEMA_VERSION,
        config_file,
        canonical: V2AuthorityCanonicalV1 {
            component,
            state,
            action,
        },
        components,
        states,
        actions,
    })
}

fn site_for(kind: &str, range: AuthoredSourceRangeV1, path: &std::path::Path) -> V2AuthoritySiteV1 {
    V2AuthoritySiteV1 {
        id: format!("{kind}:{}:{}", range.start, range.end),
        file: path.to_path_buf(),
        position: range.start,
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use presolve_parser::parse_file;

    use crate::{
        component_inheritance_sites_v1, lower_component_inheritance_v1,
        ResolvedComponentInheritanceV1, ResolvedIntrinsicIdentityV1,
    };

    use super::{build_v2_authority_request_v1, V2AuthorityRequestErrorV1};

    #[test]
    fn builds_source_faithful_queries_for_aliases_and_canonical_fields() {
        let source = r#"
import { Component as FrameworkBase, state as reactiveCell, action as activate } from "presolve";
class Counter extends FrameworkBase { count = reactiveCell(0); increment = activate(() => {}); }
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
            &source[request.canonical.component.position..][..13],
            "FrameworkBase"
        );
        assert_eq!(
            &source[request.canonical.state.position..][..12],
            "reactiveCell"
        );
        assert_eq!(
            &source[request.canonical.action.position..][..8],
            "activate"
        );
        assert_eq!(request.components.len(), 1);
        assert_eq!(request.states.len(), 2);
        assert_eq!(request.actions.len(), 2);
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
}
