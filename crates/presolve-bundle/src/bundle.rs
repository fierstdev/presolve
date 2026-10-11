use std::{
    collections::{BTreeMap, BTreeSet},
    io::{Cursor, Read},
};

use presolve_contract::ApplicationContract;
use presolve_core::ArtifactDigest;
use semver::VersionReq;
use tar::{Archive, Builder, EntryType, Header};
use thiserror::Error;

use crate::{
    BUNDLE_VERSION, BundleManifest, BundleVersion, ComponentArtifactInput, MANIFEST_PATH,
    component_artifact_path,
};

/// Complete immutable `.presolved` application bundle.
///
/// Application release identity is derived from the canonical manifest rather
/// than archive-container metadata.
#[derive(Debug, Clone)]
pub struct PresolveBundle {
    manifest: BundleManifest,
    artifacts: BTreeMap<String, Vec<u8>>,
}

impl PresolveBundle {
    /// Creates a bundle containing one WebAssembly Component workload.
    ///
    /// # Errors
    ///
    /// Returns an error if the component cannot be represented by the bundle
    /// format.
    /// Creates a bundle containing one WebAssembly Component workload.
    ///
    /// Contracts with no explicit workloads use the legacy implicit `main`
    /// workload. Contracts with exactly one declared workload use that workload's
    /// name.
    ///
    /// # Errors
    ///
    /// Returns an error if the contract declares multiple workloads or the
    /// component cannot be represented by the bundle format.
    pub fn from_component(
        contract: &ApplicationContract,
        component: &[u8],
    ) -> Result<Self, BundleError> {
        let workload = match contract.workloads() {
            [] => "main",
            [workload] => workload.name(),
            workloads => {
                return Err(BundleError::MultipleWorkloadsRequireExplicitArtifacts {
                    count: workloads.len(),
                });
            }
        };

        Self::from_components(
            contract,
            &[ComponentArtifactInput::new(workload, component)],
        )
    }

    /// Creates a bundle containing explicit component workload artifacts.
    ///
    /// Input ordering does not affect bundle or release identity.
    ///
    /// # Errors
    ///
    /// Returns an error when supplied artifacts do not exactly match the
    /// application's declared workloads, when duplicate inputs are supplied, or
    /// when an artifact cannot be represented by the bundle format.
    pub fn from_components(
        contract: &ApplicationContract,
        components: &[ComponentArtifactInput<'_>],
    ) -> Result<Self, BundleError> {
        let manifest = BundleManifest::for_components(contract, components)?;

        let mut artifacts = BTreeMap::new();

        for component in components {
            let path = component_artifact_path(component.workload());

            if artifacts.insert(path, component.bytes().to_vec()).is_some() {
                return Err(BundleError::DuplicateWorkloadArtifact(
                    component.workload().to_owned(),
                ));
            }
        }

        Ok(Self {
            manifest,
            artifacts,
        })
    }

    /// Returns the canonical bundle manifest.
    #[must_use]
    pub const fn manifest(&self) -> &BundleManifest {
        &self.manifest
    }

    /// Returns the immutable application release digest.
    ///
    /// # Errors
    ///
    /// Returns an error if canonical manifest serialization fails.
    pub fn release_digest(&self) -> Result<ArtifactDigest, BundleError> {
        self.manifest
            .release_digest()
            .map_err(BundleError::Manifest)
    }

    /// Validates manifest topology and all manifest-to-artifact integrity
    /// relationships.
    ///
    /// # Errors
    ///
    /// Returns an error when the bundle version is unsupported, topology is
    /// internally inconsistent, an artifact is missing or unexpected, or
    /// artifact size/digest metadata does not match the bundled bytes.
    pub fn verify(&self) -> Result<(), BundleError> {
        if self.manifest.bundle_version() != BUNDLE_VERSION {
            return Err(BundleError::UnsupportedBundleVersion {
                found: self.manifest.bundle_version(),
            });
        }

        verify_manifest_requirements(&self.manifest)?;

        let mut workload_names = BTreeSet::new();
        let mut referenced = BTreeSet::new();

        for workload in self.manifest.workloads() {
            if !workload_names.insert(workload.name().to_owned()) {
                return Err(BundleError::DuplicateManifestWorkload(
                    workload.name().to_owned(),
                ));
            }
            if workload.artifacts().is_empty() {
                return Err(BundleError::InvalidManifest(
                    "every workload must reference at least one artifact",
                ));
            }

            for artifact in workload.artifacts() {
                if !referenced.insert(artifact.path().to_owned()) {
                    return Err(BundleError::DuplicateManifestArtifact(
                        artifact.path().to_owned(),
                    ));
                }

                let bytes = self
                    .artifacts
                    .get(artifact.path())
                    .ok_or_else(|| BundleError::MissingArtifact(artifact.path().to_owned()))?;

                let actual_size = u64::try_from(bytes.len())
                    .map_err(|_| BundleError::ArtifactSizeOverflow { size: bytes.len() })?;

                if actual_size != artifact.size() {
                    return Err(BundleError::ArtifactSizeMismatch {
                        path: artifact.path().to_owned(),
                        expected: artifact.size(),
                        actual: actual_size,
                    });
                }

                let actual_digest = ArtifactDigest::from_content(bytes);

                if actual_digest != artifact.digest() {
                    return Err(BundleError::ArtifactDigestMismatch {
                        path: artifact.path().to_owned(),
                        expected: artifact.digest(),
                        actual: actual_digest,
                    });
                }
            }
        }

        let mut interface_names = BTreeSet::new();

        for interface in self.manifest.interfaces() {
            if !interface_names.insert(interface.name().to_owned()) {
                return Err(BundleError::DuplicateManifestInterface(
                    interface.name().to_owned(),
                ));
            }
        }

        let mut relationships = BTreeSet::new();

        for relationship in self.manifest.relationships() {
            if !workload_names.contains(relationship.from()) {
                return Err(BundleError::UnknownRelationshipSourceWorkload(
                    relationship.from().to_owned(),
                ));
            }

            if !workload_names.contains(relationship.to()) {
                return Err(BundleError::UnknownRelationshipTargetWorkload(
                    relationship.to().to_owned(),
                ));
            }

            if !interface_names.contains(relationship.interface()) {
                return Err(BundleError::UnknownRelationshipInterface(
                    relationship.interface().to_owned(),
                ));
            }

            if relationship.from() == relationship.to() {
                return Err(BundleError::SelfRelationship(
                    relationship.from().to_owned(),
                ));
            }

            if !relationships.insert((
                relationship.from(),
                relationship.to(),
                relationship.interface(),
            )) {
                return Err(BundleError::DuplicateManifestRelationship {
                    from: relationship.from().to_owned(),
                    to: relationship.to().to_owned(),
                    interface: relationship.interface().to_owned(),
                });
            }
        }

        for path in self.artifacts.keys() {
            if !referenced.contains(path) {
                return Err(BundleError::UnexpectedArtifact(path.clone()));
            }
        }

        Ok(())
    }

    /// Encodes the bundle as a deterministic `.presolved` archive.
    ///
    /// Archive metadata is normalized and entries are emitted in stable order.
    /// Release identity nevertheless comes from the canonical manifest, not
    /// from archive-container bytes.
    ///
    /// # Errors
    ///
    /// Returns an error if bundle verification, manifest serialization, or
    /// archive construction fails.
    pub fn encode(&self) -> Result<Vec<u8>, BundleError> {
        self.verify()?;

        let manifest = self
            .manifest
            .canonical_bytes()
            .map_err(BundleError::Manifest)?;

        let mut builder = Builder::new(Vec::new());

        append_entry(&mut builder, MANIFEST_PATH, &manifest)?;

        for (path, bytes) in &self.artifacts {
            append_entry(&mut builder, path, bytes)?;
        }

        builder.into_inner().map_err(BundleError::Archive)
    }

    /// Decodes and verifies a `.presolved` archive.
    ///
    /// # Errors
    ///
    /// Returns an error for malformed archives, unsafe or duplicate paths,
    /// invalid manifests, unsupported versions, or artifact integrity
    /// failures.
    pub fn decode(bytes: &[u8]) -> Result<Self, BundleError> {
        let mut archive = Archive::new(Cursor::new(bytes));
        let mut entries = BTreeMap::<String, Vec<u8>>::new();

        for entry in archive.entries().map_err(BundleError::Archive)? {
            let mut entry = entry.map_err(BundleError::Archive)?;

            if !entry.header().entry_type().is_file() {
                return Err(BundleError::NonFileEntry);
            }

            let path = entry.path().map_err(BundleError::Archive)?.into_owned();

            let path = path.to_str().ok_or(BundleError::NonUtf8Path)?.to_owned();

            validate_path(&path)?;

            if path != MANIFEST_PATH && !path.starts_with("artifacts/") {
                return Err(BundleError::UnexpectedEntry(path));
            }

            let mut content = Vec::new();

            entry
                .read_to_end(&mut content)
                .map_err(BundleError::Archive)?;

            if entries.insert(path.clone(), content).is_some() {
                return Err(BundleError::DuplicateEntry(path));
            }
        }

        let manifest_bytes = entries
            .remove(MANIFEST_PATH)
            .ok_or(BundleError::MissingManifest)?;

        let mut manifest: BundleManifest =
            serde_json::from_slice(&manifest_bytes).map_err(BundleError::Manifest)?;

        manifest.normalize();

        let bundle = Self {
            manifest,
            artifacts: entries,
        };

        bundle.verify()?;

        Ok(bundle)
    }
}

/// Failure while creating, decoding, or validating a `.presolved` bundle.
#[derive(Debug, Error)]
pub enum BundleError {
    /// Archive encoding or decoding failed.
    #[error("bundle archive error: {0}")]
    Archive(std::io::Error),

