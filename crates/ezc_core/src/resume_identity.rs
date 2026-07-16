//! Canonical Phase J resumability identity domains.
//!
//! These are compiler-only typed wrappers. They do not serialize a resume
//! product; J9 alone owns public manifest encoding.

use std::fmt;
use std::str::FromStr;

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::{
    ComponentInstanceId, ComponentRootId, ComponentStructuralRegionId, ComputedCacheSlotId,
    ComputedDirtyFlagId, FormInstanceId, IrStorageId, SemanticId,
};

macro_rules! resume_id {
    ($name:ident) => {
        #[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash, Serialize, Deserialize)]
        #[serde(transparent)]
        pub struct $name(String);

        impl $name {
            #[must_use]
            pub fn as_str(&self) -> &str {
                &self.0
            }
        }

        impl fmt::Display for $name {
            fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
                self.0.fmt(formatter)
            }
        }

        impl FromStr for $name {
            type Err = ResumeIdentityParseError;

            fn from_str(value: &str) -> Result<Self, Self::Err> {
                if value.is_empty() {
                    Err(ResumeIdentityParseError)
                } else {
                    Ok(Self(value.to_string()))
                }
            }
        }
    };
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ResumeIdentityParseError;

impl fmt::Display for ResumeIdentityParseError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("resume identity must not be empty")
    }
}

impl std::error::Error for ResumeIdentityParseError {}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub enum ResumeBoundaryKind {
    ApplicationRoot,
    ComponentInstance,
    StructuralRegion,
    FormInstance,
    Interaction,
}

impl ResumeBoundaryKind {
    const fn label(self) -> &'static str {
        match self {
            Self::ApplicationRoot => "ApplicationRoot",
            Self::ComponentInstance => "ComponentInstance",
            Self::StructuralRegion => "StructuralRegion",
            Self::FormInstance => "FormInstance",
            Self::Interaction => "Interaction",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub enum ResumeActivationRootKind {
    Eager,
    Event,
    Visible,
    Manual,
}

impl ResumeActivationRootKind {
    const fn label(self) -> &'static str {
        match self {
            Self::Eager => "Eager",
            Self::Event => "Event",
            Self::Visible => "Visible",
            Self::Manual => "Manual",
        }
    }
}

resume_id!(ResumeBoundaryId);
resume_id!(ResumeSlotId);
resume_id!(ResumeSchemaId);
resume_id!(ResumeCaptureProgramId);
resume_id!(ResumeRestoreProgramId);
resume_id!(ResumeChunkId);
resume_id!(ResumeChunkGroupId);
resume_id!(ResumeActivationId);
resume_id!(ResumeAnchorId);
resume_id!(ResumeEventId);
resume_id!(ResumeSnapshotId);
resume_id!(ResumeBuildId);
resume_id!(ResumeValueRecordId);

/// Exact ordinary-template target for one authored template entity materialized
/// by one compiler-planned component instance.
#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct TemplateInstanceTargetId {
    component_instance_id: ComponentInstanceId,
    template_entity_id: SemanticId,
}

impl TemplateInstanceTargetId {
    #[must_use]
    pub fn for_component_instance_template_entity(
        component_instance_id: ComponentInstanceId,
        template_entity_id: SemanticId,
    ) -> Self {
        Self {
            component_instance_id,
            template_entity_id,
        }
    }

    #[must_use]
    pub const fn component_instance_id(&self) -> &ComponentInstanceId {
        &self.component_instance_id
    }

    #[must_use]
    pub const fn template_entity_id(&self) -> &SemanticId {
        &self.template_entity_id
    }
}

impl fmt::Display for TemplateInstanceTargetId {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "{}/template-target:{}",
            self.component_instance_id,
            percent_encode(self.template_entity_id.as_str())
        )
    }
}

/// Exact ordinary-template binding execution for one component instance.
#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct TemplateInstanceBindingId {
    component_instance_id: ComponentInstanceId,
    binding_id: SemanticId,
}

