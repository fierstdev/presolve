use std::collections::{BTreeMap, BTreeSet};

use crate::{
    ApplicationSemanticModel, ComponentDiagnostic, ComponentDiagnosticSeverity,
    DiagnosticSecondaryLabel, Effect, EffectId, EffectSemanticViolation,
    EffectSemanticViolationKind, EffectStatementId, EffectStatementKind, ExpressionNodeKind,
    SemanticId, EFFECT_CAPABILITY_REGISTRY,
};

/// Stable public catalog for compiler-projected effect diagnostics.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum EffectDiagnosticCode {
    InvalidDeclaration,
    UnsupportedBody,
    UnresolvedReference,
    ReactiveStateMutation,
    InvalidComponentInvocation,
    AsyncOrCleanupUnsupported,
    UnknownCapability,
    CapabilitySignature,
    CapabilityBoundary,
    CapabilitySerialization,
    UnavailableComputedPrerequisite,
}

impl EffectDiagnosticCode {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::InvalidDeclaration => "PSC1041",
            Self::UnsupportedBody => "PSC1042",
            Self::UnresolvedReference => "PSC1043",
            Self::ReactiveStateMutation => "PSC1044",
            Self::InvalidComponentInvocation => "PSC1045",
            Self::AsyncOrCleanupUnsupported => "PSC1046",
            Self::UnknownCapability => "PSC1047",
            Self::CapabilitySignature => "PSC1048",
            Self::CapabilityBoundary => "PSC1049",
            Self::CapabilitySerialization => "PSC1050",
            Self::UnavailableComputedPrerequisite => "PSC1051",
        }
    }
}

/// Projects immutable F2-F5 and F9 facts into shared compiler diagnostics.
///
/// This deliberately consumes already-classified statements, compatibility
/// records, validation violations, and scheduler records. It never rewalks
/// source or attempts to rediscover an effect capability or dependency.
#[must_use]
pub fn collect_effect_diagnostics(model: &ApplicationSemanticModel) -> Vec<ComponentDiagnostic> {
    let mut diagnostics = Vec::new();
    for effect in model.effects.values() {
        let mut violations = effect.semantic_violations.iter().collect::<Vec<_>>();
        violations.sort_by(|left, right| {
            (
                statement_position(model, effect, left.statement.as_ref()),
                cascade_precedence(left.kind),
                left.provenance.path.as_path(),
                left.provenance.span.start,
                violation_code(left.kind),
            )
                .cmp(&(
                    statement_position(model, effect, right.statement.as_ref()),
                    cascade_precedence(right.kind),
                    right.provenance.path.as_path(),
                    right.provenance.span.start,
                    violation_code(right.kind),
                ))
        });

        // One root diagnostic per statement (or declaration). F5 has already
        // classified every violation; this only applies the documented cascade
        // precedence to prevent follow-on facts from obscuring the root cause.
        let mut reported_subjects = BTreeSet::new();
        for violation in violations {
            if !reported_subjects.insert(violation.statement.clone()) {
                continue;
            }
            diagnostics.push(effect_violation_diagnostic(model, effect, violation));
        }
    }

    // F9 can report the same unplanned effect from its initial plan and more
    // than one action plan. Coalesce those plan memberships into one effect
    // diagnostic with deterministic computed evidence.
    let mut unavailable = BTreeMap::<SemanticId, BTreeSet<SemanticId>>::new();
    for unplanned in model
        .effect_execution_plan
        .initial
        .unplanned_effects
        .iter()
        .chain(
            model
                .effect_execution_plan
                .actions
                .iter()
                .flat_map(|plan| &plan.unplanned_effects),
        )
    {
        unavailable
            .entry(unplanned.effect.clone())
            .or_default()
            .extend(unplanned.computed_dependencies.iter().cloned());
    }
    for (effect_id, computed) in unavailable {
        let Some(effect) = model.effects.get(&effect_id) else {
            continue;
        };
        let computed = computed.into_iter().collect::<Vec<_>>();
        diagnostics.push(unavailable_prerequisite_diagnostic(
            model, effect, &computed,
        ));
    }

    diagnostics.sort_by(|left, right| diagnostic_order(left).cmp(&diagnostic_order(right)));
    diagnostics.dedup();
    diagnostics
}

