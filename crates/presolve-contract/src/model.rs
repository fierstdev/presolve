use presolve_core::{InterfaceKind, ProductVersion, WorkloadKind};
use semver::VersionReq;
use serde::{Deserialize, Serialize};

use crate::ContractVersion;

/// Complete `Presolve` Application Contract.
///
/// An Application Contract describes application semantics, composition, and
/// environment requirements without describing how an execution environment
/// must realize them.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ApplicationContract {
    contract_version: ContractVersion,
    application: ApplicationMetadata,

    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    workloads: Vec<WorkloadDefinition>,

    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    interfaces: Vec<InterfaceDefinition>,

    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    relationships: Vec<RelationshipDefinition>,

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

    /// Returns workloads declared by the application.
    #[must_use]
    pub fn workloads(&self) -> &[WorkloadDefinition] {
        &self.workloads
    }

    /// Returns application-internal interfaces.
    #[must_use]
    pub fn interfaces(&self) -> &[InterfaceDefinition] {
        &self.interfaces
    }

    /// Returns application-internal workload relationships.
    #[must_use]
    pub fn relationships(&self) -> &[RelationshipDefinition] {
        &self.relationships
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

/// One logical executable constituent of an application.
///
/// This declaration describes workload semantics only. Build artifact paths,
/// deployment placement, environment selection, and provider bindings are not
/// part of the Application Contract.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct WorkloadDefinition {
    name: String,
    kind: WorkloadKind,
}

impl WorkloadDefinition {
    /// Returns the stable workload name within the application.
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Returns the workload execution class.
    #[must_use]
    pub const fn kind(&self) -> WorkloadKind {
        self.kind
    }
}

/// A named application-internal communication contract.
///
/// Interfaces describe logical interaction semantics. They do not prescribe a
/// deployment transport such as TCP, HTTP, IPC, or in-process component
/// linking.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct InterfaceDefinition {
    name: String,
    kind: InterfaceKind,
}

impl InterfaceDefinition {
    /// Returns the application-local interface name.
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Returns the interface interaction semantics.
    #[must_use]
    pub const fn kind(&self) -> InterfaceKind {
        self.kind
    }
}

/// A directed application-internal connection between two workloads.
///
/// The relationship declares semantic topology only. How the connection is
/// realized is a deployment concern.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RelationshipDefinition {
    from: String,
    to: String,
    interface: String,
}

impl RelationshipDefinition {
    /// Returns the workload initiating the interaction.
    #[must_use]
    pub fn from(&self) -> &str {
        &self.from
    }

    /// Returns the workload receiving the interaction.
    #[must_use]
    pub fn to(&self) -> &str {
        &self.to
    }

    /// Returns the application-local interface used by the relationship.
    #[must_use]
    pub fn interface(&self) -> &str {
        &self.interface
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
