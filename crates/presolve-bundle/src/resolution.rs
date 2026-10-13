use presolve_contract::{ApplicationContract, OutboundNetworkMode};
use serde_json::json;
use thiserror::Error;

use crate::BundleManifest;

/// Error returned when canonical release semantics cannot be reconstructed as
/// an Application Contract suitable for resolution.
#[derive(Debug, Error)]
pub enum ReleaseContractError {
    /// The canonical release projection could not be decoded as a contract.
    #[error("could not reconstruct Application Contract from release manifest: {0}")]
    Decode(#[from] serde_json::Error),

    /// The reconstructed contract violated Application Contract invariants.
    #[error("release manifest reconstructed an invalid Application Contract:\n{0}")]
    Invalid(String),
}

/// Reconstructs the environment-facing Application Contract represented by a
/// canonical `.presolved` manifest.
///
/// A release stores every application semantic required by resolution while
/// intentionally omitting source paths and deployment decisions. This function
/// turns that canonical semantic projection back into the resolver's existing
/// Application Contract input without consulting the original source tree.
///
/// Contracts that used the legacy implicit Component workload are normalized
/// by bundling to an explicit `main` Component workload. That normalization is
/// semantically equivalent for resolution.
///
/// # Errors
///
/// Returns an error if the manifest cannot be decoded into a valid Application
/// Contract.
pub fn application_contract_for_resolution(
    manifest: &BundleManifest,
) -> Result<ApplicationContract, ReleaseContractError> {
    let application = manifest.application();
    let requirements = manifest.requirements();
    let resources = requirements.resources();
    let network = requirements.network();

    let value = json!({
        "contract_version": requirements.contract_version().to_string(),
        "application": {
            "name": application.name(),
            "version": application.version().to_string(),
            "description": application.description(),
        },
        "workloads": manifest.workloads().iter().map(|workload| json!({
            "name": workload.name(),
            "kind": workload.kind().to_string(),
        })).collect::<Vec<_>>(),
        "interfaces": manifest.interfaces().iter().map(|interface| json!({
            "name": interface.name(),
            "kind": interface.kind().to_string(),
        })).collect::<Vec<_>>(),
        "relationships": manifest.relationships().iter().map(|relationship| json!({
            "from": relationship.from(),
            "to": relationship.to(),
            "interface": relationship.interface(),
        })).collect::<Vec<_>>(),
        "capabilities": requirements.capabilities().iter().map(|capability| json!({
            "interface": capability.interface(),
            "version": capability.version(),
            "optional": capability.optional(),
            "required_features": capability.required_features(),
            "preferred_features": capability.preferred_features(),
        })).collect::<Vec<_>>(),
        "resources": {
            "memory_mib": resources.memory_mib(),
            "cpu_millis": resources.cpu_millis(),
        },
        "network": {
            "outbound": outbound_mode(network.outbound()),
            "allow": network.allow(),
        },
    });

    let contract: ApplicationContract = serde_json::from_value(value)?;

    contract.validate().map_err(|diagnostics| {
        let rendered = diagnostics
            .diagnostics()
            .iter()
            .map(|diagnostic| format!("{}: {}", diagnostic.code(), diagnostic.message()))
            .collect::<Vec<_>>()
            .join("\n");

        ReleaseContractError::Invalid(rendered)
    })?;

    Ok(contract)
}

const fn outbound_mode(mode: OutboundNetworkMode) -> &'static str {
    match mode {
        OutboundNetworkMode::Deny => "deny",
        OutboundNetworkMode::AllowList => "allow_list",
        OutboundNetworkMode::AllowAll => "allow_all",
    }
}

#[cfg(test)]
mod tests {
    use presolve_contract::parse_contract;

    use crate::PresolveBundle;

    use super::*;

    #[test]
    fn explicit_contract_round_trips_through_release_resolution_projection() {
        let contract = parse_contract(
            r#"
contract_version = "0.1"

[application]
name = "portable-orders"
version = "1.2.3"
description = "Detached release resolution fixture"

[[workloads]]
name = "api"
kind = "component"

[[capabilities]]
interface = "presolve:kv/store"
version = "^0.1"
required_features = ["ttl"]
preferred_features = ["compare-and-swap"]

[resources]
memory_mib = 256
cpu_millis = 500

[network]
outbound = "allow_list"
allow = ["api.example.test"]
"#,
        )
        .expect("fixture contract should parse");

        let bundle = PresolveBundle::from_component(&contract, b"component bytes")
            .expect("fixture release should build");
        let reconstructed = application_contract_for_resolution(bundle.manifest())
            .expect("release contract should reconstruct");

        assert_eq!(reconstructed, contract);
    }

    #[test]
    fn implicit_component_workload_is_normalized_for_release_resolution() {
        let contract = parse_contract(
            r#"
contract_version = "0.1"

[application]
name = "implicit-main"
version = "1.0.0"

[[capabilities]]
interface = "presolve:kv/store"
version = "^0.1"
"#,
        )
        .expect("fixture contract should parse");

        assert_eq!(contract.workloads(), []);

        let bundle = PresolveBundle::from_component(&contract, b"component bytes")
            .expect("fixture release should build");
        let reconstructed = application_contract_for_resolution(bundle.manifest())
            .expect("release contract should reconstruct");

        assert_eq!(reconstructed.workloads().len(), 1);
        assert_eq!(reconstructed.workloads()[0].name(), "main");
        assert_eq!(reconstructed.workloads()[0].kind().to_string(), "component");
        assert_eq!(reconstructed.capabilities(), contract.capabilities());
    }
}