fn effect_violation_diagnostic(
    model: &ApplicationSemanticModel,
    effect: &Effect,
    violation: &EffectSemanticViolation,
) -> ComponentDiagnostic {
    let code = violation_code(violation.kind);
    let statement_id = canonical_statement_id(model, effect, violation.statement.as_ref());
    ComponentDiagnostic {
        code: code.as_str().to_string(),
        severity: ComponentDiagnosticSeverity::Error,
        message: violation_message(model, effect, violation, code),
        provenance: Some(violation.provenance.clone()),
        effect_id: Some(EffectId::from_semantic(&effect.id)),
        statement_id,
        context_declaration_candidate_id: None,
        context_id: None,
        provider_id: None,
        consumer_id: None,
        slot_id: None,
        invocation_id: None,
        component_instance_id: None,
        slot_binding_id: None,
        structural_region_id: None,
        component_id: None,
        provider_instance_id: None,
        consumer_instance_id: None,
        secondary_labels: normalized_labels(secondary_labels(model, effect, violation)),
    }
}

fn unavailable_prerequisite_diagnostic(
    model: &ApplicationSemanticModel,
    effect: &Effect,
    computed: &[SemanticId],
) -> ComponentDiagnostic {
    let names = computed
        .iter()
        .filter_map(|id| {
            model
                .computed_values
                .get(id)
                .map(|value| value.name.as_str())
        })
        .collect::<Vec<_>>();
    let subject = if names.is_empty() {
        "a required computed value".to_string()
    } else {
        format!(
            "computed value{} `{}`",
            if names.len() == 1 { "" } else { "s" },
            names.join("`, `")
        )
    };
    let labels = computed
        .iter()
        .filter_map(|id| {
            model
                .provenance(id)
                .map(|provenance| DiagnosticSecondaryLabel {
                    provenance: provenance.clone(),
                    message: "Required computed value is unavailable for effect scheduling."
                        .to_string(),
                })
        })
        .collect();
    ComponentDiagnostic {
        code: EffectDiagnosticCode::UnavailableComputedPrerequisite
            .as_str()
            .to_string(),
        severity: ComponentDiagnosticSeverity::Error,
        message: format!(
            "Effect `{}` cannot be scheduled because {subject} has no executable evaluation plan.",
            effect.name
        ),
        provenance: Some(effect.provenance.clone()),
        effect_id: Some(EffectId::from_semantic(&effect.id)),
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
        component_id: None,
        provider_instance_id: None,
        consumer_instance_id: None,
        secondary_labels: normalized_labels(labels),
    }
}

fn canonical_statement_id(
    model: &ApplicationSemanticModel,
    effect: &Effect,
    statement: Option<&SemanticId>,
) -> Option<EffectStatementId> {
    statement
        .filter(|id| {
            model
                .effect_statements
                .get(*id)
                .is_some_and(|candidate| candidate.owner == effect.id)
        })
        .map(EffectStatementId::from_semantic)
}

fn statement_position(
    model: &ApplicationSemanticModel,
    effect: &Effect,
    statement: Option<&SemanticId>,
) -> usize {
    statement
        .and_then(|statement| {
            model
                .effect_bodies
                .get(&effect.id)
                .and_then(|body| body.statements.iter().position(|id| id == statement))
        })
        .unwrap_or(usize::MAX)
}

