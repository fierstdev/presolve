#![forbid(unsafe_code)]

//! `Presolve` node orchestration.
//!
//! The node materializes symbolic deployment plans into concrete runtime
//! configuration using provider implementations available in the local
//! environment.

mod assembly;
mod registry;

pub use assembly::{AssemblyError, assemble_runtime};
pub use registry::{ProviderRegistry, ProviderRegistryError};
