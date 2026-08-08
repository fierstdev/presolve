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
    Opaque,
    ServerAction,
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

/// Closed request input admitted for a server-backed route loader.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SemanticPackageRouteLoaderInput {
    RouteParameters,
}

/// Cache visibility declared by an integrity-bound server capability.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SemanticPackageServerCacheScope {
    NoStore,
    Private,
    Public,
}

/// Immutable cache policy a server adapter may honor but never broaden.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SemanticPackageServerCachePolicy {
    pub scope: SemanticPackageServerCacheScope,
    #[serde(default)]
    pub max_age_seconds: Option<u64>,
}

/// Closed error transport selected by an integrity-bound route loader.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SemanticPackageRouteLoaderFailure {
    Typed,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SemanticPackageServerActionInput {
    FormData,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SemanticPackageServerActionResponse {
    Json,
    Redirect,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SemanticPackageServerAction {
    pub input: SemanticPackageServerActionInput,
    pub response: SemanticPackageServerActionResponse,
    pub failure: SemanticPackageRouteLoaderFailure,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SemanticPackageFormSubmissionExecutionBoundary {
    Client,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SemanticPackageFormSubmissionCancellation {
    Abort,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SemanticPackageFormSubmissionInput {
    FormValue,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SemanticPackageFormSubmissionResult {
    Void,
}

/// Closed contract for an asynchronous client Form submission capability.
///
/// The compiler does not inspect package source. It supplies the canonical
/// nested Form value and one submission-owned AbortSignal to the named export.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SemanticPackageFormSubmission {
    pub execution_boundary: SemanticPackageFormSubmissionExecutionBoundary,
    pub cancellation: SemanticPackageFormSubmissionCancellation,
    pub input: SemanticPackageFormSubmissionInput,
    pub result: SemanticPackageFormSubmissionResult,
}

/// Execution side for the initial opaque terminal package boundary.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SemanticPackageOpaqueExecutionBoundary {
    Client,
}

/// Resume behavior for an opaque terminal package activation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SemanticPackageOpaqueResumePolicy {
    ColdFallback,
}

/// Closed contract for a no-input, no-output opaque terminal export.
///
/// This records how the application may use a package without giving the
/// compiler authority to inspect its implementation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SemanticPackageOpaqueTerminal {
    pub execution_boundary: SemanticPackageOpaqueExecutionBoundary,
    pub resume: SemanticPackageOpaqueResumePolicy,
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

/// Additional contract required before a Resource endpoint may serve as a
/// route loader. The package implementation remains opaque to the compiler.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SemanticPackageRouteLoader {
    pub input: SemanticPackageRouteLoaderInput,
    pub cache: SemanticPackageServerCachePolicy,
    pub failure: SemanticPackageRouteLoaderFailure,
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
    #[serde(default)]
    pub route_loader: Option<SemanticPackageRouteLoader>,
    #[serde(default)]
    pub server_action: Option<SemanticPackageServerAction>,
    #[serde(default)]
    pub form_submission: Option<SemanticPackageFormSubmission>,
    #[serde(default)]
    pub opaque_terminal: Option<SemanticPackageOpaqueTerminal>,
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
    InvalidRouteLoader,
    InvalidServerAction,
    InvalidFormSubmission,
    InvalidOpaqueTerminal,
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
    if contract.exports.values().any(|export| {
        let Some(loader) = &export.route_loader else {
            return false;
        };
        let Some(endpoint) = &export.resource_endpoint else {
            return true;
        };
        !matches!(export.kind, SemanticPackageKind::Resource)
            || !matches!(
                endpoint.execution_boundary,
                SemanticPackageResourceExecutionBoundary::Server
                    | SemanticPackageResourceExecutionBoundary::Shared
            )
            || match loader.cache.scope {
                SemanticPackageServerCacheScope::Public => {
                    loader.cache.max_age_seconds.is_none_or(|age| age == 0)
                }
                SemanticPackageServerCacheScope::NoStore
                | SemanticPackageServerCacheScope::Private => {
                    loader.cache.max_age_seconds.is_some()
                }
            }
    }) {
        return Err(SemanticPackageContractError::InvalidRouteLoader);
    }
    if contract.exports.values().any(|export| {
        (export.kind == SemanticPackageKind::ServerAction) != export.server_action.is_some()
            || (export.kind == SemanticPackageKind::ServerAction
                && (export.type_signature
                    != "(FormData, AbortSignal) -> Promise<ServerActionResult>"
                    || export.resume_policy != "cold_fallback"))
    }) {
        return Err(SemanticPackageContractError::InvalidServerAction);
    }
    if contract.exports.values().any(|export| {
        (export.kind == SemanticPackageKind::Capability) != export.form_submission.is_some()
            || (export.kind == SemanticPackageKind::Capability
                && (export.type_signature != "(FormValue, AbortSignal) -> Promise<void>"
                    || export.resume_policy != "cold_fallback"))
    }) {
        return Err(SemanticPackageContractError::InvalidFormSubmission);
    }
    if contract.exports.values().any(|export| {
        (export.kind == SemanticPackageKind::Opaque) != export.opaque_terminal.is_some()
            || (export.kind == SemanticPackageKind::Opaque
                && (export.type_signature != "() -> void"
                    || export.resume_policy != "cold_fallback"))
    }) {
        return Err(SemanticPackageContractError::InvalidOpaqueTerminal);
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

    #[test]
    fn opaque_exports_require_the_closed_terminal_contract() {
        let missing_terminal = parse_semantic_package_contract(
            r#"{"schema_version":1,"package":"@acme/analytics","version":"1.2.3","integrity":"sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa","exports":{"trackPurchase":{"kind":"opaque","type_signature":"() -> void","runtime_module":"dist/track.js","resume_policy":"cold_fallback"}}}"#,
        );
        assert_eq!(
            missing_terminal,
            Err(SemanticPackageContractError::InvalidOpaqueTerminal)
        );

        let contract = parse_semantic_package_contract(
            r#"{"schema_version":1,"package":"@acme/analytics","version":"1.2.3","integrity":"sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa","exports":{"trackPurchase":{"kind":"opaque","type_signature":"() -> void","runtime_module":"dist/track.js","resume_policy":"cold_fallback","opaque_terminal":{"execution_boundary":"client","resume":"cold_fallback"}}}}"#,
        )
        .expect("opaque terminal contract");
        assert_eq!(
            contract.exports["trackPurchase"].opaque_terminal,
            Some(SemanticPackageOpaqueTerminal {
                execution_boundary: SemanticPackageOpaqueExecutionBoundary::Client,
                resume: SemanticPackageOpaqueResumePolicy::ColdFallback,
            })
        );
    }

    #[test]
    fn validates_route_loader_capabilities_on_server_or_shared_resource_exports() {
        let contract = parse_semantic_package_contract(
            r#"{"schema_version":1,"package":"post-service","version":"1.2.3","integrity":"sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa","exports":{"loadPost":{"kind":"resource","type_signature":"RouteParameters -> Resource<Post, NotFound>","runtime_module":"dist/load-post.js","resume_policy":"reload","resource_endpoint":{"execution_boundary":"server","cancellation":"abort","resume":"reload"},"route_loader":{"input":"route_parameters","cache":{"scope":"public","max_age_seconds":60},"failure":"typed"}}}}"#,
        )
        .expect("closed route loader contract");
        assert_eq!(
            contract.exports["loadPost"].route_loader,
            Some(SemanticPackageRouteLoader {
                input: SemanticPackageRouteLoaderInput::RouteParameters,
                cache: SemanticPackageServerCachePolicy {
                    scope: SemanticPackageServerCacheScope::Public,
                    max_age_seconds: Some(60),
                },
                failure: SemanticPackageRouteLoaderFailure::Typed,
            })
        );

        let invalid_client = parse_semantic_package_contract(
            r#"{"schema_version":1,"package":"post-service","version":"1.2.3","integrity":"sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa","exports":{"loadPost":{"kind":"resource","type_signature":"RouteParameters -> Resource<Post, NotFound>","runtime_module":"dist/load-post.js","resume_policy":"reload","resource_endpoint":{"execution_boundary":"client","cancellation":"abort","resume":"reload"},"route_loader":{"input":"route_parameters","cache":{"scope":"no_store"},"failure":"typed"}}}}"#,
        );
        assert_eq!(
            invalid_client,
            Err(SemanticPackageContractError::InvalidRouteLoader)
        );
    }

    #[test]
    fn validates_closed_server_action_capabilities() {
        let contract = parse_semantic_package_contract(
            r#"{"schema_version":1,"package":"post-service","version":"1.2.3","integrity":"sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa","exports":{"savePost":{"kind":"server_action","type_signature":"(FormData, AbortSignal) -> Promise<ServerActionResult>","runtime_module":"dist/save-post.js","resume_policy":"cold_fallback","server_action":{"input":"form_data","response":"json","failure":"typed"}}}}"#,
        );
        assert!(contract.is_ok());
    }

    #[test]
    fn form_submission_capabilities_require_the_closed_client_abort_contract() {
        let contract = parse_semantic_package_contract(
            r#"{"schema_version":1,"package":"profile-service","version":"1.2.3","integrity":"sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa","exports":{"saveProfile":{"kind":"capability","type_signature":"(FormValue, AbortSignal) -> Promise<void>","runtime_module":"dist/save-profile.js","resume_policy":"cold_fallback","form_submission":{"execution_boundary":"client","cancellation":"abort","input":"form_value","result":"void"}}}}"#,
        )
        .expect("closed Form submission contract");
        assert_eq!(
            contract.exports["saveProfile"].form_submission,
            Some(SemanticPackageFormSubmission {
                execution_boundary: SemanticPackageFormSubmissionExecutionBoundary::Client,
                cancellation: SemanticPackageFormSubmissionCancellation::Abort,
                input: SemanticPackageFormSubmissionInput::FormValue,
                result: SemanticPackageFormSubmissionResult::Void,
            })
        );

        let missing = parse_semantic_package_contract(
            r#"{"schema_version":1,"package":"profile-service","version":"1.2.3","integrity":"sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa","exports":{"saveProfile":{"kind":"capability","type_signature":"(FormValue, AbortSignal) -> Promise<void>","runtime_module":"dist/save-profile.js","resume_policy":"cold_fallback"}}}"#,
        );
        assert_eq!(
            missing,
            Err(SemanticPackageContractError::InvalidFormSubmission)
        );
    }
}
