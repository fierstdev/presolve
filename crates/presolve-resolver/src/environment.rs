use presolve_core::{EnvironmentId, ProviderId};
use semver::Version;

/// Resources available to an application in an `Presolve` environment.
///
/// This represents allocatable capacity visible to the resolver. It does not
/// yet model multi-application scheduling or resource reservations.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EnvironmentResources {
    memory_mib: u64,
    cpu_millis: u32,
}

impl EnvironmentResources {
    /// Creates an environment resource inventory.
    #[must_use]
    pub const fn new(memory_mib: u64, cpu_millis: u32) -> Self {
        Self {
            memory_mib,
            cpu_millis,
        }
    }

    /// Returns available memory in mebibytes.
    #[must_use]
    pub const fn memory_mib(self) -> u64 {
        self.memory_mib
    }

    /// Returns available CPU capacity in millicores.
    #[must_use]
    pub const fn cpu_millis(self) -> u32 {
        self.cpu_millis
    }
}

/// One capability exposed by an environment provider.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProvidedCapability {
    interface: String,
    version: Version,
}

impl ProvidedCapability {
    /// Creates a provider capability declaration.
    #[must_use]
    pub fn new(interface: impl Into<String>, version: Version) -> Self {
        Self {
            interface: interface.into(),
            version,
        }
    }

    /// Returns the WIT capability interface identifier.
    #[must_use]
    pub fn interface(&self) -> &str {
        &self.interface
    }

    /// Returns the exact capability version supplied by the provider.
    #[must_use]
    pub const fn version(&self) -> &Version {
        &self.version
    }
}

/// A provider available inside an `Presolve` environment.
///
/// A provider may expose one or more versioned capabilities.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProviderDescriptor {
    id: ProviderId,
    name: String,
    capabilities: Vec<ProvidedCapability>,
}

impl ProviderDescriptor {
    /// Creates an environment provider descriptor.
    #[must_use]
    pub fn new(
        id: ProviderId,
        name: impl Into<String>,
        capabilities: Vec<ProvidedCapability>,
    ) -> Self {
        Self {
            id,
            name: name.into(),
            capabilities,
        }
    }

    /// Returns the provider identifier.
    #[must_use]
    pub const fn id(&self) -> &ProviderId {
        &self.id
    }

    /// Returns the human-readable provider name.
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Returns capabilities supplied by this provider.
    #[must_use]
    pub fn capabilities(&self) -> &[ProvidedCapability] {
        &self.capabilities
    }
}

/// Capabilities and resources available in one execution environment.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EnvironmentInventory {
    id: EnvironmentId,
    resources: EnvironmentResources,
    providers: Vec<ProviderDescriptor>,
}

impl EnvironmentInventory {
    /// Creates an environment inventory.
    #[must_use]
    pub const fn new(
        id: EnvironmentId,
        resources: EnvironmentResources,
        providers: Vec<ProviderDescriptor>,
    ) -> Self {
        Self {
            id,
            resources,
            providers,
        }
    }

    /// Returns the environment identifier.
    #[must_use]
    pub const fn id(&self) -> &EnvironmentId {
        &self.id
    }

    /// Returns allocatable environment resources.
    #[must_use]
    pub const fn resources(&self) -> EnvironmentResources {
        self.resources
    }

    /// Returns providers available in the environment.
    #[must_use]
    pub fn providers(&self) -> &[ProviderDescriptor] {
        &self.providers
    }
}
