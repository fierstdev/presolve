//! Strict transport decoding for the V2 TypeScript authority bridge.

use std::collections::BTreeSet;

use serde::Deserialize;

use presolve_parser::ParsedFile;

use crate::{
    v2_authority_request::V2AuthorityRequestV1, AuthoredSourceRangeV1, ResolvedActionFieldV1,
    ResolvedComponentInheritanceV1, ResolvedEffectFieldV1, ResolvedFormDefinitionV1,
    ResolvedFormFieldDefinitionV1, ResolvedFormValidationDefinitionV1, ResolvedIntrinsicIdentityV1,
    ResolvedSlotFieldV1, ResolvedStateInitializerV1, V2AuthoringResolutionsV1,
};

pub const V2_AUTHORITY_RESPONSE_SCHEMA_VERSION: u32 = 7;

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct V2AuthorityResponseV1 {
    pub schema_version: u32,
    pub diagnostics: Vec<serde_json::Value>,
    pub components: Vec<V2AuthorityResolutionV1>,
    pub states: Vec<V2AuthorityResolutionV1>,
    pub actions: Vec<V2AuthorityResolutionV1>,
    pub effects: Vec<V2AuthorityResolutionV1>,
    #[serde(default)]
    pub slots: Vec<V2AuthorityResolutionV1>,
    #[serde(default)]
    pub forms: Vec<V2AuthorityResolutionV1>,
    #[serde(default)]
    pub form_fields: Vec<V2AuthorityResolutionV1>,
    #[serde(default)]
    pub validations: Vec<V2AuthorityResolutionV1>,
    pub environment_public: Vec<V2AuthorityResolutionV1>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct V2AuthorityResolutionV1 {
    pub id: String,
    pub identity: V2AuthorityIdentityV1,
}

/// Exact TypeScript-authoritative evidence for one environment member call.
/// The source range is recovered only from the request that selected it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResolvedEnvironmentPublicReadV1 {
    pub call_source: AuthoredSourceRangeV1,
    pub environment_public_identity: ResolvedIntrinsicIdentityV1,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct V2AuthorityIdentityV1 {
    pub name: String,
    pub flags: u32,
    pub declaration_modules: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum V2AuthorityResponseErrorV1 {
    SchemaVersion(u32),
    UnknownSite(String),
    DuplicateSite(String),
    DiagnosticsPresent(usize),
    InvalidSiteId(String),
}

impl std::fmt::Display for V2AuthorityResponseErrorV1 {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::SchemaVersion(version) => write!(
                formatter,
                "unsupported V2 TypeScript authority response schema version {version}"
            ),
            Self::UnknownSite(id) => write!(formatter, "V2 TypeScript authority returned unknown site `{id}`"),
            Self::DuplicateSite(id) => write!(formatter, "V2 TypeScript authority returned duplicate site `{id}`"),
            Self::DiagnosticsPresent(count) => write!(
                formatter,
                "V2 TypeScript authority reported {count} diagnostic(s); refusing to lower uncertain source"
            ),
            Self::InvalidSiteId(id) => write!(formatter, "invalid V2 TypeScript authority site id `{id}`"),
        }
    }
}

impl std::error::Error for V2AuthorityResponseErrorV1 {}

pub fn validate_v2_authority_response_v1(
    request: &V2AuthorityRequestV1,
    response: &V2AuthorityResponseV1,
) -> Result<(), V2AuthorityResponseErrorV1> {
    if response.schema_version != V2_AUTHORITY_RESPONSE_SCHEMA_VERSION {
        return Err(V2AuthorityResponseErrorV1::SchemaVersion(
            response.schema_version,
        ));
    }
    validate_family(&request.components, &response.components)?;
    validate_family(&request.states, &response.states)?;
    validate_family(&request.actions, &response.actions)?;
    validate_family(&request.effects, &response.effects)?;
    validate_family(&request.slots, &response.slots)?;
    validate_family(&request.forms, &response.forms)?;
    validate_family(&request.form_fields, &response.form_fields)?;
    validate_family(&request.validations, &response.validations)?;
    validate_member_family(&request.environment_public, &response.environment_public)
}

