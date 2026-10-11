use std::collections::BTreeSet;

use presolve_core::{ProviderId, WorkloadKind};
use semver::Version;
use thiserror::Error;

use crate::{ENVIRONMENT_SPECIFICATION_VERSION, EnvironmentSpecification};

/// Environment Specification semantic validation failure.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum EnvironmentSpecificationError {
    /// The specification uses a schema version not implemented by this release.
    #[error("unsupported environment specification version `{found}`; expected `{expected}`")]
    UnsupportedSpecificationVersion {
        /// Unsupported version found in the specification.
        found: crate::EnvironmentSpecificationVersion,

        /// Version implemented by this Presolve release.
        expected: crate::EnvironmentSpecificationVersion,
    },

    /// Environment names use stable lowercase kebab-case identifiers.
    #[error("invalid environment name `{0}`")]
    InvalidEnvironmentName(String),

    /// At least one workload execution class must be declared.
    #[error("environment must declare at least one supported workload kind")]
    MissingExecutionSupport,

    /// A workload execution class was declared more than once.
    #[error("workload kind `{0}` is declared more than once")]
    DuplicateWorkloadKind(WorkloadKind),

    /// Provider names use stable lowercase kebab-case identifiers.
    #[error("invalid provider name `{0}`")]
    InvalidProviderName(String),

    /// Provider IDs are unique inside one environment.
    #[error("provider id `{0}` is declared more than once")]
    DuplicateProviderId(ProviderId),

    /// Provider names are unique inside one environment.
    #[error("provider name `{0}` is declared more than once")]
    DuplicateProviderName(String),

    /// Every provider must expose at least one capability.
    #[error("provider `{0}` must expose at least one capability")]
    MissingProviderCapabilities(String),

    /// A provider capability interface is malformed.
    #[error("provider `{provider}` declares invalid capability interface `{interface}`")]
    InvalidCapabilityInterface {
        /// Provider containing the invalid declaration.
        provider: String,

        /// Invalid interface identifier.
        interface: String,
    },

    /// The same exact capability version is declared twice by one provider.
    #[error("provider `{provider}` declares capability `{interface}@{version}` more than once")]
    DuplicateProviderCapability {
        /// Provider containing the duplicate capability.
        provider: String,

        /// Duplicate capability interface.
        interface: String,

        /// Duplicate exact capability version.
        version: Version,
    },

    /// A capability feature identifier is malformed.
    #[error("provider `{provider}` capability `{interface}` declares invalid feature `{feature}`")]
    InvalidCapabilityFeature {
        /// Provider containing the invalid feature.
        provider: String,

        /// Capability interface containing the feature.
        interface: String,

        /// Invalid feature identifier.
        feature: String,
    },

    /// A capability feature is declared more than once.
    #[error(
        "provider `{provider}` capability `{interface}` declares feature `{feature}` more than once"
    )]
    DuplicateCapabilityFeature {
        /// Provider containing the duplicate feature.
        provider: String,

        /// Capability interface containing the feature.
        interface: String,

        /// Duplicate feature identifier.
        feature: String,
    },
}

pub(crate) fn validate(
    specification: &EnvironmentSpecification,
) -> Result<(), EnvironmentSpecificationError> {
    if specification.specification_version() != ENVIRONMENT_SPECIFICATION_VERSION {
        return Err(
            EnvironmentSpecificationError::UnsupportedSpecificationVersion {
                found: specification.specification_version(),
                expected: ENVIRONMENT_SPECIFICATION_VERSION,
            },
        );
    }

    if !valid_kebab_segment(specification.environment().name()) {
        return Err(EnvironmentSpecificationError::InvalidEnvironmentName(
            specification.environment().name().to_owned(),
        ));
    }

    validate_execution(specification)?;
    validate_providers(specification)
}

fn validate_execution(
    specification: &EnvironmentSpecification,
) -> Result<(), EnvironmentSpecificationError> {
    if specification.execution().workloads().is_empty() {
        return Err(EnvironmentSpecificationError::MissingExecutionSupport);
    }

    let mut workloads = BTreeSet::new();

    for workload in specification.execution().workloads() {
        if !workloads.insert(*workload) {
            return Err(EnvironmentSpecificationError::DuplicateWorkloadKind(
                *workload,
            ));
        }
    }

    Ok(())
}

