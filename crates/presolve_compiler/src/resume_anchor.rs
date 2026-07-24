//! J10 exact resume anchors and event marker plan.

use std::collections::{BTreeMap, BTreeSet};

use crate::{
    build_ordinary_template_instance_registry, build_resume_boundary_graph,
    build_resume_chunk_graph, ApplicationSemanticModel, OrdinaryTemplateBindingKind,
    OrdinaryTemplateTargetKind, ResumeAnchorId, ResumeBoundaryActivationProgram, ResumeBoundaryId,
    ResumeChunkId, ResumeEventId, TemplateInstanceTargetId,
};

pub const RESUME_ANCHOR_PLAN_VERSION: u32 = 1;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum ResumeAnchorKind {
    BoundaryRoot,
    TextTarget,
    ElementTarget,
    StructuralStart,
    StructuralEnd,
    FormControl,
    EventTarget,
}

impl ResumeAnchorKind {
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::BoundaryRoot => "boundary_root",
            Self::TextTarget => "text_target",
            Self::ElementTarget => "element_target",
            Self::StructuralStart => "structural_start",
            Self::StructuralEnd => "structural_end",
            Self::FormControl => "form_control",
            Self::EventTarget => "event_target",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum ResumeAnchorPlacement {
    ElementAttribute,
    TextTemplate,
    StructuralStartComment,
    StructuralEndComment,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResumeAnchorPlanRecord {
    pub anchor_id: ResumeAnchorId,
    pub kind: ResumeAnchorKind,
    pub boundary_id: ResumeBoundaryId,
    pub exact_target_id: String,
    pub marker_target_id: TemplateInstanceTargetId,
    pub placement: ResumeAnchorPlacement,
    pub required: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResumeEventMarkerRecord {
    pub resume_event_id: ResumeEventId,
    pub existing_dom_event_id: String,
    pub event_type: String,
    pub event_phase: String,
    pub target_id: TemplateInstanceTargetId,
    pub exact_target_anchor_id: ResumeAnchorId,
    pub interaction_boundary_id: ResumeBoundaryId,
    pub owner_boundary_id: ResumeBoundaryId,
    pub action_or_submit_program_id: String,
    pub chunk_id: ResumeChunkId,
    pub native_default_policy: String,
    pub propagation_policy: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResumeAnchorPlan {
    pub version: u32,
    pub anchors: Vec<ResumeAnchorPlanRecord>,
    pub events: Vec<ResumeEventMarkerRecord>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum ResumeAnchorIntegrityCode {
    MissingTarget,
    UnstableTarget,
    DuplicateAnchor,
    WrongKind,
    StructuralPairMismatch,
    OrderingOrOutputDrift,
}

impl ResumeAnchorIntegrityCode {
    #[must_use]
    pub const fn code(self) -> &'static str {
        match self {
            Self::MissingTarget => "PSASM1363",
            Self::UnstableTarget => "PSASM1364",
            Self::DuplicateAnchor => "PSASM1365",
            Self::WrongKind => "PSASM1366",
            Self::StructuralPairMismatch => "PSASM1367",
            Self::OrderingOrOutputDrift => "PSASM1368",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResumeAnchorIntegrityDiagnostic {
    pub code: ResumeAnchorIntegrityCode,
    pub anchor_id: Option<ResumeAnchorId>,
    pub message: String,
}

#[must_use]
#[allow(clippy::too_many_lines)]
pub fn build_resume_anchor_plan(model: &ApplicationSemanticModel) -> ResumeAnchorPlan {
    let ordinary = build_ordinary_template_instance_registry(model);
    let graph = build_resume_boundary_graph(model);
    let chunks = build_resume_chunk_graph(model);
    let binding_kinds =
        ordinary
            .bindings
            .iter()
            .fold(BTreeMap::<_, Vec<_>>::new(), |mut index, binding| {
                index
                    .entry(binding.target_id.clone())
                    .or_default()
                    .push(binding.binding_kind);
                index
            });
    let event_targets = ordinary
        .events
        .iter()
        .map(|event| event.target_id.clone())
        .collect::<BTreeSet<_>>();
    let form_controls = form_control_boundaries(model);
    let form_hosts = form_host_boundaries(model);
    let mut anchors = Vec::new();
    for target in &ordinary.targets {
        let component_boundary =
            ResumeBoundaryId::component_instance(&target.component_instance_id);
        if matches!(
            target.target_kind,
            OrdinaryTemplateTargetKind::ConditionalBoundary
                | OrdinaryTemplateTargetKind::ListBoundary
        ) {
            anchors.push(anchor_record(
                ResumeAnchorKind::StructuralStart,
                component_boundary.clone(),
                target.target_id.clone(),
                format!("{}#start", target.target_id),
                ResumeAnchorPlacement::StructuralStartComment,
            ));
            anchors.push(anchor_record(
                ResumeAnchorKind::StructuralEnd,
                component_boundary,
                target.target_id.clone(),
                format!("{}#end", target.target_id),
                ResumeAnchorPlacement::StructuralEndComment,
            ));
            continue;
        }
        let (kind, boundary, placement) =
            if let Some(boundary) = form_controls.get(&target.target_id) {
                (
                    ResumeAnchorKind::FormControl,
                    boundary.clone(),
                    ResumeAnchorPlacement::ElementAttribute,
                )
            } else if event_targets.contains(&target.target_id)
                || form_hosts.contains_key(&target.target_id)
            {
                (
                    ResumeAnchorKind::EventTarget,
                    form_hosts
                        .get(&target.target_id)
                        .cloned()
                        .unwrap_or(component_boundary),
                    ResumeAnchorPlacement::ElementAttribute,
                )
            } else if binding_kinds
                .get(&target.target_id)
                .is_some_and(|kinds| kinds.contains(&OrdinaryTemplateBindingKind::Text))
            {
                (
                    ResumeAnchorKind::TextTarget,
                    component_boundary,
                    ResumeAnchorPlacement::TextTemplate,
                )
            } else {
                (
                    ResumeAnchorKind::ElementTarget,
                    component_boundary,
                    ResumeAnchorPlacement::ElementAttribute,
                )
            };
        anchors.push(anchor_record(
            kind,
            boundary,
            target.target_id.clone(),
            target.target_id.to_string(),
            placement,
        ));
    }
    anchors.sort_by(|left, right| left.anchor_id.cmp(&right.anchor_id));
    let anchor_by_target = anchors
        .iter()
        .filter(|anchor| {
            matches!(
                anchor.placement,
                ResumeAnchorPlacement::ElementAttribute | ResumeAnchorPlacement::TextTemplate
            )
        })
        .map(|anchor| (anchor.marker_target_id.clone(), anchor.anchor_id.clone()))
        .collect::<BTreeMap<_, _>>();
    let mut events = Vec::new();
    for reference in &graph.activation_references {
        let (
            target,
            existing_dom_event_id,
            event_type,
            action_or_submit_program_id,
            native_default_policy,
        ) = match &reference.program {
            ResumeBoundaryActivationProgram::OrdinaryEvent {
                declaration_event,
                target,
                event_type,
                existing_program,
                ..
            } => (
                target.clone(),
                format!("{target}/event:{declaration_event}"),
                event_type.clone(),
                existing_program.to_string(),
                "preserve".to_string(),
            ),
            ResumeBoundaryActivationProgram::FormSubmit {
                submission_host,
                target,
                submission_plan,
                prevent_default,
                ..
            } => (
                target.clone(),
                format!("{target}/submit-event:{submission_host}"),
                "submit".to_string(),
                submission_plan.as_str().to_string(),
                if *prevent_default {
                    "prevent"
                } else {
                    "preserve"
                }
                .to_string(),
            ),
        };
        let Some(exact_target_anchor_id) = anchor_by_target.get(&target).cloned() else {
            continue;
        };
        let Some(chunk_id) = chunks
            .chunk_for_root(&reference.interaction_boundary)
            .map(|chunk| chunk.id.clone())
        else {
            continue;
        };
        events.push(ResumeEventMarkerRecord {
            resume_event_id: ResumeEventId::for_existing_event(&existing_dom_event_id),
            existing_dom_event_id,
            event_type,
            event_phase: "bubble".to_string(),
            target_id: target,
            exact_target_anchor_id,
            interaction_boundary_id: reference.interaction_boundary.clone(),
            owner_boundary_id: reference.owner_boundary.clone(),
            action_or_submit_program_id,
            chunk_id,
            native_default_policy,
            propagation_policy: "preserve".to_string(),
        });
    }
    events.sort_by(|left, right| left.resume_event_id.cmp(&right.resume_event_id));
    ResumeAnchorPlan {
        version: RESUME_ANCHOR_PLAN_VERSION,
        anchors,
        events,
    }
}

#[must_use]
#[allow(clippy::too_many_lines)]
pub fn validate_resume_anchor_plan(
    model: &ApplicationSemanticModel,
    plan: &ResumeAnchorPlan,
) -> Vec<ResumeAnchorIntegrityDiagnostic> {
    let canonical = build_resume_anchor_plan(model);
    let ordinary = build_ordinary_template_instance_registry(model);
    let known_targets = ordinary
        .targets
        .iter()
        .map(|target| target.target_id.clone())
        .collect::<BTreeSet<_>>();
    let mut diagnostics = Vec::new();
    let mut anchor_ids = BTreeSet::new();
    let mut marker_targets = BTreeSet::new();
    for anchor in &plan.anchors {
        if !known_targets.contains(&anchor.marker_target_id) {
            diagnostics.push(anchor_diagnostic(
                ResumeAnchorIntegrityCode::MissingTarget,
                Some(anchor.anchor_id.clone()),
                "resume anchor target does not exist",
            ));
        }
        if anchor.anchor_id
            != ResumeAnchorId::for_target(
                &anchor.boundary_id,
                anchor.kind.label(),
                &anchor.exact_target_id,
            )
        {
            diagnostics.push(anchor_diagnostic(
                ResumeAnchorIntegrityCode::UnstableTarget,
                Some(anchor.anchor_id.clone()),
                "resume anchor identity is not derived from its exact target",
            ));
        }
        if !anchor_ids.insert(anchor.anchor_id.clone())
            || (!matches!(
                anchor.placement,
                ResumeAnchorPlacement::StructuralStartComment
                    | ResumeAnchorPlacement::StructuralEndComment
            ) && !marker_targets.insert(anchor.marker_target_id.clone()))
        {
            diagnostics.push(anchor_diagnostic(
                ResumeAnchorIntegrityCode::DuplicateAnchor,
                Some(anchor.anchor_id.clone()),
                "resume anchor marker is duplicated",
            ));
        }
    }
    for target in &ordinary.targets {
        if matches!(
            target.target_kind,
            OrdinaryTemplateTargetKind::ConditionalBoundary
                | OrdinaryTemplateTargetKind::ListBoundary
        ) {
            let pair = plan
                .anchors
                .iter()
                .filter(|anchor| anchor.marker_target_id == target.target_id)
                .map(|anchor| anchor.kind)
                .collect::<BTreeSet<_>>();
            if pair
                != BTreeSet::from([
                    ResumeAnchorKind::StructuralStart,
                    ResumeAnchorKind::StructuralEnd,
                ])
            {
                diagnostics.push(anchor_diagnostic(
                    ResumeAnchorIntegrityCode::StructuralPairMismatch,
                    None,
                    "structural resume target lacks one exact start/end pair",
                ));
            }
        }
    }
    for anchor in &plan.anchors {
        if canonical
            .anchors
            .iter()
            .find(|record| record.anchor_id == anchor.anchor_id)
            .is_some_and(|record| record.kind != anchor.kind)
        {
            diagnostics.push(anchor_diagnostic(
                ResumeAnchorIntegrityCode::WrongKind,
                Some(anchor.anchor_id.clone()),
                "resume anchor kind does not match its compiler target",
            ));
        }
    }
    if plan.version != RESUME_ANCHOR_PLAN_VERSION || plan != &canonical {
        diagnostics.push(anchor_diagnostic(
            ResumeAnchorIntegrityCode::OrderingOrOutputDrift,
            None,
            "resume anchor/event plan drifted from canonical output",
        ));
    }
    diagnostics.sort_by(|left, right| {
        (left.code, &left.anchor_id, &left.message).cmp(&(
            right.code,
            &right.anchor_id,
            &right.message,
        ))
    });
    diagnostics.dedup();
    diagnostics
}

#[must_use]
pub fn validate_resume_marker_html(
    plan: &ResumeAnchorPlan,
    html: &str,
) -> Vec<ResumeAnchorIntegrityDiagnostic> {
    let mut diagnostics = Vec::new();
    for anchor in &plan.anchors {
        let marker = match anchor.placement {
            ResumeAnchorPlacement::ElementAttribute | ResumeAnchorPlacement::TextTemplate => {
                format!("data-presolve-r=\"{}\"", anchor.anchor_id)
            }
            ResumeAnchorPlacement::StructuralStartComment => {
                format!("<!--presolve-r-start:{}-->", anchor.anchor_id)
            }
            ResumeAnchorPlacement::StructuralEndComment => {
                format!("<!--presolve-r-end:{}-->", anchor.anchor_id)
            }
        };
        let count = html.matches(&marker).count();
        if count == 0 {
            diagnostics.push(anchor_diagnostic(
                ResumeAnchorIntegrityCode::MissingTarget,
                Some(anchor.anchor_id.clone()),
                "generated HTML is missing a required resume marker",
            ));
        } else if count > 1 {
            diagnostics.push(anchor_diagnostic(
                ResumeAnchorIntegrityCode::DuplicateAnchor,
                Some(anchor.anchor_id.clone()),
                "generated HTML duplicates a resume marker",
            ));
        }
    }
    for event in &plan.events {
        let marker = format!("data-presolve-e=\"{}\"", event.resume_event_id);
        let count = html.matches(&marker).count();
        if count == 0 {
            diagnostics.push(anchor_diagnostic(
                ResumeAnchorIntegrityCode::MissingTarget,
                Some(event.exact_target_anchor_id.clone()),
                "generated HTML is missing a required resume event marker",
            ));
        } else if count > 1 {
            diagnostics.push(anchor_diagnostic(
                ResumeAnchorIntegrityCode::DuplicateAnchor,
                Some(event.exact_target_anchor_id.clone()),
                "generated HTML duplicates a resume event marker",
            ));
        }
    }
    for start in plan
        .anchors
        .iter()
        .filter(|anchor| anchor.kind == ResumeAnchorKind::StructuralStart)
    {
        if let Some(end) = plan.anchors.iter().find(|anchor| {
            anchor.marker_target_id == start.marker_target_id
                && anchor.kind == ResumeAnchorKind::StructuralEnd
        }) {
            let start_marker = format!("<!--presolve-r-start:{}-->", start.anchor_id);
            let end_marker = format!("<!--presolve-r-end:{}-->", end.anchor_id);
            if html
                .find(&start_marker)
                .zip(html.find(&end_marker))
                .is_some_and(|(start_index, end_index)| start_index >= end_index)
            {
                diagnostics.push(anchor_diagnostic(
                    ResumeAnchorIntegrityCode::StructuralPairMismatch,
                    Some(start.anchor_id.clone()),
                    "structural resume markers are not start-before-end",
                ));
            }
        }
    }
    diagnostics.sort_by(|left, right| {
        (left.code, &left.anchor_id, &left.message).cmp(&(
            right.code,
            &right.anchor_id,
            &right.message,
        ))
    });
    diagnostics
}

fn anchor_record(
    kind: ResumeAnchorKind,
    boundary_id: ResumeBoundaryId,
    marker_target_id: TemplateInstanceTargetId,
    exact_target_id: String,
    placement: ResumeAnchorPlacement,
) -> ResumeAnchorPlanRecord {
    ResumeAnchorPlanRecord {
        anchor_id: ResumeAnchorId::for_target(&boundary_id, kind.label(), &exact_target_id),
        kind,
        boundary_id,
        exact_target_id,
        marker_target_id,
        placement,
        required: true,
    }
}

fn form_control_boundaries(
    model: &ApplicationSemanticModel,
) -> BTreeMap<TemplateInstanceTargetId, ResumeBoundaryId> {
    let mut boundaries = BTreeMap::new();
    for instance in model.optimized_form_ir.optimized.instances.values() {
        for binding in model
            .form_field_bindings
            .values()
            .filter(|binding| binding.form == instance.form)
        {
            boundaries.insert(
                TemplateInstanceTargetId::for_component_instance_template_entity(
                    instance.component_instance.clone(),
                    binding.control_entity.clone(),
                ),
                ResumeBoundaryId::form_instance(&instance.id),
            );
        }
    }
    boundaries
}

fn form_host_boundaries(
    model: &ApplicationSemanticModel,
) -> BTreeMap<TemplateInstanceTargetId, ResumeBoundaryId> {
    let mut boundaries = BTreeMap::new();
    for instance in model.optimized_form_ir.optimized.instances.values() {
        for host in model
            .submission_hosts
            .values()
            .filter(|host| host.form == instance.form)
        {
            boundaries.insert(
                TemplateInstanceTargetId::for_component_instance_template_entity(
                    instance.component_instance.clone(),
                    host.owner_template_element.clone(),
                ),
                ResumeBoundaryId::form_instance(&instance.id),
            );
        }
    }
    boundaries
}

fn anchor_diagnostic(
    code: ResumeAnchorIntegrityCode,
    anchor_id: Option<ResumeAnchorId>,
    message: &str,
) -> ResumeAnchorIntegrityDiagnostic {
    ResumeAnchorIntegrityDiagnostic {
        code,
        anchor_id,
        message: message.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn model(source: &str) -> ApplicationSemanticModel {
        crate::build_application_semantic_model(&presolve_parser::parse_file(
            "src/ResumeAnchors.tsx",
            source,
        ))
    }

    #[test]
    fn plans_exact_element_text_structural_form_and_event_markers() {
        let model = model(
            r#"
@component("x-marker") class Marker {
  value = state("ready");
  enabled = state(true);
  submitted = state(0);
  @form() @serialize("json") profile!: Form;
  @field(this.profile) name = "";
  @action() @submit(this.profile) save(): void { this.submitted += 1; }
  @action() toggle() { this.enabled = !this.enabled; }
  render() {
    return <form form={this.profile} onClick={() => this.toggle()}>
      <input field={this.name} />
      <span>{this.value}</span>
      {this.enabled ? <b>yes</b> : <i>no</i>}
    </form>;
  }
}"#,
        );
        let plan = build_resume_anchor_plan(&model);
        assert!(plan
            .anchors
            .iter()
            .any(|anchor| anchor.kind == ResumeAnchorKind::TextTarget));
        assert!(plan
            .anchors
            .iter()
            .any(|anchor| anchor.kind == ResumeAnchorKind::FormControl));
        assert!(plan
            .anchors
            .iter()
            .any(|anchor| anchor.kind == ResumeAnchorKind::StructuralStart));
        assert!(plan
            .anchors
            .iter()
            .any(|anchor| anchor.kind == ResumeAnchorKind::StructuralEnd));
        assert!(plan
            .anchors
            .iter()
            .any(|anchor| anchor.kind == ResumeAnchorKind::EventTarget));
        assert_eq!(plan.events.len(), 2);
        assert!(validate_resume_anchor_plan(&model, &plan).is_empty());
    }

    #[test]
    fn static_only_output_has_no_resume_markers() {
        let model =
            model(r#"@component("x-static") class Static { render() { return <p>static</p>; } }"#);
        let plan = build_resume_anchor_plan(&model);
        assert!(plan.anchors.is_empty());
        assert!(plan.events.is_empty());
        let html = crate::generate_ordinary_instance_html(&model);
        assert!(!html.contains("data-presolve-r"));
        assert!(!html.contains("data-presolve-e"));
        assert!(!html.contains("presolve-r-start:"));
        assert!(validate_resume_marker_html(&plan, &html).is_empty());
    }

    #[test]
    fn rejects_missing_unstable_duplicate_wrong_and_unpaired_anchors() {
        let model = model(
            r#"@component("x-marker") class Marker { value = state(true); render() { return <p>{this.value ? <b>yes</b> : <i>no</i>}</p>; } }"#,
        );
        let canonical = build_resume_anchor_plan(&model);
        let mut duplicate = canonical.clone();
        duplicate.anchors.push(duplicate.anchors[0].clone());
        assert!(validate_resume_anchor_plan(&model, &duplicate)
            .iter()
            .any(|diagnostic| { diagnostic.code == ResumeAnchorIntegrityCode::DuplicateAnchor }));

        let mut malformed = canonical;
        malformed.anchors[0].exact_target_id.push_str("-drift");
        malformed.anchors[0].kind = ResumeAnchorKind::ElementTarget;
        malformed.anchors.pop();
        let diagnostics = validate_resume_anchor_plan(&model, &malformed);
        assert!(diagnostics
            .iter()
            .any(|diagnostic| { diagnostic.code == ResumeAnchorIntegrityCode::UnstableTarget }));
        assert!(diagnostics.iter().any(|diagnostic| {
            diagnostic.code == ResumeAnchorIntegrityCode::WrongKind
                || diagnostic.code == ResumeAnchorIntegrityCode::StructuralPairMismatch
        }));
        assert!(diagnostics.iter().any(|diagnostic| {
            diagnostic.code == ResumeAnchorIntegrityCode::OrderingOrOutputDrift
        }));
    }

    #[test]
    fn reserves_the_complete_j10_integrity_range() {
        assert_eq!(
            [
                ResumeAnchorIntegrityCode::MissingTarget,
                ResumeAnchorIntegrityCode::UnstableTarget,
                ResumeAnchorIntegrityCode::DuplicateAnchor,
                ResumeAnchorIntegrityCode::WrongKind,
                ResumeAnchorIntegrityCode::StructuralPairMismatch,
                ResumeAnchorIntegrityCode::OrderingOrOutputDrift,
            ]
            .map(ResumeAnchorIntegrityCode::code),
            [
                "PSASM1363",
                "PSASM1364",
                "PSASM1365",
                "PSASM1366",
                "PSASM1367",
                "PSASM1368",
            ]
        );
    }

    #[test]
    fn generated_html_and_marker_plan_are_exact_and_byte_deterministic() {
        let model = model(
            r#"@component("x-marker") class Marker { count = state(0); @action() increment() { this.count++; } render() { return <button onClick={() => this.increment()}>{this.count}</button>; } }"#,
        );
        let plan = build_resume_anchor_plan(&model);
        let html = crate::generate_ordinary_instance_html(&model);
        assert!(validate_resume_marker_html(&plan, &html).is_empty());
        assert_eq!(html, crate::generate_ordinary_instance_html(&model));
        let missing = html.replacen("data-presolve-r=", "data-missing-r=", 1);
        assert!(validate_resume_marker_html(&plan, &missing)
            .iter()
            .any(|diagnostic| diagnostic.code == ResumeAnchorIntegrityCode::MissingTarget));
    }
}
