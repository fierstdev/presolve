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
        code: "PSC1084",
        meaning: "Duplicate form",
    },
    FormDiagnosticReservation {
        code: "PSC1085",
        meaning: "Duplicate field",
    },
    FormDiagnosticReservation {
        code: "PSC1086",
        meaning: "Missing submit",
    },
    FormDiagnosticReservation {
        code: "PSC1087",
        meaning: "Invalid validator",
    },
    FormDiagnosticReservation {
        code: "PSC1088",
        meaning: "Cyclic validation dependency",
    },
    FormDiagnosticReservation {
        code: "PSC1089",
        meaning: "Invalid serialization",
    },
    FormDiagnosticReservation {
        code: "PSC1090",
        meaning: "Invalid reset",
    },
    FormDiagnosticReservation {
        code: "PSC1091",
        meaning: "Duplicate binding",
    },
    FormDiagnosticReservation {
        code: "PSC1092",
        meaning: "Nested forms",
    },
    FormDiagnosticReservation {
        code: "PSC1093",
        meaning: "Invalid ownership",
    },
    FormDiagnosticReservation {
        code: "PSC1094",
        meaning: "Invalid submit signature",
    },
    FormDiagnosticReservation {
        code: "PSC1095",
        meaning: "Invalid field scope",
    },
];

/// Project I18 Forms diagnostics exclusively from retained compiler products.
///
/// Candidate-only findings retain their candidate semantic identity in the
/// shared diagnostic subject slot; valid semantic subjects retain their exact
/// Form, Field, Rule, or binding identity. This projector never reparses
/// source or observes runtime state.
///
/// # Panics
///
/// Panics if a retained validation cycle references a missing retained rule
/// candidate, which violates the I6 graph invariant.
#[must_use]
#[allow(clippy::too_many_lines)]
pub fn collect_form_diagnostics(
    model: &crate::ApplicationSemanticModel,
) -> Vec<crate::ComponentDiagnostic> {
    use crate::{
        FormDeclarationViolation, FormFieldBindingViolation, FormFieldDeclarationViolation,
        SubmissionDeclarationViolation, SubmissionHostViolation,
    };

    let mut diagnostics = Vec::new();
    for candidate in model
        .components
        .iter()
        .flat_map(|component| &component.form_declaration_candidates)
    {
        if candidate
            .violations()
            .contains(&FormDeclarationViolation::DuplicateName)
        {
            diagnostics.push(form_diagnostic(
                "PSC1084",
                candidate.provenance.clone(),
                candidate.id.as_semantic_id().clone(),
            ));
        } else if !candidate.violations().is_empty() {
            diagnostics.push(form_diagnostic(
                "PSC1093",
                candidate.provenance.clone(),
                candidate.id.as_semantic_id().clone(),
            ));
        }
    }
    for candidate in model
        .components
        .iter()
        .flat_map(|component| &component.form_field_declaration_candidates)
    {
        let code = if candidate
            .violations
            .contains(&FormFieldDeclarationViolation::DuplicateName)
        {
            Some("PSC1085")
        } else if candidate.violations.iter().any(|violation| {
            matches!(
                violation,
                FormFieldDeclarationViolation::InvalidFormDesignator
                    | FormFieldDeclarationViolation::UnresolvedForm
                    | FormFieldDeclarationViolation::InvalidForm
                    | FormFieldDeclarationViolation::CrossComponentForm
                    | FormFieldDeclarationViolation::InvalidOwner
            )
        }) {
            Some("PSC1093")
        } else if candidate.violations.is_empty() {
            None
        } else {
            Some("PSC1095")
        };
        if let Some(code) = code {
            diagnostics.push(form_diagnostic(
                code,
                candidate.provenance.clone(),
                candidate.id.as_semantic_id().clone(),
            ));
        }
    }
    for candidate in &model.validation_rule_candidates {
        if !candidate.violations.is_empty() {
            diagnostics.push(form_diagnostic(
                "PSC1087",
                candidate.decorator_provenance.clone(),
                candidate.id.as_semantic_id().clone(),
            ));
        }
    }
    for cycle in &model.validation_graph.cycles {
        for candidate in &cycle.candidates {
            let provenance = model
                .validation_rule_candidates
                .iter()
                .find(|item| item.id == *candidate)
                .map(|item| item.decorator_provenance.clone())
                .expect("validation cycle candidate retained");
            diagnostics.push(form_diagnostic(
                "PSC1088",
                provenance,
                candidate.as_semantic_id().clone(),
            ));
        }
    }
    for candidate in &model.submissions.candidates {
        let code = if candidate.violations.iter().any(|violation| {
            matches!(
                violation,
                SubmissionDeclarationViolation::StaticMethod
                    | SubmissionDeclarationViolation::AsyncMethod
                    | SubmissionDeclarationViolation::ParameterizedMethod
                    | SubmissionDeclarationViolation::InvalidReturnType
            )
        }) {
            Some("PSC1094")
        } else if candidate.violations.is_empty() {
            None
        } else {
            Some("PSC1093")
        };
        if let Some(code) = code {
            diagnostics.push(form_diagnostic(
                code,
                candidate.provenance.clone(),
                candidate.id.as_semantic_id().clone(),
            ));
        }
    }
    for declaration in &model.serialization.declarations {
        if let Some(form) = &declaration.form {
            if !declaration.invoked
                || declaration.argument_count != 1
                || !matches!(
                    declaration.format.as_deref(),
                    Some("json" | "form-data" | "url-encoded")
                )
            {
                diagnostics.push(form_diagnostic(
                    "PSC1089",
                    declaration.provenance.clone(),
                    form.as_semantic_id().clone(),
                ));
            }
        }
    }
    for candidate in &model.form_field_binding_candidates {
        if candidate.violations.iter().any(|violation| {
            matches!(
                violation,
                FormFieldBindingViolation::DuplicateFieldControl
                    | FormFieldBindingViolation::DuplicateBindingAttribute
                    | FormFieldBindingViolation::CompetingValueBinding
                    | FormFieldBindingViolation::CompetingCheckedBinding
            )
        }) {
            diagnostics.push(form_diagnostic(
                "PSC1091",
                candidate.provenance.clone(),
                candidate.id.as_semantic_id().clone(),
            ));
        }
    }
    for candidate in &model.submission_host_candidates {
        let code = if candidate.violations.iter().any(|violation| {
            matches!(
                violation,
                SubmissionHostViolation::NestedHost
                    | SubmissionHostViolation::InvalidHostElement
                    | SubmissionHostViolation::DuplicateHostAttribute
                    | SubmissionHostViolation::InvalidHostExpression
            )
        }) {
            Some("PSC1092")
        } else if candidate
            .violations
            .contains(&SubmissionHostViolation::MissingSubmissionPlan)
        {
            Some("PSC1086")
        } else if candidate.violations.iter().any(|violation| {
            matches!(
                violation,
                SubmissionHostViolation::CrossComponentForm
                    | SubmissionHostViolation::ContainedControlForDifferentForm
            )
        }) {
            Some("PSC1093")
        } else {
            None
        };
        if let Some(code) = code {
            diagnostics.push(form_diagnostic(
                code,
                candidate.provenance.clone(),
                candidate.id.as_semantic_id().clone(),
            ));
        }
    }
    for form in model.forms.values() {
        if !model
            .reset
            .plans
            .contains_key(&crate::ResetPlanId::for_form(&form.id))
        {
            diagnostics.push(form_diagnostic(
                "PSC1090",
                form.provenance.clone(),
                form.id.as_semantic_id().clone(),
            ));
        }
    }
    diagnostics.sort_by(|left, right| {
        (
            left.code.as_str(),
            left.provenance
                .as_ref()
                .map(|item| (&item.path, item.span.start)),
            left.component_id.as_ref(),
        )
            .cmp(&(
                right.code.as_str(),
                right
                    .provenance
                    .as_ref()
                    .map(|item| (&item.path, item.span.start)),
                right.component_id.as_ref(),
            ))
    });
    diagnostics
        .dedup_by(|left, right| left.code == right.code && left.component_id == right.component_id);
    diagnostics
}