fn validate_providers(
    specification: &EnvironmentSpecification,
) -> Result<(), EnvironmentSpecificationError> {
    let mut provider_ids = BTreeSet::new();
    let mut provider_names = BTreeSet::new();

    for provider in specification.providers() {
        if !valid_kebab_segment(provider.name()) {
            return Err(EnvironmentSpecificationError::InvalidProviderName(
                provider.name().to_owned(),
            ));
        }

        if !provider_ids.insert(*provider.id()) {
            return Err(EnvironmentSpecificationError::DuplicateProviderId(
                *provider.id(),
            ));
        }

        if !provider_names.insert(provider.name()) {
            return Err(EnvironmentSpecificationError::DuplicateProviderName(
                provider.name().to_owned(),
            ));
        }

        if provider.capabilities().is_empty() {
            return Err(EnvironmentSpecificationError::MissingProviderCapabilities(
                provider.name().to_owned(),
            ));
        }

        validate_provider_capabilities(provider.name(), provider.capabilities())?;
    }

    Ok(())
}

fn validate_provider_capabilities(
    provider: &str,
    capabilities: &[crate::ProvidedCapabilitySpecification],
) -> Result<(), EnvironmentSpecificationError> {
    let mut supplied_versions = BTreeSet::new();

    for capability in capabilities {
        if !valid_capability_interface(capability.interface()) {
            return Err(EnvironmentSpecificationError::InvalidCapabilityInterface {
                provider: provider.to_owned(),
                interface: capability.interface().to_owned(),
            });
        }

        let key = (capability.interface(), capability.version());

        if !supplied_versions.insert(key) {
            return Err(EnvironmentSpecificationError::DuplicateProviderCapability {
                provider: provider.to_owned(),
                interface: capability.interface().to_owned(),
                version: capability.version().clone(),
            });
        }

        let mut features = BTreeSet::new();

        for feature in capability.features() {
            if !valid_kebab_segment(feature) {
                return Err(EnvironmentSpecificationError::InvalidCapabilityFeature {
                    provider: provider.to_owned(),
                    interface: capability.interface().to_owned(),
                    feature: feature.clone(),
                });
            }

            if !features.insert(feature) {
                return Err(EnvironmentSpecificationError::DuplicateCapabilityFeature {
                    provider: provider.to_owned(),
                    interface: capability.interface().to_owned(),
                    feature: feature.clone(),
                });
            }
        }
    }

    Ok(())
}

fn valid_capability_interface(interface: &str) -> bool {
    let Some((namespace, remainder)) = interface.split_once(':') else {
        return false;
    };

    if remainder.contains(':') {
        return false;
    }

    let Some((package, name)) = remainder.split_once('/') else {
        return false;
    };

    if name.contains('/') {
        return false;
    }

    valid_kebab_segment(namespace) && valid_kebab_segment(package) && valid_kebab_segment(name)
}

fn valid_kebab_segment(segment: &str) -> bool {
    if segment.is_empty() || segment.len() > 63 {
        return false;
    }

    let bytes = segment.as_bytes();

    let Some(first) = bytes.first() else {
        return false;
    };

    let Some(last) = bytes.last() else {
        return false;
    };

    first.is_ascii_lowercase()
        && last.is_ascii_alphanumeric()
        && bytes
            .iter()
            .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || *byte == b'-')
}

#[cfg(test)]
mod tests {
    use presolve_core::{EnvironmentId, ProviderId, WorkloadKind};
    use semver::Version;

    use crate::{
        EnvironmentMetadata, EnvironmentSpecification, EnvironmentSpecificationVersion,
        ExecutionSupport, ProvidedCapabilitySpecification, ProviderSpecification, ResourceCapacity,
    };

    use super::*;

    fn object_capability() -> ProvidedCapabilitySpecification {
        ProvidedCapabilitySpecification::new("presolve:objects/store", Version::new(0, 1, 0))
    }

    fn provider() -> ProviderSpecification {
        ProviderSpecification::new(
            ProviderId::new(),
            "objects-memory",
            vec![object_capability()],
        )
    }

    fn specification(
        execution: ExecutionSupport,
        providers: Vec<ProviderSpecification>,
    ) -> EnvironmentSpecification {
        EnvironmentSpecification::current(
            EnvironmentMetadata::new(EnvironmentId::new(), "local-dev"),
            ResourceCapacity::new(4_096, 4_000),
            execution,
            providers,
        )
        .expect("fixture specification should be valid")
    }

