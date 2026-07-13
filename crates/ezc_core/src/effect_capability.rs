use crate::{ExecutionBoundary, SemanticType};

/// The version of the compiler-owned effect capability registry schema.
pub const EFFECT_CAPABILITY_REGISTRY_VERSION: u32 = 1;

/// Stable compiler schema identity for one capability.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct CapabilityId(pub &'static str);

/// Stable compiler schema identity for one capability operation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct CapabilityOperationId(pub &'static str);

/// The externally authored static path that identifies one capability operation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StaticCapabilityPath(pub &'static str);

/// The supported top-level effect operation forms.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CapabilityOperationKind {
    MemberAssignment,
    MethodCall,
}

/// The canonical value shapes accepted by an operation signature.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CapabilityValueContract {
    String,
    SerializableDiagnosticValue,
}

impl CapabilityValueContract {
    #[must_use]
    pub const fn semantic_type(self) -> Option<SemanticType> {
        match self {
            Self::String => Some(SemanticType::String),
            Self::SerializableDiagnosticValue => None,
        }
    }
}

/// The fixed or variadic parameters accepted by one operation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CapabilityParameters {
    Fixed(&'static [CapabilityValueContract]),
    Variadic(CapabilityValueContract),
}

/// All Phase F v1 operations are terminal and return no value.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CapabilityResultPolicy {
    Void,
}

/// Whether operation arguments require structural serialization.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ArgumentSerializationPolicy {
    None,
    Structural,
}

/// Stable runtime-facing lowering identity. F4 declares it but does not lower it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RuntimeCapabilityLowering(pub &'static str);

/// One immutable signature owned by the compiler registry.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CapabilitySignature {
    pub parameters: CapabilityParameters,
    pub result: CapabilityResultPolicy,
}

/// One immutable operation exposed by a capability.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CapabilityOperation {
    pub id: CapabilityOperationId,
    pub static_path: StaticCapabilityPath,
    pub kind: CapabilityOperationKind,
    pub signature: CapabilitySignature,
    pub boundary: ExecutionBoundary,
    pub argument_serialization: ArgumentSerializationPolicy,
    pub runtime_lowering: RuntimeCapabilityLowering,
}

/// One immutable compiler-owned capability definition.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CapabilityDefinition {
    pub id: CapabilityId,
    pub root: &'static str,
    pub boundary: ExecutionBoundary,
    pub operations: &'static [CapabilityOperation],
    pub provenance: BuiltinCapabilityProvenance,
}

/// Provenance for definitions that are part of the compiler schema rather than source input.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BuiltinCapabilityProvenance {
    EffectCapabilityRegistryV1,
}

const STRING_PARAMETER: &[CapabilityValueContract] = &[CapabilityValueContract::String];
const STRING_PAIR_PARAMETERS: &[CapabilityValueContract] = &[
    CapabilityValueContract::String,
    CapabilityValueContract::String,
];

const DOCUMENT_OPERATIONS: &[CapabilityOperation] = &[CapabilityOperation {
    id: CapabilityOperationId("builtin.browser.document.title.assign"),
    static_path: StaticCapabilityPath("document.title"),
    kind: CapabilityOperationKind::MemberAssignment,
    signature: CapabilitySignature {
        parameters: CapabilityParameters::Fixed(STRING_PARAMETER),
        result: CapabilityResultPolicy::Void,
    },
    boundary: ExecutionBoundary::Client,
    argument_serialization: ArgumentSerializationPolicy::None,
    runtime_lowering: RuntimeCapabilityLowering("builtin.browser.document.title.assign"),
}];

