//! Phase K entry-only production optimization reservations and policy constants.
//!
//! K0 records the immutable policy and diagnostic space without introducing an
//! optimizer, production artifact, report, or executable behavior.

use std::fmt;
use std::str::FromStr;

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::ResumeBuildId;

/// A Phase K diagnostic code reserved before optimization products exist.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ProductionOptimizationDiagnosticReservation {
    pub code: &'static str,
    pub meaning: &'static str,
}

/// The contiguous public compiler diagnostic range reserved by K0.
pub const PRODUCTION_OPTIMIZATION_DIAGNOSTIC_RESERVATIONS:
    [ProductionOptimizationDiagnosticReservation; 16] = [
    ProductionOptimizationDiagnosticReservation {
        code: "PSC1112",
        meaning: "Invalid optimization root",
    },
    ProductionOptimizationDiagnosticReservation {
        code: "PSC1113",
        meaning: "Invalid program fingerprint",
    },
    ProductionOptimizationDiagnosticReservation {
        code: "PSC1114",
        meaning: "Unsafe program deduplication",
    },
    ProductionOptimizationDiagnosticReservation {
        code: "PSC1115",
        meaning: "Invalid constant pool entry",
    },
    ProductionOptimizationDiagnosticReservation {
        code: "PSC1116",
        meaning: "Invalid shared chunk candidate",
    },
    ProductionOptimizationDiagnosticReservation {
        code: "PSC1117",
        meaning: "Production chunk cycle",
    },
    ProductionOptimizationDiagnosticReservation {
        code: "PSC1118",
        meaning: "Invalid runtime ordinal table",
    },
    ProductionOptimizationDiagnosticReservation {
        code: "PSC1119",
        meaning: "Production artifact mismatch",
    },
    ProductionOptimizationDiagnosticReservation {
        code: "PSC1120",
        meaning: "Unsafe binding write coalescing",
    },
    ProductionOptimizationDiagnosticReservation {
        code: "PSC1121",
        meaning: "Missing runtime cleanup",
    },
    ProductionOptimizationDiagnosticReservation {
        code: "PSC1122",
        meaning: "Invalid runtime cleanup order",
    },
    ProductionOptimizationDiagnosticReservation {
        code: "PSC1123",
        meaning: "Detached activation target",
    },
    ProductionOptimizationDiagnosticReservation {
        code: "PSC1124",
        meaning: "Invalid production failure record",
    },
    ProductionOptimizationDiagnosticReservation {
        code: "PSC1125",
        meaning: "Optimization report mismatch",
    },
    ProductionOptimizationDiagnosticReservation {
        code: "PSC1126",
        meaning: "Production budget regression",
    },
    ProductionOptimizationDiagnosticReservation {
        code: "PSC1127",
        meaning: "Production determinism failure",
    },
];

/// The K0 integrity range follows the final J21 reservation exactly.
pub const PRODUCTION_OPTIMIZATION_INTEGRITY_RESERVATION_START: u32 = 1385;
/// K0 reserves 128 contiguous internal integrity codes, inclusively.
pub const PRODUCTION_OPTIMIZATION_INTEGRITY_RESERVATION_END: u32 = 1512;

/// The immutable compiler-owned `ProductionOptimizationPolicyV1` constants.
///
/// These constants are compiler-owned and are not user configuration.
pub struct ProductionOptimizationPolicyV1;

impl ProductionOptimizationPolicyV1 {
    pub const SHARED_CHUNK_MIN_ROOT_COUNT: usize = 2;
    pub const SHARED_CHUNK_MIN_PROGRAM_COUNT: usize = 1;
    pub const SHARED_CHUNK_MIN_CANONICAL_BYTES: usize = 192;
    pub const SHARED_CHUNK_MIN_NET_SAVED_BYTES: usize = 256;
    pub const SHARED_CHUNK_MAX_DEPENDENCY_DEPTH: usize = 1;
    pub const INLINE_LITERAL_MAX_UTF8_BYTES: usize = 24;
    pub const CONSTANT_POOL_MIN_REUSE_COUNT: usize = 2;
}

/// Parse failure for a Phase K compiler identity.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ProductionOptimizationIdentityParseError;

impl fmt::Display for ProductionOptimizationIdentityParseError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("invalid production optimization identity")
    }
}

impl std::error::Error for ProductionOptimizationIdentityParseError {}

macro_rules! optimization_id {
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
    };
}

optimization_id!(OptimizationPolicyId);
optimization_id!(ExecutableProgramFingerprint);
optimization_id!(ConstantPoolEntryId);
optimization_id!(RuntimeTableId);
optimization_id!(ProductionArtifactId);
optimization_id!(SharedChunkCandidateId);
optimization_id!(ProductionChunkId);
optimization_id!(OptimizationDecisionId);
optimization_id!(OptimizationReportId);
optimization_id!(BenchmarkFixtureId);
optimization_id!(PerformanceBudgetId);

