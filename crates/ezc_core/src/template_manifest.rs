use std::collections::BTreeMap;

use serde::Serialize;

use crate::component_graph::{ComponentGraph, ComponentNode, SerializableValue, StateOperation};
use crate::html_codegen::{generate_children_html, generate_list_item_template_html};
use crate::template_graph::{
    AttributeValue, ConditionalNode, ElementNode, FragmentNode, ListNode, TemplateChild,
    TemplateGraph, TemplateNode,
};
use crate::ApplicationSemanticModel;

pub const TEMPLATE_MANIFEST_SCHEMA_VERSION: u32 = 2;
const LEGACY_TEMPLATE_MANIFEST_SCHEMA_VERSION: u32 = 1;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct TemplateManifest {
    pub schema_version: u32,
    pub components: Vec<ManifestComponent>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ManifestComponent {
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
    }
}

/// Build schema-v2 template metadata from canonical ASM action-batch facts.
#[must_use]
pub fn build_template_manifest_from_asm(model: &ApplicationSemanticModel) -> TemplateManifest {
    TemplateManifest {
        // A source unit without F8 action batches remains a legacy manifest: it
        // cannot activate completed-action effects and therefore must not claim
        // to provide the v2 action-batch bridge.
        schema_version: if model.effect_trigger_plan.action_batches.is_empty() {
            LEGACY_TEMPLATE_MANIFEST_SCHEMA_VERSION
        } else {
            TEMPLATE_MANIFEST_SCHEMA_VERSION
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
                Some(manifest)
            })
            .collect(),
    }
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
        .methods
        .iter()
        .filter_map(|method| {
            let batch = model
                .effect_trigger_plan
                .action_batches
                .values()
                .find(|batch| batch.authored_action_method == method.id)?;
            Some((
                method.name.clone(),
                ActionBinding {
                    method_id: method.id.to_string(),
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
            operand: manifest_operand(&action.operation),
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
        StateOperation::Toggle => ManifestOperation::Toggle,
    }
}

fn manifest_operand(operation: &StateOperation) -> Option<SerializableValue> {
    match operation {
        StateOperation::Increment | StateOperation::Decrement | StateOperation::Toggle => None,
        StateOperation::AddAssign(value)
        | StateOperation::SubtractAssign(value)
        | StateOperation::Assign(value) => Some(value.clone()),
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
            AttributeValue::EventHandler { event, handler } => {
                events.push(ManifestEvent {
                    node: element.id.0.clone(),
                    kind: None,
                    event: event.clone(),
                    handler: handler.clone(),
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
        build_template_manifest_from_asm, ManifestEventKind, TEMPLATE_MANIFEST_SCHEMA_VERSION,
    };
    use crate::build_application_semantic_model;

    #[test]
    fn emits_canonical_f8_action_batch_ids_on_every_action_binding() {
        let parsed = ezc_parser::parse_file(
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
            .find(|batch| batch.authored_action_method == method)
            .expect("canonical F8 batch");
        let manifest = build_template_manifest_from_asm(&model);
        let component_manifest = &manifest.components[0];
        let event = &component_manifest.template.events[0];

        assert_eq!(manifest.schema_version, TEMPLATE_MANIFEST_SCHEMA_VERSION);
        assert_eq!(event.kind, Some(ManifestEventKind::Action));
        assert_eq!(event.method_id.as_deref(), Some(method.as_str()));
        assert_eq!(event.action_batch_id.as_deref(), Some(batch.id.as_str()));
        assert_eq!(component_manifest.actions.len(), 2);
        assert!(component_manifest.actions.iter().all(|action| {
            action.method_id.as_deref() == Some(method.as_str())
                && action.action_batch_id.as_deref() == Some(batch.id.as_str())
        }));
    }
}
