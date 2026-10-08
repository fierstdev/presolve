#![forbid(unsafe_code)]

//! Canonical immutable application bundles for `Presolve`.
//!
//! A `.presolve` bundle captures application semantics and immutable workload
//! artifacts without embedding environment-specific deployment decisions.

mod bundle;
mod input;
mod manifest;
mod version;

pub use bundle::{BundleError, PresolveBundle};
pub use input::ComponentArtifactInput;
pub use manifest::{
    ApplicationManifest, ArtifactManifest, BundleManifest, COMPONENT_ARTIFACT_PATH,
    CapabilityManifest, MANIFEST_PATH, NetworkManifest, RequirementsManifest, ResourceManifest,
    WASM_MEDIA_TYPE, WorkloadManifest, component_artifact_path,
};
pub use presolve_core::WorkloadKind;
pub use version::{BUNDLE_VERSION, BundleVersion, BundleVersionParseError};
