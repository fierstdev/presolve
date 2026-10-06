#![forbid(unsafe_code)]

//! Environment capability and resource resolution for `EdgeZero`.
//!
//! The resolver compares an application contract with an environment
//! inventory and produces either a deterministic deployment plan or structured
//! reasons the application cannot run in that environment.

mod environment;
mod plan;
mod problem;
mod report;
mod resolve;

pub use environment::{
    EnvironmentInventory, EnvironmentResources, ProvidedCapability, ProviderDescriptor,
};
pub use plan::{CapabilityBinding, DeploymentPlan};
pub use problem::ResolutionProblem;
pub use report::ResolutionReport;
pub use resolve::resolve;
