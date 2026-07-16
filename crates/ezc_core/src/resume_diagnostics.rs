use crate::resume_instance::SerializableInstance;
use crate::resume_plan::ResumePlan;
use crate::semantic_id::SemanticId;

/// A Phase J diagnostic code reserved before executable resumability products
/// are introduced. J19 alone projects these from immutable Phase J products.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ResumeDiagnosticReservation {
    pub code: &'static str,
    pub meaning: &'static str,
}

/// The contiguous public compiler diagnostic range reserved by J0.
pub const RESUME_DIAGNOSTIC_RESERVATIONS: [ResumeDiagnosticReservation; 16] = [
    ResumeDiagnosticReservation {
        code: "EZC1096",
        meaning: "Unsupported resume value",
    },
    ResumeDiagnosticReservation {
        code: "EZC1097",
        meaning: "Missing resume owner",
    },
    ResumeDiagnosticReservation {
        code: "EZC1098",
        meaning: "Resume boundary cycle",
    },
    ResumeDiagnosticReservation {
        code: "EZC1099",
        meaning: "Invalid resume retention",
    },
    ResumeDiagnosticReservation {
        code: "EZC1100",
        meaning: "Invalid resume recomputation",
    },
    ResumeDiagnosticReservation {
        code: "EZC1101",
        meaning: "Invalid activation policy",
    },
    ResumeDiagnosticReservation {
        code: "EZC1102",
        meaning: "Resume chunk cycle",
    },
    ResumeDiagnosticReservation {
        code: "EZC1103",
        meaning: "Missing resume program",
    },
    ResumeDiagnosticReservation {
        code: "EZC1104",
        meaning: "Invalid resume anchor",
    },
    ResumeDiagnosticReservation {
        code: "EZC1105",
        meaning: "Resume schema collision",
    },
    ResumeDiagnosticReservation {
        code: "EZC1106",
        meaning: "Invalid snapshot stable state",
    },
    ResumeDiagnosticReservation {
        code: "EZC1107",
        meaning: "Resume artifact mismatch",
    },
    ResumeDiagnosticReservation {
        code: "EZC1108",
        meaning: "Lazy event payload unsupported",
    },
    ResumeDiagnosticReservation {
        code: "EZC1109",
        meaning: "Missing resume chunk",
    },
    ResumeDiagnosticReservation {
        code: "EZC1110",
        meaning: "Invalid resume ordering",
    },
    ResumeDiagnosticReservation {
        code: "EZC1111",
        meaning: "Unsupported resume topology",
    },
];

/// J1-J21 must allocate internal integrity codes only within this J0-reserved range.
pub const RESUME_INTEGRITY_RESERVATION_START: u32 = 1289;
pub const RESUME_INTEGRITY_RESERVATION_END: u32 = 1384;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResumeDiagnostic {
    pub code: String,
    pub component: SemanticId,
    pub state: Option<SemanticId>,
}

#[must_use]
pub fn validate_resume_instances(
    plan: &ResumePlan,
    instances: &[SerializableInstance],
) -> Vec<ResumeDiagnostic> {
    let mut diagnostics = Vec::new();
    for component in &plan.components {
        let Some(instance) = instances
            .iter()
            .find(|instance| instance.component == component.component)
        else {
            diagnostics.push(ResumeDiagnostic {
                code: "EZRSM1001".to_string(),
                component: component.component.clone(),
                state: None,
            });
            continue;
        };
        for state in &component.state {
            if !instance.state.contains_key(state) {
                diagnostics.push(ResumeDiagnostic {
                    code: "EZRSM1002".to_string(),
                    component: component.component.clone(),
                    state: Some(state.clone()),
                });
            }
        }
    }
    diagnostics
}

#[cfg(test)]
mod tests {
    use super::{
        RESUME_DIAGNOSTIC_RESERVATIONS, RESUME_INTEGRITY_RESERVATION_END,
        RESUME_INTEGRITY_RESERVATION_START,
    };

    #[test]
    fn j0_reserves_the_public_and_internal_resumability_ranges_without_products() {
        let codes = RESUME_DIAGNOSTIC_RESERVATIONS
            .iter()
            .map(|reservation| reservation.code)
            .collect::<Vec<_>>();
        assert_eq!(codes.first(), Some(&"EZC1096"));
        assert_eq!(codes.last(), Some(&"EZC1111"));
        assert_eq!(codes.len(), 16);
        assert_eq!(RESUME_INTEGRITY_RESERVATION_START, 1289);
        assert_eq!(RESUME_INTEGRITY_RESERVATION_END, 1384);
        assert_eq!(
            RESUME_INTEGRITY_RESERVATION_END - RESUME_INTEGRITY_RESERVATION_START + 1,
            96
        );
        assert_eq!(crate::RESUME_MANIFEST_SCHEMA_VERSION, 5);
        assert_eq!(crate::SEMANTIC_GRAPH_SCHEMA_VERSION, 6);
        assert_eq!(crate::TEMPLATE_MANIFEST_SCHEMA_VERSION, 3);
    }
}
