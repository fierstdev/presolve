use std::collections::BTreeSet;

use presolve_capability::CapabilityContract;
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
    semantic_contract: Option<CapabilityContract>,
    supported_features: Vec<String>,
}

impl ProvidedCapability {
    /// Creates a provider capability declaration.
    #[must_use]
    pub fn new(interface: impl Into<String>, version: Version) -> Self {
        Self {
            interface: interface.into(),
            version,
            semantic_contract: None,
            supported_features: Vec::new(),
        }
    }

    /// Creates a provider capability declaration from a canonical semantic
    /// contract.
    #[must_use]
    pub fn from_contract(contract: CapabilityContract) -> Self {
        Self {
            interface: contract.interface().to_owned(),
            version: contract.version().clone(),
            semantic_contract: Some(contract),
            supported_features: Vec::new(),
        }
    }

    /// Returns the capability interface identifier.
    #[must_use]
    pub fn interface(&self) -> &str {
        &self.interface
    }

    /// Returns the exact capability version supplied by the provider.
    #[must_use]
    pub const fn version(&self) -> &Version {
        &self.version
    }

    /// Returns the canonical semantic contract retained by this declaration,
    /// when one was supplied.
    #[must_use]
    pub fn semantic_contract(&self) -> Option<&CapabilityContract> {
        self.semantic_contract.as_ref()
    }

    /// Returns optional semantic features explicitly advertised by this provider.
    ///
    /// The canonical capability contract defines the valid feature vocabulary;
    /// it does not imply that every provider implements every optional feature.
    #[must_use]
    pub fn supported_features(&self) -> &[String] {
        &self.supported_features
    }

    /// Advertises optional semantic features implemented by this provider.
    ///
    /// # Errors
    ///
    /// Returns an error if no semantic contract is attached, a feature is not
    /// declared by that contract, or the same feature is advertised twice.
    pub fn with_supported_features<I, S>(
        mut self,
        features: I,
    ) -> Result<Self, CapabilityFeatureError>
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        let contract = self
            .semantic_contract
            .as_ref()
            .ok_or(CapabilityFeatureError::MissingSemanticContract)?;

        let mut supported = BTreeSet::new();

        for feature in features {
            let feature = feature.into();

            if !contract
                .optional_features()
                .iter()
                .any(|declared| declared == &feature)
            {
                return Err(CapabilityFeatureError::UnknownFeature {
                    interface: contract.interface().to_owned(),
                    feature,
                });
            }

            if !supported.insert(feature.clone()) {
                return Err(CapabilityFeatureError::DuplicateFeature { feature });
            }
        }

        self.supported_features = supported.into_iter().collect();
        Ok(self)
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

/// Invalid optional-feature advertisement for an environment capability.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CapabilityFeatureError {
    /// Optional features require an attached semantic contract.
    MissingSemanticContract,

    /// The provider advertised a feature outside the canonical contract vocabulary.
    UnknownFeature {
        /// Capability interface whose vocabulary was checked.
        interface: String,

        /// Unknown advertised feature.
        feature: String,
    },

    /// The provider advertised the same optional feature more than once.
    DuplicateFeature {
        /// Duplicated feature identifier.
        feature: String,
    },
}

impl std::fmt::Display for CapabilityFeatureError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::MissingSemanticContract => write!(
                formatter,
                "optional capability features require an attached semantic contract"
            ),
            Self::UnknownFeature { interface, feature } => write!(
                formatter,
                "feature `{feature}` is not declared by capability contract `{interface}`"
            ),
            Self::DuplicateFeature { feature } => {
                write!(
                    formatter,
                    "feature `{feature}` is advertised more than once"
                )
            }
        }
    }
}

impl std::error::Error for CapabilityFeatureError {}

#[cfg(test)]
mod tests {
    use presolve_capability::{OBJECT_STORE_INTERFACE, object_store_contract};

    use super::*;

    #[test]
    fn canonical_object_store_contract_can_be_advertised_by_environment() {
        let contract = object_store_contract().expect("object-store contract should be valid");
        let expected_version = contract.version().clone();
        let provider_id = ProviderId::new();

        let environment = EnvironmentInventory::new(
            EnvironmentId::new(),
            EnvironmentResources::new(512, 1_000),
            vec![ProviderDescriptor::new(
                provider_id,
                "memory-objects",
                vec![ProvidedCapability::from_contract(contract.clone())],
            )],
        );

        let capability = &environment.providers()[0].capabilities()[0];

        assert_eq!(capability.interface(), OBJECT_STORE_INTERFACE);
        assert_eq!(capability.version(), &expected_version);
        assert_eq!(capability.semantic_contract(), Some(&contract));
    }

    #[test]
    fn generic_capability_declaration_remains_supported() {
        let capability = ProvidedCapability::new("example:custom/service", Version::new(1, 2, 3));

        assert_eq!(capability.interface(), "example:custom/service");
        assert_eq!(capability.version(), &Version::new(1, 2, 3));
        assert_eq!(capability.semantic_contract(), None);
    }
}

#[cfg(test)]
mod feature_advertisement_tests {
    use presolve_capability::object_store_contract;

    use super::{CapabilityFeatureError, ProvidedCapability};

    #[test]
    fn optional_features_are_not_advertised_implicitly() {
        let capability = ProvidedCapability::from_contract(
            object_store_contract().expect("object contract should be valid"),
        );

        let expected: &[String] = &[];
        assert_eq!(capability.supported_features(), expected);
    }

    #[test]
    fn provider_can_explicitly_advertise_declared_optional_features() {
        let capability = ProvidedCapability::from_contract(
            object_store_contract().expect("object contract should be valid"),
        )
        .with_supported_features(["user-metadata", "range-read"])
        .expect("declared optional features should be accepted");

        assert_eq!(
            capability.supported_features(),
            ["range-read", "user-metadata"]
        );
    }

    #[test]
    fn provider_cannot_advertise_unknown_optional_feature() {
        let error = ProvidedCapability::from_contract(
            object_store_contract().expect("object contract should be valid"),
        )
        .with_supported_features(["multipart-upload"])
        .expect_err("unknown optional feature should be rejected");

        assert_eq!(
            error,
            CapabilityFeatureError::UnknownFeature {
                interface: "presolve:objects/store".to_owned(),
                feature: "multipart-upload".to_owned(),
            }
        );
    }
}