fn cascade_precedence(kind: EffectSemanticViolationKind) -> u8 {
    match kind {
        EffectSemanticViolationKind::UnsupportedStatement => 1,
        EffectSemanticViolationKind::Async
        | EffectSemanticViolationKind::ReactiveStateMutation
        | EffectSemanticViolationKind::ActionInvocation
        | EffectSemanticViolationKind::EffectInvocation
        | EffectSemanticViolationKind::ComponentMethodInvocation
        | EffectSemanticViolationKind::ValueReturn => 2,
        EffectSemanticViolationKind::UnresolvedComponentCall
        | EffectSemanticViolationKind::UnresolvedComponentAssignment => 3,
        EffectSemanticViolationKind::UnknownExternalCapability => 4,
        EffectSemanticViolationKind::CapabilityBoundary => 5,
        EffectSemanticViolationKind::CapabilitySignature => 6,
        EffectSemanticViolationKind::CapabilitySerialization => 7,
    }
}

fn violation_code(kind: EffectSemanticViolationKind) -> EffectDiagnosticCode {
    match kind {
        EffectSemanticViolationKind::UnsupportedStatement => EffectDiagnosticCode::UnsupportedBody,
        EffectSemanticViolationKind::UnresolvedComponentAssignment => {
            EffectDiagnosticCode::UnresolvedReference
        }
        EffectSemanticViolationKind::ReactiveStateMutation => {
            EffectDiagnosticCode::ReactiveStateMutation
        }
        EffectSemanticViolationKind::ActionInvocation
        | EffectSemanticViolationKind::EffectInvocation
        | EffectSemanticViolationKind::ComponentMethodInvocation
        | EffectSemanticViolationKind::UnresolvedComponentCall => {
            EffectDiagnosticCode::InvalidComponentInvocation
        }
        EffectSemanticViolationKind::Async | EffectSemanticViolationKind::ValueReturn => {
            EffectDiagnosticCode::AsyncOrCleanupUnsupported
        }
        EffectSemanticViolationKind::UnknownExternalCapability => {
            EffectDiagnosticCode::UnknownCapability
        }
        EffectSemanticViolationKind::CapabilitySignature => {
            EffectDiagnosticCode::CapabilitySignature
        }
        EffectSemanticViolationKind::CapabilityBoundary => EffectDiagnosticCode::CapabilityBoundary,
        EffectSemanticViolationKind::CapabilitySerialization => {
            EffectDiagnosticCode::CapabilitySerialization
        }
    }
}

fn violation_message(
    model: &ApplicationSemanticModel,
    effect: &Effect,
    violation: &EffectSemanticViolation,
    code: EffectDiagnosticCode,
) -> String {
    let statement = violation
        .statement
        .as_ref()
        .and_then(|id| model.effect_statements.get(id));
    match code {
        EffectDiagnosticCode::ReactiveStateMutation => format!(
            "Effect `{}` writes reactive state{}. Effects synchronize reactive values with external systems and cannot mutate component state. Move this write into an `@action()` method.",
            effect.name,
            reactive_state_name(model, statement).map_or_else(String::new, |name| format!(" `{name}`")),
        ),
        EffectDiagnosticCode::InvalidComponentInvocation => format!(
            "Effect `{}` {}. Effects cannot invoke component actions, effects, or methods.",
            effect.name,
            component_invocation_description(model, effect, statement),
        ),
        EffectDiagnosticCode::AsyncOrCleanupUnsupported => match violation.kind {
            EffectSemanticViolationKind::Async => format!(
                "Effect `{}` is async. Effects are synchronous and do not support async or cleanup semantics.",
                effect.name
            ),
            _ => format!(
                "Effect `{}` returns a value. Effects do not support cleanup callbacks or value-return semantics.",
                effect.name
            ),
        },
        EffectDiagnosticCode::UnknownCapability => format!(
            "Effect `{}` uses unknown capability `{}`. Effects may call only compiler-recognized capabilities from registry version 1.",
            effect.name,
            capability_path(model, statement).unwrap_or_else(|| "<dynamic capability>".to_string()),
        ),
        EffectDiagnosticCode::CapabilitySignature => format!(
            "Effect `{}` uses a capability with an incompatible signature or value type{}.",
            effect.name,
            recognized_capability_suffix(model, violation.statement.as_ref()),
        ),
        EffectDiagnosticCode::CapabilityBoundary => format!(
            "Effect `{}` uses a capability incompatible with the client execution boundary{}.",
            effect.name,
            recognized_capability_suffix(model, violation.statement.as_ref()),
        ),
        EffectDiagnosticCode::CapabilitySerialization => format!(
            "Effect `{}` passes a value incompatible with the capability serialization policy{}.",
            effect.name,
            recognized_capability_suffix(model, violation.statement.as_ref()),
        ),
        EffectDiagnosticCode::UnsupportedBody => format!(
            "Effect `{}` contains an unsupported effect-body statement.",
            effect.name
        ),
        EffectDiagnosticCode::UnresolvedReference => format!(
            "Effect `{}` assigns an unresolved component reference.",
            effect.name
        ),
        EffectDiagnosticCode::InvalidDeclaration => format!(
            "Effect `{}` has an invalid declaration.",
            effect.name
        ),
        EffectDiagnosticCode::UnavailableComputedPrerequisite => unreachable!(),
    }
}