impl TemplateInstanceBindingId {
    #[must_use]
    pub fn for_component_instance_binding(
        component_instance_id: ComponentInstanceId,
        binding_id: SemanticId,
    ) -> Self {
        Self {
            component_instance_id,
            binding_id,
        }
    }

    #[must_use]
    pub const fn component_instance_id(&self) -> &ComponentInstanceId {
        &self.component_instance_id
    }

    #[must_use]
    pub const fn binding_id(&self) -> &SemanticId {
        &self.binding_id
    }
}

impl fmt::Display for TemplateInstanceBindingId {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "{}/template-binding:{}",
            self.component_instance_id,
            percent_encode(self.binding_id.as_str())
        )
    }
}

/// Exact runtime cache address for one declaration-level computed cache in one
/// compiler-planned component instance.
#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct ComputedInstanceCacheSlotId {
    component_instance_id: ComponentInstanceId,
    cache_slot_id: ComputedCacheSlotId,
}

impl ComputedInstanceCacheSlotId {
    #[must_use]
    pub fn for_component_instance_cache_slot(
        component_instance_id: ComponentInstanceId,
        cache_slot_id: ComputedCacheSlotId,
    ) -> Self {
        Self {
            component_instance_id,
            cache_slot_id,
        }
    }

    #[must_use]
    pub const fn component_instance_id(&self) -> &ComponentInstanceId {
        &self.component_instance_id
    }

    #[must_use]
    pub const fn cache_slot_id(&self) -> &ComputedCacheSlotId {
        &self.cache_slot_id
    }
}

impl fmt::Display for ComputedInstanceCacheSlotId {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "{}/computed-cache:{}",
            self.component_instance_id,
            percent_encode(self.cache_slot_id.as_str())
        )
    }
}

/// Exact runtime dirty address for one declaration-level computed dirty flag in
/// one compiler-planned component instance.
#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct ComputedInstanceDirtySlotId {
    component_instance_id: ComponentInstanceId,
    dirty_flag_id: ComputedDirtyFlagId,
}

impl ComputedInstanceDirtySlotId {
    #[must_use]
    pub fn for_component_instance_dirty_flag(
        component_instance_id: ComponentInstanceId,
        dirty_flag_id: ComputedDirtyFlagId,
    ) -> Self {
        Self {
            component_instance_id,
            dirty_flag_id,
        }
    }

    #[must_use]
    pub const fn component_instance_id(&self) -> &ComponentInstanceId {
        &self.component_instance_id
    }

    #[must_use]
    pub const fn dirty_flag_id(&self) -> &ComputedDirtyFlagId {
        &self.dirty_flag_id
    }
}

impl fmt::Display for ComputedInstanceDirtySlotId {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "{}/computed-dirty:{}",
            self.component_instance_id,
            percent_encode(self.dirty_flag_id.as_str())
        )
    }
}

/// Exact runtime State address for one declaration-level IR storage in one
/// compiler-planned component instance.
#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct StateInstanceSlotId {
    component_instance_id: ComponentInstanceId,
    storage_id: IrStorageId,
}

impl StateInstanceSlotId {
    #[must_use]
    pub fn for_component_instance_storage(
        component_instance_id: ComponentInstanceId,
        storage_id: IrStorageId,
    ) -> Self {
        Self {
            component_instance_id,
            storage_id,
        }
    }

    #[must_use]
    pub const fn component_instance_id(&self) -> &ComponentInstanceId {
        &self.component_instance_id
    }

    #[must_use]
    pub const fn storage_id(&self) -> &IrStorageId {
        &self.storage_id
    }
}

impl fmt::Display for StateInstanceSlotId {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "{}/state-slot:{}",
            self.component_instance_id,
            percent_encode(self.storage_id.as_str())
        )
    }
}

