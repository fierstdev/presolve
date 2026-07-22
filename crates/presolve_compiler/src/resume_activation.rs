//! J4 canonical activation policy planning over J2/J3 products.

use std::collections::{BTreeMap, BTreeSet};

use crate::{
    build_resume_boundary_graph, build_resume_liveness_plan, ApplicationSemanticModel,
    FormInstanceId, ResumeBoundaryActivationProgram, ResumeBoundaryGraph, ResumeBoundaryId,
    ResumeBoundaryKind, ResumeExistingSlot, SourceProvenance,
};

pub const RESUME_ACTIVATION_PLAN_VERSION: u32 = 1;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum ResumeActivationPolicy {
    Eager,
    Visible,
    Interaction,
    Manual,
    None,
}

impl ResumeActivationPolicy {
    #[must_use]
    pub const fn precedence(self) -> u8 {
        match self {
            Self::Eager => 4,
            Self::Visible => 3,
            Self::Interaction => 2,
            Self::Manual => 1,
            Self::None => 0,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum ResumeActivationPrerequisite {
    ApplicationBootstrap,
    RuntimeRegistryInstallation,
    EventDelegationInstallation,
    PostRestoreRecomputation(ResumeExistingSlot),
    ImmediateFormRuntime(FormInstanceId),
    ExactInteraction(ResumeBoundaryId),
    RequiredBoundary(ResumeBoundaryId),
    RetainedSlot(ResumeExistingSlot),
    RecomputableSlot(ResumeExistingSlot),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResumeActivationPolicyDecision {
    pub boundary: ResumeBoundaryId,
    pub policy: ResumeActivationPolicy,
    pub prerequisites: Vec<ResumeActivationPrerequisite>,
    pub source_authority: String,
    pub provenance: SourceProvenance,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum ResumeActivationBlockReason {
    UnknownBoundary,
    MissingInteractionReference,
    RequiredBoundaryBlocked,
    UnsupportedLazyEventPayload,
    NoValidEagerFallback,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResumeActivationBlock {
    pub boundary: ResumeBoundaryId,
    pub reason: ResumeActivationBlockReason,
    pub provenance: SourceProvenance,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResumeActivationPlan {
    pub version: u32,
    pub decisions: Vec<ResumeActivationPolicyDecision>,
    pub blocks: Vec<ResumeActivationBlock>,
    pub decision_index: BTreeMap<ResumeBoundaryId, usize>,
}

impl ResumeActivationPlan {
    #[must_use]
    pub fn decision(&self, boundary: &ResumeBoundaryId) -> Option<&ResumeActivationPolicyDecision> {
        self.decision_index
            .get(boundary)
            .and_then(|index| self.decisions.get(*index))
    }

    #[must_use]
    pub fn boundaries_with_policy(&self, policy: ResumeActivationPolicy) -> Vec<&ResumeBoundaryId> {
        self.decisions
            .iter()
            .filter(|decision| decision.policy == policy)
            .map(|decision| &decision.boundary)
            .collect()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum ResumeActivationIntegrityCode {
    MissingOrDuplicatePolicy,
    InvalidPrerequisite,
    UnknownEventOrBoundary,
    InvalidPolicyAuthority,
    UnsupportedLazyPayload,
    OrderingOrIndexDrift,
}

impl ResumeActivationIntegrityCode {
    #[must_use]
    pub const fn code(self) -> &'static str {
        match self {
            Self::MissingOrDuplicatePolicy => "PSASM1337",
            Self::InvalidPrerequisite => "PSASM1338",
            Self::UnknownEventOrBoundary => "PSASM1339",
            Self::InvalidPolicyAuthority => "PSASM1340",
            Self::UnsupportedLazyPayload => "PSASM1341",
            Self::OrderingOrIndexDrift => "PSASM1342",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResumeActivationIntegrityDiagnostic {
    pub code: ResumeActivationIntegrityCode,
    pub boundary: Option<ResumeBoundaryId>,
    pub message: String,
}

#[must_use]
pub fn build_resume_activation_plan(model: &ApplicationSemanticModel) -> ResumeActivationPlan {
    let boundaries = build_resume_boundary_graph(model);
    let liveness = build_resume_liveness_plan(model);
    build_resume_activation_plan_from_products(&boundaries, &liveness)
}

#[allow(clippy::too_many_lines)]
fn build_resume_activation_plan_from_products(
    boundaries: &ResumeBoundaryGraph,
    liveness: &crate::ResumeLivenessPlan,
) -> ResumeActivationPlan {
    let blocked_boundaries = boundaries
        .blocks
        .iter()
        .filter_map(|block| block.candidate_boundary.clone())
        .collect::<BTreeSet<_>>();
    let activation_by_boundary = boundaries
        .activation_references
        .iter()
        .map(|reference| (reference.interaction_boundary.clone(), reference))
        .collect::<BTreeMap<_, _>>();
    let mut decisions = Vec::new();
    let mut blocks = Vec::new();

    for boundary in &boundaries.boundaries {
        let (policy, mut prerequisites, source_authority) = match boundary.kind {
            ResumeBoundaryKind::ApplicationRoot => (
                ResumeActivationPolicy::Eager,
                vec![
                    ResumeActivationPrerequisite::ApplicationBootstrap,
                    ResumeActivationPrerequisite::RuntimeRegistryInstallation,
                    ResumeActivationPrerequisite::EventDelegationInstallation,
                ],
                "phase-j-application-bootstrap",
            ),
            ResumeBoundaryKind::FormInstance => {
                let crate::ResumeBoundaryOwner::FormInstance(form) = &boundary.owner else {
                    blocks.push(ResumeActivationBlock {
                        boundary: boundary.id.clone(),
                        reason: ResumeActivationBlockReason::UnknownBoundary,
                        provenance: boundary.provenance.clone(),
                    });
                    continue;
                };
                (
                    ResumeActivationPolicy::Eager,
                    vec![ResumeActivationPrerequisite::ImmediateFormRuntime(
                        form.clone(),
                    )],
                    "phase-i-immediate-form-runtime",
                )
            }
            ResumeBoundaryKind::Interaction => {
                let Some(reference) = activation_by_boundary.get(&boundary.id) else {
                    blocks.push(ResumeActivationBlock {
                        boundary: boundary.id.clone(),
                        reason: ResumeActivationBlockReason::MissingInteractionReference,
                        provenance: boundary.provenance.clone(),
                    });
                    continue;
                };
                if reference
                    .required_boundaries
                    .iter()
                    .any(|required| blocked_boundaries.contains(required))
                {
                    blocks.push(ResumeActivationBlock {
                        boundary: boundary.id.clone(),
                        reason: ResumeActivationBlockReason::RequiredBoundaryBlocked,
                        provenance: boundary.provenance.clone(),
                    });
                    continue;
                }
                let mut prerequisites = vec![ResumeActivationPrerequisite::ExactInteraction(
                    boundary.id.clone(),
                )];
                prerequisites.extend(
                    reference
                        .required_boundaries
                        .iter()
                        .cloned()
                        .map(ResumeActivationPrerequisite::RequiredBoundary),
                );
                prerequisites.extend(
                    reference
                        .required_retained_slots
                        .iter()
                        .cloned()
                        .map(ResumeActivationPrerequisite::RetainedSlot),
                );
                let unsupported_payload = match &reference.program {
                    ResumeBoundaryActivationProgram::OrdinaryEvent { event_type, .. } => {
                        event_type != "click"
                    }
                    ResumeBoundaryActivationProgram::FormSubmit { .. } => false,
                };
                if unsupported_payload {
                    (
                        ResumeActivationPolicy::Eager,
                        prerequisites,
                        "phase-j-unsupported-lazy-payload-eager-fallback",
                    )
                } else {
                    (
                        ResumeActivationPolicy::Interaction,
                        prerequisites,
                        "phase-h-i-exact-interaction",
                    )
                }
            }
            ResumeBoundaryKind::ComponentInstance | ResumeBoundaryKind::StructuralRegion => {
                let recomputable = liveness
                    .recomputable
                    .iter()
                    .filter(|record| record.slot.boundary_candidate.as_ref() == Some(&boundary.id))
                    .map(|record| {
                        ResumeActivationPrerequisite::PostRestoreRecomputation(
                            record.slot.existing_slot.clone(),
                        )
                    })
                    .collect::<Vec<_>>();
                if recomputable.is_empty() {
                    (
                        ResumeActivationPolicy::None,
                        Vec::new(),
                        "phase-j-no-independent-executable-work",
                    )
                } else {
                    (
                        ResumeActivationPolicy::Eager,
                        recomputable,
                        "phase-j-post-restore-recomputation",
                    )
                }
            }
        };
        prerequisites.sort();
        prerequisites.dedup();
        decisions.push(ResumeActivationPolicyDecision {
            boundary: boundary.id.clone(),
            policy,
            prerequisites,
            source_authority: source_authority.to_string(),
            provenance: boundary.provenance.clone(),
        });
    }

    decisions.sort_by(|left, right| left.boundary.cmp(&right.boundary));
    blocks
        .sort_by(|left, right| (&left.boundary, left.reason).cmp(&(&right.boundary, right.reason)));
    let decision_index = decisions
        .iter()
        .enumerate()
        .map(|(index, decision)| (decision.boundary.clone(), index))
        .collect();
    ResumeActivationPlan {
        version: RESUME_ACTIVATION_PLAN_VERSION,
        decisions,
        blocks,
        decision_index,
    }
}

#[must_use]
pub fn validate_resume_activation_plan(
    model: &ApplicationSemanticModel,
    plan: &ResumeActivationPlan,
) -> Vec<ResumeActivationIntegrityDiagnostic> {
    let canonical = build_resume_activation_plan(model);
    let boundaries = build_resume_boundary_graph(model);
    let boundary_ids = boundaries
        .boundaries
        .iter()
        .map(|boundary| boundary.id.clone())
        .collect::<BTreeSet<_>>();
    let mut diagnostics = Vec::new();
    let mut seen = BTreeSet::new();
    for (index, decision) in plan.decisions.iter().enumerate() {
        if !seen.insert(decision.boundary.clone()) {
            diagnostics.push(integrity(
                ResumeActivationIntegrityCode::MissingOrDuplicatePolicy,
                Some(decision.boundary.clone()),
                "resume boundary received more than one activation policy",
            ));
        }
        if !boundary_ids.contains(&decision.boundary) {
            diagnostics.push(integrity(
                ResumeActivationIntegrityCode::UnknownEventOrBoundary,
                Some(decision.boundary.clone()),
                "activation policy references an unknown resume boundary",
            ));
        }
        if plan.decision_index.get(&decision.boundary) != Some(&index) {
            diagnostics.push(integrity(
                ResumeActivationIntegrityCode::OrderingOrIndexDrift,
                Some(decision.boundary.clone()),
                "activation decision index disagrees with canonical boundary order",
            ));
        }
        if matches!(
            decision.policy,
            ResumeActivationPolicy::Visible | ResumeActivationPolicy::Manual
        ) {
            diagnostics.push(integrity(
                ResumeActivationIntegrityCode::InvalidPolicyAuthority,
                Some(decision.boundary.clone()),
                "Visible or Manual policy lacks an earlier frozen source authority",
            ));
        }
        if decision.policy == ResumeActivationPolicy::None && !decision.prerequisites.is_empty() {
            diagnostics.push(integrity(
                ResumeActivationIntegrityCode::InvalidPrerequisite,
                Some(decision.boundary.clone()),
                "None policy cannot retain executable prerequisites",
            ));
        }
    }
    for boundary in &boundary_ids {
        let classified = plan.decision(boundary).is_some()
            || plan.blocks.iter().any(|block| &block.boundary == boundary);
        if !classified {
            diagnostics.push(integrity(
                ResumeActivationIntegrityCode::MissingOrDuplicatePolicy,
                Some(boundary.clone()),
                "resume boundary has neither one policy decision nor one activation block",
            ));
        }
    }
    if plan.version != RESUME_ACTIVATION_PLAN_VERSION || plan != &canonical {
        diagnostics.push(integrity(
            ResumeActivationIntegrityCode::OrderingOrIndexDrift,
            None,
            "activation plan drifted from canonical prerequisites, precedence, or order",
        ));
    }
    diagnostics.sort_by(|left, right| {
        (left.code, &left.boundary, left.message.as_str()).cmp(&(
            right.code,
            &right.boundary,
            right.message.as_str(),
        ))
    });
    diagnostics.dedup();
    diagnostics
}

fn integrity(
    code: ResumeActivationIntegrityCode,
    boundary: Option<ResumeBoundaryId>,
    message: &str,
) -> ResumeActivationIntegrityDiagnostic {
    ResumeActivationIntegrityDiagnostic {
        code,
        boundary,
        message: message.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn assigns_eager_interaction_and_none_without_visible_or_manual_heuristics() {
        let model = crate::build_application_semantic_model(&presolve_parser::parse_file(
            "src/Activation.tsx",
            r#"
@component("x-activation-child") class Child {
  count = state(1);
  @computed() get doubled() { return this.count * 2; }
  @action() increment() { this.count++; }
  render() { return <button onClick={() => this.increment()}>{this.count}</button>; }
}
@component("x-activation-page") @route("/") class Page {
  render() { return <><Child /><aside /></>; }
}"#,
        ));
        let plan = build_resume_activation_plan(&model);
        assert!(validate_resume_activation_plan(&model, &plan).is_empty());
        assert!(!plan
            .boundaries_with_policy(ResumeActivationPolicy::Eager)
            .is_empty());
        assert_eq!(
            plan.boundaries_with_policy(ResumeActivationPolicy::Interaction)
                .len(),
            1
        );
        assert!(plan
            .boundaries_with_policy(ResumeActivationPolicy::Visible)
            .is_empty());
        assert!(plan
            .boundaries_with_policy(ResumeActivationPolicy::Manual)
            .is_empty());
        assert!(plan
            .boundaries_with_policy(ResumeActivationPolicy::None)
            .iter()
            .all(|boundary| plan.decision(boundary).unwrap().prerequisites.is_empty()));
    }

    #[test]
    fn form_runtime_is_eager_while_submit_activation_is_interaction_scoped() {
        let model = crate::build_application_semantic_model(&presolve_parser::parse_file(
            "src/FormActivation.tsx",
            r#"@component("x-form-activation") @route("/") class X { @form() @serialize("json") form!: Form; @field(this.form) value = ""; @action() @submit(this.form) save(): void {} render() { return <form form={this.form}><input field={this.value}/></form>; } }"#,
        ));
        let plan = build_resume_activation_plan(&model);
        assert_eq!(
            plan.boundaries_with_policy(ResumeActivationPolicy::Eager)
                .iter()
                .filter(|boundary| {
                    plan.decision(boundary)
                        .unwrap()
                        .prerequisites
                        .iter()
                        .any(|prerequisite| {
                            matches!(
                                prerequisite,
                                ResumeActivationPrerequisite::ImmediateFormRuntime(_)
                            )
                        })
                })
                .count(),
            1
        );
        assert_eq!(
            plan.boundaries_with_policy(ResumeActivationPolicy::Interaction)
                .len(),
            1
        );
    }

    #[test]
    fn fixed_precedence_is_not_a_cost_heuristic() {
        let policies = [
            ResumeActivationPolicy::Manual,
            ResumeActivationPolicy::Interaction,
            ResumeActivationPolicy::Visible,
            ResumeActivationPolicy::Eager,
            ResumeActivationPolicy::None,
        ];
        assert_eq!(
            policies
                .into_iter()
                .max_by_key(|policy| policy.precedence()),
            Some(ResumeActivationPolicy::Eager)
        );
    }

    #[test]
    fn reserves_the_complete_j4_integrity_range() {
        assert_eq!(
            [
                ResumeActivationIntegrityCode::MissingOrDuplicatePolicy,
                ResumeActivationIntegrityCode::InvalidPrerequisite,
                ResumeActivationIntegrityCode::UnknownEventOrBoundary,
                ResumeActivationIntegrityCode::InvalidPolicyAuthority,
                ResumeActivationIntegrityCode::UnsupportedLazyPayload,
                ResumeActivationIntegrityCode::OrderingOrIndexDrift,
            ]
            .map(ResumeActivationIntegrityCode::code),
            [
                "PSASM1337",
                "PSASM1338",
                "PSASM1339",
                "PSASM1340",
                "PSASM1341",
                "PSASM1342",
            ]
        );
    }
}