    #[test]
    fn minimal_environment_is_valid() {
        let specification = specification(ExecutionSupport::components(), Vec::new());

        assert_eq!(
            specification.specification_version(),
            ENVIRONMENT_SPECIFICATION_VERSION
        );
        assert_eq!(specification.environment().name(), "local-dev");
        assert_eq!(specification.resources().memory_mib(), 4_096);
        assert_eq!(
            specification.execution().workloads(),
            [WorkloadKind::Component]
        );
    }

    #[test]
    fn zero_resource_capacity_is_representable() {
        let specification = EnvironmentSpecification::current(
            EnvironmentMetadata::new(EnvironmentId::new(), "drained"),
            ResourceCapacity::new(0, 0),
            ExecutionSupport::components(),
            Vec::new(),
        )
        .expect("zero capacity is valid environment supply");

        assert_eq!(specification.resources(), ResourceCapacity::new(0, 0));
    }

    #[test]
    fn unsupported_schema_version_is_rejected() {
        let error = EnvironmentSpecification::new(
            EnvironmentSpecificationVersion::new(9, 0),
            EnvironmentMetadata::new(EnvironmentId::new(), "local-dev"),
            ResourceCapacity::new(1, 1),
            ExecutionSupport::components(),
            Vec::new(),
        )
        .expect_err("unsupported schema version should fail");

        assert!(matches!(
            error,
            EnvironmentSpecificationError::UnsupportedSpecificationVersion { .. }
        ));
    }

    #[test]
    fn invalid_environment_name_is_rejected() {
        let error = EnvironmentSpecification::current(
            EnvironmentMetadata::new(EnvironmentId::new(), "Local Dev"),
            ResourceCapacity::new(1, 1),
            ExecutionSupport::components(),
            Vec::new(),
        )
        .expect_err("invalid environment name should fail");

        assert_eq!(
            error,
            EnvironmentSpecificationError::InvalidEnvironmentName("Local Dev".to_owned())
        );
    }

    #[test]
    fn empty_execution_support_is_rejected() {
        let error = EnvironmentSpecification::current(
            EnvironmentMetadata::new(EnvironmentId::new(), "local-dev"),
            ResourceCapacity::new(1, 1),
            ExecutionSupport::new(Vec::new()),
            Vec::new(),
        )
        .expect_err("environment needs an execution class");

        assert_eq!(
            error,
            EnvironmentSpecificationError::MissingExecutionSupport
        );
    }

    #[test]
    fn duplicate_workload_kind_is_rejected() {
        let error = EnvironmentSpecification::current(
            EnvironmentMetadata::new(EnvironmentId::new(), "local-dev"),
            ResourceCapacity::new(1, 1),
            ExecutionSupport::new(vec![WorkloadKind::Component, WorkloadKind::Component]),
            Vec::new(),
        )
        .expect_err("duplicate workload kind should fail");

        assert_eq!(
            error,
            EnvironmentSpecificationError::DuplicateWorkloadKind(WorkloadKind::Component)
        );
    }

    #[test]
    fn duplicate_provider_id_is_rejected() {
        let provider_id = ProviderId::new();

        let first =
            ProviderSpecification::new(provider_id, "objects-primary", vec![object_capability()]);
        let second =
            ProviderSpecification::new(provider_id, "objects-secondary", vec![object_capability()]);

        let error = EnvironmentSpecification::current(
            EnvironmentMetadata::new(EnvironmentId::new(), "local-dev"),
            ResourceCapacity::new(1, 1),
            ExecutionSupport::components(),
            vec![first, second],
        )
        .expect_err("duplicate provider id should fail");

        assert_eq!(
            error,
            EnvironmentSpecificationError::DuplicateProviderId(provider_id)
        );
    }

    #[test]
    fn duplicate_provider_name_is_rejected() {
        let first = provider();
        let second = ProviderSpecification::new(
            ProviderId::new(),
            "objects-memory",
            vec![object_capability()],
        );

        let error = EnvironmentSpecification::current(
            EnvironmentMetadata::new(EnvironmentId::new(), "local-dev"),
            ResourceCapacity::new(1, 1),
            ExecutionSupport::components(),
            vec![first, second],
        )
        .expect_err("duplicate provider name should fail");

        assert_eq!(
            error,
            EnvironmentSpecificationError::DuplicateProviderName("objects-memory".to_owned())
        );
    }

