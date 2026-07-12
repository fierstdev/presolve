use crate::application_semantic_model::ApplicationSemanticModel;
use crate::semantic_id::SemanticOwner;
use crate::SemanticTypeId;

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

    validate_semantic_types(model, &mut diagnostics);

    diagnostics
}

fn validate_semantic_types(
    model: &ApplicationSemanticModel,
    diagnostics: &mut Vec<AsmValidationDiagnostic>,
) {
    for (subject, assignment) in &model.semantic_types.assignments {
        if subject != &assignment.subject {
            diagnostics.push(AsmValidationDiagnostic {
                code: "EZASM1101".to_string(),
                message: format!(
                    "semantic type assignment map key `{subject}` does not match subject `{}`",
                    assignment.subject
                ),
            });
        }
        if assignment.id != SemanticTypeId::for_subject(&assignment.subject) {
            diagnostics.push(AsmValidationDiagnostic {
                code: "EZASM1102".to_string(),
                message: format!(
                    "semantic type assignment for `{subject}` has an invalid canonical type ID `{}`",
                    assignment.id
                ),
            });
        }
        if model.entity(&assignment.subject).is_none() {
            diagnostics.push(AsmValidationDiagnostic {
                code: "EZASM1103".to_string(),
                message: format!(
                    "semantic type assignment references missing subject `{}`",
                    assignment.subject
                ),
            });
        }
        if model.provenance(&assignment.subject) != Some(&assignment.provenance) {
            diagnostics.push(AsmValidationDiagnostic {
                code: "EZASM1104".to_string(),
                message: format!(
                    "semantic type assignment for `{subject}` has inconsistent provenance"
                ),
            });
        }
        if model.entity(&assignment.origin).is_none()
            && !model
                .semantic_types
                .aliases
                .contains_key(&assignment.origin)
        {
            diagnostics.push(AsmValidationDiagnostic {
                code: "EZASM1105".to_string(),
                message: format!(
                    "semantic type assignment for `{subject}` has unresolved origin `{}`",
                    assignment.origin
                ),
            });
        }
    }

    for alias in model.semantic_types.aliases.values() {
        let expected = crate::SemanticId::type_alias_in_module(&alias.provenance.path, &alias.name);
        if alias.id != expected {
            diagnostics.push(AsmValidationDiagnostic {
                code: "EZASM1106".to_string(),
                message: format!(
                    "type alias `{}` has invalid canonical identity `{}`",
                    alias.name, alias.id
                ),
            });
        }
    }
}
