use std::{
    collections::{BTreeMap, BTreeSet},
    io::{Cursor, Read},
};

use presolve_contract::ApplicationContract;
use presolve_core::ArtifactDigest;
use tar::{Archive, Builder, EntryType, Header};
use thiserror::Error;

use crate::{
    BUNDLE_VERSION, BundleManifest, BundleVersion, COMPONENT_ARTIFACT_PATH, MANIFEST_PATH,
};

/// Complete immutable `.presolve` application bundle.
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
    pub fn from_component(
        contract: &ApplicationContract,
        component: &[u8],
    ) -> Result<Self, BundleError> {
        let manifest = BundleManifest::for_component(contract, component)?;

        let artifacts = BTreeMap::from([(COMPONENT_ARTIFACT_PATH.to_owned(), component.to_vec())]);

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

    /// Validates all manifest-to-artifact integrity relationships.
    ///
    /// # Errors
    ///
    /// Returns an error when the bundle version is unsupported, an artifact is
    /// missing or unexpected, or artifact size/digest metadata does not match
    /// the bundled bytes.
    pub fn verify(&self) -> Result<(), BundleError> {
        if self.manifest.bundle_version() != BUNDLE_VERSION {
            return Err(BundleError::UnsupportedBundleVersion {
                found: self.manifest.bundle_version(),
            });
        }

        if self.manifest.workloads().is_empty() {
            return Err(BundleError::InvalidManifest(
                "bundle must contain at least one workload",
            ));
        }

        let mut referenced = BTreeSet::new();

        for workload in self.manifest.workloads() {
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

        for path in self.artifacts.keys() {
            if !referenced.contains(path) {
                return Err(BundleError::UnexpectedArtifact(path.clone()));
            }
        }

        Ok(())
    }

    /// Encodes the bundle as a deterministic `.presolve` archive.
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

    /// Decodes and verifies a `.presolve` archive.
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

        let manifest = serde_json::from_slice(&manifest_bytes).map_err(BundleError::Manifest)?;

        let bundle = Self {
            manifest,
            artifacts: entries,
        };

        bundle.verify()?;

        Ok(bundle)
    }
}

/// Failure while creating, decoding, or validating a `.presolve` bundle.
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
    use presolve_contract::parse_contract;

    use super::*;

    const COMPONENT: &[u8] = b"example component bytes";

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
}