fn secondary_labels(
    model: &ApplicationSemanticModel,
    effect: &Effect,
    violation: &EffectSemanticViolation,
) -> Vec<DiagnosticSecondaryLabel> {
    let Some(statement_id) = violation.statement.as_ref() else {
        return Vec::new();
    };
    let Some(statement) = model.effect_statements.get(statement_id) else {
        return Vec::new();
    };
    let Some(component_id) = effect.owner.entity_id() else {
        return Vec::new();
    };
    let Some(component) = model
        .components
        .iter()
        .find(|component| component.id == *component_id)
    else {
        return Vec::new();
    };
    match (&violation.kind, &statement.kind) {
        (
            EffectSemanticViolationKind::ReactiveStateMutation,
            EffectStatementKind::ExternalMemberAssignment { target, .. },
        ) => this_member_name(model, target)
            .and_then(|name| {
                component
                    .state_fields
                    .iter()
                    .find(|field| field.name == name)
                    .and_then(|field| {
                        model
                            .provenance(&field.id)
                            .map(|provenance| DiagnosticSecondaryLabel {
                                provenance: provenance.clone(),
                                message: format!("Reactive state `{name}` is declared here."),
                            })
                    })
            })
            .into_iter()
            .collect(),
        (
            EffectSemanticViolationKind::ActionInvocation
            | EffectSemanticViolationKind::EffectInvocation
            | EffectSemanticViolationKind::ComponentMethodInvocation,
            EffectStatementKind::CapabilityCall { callee, .. },
        ) => this_member_name(model, callee)
            .and_then(|name| {
                component
                    .methods
                    .iter()
                    .find(|method| method.name == name)
                    .and_then(|method| {
                        model
                            .provenance(&method.id)
                            .map(|provenance| DiagnosticSecondaryLabel {
                                provenance: provenance.clone(),
                                message: format!("Component method `{name}` is declared here."),
                            })
                    })
            })
            .into_iter()
            .collect(),
        _ => Vec::new(),
    }
}

fn reactive_state_name(
    model: &ApplicationSemanticModel,
    statement: Option<&crate::EffectStatement>,
) -> Option<String> {
    match statement?.kind {
        EffectStatementKind::ExternalMemberAssignment { ref target, .. } => {
            this_member_name(model, target)
        }
        _ => None,
    }
}

