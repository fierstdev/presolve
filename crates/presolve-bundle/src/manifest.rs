use presolve_contract::{ApplicationContract, ContractVersion, OutboundNetworkMode};
use presolve_core::{ArtifactDigest, InterfaceKind, ProductVersion, WorkloadKind};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

use crate::{BUNDLE_VERSION, BundleError, BundleVersion, ComponentArtifactInput};

/// Path of the canonical manifest inside a `.presolved` archive.
pub const MANIFEST_PATH: &str = "manifest.json";

/// Canonical path of the initial component artifact.
pub const COMPONENT_ARTIFACT_PATH: &str = "artifacts/main.wasm";

/// Returns the canonical component artifact path for `workload`.
#[must_use]
pub fn component_artifact_path(workload: &str) -> String {
    format!("artifacts/{workload}.wasm")
}

/// MIME media type used for WebAssembly artifacts.
pub const WASM_MEDIA_TYPE: &str = "application/wasm";

/// Canonical application-release manifest.
///
/// Release identity is derived from the canonical serialization of this
/// manifest. Artifact contents are incorporated transitively through their
/// content digests.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BundleManifest {
    bundle_version: BundleVersion,
    application: ApplicationManifest,
    requirements: RequirementsManifest,
    workloads: Vec<WorkloadManifest>,

    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    interfaces: Vec<InterfaceManifest>,

    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    relationships: Vec<RelationshipManifest>,
}

