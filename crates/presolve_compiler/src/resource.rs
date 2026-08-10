use std::collections::BTreeSet;

use crate::{
    serialization_compatibility, ComponentInstanceId, ResourceActivationId,
    ResourceExecutionBoundary, ResourceId, SemanticId, SemanticType, SerializationCompatibility,
    SourceProvenance,
};
use crate::{SemanticPackageKind, SemanticPackageResourceEndpoint};

/// A source Resource declaration's compiler-owned attempt to select an
/// integrity-checked semantic-package endpoint.
///
/// This is intentionally not a ResourceDeclaration. It records package
/// resolution before activation, artifact, cancellation, and resume lowering
/// exist, so source syntax remains non-executable until the full N6 contract
/// can be emitted.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResourceEndpointResolution {
    pub owner_component: SemanticId,
    pub field: String,
    pub endpoint_designator: Option<String>,
    pub outcome: ResourceEndpointResolutionOutcome,
    pub provenance: SourceProvenance,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ResourceEndpointResolutionOutcome {
    MissingDesignator,
    UnboundDesignator {
        designator: String,
    },
    NonSemanticPackageBinding {
        designator: String,
    },
    NonResourceBinding {
        designator: String,
        kind: SemanticPackageKind,
    },
    UnsupportedExecutionBoundary {
        boundary: crate::SemanticPackageResourceExecutionBoundary,
    },
    Resolved(ResourceEndpointBinding),
}

/// The exact endpoint metadata selected by a resolved Resource source fact.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResourceEndpointBinding {
    pub local_name: String,
    pub package: String,
    pub version: String,
    pub integrity: String,
    pub export: String,
    pub type_signature: String,
    pub runtime_module: String,
    pub resume_policy: String,
    pub endpoint: SemanticPackageResourceEndpoint,
    pub route_loader: Option<crate::SemanticPackageRouteLoader>,
}