    /// Canonical manifest encoding or decoding failed.
    #[error("bundle manifest error: {0}")]
    Manifest(serde_json::Error),

    /// The archive does not contain its canonical manifest.
    #[error("bundle does not contain `{MANIFEST_PATH}`")]
    MissingManifest,

    /// The archive contains a path that cannot safely be interpreted.
    #[error("bundle contains an invalid entry path `{0}`")]
    InvalidPath(String),

    /// An archive entry path is not UTF-8.
    #[error("bundle contains a non-UTF-8 entry path")]
    NonUtf8Path,

    /// Only regular files are valid bundle entries in format v0.1.
    #[error("bundle contains a non-file archive entry")]
    NonFileEntry,

    /// An archive path occurred more than once.
    #[error("bundle contains duplicate archive entry `{0}`")]
    DuplicateEntry(String),

    /// An entry is not defined by bundle format v0.1.
    #[error("bundle contains unexpected archive entry `{0}`")]
    UnexpectedEntry(String),

    /// The manifest uses an unsupported bundle version.
    #[error("unsupported bundle format version `{found}`")]
    UnsupportedBundleVersion {
        /// Unsupported bundle version.
        found: BundleVersion,
    },

    /// A single-artifact constructor was used for a multi-workload application.
    #[error("application declares {count} workloads; explicit workload artifacts are required")]
    MultipleWorkloadsRequireExplicitArtifacts {
        /// Number of declared workloads.
        count: usize,
    },

    /// More than one artifact was supplied for the same workload.
    #[error("component artifact for workload `{0}` was supplied more than once")]
    DuplicateWorkloadArtifact(String),

    /// A declared workload has no supplied component artifact.
    #[error("component artifact for workload `{0}` is missing")]
    MissingWorkloadArtifact(String),

    /// An artifact was supplied for a workload not declared by the application.
    #[error("component artifact was supplied for undeclared workload `{0}`")]
    UnexpectedWorkloadArtifact(String),

    /// The manifest declares the same capability interface more than once.
    #[error("bundle manifest declares capability `{0}` more than once")]
    DuplicateManifestCapability(String),

    /// A manifest capability interface identifier is malformed.
    #[error("bundle manifest declares invalid capability interface `{0}`")]
    InvalidManifestCapabilityInterface(String),

