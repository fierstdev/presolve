//! Compiler-owned resolution products for opaque terminal Actions.

use crate::{SemanticId, SemanticPackageKind, SemanticPackageOpaqueTerminal, SourceProvenance};

/// A valid opaque Action declaration's attempt to select one exact package
/// terminal. This is still a semantic resolution product: no package bytes are
/// loaded and no Action runtime behavior is implied by it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OpaqueActionResolution {
    pub activation: SemanticId,
    pub owner_component: SemanticId,
    pub method: SemanticId,
    pub method_name: String,
    pub package_specifier: String,
    pub export_name: String,
    pub outcome: OpaqueActionResolutionOutcome,
    pub provenance: SourceProvenance,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OpaqueActionResolutionOutcome {
    UnboundImport {
        package: String,
        export: String,
    },
    NonOpaqueBinding {
        package: String,
        export: String,
        kind: SemanticPackageKind,
    },
    Resolved(OpaqueTerminalBinding),
}

/// The exact integrity-bound package export selected for an opaque terminal
/// Action. The package implementation remains opaque to the compiler.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OpaqueTerminalBinding {
    pub local_name: String,
    pub package: String,
    pub version: String,
    pub integrity: String,
    pub export: String,
    pub type_signature: String,
    pub runtime_module: String,
    pub resume_policy: String,
    pub terminal: SemanticPackageOpaqueTerminal,
}