impl BundleManifest {
    /// Creates a manifest from declared workloads and their component artifacts.
    ///
    /// Contracts without explicit workloads retain the legacy implicit `main`
    /// component workload.
    ///
    /// # Errors
    ///
    /// Returns an error when component inputs are duplicated, missing, unexpected,
    /// or too large to represent in bundle format v0.1.
    pub fn for_components(
        contract: &ApplicationContract,
        components: &[ComponentArtifactInput<'_>],
    ) -> Result<Self, BundleError> {
        let application = ApplicationManifest {
            name: contract.application().name().to_owned(),
            version: contract.application().version().clone(),
            description: contract.application().description().map(str::to_owned),
        };

        let requirements = canonical_requirements(contract);
        let (interfaces, relationships) = canonical_topology(contract);

        let mut expected = if contract.workloads().is_empty() {
            vec![("main".to_owned(), WorkloadKind::Component)]
        } else {
            contract
                .workloads()
                .iter()
                .map(|workload| (workload.name().to_owned(), workload.kind()))
                .collect::<Vec<_>>()
        };

        expected.sort_by(|left, right| left.0.cmp(&right.0));

        let mut supplied = BTreeMap::<String, &[u8]>::new();

        for component in components {
            let workload = component.workload().to_owned();

            if supplied
                .insert(workload.clone(), component.bytes())
                .is_some()
            {
                return Err(BundleError::DuplicateWorkloadArtifact(workload));
            }
        }

        for (workload, _) in &expected {
            if !supplied.contains_key(workload) {
                return Err(BundleError::MissingWorkloadArtifact(workload.clone()));
            }
        }

        for workload in supplied.keys() {
            if !expected
                .iter()
                .any(|(expected_name, _)| expected_name == workload)
            {
                return Err(BundleError::UnexpectedWorkloadArtifact(workload.clone()));
            }
        }

        let mut workloads = Vec::with_capacity(expected.len());

        for (name, kind) in expected {
            let Some(bytes) = supplied.get(&name) else {
                return Err(BundleError::MissingWorkloadArtifact(name));
            };

            match kind {
                WorkloadKind::Component => {
                    let size = u64::try_from(bytes.len())
                        .map_err(|_| BundleError::ArtifactSizeOverflow { size: bytes.len() })?;

                    let artifact = ArtifactManifest {
                        path: component_artifact_path(&name),
                        digest: ArtifactDigest::from_content(bytes),
                        size,
                        media_type: WASM_MEDIA_TYPE.to_owned(),
                    };

                    workloads.push(WorkloadManifest {
                        name,
                        kind,
                        artifacts: vec![artifact],
                    });
                }
            }
        }

        Ok(Self {
            bundle_version: BUNDLE_VERSION,
            application,
            requirements,
            workloads,
            interfaces,
            relationships,
        })
    }

    /// Returns the bundle format version.
    #[must_use]
    pub const fn bundle_version(&self) -> BundleVersion {
        self.bundle_version
    }

    /// Returns application release metadata.
    #[must_use]
    pub const fn application(&self) -> &ApplicationManifest {
        &self.application
    }

    /// Returns canonical application requirements.
    #[must_use]
    pub const fn requirements(&self) -> &RequirementsManifest {
        &self.requirements
    }

    /// Returns workloads contained in the application release.
    #[must_use]
    pub fn workloads(&self) -> &[WorkloadManifest] {
        &self.workloads
    }

    /// Returns application-internal interfaces in canonical order.
    #[must_use]
    pub fn interfaces(&self) -> &[InterfaceManifest] {
        &self.interfaces
    }

    /// Returns application-internal relationships in canonical order.
    #[must_use]
    pub fn relationships(&self) -> &[RelationshipManifest] {
        &self.relationships
    }

    /// Serializes the semantic manifest into canonical JSON bytes.
    ///
    /// Collections whose source ordering is not semantically significant are
    /// normalized before the manifest is created. Struct field order is fixed
    /// by the bundle schema.
    ///
    /// # Errors
    ///
    /// Returns an error if JSON serialization fails.
    pub fn canonical_bytes(&self) -> Result<Vec<u8>, serde_json::Error> {
        serde_json::to_vec(self)
    }

    /// Returns the immutable application release digest.
    ///
    /// The digest covers the canonical manifest. Artifact bytes participate
    /// transitively because their content digests are stored in the manifest.
    ///
    /// # Errors
    ///
    /// Returns an error if canonical manifest serialization fails.
    pub fn release_digest(&self) -> Result<ArtifactDigest, serde_json::Error> {
        Ok(ArtifactDigest::from_content(&self.canonical_bytes()?))
    }
}

fn canonical_requirements(contract: &ApplicationContract) -> RequirementsManifest {
    let mut capabilities = contract
        .capabilities()
        .iter()
        .map(|capability| CapabilityManifest {
            interface: capability.interface().to_owned(),
            version: capability.version().to_string(),
            optional: capability.optional(),
        })
        .collect::<Vec<_>>();

    capabilities.sort_by(|left, right| {
        left.interface
            .cmp(&right.interface)
            .then_with(|| left.version.cmp(&right.version))
            .then_with(|| left.optional.cmp(&right.optional))
    });

    let mut allow = contract.network().allow().to_vec();
    allow.sort();
    allow.dedup();

    RequirementsManifest {
        contract_version: contract.contract_version(),
        capabilities,
        resources: ResourceManifest {
            memory_mib: contract.resources().memory_mib(),
            cpu_millis: contract.resources().cpu_millis(),
        },
        network: NetworkManifest {
            outbound: contract.network().outbound(),
            allow,
        },
    }
}

fn canonical_topology(
    contract: &ApplicationContract,
) -> (Vec<InterfaceManifest>, Vec<RelationshipManifest>) {
    let mut interfaces = contract
        .interfaces()
        .iter()
        .map(|interface| InterfaceManifest {
            name: interface.name().to_owned(),
            kind: interface.kind(),
        })
        .collect::<Vec<_>>();

    interfaces.sort_by(|left, right| {
        left.name
            .cmp(&right.name)
            .then_with(|| left.kind.cmp(&right.kind))
    });

    let mut relationships = contract
        .relationships()
        .iter()
        .map(|relationship| RelationshipManifest {
            from: relationship.from().to_owned(),
            to: relationship.to().to_owned(),
            interface: relationship.interface().to_owned(),
        })
        .collect::<Vec<_>>();

    relationships.sort_by(|left, right| {
        left.from
            .cmp(&right.from)
            .then_with(|| left.to.cmp(&right.to))
            .then_with(|| left.interface.cmp(&right.interface))
    });

    (interfaces, relationships)
}

/// Application metadata stored in the release manifest.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ApplicationManifest {
    name: String,
    version: ProductVersion,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    description: Option<String>,
}

impl ApplicationManifest {
    /// Returns the application name.
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Returns the application release version.
    #[must_use]
    pub const fn version(&self) -> &ProductVersion {
        &self.version
    }

