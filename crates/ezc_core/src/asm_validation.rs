use crate::application_semantic_model::ApplicationSemanticModel;
use crate::semantic_id::SemanticOwner;
use crate::{
    EffectOperationClassification, EffectRenderBoundary, EffectValidation, SemanticTypeId,
    EFFECT_CAPABILITY_REGISTRY,
};

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
        let source_provenance_matches = model.provenance(&reference.source)
            == Some(&reference.provenance)
            || model
                .expression_graph
                .nodes_for(&reference.source)
                .iter()
                .any(|node| node.provenance == reference.provenance);
        if !source_provenance_matches {
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
    validate_effect_statement_types(model, &mut diagnostics);
    validate_effect_execution_plan(model, &mut diagnostics);

    diagnostics
}

fn validate_effect_execution_plan(
    model: &ApplicationSemanticModel,
    diagnostics: &mut Vec<AsmValidationDiagnostic>,
) {
    let plan = model.effect_execution_plan();
    if plan.initial.render_boundary != Some(EffectRenderBoundary::AfterInitialRender) {
        diagnostics.push(AsmValidationDiagnostic {
            code: "EZASM1120".to_string(),
            message: "initial effect execution plan is missing the after-initial-render boundary"
                .to_string(),
        });
    }
    validate_effect_execution_entry(
        model,
        &model.effect_trigger_plan.initial_effects,
        &plan.initial.required_computed,
        &plan.initial.prerequisite_batches,
        &plan.initial.effect_batches,
        &plan.initial.unplanned_effects,
        "initial",
        diagnostics,
    );
    for action in &plan.actions {
        let Some(trigger) = model
            .effect_trigger_plan
            .action_batch_triggers
            .iter()
            .find(|trigger| trigger.action_batch == action.action_batch)
        else {
            diagnostics.push(AsmValidationDiagnostic {
                code: "EZASM1121".to_string(),
                message: format!(
                    "effect execution plan references untriggered action batch `{}`",
                    action.action_batch
                ),
            });
            continue;
        };
        validate_effect_execution_entry(
            model,
            &trigger.effects,
            &action.required_computed,
            &action.prerequisite_batches,
            &action.effect_batches,
            &action.unplanned_effects,
            action.action_batch.as_str(),
            diagnostics,
        );
    }
}

#[allow(clippy::too_many_arguments)]
fn validate_effect_execution_entry(
    model: &ApplicationSemanticModel,
    eligible_effects: &[crate::SemanticId],
    required_computed: &[crate::SemanticId],
    prerequisite_batches: &[crate::EffectComputedPrerequisiteBatch],
    effect_batches: &[crate::EffectExecutionBatch],
    unplanned_effects: &[crate::UnplannedEffect],
    context: &str,
    diagnostics: &mut Vec<AsmValidationDiagnostic>,
) {
    let required = required_computed
        .iter()
        .collect::<std::collections::BTreeSet<_>>();
    if required.len() != required_computed.len()
        || required_computed
            .iter()
            .any(|computed| !model.computed_values.contains_key(computed))
    {
        diagnostics.push(AsmValidationDiagnostic {
            code: "EZASM1122".to_string(),
            message: format!("effect plan `{context}` has invalid required computed membership"),
        });
    }
    let mut batch_membership = std::collections::BTreeSet::new();
    let mut prior_source_index = None;
    for batch in prerequisite_batches {
        let expected = model
            .computed_evaluation_plan
            .update_batches
            .get(batch.source_batch_index as usize)
            .map(|source| {
                source
                    .iter()
                    .filter_map(|id| {
                        model
                            .computed_values
                            .keys()
                            .find(|computed| computed.as_str() == id)
                    })
                    .filter(|computed| required.contains(computed))
                    .cloned()
                    .collect::<Vec<_>>()
            });
        if expected.as_ref() != Some(&batch.computed)
            || prior_source_index.is_some_and(|prior| prior >= batch.source_batch_index)
        {
            diagnostics.push(AsmValidationDiagnostic {
                code: "EZASM1123".to_string(),
                message: format!(
                    "effect plan `{context}` does not preserve canonical computed batch membership"
                ),
            });
        }
        prior_source_index = Some(batch.source_batch_index);
        batch_membership.extend(batch.computed.iter());
    }
    if batch_membership != required {
        diagnostics.push(AsmValidationDiagnostic {
            code: "EZASM1124".to_string(),
            message: format!(
                "effect plan `{context}` required computed values do not match prerequisite batches"
            ),
        });
    }

    let scheduled = effect_batches
        .iter()
        .flat_map(|batch| &batch.effects)
        .collect::<std::collections::BTreeSet<_>>();
    let unplanned = unplanned_effects
        .iter()
        .map(|record| &record.effect)
        .collect::<std::collections::BTreeSet<_>>();
    let eligible = eligible_effects
        .iter()
        .collect::<std::collections::BTreeSet<_>>();
    let covered = scheduled
        .union(&unplanned)
        .copied()
        .collect::<std::collections::BTreeSet<_>>();
    if covered != eligible
        || scheduled.intersection(&unplanned).next().is_some()
        || scheduled.iter().any(|effect| {
            model
                .effects
                .get(*effect)
                .is_none_or(|effect| effect.validation != EffectValidation::Valid)
        })
    {
        diagnostics.push(AsmValidationDiagnostic {
            code: "EZASM1125".to_string(),
            message: format!("effect plan `{context}` has invalid effect eligibility membership"),
        });
    }
    if effect_batches.iter().enumerate().any(|(index, batch)| {
        batch.index != u32::try_from(index).expect("effect scheduler batch index should fit u32")
    }) {
        diagnostics.push(AsmValidationDiagnostic {
            code: "EZASM1126".to_string(),
            message: format!("effect plan `{context}` has non-contiguous terminal batch indexes"),
        });
    }
}

