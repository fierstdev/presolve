//! K4 canonical generated-program fingerprinting and safe alias projection.

use std::collections::BTreeMap;

use crate::ExecutableProgramFingerprint;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ExecutableProgramCanonicalStream {
    pub semantic_program_id: String,
    pub bytes: Vec<u8>,
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ExecutableProgramCandidate {
    pub semantic_program_id: String,
    pub program_kind: String,
    pub opcode_stream: Vec<String>,
    pub required_protocol: String,
    pub instance_owned_identity: Option<String>,
    pub schedule_identity: Option<String>,
    pub native_default_policy: Option<String>,
    pub stable_failure_identity: Option<String>,
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ExecutableProgramFingerprintRegistry {
    pub streams: Vec<ExecutableProgramCanonicalStream>,
    pub fingerprints: Vec<ExecutableProgramFingerprint>,
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProgramAliasRecord {
    pub semantic_program_id: String,
    pub implementation_program_id: String,
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DeduplicatedProgramRegistry {
    pub implementations: Vec<String>,
    pub aliases: Vec<ProgramAliasRecord>,
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProgramDeduplicationReport {
    pub aliases: Vec<ProgramAliasRecord>,
    pub rejected_program_ids: Vec<String>,
}

#[must_use]
pub fn deduplicate_generated_programs(
    candidates: &[ExecutableProgramCandidate],
) -> (
    ExecutableProgramFingerprintRegistry,
    DeduplicatedProgramRegistry,
    ProgramDeduplicationReport,
) {
    deduplicate_with(candidates, |stream| {
        ExecutableProgramFingerprint::for_canonical_opcode_stream(stream)
    })
}

fn deduplicate_with<F>(
    candidates: &[ExecutableProgramCandidate],
    fingerprint: F,
) -> (
    ExecutableProgramFingerprintRegistry,
    DeduplicatedProgramRegistry,
    ProgramDeduplicationReport,
)
where
    F: Fn(&[u8]) -> ExecutableProgramFingerprint,
{
    let mut candidates = candidates.to_vec();
    candidates.sort_by(|a, b| a.semantic_program_id.cmp(&b.semantic_program_id));
    let streams = candidates.iter().map(canonical_stream).collect::<Vec<_>>();
    let fingerprints = streams
        .iter()
        .map(|stream| fingerprint(&stream.bytes))
        .collect::<Vec<_>>();
    let mut canonical = BTreeMap::<
        ExecutableProgramFingerprint,
        Vec<(ExecutableProgramCanonicalStream, bool)>,
    >::new();
    for (stream, candidate) in streams.iter().cloned().zip(candidates.iter()) {
        canonical
            .entry(fingerprint(&stream.bytes))
            .or_default()
            .push((stream, safe(candidate)));
    }
    let mut implementations = Vec::new();
    let mut aliases = Vec::new();
    let mut rejected = Vec::new();
    for entries in canonical.values() {
        let representative = &entries[0].0;
        implementations.push(representative.semantic_program_id.clone());
        for (stream, permitted) in entries.iter().skip(1) {
            if *permitted && stream.bytes == representative.bytes {
                aliases.push(ProgramAliasRecord {
                    semantic_program_id: stream.semantic_program_id.clone(),
                    implementation_program_id: representative.semantic_program_id.clone(),
                });
            } else {
                rejected.push(stream.semantic_program_id.clone());
                implementations.push(stream.semantic_program_id.clone());
            }
        }
    }
    implementations.sort();
    implementations.dedup();
    aliases.sort_by(|a, b| a.semantic_program_id.cmp(&b.semantic_program_id));
    rejected.sort();
    (
        ExecutableProgramFingerprintRegistry {
            streams,
            fingerprints,
        },
        DeduplicatedProgramRegistry {
            implementations,
            aliases: aliases.clone(),
        },
        ProgramDeduplicationReport {
            aliases,
            rejected_program_ids: rejected,
        },
    )
}

fn canonical_stream(candidate: &ExecutableProgramCandidate) -> ExecutableProgramCanonicalStream {
    let bytes = format!(
        "kind:{}\nprotocol:{}\nopcodes:{}\ninstance:{}\nschedule:{}\nnative-default:{}\nfailure:{}",
        candidate.program_kind,
        candidate.required_protocol,
        candidate.opcode_stream.join("\n"),
        candidate.instance_owned_identity.as_deref().unwrap_or(""),
        candidate.schedule_identity.as_deref().unwrap_or(""),
        candidate.native_default_policy.as_deref().unwrap_or(""),
        candidate.stable_failure_identity.as_deref().unwrap_or("")
    )
    .into_bytes();
    ExecutableProgramCanonicalStream {
        semantic_program_id: candidate.semantic_program_id.clone(),
        bytes,
    }
}
fn safe(candidate: &ExecutableProgramCandidate) -> bool {
    candidate.instance_owned_identity.is_none()
        && candidate.schedule_identity.is_none()
        && candidate.native_default_policy.is_none()
        && candidate.stable_failure_identity.is_none()
}

#[cfg(test)]
mod tests {
    use super::{deduplicate_generated_programs, deduplicate_with, ExecutableProgramCandidate};
    use crate::ExecutableProgramFingerprint;
    #[test]
    fn k4_deduplicates_only_identical_stateless_programs_and_rejects_boundaries() {
        let base = |id: &str| ExecutableProgramCandidate {
            semantic_program_id: id.to_string(),
            program_kind: "kernel".to_string(),
            opcode_stream: vec!["return".to_string()],
            required_protocol: "v1".to_string(),
            instance_owned_identity: None,
            schedule_identity: None,
            native_default_policy: None,
            stable_failure_identity: None,
        };
        let mut slot = base("slot");
        slot.instance_owned_identity = Some("slot:a".to_string());
        let mut event = base("event");
        event.native_default_policy = Some("prevent".to_string());
        let mut schedule = base("schedule");
        schedule.schedule_identity = Some("first".to_string());
        let (_, registry, report) =
            deduplicate_generated_programs(&[base("b"), base("a"), slot, event, schedule]);
        assert_eq!(registry.aliases[0].semantic_program_id, "b");
        assert_eq!(registry.aliases[0].implementation_program_id, "a");
        assert!(report.rejected_program_ids.is_empty());
    }
    #[test]
    fn k4_collision_hook_requires_byte_equality() {
        let candidate = |id: &str, opcode: &str| ExecutableProgramCandidate {
            semantic_program_id: id.to_string(),
            program_kind: "kernel".to_string(),
            opcode_stream: vec![opcode.to_string()],
            required_protocol: "v1".to_string(),
            instance_owned_identity: None,
            schedule_identity: None,
            native_default_policy: None,
            stable_failure_identity: None,
        };
        let (_, registry, report) =
            deduplicate_with(&[candidate("a", "one"), candidate("b", "two")], |_| {
                ExecutableProgramFingerprint::for_canonical_opcode_stream(b"collision")
            });
        assert!(registry.aliases.is_empty());
        assert_eq!(report.rejected_program_ids, vec!["b"]);
    }
}
