//! Canonical Phase J resumability identity domains.
//!
//! These are compiler-only typed wrappers. They do not serialize a resume
//! product; J9 alone owns public manifest encoding.

use std::fmt;
use std::str::FromStr;

use crate::{
    ComponentInstanceId, ComponentRootId, ComponentStructuralRegionId, FormInstanceId, SemanticId,
};

macro_rules! resume_id {
    ($name:ident) => {
        #[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
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
    #[must_use]
    pub fn for_public_inputs(inputs: &str) -> Self {
        Self(format!("resume-build:{:016x}", canonical_hash(inputs)))
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
}
