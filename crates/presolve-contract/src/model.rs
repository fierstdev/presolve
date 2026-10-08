use presolve_core::ProductVersion;
use semver::VersionReq;
use serde::{Deserialize, Serialize};

use crate::ContractVersion;

/// Complete `Presolve` Application Contract.
///
/// An Application Contract describes what an application requires from an
/// execution environment without describing how that environment must be
/// provisioned.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ApplicationContract {
    contract_version: ContractVersion,
    application: ApplicationMetadata,

    #[serde(default)]
    capabilities: Vec<CapabilityRequirement>,

    #[serde(default)]
    resources: ResourceRequirements,

    #[serde(default)]
    network: NetworkPolicy,
}

impl ApplicationContract {
    /// Returns the Application Contract schema version.
    #[must_use]
    pub const fn contract_version(&self) -> ContractVersion {
        self.contract_version
    }

    /// Returns the application's metadata.
    #[must_use]
    pub const fn application(&self) -> &ApplicationMetadata {
        &self.application
    }

    /// Returns capability requirements declared by the application.
    #[must_use]
    pub fn capabilities(&self) -> &[CapabilityRequirement] {
        &self.capabilities
    }

    /// Returns application resource requirements.
    #[must_use]
    pub const fn resources(&self) -> &ResourceRequirements {
        &self.resources
    }

    /// Returns application network policy.
    #[must_use]
    pub const fn network(&self) -> &NetworkPolicy {
        &self.network
    }

    /// Serializes this contract into normalized TOML.
    ///
    /// The resulting TOML is intended for human-readable interchange. It is
    /// not yet the canonical byte representation used for artifact identity.
    ///
    /// # Errors
    ///
    /// Returns an error if the contract cannot be serialized as TOML.
    pub fn to_toml(&self) -> Result<String, toml::ser::Error> {
        toml::to_string_pretty(self)
    }
}

/// Human-facing application metadata.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ApplicationMetadata {
    name: String,
    version: ProductVersion,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    description: Option<String>,
}

impl ApplicationMetadata {
    /// Returns the stable application name.
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Returns the application release version.
    #[must_use]
    pub const fn version(&self) -> &ProductVersion {
        &self.version
    }

    /// Returns the optional human-readable description.
    #[must_use]
    pub fn description(&self) -> Option<&str> {
        self.description.as_deref()
    }
}

/// A capability required by an application.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CapabilityRequirement {
    interface: String,
    version: VersionReq,

    #[serde(default, skip_serializing_if = "is_false")]
    optional: bool,
}

impl CapabilityRequirement {
    /// Returns the capability interface name.
    #[must_use]
    pub fn interface(&self) -> &str {
        &self.interface
    }

    /// Returns the compatible capability version range.
    #[must_use]
    pub const fn version(&self) -> &VersionReq {
        &self.version
    }

    /// Returns whether this capability may be absent.
    #[must_use]
    pub const fn optional(&self) -> bool {
        self.optional
    }
}

/// Minimum compute resources required by an application.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ResourceRequirements {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    memory_mib: Option<u64>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    cpu_millis: Option<u32>,
}

impl ResourceRequirements {
    /// Returns the minimum required memory in mebibytes.
    #[must_use]
    pub const fn memory_mib(&self) -> Option<u64> {
        self.memory_mib
    }

    /// Returns the minimum required CPU allocation in millicores.
    #[must_use]
    pub const fn cpu_millis(&self) -> Option<u32> {
        self.cpu_millis
    }
}

/// Outbound network access mode.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OutboundNetworkMode {
    /// No outbound network access is permitted.
    #[default]
    Deny,

    /// Outbound access is restricted to an explicit allow list.
    AllowList,

    /// Arbitrary outbound network access is permitted.
    AllowAll,
}

/// Network access required by an application.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct NetworkPolicy {
    #[serde(default)]
    outbound: OutboundNetworkMode,

    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    allow: Vec<String>,
}

impl NetworkPolicy {
    /// Returns the outbound network mode.
    #[must_use]
    pub const fn outbound(&self) -> OutboundNetworkMode {
        self.outbound
    }

    /// Returns explicitly allowed outbound targets.
    #[must_use]
    pub fn allow(&self) -> &[String] {
        &self.allow
    }
}

#[allow(clippy::trivially_copy_pass_by_ref)]
const fn is_false(value: &bool) -> bool {
    !*value
}