    /// A manifest capability version requirement is malformed.
    #[error(
        "bundle manifest capability `{interface}` declares invalid version requirement `{version}`"
    )]
    InvalidManifestCapabilityVersion {
        /// Capability interface containing the invalid version requirement.
        interface: String,

        /// Invalid semantic version requirement.
        version: String,
    },

    /// A manifest capability feature identifier is malformed.
    #[error("bundle manifest capability `{interface}` declares invalid feature `{feature}`")]
    InvalidManifestCapabilityFeature {
        /// Capability interface containing the invalid feature.
        interface: String,

        /// Invalid feature identifier.
        feature: String,
    },

    /// A manifest capability feature occurs more than once across required and
    /// preferred feature requirements.
    #[error("bundle manifest capability `{interface}` declares feature `{feature}` more than once")]
    DuplicateManifestCapabilityFeature {
        /// Capability interface containing the duplicate feature.
        interface: String,

        /// Duplicate feature identifier.
        feature: String,
    },

    /// The manifest declares the same workload name more than once.
    #[error("bundle manifest declares workload `{0}` more than once")]
    DuplicateManifestWorkload(String),

    /// The manifest declares the same application-internal interface more than
    /// once.
    #[error("bundle manifest declares interface `{0}` more than once")]
    DuplicateManifestInterface(String),

    /// A manifest relationship references a source workload that does not exist.
    #[error("bundle manifest relationship references unknown source workload `{0}`")]
    UnknownRelationshipSourceWorkload(String),

    /// A manifest relationship references a target workload that does not exist.
    #[error("bundle manifest relationship references unknown target workload `{0}`")]
    UnknownRelationshipTargetWorkload(String),

    /// A manifest relationship references an interface that does not exist.
    #[error("bundle manifest relationship references unknown interface `{0}`")]
    UnknownRelationshipInterface(String),

    /// A manifest relationship connects a workload to itself.
    #[error("bundle manifest relationship connects workload `{0}` to itself")]
    SelfRelationship(String),

    /// The manifest declares the same workload relationship more than once.
    #[error(
        "bundle manifest declares relationship `{from}` -> `{to}` using `{interface}` more than once"
    )]
    DuplicateManifestRelationship {
        /// Workload initiating the interaction.
        from: String,

        /// Workload receiving the interaction.
        to: String,

        /// Application-local interface used by the relationship.
        interface: String,
    },

    /// An artifact is too large to represent in the bundle format.
    #[error("artifact size `{size}` cannot be represented by bundle format v0.1")]
    ArtifactSizeOverflow {
        /// Host-side artifact length.
        size: usize,
    },

    /// The manifest is structurally invalid.
    #[error("invalid bundle manifest: {0}")]
    InvalidManifest(&'static str),

    /// The same artifact path is referenced more than once.
    #[error("bundle manifest references artifact `{0}` more than once")]
    DuplicateManifestArtifact(String),

    /// A manifest artifact does not exist in the archive.
    #[error("bundle is missing artifact `{0}`")]
    MissingArtifact(String),

    /// An artifact exists but is not referenced by the manifest.
    #[error("bundle contains unreferenced artifact `{0}`")]
    UnexpectedArtifact(String),

    /// Bundled bytes do not match the recorded artifact size.
    #[error(
        "artifact `{path}` size mismatch: expected {expected} bytes, \
         found {actual}"
    )]
    ArtifactSizeMismatch {
        /// Artifact path.
        path: String,

        /// Manifest size.
        expected: u64,

        /// Actual size.
        actual: u64,
    },

    /// Bundled bytes do not match the recorded content digest.
    #[error(
        "artifact `{path}` digest mismatch: expected {expected}, \
         found {actual}"
    )]
    ArtifactDigestMismatch {
        /// Artifact path.
        path: String,

        /// Manifest digest.
        expected: ArtifactDigest,

        /// Actual content digest.
        actual: ArtifactDigest,
    },
}

fn append_entry(
    builder: &mut Builder<Vec<u8>>,
    path: &str,
    bytes: &[u8],
) -> Result<(), BundleError> {
    let mut header = Header::new_gnu();

    header.set_path(path).map_err(BundleError::Archive)?;

    let size = u64::try_from(bytes.len())
        .map_err(|_| BundleError::ArtifactSizeOverflow { size: bytes.len() })?;

    header.set_entry_type(EntryType::Regular);
    header.set_mode(0o644);
    header.set_uid(0);
    header.set_gid(0);
    header.set_mtime(0);
    header.set_size(size);
    header.set_cksum();

    builder
        .append(&header, Cursor::new(bytes))
        .map_err(BundleError::Archive)
}

fn verify_manifest_requirements(manifest: &BundleManifest) -> Result<(), BundleError> {
    if manifest.workloads().is_empty() {
        return Err(BundleError::InvalidManifest(
            "bundle must contain at least one workload",
        ));
    }

    verify_capability_requirements(manifest)
}