impl ResumeBoundaryId {
    #[must_use]
    pub fn application_root(root: &ComponentRootId) -> Self {
        Self::new(
            root.as_str(),
            ResumeBoundaryKind::ApplicationRoot,
            root.as_str(),
        )
    }
    #[must_use]
    pub fn component_instance(instance: &ComponentInstanceId) -> Self {
        Self::new(
            instance.as_str(),
            ResumeBoundaryKind::ComponentInstance,
            instance.as_str(),
        )
    }
    #[must_use]
    pub fn structural_region(
        owner: &ComponentInstanceId,
        region: &ComponentStructuralRegionId,
    ) -> Self {
        Self::new(
            owner.as_str(),
            ResumeBoundaryKind::StructuralRegion,
            region.as_str(),
        )
    }
    #[must_use]
    pub fn form_instance(instance: &FormInstanceId) -> Self {
        Self::new(
            instance.as_str(),
            ResumeBoundaryKind::FormInstance,
            instance.as_str(),
        )
    }
    #[must_use]
    pub fn interaction(owner: &ComponentInstanceId, event: &SemanticId) -> Self {
        Self::new(
            owner.as_str(),
            ResumeBoundaryKind::Interaction,
            event.as_str(),
        )
    }
    fn new(owner: &str, kind: ResumeBoundaryKind, local: &str) -> Self {
        Self(format!("resume-boundary:{owner}:{}:{local}", kind.label()))
    }
}

impl ResumeSlotId {
    #[must_use]
    pub fn for_existing_storage(slot: &str) -> Self {
        Self(format!("resume-slot:{slot}"))
    }
}
impl ResumeSchemaId {
    #[must_use]
    pub fn for_boundary(boundary: &ResumeBoundaryId) -> Self {
        Self(format!("resume-schema:{boundary}"))
    }
}
impl ResumeCaptureProgramId {
    #[must_use]
    pub fn for_boundary(boundary: &ResumeBoundaryId) -> Self {
        Self(format!("resume-capture:{boundary}"))
    }
}
impl ResumeRestoreProgramId {
    #[must_use]
    pub fn for_boundary(boundary: &ResumeBoundaryId) -> Self {
        Self(format!("resume-restore:{boundary}"))
    }
}
impl ResumeActivationId {
    #[must_use]
    pub fn for_boundary(boundary: &ResumeBoundaryId) -> Self {
        Self(format!("resume-activation:{boundary}"))
    }
}
impl ResumeEventId {
    #[must_use]
    pub fn for_existing_event(event: &SemanticId) -> Self {
        Self(format!("resume-event:{event}"))
    }
}
impl ResumeSnapshotId {
    #[must_use]
    pub fn for_build(build: &ResumeBuildId) -> Self {
        Self(format!("resume-snapshot:{build}"))
    }
}
impl ResumeValueRecordId {
    #[must_use]
    pub fn for_slot(slot: &ResumeSlotId) -> Self {
        Self(format!("resume-value:{slot}"))
    }
}
impl ResumeChunkId {
    #[must_use]
    pub fn for_activation_root(kind: ResumeActivationRootKind, root: &str) -> Self {
        Self(format!("resume-chunk:{}:{root}", kind.label()))
    }
}
impl ResumeChunkGroupId {
    #[must_use]
    pub fn for_ordered_chunks(chunks: &[ResumeChunkId]) -> Self {
        Self(format!(
            "resume-chunk-group:{:016x}",
            canonical_hash(
                &chunks
                    .iter()
                    .map(ToString::to_string)
                    .collect::<Vec<_>>()
                    .join("\n")
            )
        ))
    }
}
impl ResumeAnchorId {
    #[must_use]
    pub fn for_target(boundary: &ResumeBoundaryId, kind: &str, target: &str) -> Self {
        Self(format!(
            "ez-r:{:016x}",
            canonical_hash(&format!("{boundary}\n{kind}\n{target}"))
        ))
    }
}
impl ResumeBuildId {
    pub const ZERO_SENTINEL: &'static str =
        "resume-build:0000000000000000000000000000000000000000000000000000000000000000";

