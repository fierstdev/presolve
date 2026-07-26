//! Strict transport decoding for the V2 TypeScript authority bridge.

use std::collections::BTreeSet;

use serde::Deserialize;

use crate::v2_authority_request::V2AuthorityRequestV1;

pub const V2_AUTHORITY_RESPONSE_SCHEMA_VERSION: u32 = 1;

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct V2AuthorityResponseV1 {
    pub schema_version: u32,
    pub diagnostics: Vec<serde_json::Value>,
    pub components: Vec<V2AuthorityResolutionV1>,
    pub states: Vec<V2AuthorityResolutionV1>,
    pub actions: Vec<V2AuthorityResolutionV1>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct V2AuthorityResolutionV1 {
    pub id: String,
    pub identity: V2AuthorityIdentityV1,
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
}

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
    validate_family(&request.actions, &response.actions)
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
