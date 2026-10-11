#![forbid(unsafe_code)]

//! `Presolve` node orchestration.
//!
//! The node materializes symbolic deployment plans into concrete runtime
//! configuration using provider implementations available in the local
//! environment.

mod assembly;
mod local;
mod registry;

pub use assembly::{AssemblyError, assemble_runtime};
pub use local::{LocalRealizationError, realize_local_providers};
pub use registry::{ProviderRegistry, ProviderRegistryError};
