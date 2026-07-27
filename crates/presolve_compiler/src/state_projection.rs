//! Immutable V2 projection of existing instance-qualified State storage.

use serde::Serialize;

use crate::{resume_value_codec, StateInstanceStorageRegistry};

pub const STATE_PROJECTION_SCHEMA_VERSION: u32 = 1;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum StateResumeAdmissionV1 {
    CodecBacked,
    Rejected,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum StateUpdateCoverageV1 {
    ExistingLowering,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct StateProjectionRecordV1 {
    pub instance_slot: String,
    pub component_instance: String,
    pub state: String,
    pub storage: String,
    pub semantic_type: String,
    pub resume_admission: StateResumeAdmissionV1,
    pub update_coverage: StateUpdateCoverageV1,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct StateProjectionV1 {
    pub schema_version: u32,
    pub records: Vec<StateProjectionRecordV1>,
}

#[must_use]
pub fn build_state_projection_v1(registry: &StateInstanceStorageRegistry) -> StateProjectionV1 {
    let mut records = registry
        .records
        .iter()
        .map(|record| StateProjectionRecordV1 {
            instance_slot: record.slot_id.to_string(),
            component_instance: record.component_instance_id.as_str().to_owned(),
            state: record.state_id.as_str().to_owned(),
            storage: record.storage_id.as_str().to_owned(),
            semantic_type: crate::semantic_type_text(&record.semantic_type),
            resume_admission: if resume_value_codec(&record.semantic_type).is_ok() {
                StateResumeAdmissionV1::CodecBacked
            } else {
                StateResumeAdmissionV1::Rejected
            },
            update_coverage: StateUpdateCoverageV1::ExistingLowering,
        })
        .collect::<Vec<_>>();
    records.sort_by(|left, right| left.instance_slot.cmp(&right.instance_slot));
    StateProjectionV1 {
        schema_version: STATE_PROJECTION_SCHEMA_VERSION,
        records,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn preserves_instance_qualified_state_identity_and_codec_admission() {
        let model = crate::build_application_semantic_model(&presolve_parser::parse_file(
            "src/App.tsx",
            r#"@component("x-app") class App { count = state(0); render() { return <p>{this.count}</p>; } }"#,
        ));
        let registry = crate::build_state_instance_storage_registry(
            &model,
            &crate::lower_components_to_ir(&model),
        );
        let projection = build_state_projection_v1(&registry);
        assert_eq!(projection.schema_version, STATE_PROJECTION_SCHEMA_VERSION);
        assert_eq!(
            projection.records[0].resume_admission,
            StateResumeAdmissionV1::CodecBacked
        );
    }
}