/// Converts a validated authority response into the exact syntax joins consumed
/// by the V2 authoring lowerer.  The response may not introduce a source range:
/// each range is recovered from a request id and checked against the parser's
/// original source before it can become canonical semantic evidence.
pub fn v2_authoring_resolutions_from_response_v1(
    parsed: &ParsedFile,
    request: &V2AuthorityRequestV1,
    response: &V2AuthorityResponseV1,
) -> Result<V2AuthoringResolutionsV1, V2AuthorityResponseErrorV1> {
    validate_v2_authority_response_v1(request, response)?;
    if !response.diagnostics.is_empty() {
        return Err(V2AuthorityResponseErrorV1::DiagnosticsPresent(
            response.diagnostics.len(),
        ));
    }
    Ok(V2AuthoringResolutionsV1 {
        components: resolutions_for(&response.components, "component", parsed)?
            .into_iter()
            .map(
                |(heritage_source, identity)| ResolvedComponentInheritanceV1 {
                    heritage_source,
                    component_identity: identity,
                },
            )
            .collect(),
        states: resolutions_for(&response.states, "state", parsed)?
            .into_iter()
            .map(|(callee_source, identity)| ResolvedStateInitializerV1 {
                callee_source,
                state_identity: identity,
            })
            .collect(),
        actions: resolutions_for(&response.actions, "action", parsed)?
            .into_iter()
            .map(|(callee_source, identity)| ResolvedActionFieldV1 {
                callee_source,
                action_identity: identity,
            })
            .collect(),
        effects: resolutions_for(&response.effects, "effect", parsed)?
            .into_iter()
            .map(|(callee_source, identity)| ResolvedEffectFieldV1 {
                callee_source,
                effect_identity: identity,
            })
            .collect(),
        slots: resolutions_for(&response.slots, "slot", parsed)?
            .into_iter()
            .map(|(callee_source, identity)| ResolvedSlotFieldV1 {
                callee_source,
                slot_identity: identity,
            })
            .collect(),
        forms: resolutions_for(&response.forms, "form", parsed)?
            .into_iter()
            .map(|(callee_source, identity)| ResolvedFormDefinitionV1 {
                callee_source,
                form_identity: identity,
            })
            .collect(),
        form_fields: resolutions_for(&response.form_fields, "form-field", parsed)?
            .into_iter()
            .map(|(callee_source, identity)| ResolvedFormFieldDefinitionV1 {
                callee_source,
                field_identity: identity,
            })
            .collect(),
        validations: resolutions_for(&response.validations, "validation", parsed)?
            .into_iter()
            .map(
                |(callee_source, identity)| ResolvedFormValidationDefinitionV1 {
                    callee_source,
                    validation_identity: identity,
                },
            )
            .collect(),
    })
}

/// Recovers validated environment-public evidence without widening the V2
/// component authoring model. Environment reads are expression products, not
/// component declarations, so their manifest join is owned separately.
pub fn v2_environment_public_resolutions_from_response_v1(
    parsed: &ParsedFile,
    request: &V2AuthorityRequestV1,
    response: &V2AuthorityResponseV1,
) -> Result<Vec<ResolvedEnvironmentPublicReadV1>, V2AuthorityResponseErrorV1> {
    validate_v2_authority_response_v1(request, response)?;
    if !response.diagnostics.is_empty() {
        return Err(V2AuthorityResponseErrorV1::DiagnosticsPresent(
            response.diagnostics.len(),
        ));
    }
    resolutions_for(&response.environment_public, "environment-public", parsed).map(|resolutions| {
        resolutions
            .into_iter()
            .map(
                |(call_source, environment_public_identity)| ResolvedEnvironmentPublicReadV1 {
                    call_source,
                    environment_public_identity,
                },
            )
            .collect()
    })
}

fn resolutions_for(
    resolutions: &[V2AuthorityResolutionV1],
    kind: &str,
    parsed: &ParsedFile,
) -> Result<Vec<(AuthoredSourceRangeV1, ResolvedIntrinsicIdentityV1)>, V2AuthorityResponseErrorV1> {
    resolutions
        .iter()
        .map(|resolution| {
            let range = range_from_site_id(&resolution.id, kind, &parsed.syntax.source)?;
            Ok((range, identity_from_response(&resolution.identity)))
        })
        .collect()
}

fn range_from_site_id(
    id: &str,
    expected_kind: &str,
    source: &str,
) -> Result<AuthoredSourceRangeV1, V2AuthorityResponseErrorV1> {
    let mut fields = id.split(':');
    let (Some(kind), Some(start), Some(end), None) =
        (fields.next(), fields.next(), fields.next(), fields.next())
    else {
        return Err(V2AuthorityResponseErrorV1::InvalidSiteId(id.to_owned()));
    };
    if kind != expected_kind {
        return Err(V2AuthorityResponseErrorV1::InvalidSiteId(id.to_owned()));
    }
    let (Ok(start), Ok(end)) = (start.parse::<usize>(), end.parse::<usize>()) else {
        return Err(V2AuthorityResponseErrorV1::InvalidSiteId(id.to_owned()));
    };
    if start >= end || source.get(start..end).is_none() {
        return Err(V2AuthorityResponseErrorV1::InvalidSiteId(id.to_owned()));
    }
    let prefix = &source[..start];
    let line = prefix.bytes().filter(|byte| *byte == b'\n').count() + 1;
    let column = prefix
        .rsplit_once('\n')
        .map_or(prefix.len() + 1, |(_, tail)| tail.len() + 1);
    Ok(AuthoredSourceRangeV1 {
        start,
        end,
        line,
        column,
    })
}