impl OptimizationPolicyId {
    pub const PRODUCTION_V1: &'static str = "optimization-policy:production-v1";

    #[must_use]
    pub fn production_v1() -> Self {
        Self(Self::PRODUCTION_V1.to_string())
    }
}

impl FromStr for OptimizationPolicyId {
    type Err = ProductionOptimizationIdentityParseError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        (value == Self::PRODUCTION_V1)
            .then(|| Self(value.to_string()))
            .ok_or(ProductionOptimizationIdentityParseError)
    }
}

impl ExecutableProgramFingerprint {
    #[must_use]
    pub fn for_canonical_opcode_stream(stream: &[u8]) -> Self {
        Self(format!("program-fingerprint:{}", canonical_hash(stream)))
    }
}

impl ConstantPoolEntryId {
    #[must_use]
    pub fn for_canonical_value(type_tag: &str, canonical_value_bytes: &[u8]) -> Option<Self> {
        canonical_label(type_tag).then(|| {
            Self(format!(
                "constant-pool:{type_tag}:{}",
                canonical_hash(canonical_value_bytes)
            ))
        })
    }
}

impl RuntimeTableId {
    #[must_use]
    pub fn for_artifact_table(artifact_kind: &str, table_kind: &str) -> Option<Self> {
        (canonical_label(artifact_kind) && canonical_label(table_kind))
            .then(|| Self(format!("runtime-table:{artifact_kind}:{table_kind}")))
    }
}

impl ProductionArtifactId {
    #[must_use]
    pub fn for_canonical_packed_bytes(artifact_kind: &str, packed_bytes: &[u8]) -> Option<Self> {
        canonical_label(artifact_kind).then(|| {
            Self(format!(
                "production-artifact:{artifact_kind}:{}",
                canonical_hash(packed_bytes)
            ))
        })
    }
}

impl SharedChunkCandidateId {
    #[must_use]
    pub fn for_roots_and_programs(
        roots: &[String],
        programs: &[ExecutableProgramFingerprint],
    ) -> Option<Self> {
        let roots = canonical_sorted(roots)?;
        let programs =
            canonical_sorted(&programs.iter().map(ToString::to_string).collect::<Vec<_>>())?;
        Some(Self(format!(
            "shared-chunk-candidate:{}",
            canonical_hash(
                format!(
                    "roots\n{}\nprograms\n{}",
                    roots.join("\n"),
                    programs.join("\n")
                )
                .as_bytes()
            )
        )))
    }
}

impl ProductionChunkId {
    /// The stable identity of the single compiler-owned eager runtime module.
    #[must_use]
    pub fn eager_runtime_v1() -> Self {
        Self("production-chunk:eager:runtime-v1".to_string())
    }

    #[must_use]
    pub fn for_activation_roots_and_programs(
        chunk_kind: &str,
        activation_roots: &[String],
        programs: &[ExecutableProgramFingerprint],
    ) -> Option<Self> {
        let roots = canonical_sorted(activation_roots)?;
        let programs = canonical_sorted_or_empty(
            &programs.iter().map(ToString::to_string).collect::<Vec<_>>(),
        )?;
        canonical_label(chunk_kind).then(|| {
            Self(format!(
                "production-chunk:{chunk_kind}:{}",
                canonical_hash(
                    format!(
                        "roots\n{}\nprograms\n{}",
                        roots.join("\n"),
                        programs.join("\n")
                    )
                    .as_bytes()
                )
            ))
        })
    }
}

impl OptimizationDecisionId {
    #[must_use]
    pub fn for_canonical_subject(optimization_kind: &str, subject_id: &str) -> Option<Self> {
        (canonical_label(optimization_kind) && canonical_subject(subject_id)).then(|| {
            Self(format!(
                "optimization-decision:{optimization_kind}:{subject_id}"
            ))
        })
    }
}

impl OptimizationReportId {
    #[must_use]
    pub fn for_resume_build_id(build_id: &ResumeBuildId) -> Self {
        Self(format!("optimization-report:{build_id}"))
    }
}

impl BenchmarkFixtureId {
    #[must_use]
    pub fn for_fixture_relative_path(path: &str) -> Option<Self> {
        canonical_fixture_relative_path(path).then(|| Self(format!("benchmark-fixture:{path}")))
    }
}

impl PerformanceBudgetId {
    #[must_use]
    pub fn for_metric(metric_name: &str) -> Option<Self> {
        canonical_label(metric_name).then(|| Self(format!("performance-budget:{metric_name}:v1")))
    }
}

