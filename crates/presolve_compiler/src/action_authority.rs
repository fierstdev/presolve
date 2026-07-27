//! Adapter-fed V2 action authority.

use serde::Serialize;

pub const ACTION_AUTHORITY_SCHEMA_VERSION: u32 = 1;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ActionCaptureCoverageV1 {
    Complete,
    Unavailable,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ActionEnvironmentV1 {
    Browser,
    RejectedServerImport,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ActionFactV1 {
    pub id: String,
    pub component: String,
    pub asynchronous: bool,
    pub accepts_abort_signal: bool,
    pub capture_coverage: ActionCaptureCoverageV1,
    pub environment: ActionEnvironmentV1,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ActionAdmissionV1 {
    Admissible,
    RejectedCaptureCoverage,
    RejectedServerImport,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ActionRecordV1 {
    pub id: String,
    pub component: String,
    pub asynchronous: bool,
    pub accepts_abort_signal: bool,
    pub cancellation_is_explicit: bool,
    pub admission: ActionAdmissionV1,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ActionAuthorityV1 {
    pub schema_version: u32,
    pub actions: Vec<ActionRecordV1>,
}
#[must_use]
pub fn build_action_authority_v1(facts: &[ActionFactV1]) -> ActionAuthorityV1 {
    let mut actions = facts
        .iter()
        .map(|fact| ActionRecordV1 {
            id: fact.id.clone(),
            component: fact.component.clone(),
            asynchronous: fact.asynchronous,
            accepts_abort_signal: fact.accepts_abort_signal,
            cancellation_is_explicit: fact.accepts_abort_signal,
            admission: match (fact.environment, fact.capture_coverage) {
                (ActionEnvironmentV1::RejectedServerImport, _) => {
                    ActionAdmissionV1::RejectedServerImport
                }
                (_, ActionCaptureCoverageV1::Unavailable) => {
                    ActionAdmissionV1::RejectedCaptureCoverage
                }
                _ => ActionAdmissionV1::Admissible,
            },
        })
        .collect::<Vec<_>>();
    actions.sort_by(|a, b| a.id.cmp(&b.id));
    ActionAuthorityV1 {
        schema_version: ACTION_AUTHORITY_SCHEMA_VERSION,
        actions,
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn rejects_unproven_capture_and_server_imports() {
        let result = build_action_authority_v1(&[ActionFactV1 {
            id: "a".into(),
            component: "c".into(),
            asynchronous: true,
            accepts_abort_signal: true,
            capture_coverage: ActionCaptureCoverageV1::Unavailable,
            environment: ActionEnvironmentV1::Browser,
        }]);
        assert_eq!(
            result.actions[0].admission,
            ActionAdmissionV1::RejectedCaptureCoverage
        );
    }
}
