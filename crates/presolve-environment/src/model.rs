use presolve_core::{EnvironmentId, ProviderId, WorkloadKind};
use semver::Version;
use serde::{Deserialize, Serialize};

use crate::{
    ENVIRONMENT_SPECIFICATION_VERSION, EnvironmentSpecificationError,
    EnvironmentSpecificationVersion,
};

/// Declarative description of one Presolve execution environment.
///
/// The specification is resolver-facing supply. It describes what can be
/// offered to applications without describing any particular application
/// deployment.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EnvironmentSpecification {
    specification_version: EnvironmentSpecificationVersion,
    environment: EnvironmentMetadata,
    resources: ResourceCapacity,
    execution: ExecutionSupport,

    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    providers: Vec<ProviderSpecification>,
}

impl EnvironmentSpecification {
    /// Creates an Environment Specification and validates its semantic invariants.
    ///
    /// # Errors
    ///
    /// Returns an error when the specification is structurally invalid or uses
    /// an unsupported schema version.
    pub fn new(
        specification_version: EnvironmentSpecificationVersion,
        environment: EnvironmentMetadata,
        resources: ResourceCapacity,
        execution: ExecutionSupport,
        providers: Vec<ProviderSpecification>,
    ) -> Result<Self, EnvironmentSpecificationError> {
        let mut specification = Self {
            specification_version,
            environment,
            resources,
            execution,
            providers,
        };

        specification.validate()?;
        specification.normalize();
        Ok(specification)
    }

    /// Creates a specification using the schema version implemented by this release.
    ///
    /// # Errors
    ///
    /// Returns an error when the specification is structurally invalid.
    pub fn current(
        environment: EnvironmentMetadata,
        resources: ResourceCapacity,
        execution: ExecutionSupport,
        providers: Vec<ProviderSpecification>,
    ) -> Result<Self, EnvironmentSpecificationError> {
        Self::new(
            ENVIRONMENT_SPECIFICATION_VERSION,
            environment,
            resources,
            execution,
            providers,
        )
    }

    /// Validates this specification.
    ///
    /// # Errors
    ///
    /// Returns the first violated Environment Specification invariant.
    pub fn validate(&self) -> Result<(), EnvironmentSpecificationError> {
        crate::validate::validate(self)
    }

    /// Returns a validated copy with all non-semantic collection ordering normalized.
    ///
    /// # Errors
    ///
    /// Returns the first violated Environment Specification invariant.
    pub fn normalized(&self) -> Result<Self, EnvironmentSpecificationError> {
        let mut specification = self.clone();
        specification.validate()?;
        specification.normalize();
        Ok(specification)
    }

    pub(crate) fn normalize(&mut self) {
        self.execution.workloads.sort();

        for provider in &mut self.providers {
            for capability in &mut provider.capabilities {
                capability.features.sort();
            }

            provider.capabilities.sort_by(|left, right| {
                left.interface
                    .cmp(&right.interface)
                    .then_with(|| left.version.cmp(&right.version))
                    .then_with(|| left.features.cmp(&right.features))
            });
        }

        self.providers.sort_by(|left, right| {
            left.name
                .cmp(&right.name)
                .then_with(|| left.id.cmp(&right.id))
        });
    }

    /// Returns the Environment Specification schema version.
    #[must_use]
    pub const fn specification_version(&self) -> EnvironmentSpecificationVersion {
        self.specification_version
    }

    /// Returns stable environment metadata.
    #[must_use]
    pub const fn environment(&self) -> &EnvironmentMetadata {
        &self.environment
    }

    /// Returns allocatable environment capacity.
    #[must_use]
    pub const fn resources(&self) -> ResourceCapacity {
        self.resources
    }

    /// Returns workload execution classes supported by the environment.
    #[must_use]
    pub const fn execution(&self) -> &ExecutionSupport {
        &self.execution
    }

    /// Returns providers available in the environment.
    #[must_use]
    pub fn providers(&self) -> &[ProviderSpecification] {
        &self.providers
    }
}

/// Stable human- and machine-facing identity of an execution environment.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EnvironmentMetadata {
    id: EnvironmentId,
    name: String,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    description: Option<String>,
}

