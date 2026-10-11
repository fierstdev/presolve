#![forbid(unsafe_code)]

//! `Presolve` node orchestration.
//!
//! The node materializes symbolic deployment plans into concrete runtime
//! configuration using provider implementations available in the local
//! environment.

mod assembly;
mod capability_bridge;
mod container;
mod local;
mod registry;

pub use assembly::{AssemblyError, assemble_runtime};
pub use capability_bridge::{
    ContainerCapabilityGateway, ContainerCapabilityGatewayError, start_container_capability_gateway,
};
pub use container::{
    ContainerMaterializationError, materialize_container_workloads,
    materialize_container_workloads_with_capability_transport,
};
pub use local::{LocalRealizationError, realize_local_providers};
pub use registry::{ProviderRegistry, ProviderRegistryError};