fn validate_effect_statement_types(
    model: &ApplicationSemanticModel,
    diagnostics: &mut Vec<AsmValidationDiagnostic>,
) {
    for (statement, record) in &model.semantic_types.effect_statements {
        if statement != &record.statement {
            diagnostics.push(AsmValidationDiagnostic {
                code: "EZASM1110".to_string(),
                message: format!(
                    "effect statement type record key `{statement}` does not match statement `{}`",
                    record.statement
                ),
            });
        }
        let Some(canonical_statement) = model.effect_statement(statement) else {
            diagnostics.push(AsmValidationDiagnostic {
                code: "EZASM1111".to_string(),
                message: format!(
                    "effect statement type record references missing statement `{statement}`"
                ),
            });
            continue;
        };
        if canonical_statement.provenance != record.provenance {
            diagnostics.push(AsmValidationDiagnostic {
                code: "EZASM1112".to_string(),
                message: format!(
                    "effect statement type record for `{statement}` has inconsistent provenance"
                ),
            });
        }
        let operation_exists = record.capability_operation.is_some_and(|operation_id| {
            EFFECT_CAPABILITY_REGISTRY
                .definitions()
                .iter()
                .flat_map(|definition| definition.operations)
                .any(|operation| operation.id == operation_id)
        });
        if record.operation_classification == EffectOperationClassification::RecognizedCapability
            && !operation_exists
        {
            diagnostics.push(AsmValidationDiagnostic {
                code: "EZASM1113".to_string(),
                message: format!(
                    "recognized effect statement `{statement}` has no registry capability operation"
                ),
            });
        }
        if record.operation_classification != EffectOperationClassification::RecognizedCapability
            && record.capability_operation.is_some()
        {
            diagnostics.push(AsmValidationDiagnostic {
                code: "EZASM1114".to_string(),
                message: format!(
                    "non-capability effect statement `{statement}` unexpectedly names a registry operation"
                ),
            });
        }
    }
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
        let subject_provenance = model
            .provenance(&assignment.subject)
            .or_else(|| model.expression_provenance(&assignment.subject));
        if subject_provenance.is_none() {
            diagnostics.push(AsmValidationDiagnostic {
                code: "EZASM1103".to_string(),
                message: format!(
                    "semantic type assignment references missing subject `{}`",
                    assignment.subject
                ),
            });
        }
        if subject_provenance != Some(&assignment.provenance) {
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