impl EnvironmentMetadata {
    /// Creates environment metadata.
    #[must_use]
    pub fn new(id: EnvironmentId, name: impl Into<String>) -> Self {
        Self {
            id,
            name: name.into(),
            description: None,
        }
    }

    /// Attaches a human-readable description.
    #[must_use]
    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }

    /// Returns the stable environment identifier.
    #[must_use]
    pub const fn id(&self) -> &EnvironmentId {
        &self.id
    }

    /// Returns the environment-local human-readable name.
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Returns the optional environment description.
    #[must_use]
    pub fn description(&self) -> Option<&str> {
        self.description.as_deref()
    }
}

/// Allocatable compute capacity exposed to the resolver.
///
/// Zero capacity is representable. Compatibility is a resolver decision rather
/// than a schema-validity rule.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ResourceCapacity {
    memory_mib: u64,
    cpu_millis: u32,
}

impl ResourceCapacity {
    /// Creates an environment capacity declaration.
    #[must_use]
    pub const fn new(memory_mib: u64, cpu_millis: u32) -> Self {
        Self {
            memory_mib,
            cpu_millis,
        }
    }

    /// Returns allocatable memory in mebibytes.
    #[must_use]
    pub const fn memory_mib(self) -> u64 {
        self.memory_mib
    }

    /// Returns allocatable CPU capacity in millicores.
    #[must_use]
    pub const fn cpu_millis(self) -> u32 {
        self.cpu_millis
    }
}

/// Workload execution classes supported by an environment.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ExecutionSupport {
    workloads: Vec<WorkloadKind>,
}

impl ExecutionSupport {
    /// Creates an execution-support declaration.
    #[must_use]
    pub const fn new(workloads: Vec<WorkloadKind>) -> Self {
        Self { workloads }
    }

    /// Creates support for the currently implemented Component workload class.
    #[must_use]
    pub fn components() -> Self {
        Self::new(vec![WorkloadKind::Component])
    }

    /// Returns supported workload classes.
    #[must_use]
    pub fn workloads(&self) -> &[WorkloadKind] {
        &self.workloads
    }
}

/// One provider visible inside an environment.
///
/// Provider identity connects resolver output to node-side provider
/// materialization. Provider implementation configuration is intentionally not
/// part of this resolver-facing type.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProviderSpecification {
    id: ProviderId,
    name: String,
    capabilities: Vec<ProvidedCapabilitySpecification>,
}

impl ProviderSpecification {
    /// Creates a provider declaration.
    #[must_use]
    pub fn new(
        id: ProviderId,
        name: impl Into<String>,
        capabilities: Vec<ProvidedCapabilitySpecification>,
    ) -> Self {
        Self {
            id,
            name: name.into(),
            capabilities,
        }
    }

    /// Returns the stable provider identifier.
    #[must_use]
    pub const fn id(&self) -> &ProviderId {
        &self.id
    }

    /// Returns the provider name within this environment.
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Returns capabilities supplied by this provider.
    #[must_use]
    pub fn capabilities(&self) -> &[ProvidedCapabilitySpecification] {
        &self.capabilities
    }
}

/// One exact capability version offered by an environment provider.
///
/// `features` contains optional semantic capabilities explicitly implemented by
/// the provider. Canonical feature vocabulary is validated when semantic
/// capability contracts are attached.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProvidedCapabilitySpecification {
    interface: String,
    version: Version,

    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    features: Vec<String>,
}

impl ProvidedCapabilitySpecification {
    /// Creates a provider capability declaration with no optional features.
    #[must_use]
    pub fn new(interface: impl Into<String>, version: Version) -> Self {
        Self {
            interface: interface.into(),
            version,
            features: Vec::new(),
        }
    }

    /// Declares optional semantic features implemented by the provider.
    #[must_use]
    pub fn with_features<I, S>(mut self, features: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        self.features = features.into_iter().map(Into::into).collect();
        self
    }

    /// Returns the capability interface identifier.
    #[must_use]
    pub fn interface(&self) -> &str {
        &self.interface
    }

    /// Returns the exact supplied capability version.
    #[must_use]
    pub const fn version(&self) -> &Version {
        &self.version
    }

    /// Returns optional semantic features explicitly supplied by the provider.
    #[must_use]
    pub fn features(&self) -> &[String] {
        &self.features
    }
}