    #[test]
    fn provider_without_capabilities_is_rejected() {
        let empty = ProviderSpecification::new(ProviderId::new(), "empty-provider", Vec::new());

        let error = EnvironmentSpecification::current(
            EnvironmentMetadata::new(EnvironmentId::new(), "local-dev"),
            ResourceCapacity::new(1, 1),
            ExecutionSupport::components(),
            vec![empty],
        )
        .expect_err("empty provider should fail");

        assert_eq!(
            error,
            EnvironmentSpecificationError::MissingProviderCapabilities("empty-provider".to_owned())
        );
    }

    #[test]
    fn same_interface_at_different_exact_versions_is_valid() {
        let multi_version = ProviderSpecification::new(
            ProviderId::new(),
            "objects-versioned",
            vec![
                ProvidedCapabilitySpecification::new(
                    "presolve:objects/store",
                    Version::new(0, 1, 0),
                ),
                ProvidedCapabilitySpecification::new(
                    "presolve:objects/store",
                    Version::new(0, 2, 0),
                ),
            ],
        );

        let specification = specification(ExecutionSupport::components(), vec![multi_version]);

        assert_eq!(specification.providers()[0].capabilities().len(), 2);
    }

    #[test]
    fn duplicate_exact_provider_capability_is_rejected() {
        let duplicate = ProviderSpecification::new(
            ProviderId::new(),
            "objects-duplicate",
            vec![object_capability(), object_capability()],
        );

        let error = EnvironmentSpecification::current(
            EnvironmentMetadata::new(EnvironmentId::new(), "local-dev"),
            ResourceCapacity::new(1, 1),
            ExecutionSupport::components(),
            vec![duplicate],
        )
        .expect_err("duplicate exact capability should fail");

        assert!(matches!(
            error,
            EnvironmentSpecificationError::DuplicateProviderCapability {
                interface,
                version,
                ..
            } if interface == "presolve:objects/store"
                && version == Version::new(0, 1, 0)
        ));
    }

    #[test]
    fn malformed_capability_interface_is_rejected() {
        let malformed = ProviderSpecification::new(
            ProviderId::new(),
            "broken-provider",
            vec![ProvidedCapabilitySpecification::new(
                "objects",
                Version::new(0, 1, 0),
            )],
        );

        let error = EnvironmentSpecification::current(
            EnvironmentMetadata::new(EnvironmentId::new(), "local-dev"),
            ResourceCapacity::new(1, 1),
            ExecutionSupport::components(),
            vec![malformed],
        )
        .expect_err("malformed capability interface should fail");

        assert!(matches!(
            error,
            EnvironmentSpecificationError::InvalidCapabilityInterface {
                interface,
                ..
            } if interface == "objects"
        ));
    }

    #[test]
    fn duplicate_capability_feature_is_rejected() {
        let capability = object_capability().with_features(["range-read", "range-read"]);
        let provider =
            ProviderSpecification::new(ProviderId::new(), "objects-memory", vec![capability]);

        let error = EnvironmentSpecification::current(
            EnvironmentMetadata::new(EnvironmentId::new(), "local-dev"),
            ResourceCapacity::new(1, 1),
            ExecutionSupport::components(),
            vec![provider],
        )
        .expect_err("duplicate feature should fail");

        assert!(matches!(
            error,
            EnvironmentSpecificationError::DuplicateCapabilityFeature {
                feature,
                ..
            } if feature == "range-read"
        ));
    }

    #[test]
    fn malformed_capability_feature_is_rejected() {
        let capability = object_capability().with_features(["Range Read"]);
        let provider =
            ProviderSpecification::new(ProviderId::new(), "objects-memory", vec![capability]);

        let error = EnvironmentSpecification::current(
            EnvironmentMetadata::new(EnvironmentId::new(), "local-dev"),
            ResourceCapacity::new(1, 1),
            ExecutionSupport::components(),
            vec![provider],
        )
        .expect_err("malformed feature should fail");

        assert!(matches!(
            error,
            EnvironmentSpecificationError::InvalidCapabilityFeature {
                feature,
                ..
            } if feature == "Range Read"
        ));
    }
}
