//! J3 canonical resume boundary graph over existing Phase H/I and J2 products.

use std::collections::{BTreeMap, BTreeSet};

use crate::{
    build_ordinary_template_instance_registry, build_resume_liveness_plan,
    ApplicationSemanticModel, ComponentInstanceId, ComponentInstanceStatus, ComponentRootId,
    ComponentStructuralRegionId, FormInstanceId, ResumeBoundaryId, ResumeBoundaryKind,
    ResumeExistingSlot, ResumeLivenessBlockReason, SemanticId, SourceProvenance, SubmissionHostId,
    SubmissionPlanId, TemplateInstanceTargetId,
};

pub const RESUME_BOUNDARY_GRAPH_VERSION: u32 = 1;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum ResumeBoundaryOwner {
    ApplicationRoot(ComponentRootId),
    ComponentInstance(ComponentInstanceId),
    StructuralRegion {
        component_instance: ComponentInstanceId,
        region: ComponentStructuralRegionId,
    },
    FormInstance(FormInstanceId),
    Interaction(ResumeBoundaryActivationIdentity),
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum ResumeBoundaryActivationIdentity {
    OrdinaryEvent {
        component_instance: ComponentInstanceId,
        declaration_event: SemanticId,
    },
    FormSubmit {
        component_instance: ComponentInstanceId,
        submission_host: SubmissionHostId,
        form_instance: FormInstanceId,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResumeBoundary {
    pub id: ResumeBoundaryId,
    pub kind: ResumeBoundaryKind,
    pub owner: ResumeBoundaryOwner,
    pub ownership_parent: Option<ResumeBoundaryId>,
    pub owned_slots: Vec<ResumeExistingSlot>,
    pub provenance: SourceProvenance,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct ResumeBoundaryOwnershipEdge {
    pub parent: ResumeBoundaryId,
    pub child: ResumeBoundaryId,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ResumeBoundaryActivationProgram {
    OrdinaryEvent {
        declaration_event: SemanticId,
        target: TemplateInstanceTargetId,
        event_type: String,
        handler_method: SemanticId,
        action_batch: SemanticId,
        existing_program: SemanticId,
    },
    FormSubmit {
        submission_host: SubmissionHostId,
        target: TemplateInstanceTargetId,
        form_instance: FormInstanceId,
        submission_plan: SubmissionPlanId,
        submit_action: SemanticId,
        action_batch: SemanticId,
        serialization_plan: crate::SerializationPlanId,
        prevent_default: bool,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResumeBoundaryActivationReference {
    pub interaction_boundary: ResumeBoundaryId,
    pub owner_boundary: ResumeBoundaryId,
    pub required_boundaries: Vec<ResumeBoundaryId>,
    pub required_retained_slots: Vec<ResumeExistingSlot>,
    pub program: ResumeBoundaryActivationProgram,
    pub provenance: SourceProvenance,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum ResumeBoundaryBlockSource {
    ComponentInstance {
        instance: ComponentInstanceId,
        reason: crate::BlockedComponentInstanceReason,
    },
    LivenessSlot {
        slot: ResumeExistingSlot,
        reason: ResumeLivenessBlockReason,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResumeBoundaryBlock {
    pub candidate_boundary: Option<ResumeBoundaryId>,
    pub source: ResumeBoundaryBlockSource,
    pub provenance: SourceProvenance,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResumeBoundaryGraph {
    pub version: u32,
    pub boundaries: Vec<ResumeBoundary>,
    pub ownership_edges: Vec<ResumeBoundaryOwnershipEdge>,
    pub activation_references: Vec<ResumeBoundaryActivationReference>,
    pub blocks: Vec<ResumeBoundaryBlock>,
    pub boundary_index: BTreeMap<ResumeBoundaryId, usize>,
    pub children_by_parent: BTreeMap<ResumeBoundaryId, Vec<ResumeBoundaryId>>,
}

impl ResumeBoundaryGraph {
    #[must_use]
    pub fn boundary(&self, id: &ResumeBoundaryId) -> Option<&ResumeBoundary> {
        self.boundary_index
            .get(id)
            .and_then(|index| self.boundaries.get(*index))
    }

    #[must_use]
    pub fn parent(&self, id: &ResumeBoundaryId) -> Option<&ResumeBoundaryId> {
        self.boundary(id)
            .and_then(|boundary| boundary.ownership_parent.as_ref())
    }

    #[must_use]
    pub fn children(&self, id: &ResumeBoundaryId) -> &[ResumeBoundaryId] {
        self.children_by_parent.get(id).map_or(&[], Vec::as_slice)
    }

    #[must_use]
    pub fn activations_for_owner(
        &self,
        owner: &ResumeBoundaryId,
    ) -> Vec<&ResumeBoundaryActivationReference> {
        self.activation_references
            .iter()
            .filter(|reference| &reference.owner_boundary == owner)
            .collect()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum ResumeBoundaryIntegrityCode {
    DuplicateBoundary,
    InvalidOwner,
    MissingOrMultipleParent,
    Cycle,
    Unreachable,
    Reciprocity,
    PhaseCorrespondence,
    ProvenanceDrift,
    OrderingOrIndexDrift,
}

impl ResumeBoundaryIntegrityCode {
    #[must_use]
    pub const fn code(self) -> &'static str {
        match self {
            Self::DuplicateBoundary => "PSASM1328",
            Self::InvalidOwner => "PSASM1329",
            Self::MissingOrMultipleParent => "PSASM1330",
            Self::Cycle => "PSASM1331",
            Self::Unreachable => "PSASM1332",
            Self::Reciprocity => "PSASM1333",
            Self::PhaseCorrespondence => "PSASM1334",
            Self::ProvenanceDrift => "PSASM1335",
            Self::OrderingOrIndexDrift => "PSASM1336",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResumeBoundaryIntegrityDiagnostic {
    pub code: ResumeBoundaryIntegrityCode,
    pub boundary: Option<ResumeBoundaryId>,
    pub message: String,
}

#[derive(Debug, Clone)]
struct PendingBoundary {
    boundary: ResumeBoundary,
    depth: usize,
}

#[must_use]
#[allow(clippy::too_many_lines)]
pub fn build_resume_boundary_graph(model: &ApplicationSemanticModel) -> ResumeBoundaryGraph {
    let liveness = build_resume_liveness_plan(model);
    let ordinary = build_ordinary_template_instance_registry(model);
    let mut pending = Vec::new();
    let mut ownership_edges = Vec::new();
    let mut activation_references = Vec::new();
    let mut blocks = Vec::new();

    for root in model.component_instance_plan.roots.values() {
        pending.push(PendingBoundary {
            boundary: ResumeBoundary {
                id: ResumeBoundaryId::application_root(&root.id),
                kind: ResumeBoundaryKind::ApplicationRoot,
                owner: ResumeBoundaryOwner::ApplicationRoot(root.id.clone()),
                ownership_parent: None,
                owned_slots: liveness_slots(
                    &liveness,
                    &ResumeBoundaryId::application_root(&root.id),
                ),
                provenance: root.provenance.clone(),
            },
            depth: 0,
        });
    }

    for instance in model.component_instance_plan.instances.values() {
        if !matches!(
            instance.status,
            ComponentInstanceStatus::Planned | ComponentInstanceStatus::StructuralTemplate
        ) {
            continue;
        }
        let id = ResumeBoundaryId::component_instance(&instance.id);
        let parent = if instance.status == ComponentInstanceStatus::StructuralTemplate {
            instance
                .parent_instance
                .as_ref()
                .zip(instance.structural_region.as_ref())
                .map_or_else(
                    || ResumeBoundaryId::application_root(&instance.owner_root),
                    |(parent, region)| ResumeBoundaryId::structural_region(parent, region),
                )
        } else {
            instance.parent_instance.as_ref().map_or_else(
                || ResumeBoundaryId::application_root(&instance.owner_root),
                ResumeBoundaryId::component_instance,
            )
        };
        pending.push(PendingBoundary {
            boundary: ResumeBoundary {
                id: id.clone(),
                kind: ResumeBoundaryKind::ComponentInstance,
                owner: ResumeBoundaryOwner::ComponentInstance(instance.id.clone()),
                ownership_parent: Some(parent.clone()),
                owned_slots: liveness_slots(&liveness, &id),
                provenance: instance.provenance.clone(),
            },
            depth: instance.depth * 2 + 1,
        });
        ownership_edges.push(ResumeBoundaryOwnershipEdge { parent, child: id });
    }

    let mut structural_regions = BTreeMap::new();
    for instance in model.component_instance_plan.instances.values() {
        let (Some(parent), Some(region)) = (
            instance.parent_instance.as_ref(),
            instance.structural_region.as_ref(),
        ) else {
            continue;
        };
        structural_regions
            .entry((parent.clone(), region.clone()))
            .or_insert_with(|| {
                (
                    instance.depth.saturating_sub(1),
                    instance.provenance.clone(),
                )
            });
    }
    for ((owner, region), (owner_depth, provenance)) in structural_regions {
        let id = ResumeBoundaryId::structural_region(&owner, &region);
        let parent = ResumeBoundaryId::component_instance(&owner);
        pending.push(PendingBoundary {
            boundary: ResumeBoundary {
                id: id.clone(),
                kind: ResumeBoundaryKind::StructuralRegion,
                owner: ResumeBoundaryOwner::StructuralRegion {
                    component_instance: owner,
                    region,
                },
                ownership_parent: Some(parent.clone()),
                owned_slots: liveness_slots(&liveness, &id),
                provenance,
            },
            depth: owner_depth * 2 + 2,
        });
        ownership_edges.push(ResumeBoundaryOwnershipEdge { parent, child: id });
    }

    for instance in model.optimized_form_ir.optimized.instances.values() {
        let id = ResumeBoundaryId::form_instance(&instance.id);
        let parent = ResumeBoundaryId::component_instance(&instance.component_instance);
        let depth = model
            .component_instance_plan
            .instances
            .get(&instance.component_instance)
            .map_or(0, |component| component.depth);
        pending.push(PendingBoundary {
            boundary: ResumeBoundary {
                id: id.clone(),
                kind: ResumeBoundaryKind::FormInstance,
                owner: ResumeBoundaryOwner::FormInstance(instance.id.clone()),
                ownership_parent: Some(parent.clone()),
                owned_slots: liveness_slots(&liveness, &id),
                provenance: model.forms[&instance.form].provenance.clone(),
            },
            depth: depth * 2 + 2,
        });
        ownership_edges.push(ResumeBoundaryOwnershipEdge { parent, child: id });
    }

    for event in &ordinary.events {
        let Some(action_batch) = event.action_batch_id.clone() else {
            continue;
        };
        let id = ResumeBoundaryId::interaction(
            &event.component_instance_id,
            &event.declaration_event_id,
        );
        let owner_boundary = ResumeBoundaryId::component_instance(&event.component_instance_id);
        let required_boundaries = vec![owner_boundary.clone()];
        let required_retained_slots =
            retained_slots_for_boundaries(&liveness, &required_boundaries);
        pending.push(PendingBoundary {
            boundary: ResumeBoundary {
                id: id.clone(),
                kind: ResumeBoundaryKind::Interaction,
                owner: ResumeBoundaryOwner::Interaction(
                    ResumeBoundaryActivationIdentity::OrdinaryEvent {
                        component_instance: event.component_instance_id.clone(),
                        declaration_event: event.declaration_event_id.clone(),
                    },
                ),
                ownership_parent: None,
                owned_slots: Vec::new(),
                provenance: event.provenance.clone(),
            },
            depth: usize::MAX,
        });
        activation_references.push(ResumeBoundaryActivationReference {
            interaction_boundary: id,
            owner_boundary,
            required_boundaries,
            required_retained_slots,
            program: ResumeBoundaryActivationProgram::OrdinaryEvent {
                declaration_event: event.declaration_event_id.clone(),
                target: event.target_id.clone(),
                event_type: event.event_type.clone(),
                handler_method: event.handler_method_id.clone(),
                action_batch,
                existing_program: event.existing_event_program_identity.clone(),
            },
            provenance: event.provenance.clone(),
        });
    }

    for form in model.optimized_form_ir.optimized.instances.values() {
        for host in model
            .submission_hosts
            .values()
            .filter(|host| host.form == form.form)
        {
            let id =
                ResumeBoundaryId::interaction(&form.component_instance, host.id.as_semantic_id());
            let owner_boundary = ResumeBoundaryId::component_instance(&form.component_instance);
            let form_boundary = ResumeBoundaryId::form_instance(&form.id);
            let required_boundaries = vec![owner_boundary.clone(), form_boundary];
            let required_retained_slots =
                retained_slots_for_boundaries(&liveness, &required_boundaries);
            let target = TemplateInstanceTargetId::for_component_instance_template_entity(
                form.component_instance.clone(),
                host.owner_template_element.clone(),
            );
            pending.push(PendingBoundary {
                boundary: ResumeBoundary {
                    id: id.clone(),
                    kind: ResumeBoundaryKind::Interaction,
                    owner: ResumeBoundaryOwner::Interaction(
                        ResumeBoundaryActivationIdentity::FormSubmit {
                            component_instance: form.component_instance.clone(),
                            submission_host: host.id.clone(),
                            form_instance: form.id.clone(),
                        },
                    ),
                    ownership_parent: None,
                    owned_slots: Vec::new(),
                    provenance: host.provenance.clone(),
                },
                depth: usize::MAX,
            });
            activation_references.push(ResumeBoundaryActivationReference {
                interaction_boundary: id,
                owner_boundary,
                required_boundaries,
                required_retained_slots,
                program: ResumeBoundaryActivationProgram::FormSubmit {
                    submission_host: host.id.clone(),
                    target,
                    form_instance: form.id.clone(),
                    submission_plan: host.submission_plan.clone(),
                    submit_action: host.submit_action.clone(),
                    action_batch: host.action_batch.clone(),
                    serialization_plan: host.serialization_plan.clone(),
                    prevent_default: host.prevent_default,
                },
                provenance: host.provenance.clone(),
            });
        }
    }

    for blocked in model.component_instance_plan.blocked.values() {
        blocks.push(ResumeBoundaryBlock {
            candidate_boundary: Some(ResumeBoundaryId::component_instance(&blocked.id)),
            source: ResumeBoundaryBlockSource::ComponentInstance {
                instance: blocked.id.clone(),
                reason: blocked.reason,
            },
            provenance: blocked.provenance.clone(),
        });
    }
    for blocked in &liveness.blocked {
        blocks.push(ResumeBoundaryBlock {
            candidate_boundary: blocked.slot.boundary_candidate.clone(),
            source: ResumeBoundaryBlockSource::LivenessSlot {
                slot: blocked.slot.existing_slot.clone(),
                reason: blocked.reason,
            },
            provenance: blocked.slot.provenance.clone(),
        });
    }

    pending.sort_by(|left, right| {
        (left.depth, &left.boundary.id).cmp(&(right.depth, &right.boundary.id))
    });
    let boundaries = pending
        .into_iter()
        .map(|pending| pending.boundary)
        .collect::<Vec<_>>();
    ownership_edges.sort();
    ownership_edges.dedup();
    activation_references
        .sort_by(|left, right| left.interaction_boundary.cmp(&right.interaction_boundary));
    blocks.sort_by(|left, right| {
        (&left.candidate_boundary, &left.source).cmp(&(&right.candidate_boundary, &right.source))
    });
    let boundary_index = boundaries
        .iter()
        .enumerate()
        .map(|(index, boundary)| (boundary.id.clone(), index))
        .collect();
    let mut children_by_parent = BTreeMap::<ResumeBoundaryId, Vec<ResumeBoundaryId>>::new();
    for edge in &ownership_edges {
        children_by_parent
            .entry(edge.parent.clone())
            .or_default()
            .push(edge.child.clone());
    }
    for children in children_by_parent.values_mut() {
        children.sort();
        children.dedup();
    }
    ResumeBoundaryGraph {
        version: RESUME_BOUNDARY_GRAPH_VERSION,
        boundaries,
        ownership_edges,
        activation_references,
        blocks,
        boundary_index,
        children_by_parent,
    }
}

fn liveness_slots(
    liveness: &crate::ResumeLivenessPlan,
    boundary: &ResumeBoundaryId,
) -> Vec<ResumeExistingSlot> {
    let mut slots = liveness
        .slots_for_boundary(boundary)
        .into_iter()
        .map(|slot| slot.existing_slot.clone())
        .collect::<Vec<_>>();
    slots.sort();
    slots.dedup();
    slots
}

fn retained_slots_for_boundaries(
    liveness: &crate::ResumeLivenessPlan,
    boundaries: &[ResumeBoundaryId],
) -> Vec<ResumeExistingSlot> {
    let boundaries = boundaries.iter().collect::<BTreeSet<_>>();
    let mut slots = liveness
        .retained
        .iter()
        .filter(|record| {
            record
                .slot
                .boundary_candidate
                .as_ref()
                .is_some_and(|boundary| boundaries.contains(boundary))
        })
        .map(|record| record.slot.existing_slot.clone())
        .collect::<Vec<_>>();
    slots.sort();
    slots.dedup();
    slots
}

#[must_use]
#[allow(clippy::too_many_lines)]
pub fn validate_resume_boundary_graph(
    model: &ApplicationSemanticModel,
    graph: &ResumeBoundaryGraph,
) -> Vec<ResumeBoundaryIntegrityDiagnostic> {
    let canonical = build_resume_boundary_graph(model);
    let mut diagnostics = Vec::new();
    let mut ids = BTreeSet::new();
    for (index, boundary) in graph.boundaries.iter().enumerate() {
        if !ids.insert(boundary.id.clone()) {
            diagnostics.push(integrity(
                ResumeBoundaryIntegrityCode::DuplicateBoundary,
                Some(boundary.id.clone()),
                "resume boundary identity was emitted more than once",
            ));
        }
        if graph.boundary_index.get(&boundary.id) != Some(&index) {
            diagnostics.push(integrity(
                ResumeBoundaryIntegrityCode::OrderingOrIndexDrift,
                Some(boundary.id.clone()),
                "resume boundary index disagrees with parent-before-child order",
            ));
        }
        match boundary.kind {
            ResumeBoundaryKind::ApplicationRoot => {
                if boundary.ownership_parent.is_some() {
                    diagnostics.push(integrity(
                        ResumeBoundaryIntegrityCode::MissingOrMultipleParent,
                        Some(boundary.id.clone()),
                        "application root boundary cannot have an ownership parent",
                    ));
                }
            }
            ResumeBoundaryKind::Interaction => {
                if boundary.ownership_parent.is_some() {
                    diagnostics.push(integrity(
                        ResumeBoundaryIntegrityCode::InvalidOwner,
                        Some(boundary.id.clone()),
                        "interaction boundary references an owner and cannot own children",
                    ));
                }
            }
            ResumeBoundaryKind::ComponentInstance
            | ResumeBoundaryKind::StructuralRegion
            | ResumeBoundaryKind::FormInstance => {
                if boundary.ownership_parent.is_none() {
                    diagnostics.push(integrity(
                        ResumeBoundaryIntegrityCode::MissingOrMultipleParent,
                        Some(boundary.id.clone()),
                        "non-root ownership boundary must have exactly one parent",
                    ));
                }
            }
        }
        if let Some(expected) = canonical.boundary(&boundary.id) {
            if boundary.owner != expected.owner || boundary.kind != expected.kind {
                diagnostics.push(integrity(
                    ResumeBoundaryIntegrityCode::InvalidOwner,
                    Some(boundary.id.clone()),
                    "resume boundary owner disagrees with canonical Phase H/I ownership",
                ));
            }
            if boundary.provenance != expected.provenance {
                diagnostics.push(integrity(
                    ResumeBoundaryIntegrityCode::ProvenanceDrift,
                    Some(boundary.id.clone()),
                    "resume boundary provenance drifted from its canonical owner",
                ));
            }
        } else {
            diagnostics.push(integrity(
                ResumeBoundaryIntegrityCode::PhaseCorrespondence,
                Some(boundary.id.clone()),
                "resume boundary has no corresponding Phase H/I product",
            ));
        }
    }

    let edges = graph
        .ownership_edges
        .iter()
        .cloned()
        .collect::<BTreeSet<_>>();
    for boundary in &graph.boundaries {
        if let Some(parent) = &boundary.ownership_parent {
            let edge = ResumeBoundaryOwnershipEdge {
                parent: parent.clone(),
                child: boundary.id.clone(),
            };
            if !edges.contains(&edge)
                || !graph.children(parent).contains(&boundary.id)
                || graph.boundary(parent).is_none()
            {
                diagnostics.push(integrity(
                    ResumeBoundaryIntegrityCode::Reciprocity,
                    Some(boundary.id.clone()),
                    "boundary parent, ownership edge, and child index are not reciprocal",
                ));
            }
        }
    }
    for edge in &graph.ownership_edges {
        if graph
            .boundary(&edge.child)
            .is_none_or(|child| child.ownership_parent.as_ref() != Some(&edge.parent))
            || !graph.children(&edge.parent).contains(&edge.child)
        {
            diagnostics.push(integrity(
                ResumeBoundaryIntegrityCode::Reciprocity,
                Some(edge.child.clone()),
                "ownership edge does not reciprocate the boundary parent relation",
            ));
        }
    }

    for boundary in &graph.boundaries {
        if matches!(
            boundary.kind,
            ResumeBoundaryKind::ApplicationRoot | ResumeBoundaryKind::Interaction
        ) {
            continue;
        }
        let mut seen = BTreeSet::new();
        let mut current = &boundary.id;
        let mut reached_root = false;
        while let Some(parent) = graph.parent(current) {
            if !seen.insert(current.clone()) {
                diagnostics.push(integrity(
                    ResumeBoundaryIntegrityCode::Cycle,
                    Some(boundary.id.clone()),
                    "resume boundary ownership graph contains a cycle",
                ));
                break;
            }
            let Some(parent_boundary) = graph.boundary(parent) else {
                break;
            };
            if parent_boundary.kind == ResumeBoundaryKind::ApplicationRoot {
                reached_root = true;
                break;
            }
            current = parent;
        }
        if !reached_root {
            diagnostics.push(integrity(
                ResumeBoundaryIntegrityCode::Unreachable,
                Some(boundary.id.clone()),
                "resume boundary is not reachable from an application root",
            ));
        }
    }

    for reference in &graph.activation_references {
        if graph
            .boundary(&reference.interaction_boundary)
            .is_none_or(|boundary| boundary.kind != ResumeBoundaryKind::Interaction)
            || graph.boundary(&reference.owner_boundary).is_none()
            || reference
                .required_boundaries
                .iter()
                .any(|boundary| graph.boundary(boundary).is_none())
        {
            diagnostics.push(integrity(
                ResumeBoundaryIntegrityCode::InvalidOwner,
                Some(reference.interaction_boundary.clone()),
                "interaction activation references an unknown or non-interaction boundary",
            ));
        }
    }

    if graph.version != RESUME_BOUNDARY_GRAPH_VERSION
        || graph
            .boundaries
            .iter()
            .map(|boundary| &boundary.id)
            .collect::<Vec<_>>()
            != canonical
                .boundaries
                .iter()
                .map(|boundary| &boundary.id)
                .collect::<Vec<_>>()
        || graph.blocks != canonical.blocks
    {
        diagnostics.push(integrity(
            ResumeBoundaryIntegrityCode::PhaseCorrespondence,
            None,
            "resume boundary membership drifted from exact Phase H/I and J2 products",
        ));
    }
    if graph != &canonical {
        diagnostics.push(integrity(
            ResumeBoundaryIntegrityCode::OrderingOrIndexDrift,
            None,
            "resume boundary graph drifted from canonical order, edges, references, or indexes",
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
    code: ResumeBoundaryIntegrityCode,
    boundary: Option<ResumeBoundaryId>,
    message: &str,
) -> ResumeBoundaryIntegrityDiagnostic {
    ResumeBoundaryIntegrityDiagnostic {
        code,
        boundary,
        message: message.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn representative_model() -> ApplicationSemanticModel {
        crate::build_application_semantic_model(&presolve_parser::parse_file(
            "src/Boundary.tsx",
            r#"
@component("x-boundary-child") class Child extends Component {
  count = state(1);
  @action() increment() { this.count++; }
  render() { return <button onClick={() => this.increment()}>{this.count}</button>; }
}
@component("x-boundary-profile") class Profile {
  @form() @serialize("json") profile!: Form;
  @field(this.profile) name = "";
  @action() @submit(this.profile) save(): void {}
  render() { return <form form={this.profile}><input field={this.name}/></form>; }
}
@component("x-boundary-page") @route("/") class Page extends Component {
  show = state(true);
  render() {
    return <>
      {this.show ? <Child /> : <span />}
      <Profile />
    </>;
  }
}"#,
        ))
    }

    #[test]
    fn builds_parent_before_child_boundaries_and_activation_references() {
        let model = representative_model();
        let graph = build_resume_boundary_graph(&model);
        let diagnostics = validate_resume_boundary_graph(&model, &graph);
        assert!(diagnostics.is_empty(), "{diagnostics:#?}");
        assert_eq!(
            graph
                .boundaries
                .iter()
                .filter(|boundary| boundary.kind == ResumeBoundaryKind::ApplicationRoot)
                .count(),
            1
        );
        assert_eq!(
            graph
                .boundaries
                .iter()
                .filter(|boundary| boundary.kind == ResumeBoundaryKind::StructuralRegion)
                .count(),
            1
        );
        assert_eq!(
            graph
                .boundaries
                .iter()
                .filter(|boundary| boundary.kind == ResumeBoundaryKind::FormInstance)
                .count(),
            1
        );
        assert_eq!(graph.activation_references.len(), 2);
        assert!(graph.activation_references.iter().all(|reference| {
            graph.parent(&reference.interaction_boundary).is_none()
                && graph.boundary(&reference.owner_boundary).is_some()
        }));
        assert!(graph.activation_references.iter().any(|reference| {
            matches!(
                reference.program,
                ResumeBoundaryActivationProgram::FormSubmit { .. }
            ) && !reference.required_retained_slots.is_empty()
        }));
        for edge in &graph.ownership_edges {
            assert!(graph.boundary_index[&edge.parent] < graph.boundary_index[&edge.child]);
        }
    }

    #[test]
    fn preserves_blocked_component_and_liveness_products() {
        let mut model = crate::build_application_semantic_model(&presolve_parser::parse_file(
            "src/BlockedBoundary.tsx",
            r#"
@component("x-blocked-boundary") @route("/") class Page extends Component {
  value = state(null);
  render() { return <Missing />; }
}"#,
        ));
        let state = model.components[0].state_fields[0].id.clone();
        model
            .semantic_types
            .assignments
            .get_mut(&state)
            .expect("state assignment")
            .semantic_type = crate::SemanticType::Unknown;
        let graph = build_resume_boundary_graph(&model);
        assert!(graph.blocks.iter().any(|block| matches!(
            block.source,
            ResumeBoundaryBlockSource::ComponentInstance { .. }
        )));
        assert!(
            graph.blocks.iter().any(|block| matches!(
                block.source,
                ResumeBoundaryBlockSource::LivenessSlot { .. }
            )),
            "liveness={:#?}",
            build_resume_liveness_plan(&model)
        );
        assert!(validate_resume_boundary_graph(&model, &graph).is_empty());
    }

    #[test]
    fn boundary_graph_is_deterministic_and_rejects_noncanonical_parentage() {
        let first = presolve_parser::parse_file(
            "src/A.tsx",
            r#"@component("x-a") @route("/a") class A extends Component { render() { return <div />; } }"#,
        );
        let second = presolve_parser::parse_file(
            "src/B.tsx",
            r#"@component("x-b") @route("/b") class B extends Component { render() { return <div />; } }"#,
        );
        let forward = crate::build_application_semantic_model_for_unit(
            &crate::CompilationUnit::from_parsed_files(vec![first.clone(), second.clone()]),
        );
        let reverse = crate::build_application_semantic_model_for_unit(
            &crate::CompilationUnit::from_parsed_files(vec![second, first]),
        );
        assert_eq!(
            build_resume_boundary_graph(&forward),
            build_resume_boundary_graph(&reverse)
        );
        let mut invalid = build_resume_boundary_graph(&forward);
        let child = invalid
            .boundaries
            .iter_mut()
            .find(|boundary| boundary.kind == ResumeBoundaryKind::ComponentInstance)
            .expect("component boundary");
        child.ownership_parent = None;
        let diagnostics = validate_resume_boundary_graph(&forward, &invalid);
        assert!(diagnostics.iter().any(|diagnostic| {
            diagnostic.code == ResumeBoundaryIntegrityCode::MissingOrMultipleParent
        }));
        assert!(diagnostics
            .iter()
            .any(|diagnostic| { diagnostic.code == ResumeBoundaryIntegrityCode::Reciprocity }));
    }

    #[test]
    fn reserves_the_complete_j3_integrity_range() {
        assert_eq!(
            [
                ResumeBoundaryIntegrityCode::DuplicateBoundary,
                ResumeBoundaryIntegrityCode::InvalidOwner,
                ResumeBoundaryIntegrityCode::MissingOrMultipleParent,
                ResumeBoundaryIntegrityCode::Cycle,
                ResumeBoundaryIntegrityCode::Unreachable,
                ResumeBoundaryIntegrityCode::Reciprocity,
                ResumeBoundaryIntegrityCode::PhaseCorrespondence,
                ResumeBoundaryIntegrityCode::ProvenanceDrift,
                ResumeBoundaryIntegrityCode::OrderingOrIndexDrift,
            ]
            .map(ResumeBoundaryIntegrityCode::code),
            [
                "PSASM1328",
                "PSASM1329",
                "PSASM1330",
                "PSASM1331",
                "PSASM1332",
                "PSASM1333",
                "PSASM1334",
                "PSASM1335",
                "PSASM1336",
            ]
        );
    }
}
