use std::collections::BTreeSet;

use crate::{
    ApplicationSemanticModel, ComponentDiagnostic, ComponentDiagnosticSeverity,
    ContextDeclarationCandidateKind, ContextDeclarationStatus, ContextDeclarationViolation,
};

/// Projects only retained Context declaration candidates into the frozen G18
/// catalog.  This module deliberately has no parser or source-text dependency.
#[must_use]
pub fn collect_context_diagnostics(model: &ApplicationSemanticModel) -> Vec<ComponentDiagnostic> {
    let mut emitted_duplicate_groups = BTreeSet::new();
    let mut diagnostics = Vec::new();
    for candidate in model.context_declaration_candidates().invalid_candidates() {
        let ContextDeclarationStatus::Invalid(violations) = &candidate.status else {
            continue;
        };
        let primary = violations.first();
        let Some(violation) = primary else {
            continue;
        };
        let code = match violation {
            ContextDeclarationViolation::UnresolvedContextDesignator => "EZC1055",
            ContextDeclarationViolation::DuplicateProvider => "EZC1056",
            _ => match candidate.authored.kind {
                ContextDeclarationCandidateKind::Context => "EZC1052",
                ContextDeclarationCandidateKind::Provider => "EZC1053",
                ContextDeclarationCandidateKind::Consumer => "EZC1054",
            },
        };
        if code == "EZC1056" {
            let Some(designator) = &candidate.authored.context_designator else {
                continue;
            };
            let group = (
                candidate.authored.owner_component.clone(),
                designator.component_symbol.clone(),
                designator.context_member.clone(),
            );
            if !emitted_duplicate_groups.insert(group) {
                continue;
            }
        }
        let provenance = match violation {
            ContextDeclarationViolation::StaticDeclarationUnsupported => {
                candidate.authored.static_modifier_provenance.as_ref()
            }
            ContextDeclarationViolation::UnsupportedInitializer
            | ContextDeclarationViolation::ForbiddenInitializer
            | ContextDeclarationViolation::MissingInitializer => {
                candidate.authored.initializer_provenance.as_ref()
            }
            ContextDeclarationViolation::ContextDesignatorUnsupported
            | ContextDeclarationViolation::UnresolvedContextDesignator => candidate
                .authored
                .context_designator
                .as_ref()
                .map(|designator| &designator.provenance),
            _ => Some(&candidate.authored.decorator_provenance),
        }
        .or(Some(&candidate.authored.provenance));
        diagnostics.push(ComponentDiagnostic {
            code: code.to_string(),
            severity: ComponentDiagnosticSeverity::Error,
            message: message(code, candidate.authored.field_name.as_deref()),
            provenance: provenance.cloned(),
            effect_id: None,
            statement_id: None,
            secondary_labels: Vec::new(),
        });
    }
    diagnostics.sort_by(|left, right| {
        (
            left.provenance
                .as_ref()
                .map(|value| (&value.path, value.span.start)),
            &left.code,
        )
            .cmp(&(
                right
                    .provenance
                    .as_ref()
                    .map(|value| (&value.path, value.span.start)),
                &right.code,
            ))
    });
    diagnostics
}

fn message(code: &str, field: Option<&str>) -> String {
    let subject = field.map_or("declaration".to_string(), |field| {
        format!("declaration `{field}`")
    });
    match code {
        "EZC1052" => format!("Invalid Context {subject}."),
        "EZC1053" => format!("Invalid Provider {subject}."),
        "EZC1054" => format!("Invalid Consumer {subject}."),
        "EZC1055" => format!("Unresolved Context designator for {subject}."),
        "EZC1056" => "Duplicate Provider declarations target the same Context.".to_string(),
        _ => unreachable!("frozen Context diagnostic code"),
    }
}