macro_rules! prefixed_parse {
    ($name:ident, $prefix:literal) => {
        impl FromStr for $name {
            type Err = ProductionOptimizationIdentityParseError;

            fn from_str(value: &str) -> Result<Self, Self::Err> {
                value
                    .strip_prefix($prefix)
                    .filter(|payload| !payload.is_empty())
                    .map(|_| Self(value.to_string()))
                    .ok_or(ProductionOptimizationIdentityParseError)
            }
        }
    };
}

prefixed_parse!(ExecutableProgramFingerprint, "program-fingerprint:");
prefixed_parse!(ConstantPoolEntryId, "constant-pool:");
prefixed_parse!(RuntimeTableId, "runtime-table:");
prefixed_parse!(ProductionArtifactId, "production-artifact:");
prefixed_parse!(SharedChunkCandidateId, "shared-chunk-candidate:");
prefixed_parse!(ProductionChunkId, "production-chunk:");
prefixed_parse!(OptimizationDecisionId, "optimization-decision:");
prefixed_parse!(OptimizationReportId, "optimization-report:resume-build:");
prefixed_parse!(BenchmarkFixtureId, "benchmark-fixture:");
prefixed_parse!(PerformanceBudgetId, "performance-budget:");

/// Immutable compiler-owned policy product. It is intentionally not a public
/// artifact in K1 and has no optimizer or runtime consumer.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProductionOptimizationPolicy {
    id: OptimizationPolicyId,
}

impl ProductionOptimizationPolicy {
    #[must_use]
    pub fn production_v1() -> Self {
        Self {
            id: OptimizationPolicyId::production_v1(),
        }
    }

    #[must_use]
    pub const fn id(&self) -> &OptimizationPolicyId {
        &self.id
    }
}