fn form_diagnostic(
    code: &str,
    provenance: crate::SourceProvenance,
    subject: crate::SemanticId,
) -> crate::ComponentDiagnostic {
    let message = FORM_DIAGNOSTIC_RESERVATIONS
        .iter()
        .find(|reservation| reservation.code == code)
        .expect("I18 reserved code")
        .meaning;
    crate::ComponentDiagnostic {
        code: code.to_string(),
        severity: crate::ComponentDiagnosticSeverity::Error,
        message: format!("{message}."),
        provenance: Some(provenance),
        effect_id: None,
        statement_id: None,
        context_declaration_candidate_id: None,
        context_id: None,
        provider_id: None,
        consumer_id: None,
        slot_id: None,
        invocation_id: None,
        component_instance_id: None,
        slot_binding_id: None,
        structural_region_id: None,
        component_id: Some(subject),
        provider_instance_id: None,
        consumer_instance_id: None,
        secondary_labels: Vec::new(),
    }
}

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
    fn projects_retained_form_candidates_without_runtime_evidence() {
        let model = crate::build_application_semantic_model(&presolve_parser::parse_file(
            "src/Duplicate.tsx",
            r#"@component("duplicate") class Duplicate { @form() profile!: Form; @form() profile!: Form; render() { return <main />; } }"#,
        ));
        let diagnostics = super::collect_form_diagnostics(&model);
        assert!(diagnostics
            .iter()
            .all(|diagnostic| diagnostic.code == "PSC1084"));
        assert_eq!(diagnostics.len(), 2);
        assert!(diagnostics
            .iter()
            .all(|diagnostic| diagnostic.component_id.is_some()));
    }

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

        assert_eq!(phase_h_codes.last(), Some(&"PSC1083"));
        assert_eq!(form_codes.first(), Some(&"PSC1084"));
        assert_eq!(form_codes.last(), Some(&"PSC1095"));
        assert_eq!(
            form_codes.len(),
            form_codes.iter().collect::<BTreeSet<_>>().len()
        );

        for (offset, reservation) in FORM_DIAGNOSTIC_RESERVATIONS.iter().enumerate() {
            assert_eq!(reservation.code, format!("PSC{}", 1084 + offset));
            assert!(!reservation.meaning.is_empty());
        }
    }

    #[test]
    fn phase_i_i17_updates_the_forms_inspection_schema_versions() {
        assert_eq!(RUNTIME_COMPONENT_ARTIFACT_SCHEMA_VERSION, 14);
        assert_eq!(RESUME_MANIFEST_SCHEMA_VERSION, 6);
        assert_eq!(TEMPLATE_MANIFEST_SCHEMA_VERSION, 5);
        assert_eq!(RUNTIME_CONTEXT_ARTIFACT_SCHEMA_VERSION, 2);
        assert_eq!(SEMANTIC_GRAPH_SCHEMA_VERSION, 6);
    }
}