const CONSOLE_OPERATIONS: &[CapabilityOperation] = &[
    CapabilityOperation {
        id: CapabilityOperationId("builtin.browser.console.log"),
        static_path: StaticCapabilityPath("console.log"),
        kind: CapabilityOperationKind::MethodCall,
        signature: CapabilitySignature {
            parameters: CapabilityParameters::Variadic(
                CapabilityValueContract::SerializableDiagnosticValue,
            ),
            result: CapabilityResultPolicy::Void,
        },
        boundary: ExecutionBoundary::Client,
        argument_serialization: ArgumentSerializationPolicy::Structural,
        runtime_lowering: RuntimeCapabilityLowering("builtin.browser.console.log"),
    },
    CapabilityOperation {
        id: CapabilityOperationId("builtin.browser.console.info"),
        static_path: StaticCapabilityPath("console.info"),
        kind: CapabilityOperationKind::MethodCall,
        signature: CapabilitySignature {
            parameters: CapabilityParameters::Variadic(
                CapabilityValueContract::SerializableDiagnosticValue,
            ),
            result: CapabilityResultPolicy::Void,
        },
        boundary: ExecutionBoundary::Client,
        argument_serialization: ArgumentSerializationPolicy::Structural,
        runtime_lowering: RuntimeCapabilityLowering("builtin.browser.console.info"),
    },
    CapabilityOperation {
        id: CapabilityOperationId("builtin.browser.console.warn"),
        static_path: StaticCapabilityPath("console.warn"),
        kind: CapabilityOperationKind::MethodCall,
        signature: CapabilitySignature {
            parameters: CapabilityParameters::Variadic(
                CapabilityValueContract::SerializableDiagnosticValue,
            ),
            result: CapabilityResultPolicy::Void,
        },
        boundary: ExecutionBoundary::Client,
        argument_serialization: ArgumentSerializationPolicy::Structural,
        runtime_lowering: RuntimeCapabilityLowering("builtin.browser.console.warn"),
    },
    CapabilityOperation {
        id: CapabilityOperationId("builtin.browser.console.error"),
        static_path: StaticCapabilityPath("console.error"),
        kind: CapabilityOperationKind::MethodCall,
        signature: CapabilitySignature {
            parameters: CapabilityParameters::Variadic(
                CapabilityValueContract::SerializableDiagnosticValue,
            ),
            result: CapabilityResultPolicy::Void,
        },
        boundary: ExecutionBoundary::Client,
        argument_serialization: ArgumentSerializationPolicy::Structural,
        runtime_lowering: RuntimeCapabilityLowering("builtin.browser.console.error"),
    },
];

const LOCAL_STORAGE_OPERATIONS: &[CapabilityOperation] = &[
    CapabilityOperation {
        id: CapabilityOperationId("builtin.browser.local_storage.set_item"),
        static_path: StaticCapabilityPath("localStorage.setItem"),
        kind: CapabilityOperationKind::MethodCall,
        signature: CapabilitySignature {
            parameters: CapabilityParameters::Fixed(STRING_PAIR_PARAMETERS),
            result: CapabilityResultPolicy::Void,
        },
        boundary: ExecutionBoundary::Client,
        argument_serialization: ArgumentSerializationPolicy::None,
        runtime_lowering: RuntimeCapabilityLowering("builtin.browser.local_storage.set_item"),
    },
    CapabilityOperation {
        id: CapabilityOperationId("builtin.browser.local_storage.remove_item"),
        static_path: StaticCapabilityPath("localStorage.removeItem"),
        kind: CapabilityOperationKind::MethodCall,
        signature: CapabilitySignature {
            parameters: CapabilityParameters::Fixed(STRING_PARAMETER),
            result: CapabilityResultPolicy::Void,
        },
        boundary: ExecutionBoundary::Client,
        argument_serialization: ArgumentSerializationPolicy::None,
        runtime_lowering: RuntimeCapabilityLowering("builtin.browser.local_storage.remove_item"),
    },
];