fn identity_from_response(identity: &V2AuthorityIdentityV1) -> ResolvedIntrinsicIdentityV1 {
    let mut declaration_modules = identity.declaration_modules.clone();
    declaration_modules.sort();
    declaration_modules.dedup();
    ResolvedIntrinsicIdentityV1 {
        name: identity.name.clone(),
        flags: identity.flags,
        declaration_modules,
    }
}

fn validate_family(
    request: &[crate::v2_authority_request::V2AuthoritySiteV1],
    response: &[V2AuthorityResolutionV1],
) -> Result<(), V2AuthorityResponseErrorV1> {
    let allowed = request
        .iter()
        .map(|site| site.id.as_str())
        .collect::<BTreeSet<_>>();
    let mut seen = BTreeSet::new();
    for resolution in response {
        if !allowed.contains(resolution.id.as_str()) {
            return Err(V2AuthorityResponseErrorV1::UnknownSite(
                resolution.id.clone(),
            ));
        }
        if !seen.insert(resolution.id.as_str()) {
            return Err(V2AuthorityResponseErrorV1::DuplicateSite(
                resolution.id.clone(),
            ));
        }
    }
    Ok(())
}

fn validate_member_family(
    request: &[crate::v2_authority_request::V2AuthorityMemberSiteV1],
    response: &[V2AuthorityResolutionV1],
) -> Result<(), V2AuthorityResponseErrorV1> {
    let allowed = request
        .iter()
        .map(|site| site.id.as_str())
        .collect::<BTreeSet<_>>();
    let mut seen = BTreeSet::new();
    for resolution in response {
        if !allowed.contains(resolution.id.as_str()) {
            return Err(V2AuthorityResponseErrorV1::UnknownSite(
                resolution.id.clone(),
            ));
        }
        if !seen.insert(resolution.id.as_str()) {
            return Err(V2AuthorityResponseErrorV1::DuplicateSite(
                resolution.id.clone(),
            ));
        }
    }
    Ok(())
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
        v2_authoring_resolutions_from_response_v1,
        v2_environment_public_resolutions_from_response_v1, validate_v2_authority_response_v1,
        V2AuthorityIdentityV1, V2AuthorityResolutionV1, V2AuthorityResponseErrorV1,
        V2AuthorityResponseV1,
    };

    fn identity(name: &str) -> V2AuthorityIdentityV1 {
        V2AuthorityIdentityV1 {
            name: name.to_owned(),
            flags: 32,
            declaration_modules: vec!["presolve".to_owned()],
        }
    }

    fn component_identity() -> ResolvedIntrinsicIdentityV1 {
        ResolvedIntrinsicIdentityV1 {
            name: "Component".to_owned(),
            flags: 32,
            declaration_modules: vec!["presolve".to_owned()],
        }
    }

    fn request_and_response() -> (
        presolve_parser::ParsedFile,
        crate::V2AuthorityRequestV1,
        V2AuthorityResponseV1,
    ) {
        let source = r#"
import { Component, state, action } from "presolve";
class Counter extends Component { count = state(0); increment = action(() => {}); }
"#;
        let parsed = parse_file("src/Counter.tsx", source);
        let heritage = component_inheritance_sites_v1(&parsed).pop().unwrap();
        let components = lower_component_inheritance_v1(
            &parsed,
            [ResolvedComponentInheritanceV1 {
                heritage_source: heritage.heritage_source,
                component_identity: component_identity(),
            }],
        )
        .unwrap()
        .model;
        let request = crate::build_v2_authority_request_v1(
            &parsed,
            PathBuf::from("tsconfig.json"),
            &components,
        )
        .unwrap();
        let response = V2AuthorityResponseV1 {
            schema_version: 7,
            diagnostics: Vec::new(),
            components: vec![V2AuthorityResolutionV1 {
                id: request.components[0].id.clone(),
                identity: identity("Component"),
            }],
            states: vec![V2AuthorityResolutionV1 {
                id: request.states[0].id.clone(),
                identity: identity("state"),
            }],
            actions: vec![V2AuthorityResolutionV1 {
                id: request.actions[1].id.clone(),
                identity: identity("action"),
            }],
            effects: Vec::new(),
            slots: Vec::new(),
            forms: Vec::new(),
            form_fields: Vec::new(),
            validations: Vec::new(),
            environment_public: Vec::new(),
        };
        (parsed, request, response)
    }

    #[test]
    fn validates_known_unique_response_sites() {
        let (_, request, response) = request_and_response();
        assert_eq!(
            validate_v2_authority_response_v1(&request, &response),
            Ok(())
        );
    }

    #[test]
    fn rejects_unknown_duplicate_and_incompatible_response_sites() {
        let (_, request, mut response) = request_and_response();
        response.components[0].id = "component:0:1".to_owned();
        assert!(matches!(
            validate_v2_authority_response_v1(&request, &response),
            Err(V2AuthorityResponseErrorV1::UnknownSite(_))
        ));

        let (_, request, mut response) = request_and_response();
        response.states.push(response.states[0].clone());
        assert!(matches!(
            validate_v2_authority_response_v1(&request, &response),
            Err(V2AuthorityResponseErrorV1::DuplicateSite(_))
        ));

        let (_, request, mut response) = request_and_response();
        response.schema_version = 1;
        assert_eq!(
            validate_v2_authority_response_v1(&request, &response),
            Err(V2AuthorityResponseErrorV1::SchemaVersion(1))
        );
    }

    #[test]
    fn maps_validated_evidence_to_exact_canonical_resolutions() {
        let (parsed, request, response) = request_and_response();
        let resolutions =
            v2_authoring_resolutions_from_response_v1(&parsed, &request, &response).unwrap();
        assert_eq!(resolutions.components.len(), 1);
        assert_eq!(resolutions.states.len(), 1);
        assert_eq!(resolutions.actions.len(), 1);
        assert_eq!(
            &parsed.syntax.source[resolutions.states[0].callee_source.start
                ..resolutions.states[0].callee_source.end],
            "state"
        );
        assert_eq!(
            &parsed.syntax.source[resolutions.actions[0].callee_source.start
                ..resolutions.actions[0].callee_source.end],
            "action"
        );
    }

    #[test]
    fn refuses_diagnostic_bearing_or_malformed_authority_results() {
        let (parsed, request, mut response) = request_and_response();
        response
            .diagnostics
            .push(serde_json::json!({ "code": 2322 }));
        assert_eq!(
            v2_authoring_resolutions_from_response_v1(&parsed, &request, &response),
            Err(V2AuthorityResponseErrorV1::DiagnosticsPresent(1))
        );

        let (parsed, request, mut response) = request_and_response();
        response.components[0].id = request.components[0].id.replace("component:", "state:");
        assert!(matches!(
            v2_authoring_resolutions_from_response_v1(&parsed, &request, &response),
            Err(V2AuthorityResponseErrorV1::UnknownSite(_))
        ));
    }

    #[test]
    fn recovers_environment_evidence_only_from_known_member_request_ids() {
        let source = r#"
import { Component, environment } from "presolve";
class Counter extends Component {}
const applicationName = environment.public("PRESOLVE_PUBLIC_APP_NAME");
"#;
        let parsed = parse_file("src/Counter.tsx", source);
        let heritage = component_inheritance_sites_v1(&parsed).pop().unwrap();
        let model = lower_component_inheritance_v1(
            &parsed,
            [ResolvedComponentInheritanceV1 {
                heritage_source: heritage.heritage_source,
                component_identity: component_identity(),
            }],
        )
        .unwrap()
        .model;
        let request =
            crate::build_v2_authority_request_v1(&parsed, PathBuf::from("tsconfig.json"), &model)
                .unwrap();
        let response = V2AuthorityResponseV1 {
            schema_version: 7,
            diagnostics: Vec::new(),
            components: vec![V2AuthorityResolutionV1 {
                id: request.components[0].id.clone(),
                identity: identity("Component"),
            }],
            states: Vec::new(),
            actions: Vec::new(),
            effects: Vec::new(),
            slots: Vec::new(),
            forms: Vec::new(),
            form_fields: Vec::new(),
            validations: Vec::new(),
            environment_public: vec![V2AuthorityResolutionV1 {
                id: request.environment_public[0].id.clone(),
                identity: identity("public"),
            }],
        };

        let evidence =
            v2_environment_public_resolutions_from_response_v1(&parsed, &request, &response)
                .unwrap();
        assert_eq!(evidence.len(), 1);
        assert_eq!(evidence[0].environment_public_identity.name, "public");
        assert_eq!(
            &source[evidence[0].call_source.start..evidence[0].call_source.end],
            "environment.public(\"PRESOLVE_PUBLIC_APP_NAME\")"
        );
    }
}
