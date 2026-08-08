//! Strict transport decoding for the V2 TypeScript authority bridge.

use std::collections::BTreeSet;

use serde::Deserialize;

use presolve_parser::ParsedFile;

use crate::{
    v2_authority_request::V2AuthorityRequestV1, AuthoredSourceRangeV1, ResolvedActionFieldV1,
    ResolvedComponentInheritanceV1, ResolvedEffectFieldV1, ResolvedFormDefinitionV1,
    ResolvedFormFieldDefinitionV1, ResolvedFormFieldValueClassificationV1,
    ResolvedFormValidationDefinitionKindV1, ResolvedFormValidationDefinitionV1,
    ResolvedIntrinsicIdentityV1, ResolvedSlotFieldV1, ResolvedStateInitializerV1,
    ResolvedTerminalPackageInvocationV1, V2AuthoringResolutionsV1,
};

pub const V2_AUTHORITY_RESPONSE_SCHEMA_VERSION: u32 = 12;

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
    pub form_fields: Vec<V2AuthorityFormFieldResolutionV1>,
    #[serde(default)]
    pub validations: Vec<V2AuthorityResolutionV1>,
    #[serde(default)]
    pub standard_validations: Vec<V2AuthorityStandardValidationResolutionV1>,
    #[serde(default)]
    pub package_invocations: Vec<V2AuthorityPackageInvocationResolutionV1>,
    #[serde(default)]
    pub server_action_invocations: Vec<V2AuthorityServerActionInvocationResolutionV1>,
    pub environment_public: Vec<V2AuthorityResolutionV1>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct V2AuthorityResolutionV1 {
    pub id: String,
    pub identity: V2AuthorityIdentityV1,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum V2AuthorityFormFieldValueClassificationV1 {
    FileArray,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct V2AuthorityFormFieldResolutionV1 {
    pub id: String,
    pub identity: V2AuthorityIdentityV1,
    #[serde(default)]
    pub value_classification: Option<V2AuthorityFormFieldValueClassificationV1>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct V2AuthorityStandardValidationResolutionV1 {
    pub id: String,
    pub identity: V2AuthorityIdentityV1,
    pub module_specifier: String,
    pub export_name: String,
    #[serde(default)]
    pub input_type: Option<String>,
    #[serde(default)]
    pub output_type: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum V2AuthorityPackageInvocationCompletionV1 {
    Synchronous,
    Promise,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct V2AuthorityPackageInvocationResolutionV1 {
    pub id: String,
    pub identity: V2AuthorityIdentityV1,
    pub module_specifier: String,
    pub export_name: String,
    pub argument_types: Vec<String>,
    pub completion: V2AuthorityPackageInvocationCompletionV1,
    pub inject_abort_signal: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct V2AuthorityServerActionInvocationResolutionV1 {
    pub id: String,
    pub identity: V2AuthorityIdentityV1,
    pub module_specifier: String,
    pub export_name: String,
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
    IncompatibleSite(String),
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
            Self::IncompatibleSite(id) => write!(
                formatter,
                "V2 TypeScript authority returned incompatible evidence for site `{id}`"
            ),
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
    validate_form_field_family(&request.form_fields, &response.form_fields)?;
    validate_family(&request.validations, &response.validations)?;
    validate_standard_validation_family(
        &request.standard_validations,
        &response.standard_validations,
    )?;
    validate_package_invocation_family(
        &request.package_invocations,
        &response.package_invocations,
    )?;
    validate_server_action_invocation_family(
        &request.server_action_invocations,
        &response.server_action_invocations,
    )?;
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
    let package_invocations = response
        .package_invocations
        .iter()
        .map(|resolution| {
            Ok((
                range_from_site_id(&resolution.id, "package-invocation", &parsed.syntax.source)?,
                ResolvedTerminalPackageInvocationV1 {
                    call_source: range_from_site_id(
                        &resolution.id,
                        "package-invocation",
                        &parsed.syntax.source,
                    )?,
                    module_specifier: resolution.module_specifier.clone(),
                    export_name: resolution.export_name.clone(),
                    declaration_modules: resolution.identity.declaration_modules.clone(),
                    argument_types: resolution.argument_types.clone(),
                    completion: match resolution.completion {
                        V2AuthorityPackageInvocationCompletionV1::Synchronous => {
                            crate::PackageInvocationCompletionV1::Synchronous
                        }
                        V2AuthorityPackageInvocationCompletionV1::Promise => {
                            crate::PackageInvocationCompletionV1::Promise
                        }
                    },
                    inject_abort_signal: resolution.inject_abort_signal,
                },
            ))
        })
        .collect::<Result<Vec<_>, V2AuthorityResponseErrorV1>>()?;
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
            .map(|(callee_source, identity)| {
                let direct_call = parsed
                    .classes
                    .iter()
                    .flat_map(|class| &class.properties)
                    .find_map(|property| {
                        let call = property.initializer_call.as_ref()?;
                        ((call.callee_span.start, call.callee_span.end)
                            == (callee_source.start, callee_source.end))
                            .then_some(call.inline_handler.as_ref()?.direct_call.as_ref()?)
                    });
                let terminal_package_invocation = direct_call.and_then(|call| {
                    package_invocations
                        .iter()
                        .find(|(source, _)| {
                            (source.start, source.end)
                                == (call.callee_span.start, call.callee_span.end)
                        })
                        .map(|(_, invocation)| invocation.clone())
                });
                ResolvedActionFieldV1 {
                    callee_source,
                    action_identity: identity,
                    terminal_package_invocation,
                }
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
        form_fields: response
            .form_fields
            .iter()
            .map(|resolution| {
                Ok(ResolvedFormFieldDefinitionV1 {
                    callee_source: range_from_site_id(
                        &resolution.id,
                        "form-field",
                        &parsed.syntax.source,
                    )?,
                    field_identity: identity_from_response(&resolution.identity),
                    value_classification: resolution.value_classification.map(|classification| {
                        match classification {
                            V2AuthorityFormFieldValueClassificationV1::FileArray => {
                                ResolvedFormFieldValueClassificationV1::FileArray
                            }
                        }
                    }),
                })
            })
            .collect::<Result<Vec<_>, V2AuthorityResponseErrorV1>>()?,
        validations: resolutions_for(&response.validations, "validation", parsed)?
            .into_iter()
            .map(|(callee_source, identity)| {
                Ok(ResolvedFormValidationDefinitionV1 {
                    callee_source,
                    kind: ResolvedFormValidationDefinitionKindV1::PresolveRule {
                        validation_identity: identity,
                    },
                })
            })
            .chain(response.standard_validations.iter().map(|resolution| {
                Ok(ResolvedFormValidationDefinitionV1 {
                    callee_source: range_from_site_id(
                        &resolution.id,
                        "standard-validation",
                        &parsed.syntax.source,
                    )?,
                    kind: ResolvedFormValidationDefinitionKindV1::StandardSchema {
                        module_specifier: resolution.module_specifier.clone(),
                        export_name: resolution.export_name.clone(),
                        declaration_modules: resolution.identity.declaration_modules.clone(),
                        input_type: resolution.input_type.clone(),
                        output_type: resolution.output_type.clone(),
                    },
                })
            }))
            .collect::<Result<Vec<_>, V2AuthorityResponseErrorV1>>()?,
        server_action_invocations: response
            .server_action_invocations
            .iter()
            .map(|resolution| {
                Ok(crate::ResolvedServerActionInvocationV1 {
                    call_source: range_from_site_id(
                        &resolution.id,
                        "server-action-invocation",
                        &parsed.syntax.source,
                    )?,
                    module_specifier: resolution.module_specifier.clone(),
                    export_name: resolution.export_name.clone(),
                })
            })
            .collect::<Result<Vec<_>, V2AuthorityResponseErrorV1>>()?,
    })
}

fn validate_standard_validation_family(
    request: &[crate::v2_authority_request::V2AuthorityStandardValidationSiteV1],
    response: &[V2AuthorityStandardValidationResolutionV1],
) -> Result<(), V2AuthorityResponseErrorV1> {
    let allowed = request
        .iter()
        .map(|site| (site.id.as_str(), site))
        .collect::<std::collections::BTreeMap<_, _>>();
    let mut seen = BTreeSet::new();
    for resolution in response {
        let Some(site) = allowed.get(resolution.id.as_str()) else {
            return Err(V2AuthorityResponseErrorV1::UnknownSite(
                resolution.id.clone(),
            ));
        };
        if !seen.insert(resolution.id.as_str()) {
            return Err(V2AuthorityResponseErrorV1::DuplicateSite(
                resolution.id.clone(),
            ));
        }
        if resolution.module_specifier != site.module_specifier
            || resolution.export_name != site.export_name
            || resolution.identity.name != site.export_name
            || resolution.identity.declaration_modules.is_empty()
        {
            return Err(V2AuthorityResponseErrorV1::IncompatibleSite(
                resolution.id.clone(),
            ));
        }
    }
    Ok(())
}

fn validate_package_invocation_family(
    request: &[crate::v2_authority_request::V2AuthorityPackageInvocationSiteV1],
    response: &[V2AuthorityPackageInvocationResolutionV1],
) -> Result<(), V2AuthorityResponseErrorV1> {
    let allowed = request
        .iter()
        .map(|site| (site.id.as_str(), site))
        .collect::<std::collections::BTreeMap<_, _>>();
    let mut seen = BTreeSet::new();
    for resolution in response {
        let Some(site) = allowed.get(resolution.id.as_str()) else {
            return Err(V2AuthorityResponseErrorV1::UnknownSite(
                resolution.id.clone(),
            ));
        };
        if !seen.insert(resolution.id.as_str()) {
            return Err(V2AuthorityResponseErrorV1::DuplicateSite(
                resolution.id.clone(),
            ));
        }
        let completion_matches = matches!(
            (site.completion, resolution.completion),
            (
                crate::v2_authority_request::V2AuthorityPackageInvocationCompletionV1::Synchronous,
                V2AuthorityPackageInvocationCompletionV1::Synchronous
            ) | (
                crate::v2_authority_request::V2AuthorityPackageInvocationCompletionV1::Promise,
                V2AuthorityPackageInvocationCompletionV1::Promise
            )
        );
        if resolution.module_specifier != site.module_specifier
            || resolution.export_name != site.export_name
            || resolution.identity.name != site.export_name
            || resolution.identity.declaration_modules.is_empty()
            || resolution.argument_types != site.argument_types
            || !completion_matches
            || resolution.inject_abort_signal != site.signal_position.is_some()
        {
            return Err(V2AuthorityResponseErrorV1::IncompatibleSite(
                resolution.id.clone(),
            ));
        }
    }
    Ok(())
}

fn validate_server_action_invocation_family(
    request: &[crate::v2_authority_request::V2AuthorityServerActionInvocationSiteV1],
    response: &[V2AuthorityServerActionInvocationResolutionV1],
) -> Result<(), V2AuthorityResponseErrorV1> {
    let allowed = request
        .iter()
        .map(|site| (site.id.as_str(), site))
        .collect::<std::collections::BTreeMap<_, _>>();
    let mut seen = BTreeSet::new();
    for resolution in response {
        let Some(site) = allowed.get(resolution.id.as_str()) else {
            return Err(V2AuthorityResponseErrorV1::UnknownSite(
                resolution.id.clone(),
            ));
        };
        if !seen.insert(resolution.id.as_str()) {
            return Err(V2AuthorityResponseErrorV1::DuplicateSite(
                resolution.id.clone(),
            ));
        }
        if resolution.module_specifier != site.module_specifier
            || resolution.export_name != site.export_name
            || resolution.identity.name != site.export_name
            || resolution.identity.declaration_modules.is_empty()
        {
            return Err(V2AuthorityResponseErrorV1::IncompatibleSite(
                resolution.id.clone(),
            ));
        }
    }
    Ok(())
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

fn validate_form_field_family(
    request: &[crate::v2_authority_request::V2AuthorityFormFieldSiteV1],
    response: &[V2AuthorityFormFieldResolutionV1],
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
    use std::collections::BTreeMap;
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
        V2AuthorityResponseV1, V2_AUTHORITY_RESPONSE_SCHEMA_VERSION,
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
            schema_version: V2_AUTHORITY_RESPONSE_SCHEMA_VERSION,
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
            standard_validations: Vec::new(),
            package_invocations: Vec::new(),
            server_action_invocations: Vec::new(),
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
            schema_version: V2_AUTHORITY_RESPONSE_SCHEMA_VERSION,
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
            standard_validations: Vec::new(),
            package_invocations: Vec::new(),
            server_action_invocations: Vec::new(),
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

    #[test]
    fn retains_exact_standard_schema_module_authority_for_runtime_linkage() {
        let source = r#"
import { Component, defineForm, field, required } from "presolve";
import { displayNameSchema } from "./schemas.js";
export class Profile extends Component {
  profile = defineForm({ fields: {
    displayName: field({ initial: "", validate: [required(), displayNameSchema] })
  } });
}
"#;
        let parsed = parse_file("app/routes/profile.tsx", source);
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
            schema_version: V2_AUTHORITY_RESPONSE_SCHEMA_VERSION,
            diagnostics: Vec::new(),
            components: vec![V2AuthorityResolutionV1 {
                id: request.components[0].id.clone(),
                identity: identity("Component"),
            }],
            states: Vec::new(),
            actions: Vec::new(),
            effects: Vec::new(),
            slots: Vec::new(),
            forms: vec![V2AuthorityResolutionV1 {
                id: request.forms[0].id.clone(),
                identity: identity("defineForm"),
            }],
            form_fields: vec![super::V2AuthorityFormFieldResolutionV1 {
                id: request.form_fields[0].id.clone(),
                identity: identity("field"),
                value_classification: None,
            }],
            validations: vec![V2AuthorityResolutionV1 {
                id: request.validations[0].id.clone(),
                identity: identity("required"),
            }],
            standard_validations: vec![super::V2AuthorityStandardValidationResolutionV1 {
                id: request.standard_validations[0].id.clone(),
                identity: V2AuthorityIdentityV1 {
                    name: "displayNameSchema".into(),
                    flags: 2,
                    declaration_modules: vec!["app/schemas.ts".into()],
                },
                module_specifier: "./schemas.js".into(),
                export_name: "displayNameSchema".into(),
                input_type: Some("string".into()),
                output_type: Some("string".into()),
            }],
            package_invocations: Vec::new(),
            server_action_invocations: Vec::new(),
            environment_public: Vec::new(),
        };
        let resolutions =
            v2_authoring_resolutions_from_response_v1(&parsed, &request, &response).unwrap();
        let authored = crate::lower_v2_authoring_v1(&parsed, resolutions)
            .unwrap()
            .model;
        let graph = crate::build_v2_component_graph_for_module(&parsed, &authored);
        let facts = &graph.components[0].validation_rule_declaration_facts;
        assert_eq!(facts.len(), 2);
        let standard = facts[1]
            .standard_schema
            .as_ref()
            .expect("retained Standard Schema authority");
        assert_eq!(standard.module_specifier, "./schemas.js");
        assert_eq!(standard.export_name, "displayNameSchema");
        assert_eq!(standard.declaration_modules, ["app/schemas.ts"]);

        let unit = crate::CompilationUnit::parse_sources([("app/routes/profile.tsx", source)]);
        let models = BTreeMap::from([(parsed.path.clone(), authored)]);
        let application =
            crate::build_file_route_application_semantic_model_for_unit_with_packages_and_v2_authoring(
                &unit,
                &crate::SemanticPackageResolutionTable::default(),
                &models,
            )
            .expect("V2 Standard Schema application assembly");
        let candidate = application
            .validation_rule_candidates
            .iter()
            .find(|candidate| candidate.standard_schema.is_some())
            .expect("Standard Schema validation candidate");
        assert!(candidate.violations.is_empty());
        assert_eq!(
            candidate.kind,
            Some(crate::ValidationRuleKind::StandardSchema)
        );
        assert!(application
            .validation_rules
            .values()
            .any(|rule| rule.kind == crate::ValidationRuleKind::StandardSchema));
        assert!(!application
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.code == "PSC1087"));
    }
}