/// Compiler-owned declaration for a Resource operation.
///
/// This is intentionally a semantic product, not an async runtime. A later
/// N6 source/lowering slice must supply a registered endpoint or capability,
/// activation artifact, cancellation transport, and resume codec before a
/// Resource becomes executable.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResourceDeclaration {
    pub id: ResourceId,
    pub owner_component: SemanticId,
    pub name: String,
    pub key: String,
    pub data_type: SemanticType,
    pub error_type: SemanticType,
    pub execution_boundary: ResourceExecutionBoundary,
    pub input_dependencies: BTreeSet<SemanticId>,
    pub retry_policy: ResourceRetryPolicy,
    pub invalidation_policy: ResourceInvalidationPolicy,
    pub provenance: SourceProvenance,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ResourceRetryPolicy {
    Never,
    ExplicitOnly,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ResourceInvalidationPolicy {
    OnInputChange,
    ExplicitOnly,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ResourceLifecycleState {
    Idle,
    Pending { generation: u64 },
    Ready { generation: u64 },
    Failed { generation: u64 },
    Cancelled { generation: u64 },
}

impl ResourceLifecycleState {
    #[must_use]
    pub const fn generation(self) -> Option<u64> {
        match self {
            Self::Idle => None,
            Self::Pending { generation }
            | Self::Ready { generation }
            | Self::Failed { generation }
            | Self::Cancelled { generation } => Some(generation),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ResourceLifecycleEvent {
    Activate,
    Resolve,
    Reject,
    Cancel,
    Invalidate,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResourceActivation {
    pub id: ResourceActivationId,
    pub declaration: ResourceId,
    pub component_instance: ComponentInstanceId,
    pub state: ResourceLifecycleState,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ResourceDeclarationError {
    EmptyName,
    EmptyKey,
    NonSerializableData,
    NonSerializableError,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ResourceLifecycleError {
    InvalidTransition {
        state: ResourceLifecycleState,
        event: ResourceLifecycleEvent,
    },
}

impl ResourceDeclaration {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        owner_component: SemanticId,
        name: String,
        key: String,
        data_type: SemanticType,
        error_type: SemanticType,
        execution_boundary: ResourceExecutionBoundary,
        input_dependencies: BTreeSet<SemanticId>,
        retry_policy: ResourceRetryPolicy,
        invalidation_policy: ResourceInvalidationPolicy,
        provenance: SourceProvenance,
    ) -> Result<Self, ResourceDeclarationError> {
        if name.is_empty() {
            return Err(ResourceDeclarationError::EmptyName);
        }
        if key.is_empty() {
            return Err(ResourceDeclarationError::EmptyKey);
        }
        if serialization_compatibility(&data_type) != SerializationCompatibility::Serializable {
            return Err(ResourceDeclarationError::NonSerializableData);
        }
        if serialization_compatibility(&error_type) != SerializationCompatibility::Serializable {
            return Err(ResourceDeclarationError::NonSerializableError);
        }
        let id = ResourceId::for_owner(&owner_component, &name);
        Ok(Self {
            id,
            owner_component,
            name,
            key,
            data_type,
            error_type,
            execution_boundary,
            input_dependencies,
            retry_policy,
            invalidation_policy,
            provenance,
        })
    }

    #[must_use]
    pub fn activation_for(&self, component_instance: ComponentInstanceId) -> ResourceActivation {
        ResourceActivation {
            id: ResourceActivationId::for_component_instance(&component_instance, &self.id),
            declaration: self.id.clone(),
            component_instance,
            state: ResourceLifecycleState::Idle,
        }
    }
}

impl ResourceActivation {
    pub fn transition(
        &mut self,
        event: ResourceLifecycleEvent,
    ) -> Result<(), ResourceLifecycleError> {
        let next = match (self.state, event) {
            (ResourceLifecycleState::Idle, ResourceLifecycleEvent::Activate)
            | (ResourceLifecycleState::Ready { .. }, ResourceLifecycleEvent::Invalidate)
            | (ResourceLifecycleState::Failed { .. }, ResourceLifecycleEvent::Invalidate)
            | (ResourceLifecycleState::Cancelled { .. }, ResourceLifecycleEvent::Activate)
            | (ResourceLifecycleState::Cancelled { .. }, ResourceLifecycleEvent::Invalidate) => {
                ResourceLifecycleState::Pending {
                    generation: self.state.generation().unwrap_or(0) + 1,
                }
            }
            (ResourceLifecycleState::Pending { generation }, ResourceLifecycleEvent::Resolve) => {
                ResourceLifecycleState::Ready { generation }
            }
            (ResourceLifecycleState::Pending { generation }, ResourceLifecycleEvent::Reject) => {
                ResourceLifecycleState::Failed { generation }
            }
            (ResourceLifecycleState::Pending { generation }, ResourceLifecycleEvent::Cancel) => {
                ResourceLifecycleState::Cancelled { generation }
            }
            (state, event) => {
                return Err(ResourceLifecycleError::InvalidTransition { state, event })
            }
        };
        self.state = next;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;
    use std::path::Path;

    use presolve_parser::SourceSpan;

    use super::{
        ResourceDeclaration, ResourceDeclarationError, ResourceInvalidationPolicy,
        ResourceLifecycleError, ResourceLifecycleEvent, ResourceLifecycleState,
        ResourceRetryPolicy,
    };
    use crate::{
        ComponentInstanceId, ComponentRootId, ResourceExecutionBoundary, SemanticId, SemanticType,
        SourceProvenance,
    };

    fn declaration() -> ResourceDeclaration {
        ResourceDeclaration::new(
            SemanticId::component(Some("x-profile"), "Profile"),
            "profile".to_owned(),
            "profile-by-user".to_owned(),
            SemanticType::String,
            SemanticType::String,
            ResourceExecutionBoundary::Shared,
            BTreeSet::from([
                SemanticId::component(Some("x-profile"), "Profile").state_field("userId")
            ]),
            ResourceRetryPolicy::ExplicitOnly,
            ResourceInvalidationPolicy::OnInputChange,
            SourceProvenance::new(
                Path::new("src/Profile.tsx"),
                SourceSpan {
                    start: 0,
                    end: 1,
                    line: 1,
                    column: 1,
                },
            ),
        )
        .expect("valid resource declaration")
    }

    #[test]
    fn declaration_and_instance_activation_have_separate_stable_identities() {
        let declaration = declaration();
        let root =
            ComponentRootId::for_component(&SemanticId::component(Some("x-profile"), "Profile"));
        let activation = declaration.activation_for(ComponentInstanceId::for_root(&root));

        assert_eq!(
            declaration.id.as_str(),
            "component:x-profile/resource:profile"
        );
        assert_eq!(
            activation.id.as_str(),
            "root:component:x-profile/resource-activation:component:x-profile/resource:profile"
        );
        assert_eq!(activation.state, ResourceLifecycleState::Idle);
    }

    #[test]
    fn lifecycle_is_generation_scoped_and_rejects_impossible_transitions() {
        let declaration = declaration();
        let root =
            ComponentRootId::for_component(&SemanticId::component(Some("x-profile"), "Profile"));
        let mut activation = declaration.activation_for(ComponentInstanceId::for_root(&root));

        activation
            .transition(ResourceLifecycleEvent::Activate)
            .expect("activate");
        assert_eq!(
            activation.state,
            ResourceLifecycleState::Pending { generation: 1 }
        );
        activation
            .transition(ResourceLifecycleEvent::Resolve)
            .expect("resolve");
        assert_eq!(
            activation.state,
            ResourceLifecycleState::Ready { generation: 1 }
        );
        activation
            .transition(ResourceLifecycleEvent::Invalidate)
            .expect("invalidate");
        assert_eq!(
            activation.state,
            ResourceLifecycleState::Pending { generation: 2 }
        );
        activation
            .transition(ResourceLifecycleEvent::Cancel)
            .expect("cancel");
        assert_eq!(
            activation.state,
            ResourceLifecycleState::Cancelled { generation: 2 }
        );
        assert_eq!(
            activation.transition(ResourceLifecycleEvent::Resolve),
            Err(ResourceLifecycleError::InvalidTransition {
                state: ResourceLifecycleState::Cancelled { generation: 2 },
                event: ResourceLifecycleEvent::Resolve,
            })
        );
    }

    #[test]
    fn declaration_rejects_nonserializable_cross_boundary_data() {
        let result = ResourceDeclaration::new(
            SemanticId::component(Some("x-profile"), "Profile"),
            "profile".to_owned(),
            "profile-by-user".to_owned(),
            SemanticType::Unknown,
            SemanticType::String,
            ResourceExecutionBoundary::Shared,
            BTreeSet::new(),
            ResourceRetryPolicy::Never,
            ResourceInvalidationPolicy::ExplicitOnly,
            SourceProvenance::new(
                Path::new("src/Profile.tsx"),
                SourceSpan {
                    start: 0,
                    end: 1,
                    line: 1,
                    column: 1,
                },
            ),
        );

        assert_eq!(result, Err(ResourceDeclarationError::NonSerializableData));
    }
}
