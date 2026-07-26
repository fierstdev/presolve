use serde::{Deserialize, Serialize};

use crate::{
    build_computed_instance_slot_registry, build_ordinary_template_instance_registry,
    build_runtime_component_registry, build_state_instance_storage_registry,
    lower_components_to_ir, resume_value_codec, semantic_type_text, ApplicationSemanticModel,
    OptimizedComponentIrReport, OrdinaryTemplateBindingKind, OrdinaryTemplateTargetKind,
    ResumeValueCodec, RuntimeComponentRegistry, SerializationCompatibility,
};
use crate::{TemplateChild, TemplateNode, TemplateSemanticKind};

pub const RUNTIME_COMPONENT_ARTIFACT_SCHEMA_VERSION: u32 = 20;

/// Public H14 compiler artifact. All executable references are canonical IDs.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RuntimeComponentArtifact {
    pub schema_version: u32,
    pub component_definitions: Vec<SerializedComponentDefinition>,
    pub instances: Vec<SerializedComponentInstance>,
    pub initialization_batches: Vec<SerializedComponentBatch>,
    pub slot_binding_programs: Vec<SerializedSlotBinding>,
    pub instance_context_bindings: Vec<SerializedInstanceContextBinding>,
    pub ordinary_template_targets: Vec<SerializedOrdinaryTemplateTarget>,
    pub ordinary_template_bindings: Vec<SerializedOrdinaryTemplateBinding>,
    pub ordinary_template_events: Vec<SerializedOrdinaryTemplateEvent>,
    pub destruction: SerializedDestructionMetadata,
    pub structural_programs: Vec<SerializedStructuralComponentProgram>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SerializedComponentDefinition {
    pub component: String,
    pub template: String,
    pub slots: Vec<String>,
    pub boundary: String,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SerializedComponentInstance {
    pub instance: String,
    pub component: String,
    pub invocation: Option<String>,
    pub parent: Option<String>,
    pub depth: usize,
    pub initialization_batch: usize,
    pub ordinary_template_targets: Vec<String>,
    pub ordinary_template_bindings: Vec<String>,
    pub ordinary_template_events: Vec<String>,
    pub state_slots: Vec<SerializedRuntimeStateSlot>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub computed_slots: Vec<SerializedRuntimeComputedSlot>,
    pub context_prefix: String,
    pub instruction_indices: Vec<usize>,
    pub structural_region: Option<String>,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SerializedRuntimeStateSlot {
    pub slot_id: String,
    pub state_id: String,
    pub storage_id: String,
    pub initial_value: crate::SerializableValue,
    pub semantic_type: String,
    pub serializable: bool,
    /// The compiler-issued closed resume codec for a serializable structural
    /// occurrence slot. Static boundary schemas remain the authority for
    /// static-instance capture.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub resume_codec: Option<ResumeValueCodec>,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SerializedRuntimeComputedSlot {
    pub computed_id: String,
    pub cache_slot_id: String,
    pub dirty_slot_id: String,
    pub dirty_initial_value: bool,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SerializedOrdinaryTemplateTarget {
    pub id: String,
    pub component_instance_id: String,
    pub component_id: String,
    pub template_entity_id: String,
    pub kind: String,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SerializedOrdinaryTemplateBinding {
    pub id: String,
    pub component_instance_id: String,
    pub component_id: String,
    pub declaration_binding_id: String,
    pub target_id: String,
    pub kind: String,
    pub state_storage_ids: Vec<String>,
    pub computed_ids: Vec<String>,
    pub program_id: String,
    pub expression: Option<String>,
    pub attribute_name: Option<String>,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SerializedOrdinaryTemplateEvent {
    pub component_instance_id: String,
    pub component_id: String,
    pub target_id: String,
    pub declaration_event_id: String,
    pub event_type: String,
    pub handler_method_id: String,
    pub action_batch_id: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub arguments: Vec<crate::component_graph::SerializableValue>,
    pub program_id: String,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SerializedComponentBatch {
    pub index: usize,
    pub instances: Vec<String>,
    pub context_sources: Vec<String>,
    pub slot_bindings: Vec<String>,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SerializedSlotBinding {
    pub binding: String,
    pub caller_instance: String,
    pub callee_instance: String,
    pub slot: String,
    pub outlet: String,
    pub fragment: String,
    pub content_owner_instance: String,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SerializedInstanceContextBinding {
    pub consumer_instance: String,
    pub selected_source: String,
    pub provider_source: Option<String>,
    pub runtime_slot: String,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SerializedDestructionMetadata {
    pub operation: String,
    pub enabled: bool,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SerializedStructuralComponentProgram {
    pub region: String,
    /// Canonical component that owns the conditional or keyed-list host.
    pub host_component: String,
    /// Exact compiler-generated template node ID for the structural host.
    pub host_node: String,
    /// Exact semantic template entity for the conditional or keyed-list host.
    pub host_template_entity: String,
    /// Compiler-rendered conditional branches for initially static host
    /// instances. Keyed-list and nested host scopes remain absent until their
    /// complete compiler input scopes have an authored product.
    pub conditional_host_fragments: Vec<SerializedStructuralConditionalHostFragments>,
    /// Compiler-rendered keyed item fragments with exact structural invocation
    /// anchors. They remain inactive until keyed materialization is admitted.
    pub keyed_host_fragments: Vec<SerializedStructuralKeyedHostFragment>,
    pub template_occurrences: Vec<SerializedStructuralTemplateOccurrence>,
    pub template_instances: Vec<String>,
    pub destroy_order: Vec<String>,
    pub create_order: Vec<String>,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SerializedStructuralConditionalHostFragments {
    /// `static-instance` or `structural-occurrence`; selected by the compiler.
    pub host_scope: String,
    /// Exact static component instance or structural template instance that
    /// owns the conditional according to `host_scope`.
    pub host_instance: String,
    pub when_true_html: String,
    pub when_false_html: String,
    /// Exact compiler-issued occurrence invocations anchored by each branch.
    pub when_true_invocations: Vec<String>,
    pub when_false_invocations: Vec<String>,
    pub slot_projection_bindings: Vec<String>,
    /// Exact compiler-selected caller-owned ordinary members rendered into
    /// this host's Slot outlets. The runtime never discovers these from DOM.
    pub slot_projection_programs: Vec<SerializedStructuralSlotProjectionProgram>,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SerializedStructuralKeyedHostFragment {
    pub host_scope: String,
    pub host_instance: String,
    pub item_template_html: String,
    pub item_invocations: Vec<String>,
    pub slot_projection_bindings: Vec<String>,
    /// Exact compiler-selected caller-owned ordinary members rendered into
    /// this host's Slot outlets. The runtime never discovers these from DOM.
    pub slot_projection_programs: Vec<SerializedStructuralSlotProjectionProgram>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SerializedStructuralSlotProjectionProgram {
    pub binding: String,
    pub caller_instance: String,
    pub content_owner_instance: String,
    pub target_ids: Vec<String>,
    pub binding_ids: Vec<String>,
    pub event_ids: Vec<String>,
    pub nested_invocations: Vec<String>,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SerializedStructuralTemplateOccurrence {
    pub template_instance: String,
    pub invocation: String,
    /// Exact semantic template entity for this invocation marker.
    pub invocation_template_entity: String,
    pub component: String,
    /// Compiler template parent. The runtime substitutes only the opaque
    /// occurrence identity prefix when restoring a dynamic descendant.
    pub parent_template_instance: String,
    /// Compiler template slots; runtime occurrence identity replaces only the
    /// template-instance prefix when materialization is later admitted.
    pub state_slots: Vec<SerializedRuntimeStateSlot>,
    pub computed_slots: Vec<SerializedRuntimeComputedSlot>,
    /// Compiler-rendered target component template. It remains inactive until
    /// the materializer consumes it under an opaque occurrence identity.
    pub template_html: String,
    pub nested_invocations: Vec<String>,
    /// Inactive compiler-owned template projection for a future materializer.
    /// These IDs must be used as emitted; they are never selected from the DOM.
    pub ordinary_template_targets: Vec<String>,
    pub ordinary_template_bindings: Vec<String>,
    pub ordinary_template_events: Vec<String>,
}

const fn target_kind_text(kind: OrdinaryTemplateTargetKind) -> &'static str {
    match kind {
        OrdinaryTemplateTargetKind::Element => "element",
        OrdinaryTemplateTargetKind::AttributeOrPropertyHost => "attribute_or_property_host",
        OrdinaryTemplateTargetKind::EventHost => "event_host",
        OrdinaryTemplateTargetKind::ConditionalBoundary => "conditional_boundary",
        OrdinaryTemplateTargetKind::ListBoundary => "list_boundary",
        OrdinaryTemplateTargetKind::FormControlHost => "form_control_host",
        OrdinaryTemplateTargetKind::FormSubmissionHost => "form_submission_host",
    }
}

const fn binding_kind_text(kind: OrdinaryTemplateBindingKind) -> &'static str {
    match kind {
        OrdinaryTemplateBindingKind::Text => "text",
        OrdinaryTemplateBindingKind::Attribute => "attribute",
        OrdinaryTemplateBindingKind::Property => "property",
        OrdinaryTemplateBindingKind::Conditional => "conditional",
        OrdinaryTemplateBindingKind::List => "list",
        OrdinaryTemplateBindingKind::FormControl => "form_control",
    }
}

#[must_use]
#[allow(clippy::too_many_lines)]
pub fn build_runtime_component_artifact(
    model: &ApplicationSemanticModel,
    optimized: &OptimizedComponentIrReport,
) -> RuntimeComponentArtifact {
    let ordinary = build_ordinary_template_instance_registry(model);
    let ir = lower_components_to_ir(model);
    let state_slots = build_state_instance_storage_registry(model, &ir);
    let computed_slots = build_computed_instance_slot_registry(model, &ir);
    let mut artifact = artifact_from_registry(&build_runtime_component_registry(model, optimized));
    artifact.ordinary_template_targets = ordinary
        .targets
        .iter()
        .map(|target| SerializedOrdinaryTemplateTarget {
            id: target.target_id.to_string(),
            component_instance_id: target.component_instance_id.to_string(),
            component_id: target.component_id.to_string(),
            template_entity_id: target.template_entity_id.to_string(),
            kind: target_kind_text(target.target_kind).to_string(),
        })
        .collect();
    artifact.ordinary_template_bindings = ordinary
        .bindings
        .iter()
        .map(|binding| SerializedOrdinaryTemplateBinding {
            id: binding.instance_binding_id.to_string(),
            component_instance_id: binding.component_instance_id.to_string(),
            component_id: binding.component_id.to_string(),
            declaration_binding_id: binding.declaration_binding_id.to_string(),
            target_id: binding.target_id.to_string(),
            kind: binding_kind_text(binding.binding_kind).to_string(),
            state_storage_ids: binding
                .state_storage_ids
                .iter()
                .map(ToString::to_string)
                .collect(),
            computed_ids: binding
                .computed_ids
                .iter()
                .map(ToString::to_string)
                .collect(),
            program_id: binding.existing_program_identity.to_string(),
            expression: binding.expression.clone(),
            attribute_name: binding.attribute_name.clone(),
        })
        .collect();
    artifact.ordinary_template_events = ordinary
        .events
        .iter()
        .map(|event| SerializedOrdinaryTemplateEvent {
            component_instance_id: event.component_instance_id.to_string(),
            component_id: event.component_id.to_string(),
            target_id: event.target_id.to_string(),
            declaration_event_id: event.declaration_event_id.to_string(),
            event_type: event.event_type.clone(),
            handler_method_id: event.handler_method_id.to_string(),
            action_batch_id: event.action_batch_id.as_ref().map(ToString::to_string),
            arguments: event.arguments.clone(),
            program_id: event.existing_event_program_identity.to_string(),
        })
        .collect();
    for instance in &mut artifact.instances {
        instance.ordinary_template_targets = artifact
            .ordinary_template_targets
            .iter()
            .filter(|target| target.component_instance_id == instance.instance)
            .map(|target| target.id.clone())
            .collect();
        instance.ordinary_template_bindings = artifact
            .ordinary_template_bindings
            .iter()
            .filter(|binding| binding.component_instance_id == instance.instance)
            .map(|binding| binding.id.clone())
            .collect();
        instance.ordinary_template_events = artifact
            .ordinary_template_events
            .iter()
            .filter(|event| event.component_instance_id == instance.instance)
            .map(|event| event.declaration_event_id.clone())
            .collect();
        instance.state_slots = state_slots
            .records
            .iter()
            .filter(|slot| slot.component_instance_id.to_string() == instance.instance)
            .map(|slot| SerializedRuntimeStateSlot {
                slot_id: slot.slot_id.to_string(),
                state_id: slot.state_id.to_string(),
                storage_id: slot.storage_id.to_string(),
                initial_value: slot.initial_value.clone(),
                semantic_type: semantic_type_text(&slot.semantic_type),
                serializable: slot.serialization == SerializationCompatibility::Serializable,
                resume_codec: (slot.serialization == SerializationCompatibility::Serializable)
                    .then(|| resume_value_codec(&slot.semantic_type).ok())
                    .flatten(),
            })
            .collect();
        instance.computed_slots = computed_slots
            .records
            .iter()
            .filter(|slot| slot.component_instance_id.to_string() == instance.instance)
            .map(|slot| SerializedRuntimeComputedSlot {
                computed_id: slot.computed_id.to_string(),
                cache_slot_id: slot.cache_slot_id.to_string(),
                dirty_slot_id: slot.dirty_slot_id.to_string(),
                dirty_initial_value: slot.dirty_initial_value,
            })
            .collect();
    }
    let mut programs = std::collections::BTreeMap::<String, StructuralProgramBuild>::new();
    for instance in model.component_instance_plan.instances.values() {
        if instance.status == crate::ComponentInstanceStatus::StructuralTemplate {
            if let Some(region) = &instance.structural_region {
                let program = programs
                    .entry(region.to_string())
                    .or_insert_with(|| structural_program_build(model, region));
                program.template_instances.push(instance.id.to_string());
                program
                    .template_occurrences
                    .push(SerializedStructuralTemplateOccurrence {
                        template_instance: instance.id.to_string(),
                        invocation: instance
                            .invocation
                            .as_ref()
                            .expect("structural template invocation")
                            .to_string(),
                        invocation_template_entity: model
                            .component_invocations
                            .get(
                                instance
                                    .invocation
                                    .as_ref()
                                    .expect("structural template invocation"),
                            )
                            .expect("structural template invocation is compiler-owned")
                            .template_entity
                            .to_string(),
                        component: instance.component.to_string(),
                        parent_template_instance: instance
                            .parent_instance
                            .as_ref()
                            .expect("structural template parent")
                            .to_string(),
                        state_slots: state_slots
                            .records
                            .iter()
                            .filter(|slot| slot.component_instance_id == instance.id)
                            .map(|slot| SerializedRuntimeStateSlot {
                                slot_id: slot.slot_id.to_string(),
                                state_id: slot.state_id.to_string(),
                                storage_id: slot.storage_id.to_string(),
                                initial_value: slot.initial_value.clone(),
                                semantic_type: semantic_type_text(&slot.semantic_type),
                                serializable: slot.serialization
                                    == SerializationCompatibility::Serializable,
                                resume_codec: (slot.serialization
                                    == SerializationCompatibility::Serializable)
                                    .then(|| resume_value_codec(&slot.semantic_type).ok())
                                    .flatten(),
                            })
                            .collect(),
                        computed_slots: computed_slots
                            .records
                            .iter()
                            .filter(|slot| slot.component_instance_id == instance.id)
                            .map(|slot| SerializedRuntimeComputedSlot {
                                computed_id: slot.computed_id.to_string(),
                                cache_slot_id: slot.cache_slot_id.to_string(),
                                dirty_slot_id: slot.dirty_slot_id.to_string(),
                                dirty_initial_value: slot.dirty_initial_value,
                            })
                            .collect(),
                        template_html: crate::generate_structural_template_instance_html(
                            model,
                            &instance.id,
                        )
                        .unwrap_or_default(),
                        nested_invocations:
                            crate::ordinary_html_codegen::structural_invocations_in_compiler_html(
                                &crate::generate_structural_template_instance_html(
                                    model,
                                    &instance.id,
                                )
                                .unwrap_or_default(),
                            ),
                        ordinary_template_targets: artifact
                            .ordinary_template_targets
                            .iter()
                            .filter(|target| {
                                target.component_instance_id == instance.id.to_string()
                            })
                            .map(|target| target.id.clone())
                            .collect(),
                        ordinary_template_bindings: artifact
                            .ordinary_template_bindings
                            .iter()
                            .filter(|binding| {
                                binding.component_instance_id == instance.id.to_string()
                            })
                            .map(|binding| binding.id.clone())
                            .collect(),
                        ordinary_template_events: artifact
                            .ordinary_template_events
                            .iter()
                            .filter(|event| event.component_instance_id == instance.id.to_string())
                            .map(|event| event.declaration_event_id.clone())
                            .collect(),
                    });
            }
        }
    }
    artifact.structural_programs = programs
        .into_iter()
        .map(|(region, program)| SerializedStructuralComponentProgram {
            region,
            host_component: program.host_component,
            host_node: program.host_node,
            host_template_entity: program.host_template_entity,
            conditional_host_fragments: crate::generate_structural_conditional_host_fragments(
                model,
                &program.region,
            )
            .into_iter()
            .map(|fragments| {
                let compiler_html =
                    format!("{}{}", fragments.when_true_html, fragments.when_false_html);
                SerializedStructuralConditionalHostFragments {
                    host_scope: fragments.host_scope.artifact_text().to_string(),
                    host_instance: fragments.host_instance.to_string(),
                    when_true_html: fragments.when_true_html,
                    when_false_html: fragments.when_false_html,
                    when_true_invocations: fragments.when_true_invocations,
                    when_false_invocations: fragments.when_false_invocations,
                    slot_projection_programs: structural_slot_projection_programs(
                        model,
                        &ordinary,
                        &fragments.slot_projection_bindings,
                        &compiler_html,
                    ),
                    slot_projection_bindings: fragments.slot_projection_bindings,
                }
            })
            .collect(),
            keyed_host_fragments: crate::generate_structural_keyed_host_fragments(
                model,
                &program.region,
            )
            .into_iter()
            .map(|fragments| {
                let slot_projection_programs = structural_slot_projection_programs(
                    model,
                    &ordinary,
                    &fragments.slot_projection_bindings,
                    &fragments.item_template_html,
                );
                SerializedStructuralKeyedHostFragment {
                    host_scope: fragments.host_scope.artifact_text().to_string(),
                    host_instance: fragments.host_instance.to_string(),
                    item_template_html: fragments.item_template_html,
                    item_invocations: fragments.item_invocations,
                    slot_projection_programs,
                    slot_projection_bindings: fragments.slot_projection_bindings,
                }
            })
            .collect(),
            template_occurrences: program.template_occurrences,
            create_order: program.template_instances.clone(),
            destroy_order: program.template_instances.iter().rev().cloned().collect(),
            template_instances: program.template_instances,
        })
        .collect();
    artifact
}

fn structural_slot_projection_programs(
    model: &ApplicationSemanticModel,
    ordinary: &crate::OrdinaryTemplateInstanceRegistry,
    bindings: &[String],
    compiler_html: &str,
) -> Vec<SerializedStructuralSlotProjectionProgram> {
    let semantic_entities = crate::build_template_semantic_entities(&model.templates);
    let semantic_entities = semantic_entities
        .iter()
        .map(|entity| (&entity.id, entity))
        .collect::<std::collections::BTreeMap<_, _>>();
    bindings
        .iter()
        .filter_map(|binding_id| {
            let binding = model
                .slot_bindings
                .bindings
                .values()
                .find(|binding| binding.id.as_str() == binding_id)?;
            let fragment = model
                .slot_content_fragments
                .get(binding.content_fragment.as_ref()?)?;
            let roots = fragment
                .content_template_entities
                .iter()
                .filter_map(|entity| semantic_entities.get(entity).copied())
                .collect::<Vec<_>>();
            let owns_entity = |entity_id: &crate::SemanticId| {
                let Some(entity) = semantic_entities.get(entity_id).copied() else {
                    return false;
                };
                roots.iter().any(|root| {
                    root.provenance.path == entity.provenance.path
                        && root.provenance.span.start <= entity.provenance.span.start
                        && entity.provenance.span.end <= root.provenance.span.end
                })
            };
            let rendered_binding_targets = ordinary
                .bindings
                .iter()
                .filter(|record| {
                    record.component_instance_id == binding.caller_instance
                        && compiler_html.contains(&record.instance_binding_id.to_string())
                })
                .map(|record| record.target_id.to_string())
                .collect::<std::collections::BTreeSet<_>>();
            let target_ids = ordinary
                .targets
                .iter()
                .filter(|target| {
                    target.component_instance_id == binding.caller_instance
                        && owns_entity(&target.template_entity_id)
                        && (compiler_html.contains(&target.target_id.to_string())
                            || rendered_binding_targets.contains(&target.target_id.to_string()))
                })
                .map(|target| target.target_id.to_string())
                .collect::<Vec<_>>();
            let target_set = target_ids.iter().collect::<std::collections::BTreeSet<_>>();
            let binding_ids = ordinary
                .bindings
                .iter()
                .filter(|record| {
                    record.component_instance_id == binding.caller_instance
                        && target_set.contains(&record.target_id.to_string())
                })
                .map(|record| record.instance_binding_id.to_string())
                .collect::<Vec<_>>();
            let event_ids = ordinary
                .events
                .iter()
                .filter(|event| {
                    event.component_instance_id == binding.caller_instance
                        && target_set.contains(&event.target_id.to_string())
                })
                .map(|event| event.declaration_event_id.to_string())
                .collect::<Vec<_>>();
            let caller_component = &model
                .component_instance_plan
                .instances
                .get(&binding.caller_instance)?
                .component;
            let nested_invocations = model
                .component_invocations_owned_by(caller_component)
                .into_iter()
                .filter(|invocation| {
                    model
                        .component_invocations
                        .get(invocation)
                        .is_some_and(|record| {
                            owns_entity(&record.template_entity)
                                && compiler_html.contains(invocation.as_str())
                        })
                })
                .map(ToString::to_string)
                .collect();
            Some(SerializedStructuralSlotProjectionProgram {
                binding: binding_id.clone(),
                caller_instance: binding.caller_instance.to_string(),
                content_owner_instance: binding.content_owner_instance.to_string(),
                target_ids,
                binding_ids,
                event_ids,
                nested_invocations,
            })
        })
        .collect()
}

#[derive(Debug, Clone)]
struct StructuralProgramBuild {
    region: crate::ComponentStructuralRegionId,
    host_component: String,
    host_node: String,
    host_template_entity: String,
    template_instances: Vec<String>,
    template_occurrences: Vec<SerializedStructuralTemplateOccurrence>,
}

/// Resolve a structural-region ID back to its exact compiler-authored host.
///
/// A region ID is derived from the semantic template entity, while the DOM
/// renderer addresses the same construct by its generated template node ID.
/// This is the only compiler join between those domains; runtime code must not
/// infer it from DOM shape, selectors, or user data.
fn structural_program_build(
    model: &ApplicationSemanticModel,
    region: &crate::ComponentStructuralRegionId,
) -> StructuralProgramBuild {
    let entity = crate::structural_template_entity_for_region(region, &model.template_entities)
        .expect("structural component instance references a semantic template host");
    let template_id = entity
        .owner
        .entity_id()
        .expect("structural template host has a template owner");
    let template = model
        .templates
        .iter()
        .find(|template| &template.id == template_id)
        .expect("structural template host has an emitted template");
    let component = template
        .owner
        .entity_id()
        .expect("emitted template has a component owner");
    let host_node = structural_host_node(template, entity.kind, entity.provenance.span)
        .expect("structural template host has an emitted runtime node");

    StructuralProgramBuild {
        region: region.clone(),
        host_component: component.to_string(),
        host_node,
        host_template_entity: entity.id.to_string(),
        template_instances: Vec::new(),
        template_occurrences: Vec::new(),
    }
}

fn structural_host_node(
    template: &TemplateNode,
    kind: TemplateSemanticKind,
    span: presolve_parser::SourceSpan,
) -> Option<String> {
    fn visit(
        children: &[TemplateChild],
        kind: TemplateSemanticKind,
        span: presolve_parser::SourceSpan,
    ) -> Option<String> {
        for child in children {
            match child {
                TemplateChild::Element(element) => {
                    if let Some(id) = visit(&element.children, kind, span) {
                        return Some(id);
                    }
                }
                TemplateChild::Fragment(fragment) => {
                    if let Some(id) = visit(&fragment.children, kind, span) {
                        return Some(id);
                    }
                }
                TemplateChild::Conditional(conditional) => {
                    if kind == TemplateSemanticKind::Conditional && conditional.span == span {
                        return Some(conditional.id.0.clone());
                    }
                    if let Some(id) = visit(&conditional.when_true, kind, span)
                        .or_else(|| visit(&conditional.when_false, kind, span))
                    {
                        return Some(id);
                    }
                }
                TemplateChild::List(list) => {
                    if kind == TemplateSemanticKind::List && list.span == span {
                        return Some(list.id.0.clone());
                    }
                    if let Some(id) = visit(&list.item_template, kind, span) {
                        return Some(id);
                    }
                }
                TemplateChild::Text { .. } | TemplateChild::Binding { .. } => {}
            }
        }
        None
    }

    template
        .root
        .as_ref()
        .and_then(|root| visit(&root.children, kind, span))
        .or_else(|| {
            template
                .root_fragment
                .as_ref()
                .and_then(|fragment| visit(&fragment.children, kind, span))
        })
}

#[must_use]
pub fn artifact_from_registry(registry: &RuntimeComponentRegistry) -> RuntimeComponentArtifact {
    RuntimeComponentArtifact {
        schema_version: RUNTIME_COMPONENT_ARTIFACT_SCHEMA_VERSION,
        component_definitions: registry
            .component_definitions
            .iter()
            .map(|r| SerializedComponentDefinition {
                component: r.component.to_string(),
                template: r.template.to_string(),
                slots: r.declared_slots.iter().map(ToString::to_string).collect(),
                boundary: "client".to_string(),
            })
            .collect(),
        instances: registry
            .instances
            .iter()
            .map(|r| SerializedComponentInstance {
                instance: r.instance.to_string(),
                component: r.component.to_string(),
                invocation: r.invocation.as_ref().map(ToString::to_string),
                parent: r.parent.as_ref().map(ToString::to_string),
                depth: r.depth,
                initialization_batch: r.initialization_batch,
                ordinary_template_targets: Vec::new(),
                ordinary_template_bindings: Vec::new(),
                ordinary_template_events: Vec::new(),
                state_slots: Vec::new(),
                computed_slots: Vec::new(),
                context_prefix: r.instance_context_prefix.clone(),
                instruction_indices: r.optimized_instruction_indices.clone(),
                structural_region: r.structural_region.as_ref().map(ToString::to_string),
            })
            .collect(),
        initialization_batches: registry
            .initialization_batches
            .iter()
            .map(|r| SerializedComponentBatch {
                index: r.index,
                instances: r.instances.iter().map(ToString::to_string).collect(),
                context_sources: r.context_sources.iter().map(ToString::to_string).collect(),
                slot_bindings: r.slot_bindings.iter().map(ToString::to_string).collect(),
            })
            .collect(),
        slot_binding_programs: registry
            .slot_bindings
            .iter()
            .map(|r| SerializedSlotBinding {
                binding: r.binding.to_string(),
                caller_instance: r.caller_instance.to_string(),
                callee_instance: r.callee_instance.to_string(),
                slot: r.slot.to_string(),
                outlet: r.outlet.to_string(),
                fragment: r.fragment.to_string(),
                content_owner_instance: r.content_owner_instance.to_string(),
            })
            .collect(),
        instance_context_bindings: registry
            .instance_context_bindings
            .iter()
            .map(|r| SerializedInstanceContextBinding {
                consumer_instance: r.consumer_instance.to_string(),
                selected_source: r.selected_source.to_string(),
                provider_source: r.provider_source.as_ref().map(ToString::to_string),
                runtime_slot: r.runtime_slot.to_string(),
            })
            .collect(),
        ordinary_template_targets: Vec::new(),
        ordinary_template_bindings: Vec::new(),
        ordinary_template_events: Vec::new(),
        destruction: SerializedDestructionMetadata {
            operation: "destroy_component_instance".to_string(),
            enabled: true,
        },
        structural_programs: Vec::new(),
    }
}

/// # Panics
///
/// Panics only if compiler-owned artifact data cannot be serialized.
#[must_use]
pub fn runtime_component_artifact_json(artifact: &RuntimeComponentArtifact) -> String {
    serde_json::to_string_pretty(artifact).expect("component runtime artifact serializes") + "\n"
}

/// # Errors
///
/// Returns an error for an unsupported schema or invalid canonical endpoints/order.
#[allow(clippy::too_many_lines)]
pub fn validate_runtime_component_artifact(
    artifact: &RuntimeComponentArtifact,
) -> Result<(), String> {
    if artifact.schema_version != RUNTIME_COMPONENT_ARTIFACT_SCHEMA_VERSION {
        return Err("unsupported component runtime artifact schema version".to_string());
    }
    let structural_occurrence_invocations = artifact
        .structural_programs
        .iter()
        .flat_map(|program| &program.template_occurrences)
        .map(|occurrence| occurrence.invocation.as_str())
        .collect::<std::collections::BTreeSet<_>>();
    if artifact.structural_programs.iter().any(|program| {
        program.region.is_empty()
            || program.host_component.is_empty()
            || program.host_node.is_empty()
            || program.host_template_entity.is_empty()
            || program.conditional_host_fragments.iter().any(|fragments| {
                let known_invocations = program
                    .template_occurrences
                    .iter()
                    .map(|occurrence| occurrence.invocation.as_str())
                    .collect::<std::collections::BTreeSet<_>>();
                fragments.host_instance.is_empty()
                    || !matches!(
                        fragments.host_scope.as_str(),
                        "static-instance" | "structural-occurrence"
                    )
                    || fragments.when_true_html.is_empty()
                    || fragments.when_false_html.is_empty()
                    || fragments.when_true_invocations.len()
                        != fragments
                            .when_true_invocations
                            .iter()
                            .collect::<std::collections::BTreeSet<_>>()
                            .len()
                    || fragments.when_false_invocations.len()
                        != fragments
                            .when_false_invocations
                            .iter()
                            .collect::<std::collections::BTreeSet<_>>()
                            .len()
                    || fragments
                        .when_true_invocations
                        .iter()
                        .chain(&fragments.when_false_invocations)
                        .any(|invocation| !known_invocations.contains(invocation.as_str()))
            })
            || program.keyed_host_fragments.iter().any(|fragments| {
                let known_invocations = program
                    .template_occurrences
                    .iter()
                    .map(|occurrence| occurrence.invocation.as_str())
                    .collect::<std::collections::BTreeSet<_>>();
                fragments.host_instance.is_empty()
                    || !matches!(
                        fragments.host_scope.as_str(),
                        "static-instance" | "structural-occurrence"
                    )
                    || fragments.item_template_html.is_empty()
                    || fragments.item_invocations.len()
                        != fragments
                            .item_invocations
                            .iter()
                            .collect::<std::collections::BTreeSet<_>>()
                            .len()
                    || fragments
                        .item_invocations
                        .iter()
                        .any(|invocation| !known_invocations.contains(invocation.as_str()))
            })
            || program.template_occurrences.len() != program.template_instances.len()
            || program.template_occurrences.iter().any(|occurrence| {
                occurrence.template_html.is_empty()
                    || occurrence.invocation_template_entity.is_empty()
                    || occurrence.nested_invocations.len()
                        != occurrence
                            .nested_invocations
                            .iter()
                            .collect::<std::collections::BTreeSet<_>>()
                            .len()
                    || occurrence.nested_invocations.iter().any(|invocation| {
                        !structural_occurrence_invocations.contains(invocation.as_str())
                    })
            })
            || program
                .template_occurrences
                .iter()
                .map(|occurrence| occurrence.template_instance.as_str())
                .collect::<Vec<_>>()
                != program.template_instances
            || program.create_order != program.template_instances
            || program
                .destroy_order
                .iter()
                .rev()
                .cloned()
                .collect::<Vec<_>>()
                != program.template_instances
    }) {
        return Err("component artifact has invalid structural program ordering".to_string());
    }
    let structural_regions = artifact
        .structural_programs
        .iter()
        .map(|program| program.region.as_str())
        .collect::<std::collections::BTreeSet<_>>();
    let structural_hosts = artifact
        .structural_programs
        .iter()
        .map(|program| (program.host_component.as_str(), program.host_node.as_str()))
        .collect::<std::collections::BTreeSet<_>>();
    if structural_regions.len() != artifact.structural_programs.len()
        || structural_hosts.len() != artifact.structural_programs.len()
    {
        return Err("component artifact has duplicate structural program addresses".to_string());
    }
    let instances = artifact
        .instances
        .iter()
        .map(|r| r.instance.as_str())
        .collect::<std::collections::BTreeSet<_>>();
    let instance_components = artifact
        .instances
        .iter()
        .map(|instance| (instance.instance.as_str(), instance.component.as_str()))
        .collect::<std::collections::BTreeMap<_, _>>();
    let structural_template_instances = artifact
        .structural_programs
        .iter()
        .flat_map(|program| program.template_instances.iter().map(String::as_str))
        .collect::<std::collections::BTreeSet<_>>();
    let structural_template_components = artifact
        .structural_programs
        .iter()
        .flat_map(|program| {
            program.template_occurrences.iter().map(|occurrence| {
                (
                    occurrence.template_instance.as_str(),
                    occurrence.component.as_str(),
                )
            })
        })
        .collect::<std::collections::BTreeMap<_, _>>();
    let slot_binding_callees = artifact
        .slot_binding_programs
        .iter()
        .map(|binding| (binding.binding.as_str(), binding.callee_instance.as_str()))
        .collect::<std::collections::BTreeMap<_, _>>();
    let slot_projection_programs_valid = |bindings: &[String],
                                         programs: &[SerializedStructuralSlotProjectionProgram],
                                         host: &str| {
        bindings.len() == programs.len()
            && bindings.iter().collect::<std::collections::BTreeSet<_>>().len()
                == bindings.len()
            && programs
                .iter()
                .map(|program| program.binding.as_str())
                .collect::<std::collections::BTreeSet<_>>()
                .len()
                == programs.len()
            && programs.iter().all(|program| {
                let Some(slot_binding) = artifact
                    .slot_binding_programs
                    .iter()
                    .find(|binding| binding.binding == program.binding)
                else {
                    return false;
                };
                let targets = program
                    .target_ids
                    .iter()
                    .collect::<std::collections::BTreeSet<_>>();
                bindings.contains(&program.binding)
                    && slot_binding.callee_instance == host
                    && slot_binding.caller_instance == program.caller_instance
                    && slot_binding.content_owner_instance == program.content_owner_instance
                    && targets.len() == program.target_ids.len()
                    && program
                        .binding_ids
                        .iter()
                        .collect::<std::collections::BTreeSet<_>>()
                        .len()
                        == program.binding_ids.len()
                    && program
                        .event_ids
                        .iter()
                        .collect::<std::collections::BTreeSet<_>>()
                        .len()
                        == program.event_ids.len()
                    && program
                        .nested_invocations
                        .iter()
                        .collect::<std::collections::BTreeSet<_>>()
                        .len()
                        == program.nested_invocations.len()
                    && program.target_ids.iter().all(|id| {
                        artifact.ordinary_template_targets.iter().any(|target| {
                            target.id == *id && target.component_instance_id == program.caller_instance
                        })
                    })
                    && program.binding_ids.iter().all(|id| {
                        artifact.ordinary_template_bindings.iter().any(|binding| {
                            binding.id == *id
                                && binding.component_instance_id == program.caller_instance
                                && targets.contains(&binding.target_id)
                        })
                    })
                    && program.event_ids.iter().all(|id| {
                        artifact.ordinary_template_events.iter().any(|event| {
                            event.declaration_event_id == *id
                                && event.component_instance_id == program.caller_instance
                                && targets.contains(&event.target_id)
                        })
                    })
            })
    };
    if artifact.structural_programs.iter().any(|program| {
        let host_instances = program
            .conditional_host_fragments
            .iter()
            .map(|fragments| fragments.host_instance.as_str())
            .collect::<std::collections::BTreeSet<_>>();
        host_instances.len() != program.conditional_host_fragments.len()
            || program.conditional_host_fragments.iter().any(|fragments| {
                (match fragments.host_scope.as_str() {
                    "static-instance" => {
                        !instances.contains(fragments.host_instance.as_str())
                            || instance_components
                                .get(fragments.host_instance.as_str())
                                .is_none_or(|component| *component != program.host_component)
                    }
                    "structural-occurrence" => {
                        !structural_template_instances.contains(fragments.host_instance.as_str())
                            || structural_template_components
                                .get(fragments.host_instance.as_str())
                                .is_none_or(|component| *component != program.host_component)
                    }
                    _ => true,
                }) || {
                    fragments.slot_projection_bindings.len()
                        != fragments
                            .slot_projection_bindings
                            .iter()
                            .collect::<std::collections::BTreeSet<_>>()
                            .len()
                        || fragments.slot_projection_bindings.iter().any(|binding| {
                            slot_binding_callees
                                .get(binding.as_str())
                                .is_none_or(|callee| *callee != fragments.host_instance)
                        })
                        || !slot_projection_programs_valid(
                            &fragments.slot_projection_bindings,
                            &fragments.slot_projection_programs,
                            &fragments.host_instance,
                        )
                }
            })
    }) {
        return Err("component artifact has invalid conditional host fragments".to_string());
    }
    if artifact.structural_programs.iter().any(|program| {
        let host_instances = program
            .keyed_host_fragments
            .iter()
            .map(|fragments| fragments.host_instance.as_str())
            .collect::<std::collections::BTreeSet<_>>();
        host_instances.len() != program.keyed_host_fragments.len()
            || program.keyed_host_fragments.iter().any(|fragments| {
                (match fragments.host_scope.as_str() {
                    "static-instance" => {
                        !instances.contains(fragments.host_instance.as_str())
                            || instance_components
                                .get(fragments.host_instance.as_str())
                                .is_none_or(|component| *component != program.host_component)
                    }
                    "structural-occurrence" => {
                        !structural_template_instances.contains(fragments.host_instance.as_str())
                            || structural_template_components
                                .get(fragments.host_instance.as_str())
                                .is_none_or(|component| *component != program.host_component)
                    }
                    _ => true,
                }) || {
                    fragments.slot_projection_bindings.len()
                        != fragments
                            .slot_projection_bindings
                            .iter()
                            .collect::<std::collections::BTreeSet<_>>()
                            .len()
                        || fragments.slot_projection_bindings.iter().any(|binding| {
                            slot_binding_callees
                                .get(binding.as_str())
                                .is_none_or(|callee| *callee != fragments.host_instance)
                        })
                        || !slot_projection_programs_valid(
                            &fragments.slot_projection_bindings,
                            &fragments.slot_projection_programs,
                            &fragments.host_instance,
                        )
                }
            })
    }) {
        return Err("component artifact has invalid keyed host fragments".to_string());
    }
    let target_ids = artifact
        .ordinary_template_targets
        .iter()
        .map(|target| target.id.as_str())
        .collect::<std::collections::BTreeSet<_>>();
    let binding_ids = artifact
        .ordinary_template_bindings
        .iter()
        .map(|binding| binding.id.as_str())
        .collect::<std::collections::BTreeSet<_>>();
    let event_keys = artifact
        .ordinary_template_events
        .iter()
        .map(|event| {
            (
                event.component_instance_id.as_str(),
                event.target_id.as_str(),
                event.event_type.as_str(),
            )
        })
        .collect::<std::collections::BTreeSet<_>>();
    if artifact
        .structural_programs
        .iter()
        .flat_map(|program| {
            program.template_occurrences.iter().map(move |occurrence| {
                (
                    occurrence,
                    artifact
                        .ordinary_template_targets
                        .iter()
                        .filter(|target| {
                            target.component_instance_id == occurrence.template_instance
                        })
                        .map(|target| target.id.clone())
                        .collect::<Vec<_>>(),
                    artifact
                        .ordinary_template_bindings
                        .iter()
                        .filter(|binding| {
                            binding.component_instance_id == occurrence.template_instance
                        })
                        .map(|binding| binding.id.clone())
                        .collect::<Vec<_>>(),
                    artifact
                        .ordinary_template_events
                        .iter()
                        .filter(|event| event.component_instance_id == occurrence.template_instance)
                        .map(|event| event.declaration_event_id.clone())
                        .collect::<Vec<_>>(),
                )
            })
        })
        .any(|(occurrence, targets, bindings, events)| {
            occurrence.ordinary_template_targets != targets
                || occurrence.ordinary_template_bindings != bindings
                || occurrence.ordinary_template_events != events
        })
    {
        return Err("component artifact has invalid structural template projection".to_string());
    }
    let mut structural_computed_cache_slots = std::collections::BTreeSet::new();
    let mut structural_computed_dirty_slots = std::collections::BTreeSet::new();
    let mut structural_computed_instance_pairs = std::collections::BTreeSet::new();
    let mut structural_state_slots = std::collections::BTreeSet::new();
    let mut structural_state_instance_pairs = std::collections::BTreeSet::new();
    if artifact
        .structural_programs
        .iter()
        .flat_map(|program| &program.template_occurrences)
        .any(|occurrence| {
            occurrence.state_slots.iter().any(|slot| {
                slot.slot_id
                    != canonical_state_slot_text(&occurrence.template_instance, &slot.storage_id)
                    || slot.storage_id != format!("storage:{}", slot.state_id)
                    || slot.state_id.is_empty()
                    || slot.semantic_type.is_empty()
                    || (!slot.serializable && slot.resume_codec.is_some())
                    || !structural_state_slots.insert(slot.slot_id.as_str())
                    || !structural_state_instance_pairs.insert((
                        occurrence.template_instance.as_str(),
                        slot.storage_id.as_str(),
                    ))
            }) || occurrence.computed_slots.iter().any(|slot| {
                !slot
                    .cache_slot_id
                    .starts_with(&format!("{}/computed-cache:", occurrence.template_instance))
                    || !slot
                        .dirty_slot_id
                        .starts_with(&format!("{}/computed-dirty:", occurrence.template_instance))
                    || slot.computed_id.is_empty()
                    || !structural_computed_cache_slots.insert(slot.cache_slot_id.as_str())
                    || !structural_computed_dirty_slots.insert(slot.dirty_slot_id.as_str())
                    || !structural_computed_instance_pairs.insert((
                        occurrence.template_instance.as_str(),
                        slot.computed_id.as_str(),
                    ))
            })
        })
    {
        return Err("component artifact has invalid structural instance slots".to_string());
    }
    let mut computed_cache_slots = std::collections::BTreeSet::new();
    let mut computed_dirty_slots = std::collections::BTreeSet::new();
    let mut computed_instance_pairs = std::collections::BTreeSet::new();
    let mut state_slots = std::collections::BTreeSet::new();
    let mut state_instance_pairs = std::collections::BTreeSet::new();
    if target_ids.len() != artifact.ordinary_template_targets.len()
        || binding_ids.len() != artifact.ordinary_template_bindings.len()
        || event_keys.len() != artifact.ordinary_template_events.len()
        || artifact.ordinary_template_targets.iter().any(|target| {
            !instances.contains(target.component_instance_id.as_str())
                && !structural_template_instances.contains(target.component_instance_id.as_str())
                || !target.id.starts_with(&format!(
                    "{}/template-target:",
                    target.component_instance_id
                ))
        })
        || artifact.ordinary_template_bindings.iter().any(|binding| {
            !instances.contains(binding.component_instance_id.as_str())
                && !structural_template_instances.contains(binding.component_instance_id.as_str())
                || !target_ids.contains(binding.target_id.as_str())
                || !binding.id.starts_with(&format!(
                    "{}/template-binding:",
                    binding.component_instance_id
                ))
        })
        || artifact.ordinary_template_events.iter().any(|event| {
            !instances.contains(event.component_instance_id.as_str())
                && !structural_template_instances.contains(event.component_instance_id.as_str())
                || !target_ids.contains(event.target_id.as_str())
                || event.action_batch_id.is_none()
        })
        || artifact.instances.iter().any(|instance| {
            instance
                .ordinary_template_targets
                .iter()
                .any(|id| !target_ids.contains(id.as_str()))
                || instance
                    .ordinary_template_bindings
                    .iter()
                    .any(|id| !binding_ids.contains(id.as_str()))
                || instance.ordinary_template_targets
                    != artifact
                        .ordinary_template_targets
                        .iter()
                        .filter(|target| target.component_instance_id == instance.instance)
                        .map(|target| target.id.clone())
                        .collect::<Vec<_>>()
                || instance.ordinary_template_bindings
                    != artifact
                        .ordinary_template_bindings
                        .iter()
                        .filter(|binding| binding.component_instance_id == instance.instance)
                        .map(|binding| binding.id.clone())
                        .collect::<Vec<_>>()
                || instance.ordinary_template_events
                    != artifact
                        .ordinary_template_events
                        .iter()
                        .filter(|event| event.component_instance_id == instance.instance)
                        .map(|event| event.declaration_event_id.clone())
                        .collect::<Vec<_>>()
                || instance.state_slots.iter().any(|slot| {
                    slot.slot_id != canonical_state_slot_text(&instance.instance, &slot.storage_id)
                        || slot.storage_id != format!("storage:{}", slot.state_id)
                        || !state_slots.insert(slot.slot_id.as_str())
                        || !state_instance_pairs
                            .insert((instance.instance.as_str(), slot.storage_id.as_str()))
                })
                || instance.computed_slots.iter().any(|slot| {
                    !slot
                        .cache_slot_id
                        .starts_with(&format!("{}/computed-cache:", instance.instance))
                        || !slot
                            .dirty_slot_id
                            .starts_with(&format!("{}/computed-dirty:", instance.instance))
                        || !computed_cache_slots.insert(slot.cache_slot_id.as_str())
                        || !computed_dirty_slots.insert(slot.dirty_slot_id.as_str())
                        || !computed_instance_pairs
                            .insert((instance.instance.as_str(), slot.computed_id.as_str()))
                })
        })
    {
        return Err("component artifact has invalid ordinary template projection".to_string());
    }
    if artifact.instances.iter().any(|r| {
        r.parent.as_deref().is_some_and(|parent| {
            !instances.contains(parent) && !structural_template_instances.contains(parent)
        })
    }) {
        return Err("component artifact has an unknown parent instance".to_string());
    }
    if artifact
        .structural_programs
        .iter()
        .flat_map(|program| &program.template_occurrences)
        .any(|occurrence| {
            occurrence.parent_template_instance.is_empty()
                || (!instances.contains(occurrence.parent_template_instance.as_str())
                    && !structural_template_instances
                        .contains(occurrence.parent_template_instance.as_str()))
        })
    {
        return Err("component artifact has an unknown structural template parent".to_string());
    }
    if artifact.slot_binding_programs.iter().any(|r| {
        !instances.contains(r.caller_instance.as_str())
            || !instances.contains(r.callee_instance.as_str())
    }) {
        return Err("component artifact has an unknown Slot-binding endpoint".to_string());
    }
    if artifact
        .initialization_batches
        .iter()
        .enumerate()
        .any(|(index, batch)| {
            batch.index != index
                || batch.instances.iter().any(|id| {
                    !instances.contains(id.as_str())
                        && !structural_template_instances.contains(id.as_str())
                })
        })
    {
        return Err("component artifact has invalid initialization ordering".to_string());
    }
    Ok(())
}

fn canonical_state_slot_text(instance: &str, storage: &str) -> String {
    let encoded = storage.bytes().fold(String::new(), |mut encoded, byte| {
        if byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'.' | b'_' | b'~') {
            encoded.push(char::from(byte));
        } else {
            use std::fmt::Write as _;
            write!(encoded, "%{byte:02X}").expect("writing to a String cannot fail");
        }
        encoded
    });
    format!("{instance}/state-slot:{encoded}")
}

#[cfg(test)]
mod tests {
    use crate::{
        build_application_semantic_model, build_runtime_component_artifact,
        runtime_component_artifact_json, validate_runtime_component_artifact,
    };
    #[test]
    fn serializes_deterministically_and_rejects_unknown_endpoints() {
        let model = build_application_semantic_model(&presolve_parser::parse_file(
            "src/Artifact.tsx",
            r#"@component("x-page") class Page extends Component { render() { return <main />; } }"#,
        ));
        let mut artifact =
            build_runtime_component_artifact(&model, &model.component_ir_optimization);
        assert!(validate_runtime_component_artifact(&artifact).is_ok());
        assert_eq!(
            runtime_component_artifact_json(&artifact),
            runtime_component_artifact_json(&build_runtime_component_artifact(
                &model,
                &model.component_ir_optimization
            ))
        );
        artifact.instances[0].parent = Some("missing".to_string());
        assert!(validate_runtime_component_artifact(&artifact).is_err());
    }

    #[test]
    fn projects_exact_computed_slots_for_each_repeated_instance() {
        let model = build_application_semantic_model(&presolve_parser::parse_file(
            "src/RepeatedComputedArtifact.tsx",
            r#"@component("x-child") class Child { count = state(1); @computed() get doubled() { return this.count * 2; } render() { return <span>{this.doubled}</span>; } }
@component("x-parent") class Parent { render() { return <><Child /><Child /></>; } }"#,
        ));
        let mut artifact =
            build_runtime_component_artifact(&model, &model.component_ir_optimization);
        let slots = artifact
            .instances
            .iter()
            .flat_map(|instance| instance.computed_slots.iter())
            .collect::<Vec<_>>();
        assert_eq!(slots.len(), 2);
        assert_ne!(slots[0].cache_slot_id, slots[1].cache_slot_id);
        assert_ne!(slots[0].dirty_slot_id, slots[1].dirty_slot_id);
        assert!(validate_runtime_component_artifact(&artifact).is_ok());
        let computed_instances = artifact
            .instances
            .iter()
            .enumerate()
            .filter(|(_, instance)| !instance.computed_slots.is_empty())
            .map(|(index, _)| index)
            .collect::<Vec<_>>();
        let duplicate = artifact.instances[computed_instances[0]].computed_slots[0].clone();
        artifact.instances[computed_instances[1]].computed_slots[0].cache_slot_id =
            duplicate.cache_slot_id;
        assert!(validate_runtime_component_artifact(&artifact).is_err());
    }

    #[test]
    fn projects_and_validates_exact_state_slots_for_repeated_instances() {
        let model = build_application_semantic_model(&presolve_parser::parse_file(
            "src/RepeatedStateArtifact.tsx",
            r#"@component("x-child") class Child { count = state(1); render() { return <span>{this.count}</span>; } }
@component("x-parent") class Parent { render() { return <><Child /><Child /></>; } }"#,
        ));
        let mut artifact =
            build_runtime_component_artifact(&model, &model.component_ir_optimization);
        let state_instances = artifact
            .instances
            .iter()
            .enumerate()
            .filter(|(_, instance)| !instance.state_slots.is_empty())
            .map(|(index, _)| index)
            .collect::<Vec<_>>();
        assert_eq!(state_instances.len(), 2);
        let first = &artifact.instances[state_instances[0]].state_slots[0];
        let second = &artifact.instances[state_instances[1]].state_slots[0];
        assert_eq!(first.storage_id, second.storage_id);
        assert_ne!(first.slot_id, second.slot_id);
        assert!(validate_runtime_component_artifact(&artifact).is_ok());

        artifact.instances[state_instances[1]].state_slots[0].slot_id = first.slot_id.clone();
        assert!(validate_runtime_component_artifact(&artifact).is_err());
    }

    #[test]
    fn structural_programs_name_the_exact_compiler_owned_runtime_hosts() {
        let model = build_application_semantic_model(&presolve_parser::parse_file(
            "src/StructuralHostArtifact.tsx",
            r#"
@component("x-leaf") class Leaf extends Component {
  count = state(0);
  @action() increment() { this.count++; }
  @computed() get doubled() { return this.count * 2; }
  render() { return <button onClick={() => this.increment()}>{this.doubled}</button>; }
}
@component("x-page") class Page extends Component {
  visible = state(true);
  items = state([{ id: "a" }]);
  render() { return <main>{this.visible ? <Leaf /> : <span>Hidden</span>}<ul>{this.items.map(item => <li key={item.id}><Leaf /></li>)}</ul></main>; }
}
@component("x-idle") class Idle extends Component { render() { return <aside />; } }
"#,
        ));
        let mut artifact =
            build_runtime_component_artifact(&model, &model.component_ir_optimization);
        let manifest = crate::build_template_manifest_from_asm(&model);

        assert_eq!(artifact.structural_programs.len(), 2);
        assert!(
            validate_runtime_component_artifact(&artifact).is_ok(),
            "{:?}",
            validate_runtime_component_artifact(&artifact)
        );
        for program in &artifact.structural_programs {
            assert!(!program.host_template_entity.is_empty());
            assert_eq!(
                program
                    .template_occurrences
                    .iter()
                    .map(|occurrence| occurrence.template_instance.as_str())
                    .collect::<Vec<_>>(),
                program
                    .template_instances
                    .iter()
                    .map(String::as_str)
                    .collect::<Vec<_>>()
            );
            assert!(program.template_occurrences.iter().all(|occurrence| {
                !occurrence.invocation.is_empty()
                    && !occurrence.invocation_template_entity.is_empty()
                    && !occurrence.component.is_empty()
            }));
            for occurrence in &program.template_occurrences {
                assert_eq!(
                    occurrence.ordinary_template_targets,
                    artifact
                        .ordinary_template_targets
                        .iter()
                        .filter(
                            |target| target.component_instance_id == occurrence.template_instance
                        )
                        .map(|target| target.id.clone())
                        .collect::<Vec<_>>()
                );
                assert_eq!(
                    occurrence.ordinary_template_bindings,
                    artifact
                        .ordinary_template_bindings
                        .iter()
                        .filter(
                            |binding| binding.component_instance_id == occurrence.template_instance
                        )
                        .map(|binding| binding.id.clone())
                        .collect::<Vec<_>>()
                );
                assert_eq!(
                    occurrence.ordinary_template_events,
                    artifact
                        .ordinary_template_events
                        .iter()
                        .filter(|event| event.component_instance_id == occurrence.template_instance)
                        .map(|event| event.declaration_event_id.clone())
                        .collect::<Vec<_>>()
                );
                assert!(occurrence.template_html.contains("data-presolve-node"));
                assert!(occurrence.state_slots.iter().all(|slot| {
                    slot.slot_id
                        .starts_with(&format!("{}/state-slot:", occurrence.template_instance))
                }));
                assert!(occurrence.computed_slots.iter().all(|slot| {
                    slot.cache_slot_id
                        .starts_with(&format!("{}/computed-cache:", occurrence.template_instance))
                        && slot.dirty_slot_id.starts_with(&format!(
                            "{}/computed-dirty:",
                            occurrence.template_instance
                        ))
                }));
            }
            let component = manifest
                .components
                .iter()
                .find(|component| component.component_id == program.host_component)
                .expect("structural program host component");
            assert!(component.template.nodes.iter().any(|node| {
                matches!(
                    node,
                    crate::template_manifest::ManifestNode::Conditional { id, .. }
                        | crate::template_manifest::ManifestNode::List { id, .. }
                        if id == &program.host_node
                )
            }));
        }

        assert!(artifact
            .structural_programs
            .iter()
            .flat_map(|program| &program.template_occurrences)
            .any(|occurrence| {
                !occurrence.ordinary_template_targets.is_empty()
                    && !occurrence.ordinary_template_bindings.is_empty()
                    && !occurrence.ordinary_template_events.is_empty()
                    && occurrence
                        .template_html
                        .contains("__PRESOLVE_STRUCTURAL_OCCURRENCE__")
            }));

        let conditional = artifact
            .structural_programs
            .iter()
            .find(|program| !program.conditional_host_fragments.is_empty())
            .expect("conditional host has compiler-authored fragments");
        assert_eq!(conditional.conditional_host_fragments.len(), 1);
        let fragments = &conditional.conditional_host_fragments[0];
        assert!(artifact
            .instances
            .iter()
            .any(|instance| instance.instance == fragments.host_instance));
        assert!(fragments
            .when_true_html
            .contains("data-presolve-structural-invocation="));
        assert_eq!(
            fragments.when_true_invocations,
            crate::ordinary_html_codegen::structural_invocations_in_compiler_html(
                &fragments.when_true_html
            )
        );
        assert_eq!(
            fragments.when_false_invocations,
            crate::ordinary_html_codegen::structural_invocations_in_compiler_html(
                &fragments.when_false_html
            )
        );
        assert!(fragments.when_false_html.contains("Hidden"));

        let keyed = artifact
            .structural_programs
            .iter()
            .find(|program| !program.keyed_host_fragments.is_empty())
            .expect("keyed host has compiler-authored fragments");
        let keyed_fragments = &keyed.keyed_host_fragments[0];
        assert!(keyed_fragments
            .item_template_html
            .contains("__ez_list_key__"));
        assert!(keyed_fragments
            .item_template_html
            .contains("data-presolve-structural-invocation="));
        assert_eq!(
            keyed_fragments.item_invocations,
            crate::ordinary_html_codegen::structural_invocations_in_compiler_html(
                &keyed_fragments.item_template_html
            )
        );

        let mut invalid_fragments = artifact.clone();
        invalid_fragments
            .structural_programs
            .iter_mut()
            .find(|program| !program.conditional_host_fragments.is_empty())
            .expect("conditional host has compiler-authored fragments")
            .conditional_host_fragments[0]
            .when_false_html
            .clear();
        assert!(validate_runtime_component_artifact(&invalid_fragments).is_err());

        let mut invalid_membership = artifact.clone();
        invalid_membership
            .structural_programs
            .iter_mut()
            .find(|program| !program.conditional_host_fragments.is_empty())
            .expect("conditional host has compiler-authored fragments")
            .conditional_host_fragments[0]
            .when_true_invocations
            .push("fabricated-invocation".to_string());
        assert!(validate_runtime_component_artifact(&invalid_membership).is_err());

        let mut invalid_keyed = artifact.clone();
        invalid_keyed
            .structural_programs
            .iter_mut()
            .find(|program| !program.keyed_host_fragments.is_empty())
            .expect("keyed host has compiler-authored fragments")
            .keyed_host_fragments[0]
            .item_template_html
            .clear();
        assert!(validate_runtime_component_artifact(&invalid_keyed).is_err());

        let mut invalid_keyed_membership = artifact.clone();
        invalid_keyed_membership
            .structural_programs
            .iter_mut()
            .find(|program| !program.keyed_host_fragments.is_empty())
            .expect("keyed host has compiler-authored fragments")
            .keyed_host_fragments[0]
            .item_invocations
            .push("fabricated-invocation".to_string());
        assert!(validate_runtime_component_artifact(&invalid_keyed_membership).is_err());

        let mut invalid_host = artifact.clone();
        let idle = invalid_host
            .instances
            .iter()
            .find(|instance| instance.component.ends_with("/component:x-idle"))
            .expect("unrelated static host instance")
            .instance
            .clone();
        invalid_host
            .structural_programs
            .iter_mut()
            .find(|program| !program.conditional_host_fragments.is_empty())
            .expect("conditional host has compiler-authored fragments")
            .conditional_host_fragments[0]
            .host_instance = idle;
        assert!(validate_runtime_component_artifact(&invalid_host).is_err());

        let mut invalid_state_slot = artifact.clone();
        invalid_state_slot.structural_programs[0].template_occurrences[0].state_slots[0]
            .storage_id = "storage:fabricated".to_string();
        assert!(validate_runtime_component_artifact(&invalid_state_slot).is_err());

        let mut invalid_computed_slot = artifact.clone();
        invalid_computed_slot.structural_programs[0].template_occurrences[0].computed_slots[0]
            .cache_slot_id = "fabricated-cache".to_string();
        assert!(validate_runtime_component_artifact(&invalid_computed_slot).is_err());

        let first = &mut artifact.structural_programs[0].template_occurrences[0];
        first
            .ordinary_template_targets
            .push("fabricated-target".to_string());
        assert!(validate_runtime_component_artifact(&artifact).is_err());
    }

    #[test]
    fn structural_host_fragments_preserve_their_compiler_scope() {
        let model = build_application_semantic_model(&presolve_parser::parse_file(
            "src/NestedStructuralHostArtifact.tsx",
            r#"
@component("x-leaf") class Leaf { render() { return <small />; } }
@component("x-card") class Card {
  expanded = state(true);
  render() { return <article>{this.expanded ? <section><Leaf />{this.expanded}</section> : <em>Collapsed</em>}</article>; }
}
@component("x-page") class Page {
  shown = state(true);
  render() { return <main>{this.shown ? <Card /> : <span>Hidden</span>}</main>; }
}
"#,
        ));
        let artifact = build_runtime_component_artifact(&model, &model.component_ir_optimization);

        assert!(validate_runtime_component_artifact(&artifact).is_ok());
        let fragments = artifact
            .structural_programs
            .iter()
            .flat_map(|program| &program.conditional_host_fragments)
            .find(|fragments| fragments.host_scope == "structural-occurrence")
            .expect("nested structural occurrence host fragments");
        assert!(fragments
            .when_true_html
            .contains("__PRESOLVE_STRUCTURAL_OCCURRENCE__"));
        assert!(fragments.when_false_html.contains("Collapsed"));
    }
}