    /// Returns the optional application description.
    #[must_use]
    pub fn description(&self) -> Option<&str> {
        self.description.as_deref()
    }
}

/// Canonical environment requirements stored in a release.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RequirementsManifest {
    contract_version: ContractVersion,
    capabilities: Vec<CapabilityManifest>,
    resources: ResourceManifest,
    network: NetworkManifest,
}

impl RequirementsManifest {
    /// Returns the originating Application Contract version.
    #[must_use]
    pub const fn contract_version(&self) -> ContractVersion {
        self.contract_version
    }

    /// Returns required capabilities in canonical order.
    #[must_use]
    pub fn capabilities(&self) -> &[CapabilityManifest] {
        &self.capabilities
    }

    /// Returns resource requirements.
    #[must_use]
    pub const fn resources(&self) -> ResourceManifest {
        self.resources
    }

    /// Returns network requirements.
    #[must_use]
    pub const fn network(&self) -> &NetworkManifest {
        &self.network
    }
}

/// Canonical capability requirement.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CapabilityManifest {
    interface: String,
    version: String,

    #[serde(default, skip_serializing_if = "is_false")]
    optional: bool,
}

impl CapabilityManifest {
    /// Returns the capability interface identifier.
    #[must_use]
    pub fn interface(&self) -> &str {
        &self.interface
    }

    /// Returns the accepted provider version requirement.
    #[must_use]
    pub fn version(&self) -> &str {
        &self.version
    }

    /// Returns whether the capability is optional.
    #[must_use]
    pub const fn optional(&self) -> bool {
        self.optional
    }
}

/// Canonical compute requirements.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ResourceManifest {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    memory_mib: Option<u64>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    cpu_millis: Option<u32>,
}

impl ResourceManifest {
    /// Returns minimum memory in mebibytes.
    #[must_use]
    pub const fn memory_mib(self) -> Option<u64> {
        self.memory_mib
    }

    /// Returns minimum CPU capacity in millicores.
    #[must_use]
    pub const fn cpu_millis(self) -> Option<u32> {
        self.cpu_millis
    }
}

/// Canonical network requirements.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct NetworkManifest {
    outbound: OutboundNetworkMode,

    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    allow: Vec<String>,
}

impl NetworkManifest {
    /// Returns the outbound networking mode.
    #[must_use]
    pub const fn outbound(&self) -> OutboundNetworkMode {
        self.outbound
    }

    /// Returns the canonical outbound allow list.
    #[must_use]
    pub fn allow(&self) -> &[String] {
        &self.allow
    }
}

/// One logical executable constituent of an application.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct WorkloadManifest {
    name: String,
    kind: WorkloadKind,
    artifacts: Vec<ArtifactManifest>,
}

impl WorkloadManifest {
    /// Returns the workload name.
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Returns the workload execution class.
    #[must_use]
    pub const fn kind(&self) -> WorkloadKind {
        self.kind
    }

    /// Returns artifact variants capable of realizing this workload.
    #[must_use]
    pub fn artifacts(&self) -> &[ArtifactManifest] {
        &self.artifacts
    }
}

/// Canonical application-internal interface.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct InterfaceManifest {
    name: String,
    kind: InterfaceKind,
}

impl InterfaceManifest {
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

/// Canonical directed relationship between application workloads.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RelationshipManifest {
    from: String,
    to: String,
    interface: String,
}

impl RelationshipManifest {
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

/// Immutable artifact referenced by a workload.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ArtifactManifest {
    path: String,
    digest: ArtifactDigest,
    size: u64,
    media_type: String,
}

impl ArtifactManifest {
    /// Returns the artifact path inside the bundle.
    #[must_use]
    pub fn path(&self) -> &str {
        &self.path
    }

    /// Returns the content digest of the artifact.
    #[must_use]
    pub const fn digest(&self) -> ArtifactDigest {
        self.digest
    }

    /// Returns the artifact length in bytes.
    #[must_use]
    pub const fn size(&self) -> u64 {
        self.size
    }

    /// Returns the artifact media type.
    #[must_use]
    pub fn media_type(&self) -> &str {
        &self.media_type
    }
}

#[allow(clippy::trivially_copy_pass_by_ref)]
const fn is_false(value: &bool) -> bool {
    !*value
}
