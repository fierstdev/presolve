/// A Phase I diagnostic code reserved before form syntax is lowered.
///
/// I0 owns only the stable range and its roadmap-defined meanings. The
/// canonical I18 projector remains responsible for creating diagnostics from
/// immutable form products.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct FormDiagnosticReservation {
    pub code: &'static str,
    pub meaning: &'static str,
}

/// The next contiguous public compiler diagnostic range after Phase H.
pub const FORM_DIAGNOSTIC_RESERVATIONS: [FormDiagnosticReservation; 12] = [
    FormDiagnosticReservation {
        code: "EZC1084",
        meaning: "Duplicate form",
    },
    FormDiagnosticReservation {
        code: "EZC1085",
        meaning: "Duplicate field",
    },
    FormDiagnosticReservation {
        code: "EZC1086",
        meaning: "Missing submit",
    },
    FormDiagnosticReservation {
        code: "EZC1087",
        meaning: "Invalid validator",
    },
    FormDiagnosticReservation {
        code: "EZC1088",
        meaning: "Cyclic validation dependency",
    },
    FormDiagnosticReservation {
        code: "EZC1089",
        meaning: "Invalid serialization",
    },
    FormDiagnosticReservation {
        code: "EZC1090",
        meaning: "Invalid reset",
    },
    FormDiagnosticReservation {
        code: "EZC1091",
        meaning: "Duplicate binding",
    },
    FormDiagnosticReservation {
        code: "EZC1092",
        meaning: "Nested forms",
    },
    FormDiagnosticReservation {
        code: "EZC1093",
        meaning: "Invalid ownership",
    },
    FormDiagnosticReservation {
        code: "EZC1094",
        meaning: "Invalid submit signature",
    },
    FormDiagnosticReservation {
        code: "EZC1095",
        meaning: "Invalid field scope",
    },
];

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;

    use crate::{
        COMPONENT_DIAGNOSTIC_CONTRACTS, RESUME_MANIFEST_SCHEMA_VERSION,
        RUNTIME_COMPONENT_ARTIFACT_SCHEMA_VERSION, RUNTIME_CONTEXT_ARTIFACT_SCHEMA_VERSION,
        SEMANTIC_GRAPH_SCHEMA_VERSION, TEMPLATE_MANIFEST_SCHEMA_VERSION,
    };

    use super::FORM_DIAGNOSTIC_RESERVATIONS;

    #[test]
    fn phase_i_entry_reserves_the_next_diagnostic_range_after_the_frozen_h_baseline() {
        let phase_h_codes = COMPONENT_DIAGNOSTIC_CONTRACTS
            .iter()
            .map(|contract| contract.code)
            .collect::<BTreeSet<_>>();
        let form_codes = FORM_DIAGNOSTIC_RESERVATIONS
            .iter()
            .map(|reservation| reservation.code)
            .collect::<Vec<_>>();

        assert_eq!(phase_h_codes.last(), Some(&"EZC1083"));
        assert_eq!(form_codes.first(), Some(&"EZC1084"));
        assert_eq!(form_codes.last(), Some(&"EZC1095"));
        assert_eq!(
            form_codes.len(),
            form_codes.iter().collect::<BTreeSet<_>>().len()
        );

        for (offset, reservation) in FORM_DIAGNOSTIC_RESERVATIONS.iter().enumerate() {
            assert_eq!(reservation.code, format!("EZC{}", 1084 + offset));
            assert!(!reservation.meaning.is_empty());
        }
    }

    #[test]
    fn phase_i_i17_updates_the_forms_inspection_schema_versions() {
        assert_eq!(RUNTIME_COMPONENT_ARTIFACT_SCHEMA_VERSION, 2);
        assert_eq!(RESUME_MANIFEST_SCHEMA_VERSION, 5);
        assert_eq!(TEMPLATE_MANIFEST_SCHEMA_VERSION, 3);
        assert_eq!(RUNTIME_CONTEXT_ARTIFACT_SCHEMA_VERSION, 2);
        assert_eq!(SEMANTIC_GRAPH_SCHEMA_VERSION, 6);
    }
}
