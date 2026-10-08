use presolve_contract::{ApplicationContract, ContractVersion, OutboundNetworkMode};
use presolve_core::{ArtifactDigest, ProductVersion};
use serde::{Deserialize, Serialize};

use crate::{BUNDLE_VERSION, BundleError, BundleVersion};

/// Path of the canonical manifest inside a `.presolve` archive.
pub const MANIFEST_PATH: &str = "manifest.json";

/// Canonical path of the initial component artifact.
pub const COMPONENT_ARTIFACT_PATH: &str = "artifacts/main.wasm";

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
}

impl BundleManifest {
    /// Creates the initial single-component application manifest.
    ///
    /// # Errors
    ///
    /// Returns an error if the component size cannot be represented by bundle
    /// format v0.1.
    pub fn for_component(
        contract: &ApplicationContract,
        component: &[u8],
    ) -> Result<Self, BundleError> {
        let application = ApplicationManifest {
            name: contract.application().name().to_owned(),
            version: contract.application().version().clone(),
            description: contract.application().description().map(str::to_owned),
        };

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

        let requirements = RequirementsManifest {
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
        };

        let size =
            u64::try_from(component.len()).map_err(|_| BundleError::ArtifactSizeOverflow {
                size: component.len(),
            })?;

        let artifact = ArtifactManifest {
            path: COMPONENT_ARTIFACT_PATH.to_owned(),
            digest: ArtifactDigest::from_content(component),
            size,
            media_type: WASM_MEDIA_TYPE.to_owned(),
        };

        Ok(Self {
            bundle_version: BUNDLE_VERSION,
            application,
            requirements,
            workloads: vec![WorkloadManifest {
                name: "main".to_owned(),
                kind: WorkloadKind::Component,
                artifacts: vec![artifact],
            }],
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

/// Workload execution class represented in a bundle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum WorkloadKind {
    /// WebAssembly Component workload.
    Component,
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