const SESSION_STORAGE_OPERATIONS: &[CapabilityOperation] = &[
    CapabilityOperation {
        id: CapabilityOperationId("builtin.browser.session_storage.set_item"),
        static_path: StaticCapabilityPath("sessionStorage.setItem"),
        kind: CapabilityOperationKind::MethodCall,
        signature: CapabilitySignature {
            parameters: CapabilityParameters::Fixed(STRING_PAIR_PARAMETERS),
            result: CapabilityResultPolicy::Void,
        },
        boundary: ExecutionBoundary::Client,
        argument_serialization: ArgumentSerializationPolicy::None,
        runtime_lowering: RuntimeCapabilityLowering("builtin.browser.session_storage.set_item"),
    },
    CapabilityOperation {
        id: CapabilityOperationId("builtin.browser.session_storage.remove_item"),
        static_path: StaticCapabilityPath("sessionStorage.removeItem"),
        kind: CapabilityOperationKind::MethodCall,
        signature: CapabilitySignature {
            parameters: CapabilityParameters::Fixed(STRING_PARAMETER),
            result: CapabilityResultPolicy::Void,
        },
        boundary: ExecutionBoundary::Client,
        argument_serialization: ArgumentSerializationPolicy::None,
        runtime_lowering: RuntimeCapabilityLowering("builtin.browser.session_storage.remove_item"),
    },
];

const DEFINITIONS: &[CapabilityDefinition] = &[
    CapabilityDefinition {
        id: CapabilityId("builtin.browser.document"),
        root: "document",
        boundary: ExecutionBoundary::Client,
        operations: DOCUMENT_OPERATIONS,
        provenance: BuiltinCapabilityProvenance::EffectCapabilityRegistryV1,
    },
    CapabilityDefinition {
        id: CapabilityId("builtin.browser.console"),
        root: "console",
        boundary: ExecutionBoundary::Client,
        operations: CONSOLE_OPERATIONS,
        provenance: BuiltinCapabilityProvenance::EffectCapabilityRegistryV1,
    },
    CapabilityDefinition {
        id: CapabilityId("builtin.browser.local_storage"),
        root: "localStorage",
        boundary: ExecutionBoundary::Client,
        operations: LOCAL_STORAGE_OPERATIONS,
        provenance: BuiltinCapabilityProvenance::EffectCapabilityRegistryV1,
    },
    CapabilityDefinition {
        id: CapabilityId("builtin.browser.session_storage"),
        root: "sessionStorage",
        boundary: ExecutionBoundary::Client,
        operations: SESSION_STORAGE_OPERATIONS,
        provenance: BuiltinCapabilityProvenance::EffectCapabilityRegistryV1,
    },
];

/// The immutable, deterministic built-in registry for Phase F effects.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EffectCapabilityRegistry;

impl EffectCapabilityRegistry {
    #[must_use]
    pub const fn version(self) -> u32 {
        EFFECT_CAPABILITY_REGISTRY_VERSION
    }

    #[must_use]
    pub const fn definitions(self) -> &'static [CapabilityDefinition] {
        DEFINITIONS
    }

    #[must_use]
    pub fn operation_at(
        self,
        path: &str,
        kind: CapabilityOperationKind,
    ) -> Option<&'static CapabilityOperation> {
        self.definitions()
            .iter()
            .flat_map(|definition| definition.operations)
            .find(|operation| operation.kind == kind && operation.static_path.0 == path)
    }
}

/// The one authoritative registry instance used by all compiler consumers.
pub const EFFECT_CAPABILITY_REGISTRY: EffectCapabilityRegistry = EffectCapabilityRegistry;

#[cfg(test)]
mod tests {
    use super::{
        CapabilityOperationKind, EFFECT_CAPABILITY_REGISTRY, EFFECT_CAPABILITY_REGISTRY_VERSION,
    };

    #[test]
    fn registry_v1_exposes_only_the_initial_exact_static_operations() {
        let registry = EFFECT_CAPABILITY_REGISTRY;
        assert_eq!(registry.version(), EFFECT_CAPABILITY_REGISTRY_VERSION);
        assert_eq!(registry.definitions().len(), 4);
        assert_eq!(
            registry
                .operation_at("document.title", CapabilityOperationKind::MemberAssignment)
                .expect("document title operation")
                .id
                .0,
            "builtin.browser.document.title.assign"
        );
        assert!(registry
            .operation_at("window.console.log", CapabilityOperationKind::MethodCall)
            .is_none());
        assert!(registry
            .operation_at("analytics.track", CapabilityOperationKind::MethodCall)
            .is_none());
    }
}