fn verify_capability_requirements(manifest: &BundleManifest) -> Result<(), BundleError> {
    let mut interfaces = BTreeSet::new();

    for capability in manifest.requirements().capabilities() {
        let interface = capability.interface();

        if !valid_capability_interface(interface) {
            return Err(BundleError::InvalidManifestCapabilityInterface(
                interface.to_owned(),
            ));
        }

        if !interfaces.insert(interface) {
            return Err(BundleError::DuplicateManifestCapability(
                interface.to_owned(),
            ));
        }

        if VersionReq::parse(capability.version()).is_err() {
            return Err(BundleError::InvalidManifestCapabilityVersion {
                interface: interface.to_owned(),
                version: capability.version().to_owned(),
            });
        }

        let mut features = BTreeSet::new();

        for feature in capability
            .required_features()
            .iter()
            .chain(capability.preferred_features())
        {
            if !valid_kebab_segment(feature) {
                return Err(BundleError::InvalidManifestCapabilityFeature {
                    interface: interface.to_owned(),
                    feature: feature.clone(),
                });
            }

            if !features.insert(feature.as_str()) {
                return Err(BundleError::DuplicateManifestCapabilityFeature {
                    interface: interface.to_owned(),
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

fn validate_path(path: &str) -> Result<(), BundleError> {
    if path.is_empty()
        || path.starts_with('/')
        || path
            .split('/')
            .any(|component| component.is_empty() || component == "." || component == "..")
    {
        return Err(BundleError::InvalidPath(path.to_owned()));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use presolve_contract::{ApplicationContract, parse_contract};

    use super::*;
    use crate::{COMPONENT_ARTIFACT_PATH, InterfaceKind, WorkloadKind};

    const COMPONENT: &[u8] = b"example component bytes";
    const API_COMPONENT: &[u8] = b"api component bytes";
    const WORKER_COMPONENT: &[u8] = b"worker component bytes";

    fn multi_workload_contract() -> ApplicationContract {
        parse_contract(
            r#"
    contract_version = "0.1"

    [application]
    name = "multi-example"
    version = "1.0.0"

    [[workloads]]
    name = "api"
    kind = "component"

    [[workloads]]
    name = "worker"
    kind = "component"
    "#,
        )
        .expect("multi-workload contract should parse")
    }

    fn topology_contract() -> ApplicationContract {
        parse_contract(
            r#"
    contract_version = "0.1"

    [application]
    name = "topology-example"
    version = "1.0.0"

    [[workloads]]
    name = "api"
    kind = "component"

    [[workloads]]
    name = "worker"
    kind = "component"

    [[interfaces]]
    name = "jobs"
    kind = "request_response"

    [[interfaces]]
    name = "audit"
    kind = "event"

    [[relationships]]
    from = "api"
    to = "worker"
    interface = "jobs"

    [[relationships]]
    from = "worker"
    to = "api"
    interface = "audit"
    "#,
        )
        .expect("topology contract should parse")
    }

    fn capability_feature_bundle() -> PresolveBundle {
        let contract = parse_contract(
            r#"
contract_version = "0.1"

[application]
name = "feature-example"
version = "1.0.0"

[[capabilities]]
interface = "presolve:objects/store"
version = "^0.1"
required_features = ["range-read", "conditional-write"]
preferred_features = ["user-metadata"]
"#,
        )
        .expect("feature contract should parse");

        PresolveBundle::from_component(&contract, COMPONENT).expect("feature bundle should build")
    }

    fn topology_bundle() -> PresolveBundle {
        PresolveBundle::from_components(
            &topology_contract(),
            &[
                ComponentArtifactInput::new("api", API_COMPONENT),
                ComponentArtifactInput::new("worker", WORKER_COMPONENT),
            ],
        )
        .expect("topology bundle should build")
    }

    fn mutate_manifest(bundle: &mut PresolveBundle, mutate: impl FnOnce(&mut serde_json::Value)) {
        let mut manifest =
            serde_json::to_value(bundle.manifest()).expect("manifest should serialize");

        mutate(&mut manifest);

        bundle.manifest =
            serde_json::from_value(manifest).expect("mutated manifest should deserialize");
    }

    fn decode_mutated_manifest(
        bundle: &PresolveBundle,
        mutate: impl FnOnce(&mut serde_json::Value),
    ) -> Result<PresolveBundle, BundleError> {
        let mut manifest =
            serde_json::to_value(bundle.manifest()).expect("manifest should serialize");

        mutate(&mut manifest);

        let manifest = serde_json::to_vec(&manifest).expect("mutated manifest should serialize");

        let mut builder = Builder::new(Vec::new());

        append_entry(&mut builder, MANIFEST_PATH, &manifest)
            .expect("manifest should append to adversarial archive");

        for (path, bytes) in &bundle.artifacts {
            append_entry(&mut builder, path, bytes)
                .expect("artifact should append to adversarial archive");
        }

        let encoded = builder
            .into_inner()
            .expect("adversarial archive should finalize");

        PresolveBundle::decode(&encoded)
    }

    #[test]
    fn application_topology_is_stored_in_manifest() {
        let contract = topology_contract();

        let bundle = PresolveBundle::from_components(
            &contract,
            &[
                ComponentArtifactInput::new("api", API_COMPONENT),
                ComponentArtifactInput::new("worker", WORKER_COMPONENT),
            ],
        )
        .expect("bundle should build");

        let interfaces = bundle.manifest().interfaces();

        assert_eq!(interfaces.len(), 2);

        assert_eq!(interfaces[0].name(), "audit");
        assert_eq!(interfaces[0].kind(), InterfaceKind::Event);

        assert_eq!(interfaces[1].name(), "jobs");
        assert_eq!(interfaces[1].kind(), InterfaceKind::RequestResponse);

        let relationships = bundle.manifest().relationships();

        assert_eq!(relationships.len(), 2);

        assert_eq!(relationships[0].from(), "api");
        assert_eq!(relationships[0].to(), "worker");
        assert_eq!(relationships[0].interface(), "jobs");

        assert_eq!(relationships[1].from(), "worker");
        assert_eq!(relationships[1].to(), "api");
        assert_eq!(relationships[1].interface(), "audit");
    }

    #[test]
    fn duplicate_manifest_interface_is_rejected() {
        let mut bundle = topology_bundle();

        mutate_manifest(&mut bundle, |manifest| {
            let interfaces = manifest
                .get_mut("interfaces")
                .and_then(serde_json::Value::as_array_mut)
                .expect("interfaces should be an array");

            let duplicate = interfaces
                .first()
                .expect("topology should contain an interface")
                .clone();

            interfaces.push(duplicate);
        });

        let error = bundle
            .verify()
            .expect_err("duplicate interface should fail verification");

        assert!(matches!(
            error,
            BundleError::DuplicateManifestInterface(name) if name == "audit"
        ));
    }

    #[test]
    fn unknown_relationship_source_workload_is_rejected() {
        let mut bundle = topology_bundle();

        mutate_manifest(&mut bundle, |manifest| {
            let relationship = manifest
                .get_mut("relationships")
                .and_then(serde_json::Value::as_array_mut)
                .and_then(|relationships| relationships.first_mut())
                .expect("topology should contain a relationship");

            relationship["from"] = serde_json::json!("scheduler");
        });

        let error = bundle
            .verify()
            .expect_err("unknown source workload should fail verification");

        assert!(matches!(
            error,
            BundleError::UnknownRelationshipSourceWorkload(workload)
                if workload == "scheduler"
        ));
    }

    #[test]
    fn unknown_relationship_target_workload_is_rejected() {
        let mut bundle = topology_bundle();

        mutate_manifest(&mut bundle, |manifest| {
            let relationship = manifest
                .get_mut("relationships")
                .and_then(serde_json::Value::as_array_mut)
                .and_then(|relationships| relationships.first_mut())
                .expect("topology should contain a relationship");

            relationship["to"] = serde_json::json!("scheduler");
        });

        let error = bundle
            .verify()
            .expect_err("unknown target workload should fail verification");

        assert!(matches!(
            error,
            BundleError::UnknownRelationshipTargetWorkload(workload)
                if workload == "scheduler"
        ));
    }

    #[test]
    fn unknown_relationship_interface_is_rejected() {
        let mut bundle = topology_bundle();

        mutate_manifest(&mut bundle, |manifest| {
            let relationship = manifest
                .get_mut("relationships")
                .and_then(serde_json::Value::as_array_mut)
                .and_then(|relationships| relationships.first_mut())
                .expect("topology should contain a relationship");

            relationship["interface"] = serde_json::json!("missing");
        });

        let error = bundle
            .verify()
            .expect_err("unknown relationship interface should fail verification");

        assert!(matches!(
            error,
            BundleError::UnknownRelationshipInterface(interface)
                if interface == "missing"
        ));
    }

    #[test]
    fn self_relationship_is_rejected() {
        let mut bundle = topology_bundle();

        mutate_manifest(&mut bundle, |manifest| {
            let relationship = manifest
                .get_mut("relationships")
                .and_then(serde_json::Value::as_array_mut)
                .and_then(|relationships| relationships.first_mut())
                .expect("topology should contain a relationship");

            relationship["to"] = serde_json::json!("api");
        });

        let error = bundle
            .verify()
            .expect_err("self relationship should fail verification");

        assert!(matches!(
            error,
            BundleError::SelfRelationship(workload) if workload == "api"
        ));
    }

    #[test]
    fn duplicate_manifest_relationship_is_rejected() {
        let mut bundle = topology_bundle();

        mutate_manifest(&mut bundle, |manifest| {
            let relationships = manifest
                .get_mut("relationships")
                .and_then(serde_json::Value::as_array_mut)
                .expect("relationships should be an array");

            let duplicate = relationships
                .first()
                .expect("topology should contain a relationship")
                .clone();

            relationships.push(duplicate);
        });

        let error = bundle
            .verify()
            .expect_err("duplicate relationship should fail verification");

        assert!(matches!(
            error,
            BundleError::DuplicateManifestRelationship {
                from,
                to,
                interface
            } if from == "api" && to == "worker" && interface == "jobs"
        ));
    }

    #[test]
    fn topology_declaration_order_does_not_change_release_identity() {
        let first_contract = parse_contract(
            r#"
    contract_version = "0.1"

    [application]
    name = "topology-example"
    version = "1.0.0"

    [[workloads]]
    name = "api"
    kind = "component"

    [[workloads]]
    name = "worker"
    kind = "component"

    [[interfaces]]
    name = "jobs"
    kind = "request_response"

    [[interfaces]]
    name = "audit"
    kind = "event"

    [[relationships]]
    from = "api"
    to = "worker"
    interface = "jobs"

    [[relationships]]
    from = "worker"
    to = "api"
    interface = "audit"
    "#,
        )
        .expect("first contract should parse");

        let second_contract = parse_contract(
            r#"
    contract_version = "0.1"

    [application]
    name = "topology-example"
    version = "1.0.0"

    [[workloads]]
    name = "api"
    kind = "component"

    [[workloads]]
    name = "worker"
    kind = "component"

    [[interfaces]]
    name = "audit"
    kind = "event"

    [[interfaces]]
    name = "jobs"
    kind = "request_response"

    [[relationships]]
    from = "worker"
    to = "api"
    interface = "audit"

    [[relationships]]
    from = "api"
    to = "worker"
    interface = "jobs"
    "#,
        )
        .expect("second contract should parse");

        let components = [
            ComponentArtifactInput::new("api", API_COMPONENT),
            ComponentArtifactInput::new("worker", WORKER_COMPONENT),
        ];

        let first = PresolveBundle::from_components(&first_contract, &components)
            .expect("first bundle should build");

        let second = PresolveBundle::from_components(&second_contract, &components)
            .expect("second bundle should build");

        assert_eq!(
            first
                .release_digest()
                .expect("first release digest should compute"),
            second
                .release_digest()
                .expect("second release digest should compute")
        );

        assert_eq!(
            first.encode().expect("first bundle should encode"),
            second.encode().expect("second bundle should encode")
        );
    }

    #[test]
    fn topology_change_changes_release_identity() {
        let first_contract = parse_contract(
            r#"
    contract_version = "0.1"

    [application]
    name = "topology-example"
    version = "1.0.0"

    [[workloads]]
    name = "api"
    kind = "component"

    [[workloads]]
    name = "worker"
    kind = "component"

    [[interfaces]]
    name = "jobs"
    kind = "request_response"

    [[relationships]]
    from = "api"
    to = "worker"
    interface = "jobs"
    "#,
        )
        .expect("first contract should parse");

        let second_contract = parse_contract(
            r#"
    contract_version = "0.1"

    [application]
    name = "topology-example"
    version = "1.0.0"

    [[workloads]]
    name = "api"
    kind = "component"

    [[workloads]]
    name = "worker"
    kind = "component"

    [[interfaces]]
    name = "jobs"
    kind = "request_response"

    [[relationships]]
    from = "worker"
    to = "api"
    interface = "jobs"
    "#,
        )
        .expect("second contract should parse");

        let components = [
            ComponentArtifactInput::new("api", API_COMPONENT),
            ComponentArtifactInput::new("worker", WORKER_COMPONENT),
        ];

        let first = PresolveBundle::from_components(&first_contract, &components)
            .expect("first bundle should build");

        let second = PresolveBundle::from_components(&second_contract, &components)
            .expect("second bundle should build");

        assert_ne!(
            first
                .release_digest()
                .expect("first release digest should compute"),
            second
                .release_digest()
                .expect("second release digest should compute")
        );
    }

    #[test]
    fn component_bundle_round_trips() {
        let contract = parse_contract(
            r#"
contract_version = "0.1"

[application]
name = "example"
version = "1.0.0"

[[capabilities]]
interface = "presolve:kv/store"
version = "^0.1"

[resources]
memory_mib = 64

[network]
outbound = "deny"
"#,
        )
        .expect("contract should parse");

        let bundle =
            PresolveBundle::from_component(&contract, COMPONENT).expect("bundle should build");

        let encoded = bundle.encode().expect("bundle should encode");
        let decoded = PresolveBundle::decode(&encoded).expect("bundle should decode");

        assert_eq!(
            decoded
                .release_digest()
                .expect("release digest should compute"),
            bundle
                .release_digest()
                .expect("release digest should compute")
        );

        assert_eq!(decoded.manifest(), bundle.manifest());
    }

    #[test]
    fn semantic_capability_order_does_not_change_release_identity() {
        let first = parse_contract(
            r#"
contract_version = "0.1"

[application]
name = "example"
version = "1.0.0"

[[capabilities]]
interface = "presolve:objects/store"
version = "^1"

[[capabilities]]
interface = "presolve:kv/store"
version = "^0.1"
"#,
        )
        .expect("first contract should parse");

        let second = parse_contract(
            r#"
contract_version = "0.1"

[application]
name = "example"
version = "1.0.0"

[[capabilities]]
interface = "presolve:kv/store"
version = "^0.1"

[[capabilities]]
interface = "presolve:objects/store"
version = "^1"
"#,
        )
        .expect("second contract should parse");

        let first_bundle =
            PresolveBundle::from_component(&first, COMPONENT).expect("first bundle should build");

        let second_bundle =
            PresolveBundle::from_component(&second, COMPONENT).expect("second bundle should build");

        assert_eq!(
            first_bundle
                .release_digest()
                .expect("first digest should compute"),
            second_bundle
                .release_digest()
                .expect("second digest should compute")
        );

        assert_eq!(
            first_bundle.encode().expect("first bundle should encode"),
            second_bundle.encode().expect("second bundle should encode")
        );
    }

    #[test]
    fn multiple_component_workloads_package_successfully() {
        let contract = multi_workload_contract();

        let bundle = PresolveBundle::from_components(
            &contract,
            &[
                ComponentArtifactInput::new("api", API_COMPONENT),
                ComponentArtifactInput::new("worker", WORKER_COMPONENT),
            ],
        )
        .expect("multi-workload bundle should build");

        let workloads = bundle.manifest().workloads();

        assert_eq!(workloads.len(), 2);

        assert_eq!(workloads[0].name(), "api");
        assert_eq!(workloads[0].kind(), WorkloadKind::Component);
        assert_eq!(workloads[0].artifacts().len(), 1);
        assert_eq!(workloads[0].artifacts()[0].path(), "artifacts/api.wasm");

        assert_eq!(workloads[1].name(), "worker");
        assert_eq!(workloads[1].kind(), WorkloadKind::Component);
        assert_eq!(workloads[1].artifacts().len(), 1);
        assert_eq!(workloads[1].artifacts()[0].path(), "artifacts/worker.wasm");

        bundle.verify().expect("bundle should verify");

        let encoded = bundle.encode().expect("bundle should encode");

        let decoded = PresolveBundle::decode(&encoded).expect("bundle should decode");

        assert_eq!(decoded.manifest(), bundle.manifest());
    }

    #[test]
    fn contract_workload_order_does_not_change_release_identity() {
        let first_contract = parse_contract(
            r#"
    contract_version = "0.1"

    [application]
    name = "multi-example"
    version = "1.0.0"

    [[workloads]]
    name = "api"
    kind = "component"

    [[workloads]]
    name = "worker"
    kind = "component"
    "#,
        )
        .expect("first contract should parse");

        let second_contract = parse_contract(
            r#"
    contract_version = "0.1"

    [application]
    name = "multi-example"
    version = "1.0.0"

    [[workloads]]
    name = "worker"
    kind = "component"

    [[workloads]]
    name = "api"
    kind = "component"
    "#,
        )
        .expect("second contract should parse");

        let components = [
            ComponentArtifactInput::new("api", API_COMPONENT),
            ComponentArtifactInput::new("worker", WORKER_COMPONENT),
        ];

        let first = PresolveBundle::from_components(&first_contract, &components)
            .expect("first bundle should build");

        let second = PresolveBundle::from_components(&second_contract, &components)
            .expect("second bundle should build");

        assert_eq!(
            first
                .release_digest()
                .expect("first release digest should compute"),
            second
                .release_digest()
                .expect("second release digest should compute")
        );

        assert_eq!(
            first.encode().expect("first bundle should encode"),
            second.encode().expect("second bundle should encode")
        );
    }

    #[test]
    fn missing_workload_artifact_is_rejected() {
        let contract = multi_workload_contract();

        let error = PresolveBundle::from_components(
            &contract,
            &[ComponentArtifactInput::new("api", API_COMPONENT)],
        )
        .expect_err("missing worker artifact should fail");

        assert!(matches!(
            error,
            BundleError::MissingWorkloadArtifact(workload)
                if workload == "worker"
        ));
    }

    #[test]
    fn unexpected_workload_artifact_is_rejected() {
        let contract = multi_workload_contract();

        let error = PresolveBundle::from_components(
            &contract,
            &[
                ComponentArtifactInput::new("api", API_COMPONENT),
                ComponentArtifactInput::new("worker", WORKER_COMPONENT),
                ComponentArtifactInput::new("scheduler", b"scheduler"),
            ],
        )
        .expect_err("undeclared workload artifact should fail");

        assert!(matches!(
            error,
            BundleError::UnexpectedWorkloadArtifact(workload)
                if workload == "scheduler"
        ));
    }

    #[test]
    fn duplicate_workload_artifact_is_rejected() {
        let contract = multi_workload_contract();

        let error = PresolveBundle::from_components(
            &contract,
            &[
                ComponentArtifactInput::new("api", API_COMPONENT),
                ComponentArtifactInput::new("api", API_COMPONENT),
                ComponentArtifactInput::new("worker", WORKER_COMPONENT),
            ],
        )
        .expect_err("duplicate workload artifact should fail");

        assert!(matches!(
            error,
            BundleError::DuplicateWorkloadArtifact(workload)
                if workload == "api"
        ));
    }

    #[test]
    fn single_component_constructor_rejects_multi_workload_contract() {
        let contract = multi_workload_contract();

        let error = PresolveBundle::from_component(&contract, API_COMPONENT)
            .expect_err("single-component constructor should reject multiple workloads");

        assert!(matches!(
            error,
            BundleError::MultipleWorkloadsRequireExplicitArtifacts { count: 2 }
        ));
    }

    #[test]
    fn single_component_constructor_uses_declared_workload_name() {
        let contract = parse_contract(
            r#"
    contract_version = "0.1"

    [application]
    name = "example"
    version = "1.0.0"

    [[workloads]]
    name = "api"
    kind = "component"
    "#,
        )
        .expect("contract should parse");

        let bundle =
            PresolveBundle::from_component(&contract, API_COMPONENT).expect("bundle should build");

        let workloads = bundle.manifest().workloads();

        assert_eq!(workloads.len(), 1);
        assert_eq!(workloads[0].name(), "api");
        assert_eq!(workloads[0].artifacts()[0].path(), "artifacts/api.wasm");
    }

    #[test]
    fn component_input_order_does_not_change_release_identity() {
        let contract = multi_workload_contract();

        let first = PresolveBundle::from_components(
            &contract,
            &[
                ComponentArtifactInput::new("api", API_COMPONENT),
                ComponentArtifactInput::new("worker", WORKER_COMPONENT),
            ],
        )
        .expect("first bundle should build");

        let second = PresolveBundle::from_components(
            &contract,
            &[
                ComponentArtifactInput::new("worker", WORKER_COMPONENT),
                ComponentArtifactInput::new("api", API_COMPONENT),
            ],
        )
        .expect("second bundle should build");

        assert_eq!(
            first
                .release_digest()
                .expect("first release digest should compute"),
            second
                .release_digest()
                .expect("second release digest should compute")
        );

        assert_eq!(
            first.encode().expect("first bundle should encode"),
            second.encode().expect("second bundle should encode")
        );
    }

    #[test]
    fn artifact_tampering_is_detected() {
        let contract = parse_contract(
            r#"
contract_version = "0.1"

[application]
name = "example"
version = "1.0.0"
"#,
        )
        .expect("contract should parse");

        let mut bundle =
            PresolveBundle::from_component(&contract, COMPONENT).expect("bundle should build");

        bundle
            .artifacts
            .get_mut(COMPONENT_ARTIFACT_PATH)
            .expect("component should exist")[0] ^= 0xff;

        assert!(matches!(
            bundle.verify(),
            Err(BundleError::ArtifactDigestMismatch { .. })
        ));
    }

    #[test]
    fn artifact_content_changes_release_identity() {
        let contract = parse_contract(
            r#"
    contract_version = "0.1"

    [application]
    name = "example"
    version = "1.0.0"
    "#,
        )
        .expect("contract should parse");

        let first = PresolveBundle::from_component(&contract, b"first component")
            .expect("first bundle should build");

        let second = PresolveBundle::from_component(&contract, b"second component")
            .expect("second bundle should build");

        assert_ne!(
            first.release_digest().expect("first digest should compute"),
            second
                .release_digest()
                .expect("second digest should compute")
        );
    }

    #[test]
    fn decode_normalizes_manifest_collection_order() {
        let bundle = topology_bundle();

        let decoded = decode_mutated_manifest(&bundle, |manifest| {
            for key in ["workloads", "interfaces", "relationships"] {
                manifest
                    .get_mut(key)
                    .and_then(serde_json::Value::as_array_mut)
                    .expect("manifest collection should be an array")
                    .reverse();
            }
        })
        .expect("non-canonical collection order should normalize during decode");

        assert_eq!(decoded.manifest(), bundle.manifest());

        assert_eq!(
            decoded
                .release_digest()
                .expect("decoded release digest should compute"),
            bundle
                .release_digest()
                .expect("original release digest should compute")
        );
    }

    #[test]
    fn decode_rejects_unknown_manifest_field() {
        let bundle = topology_bundle();

        let error = decode_mutated_manifest(&bundle, |manifest| {
            manifest
                .as_object_mut()
                .expect("manifest should be an object")
                .insert("unexpected".to_owned(), serde_json::json!(true));
        })
        .expect_err("unknown manifest fields should fail decoding");

        assert!(matches!(error, BundleError::Manifest(_)));
    }

    #[test]
    fn decode_rejects_duplicate_manifest_workload() {
        let bundle = topology_bundle();

        let error = decode_mutated_manifest(&bundle, |manifest| {
            let workloads = manifest
                .get_mut("workloads")
                .and_then(serde_json::Value::as_array_mut)
                .expect("workloads should be an array");

            let duplicate = workloads
                .first()
                .expect("manifest should contain a workload")
                .clone();

            workloads.push(duplicate);
        })
        .expect_err("duplicate workload should fail decoding");

        assert!(matches!(
            error,
            BundleError::DuplicateManifestWorkload(workload) if workload == "api"
        ));
    }

    #[test]
    fn decode_rejects_duplicate_manifest_interface() {
        let bundle = topology_bundle();

        let error = decode_mutated_manifest(&bundle, |manifest| {
            let interfaces = manifest
                .get_mut("interfaces")
                .and_then(serde_json::Value::as_array_mut)
                .expect("interfaces should be an array");

            let duplicate = interfaces
                .first()
                .expect("manifest should contain an interface")
                .clone();

            interfaces.push(duplicate);
        })
        .expect_err("duplicate interface should fail decoding");

        assert!(matches!(
            error,
            BundleError::DuplicateManifestInterface(interface) if interface == "audit"
        ));
    }

    #[test]
    fn decode_rejects_unknown_relationship_source_workload() {
        let bundle = topology_bundle();

        let error = decode_mutated_manifest(&bundle, |manifest| {
            let relationship = manifest
                .get_mut("relationships")
                .and_then(serde_json::Value::as_array_mut)
                .and_then(|relationships| relationships.first_mut())
                .expect("manifest should contain a relationship");

            relationship["from"] = serde_json::json!("scheduler");
        })
        .expect_err("unknown relationship source should fail decoding");

        assert!(matches!(
            error,
            BundleError::UnknownRelationshipSourceWorkload(workload)
                if workload == "scheduler"
        ));
    }

    #[test]
    fn decode_rejects_unknown_relationship_target_workload() {
        let bundle = topology_bundle();

        let error = decode_mutated_manifest(&bundle, |manifest| {
            let relationship = manifest
                .get_mut("relationships")
                .and_then(serde_json::Value::as_array_mut)
                .and_then(|relationships| relationships.first_mut())
                .expect("manifest should contain a relationship");

            relationship["to"] = serde_json::json!("scheduler");
        })
        .expect_err("unknown relationship target should fail decoding");

        assert!(matches!(
            error,
            BundleError::UnknownRelationshipTargetWorkload(workload)
                if workload == "scheduler"
        ));
    }

    #[test]
    fn decode_rejects_unknown_relationship_interface() {
        let bundle = topology_bundle();

        let error = decode_mutated_manifest(&bundle, |manifest| {
            let relationship = manifest
                .get_mut("relationships")
                .and_then(serde_json::Value::as_array_mut)
                .and_then(|relationships| relationships.first_mut())
                .expect("manifest should contain a relationship");

            relationship["interface"] = serde_json::json!("missing");
        })
        .expect_err("unknown relationship interface should fail decoding");

        assert!(matches!(
            error,
            BundleError::UnknownRelationshipInterface(interface)
                if interface == "missing"
        ));
    }

    #[test]
    fn decode_rejects_self_relationship() {
        let bundle = topology_bundle();

        let error = decode_mutated_manifest(&bundle, |manifest| {
            let relationship = manifest
                .get_mut("relationships")
                .and_then(serde_json::Value::as_array_mut)
                .and_then(|relationships| relationships.first_mut())
                .expect("manifest should contain a relationship");

            relationship["to"] = serde_json::json!("api");
        })
        .expect_err("self relationship should fail decoding");

        assert!(matches!(
            error,
            BundleError::SelfRelationship(workload) if workload == "api"
        ));
    }

    #[test]
    fn decode_rejects_duplicate_manifest_relationship() {
        let bundle = topology_bundle();

        let error = decode_mutated_manifest(&bundle, |manifest| {
            let relationships = manifest
                .get_mut("relationships")
                .and_then(serde_json::Value::as_array_mut)
                .expect("relationships should be an array");

            let duplicate = relationships
                .first()
                .expect("manifest should contain a relationship")
                .clone();

            relationships.push(duplicate);
        })
        .expect_err("duplicate relationship should fail decoding");

        assert!(matches!(
            error,
            BundleError::DuplicateManifestRelationship {
                from,
                to,
                interface
            } if from == "api" && to == "worker" && interface == "jobs"
        ));
    }
    #[test]
    fn decode_normalizes_capability_feature_order() {
        let bundle = capability_feature_bundle();

        let decoded = decode_mutated_manifest(&bundle, |manifest| {
            let capability = manifest
                .get_mut("requirements")
                .and_then(|requirements| requirements.get_mut("capabilities"))
                .and_then(serde_json::Value::as_array_mut)
                .and_then(|capabilities| capabilities.first_mut())
                .expect("manifest should contain capability");

            capability["required_features"] =
                serde_json::json!(["range-read", "conditional-write"]);
        })
        .expect("feature ordering should normalize");

        assert_eq!(decoded.manifest(), bundle.manifest());
        assert_eq!(
            decoded
                .release_digest()
                .expect("decoded digest should compute"),
            bundle
                .release_digest()
                .expect("original digest should compute")
        );
    }

    #[test]
    fn decode_rejects_duplicate_manifest_capability() {
        let bundle = capability_feature_bundle();

        let error = decode_mutated_manifest(&bundle, |manifest| {
            let capabilities = manifest
                .get_mut("requirements")
                .and_then(|requirements| requirements.get_mut("capabilities"))
                .and_then(serde_json::Value::as_array_mut)
                .expect("manifest should contain capabilities");

            let duplicate = capabilities
                .first()
                .expect("manifest should contain capability")
                .clone();

            capabilities.push(duplicate);
        })
        .expect_err("duplicate capability should fail");

        assert!(matches!(
            error,
            BundleError::DuplicateManifestCapability(interface)
                if interface == "presolve:objects/store"
        ));
    }

    #[test]
    fn decode_rejects_duplicate_capability_feature() {
        let bundle = capability_feature_bundle();

        let error = decode_mutated_manifest(&bundle, |manifest| {
            let capability = manifest
                .get_mut("requirements")
                .and_then(|requirements| requirements.get_mut("capabilities"))
                .and_then(serde_json::Value::as_array_mut)
                .and_then(|capabilities| capabilities.first_mut())
                .expect("manifest should contain capability");

            capability["preferred_features"] = serde_json::json!(["range-read", "user-metadata"]);
        })
        .expect_err("required/preferred feature overlap should fail");

        assert!(matches!(
            error,
            BundleError::DuplicateManifestCapabilityFeature {
                interface,
                feature
            } if interface == "presolve:objects/store" && feature == "range-read"
        ));
    }

    #[test]
    fn decode_rejects_invalid_capability_feature() {
        let bundle = capability_feature_bundle();

        let error = decode_mutated_manifest(&bundle, |manifest| {
            let capability = manifest
                .get_mut("requirements")
                .and_then(|requirements| requirements.get_mut("capabilities"))
                .and_then(serde_json::Value::as_array_mut)
                .and_then(|capabilities| capabilities.first_mut())
                .expect("manifest should contain capability");

            capability["preferred_features"] = serde_json::json!(["Range Read"]);
        })
        .expect_err("invalid feature identifier should fail");

        assert!(matches!(
            error,
            BundleError::InvalidManifestCapabilityFeature {
                interface,
                feature
            } if interface == "presolve:objects/store" && feature == "Range Read"
        ));
    }

    #[test]
    fn decode_rejects_invalid_capability_version_requirement() {
        let bundle = capability_feature_bundle();

        let error = decode_mutated_manifest(&bundle, |manifest| {
            let capability = manifest
                .get_mut("requirements")
                .and_then(|requirements| requirements.get_mut("capabilities"))
                .and_then(serde_json::Value::as_array_mut)
                .and_then(|capabilities| capabilities.first_mut())
                .expect("manifest should contain capability");

            capability["version"] = serde_json::json!("not-semver");
        })
        .expect_err("invalid version requirement should fail");

        assert!(matches!(
            error,
            BundleError::InvalidManifestCapabilityVersion {
                interface,
                version
            } if interface == "presolve:objects/store" && version == "not-semver"
        ));
    }
}