fn component_invocation_description(
    model: &ApplicationSemanticModel,
    effect: &Effect,
    statement: Option<&crate::EffectStatement>,
) -> String {
    let name = match statement.map(|statement| &statement.kind) {
        Some(EffectStatementKind::CapabilityCall { callee, .. }) => this_member_name(model, callee),
        _ => None,
    };
    let category = effect
        .owner
        .entity_id()
        .and_then(|owner| {
            model
                .components
                .iter()
                .find(|component| component.id == *owner)
        })
        .and_then(|component| {
            name.as_ref()
                .and_then(|name| component.methods.iter().find(|method| method.name == *name))
        })
        .map_or("an unresolved component method", |method| {
            if method.is_action() {
                "an `@action()` method"
            } else if method.is_effect() {
                "an `@effect()` method"
            } else {
                "a component method"
            }
        });
    name.map_or_else(
        || format!("invokes {category}"),
        |name| format!("invokes {category} `{name}`"),
    )
}

fn recognized_capability_suffix(
    model: &ApplicationSemanticModel,
    statement: Option<&SemanticId>,
) -> String {
    let operation = statement
        .and_then(|id| model.semantic_types.effect_statements.get(id))
        .and_then(|record| record.capability_operation)
        .and_then(|id| EFFECT_CAPABILITY_REGISTRY.operation(id));
    operation.map_or_else(String::new, |operation| {
        format!(" for `{}`", operation.static_path.0)
    })
}

fn capability_path(
    model: &ApplicationSemanticModel,
    statement: Option<&crate::EffectStatement>,
) -> Option<String> {
    match statement?.kind {
        EffectStatementKind::ExternalMemberAssignment { ref target, .. } => {
            static_path(model, target)
        }
        EffectStatementKind::CapabilityCall { ref callee, .. } => static_path(model, callee),
        _ => None,
    }
}

fn static_path(model: &ApplicationSemanticModel, id: &SemanticId) -> Option<String> {
    match &model.expression(id)?.kind {
        ExpressionNodeKind::Identifier(name) => Some(name.clone()),
        ExpressionNodeKind::ThisMember { name } => Some(format!("this.{name}")),
        ExpressionNodeKind::MemberAccess {
            object, property, ..
        } => Some(format!("{}.{}", static_path(model, object)?, property)),
        _ => None,
    }
}

fn this_member_name(model: &ApplicationSemanticModel, id: &SemanticId) -> Option<String> {
    match &model.expression(id)?.kind {
        ExpressionNodeKind::ThisMember { name } => Some(name.clone()),
        _ => None,
    }
}

fn normalized_labels(mut labels: Vec<DiagnosticSecondaryLabel>) -> Vec<DiagnosticSecondaryLabel> {
    labels.sort_by(|left, right| {
        (
            left.provenance.path.as_path(),
            left.provenance.span.start,
            left.provenance.span.end,
            left.message.as_str(),
        )
            .cmp(&(
                right.provenance.path.as_path(),
                right.provenance.span.start,
                right.provenance.span.end,
                right.message.as_str(),
            ))
    });
    labels.dedup();
    labels
}

fn diagnostic_order(diagnostic: &ComponentDiagnostic) -> (&str, &std::path::Path, usize, &str) {
    (
        diagnostic.effect_id.as_ref().map_or("", EffectId::as_str),
        diagnostic.provenance.as_ref().map_or_else(
            || std::path::Path::new(""),
            |provenance| provenance.path.as_path(),
        ),
        diagnostic
            .provenance
            .as_ref()
            .map_or(usize::MAX, |provenance| provenance.span.start),
        diagnostic.code.as_str(),
    )
}

#[cfg(test)]
mod tests {
    use crate::{
        build_application_semantic_model, collect_effect_diagnostics, EffectSemanticViolationKind,
        UnplannedEffect, UnplannedEffectReason,
    };