fn canonical_hash(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

fn canonical_label(value: &str) -> bool {
    !value.is_empty()
        && value.bytes().all(|byte| {
            byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'-' || byte == b'_'
        })
}

fn canonical_subject(value: &str) -> bool {
    !value.is_empty() && !value.contains(['\n', '\r'])
}

fn canonical_fixture_relative_path(value: &str) -> bool {
    !value.is_empty()
        && !value.starts_with('/')
        && !value.contains('\\')
        && value
            .split('/')
            .all(|segment| !segment.is_empty() && segment != "." && segment != "..")
}

fn canonical_sorted(values: &[String]) -> Option<Vec<String>> {
    (!values.is_empty() && values.iter().all(|value| canonical_subject(value)))
        .then(|| {
            let mut sorted = values.to_vec();
            sorted.sort();
            if sorted.windows(2).any(|pair| pair[0] == pair[1]) {
                return Vec::new();
            }
            sorted
        })
        .filter(|values| !values.is_empty())
}

fn canonical_sorted_or_empty(values: &[String]) -> Option<Vec<String>> {
    if !values.iter().all(|value| canonical_subject(value)) {
        return None;
    }
    let mut sorted = values.to_vec();
    sorted.sort();
    (!sorted.windows(2).any(|pair| pair[0] == pair[1])).then_some(sorted)
}

#[cfg(test)]
mod tests {
    use super::{
        BenchmarkFixtureId, ConstantPoolEntryId, ExecutableProgramFingerprint,
        OptimizationDecisionId, OptimizationPolicyId, OptimizationReportId, PerformanceBudgetId,
        ProductionArtifactId, ProductionChunkId, ProductionOptimizationPolicy,
        ProductionOptimizationPolicyV1, RuntimeTableId, SharedChunkCandidateId,
        PRODUCTION_OPTIMIZATION_DIAGNOSTIC_RESERVATIONS,
        PRODUCTION_OPTIMIZATION_INTEGRITY_RESERVATION_END,
        PRODUCTION_OPTIMIZATION_INTEGRITY_RESERVATION_START,
    };
    use crate::ResumeBuildId;
    use std::str::FromStr;

    #[test]
    fn k0_reserves_phase_k_ranges_and_inert_policy_without_products() {
        let codes = PRODUCTION_OPTIMIZATION_DIAGNOSTIC_RESERVATIONS
            .iter()
            .map(|reservation| reservation.code)
            .collect::<Vec<_>>();
        assert_eq!(codes.first(), Some(&"PSC1112"));
        assert_eq!(codes.last(), Some(&"PSC1127"));
        assert_eq!(codes.len(), 16);
        assert_eq!(PRODUCTION_OPTIMIZATION_INTEGRITY_RESERVATION_START, 1385);
        assert_eq!(PRODUCTION_OPTIMIZATION_INTEGRITY_RESERVATION_END, 1512);
        assert_eq!(
            PRODUCTION_OPTIMIZATION_INTEGRITY_RESERVATION_END
                - PRODUCTION_OPTIMIZATION_INTEGRITY_RESERVATION_START
                + 1,
            128
        );
        assert_eq!(
            ProductionOptimizationPolicyV1::SHARED_CHUNK_MIN_ROOT_COUNT,
            2
        );
        assert_eq!(
            ProductionOptimizationPolicyV1::SHARED_CHUNK_MIN_PROGRAM_COUNT,
            1
        );
        assert_eq!(
            ProductionOptimizationPolicyV1::SHARED_CHUNK_MIN_CANONICAL_BYTES,
            192
        );
        assert_eq!(
            ProductionOptimizationPolicyV1::SHARED_CHUNK_MIN_NET_SAVED_BYTES,
            256
        );
        assert_eq!(
            ProductionOptimizationPolicyV1::SHARED_CHUNK_MAX_DEPENDENCY_DEPTH,
            1
        );
        assert_eq!(
            ProductionOptimizationPolicyV1::INLINE_LITERAL_MAX_UTF8_BYTES,
            24
        );
        assert_eq!(
            ProductionOptimizationPolicyV1::CONSTANT_POOL_MIN_REUSE_COUNT,
            2
        );
    }

    #[test]
    fn k1_optimization_identities_are_typed_canonical_and_order_independent() {
        let policy = ProductionOptimizationPolicy::production_v1();
        assert_eq!(policy.id(), &OptimizationPolicyId::production_v1());
        assert_eq!(policy.id().to_string(), "optimization-policy:production-v1");
        assert_eq!(
            OptimizationPolicyId::from_str("optimization-policy:production-v1"),
            Ok(OptimizationPolicyId::production_v1())
        );
        assert!(OptimizationPolicyId::from_str("optimization-policy:development-v1").is_err());

        let first =
            ExecutableProgramFingerprint::for_canonical_opcode_stream(b"opcode\nread-state");
        let second =
            ExecutableProgramFingerprint::for_canonical_opcode_stream(b"opcode\nwrite-state");
        assert_ne!(first, second);
        assert_ne!(
            first.to_string(),
            ConstantPoolEntryId::for_canonical_value("string", b"opcode\nread-state")
                .expect("canonical constant")
                .to_string()
        );
        assert_eq!(
            serde_json::from_str::<ExecutableProgramFingerprint>(
                &serde_json::to_string(&first).expect("serialize fingerprint")
            )
            .expect("deserialize fingerprint"),
            first
        );

        let roots = vec![
            "resume-boundary:z".to_string(),
            "resume-boundary:a".to_string(),
        ];
        let reversed_roots = vec![
            "resume-boundary:a".to_string(),
            "resume-boundary:z".to_string(),
        ];
        let programs = vec![second.clone(), first.clone()];
        let reversed_programs = vec![first.clone(), second.clone()];
        assert_eq!(
            SharedChunkCandidateId::for_roots_and_programs(&roots, &programs),
            SharedChunkCandidateId::for_roots_and_programs(&reversed_roots, &reversed_programs)
        );
        assert_eq!(
            ProductionChunkId::for_activation_roots_and_programs("shared", &roots, &programs),
            ProductionChunkId::for_activation_roots_and_programs(
                "shared",
                &reversed_roots,
                &reversed_programs
            )
        );
        assert!(SharedChunkCandidateId::for_roots_and_programs(
            &["duplicate".to_string(), "duplicate".to_string()],
            std::slice::from_ref(&first)
        )
        .is_none());

        assert!(RuntimeTableId::for_artifact_table("production_runtime", "anchors").is_some());
        assert!(RuntimeTableId::for_artifact_table("ProductionRuntime", "anchors").is_none());
        assert!(
            ProductionArtifactId::for_canonical_packed_bytes("production_runtime", b"packed")
                .is_some()
        );
        assert!(OptimizationDecisionId::for_canonical_subject(
            "deduplication",
            "program-fingerprint:abc"
        )
        .is_some());
        assert!(BenchmarkFixtureId::for_fixture_relative_path("fixtures/k/fixture.tsx").is_some());
        assert!(BenchmarkFixtureId::for_fixture_relative_path("/host/fixture.tsx").is_none());
        assert!(BenchmarkFixtureId::for_fixture_relative_path("fixtures/../fixture.tsx").is_none());
        assert_eq!(
            PerformanceBudgetId::for_metric("runtime_bytes")
                .expect("metric")
                .to_string(),
            "performance-budget:runtime_bytes:v1"
        );
        assert_eq!(
            OptimizationReportId::for_resume_build_id(&ResumeBuildId::for_public_inputs(
                "canonical-inputs"
            ))
            .to_string(),
            format!(
                "optimization-report:{}",
                ResumeBuildId::for_public_inputs("canonical-inputs")
            )
        );
    }
}
