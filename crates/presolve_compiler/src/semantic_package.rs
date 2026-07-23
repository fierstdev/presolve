use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

pub const SEMANTIC_PACKAGE_CONTRACT_SCHEMA_VERSION: u32 = 1;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SemanticPackageKind {
    Pure,
    Capability,
    Resource,
    Codec,
    Component,
}

/// A closed compiler-lowered operation that a `pure` package export may declare.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SemanticPackagePureOperation {
    Identity,
}

/// Execution side declared by a third-party Resource endpoint.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SemanticPackageResourceExecutionBoundary {
    Client,
    Server,
    Shared,
}

/// How the generated Resource activation may cancel an endpoint invocation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SemanticPackageResourceCancellation {
    Abort,
}

/// How an endpoint's completed result participates in a resumed application.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SemanticPackageResourceResumePolicy {
    Reload,
    Snapshot,
}

/// Closed semantic contract for an executable Resource package export.
///
/// The compiler still does not execute or inspect package implementation. This
/// metadata is the prerequisite for later Resource source lowering.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SemanticPackageResourceEndpoint {
    pub execution_boundary: SemanticPackageResourceExecutionBoundary,
    pub cancellation: SemanticPackageResourceCancellation,
    pub resume: SemanticPackageResourceResumePolicy,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SemanticPackageExport {
    pub kind: SemanticPackageKind,
    pub type_signature: String,
    pub runtime_module: String,
    pub resume_policy: String,
    #[serde(default)]
    pub pure_operation: Option<SemanticPackagePureOperation>,
    #[serde(default)]
    pub resource_endpoint: Option<SemanticPackageResourceEndpoint>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SemanticPackageContract {
    pub schema_version: u32,
    pub package: String,
    pub version: String,
    pub integrity: String,
    pub exports: BTreeMap<String, SemanticPackageExport>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SemanticPackageContractError {
    InvalidJson,
    UnsupportedSchema,
    InvalidIntegrity,
    EmptyPackage,
    EmptyVersion,
    EmptyExport,
    InvalidExportContract,
    InvalidPureOperation,
    InvalidResourceEndpoint,
    DuplicateSpecifier,
}

pub fn parse_semantic_package_contract(
    json: &str,
) -> Result<SemanticPackageContract, SemanticPackageContractError> {
    let contract = serde_json::from_str::<SemanticPackageContract>(json)
        .map_err(|_| SemanticPackageContractError::InvalidJson)?;
    if contract.schema_version != SEMANTIC_PACKAGE_CONTRACT_SCHEMA_VERSION {
        return Err(SemanticPackageContractError::UnsupportedSchema);
    }
    if contract.package.is_empty() {
        return Err(SemanticPackageContractError::EmptyPackage);
    }
    if contract.version.is_empty() {
        return Err(SemanticPackageContractError::EmptyVersion);
    }
    if !contract.integrity.starts_with("sha256:")
        || contract.integrity.len() != 71
        || !contract.integrity[7..]
            .chars()
            .all(|c| c.is_ascii_hexdigit())
    {
        return Err(SemanticPackageContractError::InvalidIntegrity);
    }
    if contract.exports.is_empty() || contract.exports.keys().any(|name| name.is_empty()) {
        return Err(SemanticPackageContractError::EmptyExport);
    }
    if contract.exports.values().any(|export| {
        export.type_signature.is_empty()
            || export.runtime_module.is_empty()
            || export.resume_policy.is_empty()
    }) {
        return Err(SemanticPackageContractError::InvalidExportContract);
    }
    if contract
        .exports
        .values()
        .any(|export| export.pure_operation.is_some() && export.kind != SemanticPackageKind::Pure)
    {
        return Err(SemanticPackageContractError::InvalidPureOperation);
    }
    if contract.exports.values().any(|export| {
        (export.kind == SemanticPackageKind::Resource) != export.resource_endpoint.is_some()
    }) {
        return Err(SemanticPackageContractError::InvalidResourceEndpoint);
    }
    Ok(contract)
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct SemanticPackageResolutionTable {
    contracts: BTreeMap<String, SemanticPackageContract>,
}

impl SemanticPackageResolutionTable {
    pub fn insert(
        &mut self,
        specifier: String,
        contract: SemanticPackageContract,
    ) -> Result<(), SemanticPackageContractError> {
        if self.contracts.contains_key(&specifier) {
            return Err(SemanticPackageContractError::DuplicateSpecifier);
        }
        self.contracts.insert(specifier, contract);
        Ok(())
    }

    #[must_use]
    pub fn contract(&self, specifier: &str) -> Option<&SemanticPackageContract> {
        self.contracts.get(specifier)
    }
    #[must_use]
    pub fn resolve(&self, specifier: &str, export: &str) -> Option<&SemanticPackageExport> {
        self.contracts.get(specifier)?.exports.get(export)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn parses_integrity_checked_contract_and_resolves_an_export() {
        let contract = parse_semantic_package_contract(r#"{"schema_version":1,"package":"date-kit","version":"1.2.3","integrity":"sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa","exports":{"format":{"kind":"pure","type_signature":"(Date) -> string","runtime_module":"dist/format.js","resume_policy":"input_only"}}}"#).unwrap();
        let mut table = SemanticPackageResolutionTable::default();
        table.insert("date-kit".into(), contract).unwrap();
        assert_eq!(
            table.resolve("date-kit", "format").unwrap().kind,
            SemanticPackageKind::Pure
        );
    }

    #[test]
    fn rejects_incomplete_contracts_without_replacing_existing_resolution() {
        let valid = parse_semantic_package_contract(r#"{"schema_version":1,"package":"date-kit","version":"1.2.3","integrity":"sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa","exports":{"format":{"kind":"pure","type_signature":"(Date) -> string","runtime_module":"dist/format.js","resume_policy":"input_only"}}}"#).unwrap();
        let invalid = parse_semantic_package_contract(
            r#"{"schema_version":1,"package":"date-kit","version":"","integrity":"sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa","exports":{"format":{"kind":"pure","type_signature":"","runtime_module":"","resume_policy":""}}}"#,
        );
        assert_eq!(invalid, Err(SemanticPackageContractError::EmptyVersion));

        let mut table = SemanticPackageResolutionTable::default();
        table.insert("date-kit".into(), valid.clone()).unwrap();
        assert_eq!(
            table.insert("date-kit".into(), valid),
            Err(SemanticPackageContractError::DuplicateSpecifier)
        );
        assert_eq!(table.contract("date-kit").unwrap().version, "1.2.3");
    }

    #[test]
    fn resource_exports_require_a_closed_endpoint_contract() {
        let missing_endpoint = parse_semantic_package_contract(
            r#"{"schema_version":1,"package":"profile-service","version":"1.2.3","integrity":"sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa","exports":{"loadProfile":{"kind":"resource","type_signature":"(ProfileKey) -> Resource<Profile, ProfileError>","runtime_module":"dist/load-profile.js","resume_policy":"snapshot"}}}"#,
        );
        assert_eq!(
            missing_endpoint,
            Err(SemanticPackageContractError::InvalidResourceEndpoint)
        );

        let contract = parse_semantic_package_contract(
            r#"{"schema_version":1,"package":"profile-service","version":"1.2.3","integrity":"sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa","exports":{"loadProfile":{"kind":"resource","type_signature":"(ProfileKey) -> Resource<Profile, ProfileError>","runtime_module":"dist/load-profile.js","resume_policy":"snapshot","resource_endpoint":{"execution_boundary":"shared","cancellation":"abort","resume":"snapshot"}}}}"#,
        )
        .expect("resource endpoint contract");
        assert_eq!(
            contract.exports["loadProfile"].resource_endpoint,
            Some(SemanticPackageResourceEndpoint {
                execution_boundary: SemanticPackageResourceExecutionBoundary::Shared,
                cancellation: SemanticPackageResourceCancellation::Abort,
                resume: SemanticPackageResourceResumePolicy::Snapshot,
            })
        );
    }
}
