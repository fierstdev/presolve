//! K6 deterministic shared lazy-chunk candidate planning only.

use crate::{ExecutableProgramFingerprint, ProductionOptimizationPolicyV1, SharedChunkCandidateId};
use std::collections::BTreeMap;

#[derive(Clone, Debug, Eq, PartialEq)]
#[allow(clippy::struct_excessive_bools)]
pub struct SharedChunkProgramOccurrence {
    pub root_id: String,
    pub fingerprint: ExecutableProgramFingerprint,
    pub canonical_bytes: Vec<u8>,
    pub eager_required: bool,
    pub root_specific: bool,
    pub captures_mutable_identity: bool,
    pub registration_only: bool,
    pub runtime_protocol: String,
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SharedChunkConsumerRoot {
    pub root_id: String,
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SharedChunkSavingsCalculation {
    pub canonical_bytes: usize,
    pub root_count: usize,
    pub wrapper_bytes: usize,
    pub import_bytes_per_root: usize,
    pub net_saved_bytes: i128,
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum SharedChunkRejectionReason {
    TooFewRoots,
    EagerRequired,
    RootSpecific,
    MutableIdentity,
    ExecutesOnRegistration,
    ProtocolMismatch,
    BelowCanonicalBytes,
    BelowNetSavings,
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SharedChunkCandidate {
    pub id: SharedChunkCandidateId,
    pub programs: Vec<ExecutableProgramFingerprint>,
    pub consumers: Vec<SharedChunkConsumerRoot>,
    pub savings: SharedChunkSavingsCalculation,
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SharedChunkCandidatePlan {
    pub candidates: Vec<SharedChunkCandidate>,
    pub rejections: Vec<(ExecutableProgramFingerprint, SharedChunkRejectionReason)>,
}

#[must_use]
///
/// # Panics
///
/// Panics only if internally validated canonical roots or fingerprints are invalid.
pub fn plan_shared_lazy_chunk_candidates(
    occurrences: &[SharedChunkProgramOccurrence],
    wrapper_bytes: usize,
    import_bytes_per_root: usize,
) -> SharedChunkCandidatePlan {
    let mut by_program =
        BTreeMap::<ExecutableProgramFingerprint, Vec<&SharedChunkProgramOccurrence>>::new();
    for occurrence in occurrences {
        by_program
            .entry(occurrence.fingerprint.clone())
            .or_default()
            .push(occurrence);
    }
    let mut groups = BTreeMap::<Vec<String>, Vec<(ExecutableProgramFingerprint, usize)>>::new();
    let mut rejections = Vec::new();
    for (fingerprint, entries) in by_program {
        let mut roots = entries
            .iter()
            .map(|entry| entry.root_id.clone())
            .collect::<Vec<_>>();
        roots.sort();
        roots.dedup();
        let reason = if roots.len() < ProductionOptimizationPolicyV1::SHARED_CHUNK_MIN_ROOT_COUNT {
            Some(SharedChunkRejectionReason::TooFewRoots)
        } else if entries.iter().any(|entry| entry.eager_required) {
            Some(SharedChunkRejectionReason::EagerRequired)
        } else if entries.iter().any(|entry| entry.root_specific) {
            Some(SharedChunkRejectionReason::RootSpecific)
        } else if entries.iter().any(|entry| entry.captures_mutable_identity) {
            Some(SharedChunkRejectionReason::MutableIdentity)
        } else if entries.iter().any(|entry| !entry.registration_only) {
            Some(SharedChunkRejectionReason::ExecutesOnRegistration)
        } else if entries
            .iter()
            .map(|entry| &entry.runtime_protocol)
            .any(|protocol| protocol != &entries[0].runtime_protocol)
        {
            Some(SharedChunkRejectionReason::ProtocolMismatch)
        } else {
            None
        };
        if let Some(reason) = reason {
            rejections.push((fingerprint, reason));
        } else {
            groups
                .entry(roots)
                .or_default()
                .push((fingerprint, entries[0].canonical_bytes.len()));
        }
    }
    let mut candidates = Vec::new();
    for (roots, mut programs) in groups {
        programs.sort_by(|a, b| a.0.cmp(&b.0));
        let bytes = programs.iter().map(|(_, bytes)| bytes).sum::<usize>();
        let savings = SharedChunkSavingsCalculation {
            canonical_bytes: bytes,
            root_count: roots.len(),
            wrapper_bytes,
            import_bytes_per_root,
            net_saved_bytes: (bytes as i128 * roots.len() as i128)
                - (bytes as i128
                    + wrapper_bytes as i128
                    + import_bytes_per_root as i128 * roots.len() as i128),
        };
        let fingerprints = programs
            .iter()
            .map(|(fingerprint, _)| fingerprint.clone())
            .collect::<Vec<_>>();
        if bytes < ProductionOptimizationPolicyV1::SHARED_CHUNK_MIN_CANONICAL_BYTES {
            rejections.extend(
                fingerprints.into_iter().map(|fingerprint| {
                    (fingerprint, SharedChunkRejectionReason::BelowCanonicalBytes)
                }),
            );
        } else if savings.net_saved_bytes
            < ProductionOptimizationPolicyV1::SHARED_CHUNK_MIN_NET_SAVED_BYTES as i128
        {
            rejections.extend(
                fingerprints
                    .into_iter()
                    .map(|fingerprint| (fingerprint, SharedChunkRejectionReason::BelowNetSavings)),
            );
        } else {
            candidates.push(SharedChunkCandidate {
                id: SharedChunkCandidateId::for_roots_and_programs(&roots, &fingerprints)
                    .expect("canonical candidate"),
                programs: fingerprints,
                consumers: roots
                    .into_iter()
                    .map(|root_id| SharedChunkConsumerRoot { root_id })
                    .collect(),
                savings,
            });
        }
    }
    candidates.sort_by(|a, b| a.id.cmp(&b.id));
    rejections.sort_by(|a, b| a.0.cmp(&b.0));
    SharedChunkCandidatePlan {
        candidates,
        rejections,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn occurrence(root: &str, program: &str, bytes: usize) -> SharedChunkProgramOccurrence {
        SharedChunkProgramOccurrence {
            root_id: root.to_string(),
            fingerprint: ExecutableProgramFingerprint::for_canonical_opcode_stream(
                program.as_bytes(),
            ),
            canonical_bytes: vec![b'x'; bytes],
            eager_required: false,
            root_specific: false,
            captures_mutable_identity: false,
            registration_only: true,
            runtime_protocol: "v1".to_string(),
        }
    }
    #[test]
    fn k6_groups_exact_root_sets_and_uses_fixed_savings() {
        let plan = plan_shared_lazy_chunk_candidates(
            &[
                occurrence("b", "one", 300),
                occurrence("a", "one", 300),
                occurrence("a", "two", 300),
                occurrence("b", "two", 300),
                occurrence("c", "three", 300),
            ],
            32,
            8,
        );
        assert_eq!(plan.candidates.len(), 1);
        assert_eq!(plan.candidates[0].programs.len(), 2);
        assert_eq!(plan.candidates[0].savings.net_saved_bytes, 552);
        assert_eq!(plan.rejections.len(), 1);
    }
    #[test]
    fn k6_rejects_root_specific_and_below_threshold_programs() {
        let mut specific = occurrence("a", "specific", 300);
        specific.root_specific = true;
        let plan = plan_shared_lazy_chunk_candidates(
            &[
                specific,
                occurrence("b", "specific", 300),
                occurrence("a", "small", 100),
                occurrence("b", "small", 100),
            ],
            0,
            0,
        );
        assert!(plan.candidates.is_empty());
        assert_eq!(plan.rejections.len(), 2);
    }
}
