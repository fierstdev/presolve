#![forbid(unsafe_code)]

//! Docker CLI adapter for Presolve's backend-neutral container executor.
//!
//! This crate is an environment adapter, not part of application semantics.
//! It intentionally depends only on the executor SDK and the local `docker`
//! command-line interface.

mod backend;

pub use backend::DockerCliBackend;
