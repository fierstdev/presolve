#![forbid(unsafe_code)]

//! Workload execution interfaces implemented by `Presolve` execution backends.
//!
//! Workload semantics and immutable artifact identity remain independent from
//! the mechanism used to locate and execute those artifacts. This crate defines
//! the host-side boundary used by node materialization.

mod container;

pub use container::{
    ContainerCapabilityBinding, ContainerCapabilityToken, ContainerCapabilityTransport,
    ContainerExecutor, ContainerExecutorError, ContainerExecutorErrorCode, ContainerHandle,
    ContainerImageDescriptor, ContainerImageResolveError, ContainerImageResolveErrorCode,
    ContainerImageResolver, ContainerImageSource, ContainerLaunchRequest, ContainerNetworkMode,
    ContainerNetworkPolicy, ContainerResourceLimits,
};