    #[test]
    fn projects_ordered_effect_validation_facts_with_shared_diagnostic_metadata() {
        let parsed = presolve_parser::parse_file(
            "src/EffectDiagnostics.tsx",
            r#"
@component("x-effect-diagnostics")
class EffectDiagnostics extends Component {
  count = state(1);
  @action() increment() { this.count += 1; }
  @effect() invalid() { this.count = 0; this.increment(); analytics.track(this.count); }
}
"#,
        );
        let model = build_application_semantic_model(&parsed);
        let diagnostics = collect_effect_diagnostics(&model);
        let codes = diagnostics
            .iter()
            .map(|diagnostic| diagnostic.code.as_str())
            .collect::<Vec<_>>();
        assert_eq!(codes, vec!["PSC1044", "PSC1045", "PSC1047"]);
        assert!(diagnostics.iter().all(|diagnostic| {
            diagnostic.effect_id.is_some()
                && diagnostic.statement_id.is_some()
                && diagnostic.severity == crate::ComponentDiagnosticSeverity::Error
        }));
        assert_eq!(diagnostics[0].secondary_labels.len(), 1);
        assert_eq!(diagnostics[1].secondary_labels.len(), 1);
    }

    #[test]
    fn maps_each_existing_f5_violation_category_to_one_stable_public_code() {
        let cases = [
            (EffectSemanticViolationKind::UnsupportedStatement, "PSC1042"),
            (
                EffectSemanticViolationKind::UnresolvedComponentAssignment,
                "PSC1043",
            ),
            (
                EffectSemanticViolationKind::ReactiveStateMutation,
                "PSC1044",
            ),
            (EffectSemanticViolationKind::ActionInvocation, "PSC1045"),
            (EffectSemanticViolationKind::EffectInvocation, "PSC1045"),
            (
                EffectSemanticViolationKind::ComponentMethodInvocation,
                "PSC1045",
            ),
            (
                EffectSemanticViolationKind::UnresolvedComponentCall,
                "PSC1045",
            ),
            (EffectSemanticViolationKind::Async, "PSC1046"),
            (EffectSemanticViolationKind::ValueReturn, "PSC1046"),
            (
                EffectSemanticViolationKind::UnknownExternalCapability,
                "PSC1047",
            ),
            (EffectSemanticViolationKind::CapabilitySignature, "PSC1048"),
            (EffectSemanticViolationKind::CapabilityBoundary, "PSC1049"),
            (
                EffectSemanticViolationKind::CapabilitySerialization,
                "PSC1050",
            ),
        ];

        for (kind, code) in cases {
            assert_eq!(super::violation_code(kind).as_str(), code);
        }
    }

    #[test]
    fn projects_f9_unavailable_prerequisites_once_with_computed_evidence() {
        let parsed = presolve_parser::parse_file(
            "src/UnplannedEffect.tsx",
            r#"
@component("x-unplanned-effect")
class UnplannedEffect extends Component {
  count = state(1);
  @action() increment() { this.count += 1; }
  @computed() get total() { return this.count; }
  @effect() report() { console.log(this.total); }
  render() { return <p />; }
}
"#,
        );
        let mut model = build_application_semantic_model(&parsed);
        let component = &model.components[0];
        let effect = component.id.effect("report");
        let computed = component.id.computed("total");
        model.effect_execution_plan.initial.unplanned_effects = vec![UnplannedEffect {
            effect: effect.clone(),
            reason: UnplannedEffectReason::UnavailableComputedPrerequisite,
            computed_dependencies: vec![computed.clone()],
        }];
        // Repeating the same F9 fact in an action plan cannot duplicate the
        // root scheduling diagnostic.
        model.effect_execution_plan.actions[0].unplanned_effects = vec![UnplannedEffect {
            effect: effect.clone(),
            reason: UnplannedEffectReason::UnavailableComputedPrerequisite,
            computed_dependencies: vec![computed],
        }];

        let diagnostics = collect_effect_diagnostics(&model);
        assert_eq!(diagnostics.len(), 1);
        let diagnostic = &diagnostics[0];
        assert_eq!(diagnostic.code, "PSC1051");
        assert_eq!(
            diagnostic.effect_id.as_ref().map(crate::EffectId::as_str),
            Some(effect.as_str())
        );
        assert!(diagnostic.statement_id.is_none());
        assert_eq!(diagnostic.secondary_labels.len(), 1);
    }
}
