use presolve_core::{EnvironmentId, ProviderId};
use semver::Version;

/// Binding between an application capability and an environment provider.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CapabilityBinding {
    interface: String,
    provider_id: ProviderId,
    provider_name: String,
    provider_version: Version,
}

impl CapabilityBinding {
    pub(crate) fn new(
        interface: String,
        provider_id: ProviderId,
        provider_name: String,
        provider_version: Version,
    ) -> Self {
        Self {
            interface,
            provider_id,
            provider_name,
            provider_version,
        }
    }

    /// Returns the capability interface being satisfied.
    #[must_use]
    pub fn interface(&self) -> &str {
        &self.interface
    }

    /// Returns the selected provider identifier.
    #[must_use]
    pub const fn provider_id(&self) -> &ProviderId {
        &self.provider_id
    }

    /// Returns the selected provider name.
    #[must_use]
    pub fn provider_name(&self) -> &str {
        &self.provider_name
    }

    /// Returns the exact selected provider capability version.
    #[must_use]
    pub const fn provider_version(&self) -> &Version {
        &self.provider_version
    }
}

/// Deterministic plan for running an application in an environment.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DeploymentPlan {
    environment_id: EnvironmentId,
    bindings: Vec<CapabilityBinding>,
    unbound_optional_capabilities: Vec<String>,
}

impl DeploymentPlan {
    pub(crate) fn new(
        environment_id: EnvironmentId,
        bindings: Vec<CapabilityBinding>,
        unbound_optional_capabilities: Vec<String>,
    ) -> Self {
        Self {
            environment_id,
            bindings,
            unbound_optional_capabilities,
        }
    }

    /// Returns the selected environment.
    #[must_use]
    pub const fn environment_id(&self) -> &EnvironmentId {
        &self.environment_id
    }

    /// Returns resolved capability bindings.
    #[must_use]
    pub fn bindings(&self) -> &[CapabilityBinding] {
        &self.bindings
    }

    /// Returns optional capabilities for which no provider was selected.
    #[must_use]
    pub fn unbound_optional_capabilities(&self) -> &[String] {
        &self.unbound_optional_capabilities
    }
}
