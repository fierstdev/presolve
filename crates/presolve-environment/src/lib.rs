#![forbid(unsafe_code)]

//! Declarative execution-environment semantics for Presolve.
//!
//! An Environment Specification describes what one execution environment can
//! offer to applications: stable environment identity, allocatable resources,
//! supported workload execution classes, and provider capability supply.
//!
//! It deliberately does not contain application-specific provider bindings,
//! provider credentials, SDK configuration, infrastructure secrets, or other
//! node realization details.

mod format;
mod model;
mod validate;
mod version;

pub use format::{
    EnvironmentEncodeError, EnvironmentParseError, encode_environment, parse_environment,
};
pub use model::{
    EnvironmentMetadata, EnvironmentSpecification, ExecutionSupport,
    ProvidedCapabilitySpecification, ProviderSpecification, ResourceCapacity,
};
pub use validate::EnvironmentSpecificationError;
pub use version::{
    ENVIRONMENT_SPECIFICATION_VERSION, EnvironmentSpecificationVersion,
    EnvironmentSpecificationVersionParseError,
};
