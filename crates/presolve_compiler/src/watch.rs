//! L8 explicit, caller-driven watch-session orchestration products.
//!
//! This module deliberately has no filesystem or timer APIs.  Its caller gives
//! it monotonic timestamps and invokes scheduler turns explicitly.
#![allow(
    clippy::missing_errors_doc,
    clippy::missing_panics_doc,
    clippy::needless_pass_by_value,
    clippy::too_many_lines
)]

use sha2::{Digest as _, Sha256};

pub const WATCH_SESSION_CONFIGURATION_V1_SCHEMA: &str = "presolve.watch-session-configuration";
pub const WATCH_CHANGE_BATCH_V1_SCHEMA: &str = "presolve.watch-change-batch";
pub const WATCH_EXECUTION_PLAN_V1_SCHEMA: &str = "presolve.watch-execution-plan";
pub const WATCH_EVENT_V1_SCHEMA: &str = "presolve.watch-event";
pub const WATCH_SESSION_SNAPSHOT_V1_SCHEMA: &str = "presolve.watch-session-snapshot";
pub const WATCH_EXECUTION_REPORT_V1_SCHEMA: &str = "presolve.watch-execution-report";
pub const MAX_WATCH_SESSIONS: usize = 64;
pub const MAX_WATCH_EVENTS: usize = 1024;
pub const MAX_OBSERVED_CHANGES: usize = 4096;
pub const MAX_LOGICAL_PATH_BYTES: usize = 4096;
pub const MAX_DEBOUNCE_MILLISECONDS: u64 = 86_400_000;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WatchDebounceV1 {
    pub quiet_period_milliseconds: u64,
    pub maximum_delay_milliseconds: u64,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WatchSessionConfigurationV1 {
    pub schema: String,
    pub version: u32,
    pub watch_session_id: String,
    pub workspace_id: String,
    pub debounce: WatchDebounceV1,
    pub supersession_policy: String,
    pub event_detail: String,
    pub event_journal_capacity: usize,
}
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct ObservedChangeV1 {
    pub kind: String,
    pub logical_path: String,
    pub previous_logical_path: Option<String>,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WatchCandidateMetadataV1 {
    pub candidate_id: String,
    pub fingerprint: String,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WatchChangeBatchMetadataV1 {
    pub schema: String,
    pub version: u32,
    pub watch_session_id: String,
    pub sequence: u64,
    pub observed_changes: Vec<ObservedChangeV1>,
    pub candidate: WatchCandidateMetadataV1,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WatchExecutionPlanV1 {
    pub schema: &'static str,
    pub version: u32,
    pub watch_session_id: String,
    pub workspace_id: String,
    pub first_sequence: u64,
    pub last_sequence: u64,
    pub coalesced_batch_count: u64,
    pub candidate_fingerprint: String,
    pub observed_changes: Vec<ObservedChangeV1>,
    pub eligibility: String,
    pub plan_identity: String,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WatchEventV1 {
    pub schema: &'static str,
    pub version: u32,
    pub watch_session_id: String,
    pub event_sequence: u64,
    pub kind: String,
    pub first_sequence: Option<u64>,
    pub last_sequence: Option<u64>,
    pub candidate_fingerprint: Option<String>,
    pub workspace_result_identity: Option<String>,
    pub code: Option<String>,
    pub event_identity: String,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WatchSessionSnapshotV1 {
    pub schema: &'static str,
    pub version: u32,
    pub watch_session_id: String,
    pub workspace_id: String,
    pub state: String,
    pub last_accepted_sequence: Option<u64>,
    pub active_candidate_fingerprint: Option<String>,
    pub pending_candidate_fingerprint: Option<String>,
    pub pending_sequence_range: Option<(u64, u64)>,
    pub last_successful_candidate_fingerprint: Option<String>,
    pub last_successful_workspace_result_identity: Option<String>,
    pub next_event_sequence: u64,
    pub retained_event_cursor_range: Option<(u64, u64)>,
    pub pending_count: usize,
    pub active_count: usize,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WatchExecutionReportV1 {
    pub plan: WatchExecutionPlanV1,
    pub outcome: String,
    pub workspace_result_identity: Option<String>,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WatchAcceptanceV1 {
    pub classification: &'static str,
    pub retained_pending: bool,
    pub obsolete_active: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WatchErrorV1 {
    Code(&'static str),
}
impl WatchErrorV1 {
    #[must_use]
    pub const fn code(&self) -> &'static str {
        match self {
            Self::Code(c) => c,
        }
    }
}

#[derive(Debug, Clone)]
struct Pending {
    first: u64,
    last: u64,
    batches: u64,
    window_start: u64,
    quiet_deadline: u64,
    maximum_deadline: u64,
    changes: Vec<ObservedChangeV1>,
    candidate: WatchCandidateMetadataV1,
}
#[derive(Debug, Clone)]
struct Active {
    plan: WatchExecutionPlanV1,
    obsolete: bool,
}
#[derive(Debug, Clone)]
pub struct WatchSessionV1 {
    configuration: WatchSessionConfigurationV1,
    last_sequence: Option<u64>,
    pending: Option<Pending>,
    active: Option<Active>,
    last_success: Option<(String, String, u64)>,
    events: Vec<WatchEventV1>,
    next_event: u64,
    open: bool,
}
fn hash(value: impl AsRef<[u8]>) -> String {
    format!("sha256:{:x}", Sha256::digest(value))
}
fn normalize_text(value: &str) -> Result<String, WatchErrorV1> {
    let value = value.trim();
    if value.is_empty() || value.bytes().any(|b| b.is_ascii_control()) {
        Err(WatchErrorV1::Code("L8W004_INVALID_WATCH_CONFIGURATION"))
    } else {
        Ok(value.into())
    }
}
fn normalize_path(value: &str) -> Result<String, WatchErrorV1> {
    if value.is_empty()
        || value.len() > MAX_LOGICAL_PATH_BYTES
        || value.starts_with('/')
        || value
            .split('/')
            .any(|p| p.is_empty() || p == "." || p == "..")
    {
        return Err(WatchErrorV1::Code("L8W007_INVALID_CHANGE_BATCH"));
    }
    Ok(value.into())
}
fn canonical_changes(changes: &mut Vec<ObservedChangeV1>) -> Result<(), WatchErrorV1> {
    if changes.len() > MAX_OBSERVED_CHANGES {
        return Err(WatchErrorV1::Code("L8W010_WATCH_RESOURCE_LIMIT_EXCEEDED"));
    }
    for change in changes.iter_mut() {
        if !matches!(
            change.kind.as_str(),
            "created" | "modified" | "deleted" | "renamed"
        ) {
            return Err(WatchErrorV1::Code("L8W007_INVALID_CHANGE_BATCH"));
        }
        change.logical_path = normalize_path(&change.logical_path)?;
        if change.kind == "renamed" {
            change.previous_logical_path = Some(normalize_path(
                change
                    .previous_logical_path
                    .as_deref()
                    .ok_or(WatchErrorV1::Code("L8W007_INVALID_CHANGE_BATCH"))?,
            )?);
        } else if change.previous_logical_path.is_some() {
            return Err(WatchErrorV1::Code("L8W007_INVALID_CHANGE_BATCH"));
        }
    }
    changes.sort();
    changes.dedup();
    Ok(())
}
impl WatchSessionConfigurationV1 {
    pub fn normalize_validate(&self) -> Result<Self, WatchErrorV1> {
        if self.schema != WATCH_SESSION_CONFIGURATION_V1_SCHEMA || self.version != 1 {
            return Err(WatchErrorV1::Code("L8W005_UNSUPPORTED_WATCH_SCHEMA"));
        }
        if self.debounce.maximum_delay_milliseconds < self.debounce.quiet_period_milliseconds
            || self.debounce.maximum_delay_milliseconds > MAX_DEBOUNCE_MILLISECONDS
            || self.supersession_policy != "cancel_obsolete"
            || !matches!(self.event_detail.as_str(), "summary" | "full")
        {
            return Err(WatchErrorV1::Code("L8W004_INVALID_WATCH_CONFIGURATION"));
        }
        if self.event_journal_capacity == 0 || self.event_journal_capacity > MAX_WATCH_EVENTS {
            return Err(WatchErrorV1::Code("L8W010_WATCH_RESOURCE_LIMIT_EXCEEDED"));
        }
        let mut out = self.clone();
        out.watch_session_id = normalize_text(&out.watch_session_id)?;
        out.workspace_id = normalize_text(&out.workspace_id)?;
        Ok(out)
    }
}
impl WatchSessionV1 {
    pub fn new(configuration: WatchSessionConfigurationV1) -> Result<Self, WatchErrorV1> {
        let mut out = Self {
            configuration: configuration.normalize_validate()?,
            last_sequence: None,
            pending: None,
            active: None,
            last_success: None,
            events: vec![],
            next_event: 1,
            open: true,
        };
        out.event("watch_started", None, None, None, None, None);
        Ok(out)
    }
    #[must_use]
    pub fn configuration(&self) -> &WatchSessionConfigurationV1 {
        &self.configuration
    }
    fn event(
        &mut self,
        kind: &str,
        first: Option<u64>,
        last: Option<u64>,
        fingerprint: Option<String>,
        result: Option<String>,
        code: Option<&str>,
    ) {
        let sequence = self.next_event;
        self.next_event += 1;
        let identity = hash(format!(
            "{}|{}|{}|{:?}|{:?}|{:?}|{:?}",
            self.configuration.watch_session_id, sequence, kind, first, last, fingerprint, result
        ));
        self.events.push(WatchEventV1 {
            schema: WATCH_EVENT_V1_SCHEMA,
            version: 1,
            watch_session_id: self.configuration.watch_session_id.clone(),
            event_sequence: sequence,
            kind: kind.into(),
            first_sequence: first,
            last_sequence: last,
            candidate_fingerprint: fingerprint,
            workspace_result_identity: result,
            code: code.map(Into::into),
            event_identity: identity,
        });
        if self.events.len() > self.configuration.event_journal_capacity {
            self.events.remove(0);
        }
    }
    pub fn accept(
        &mut self,
        mut batch: WatchChangeBatchMetadataV1,
        now: u64,
    ) -> Result<WatchAcceptanceV1, WatchErrorV1> {
        if !self.open {
            return Err(WatchErrorV1::Code("L8W009_WATCH_SESSION_CLOSED"));
        }
        if batch.schema != WATCH_CHANGE_BATCH_V1_SCHEMA || batch.version != 1 {
            return Err(WatchErrorV1::Code("L8W005_UNSUPPORTED_WATCH_SCHEMA"));
        }
        if batch.watch_session_id != self.configuration.watch_session_id || batch.sequence == 0 {
            return Err(WatchErrorV1::Code("L8W007_INVALID_CHANGE_BATCH"));
        }
        if self
            .last_sequence
            .is_some_and(|last| batch.sequence <= last)
        {
            self.event(
                "candidate_rejected",
                Some(batch.sequence),
                Some(batch.sequence),
                None,
                None,
                Some("L8W006_NON_MONOTONIC_CHANGE_SEQUENCE"),
            );
            return Err(WatchErrorV1::Code("L8W006_NON_MONOTONIC_CHANGE_SEQUENCE"));
        }
        batch.candidate.candidate_id = normalize_text(&batch.candidate.candidate_id)?;
        batch.candidate.fingerprint = normalize_text(&batch.candidate.fingerprint)?;
        canonical_changes(&mut batch.observed_changes)?;
        self.last_sequence = Some(batch.sequence);
        if self
            .last_success
            .as_ref()
            .is_some_and(|(f, _, _)| *f == batch.candidate.fingerprint)
        {
            self.event(
                "candidate_noop",
                Some(batch.sequence),
                Some(batch.sequence),
                Some(batch.candidate.fingerprint),
                None,
                Some("L8R005_EQUAL_TO_PUBLISHED_CANDIDATE"),
            );
            return Ok(WatchAcceptanceV1 {
                classification: "L8R005_EQUAL_TO_PUBLISHED_CANDIDATE",
                retained_pending: false,
                obsolete_active: false,
            });
        }
        if self
            .active
            .as_ref()
            .is_some_and(|a| a.plan.candidate_fingerprint == batch.candidate.fingerprint)
        {
            self.event(
                "candidate_noop",
                Some(batch.sequence),
                Some(batch.sequence),
                Some(batch.candidate.fingerprint),
                None,
                Some("L8R004_EQUAL_TO_ACTIVE_CANDIDATE"),
            );
            return Ok(WatchAcceptanceV1 {
                classification: "L8R004_EQUAL_TO_ACTIVE_CANDIDATE",
                retained_pending: false,
                obsolete_active: false,
            });
        }
        let (first, batches, window_start, mut changes) = match self.pending.take() {
            Some(p) => (p.first, p.batches + 1, p.window_start, p.changes),
            None => (batch.sequence, 1, now, vec![]),
        };
        changes.extend(batch.observed_changes);
        canonical_changes(&mut changes)?;
        let maximum =
            window_start.saturating_add(self.configuration.debounce.maximum_delay_milliseconds);
        let quiet = now.saturating_add(self.configuration.debounce.quiet_period_milliseconds);
        let replaced = first != batch.sequence;
        self.pending = Some(Pending {
            first,
            last: batch.sequence,
            batches,
            window_start,
            quiet_deadline: quiet,
            maximum_deadline: maximum,
            changes,
            candidate: batch.candidate.clone(),
        });
        let obsolete = if let Some(active) = self.active.as_mut() {
            active.obsolete = true;
            true
        } else {
            false
        };
        self.event(
            if replaced {
                "candidate_coalesced"
            } else {
                "candidate_accepted"
            },
            Some(first),
            Some(batch.sequence),
            Some(batch.candidate.fingerprint.clone()),
            None,
            Some(if obsolete {
                "L8R003_ACTIVE_CANDIDATE_SUPERSEDED"
            } else if replaced {
                "L8R002_PENDING_CANDIDATE_REPLACED"
            } else {
                "L8R001_FIRST_CANDIDATE"
            }),
        );
        if obsolete {
            self.event(
                "candidate_superseded",
                Some(first),
                Some(batch.sequence),
                Some(batch.candidate.fingerprint),
                None,
                Some("L8R003_ACTIVE_CANDIDATE_SUPERSEDED"),
            );
        }
        Ok(WatchAcceptanceV1 {
            classification: if obsolete {
                "L8R003_ACTIVE_CANDIDATE_SUPERSEDED"
            } else if replaced {
                "L8R002_PENDING_CANDIDATE_REPLACED"
            } else {
                "L8R001_FIRST_CANDIDATE"
            },
            retained_pending: true,
            obsolete_active: obsolete,
        })
    }
    pub fn scheduler_turn(&mut self, now: u64) -> Option<WatchExecutionPlanV1> {
        if !self.open || self.active.is_some() {
            return None;
        }
        let pending = self.pending.as_ref()?;
        if now < pending.quiet_deadline && now < pending.maximum_deadline {
            return None;
        }
        let pending = self.pending.take()?;
        let eligibility = if self.configuration.debounce.quiet_period_milliseconds == 0
            && self.configuration.debounce.maximum_delay_milliseconds == 0
        {
            "immediate_zero_debounce"
        } else if now >= pending.maximum_deadline
            && pending.maximum_deadline <= pending.quiet_deadline
        {
            "maximum_delay_elapsed"
        } else {
            "quiet_period_elapsed"
        };
        let plan_identity = hash(format!(
            "{}|{}|{}|{}|{}|{:?}|{}",
            self.configuration.watch_session_id,
            pending.first,
            pending.last,
            pending.batches,
            pending.candidate.fingerprint,
            pending.changes,
            eligibility
        ));
        let plan = WatchExecutionPlanV1 {
            schema: WATCH_EXECUTION_PLAN_V1_SCHEMA,
            version: 1,
            watch_session_id: self.configuration.watch_session_id.clone(),
            workspace_id: self.configuration.workspace_id.clone(),
            first_sequence: pending.first,
            last_sequence: pending.last,
            coalesced_batch_count: pending.batches,
            candidate_fingerprint: pending.candidate.fingerprint,
            observed_changes: pending.changes,
            eligibility: eligibility.into(),
            plan_identity,
        };
        self.event(
            "build_scheduled",
            Some(plan.first_sequence),
            Some(plan.last_sequence),
            Some(plan.candidate_fingerprint.clone()),
            None,
            Some(match eligibility {
                "maximum_delay_elapsed" => "L8R007_MAXIMUM_DELAY_ELAPSED",
                "immediate_zero_debounce" => "L8R012_ZERO_DEBOUNCE_COALESCED",
                _ => "L8R006_QUIET_PERIOD_ELAPSED",
            }),
        );
        self.event(
            "build_started",
            Some(plan.first_sequence),
            Some(plan.last_sequence),
            Some(plan.candidate_fingerprint.clone()),
            None,
            None,
        );
        self.active = Some(Active {
            plan: plan.clone(),
            obsolete: false,
        });
        Some(plan)
    }
    pub fn flush(&mut self) -> Result<(), WatchErrorV1> {
        if !self.open {
            return Err(WatchErrorV1::Code("L8W009_WATCH_SESSION_CLOSED"));
        }
        let Some(p) = self.pending.as_mut() else {
            return Err(WatchErrorV1::Code("L8W011_FLUSH_WITHOUT_PENDING_CANDIDATE"));
        };
        p.quiet_deadline = 0;
        p.maximum_deadline = 0;
        let (first, last, fingerprint) = (p.first, p.last, p.candidate.fingerprint.clone());
        self.event(
            "watch_flushed",
            Some(first),
            Some(last),
            Some(fingerprint),
            None,
            Some("L8R008_MANUAL_FLUSH"),
        );
        Ok(())
    }
    pub fn finish(
        &mut self,
        success: bool,
        workspace_result_identity: Option<String>,
    ) -> Option<WatchExecutionReportV1> {
        let active = self.active.take()?;
        let outcome = if active.obsolete {
            if success {
                self.event(
                    "obsolete_result_discarded",
                    Some(active.plan.first_sequence),
                    Some(active.plan.last_sequence),
                    Some(active.plan.candidate_fingerprint.clone()),
                    workspace_result_identity.clone(),
                    Some("L8R011_OBSOLETE_SUCCESS_DISCARDED"),
                );
                "obsolete_discarded"
            } else {
                self.event(
                    "build_superseded",
                    Some(active.plan.first_sequence),
                    Some(active.plan.last_sequence),
                    Some(active.plan.candidate_fingerprint.clone()),
                    None,
                    None,
                );
                "superseded"
            }
        } else if success {
            let id = workspace_result_identity.clone().unwrap_or_default();
            self.last_success = Some((
                active.plan.candidate_fingerprint.clone(),
                id,
                active.plan.last_sequence,
            ));
            self.event(
                "build_succeeded",
                Some(active.plan.first_sequence),
                Some(active.plan.last_sequence),
                Some(active.plan.candidate_fingerprint.clone()),
                workspace_result_identity.clone(),
                None,
            );
            "succeeded"
        } else {
            self.event(
                "build_failed",
                Some(active.plan.first_sequence),
                Some(active.plan.last_sequence),
                Some(active.plan.candidate_fingerprint.clone()),
                None,
                Some("L8R009_BUILD_FAILED_RETAINING_PRIOR_SUCCESS"),
            );
            "failed"
        };
        Some(WatchExecutionReportV1 {
            plan: active.plan,
            outcome: outcome.into(),
            workspace_result_identity,
        })
    }
    pub fn poll(&self, after: u64) -> Result<Vec<WatchEventV1>, WatchErrorV1> {
        if let Some(first) = self.events.first().map(|e| e.event_sequence) {
            if after.saturating_add(1) < first {
                return Err(WatchErrorV1::Code("L8W012_EVENT_CURSOR_OUT_OF_RANGE"));
            }
        }
        Ok(self
            .events
            .iter()
            .filter(|e| e.event_sequence > after)
            .cloned()
            .collect())
    }
    #[must_use]
    pub fn snapshot(&self) -> WatchSessionSnapshotV1 {
        let range = self
            .events
            .first()
            .zip(self.events.last())
            .map(|(a, b)| (a.event_sequence, b.event_sequence));
        WatchSessionSnapshotV1 {
            schema: WATCH_SESSION_SNAPSHOT_V1_SCHEMA,
            version: 1,
            watch_session_id: self.configuration.watch_session_id.clone(),
            workspace_id: self.configuration.workspace_id.clone(),
            state: if self.open { "open" } else { "closed" }.into(),
            last_accepted_sequence: self.last_sequence,
            active_candidate_fingerprint: self
                .active
                .as_ref()
                .map(|a| a.plan.candidate_fingerprint.clone()),
            pending_candidate_fingerprint: self
                .pending
                .as_ref()
                .map(|p| p.candidate.fingerprint.clone()),
            pending_sequence_range: self.pending.as_ref().map(|p| (p.first, p.last)),
            last_successful_candidate_fingerprint: self.last_success.as_ref().map(|x| x.0.clone()),
            last_successful_workspace_result_identity: self
                .last_success
                .as_ref()
                .map(|x| x.1.clone()),
            next_event_sequence: self.next_event,
            retained_event_cursor_range: range,
            pending_count: usize::from(self.pending.is_some()),
            active_count: usize::from(self.active.is_some()),
        }
    }
    pub fn stop(&mut self) {
        self.pending = None;
        self.open = false;
        self.event("watch_stopped", None, None, None, None, None);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn config() -> WatchSessionConfigurationV1 {
        WatchSessionConfigurationV1 {
            schema: WATCH_SESSION_CONFIGURATION_V1_SCHEMA.into(),
            version: 1,
            watch_session_id: "watch-a".into(),
            workspace_id: "workspace-a".into(),
            debounce: WatchDebounceV1 {
                quiet_period_milliseconds: 10,
                maximum_delay_milliseconds: 30,
            },
            supersession_policy: "cancel_obsolete".into(),
            event_detail: "summary".into(),
            event_journal_capacity: 8,
        }
    }
    fn batch(sequence: u64, fingerprint: &str) -> WatchChangeBatchMetadataV1 {
        WatchChangeBatchMetadataV1 {
            schema: WATCH_CHANGE_BATCH_V1_SCHEMA.into(),
            version: 1,
            watch_session_id: "watch-a".into(),
            sequence,
            observed_changes: vec![ObservedChangeV1 {
                kind: "modified".into(),
                logical_path: format!("src/{sequence}.ts"),
                previous_logical_path: None,
            }],
            candidate: WatchCandidateMetadataV1 {
                candidate_id: format!("c{sequence}"),
                fingerprint: fingerprint.into(),
            },
        }
    }
    #[test]
    fn fake_clock_debounce_coalesces_latest() {
        let mut s = WatchSessionV1::new(config()).unwrap();
        s.accept(batch(1, "one"), 0).unwrap();
        s.accept(batch(2, "two"), 5).unwrap();
        assert!(s.scheduler_turn(14).is_none());
        let p = s.scheduler_turn(15).unwrap();
        assert_eq!(
            (
                p.first_sequence,
                p.last_sequence,
                p.candidate_fingerprint.as_str()
            ),
            (1, 2, "two")
        );
    }
    #[test]
    fn maximum_delay_is_not_reset() {
        let mut s = WatchSessionV1::new(config()).unwrap();
        s.accept(batch(1, "one"), 0).unwrap();
        s.accept(batch(2, "two"), 25).unwrap();
        assert_eq!(
            s.scheduler_turn(30).unwrap().eligibility,
            "maximum_delay_elapsed"
        );
    }
    #[test]
    fn zero_debounce_waits_for_scheduler_turn() {
        let mut c = config();
        c.debounce = WatchDebounceV1 {
            quiet_period_milliseconds: 0,
            maximum_delay_milliseconds: 0,
        };
        let mut s = WatchSessionV1::new(c).unwrap();
        s.accept(batch(1, "one"), 0).unwrap();
        s.accept(batch(2, "two"), 0).unwrap();
        assert_eq!(s.scheduler_turn(0).unwrap().last_sequence, 2);
    }
    #[test]
    fn obsolete_success_is_discarded() {
        let mut s = WatchSessionV1::new(config()).unwrap();
        s.accept(batch(1, "one"), 0).unwrap();
        s.scheduler_turn(10).unwrap();
        assert!(s.accept(batch(2, "two"), 10).unwrap().obsolete_active);
        assert_eq!(
            s.finish(true, Some("old".into())).unwrap().outcome,
            "obsolete_discarded"
        );
    }
    #[test]
    fn equal_published_candidate_is_a_noop_and_sequences_remain_monotonic() {
        let mut s = WatchSessionV1::new(config()).unwrap();
        s.accept(batch(4, "same"), 0).unwrap();
        s.scheduler_turn(10).unwrap();
        s.finish(true, Some("result".into())).unwrap();
        assert_eq!(
            s.accept(batch(8, "same"), 11).unwrap().classification,
            "L8R005_EQUAL_TO_PUBLISHED_CANDIDATE"
        );
        assert_eq!(
            s.accept(batch(8, "later"), 12).unwrap_err().code(),
            "L8W006_NON_MONOTONIC_CHANGE_SEQUENCE"
        );
    }
    #[test]
    fn flush_and_event_eviction_are_source_free() {
        let mut c = config();
        c.event_journal_capacity = 3;
        let mut s = WatchSessionV1::new(c).unwrap();
        assert_eq!(
            s.flush().unwrap_err().code(),
            "L8W011_FLUSH_WITHOUT_PENDING_CANDIDATE"
        );
        s.accept(batch(1, "one"), 1).unwrap();
        s.flush().unwrap();
        s.scheduler_turn(1).unwrap();
        s.finish(false, None).unwrap();
        let events = s.poll(3).unwrap();
        assert!(events
            .iter()
            .all(|event| !format!("{event:?}").contains("source text")));
        assert_eq!(s.snapshot().retained_event_cursor_range, Some((4, 6)));
        assert_eq!(
            s.poll(0).unwrap_err().code(),
            "L8W012_EVENT_CURSOR_OUT_OF_RANGE"
        );
    }
    #[test]
    fn stop_releases_pending_and_rejects_future_batches() {
        let mut s = WatchSessionV1::new(config()).unwrap();
        s.accept(batch(1, "one"), 0).unwrap();
        s.stop();
        assert_eq!(s.snapshot().pending_count, 0);
        assert_eq!(
            s.accept(batch(2, "two"), 1).unwrap_err().code(),
            "L8W009_WATCH_SESSION_CLOSED"
        );
    }
    #[test]
    fn deterministic_twenty_fresh_runs() {
        let mut expected = None;
        for _ in 0..20 {
            let mut s = WatchSessionV1::new(config()).unwrap();
            s.accept(batch(1, "one"), 0).unwrap();
            s.accept(batch(2, "two"), 4).unwrap();
            let p = s.scheduler_turn(14).unwrap();
            let _r = s.finish(true, Some("result".into())).unwrap();
            let got = format!("{:?}{:?}", p, s.poll(0).unwrap());
            if let Some(old) = &expected {
                assert_eq!(old, &got);
            } else {
                expected = Some(got);
            }
        }
    }
}
