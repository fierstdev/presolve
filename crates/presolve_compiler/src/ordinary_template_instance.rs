//! J1-P projection of ordinary template execution onto exact component instances.

use std::collections::{BTreeMap, BTreeSet};

use crate::component_graph::render_event_handlers;
use crate::{
    ApplicationSemanticModel, ComponentInstanceId, ComponentInstanceStatus, IrStorageId,
    SemanticId, SemanticReferenceKind, SourceProvenance, TemplateInstanceBindingId,
    TemplateInstanceTargetId, TemplateSemanticKind,
};

pub const ORDINARY_TEMPLATE_INSTANCE_REGISTRY_VERSION: u32 = 1;

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum OrdinaryTemplateTargetKind {
    Element,
    AttributeOrPropertyHost,
    EventHost,
    ConditionalBoundary,
    ListBoundary,
    FormControlHost,
    FormSubmissionHost,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum OrdinaryTemplateBindingKind {
    Text,
    Attribute,
    Property,
    Conditional,
    List,
    FormControl,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OrdinaryTemplateInstanceTargetRecord {
    pub target_id: TemplateInstanceTargetId,
    pub component_instance_id: ComponentInstanceId,
    pub component_id: SemanticId,
    pub template_entity_id: SemanticId,
    pub target_kind: OrdinaryTemplateTargetKind,
    pub provenance: SourceProvenance,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OrdinaryTemplateInstanceBindingRecord {
    pub instance_binding_id: TemplateInstanceBindingId,
    pub component_instance_id: ComponentInstanceId,
    pub component_id: SemanticId,
    pub declaration_binding_id: SemanticId,
    pub target_id: TemplateInstanceTargetId,
    pub binding_kind: OrdinaryTemplateBindingKind,
    pub state_storage_ids: Vec<IrStorageId>,
    pub computed_ids: Vec<SemanticId>,
    pub existing_program_identity: SemanticId,
    pub expression: Option<String>,
    pub attribute_name: Option<String>,
    pub provenance: SourceProvenance,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OrdinaryTemplateInstanceEventRecord {
    pub component_instance_id: ComponentInstanceId,
    pub component_id: SemanticId,
    pub target_id: TemplateInstanceTargetId,
    pub declaration_event_id: SemanticId,
    pub event_type: String,
    pub handler_method_id: SemanticId,
    pub action_batch_id: Option<SemanticId>,
    pub existing_event_program_identity: SemanticId,
    pub provenance: SourceProvenance,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OrdinaryTemplateInstanceRegistry {
    pub version: u32,
    pub targets: Vec<OrdinaryTemplateInstanceTargetRecord>,
    pub bindings: Vec<OrdinaryTemplateInstanceBindingRecord>,
    pub events: Vec<OrdinaryTemplateInstanceEventRecord>,
}

#[must_use]
#[allow(clippy::too_many_lines)]
pub fn build_ordinary_template_instance_registry(
    model: &ApplicationSemanticModel,
) -> OrdinaryTemplateInstanceRegistry {
    let mut targets = BTreeMap::new();
    let mut bindings = Vec::new();
    let mut events = Vec::new();
    for instance in model.component_instance_plan.instances.values() {
        if !matches!(
            instance.status,
            ComponentInstanceStatus::Planned | ComponentInstanceStatus::StructuralTemplate
        ) {
            continue;
        }
        let template = instance.component.template();
        let entities = model.template_entities_for(&template);
        for entity in &entities {
            let Some(binding_kind) = binding_kind(entity.kind) else {
                continue;
            };
            let target_entity = target_entity_for_binding(entity, &entities);
            let target_id = TemplateInstanceTargetId::for_component_instance_template_entity(
                instance.id.clone(),
                target_entity.id.clone(),
            );
            targets.entry(target_id.clone()).or_insert_with(|| {
                OrdinaryTemplateInstanceTargetRecord {
                    target_id: target_id.clone(),
                    component_instance_id: instance.id.clone(),
                    component_id: instance.component.clone(),
                    template_entity_id: target_entity.id.clone(),
                    target_kind: target_kind(target_entity.kind),
                    provenance: target_entity.provenance.clone(),
                }
            });
            let (state_storage_ids, computed_ids) = binding_dependencies(model, &entity.id);
            bindings.push(OrdinaryTemplateInstanceBindingRecord {
                instance_binding_id: TemplateInstanceBindingId::for_component_instance_binding(
                    instance.id.clone(),
                    entity.id.clone(),
                ),
                component_instance_id: instance.id.clone(),
                component_id: instance.component.clone(),
                declaration_binding_id: entity.id.clone(),
                target_id,
                binding_kind,
                state_storage_ids,
                computed_ids,
                existing_program_identity: entity.id.clone(),
                expression: entity.expression.clone(),
                attribute_name: entity.attribute_name.clone(),
                provenance: entity.provenance.clone(),
            });
        }
        if let Some(component) = model.component(&instance.component) {
            if let Some(render) = &component.render {
                for event in render_event_handlers(render) {
                    let Some(event_entity) = entities.iter().find(|entity| {
                        entity.kind == TemplateSemanticKind::EventAttribute
                            && provenance_matches_span(&entity.provenance, &event.span)
                    }) else {
                        continue;
                    };
                    let target_entity = target_entity_for_event(event_entity, &entities);
                    let target_id =
                        TemplateInstanceTargetId::for_component_instance_template_entity(
                            instance.id.clone(),
                            target_entity.id.clone(),
                        );
                    let method = component.methods.iter().find(|method| {
                        method.name
                            == event
                                .handler
                                .strip_prefix("this.")
                                .unwrap_or(&event.handler)
                    });
                    if let Some(method) = method {
                        let Some(action_batch_id) = model
                            .effect_trigger_plan
                            .action_batches
                            .values()
                            .find(|batch| batch.authored_action_method == method.id)
                            .map(|batch| batch.id.clone())
                        else {
                            continue;
                        };
                        targets.entry(target_id.clone()).or_insert_with(|| {
                            OrdinaryTemplateInstanceTargetRecord {
                                target_id: target_id.clone(),
                                component_instance_id: instance.id.clone(),
                                component_id: instance.component.clone(),
                                template_entity_id: target_entity.id.clone(),
                                target_kind: target_kind(target_entity.kind),
                                provenance: target_entity.provenance.clone(),
                            }
                        });
                        events.push(OrdinaryTemplateInstanceEventRecord {
                            component_instance_id: instance.id.clone(),
                            component_id: instance.component.clone(),
                            target_id,
                            declaration_event_id: event.id.clone(),
                            event_type: event.event.clone(),
                            handler_method_id: method.id.clone(),
                            action_batch_id: Some(action_batch_id),
                            existing_event_program_identity: event.id.clone(),
                            provenance: model
                                .provenance(&event.id)
                                .cloned()
                                .unwrap_or_else(|| event_entity.provenance.clone()),
                        });
                    }
                }
            }
        }
        // Forms retain their Phase-I ownership and programs. This bridge only
        // attaches their already-canonical control/host entities to the same
        // ordinary instance target table used by all other executable DOM.
        for binding in model
            .form_field_bindings
            .values()
            .filter(|binding| binding.component == instance.component)
        {
            if !model
                .optimized_form_ir
                .optimized
                .instances
                .values()
                .any(|form| form.form == binding.form && form.component_instance == instance.id)
            {
                continue;
            }
            insert_form_target(
                &mut targets,
                instance,
                binding.control_entity.clone(),
                OrdinaryTemplateTargetKind::FormControlHost,
                binding.provenance.clone(),
            );
        }
        for host in model
            .submission_hosts
            .values()
            .filter(|host| host.component == instance.component)
        {
            if !model
                .optimized_form_ir
                .optimized
                .instances
                .values()
                .any(|form| form.form == host.form && form.component_instance == instance.id)
            {
                continue;
            }
            insert_form_target(
                &mut targets,
                instance,
                host.owner_template_element.clone(),
                OrdinaryTemplateTargetKind::FormSubmissionHost,
                host.provenance.clone(),
            );
        }
    }
    let mut targets = targets.into_values().collect::<Vec<_>>();
    targets.sort_by(|left, right| left.target_id.cmp(&right.target_id));
    bindings.sort_by(|left, right| left.instance_binding_id.cmp(&right.instance_binding_id));
    events.sort_by(|left, right| {
        (&left.component_instance_id, &left.declaration_event_id)
            .cmp(&(&right.component_instance_id, &right.declaration_event_id))
    });
    OrdinaryTemplateInstanceRegistry {
        version: ORDINARY_TEMPLATE_INSTANCE_REGISTRY_VERSION,
        targets,
        bindings,
        events,
    }
}

fn binding_kind(kind: TemplateSemanticKind) -> Option<OrdinaryTemplateBindingKind> {
    match kind {
        TemplateSemanticKind::AttributeBinding => Some(OrdinaryTemplateBindingKind::Attribute),
        TemplateSemanticKind::Binding => Some(OrdinaryTemplateBindingKind::Text),
        TemplateSemanticKind::Conditional => Some(OrdinaryTemplateBindingKind::Conditional),
        TemplateSemanticKind::List => Some(OrdinaryTemplateBindingKind::List),
        TemplateSemanticKind::Fragment
        | TemplateSemanticKind::Element
        | TemplateSemanticKind::Text
        | TemplateSemanticKind::Attribute
        | TemplateSemanticKind::EventAttribute => None,
    }
}

const fn target_kind(kind: TemplateSemanticKind) -> OrdinaryTemplateTargetKind {
    match kind {
        TemplateSemanticKind::Conditional => OrdinaryTemplateTargetKind::ConditionalBoundary,
        TemplateSemanticKind::List => OrdinaryTemplateTargetKind::ListBoundary,
        _ => OrdinaryTemplateTargetKind::Element,
    }
}

fn target_entity_for_binding<'a>(
    entity: &'a crate::TemplateSemanticEntity,
    entities: &[&'a crate::TemplateSemanticEntity],
) -> &'a crate::TemplateSemanticEntity {
    match entity.kind {
        TemplateSemanticKind::AttributeBinding => containing_element(entity, entities),
        _ => entity,
    }
}

fn target_entity_for_event<'a>(
    entity: &'a crate::TemplateSemanticEntity,
    entities: &[&'a crate::TemplateSemanticEntity],
) -> &'a crate::TemplateSemanticEntity {
    containing_element(entity, entities)
}

fn containing_element<'a>(
    entity: &'a crate::TemplateSemanticEntity,
    entities: &[&'a crate::TemplateSemanticEntity],
) -> &'a crate::TemplateSemanticEntity {
    entities
        .iter()
        .copied()
        .filter(|candidate| {
            candidate.kind == TemplateSemanticKind::Element
                && provenance_contains(&candidate.provenance, &entity.provenance)
        })
        .min_by_key(|candidate| candidate.provenance.span.end - candidate.provenance.span.start)
        .unwrap_or(entity)
}

fn provenance_contains(parent: &SourceProvenance, child: &SourceProvenance) -> bool {
    parent.path == child.path
        && parent.span.start <= child.span.start
        && parent.span.end >= child.span.end
}

fn provenance_matches_span(
    provenance: &SourceProvenance,
    span: &presolve_parser::SourceSpan,
) -> bool {
    provenance.span.start == span.start && provenance.span.end == span.end
}

fn insert_form_target(
    targets: &mut BTreeMap<TemplateInstanceTargetId, OrdinaryTemplateInstanceTargetRecord>,
    instance: &crate::ComponentInstance,
    template_entity_id: SemanticId,
    target_kind: OrdinaryTemplateTargetKind,
    provenance: SourceProvenance,
) {
    let target_id = TemplateInstanceTargetId::for_component_instance_template_entity(
        instance.id.clone(),
        template_entity_id.clone(),
    );
    targets
        .entry(target_id.clone())
        .or_insert(OrdinaryTemplateInstanceTargetRecord {
            target_id,
            component_instance_id: instance.id.clone(),
            component_id: instance.component.clone(),
            template_entity_id,
            target_kind,
            provenance,
        });
}

fn binding_dependencies(
    model: &ApplicationSemanticModel,
    binding: &SemanticId,
) -> (Vec<IrStorageId>, Vec<SemanticId>) {
    let state_storage_ids = model
        .references_from(binding)
        .into_iter()
        .filter(|reference| reference.kind == SemanticReferenceKind::TemplateState)
        .map(|reference| IrStorageId::for_semantic_origin(&reference.target))
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect();
    let computed_ids = model
        .references_from(binding)
        .into_iter()
        .filter(|reference| reference.kind == SemanticReferenceKind::TemplateComputed)
        .map(|reference| reference.target.clone())
        .collect();
    (state_storage_ids, computed_ids)
}

/// # Errors
///
/// Returns an error when a retained record is malformed, duplicate, or no
/// longer equals the canonical Phase H/template projection.
pub fn validate_ordinary_template_instance_registry(
    model: &ApplicationSemanticModel,
    registry: &OrdinaryTemplateInstanceRegistry,
) -> Result<(), String> {
    if registry.version != ORDINARY_TEMPLATE_INSTANCE_REGISTRY_VERSION {
        return Err("unsupported ordinary template instance registry version".to_string());
    }
    if registry != &build_ordinary_template_instance_registry(model) {
        return Err(
            "ordinary template instance registry drifted from canonical products".to_string(),
        );
    }
    let mut targets = BTreeSet::new();
    let mut bindings = BTreeSet::new();
    for target in &registry.targets {
        if target.target_id
            != TemplateInstanceTargetId::for_component_instance_template_entity(
                target.component_instance_id.clone(),
                target.template_entity_id.clone(),
            )
            || !targets.insert(target.target_id.clone())
        {
            return Err("ordinary template target is duplicate or malformed".to_string());
        }
    }
    for binding in &registry.bindings {
        if binding.instance_binding_id
            != TemplateInstanceBindingId::for_component_instance_binding(
                binding.component_instance_id.clone(),
                binding.declaration_binding_id.clone(),
            )
            || !targets.contains(&binding.target_id)
            || !bindings.insert(binding.instance_binding_id.clone())
        {
            return Err("ordinary template binding is duplicate or malformed".to_string());
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{build_application_semantic_model_for_unit, CompilationUnit};

    #[test]
    fn projects_distinct_targets_and_bindings_for_repeated_instances() {
        let source = presolve_parser::parse_file(
            "src/Repeated.tsx",
            r#"
@component("x-child") class Child {
  count = state(1);
  @action()
  increment() { this.count++; }
  render() { return <button title={this.count} onClick={() => this.increment()}>{this.count}</button>; }
}
@component("x-parent") class Parent { render() { return <><Child /><Child /></>; } }
"#,
        );
        let model =
            build_application_semantic_model_for_unit(&CompilationUnit::from_parsed_files(vec![
                source,
            ]));
        let registry = build_ordinary_template_instance_registry(&model);
        let child_targets = registry
            .targets
            .iter()
            .filter(|target| target.component_id.as_str().contains("x-child"))
            .collect::<Vec<_>>();
        assert!(child_targets.len() >= 2);
        assert!(child_targets
            .windows(2)
            .any(|pair| pair[0].target_id != pair[1].target_id));
        let child_events = registry
            .events
            .iter()
            .filter(|event| event.component_id.as_str().contains("x-child"))
            .collect::<Vec<_>>();
        assert_eq!(child_events.len(), 2);
        assert_ne!(child_events[0].target_id, child_events[1].target_id);
        assert!(registry.bindings.iter().all(|binding| {
            registry
                .targets
                .iter()
                .any(|target| target.target_id == binding.target_id)
        }));
        assert!(validate_ordinary_template_instance_registry(&model, &registry).is_ok());
    }
}
