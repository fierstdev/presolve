use std::collections::BTreeMap;

use serde::Serialize;

use crate::component_graph::{ComponentGraph, ComponentNode, SerializableValue, StateOperation};
use crate::html_codegen::{generate_children_html, generate_list_item_template_html};
use crate::template_graph::{
    AttributeValue, ConditionalNode, ElementNode, FragmentNode, ListNode, TemplateChild,
    TemplateGraph, TemplateNode,
};
use crate::{
    ApplicationSemanticModel, IrStorageId, OrdinaryTemplateBindingKind, OrdinaryTemplateTargetKind,
};

pub const TEMPLATE_MANIFEST_SCHEMA_VERSION: u32 = 5;
const LEGACY_TEMPLATE_MANIFEST_SCHEMA_VERSION: u32 = 1;
const ACTION_TEMPLATE_MANIFEST_SCHEMA_VERSION: u32 = 2;

const fn ordinary_target_kind_text(kind: OrdinaryTemplateTargetKind) -> &'static str {
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

const fn ordinary_binding_kind_text(kind: OrdinaryTemplateBindingKind) -> &'static str {
    match kind {
        OrdinaryTemplateBindingKind::Text => "text",
        OrdinaryTemplateBindingKind::Attribute => "attribute",
        OrdinaryTemplateBindingKind::Property => "property",
        OrdinaryTemplateBindingKind::Conditional => "conditional",
        OrdinaryTemplateBindingKind::List => "list",
        OrdinaryTemplateBindingKind::FormControl => "form_control",
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct TemplateManifest {
    pub schema_version: u32,
    pub components: Vec<ManifestComponent>,
    /// Exact compiler-generated bridges from template control anchors to
    /// instance-qualified Forms programs. Empty for v1/v2 manifests.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub form_bindings: Vec<ManifestFormBinding>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub form_hosts: Vec<ManifestFormHost>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub ordinary_targets: Vec<ManifestOrdinaryTarget>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub ordinary_bindings: Vec<ManifestOrdinaryBinding>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub ordinary_events: Vec<ManifestOrdinaryEvent>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ManifestOrdinaryTarget {
    pub id: String,
    pub component_instance_id: String,
    pub template_entity_id: String,
    pub kind: String,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ManifestOrdinaryBinding {
    pub instance_binding_id: String,
    pub component_instance_id: String,
    pub instance_target_id: String,
    pub declaration_binding_id: String,
    pub kind: String,
    pub program_id: String,
    pub expression: Option<String>,
    pub attribute_name: Option<String>,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ManifestOrdinaryEvent {
    pub component_instance_id: String,
    pub component_id: String,
    pub instance_target_id: String,
    pub declaration_event_id: String,
    pub event_type: String,
    pub handler_method_id: String,
    pub action_batch_id: Option<String>,
    #[serde(
        default,
        skip_serializing_if = "Vec::is_empty",
        serialize_with = "crate::component_graph::runtime_arguments_serde::serialize"
    )]
    pub arguments: Vec<SerializableValue>,
    pub program_id: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ManifestFormBinding {
    pub control_anchor: String,
    pub component_instance_id: String,
    pub instance_target_id: String,
    pub field_binding_id: String,
    pub form_instance_id: String,
    pub input_program: String,
    pub blur_program: String,
    pub channel: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ManifestFormHost {
    pub host_anchor: String,
    pub component_instance_id: String,
    pub instance_target_id: String,
    pub submission_host_id: String,
    pub form_instance_id: String,
    pub submission_plan: String,
    pub submit_action: String,
    pub action_batch: String,
    pub serialization_plan: String,
    pub event: String,
    pub prevent_default: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ManifestComponent {
    pub component_id: String,
    pub name: String,
    pub template: ManifestTemplate,
    pub actions: Vec<ManifestAction>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ManifestTemplate {
    pub nodes: Vec<ManifestNode>,
    pub events: Vec<ManifestEvent>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(tag = "kind")]
pub enum ManifestNode {
    #[serde(rename = "element")]
    Element { id: String, tag: String },

    #[serde(rename = "binding")]
    Binding {
        id: String,
        expression: String,
        initial_value: Option<SerializableValue>,
        #[serde(skip_serializing_if = "Option::is_none")]
        target: Option<ManifestBindingTarget>,
        #[serde(skip_serializing_if = "Option::is_none")]
        element: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        attribute: Option<String>,
    },

    #[serde(rename = "conditional")]
    Conditional {
        id: String,
        start: String,
        end: String,
        condition: String,
        initial_value: Option<SerializableValue>,
        when_true_html: String,
        when_false_html: String,
    },

    #[serde(rename = "list")]
    List {
        id: String,
        start: String,
        end: String,
        iterable: String,
        initial_value: Option<SerializableValue>,
        item_variable: String,
        index_variable: Option<String>,
        key_expression: String,
        item_root: String,
        item_template_html: String,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub enum ManifestBindingTarget {
    #[serde(rename = "attribute")]
    Attribute,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ManifestEvent {
    pub node: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub kind: Option<ManifestEventKind>,
    pub event: String,
    pub handler: String,
    #[serde(
        default,
        skip_serializing_if = "Vec::is_empty",
        serialize_with = "crate::component_graph::runtime_arguments_serde::serialize"
    )]
    pub arguments: Vec<SerializableValue>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub method_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub action_batch_id: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub enum ManifestEventKind {
    #[serde(rename = "action")]
    Action,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ManifestAction {
    pub method: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub method_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub action_batch_id: Option<String>,
    pub operation: ManifestOperation,
    pub field: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub storage_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub operand: Option<SerializableValue>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub enum ManifestOperation {
    #[serde(rename = "increment")]
    Increment,

    #[serde(rename = "decrement")]
    Decrement,

    #[serde(rename = "add_assign")]
    AddAssign,

    #[serde(rename = "subtract_assign")]
    SubtractAssign,

    #[serde(rename = "assign")]
    Assign,

    #[serde(rename = "assign_parameter")]
    AssignParameter,

    #[serde(rename = "toggle")]
    Toggle,
}

#[must_use]
pub fn build_template_manifest(
    component_graph: &ComponentGraph,
    template_graph: &TemplateGraph,
) -> TemplateManifest {
    TemplateManifest {
        schema_version: LEGACY_TEMPLATE_MANIFEST_SCHEMA_VERSION,
        components: template_graph
            .templates
            .iter()
            .map(|template| manifest_component(component_graph, template))
            .collect::<Vec<_>>(),
        form_bindings: Vec::new(),
        form_hosts: Vec::new(),
        ordinary_targets: Vec::new(),
        ordinary_bindings: Vec::new(),
        ordinary_events: Vec::new(),
    }
}

/// Build schema-v2 template metadata from canonical ASM action-batch facts.
#[must_use]
pub fn build_template_manifest_from_asm(model: &ApplicationSemanticModel) -> TemplateManifest {
    let form_bindings = form_binding_bridges(model);
    let form_hosts = form_host_bridges(model);
    let ordinary = crate::build_ordinary_template_instance_registry(model);
    TemplateManifest {
        // A Forms artifact always requires its v3 instance-qualified bridge.
        // Otherwise preserve the pre-Forms v1/v2 compatibility boundary.
        schema_version: if !model.component_instance_plan.instances.is_empty() {
            TEMPLATE_MANIFEST_SCHEMA_VERSION
        } else if !form_bindings.is_empty() || !form_hosts.is_empty() {
            3
        } else if model.effect_trigger_plan.action_batches.is_empty() {
            LEGACY_TEMPLATE_MANIFEST_SCHEMA_VERSION
        } else {
            ACTION_TEMPLATE_MANIFEST_SCHEMA_VERSION
        },
        components: model
            .templates
            .iter()
            .filter_map(|template| {
                let component = model
                    .components
                    .iter()
                    .find(|component| component.class_name == template.component_name)?;
                let mut manifest = manifest_component_for(component, template);
                apply_action_bindings(&mut manifest, &action_bindings(model, component));
                apply_action_storage_ids(&mut manifest, component);
                Some(manifest)
            })
            .collect(),
        form_bindings,
        form_hosts,
        ordinary_targets: ordinary
            .targets
            .iter()
            .map(|target| ManifestOrdinaryTarget {
                id: target.target_id.to_string(),
                component_instance_id: target.component_instance_id.to_string(),
                template_entity_id: target.template_entity_id.to_string(),
                kind: ordinary_target_kind_text(target.target_kind).to_string(),
            })
            .collect(),
        ordinary_bindings: ordinary
            .bindings
            .iter()
            .map(|binding| ManifestOrdinaryBinding {
                instance_binding_id: binding.instance_binding_id.to_string(),
                component_instance_id: binding.component_instance_id.to_string(),
                instance_target_id: binding.target_id.to_string(),
                declaration_binding_id: binding.declaration_binding_id.to_string(),
                kind: ordinary_binding_kind_text(binding.binding_kind).to_string(),
                program_id: binding.existing_program_identity.to_string(),
                expression: binding.expression.clone(),
                attribute_name: binding.attribute_name.clone(),
            })
            .collect(),
        ordinary_events: ordinary
            .events
            .iter()
            .map(|event| ManifestOrdinaryEvent {
                component_instance_id: event.component_instance_id.to_string(),
                component_id: event.component_id.to_string(),
                instance_target_id: event.target_id.to_string(),
                declaration_event_id: event.declaration_event_id.to_string(),
                event_type: event.event_type.clone(),
                handler_method_id: event.handler_method_id.to_string(),
                action_batch_id: event.action_batch_id.as_ref().map(ToString::to_string),
                arguments: event.arguments.clone(),
                program_id: event.existing_event_program_identity.to_string(),
            })
            .collect(),
    }
}

fn form_host_bridges(model: &ApplicationSemanticModel) -> Vec<ManifestFormHost> {
    let mut bridges = model
        .submission_hosts
        .values()
        .flat_map(|host| {
            model
                .optimized_form_ir
                .optimized
                .instances
                .values()
                .filter(move |instance| {
                    instance.form == host.form
                        && model
                            .component_instance_plan
                            .instances
                            .get(&instance.component_instance)
                            .is_some_and(|component| component.component == host.component)
                })
                .filter_map(move |instance| {
                    form_element_runtime_anchor(
                        model,
                        &host.owner_template,
                        &host.owner_template_element,
                    )
                    .map(|host_anchor| ManifestFormHost {
                        host_anchor,
                        component_instance_id: instance.component_instance.to_string(),
                        instance_target_id:
                            crate::TemplateInstanceTargetId::for_component_instance_template_entity(
                                instance.component_instance.clone(),
                                host.owner_template_element.clone(),
                            )
                            .to_string(),
                        submission_host_id: host.id.to_string(),
                        form_instance_id: instance.id.to_string(),
                        submission_plan: host.submission_plan.as_str().to_string(),
                        submit_action: host.submit_action.to_string(),
                        action_batch: host.action_batch.to_string(),
                        serialization_plan: host.serialization_plan.as_str().to_string(),
                        event: host.event.to_string(),
                        prevent_default: host.prevent_default,
                    })
                })
        })
        .collect::<Vec<_>>();
    bridges.sort_by(|left, right| {
        (&left.host_anchor, &left.form_instance_id)
            .cmp(&(&right.host_anchor, &right.form_instance_id))
    });
    bridges
}

#[allow(clippy::items_after_statements)]
fn form_element_runtime_anchor(
    model: &ApplicationSemanticModel,
    template_id: &crate::SemanticId,
    target: &crate::SemanticId,
) -> Option<String> {
    let template = model
        .templates
        .iter()
        .find(|template| &template.id == template_id)?;
    fn find(
        element: &ElementNode,
        template: &TemplateNode,
        path: &str,
        target: &crate::SemanticId,
    ) -> Option<String> {
        if template.id.template_entity("element", path) == *target {
            return Some(element.id.0.clone());
        }
        for (index, child) in element.children.iter().enumerate() {
            if let TemplateChild::Element(child) = child {
                if let Some(anchor) = find(child, template, &format!("{path}.{index}"), target) {
                    return Some(anchor);
                }
            }
        }
        None
    }
    template
        .root
        .as_ref()
        .and_then(|root| find(root, template, "root", target))
}

fn form_binding_bridges(model: &ApplicationSemanticModel) -> Vec<ManifestFormBinding> {
    let mut bridges = Vec::new();
    for binding in model.form_field_bindings.values() {
        for instance in model
            .optimized_form_ir
            .optimized
            .instances
            .values()
            .filter(|instance| instance.form == binding.form)
        {
            if !instance.input.contains_key(&binding.field)
                || !instance.blur.contains_key(&binding.field)
            {
                continue;
            }
            let Some(control_anchor) = form_control_runtime_anchor(model, binding) else {
                continue;
            };
            bridges.push(ManifestFormBinding {
                control_anchor,
                component_instance_id: instance.component_instance.to_string(),
                instance_target_id:
                    crate::TemplateInstanceTargetId::for_component_instance_template_entity(
                        instance.component_instance.clone(),
                        binding.control_entity.clone(),
                    )
                    .to_string(),
                field_binding_id: binding.id.to_string(),
                form_instance_id: instance.id.to_string(),
                input_program: format!("{}/input:{}", instance.id, binding.field),
                blur_program: format!("{}/blur:{}", instance.id, binding.field),
                channel: format!("{:?}", binding.channel),
            });
        }
    }
    bridges.sort_by(|left, right| {
        (
            left.control_anchor.as_str(),
            left.field_binding_id.as_str(),
            left.form_instance_id.as_str(),
        )
            .cmp(&(
                right.control_anchor.as_str(),
                right.field_binding_id.as_str(),
                right.form_instance_id.as_str(),
            ))
    });
    bridges
}

fn form_control_runtime_anchor(
    model: &ApplicationSemanticModel,
    binding: &crate::FormFieldBinding,
) -> Option<String> {
    let template = model
        .templates
        .iter()
        .find(|template| template.id == binding.owner_template)?;
    let root = template.root.as_ref()?;
    form_control_anchor_in_element(root, template, "root", &binding.control_entity)
}

fn form_control_anchor_in_element(
    element: &ElementNode,
    template: &TemplateNode,
    path: &str,
    control: &crate::SemanticId,
) -> Option<String> {
    if template.id.template_entity("element", path) == *control {
        return Some(element.id.0.clone());
    }
    for (index, child) in element.children.iter().enumerate() {
        let TemplateChild::Element(child) = child else {
            continue;
        };
        let child_path = format!("{path}.{index}");
        if let Some(anchor) = form_control_anchor_in_element(child, template, &child_path, control)
        {
            return Some(anchor);
        }
    }
    None
}

/// Serialize a template manifest as pretty JSON.
///
/// # Panics
///
/// Panics if serde cannot serialize the compiler-owned manifest model.
#[must_use]
pub fn template_manifest_json(manifest: &TemplateManifest) -> String {
    serde_json::to_string_pretty(manifest).expect("template manifest should serialize")
}

/// Validate the closed J1-P ordinary-instance projection in a v4 manifest.
///
/// v1-v3 remain declaration-level legacy products and intentionally have no
/// ordinary-instance tables. A v4 manifest may not fall back to those records.
///
/// # Errors
///
/// Returns an error for unsupported schema versions or any missing, duplicate,
/// malformed, or non-reciprocal ordinary-instance record.
pub fn validate_template_manifest(manifest: &TemplateManifest) -> Result<(), String> {
    if manifest.schema_version > TEMPLATE_MANIFEST_SCHEMA_VERSION || manifest.schema_version == 0 {
        return Err("unsupported template manifest schema version".to_string());
    }
    if manifest.schema_version != TEMPLATE_MANIFEST_SCHEMA_VERSION {
        if !manifest.ordinary_targets.is_empty()
            || !manifest.ordinary_bindings.is_empty()
            || !manifest.ordinary_events.is_empty()
        {
            return Err("legacy template manifest contains ordinary instance records".to_string());
        }
        return Ok(());
    }
    let targets = manifest
        .ordinary_targets
        .iter()
        .map(|record| record.id.as_str())
        .collect::<std::collections::BTreeSet<_>>();
    let bindings = manifest
        .ordinary_bindings
        .iter()
        .map(|record| record.instance_binding_id.as_str())
        .collect::<std::collections::BTreeSet<_>>();
    if targets.len() != manifest.ordinary_targets.len()
        || bindings.len() != manifest.ordinary_bindings.len()
        || manifest.ordinary_targets.iter().any(|record| {
            record.component_instance_id.is_empty()
                || !record.id.starts_with(&format!(
                    "{}/template-target:",
                    record.component_instance_id
                ))
        })
        || manifest.ordinary_bindings.iter().any(|record| {
            record.component_instance_id.is_empty()
                || !targets.contains(record.instance_target_id.as_str())
                || !record.instance_binding_id.starts_with(&format!(
                    "{}/template-binding:",
                    record.component_instance_id
                ))
        })
        || manifest.ordinary_events.iter().any(|record| {
            record.component_instance_id.is_empty()
                || !targets.contains(record.instance_target_id.as_str())
                || record.action_batch_id.is_none()
        })
        || manifest.form_bindings.iter().any(|record| {
            !targets.contains(record.instance_target_id.as_str())
                || !record.instance_target_id.starts_with(&format!(
                    "{}/template-target:",
                    record.component_instance_id
                ))
        })
        || manifest.form_hosts.iter().any(|record| {
            !targets.contains(record.instance_target_id.as_str())
                || !record.instance_target_id.starts_with(&format!(
                    "{}/template-target:",
                    record.component_instance_id
                ))
        })
    {
        return Err("template manifest has an invalid ordinary instance projection".to_string());
    }
    Ok(())
}

fn manifest_component(
    component_graph: &ComponentGraph,
    template: &TemplateNode,
) -> ManifestComponent {
    component_graph
        .components
        .iter()
        .find(|component| component.class_name == template.component_name)
        .map_or_else(
            || ManifestComponent {
                component_id: String::new(),
                name: template.component_name.clone(),
                template: ManifestTemplate {
                    nodes: Vec::new(),
                    events: Vec::new(),
                },
                actions: Vec::new(),
            },
            |component| manifest_component_for(component, template),
        )
}

fn manifest_component_for(component: &ComponentNode, template: &TemplateNode) -> ManifestComponent {
    let mut nodes = Vec::new();
    let mut events = Vec::new();

    if let Some(root) = &template.root {
        collect_element(root, &mut nodes, &mut events);
    } else if let Some(fragment) = &template.root_fragment {
        collect_fragment(fragment, &mut nodes, &mut events);
    }

    ManifestComponent {
        component_id: component.id.to_string(),
        name: template.component_name.clone(),
        template: ManifestTemplate { nodes, events },
        actions: manifest_actions(component),
    }
}

#[derive(Debug, Clone)]
struct ActionBinding {
    method_id: String,
    action_batch_id: String,
}

fn action_bindings(
    model: &ApplicationSemanticModel,
    component: &ComponentNode,
) -> BTreeMap<String, ActionBinding> {
    component
        .action_endpoint_ids()
        .into_iter()
        .filter_map(|(name, endpoint)| {
            let batch = model
                .effect_trigger_plan
                .action_batches
                .values()
                .find(|batch| batch.authored_action_endpoint == endpoint)?;
            Some((
                name,
                ActionBinding {
                    method_id: endpoint.to_string(),
                    action_batch_id: batch.id.to_string(),
                },
            ))
        })
        .collect()
}

fn apply_action_bindings(
    manifest: &mut ManifestComponent,
    bindings: &BTreeMap<String, ActionBinding>,
) {
    for action in &mut manifest.actions {
        if let Some(binding) = bindings.get(&action.method) {
            action.method_id = Some(binding.method_id.clone());
            action.action_batch_id = Some(binding.action_batch_id.clone());
        }
    }
    for event in &mut manifest.template.events {
        let method = event
            .handler
            .strip_prefix("this.")
            .unwrap_or(&event.handler);
        if let Some(binding) = bindings.get(method) {
            event.kind = Some(ManifestEventKind::Action);
            event.method_id = Some(binding.method_id.clone());
            event.action_batch_id = Some(binding.action_batch_id.clone());
        }
    }
}

fn apply_action_storage_ids(manifest: &mut ManifestComponent, component: &ComponentNode) {
    for action in &mut manifest.actions {
        action.storage_id = component
            .state_fields
            .iter()
            .find(|state| state.name == action.field)
            .map(|state| IrStorageId::for_semantic_origin(&state.id).to_string());
    }
}

fn collect_fragment(
    fragment: &FragmentNode,
    nodes: &mut Vec<ManifestNode>,
    events: &mut Vec<ManifestEvent>,
) {
    for child in &fragment.children {
        collect_child(child, nodes, events);
    }
}

fn manifest_actions(component: &ComponentNode) -> Vec<ManifestAction> {
    component
        .actions
        .iter()
        .map(|action| ManifestAction {
            method: action.method.clone(),
            method_id: None,
            action_batch_id: None,
            operation: manifest_operation(&action.operation),
            field: action.field.clone(),
            storage_id: None,
            operand: match &action.operation {
                StateOperation::AssignParameter(parameter) => component
                    .methods
                    .iter()
                    .find(|method| method.name == action.method)
                    .and_then(|method| {
                        method
                            .parameters
                            .iter()
                            .position(|item| item.name == *parameter)
                    })
                    .map(|index| SerializableValue::Number(index.to_string()))
                    .or_else(|| {
                        parameter
                            .parse::<usize>()
                            .ok()
                            .map(|index| SerializableValue::Number(index.to_string()))
                    }),
                _ => manifest_operand(&action.operation),
            },
        })
        .collect()
}

fn manifest_operation(operation: &StateOperation) -> ManifestOperation {
    match operation {
        StateOperation::Increment => ManifestOperation::Increment,
        StateOperation::Decrement => ManifestOperation::Decrement,
        StateOperation::AddAssign(_) => ManifestOperation::AddAssign,
        StateOperation::SubtractAssign(_) => ManifestOperation::SubtractAssign,
        StateOperation::Assign(_) => ManifestOperation::Assign,
        StateOperation::AssignParameter(_) => ManifestOperation::AssignParameter,
        StateOperation::Toggle => ManifestOperation::Toggle,
    }
}

fn manifest_operand(operation: &StateOperation) -> Option<SerializableValue> {
    match operation {
        StateOperation::Increment | StateOperation::Decrement | StateOperation::Toggle => None,
        StateOperation::AddAssign(value)
        | StateOperation::SubtractAssign(value)
        | StateOperation::Assign(value) => Some(value.clone()),
        StateOperation::AssignParameter(_) => None,
    }
}

fn collect_element(
    element: &ElementNode,
    nodes: &mut Vec<ManifestNode>,
    events: &mut Vec<ManifestEvent>,
) {
    nodes.push(ManifestNode::Element {
        id: element.id.0.clone(),
        tag: element.tag_name.clone(),
    });

    for attribute in &element.attributes {
        match &attribute.value {
            AttributeValue::EventHandler {
                event,
                handler,
                arguments,
            } => {
                events.push(ManifestEvent {
                    node: element.id.0.clone(),
                    kind: None,
                    event: event.clone(),
                    handler: handler.clone(),
                    arguments: arguments.clone(),
                    method_id: None,
                    action_batch_id: None,
                });
            }
            AttributeValue::Binding {
                id,
                expression,
                initial_value,
                ..
            } => {
                nodes.push(ManifestNode::Binding {
                    id: id.0.clone(),
                    expression: expression.clone(),
                    initial_value: initial_value.clone(),
                    target: Some(ManifestBindingTarget::Attribute),
                    element: Some(element.id.0.clone()),
                    attribute: Some(attribute.name.clone()),
                });
            }
            _ => {}
        }
    }

    for child in &element.children {
        collect_child(child, nodes, events);
    }
}

fn collect_child(
    child: &TemplateChild,
    nodes: &mut Vec<ManifestNode>,
    events: &mut Vec<ManifestEvent>,
) {
    match child {
        TemplateChild::Text { .. } => {}
        TemplateChild::Binding {
            id,
            expression,
            initial_value,
            ..
        } => {
            nodes.push(ManifestNode::Binding {
                id: id.0.clone(),
                expression: expression.clone(),
                initial_value: initial_value.clone(),
                target: None,
                element: None,
                attribute: None,
            });
        }
        TemplateChild::Element(element) => collect_element(element, nodes, events),
        TemplateChild::Fragment(fragment) => collect_fragment(fragment, nodes, events),
        TemplateChild::Conditional(conditional) => collect_conditional(conditional, nodes),
        TemplateChild::List(list) => collect_list(list, nodes),
    }
}

fn collect_list(list: &ListNode, nodes: &mut Vec<ManifestNode>) {
    let Some(TemplateChild::Element(item_root)) = list.item_template.first() else {
        return;
    };

    nodes.push(ManifestNode::List {
        id: list.id.0.clone(),
        start: list.start_id.0.clone(),
        end: list.end_id.0.clone(),
        iterable: list.iterable.clone(),
        initial_value: list.initial_value.clone(),
        item_variable: list.item_variable.clone(),
        index_variable: list.index_variable.clone(),
        key_expression: list.key_expression.clone(),
        item_root: item_root.id.0.clone(),
        item_template_html: generate_list_item_template_html(list),
    });
}

fn collect_conditional(conditional: &ConditionalNode, nodes: &mut Vec<ManifestNode>) {
    nodes.push(ManifestNode::Conditional {
        id: conditional.id.0.clone(),
        start: conditional.start_id.0.clone(),
        end: conditional.end_id.0.clone(),
        condition: conditional.condition.clone(),
        initial_value: conditional.initial_value.clone(),
        when_true_html: generate_children_html(&conditional.when_true),
        when_false_html: generate_children_html(&conditional.when_false),
    });
}

#[cfg(test)]
mod tests {
    use super::{
        build_template_manifest_from_asm, ManifestEvent, ManifestEventKind,
        TEMPLATE_MANIFEST_SCHEMA_VERSION,
    };
    use crate::build_application_semantic_model;

    #[test]
    fn action_event_arguments_encode_exact_runtime_primitive_types() {
        let event = ManifestEvent {
            node: "n1".into(),
            kind: Some(ManifestEventKind::Action),
            event: "click".into(),
            handler: "this.record".into(),
            arguments: vec![
                crate::SerializableValue::String("checkout".into()),
                crate::SerializableValue::Number("2".into()),
                crate::SerializableValue::Boolean(true),
                crate::SerializableValue::Null,
            ],
            method_id: Some("action:record".into()),
            action_batch_id: Some("batch:record".into()),
        };
        let value = serde_json::to_value(event).expect("event JSON");
        assert_eq!(
            value["arguments"],
            serde_json::json!(["checkout", 2, true, null])
        );
    }

    #[test]
    fn emits_canonical_f8_action_batch_ids_on_every_action_binding() {
        let parsed = presolve_parser::parse_file(
            "src/TemplateActionBatch.tsx",
            r#"
@component("x-template-action-batch")
class TemplateActionBatch extends Component {
  count = state(0);

  @action()
  update() { this.count += 1; this.count += 1; }

  render() { return <button onClick={() => this.update()}>Update</button>; }
}
"#,
        );
        let model = build_application_semantic_model(&parsed);
        let component = &model.components[0];
        let method = component.id.method("update");
        let batch = model
            .effect_trigger_plan
            .action_batches
            .values()
            .find(|batch| batch.authored_action_endpoint == method)
            .expect("canonical F8 batch");
        let manifest = build_template_manifest_from_asm(&model);
        let component_manifest = &manifest.components[0];
        let event = &component_manifest.template.events[0];

        assert_eq!(manifest.schema_version, TEMPLATE_MANIFEST_SCHEMA_VERSION);
        assert_eq!(event.kind, Some(ManifestEventKind::Action));
        assert_eq!(event.method_id.as_deref(), Some(method.as_str()));
        assert_eq!(event.action_batch_id.as_deref(), Some(batch.id.as_str()));
        assert_eq!(component_manifest.actions.len(), 2);
        let storage_id =
            crate::IrStorageId::for_semantic_origin(&component.id.state_field("count")).to_string();
        assert!(component_manifest.actions.iter().all(|action| {
            action.method_id.as_deref() == Some(method.as_str())
                && action.action_batch_id.as_deref() == Some(batch.id.as_str())
                && action.storage_id.as_deref() == Some(storage_id.as_str())
        }));
    }

    #[test]
    fn v4_projects_only_exact_instance_qualified_ordinary_records() {
        let model = build_application_semantic_model(&presolve_parser::parse_file(
            "src/RepeatedManifest.tsx",
            r#"
@component("x-child") class Child {
  count = state(0);
  @action() increment() { this.count++; }
  render() { return <button title={this.count} onClick={() => this.increment()}>{this.count}</button>; }
}
@component("x-parent") class Parent { render() { return <><Child /><Child /></>; } }
"#,
        ));
        let manifest = build_template_manifest_from_asm(&model);
        assert_eq!(manifest.schema_version, TEMPLATE_MANIFEST_SCHEMA_VERSION);
        assert!(super::validate_template_manifest(&manifest).is_ok());
        assert_eq!(manifest.ordinary_events.len(), 2);
        assert!(manifest.ordinary_events.iter().all(|event| {
            event
                .instance_target_id
                .starts_with(&format!("{}/template-target:", event.component_instance_id))
        }));
        assert!(manifest.ordinary_bindings.iter().all(|binding| {
            binding.instance_binding_id.starts_with(&format!(
                "{}/template-binding:",
                binding.component_instance_id
            ))
        }));
    }

    #[test]
    fn emits_v3_instance_qualified_form_bridges() {
        let parsed = presolve_parser::parse_file(
            "src/FormManifest.tsx",
            r#"
@component("form-manifest")
class FormManifest {
  @form() profile!: Form;
  @field(this.profile) name = "";
  render() { return <input field={this.name} />; }
}
"#,
        );
        let model = build_application_semantic_model(&parsed);
        let manifest = build_template_manifest_from_asm(&model);
        assert_eq!(
            manifest.schema_version,
            super::TEMPLATE_MANIFEST_SCHEMA_VERSION
        );
        assert_eq!(manifest.form_bindings.len(), 1);
        let bridge = &manifest.form_bindings[0];
        assert!(bridge.field_binding_id.contains("field-binding"));
        assert!(bridge.form_instance_id.contains("form-instance"));
        assert!(bridge.input_program.contains("/input:"));
        assert!(bridge.blur_program.contains("/blur:"));
    }
}