    #[must_use]
    pub fn for_public_inputs(inputs: &str) -> Self {
        let digest = Sha256::digest(inputs.as_bytes());
        Self(format!("resume-build:{digest:x}"))
    }

    #[must_use]
    pub fn zero_sentinel() -> Self {
        Self(Self::ZERO_SENTINEL.to_string())
    }
}

fn canonical_hash(value: &str) -> u64 {
    value
        .as_bytes()
        .iter()
        .fold(0xcbf2_9ce4_8422_2325, |hash, byte| {
            (hash ^ u64::from(*byte)).wrapping_mul(0x0000_0100_0000_01b3)
        })
}
fn percent_encode(value: &str) -> String {
    value.bytes().fold(String::new(), |mut encoded, byte| {
        if byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'.' | b'_' | b'~') {
            encoded.push(char::from(byte));
        } else {
            use std::fmt::Write as _;
            write!(encoded, "%{byte:02X}").expect("writing to a String cannot fail");
        }
        encoded
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{ComponentRootId, FormId};

    #[test]
    fn j1_resume_identities_are_typed_deterministic_and_instance_qualified() {
        let component = SemanticId::component(Some("x-one"), "One");
        let root = ComponentRootId::for_component(&component);
        let first = ComponentInstanceId::for_root(&root);
        let second = ComponentInstanceId::for_invocation(
            &first,
            &crate::ComponentInvocationId::for_template_entity(&component, "child"),
        );
        let form = FormId::for_owner(&component, "profile");
        let first_form = FormInstanceId::for_component_instance(&first, &form);
        let second_form = FormInstanceId::for_component_instance(&second, &form);
        assert_ne!(
            ResumeBoundaryId::component_instance(&first),
            ResumeBoundaryId::component_instance(&second)
        );
        assert_ne!(
            ResumeBoundaryId::form_instance(&first_form),
            ResumeBoundaryId::form_instance(&second_form)
        );
        let template_entity = component.template().template_entity("element", "root");
        let binding = component.template().template_entity("binding", "root.0");
        assert_ne!(
            TemplateInstanceTargetId::for_component_instance_template_entity(
                first.clone(),
                template_entity.clone(),
            ),
            TemplateInstanceTargetId::for_component_instance_template_entity(
                second.clone(),
                template_entity,
            )
        );
        assert!(
            TemplateInstanceBindingId::for_component_instance_binding(first, binding)
                .to_string()
                .contains("/template-binding:")
        );
        let event = SemanticId::component(Some("x-one-event"), "Event");
        assert_eq!(
            ResumeEventId::for_existing_event(&event).to_string(),
            format!("resume-event:{event}")
        );
        let slot = ResumeSlotId::for_existing_storage("runtime-slot:one");
        assert_eq!(
            ResumeValueRecordId::for_slot(&slot).to_string(),
            "resume-value:resume-slot:runtime-slot:one"
        );
        let chunk =
            ResumeChunkId::for_activation_root(ResumeActivationRootKind::Event, event.as_str());
        assert_eq!(
            ResumeChunkGroupId::for_ordered_chunks(std::slice::from_ref(&chunk)),
            ResumeChunkGroupId::for_ordered_chunks(&[chunk])
        );
        assert!("".parse::<ResumeBuildId>().is_err());
    }

    #[test]
    fn j1a_state_slot_identity_preserves_the_exact_typed_pair() {
        let component = SemanticId::component(Some("x-state"), "State");
        let instance = ComponentInstanceId::for_root(&ComponentRootId::for_component(&component));
        let storage = IrStorageId::for_semantic_origin(&component.state_field("count/value"));
        let slot =
            StateInstanceSlotId::for_component_instance_storage(instance.clone(), storage.clone());
        assert_eq!(slot.component_instance_id(), &instance);
        assert_eq!(slot.storage_id(), &storage);
        assert_eq!(
            slot.to_string(),
            format!("{instance}/state-slot:storage%3Acomponent%3Ax-state%2Fstate%3Acount%2Fvalue")
        );
    }
}
