//! Phase K entry-only production optimization reservations and policy constants.
//!
//! K0 records the immutable policy and diagnostic space without introducing an
//! optimizer, production artifact, report, or executable behavior.

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
        code: "EZC1112",
        meaning: "Invalid optimization root",
    },
    ProductionOptimizationDiagnosticReservation {
        code: "EZC1113",
        meaning: "Invalid program fingerprint",
    },
    ProductionOptimizationDiagnosticReservation {
        code: "EZC1114",
        meaning: "Unsafe program deduplication",
    },
    ProductionOptimizationDiagnosticReservation {
        code: "EZC1115",
        meaning: "Invalid constant pool entry",
    },
    ProductionOptimizationDiagnosticReservation {
        code: "EZC1116",
        meaning: "Invalid shared chunk candidate",
    },
    ProductionOptimizationDiagnosticReservation {
        code: "EZC1117",
        meaning: "Production chunk cycle",
    },
    ProductionOptimizationDiagnosticReservation {
        code: "EZC1118",
        meaning: "Invalid runtime ordinal table",
    },
    ProductionOptimizationDiagnosticReservation {
        code: "EZC1119",
        meaning: "Production artifact mismatch",
    },
    ProductionOptimizationDiagnosticReservation {
        code: "EZC1120",
        meaning: "Unsafe binding write coalescing",
    },
    ProductionOptimizationDiagnosticReservation {
        code: "EZC1121",
        meaning: "Missing runtime cleanup",
    },
    ProductionOptimizationDiagnosticReservation {
        code: "EZC1122",
        meaning: "Invalid runtime cleanup order",
    },
    ProductionOptimizationDiagnosticReservation {
        code: "EZC1123",
        meaning: "Detached activation target",
    },
    ProductionOptimizationDiagnosticReservation {
        code: "EZC1124",
        meaning: "Invalid production failure record",
    },
    ProductionOptimizationDiagnosticReservation {
        code: "EZC1125",
        meaning: "Optimization report mismatch",
    },
    ProductionOptimizationDiagnosticReservation {
        code: "EZC1126",
        meaning: "Production budget regression",
    },
    ProductionOptimizationDiagnosticReservation {
        code: "EZC1127",
        meaning: "Production determinism failure",
    },
];

/// The K0 integrity range follows the final J21 reservation exactly.
pub const PRODUCTION_OPTIMIZATION_INTEGRITY_RESERVATION_START: u32 = 1385;
/// K0 reserves 128 contiguous internal integrity codes, inclusively.
pub const PRODUCTION_OPTIMIZATION_INTEGRITY_RESERVATION_END: u32 = 1512;

/// The immutable compiler-owned ProductionOptimizationPolicyV1 constants.
///
/// These constants are deliberately inert in K0. K1 introduces typed policy
/// identity and later slices may consume the policy only through canonical
/// compiler products.
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

#[cfg(test)]
mod tests {
    use super::{
        ProductionOptimizationPolicyV1, PRODUCTION_OPTIMIZATION_DIAGNOSTIC_RESERVATIONS,
        PRODUCTION_OPTIMIZATION_INTEGRITY_RESERVATION_END,
        PRODUCTION_OPTIMIZATION_INTEGRITY_RESERVATION_START,
    };

    #[test]
    fn k0_reserves_phase_k_ranges_and_inert_policy_without_products() {
        let codes = PRODUCTION_OPTIMIZATION_DIAGNOSTIC_RESERVATIONS
            .iter()
            .map(|reservation| reservation.code)
            .collect::<Vec<_>>();
        assert_eq!(codes.first(), Some(&"EZC1112"));
        assert_eq!(codes.last(), Some(&"EZC1127"));
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
}
