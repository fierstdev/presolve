//! Stable slot ownership and composition projection.

use serde::Serialize;

use crate::{SlotBindingRegistry, SlotBindingStatus};

pub const SLOT_PROJECTION_SCHEMA_VERSION: u32 = 1;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum SlotProjectionStatusV1 {
    Bound,
    Empty,
    Blocked,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum SlotResumabilityCoverageV1 {
    Unavailable,
}

/// One binding preserves the caller lexical owner and callee composition site.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct SlotProjectionRecordV1 {
    pub binding: String,
    pub caller_instance: String,
    pub callee_instance: String,
    pub lexical_content_owner_instance: String,
    pub slot: Option<String>,
    pub content_fragment: Option<String>,
    pub outlet: Option<String>,
    pub direct_child_instance: Option<String>,
    pub status: SlotProjectionStatusV1,
    pub resumability_coverage: SlotResumabilityCoverageV1,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct SlotProjectionGraphV1 {
    pub schema_version: u32,
    pub bindings: Vec<SlotProjectionRecordV1>,
}

/// Projects existing resolved composition facts without changing slot lowering.
#[must_use]
pub fn build_slot_projection_graph_v1(bindings: &SlotBindingRegistry) -> SlotProjectionGraphV1 {
    let mut records = bindings
        .bindings
        .values()
        .map(|binding| SlotProjectionRecordV1 {
            binding: binding.id.as_str().to_owned(),
            caller_instance: binding.caller_instance.as_str().to_owned(),
            callee_instance: binding.callee_instance.as_str().to_owned(),
            lexical_content_owner_instance: binding.content_owner_instance.as_str().to_owned(),
            slot: binding.slot.as_ref().map(|slot| slot.as_str().to_owned()),
            content_fragment: binding
                .content_fragment
                .as_ref()
                .map(|fragment| fragment.as_str().to_owned()),
            outlet: binding
                .outlet
                .as_ref()
                .map(|outlet| outlet.as_str().to_owned()),
            direct_child_instance: binding
                .direct_child_instance
                .as_ref()
                .map(|instance| instance.as_str().to_owned()),
            status: status(binding.status),
            resumability_coverage: SlotResumabilityCoverageV1::Unavailable,
        })
        .collect::<Vec<_>>();
    records.sort_by(|left, right| left.binding.cmp(&right.binding));
    SlotProjectionGraphV1 {
        schema_version: SLOT_PROJECTION_SCHEMA_VERSION,
        bindings: records,
    }
}

const fn status(status: SlotBindingStatus) -> SlotProjectionStatusV1 {
    match status {
        SlotBindingStatus::Bound => SlotProjectionStatusV1::Bound,
        SlotBindingStatus::Empty => SlotProjectionStatusV1::Empty,
        SlotBindingStatus::MissingOutlet
        | SlotBindingStatus::UnknownSlot
        | SlotBindingStatus::DuplicateContent
        | SlotBindingStatus::DuplicateOutlet
        | SlotBindingStatus::InvalidOwnership
        | SlotBindingStatus::BlockedInvocation => SlotProjectionStatusV1::Blocked,
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use crate::{
        ComponentInstanceId, ComponentInvocationId, ComponentRootId, SemanticId, SlotBinding,
        SlotBindingId, SlotBindingRegistry, SlotBindingStatus,
    };

    use super::*;

    #[test]
    fn preserves_lexical_content_ownership_and_explicitly_withholds_resume_claims() {
        let component = SemanticId::component(None, "Panel");
        let root = ComponentInstanceId::for_root(&ComponentRootId::for_component(&component));
        let invocation = ComponentInvocationId::for_template_entity(&component, "Panel");
        let child = ComponentInstanceId::for_invocation(&root, &invocation);
        let id = SlotBindingId::for_instance(&child, "children");
        let registry = SlotBindingRegistry {
            bindings: BTreeMap::from([(
                id.clone(),
                SlotBinding {
                    id,
                    caller_instance: root.clone(),
                    callee_instance: child.clone(),
                    invocation,
                    slot: None,
                    outlet: None,
                    content_fragment: None,
                    direct_child_instance: None,
                    content_owner_instance: root.clone(),
                    status: SlotBindingStatus::Bound,
                    provenance: crate::SourceProvenance::new(
                        "src/App.tsx",
                        presolve_parser::SourceSpan {
                            start: 0,
                            end: 1,
                            line: 1,
                            column: 1,
                        },
                    ),
                },
            )]),
        };
        let graph = build_slot_projection_graph_v1(&registry);
        assert_eq!(graph.schema_version, SLOT_PROJECTION_SCHEMA_VERSION);
        assert_eq!(
            graph.bindings[0].lexical_content_owner_instance,
            root.as_str()
        );
        assert_eq!(graph.bindings[0].callee_instance, child.as_str());
        assert_eq!(
            graph.bindings[0].resumability_coverage,
            SlotResumabilityCoverageV1::Unavailable
        );
    }
}
