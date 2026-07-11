use crate::application_semantic_model::ApplicationSemanticModel;
use crate::semantic_id::SemanticOwner;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AsmValidationDiagnostic {
    pub code: String,
    pub message: String,
}

#[must_use]
pub fn validate_application_semantic_model(
    model: &ApplicationSemanticModel,
) -> Vec<AsmValidationDiagnostic> {
    let mut diagnostics = Vec::new();

    for (id, owner) in &model.ownership {
        if model.entity(id).is_none() {
            diagnostics.push(AsmValidationDiagnostic {
                code: "EZASM1001".to_string(),
                message: format!("ownership references missing semantic entity `{id}`"),
            });
        }
        if !model.provenance.contains_key(id) {
            diagnostics.push(AsmValidationDiagnostic {
                code: "EZASM1002".to_string(),
                message: format!("semantic entity `{id}` is missing source provenance"),
            });
        }
        if let SemanticOwner::Entity(owner_id) = owner {
            if model.entity(owner_id).is_none() {
                diagnostics.push(AsmValidationDiagnostic {
                    code: "EZASM1003".to_string(),
                    message: format!("semantic entity `{id}` has missing owner `{owner_id}`"),
                });
            }
        }
    }

    for id in model.provenance.keys() {
        if !model.ownership.contains_key(id) {
            diagnostics.push(AsmValidationDiagnostic {
                code: "EZASM1004".to_string(),
                message: format!("provenance references unowned semantic entity `{id}`"),
            });
        }
    }

    for reference in &model.references {
        if model.entity(&reference.source).is_none() || model.entity(&reference.target).is_none() {
            diagnostics.push(AsmValidationDiagnostic {
                code: "EZASM1005".to_string(),
                message: format!(
                    "reference from `{}` to `{}` has a missing endpoint",
                    reference.source, reference.target
                ),
            });
        }
        if model.provenance(&reference.source) != Some(&reference.provenance) {
            diagnostics.push(AsmValidationDiagnostic {
                code: "EZASM1006".to_string(),
                message: format!(
                    "reference source `{}` has mismatched provenance",
                    reference.source
                ),
            });
        }
    }

    diagnostics
}
