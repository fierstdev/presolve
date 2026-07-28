//! I17 canonical Forms inspection projection.
//!
//! This module consumes staged Forms products only; it never reinterprets
//! syntax or inspects live runtime state.

use std::collections::BTreeMap;

use serde::Serialize;

use crate::{
    semantic_type_text, ApplicationSemanticModel, FormControlChannel, SemanticId, SerializableValue,
};

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct FormInspectionRegistry {
    pub records: BTreeMap<SemanticId, FormInspection>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct FormInspection {
    pub role: &'static str,
    pub form: String,
    pub owner_component: String,
    pub field_order: Vec<String>,
    pub field_semantic_type: Option<String>,
    pub initial_value: Option<SerializableValue>,
    pub bindings: Vec<String>,
    pub binding_channels: Vec<String>,
    pub validation_rules: Vec<String>,
    pub source_rules: Vec<String>,
    pub dependent_rules: Vec<String>,
    pub dirty_tracking: Option<String>,
    pub touched_tracking: Option<String>,
    pub submission_plan: Option<String>,
    pub serialization_plan: Option<String>,
    pub reset_plan: Option<String>,
    pub instances: Vec<FormInspectionInstance>,
    pub runtime_registry_member: bool,
    pub runtime_artifact_member: bool,
    pub resume_planned: bool,
    pub blocked_reasons: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct FormInspectionInstance {
    pub form_instance: String,
    pub component_instance: String,
    pub value_slots: Vec<String>,
    pub dirty_slots: Vec<String>,
    pub touched_slots: Vec<String>,
    pub validation_slots: Vec<String>,
    pub aggregate_slot: String,
    pub submission_slot: String,
    pub input_program_fields: Vec<String>,
    pub blur_program_fields: Vec<String>,
}

/// Build one shared I17 projection for full/selected ASM, graph, and explain.
///
/// # Panics
///
/// Panics if a retained canonical Form has no Component owner, or if a
/// retained binding or validation rule has no corresponding Form projection.
#[must_use]
#[allow(clippy::too_many_lines)]
pub fn build_form_inspection_registry(model: &ApplicationSemanticModel) -> FormInspectionRegistry {
    let artifact = crate::build_runtime_forms_artifact(model);
    let resume = crate::build_resume_plan(model);
    let mut records = BTreeMap::new();
    for form in model.forms.values() {
        let fields = model
            .form_fields
            .values()
            .filter(|field| field.owner_form == form.id)
            .collect::<Vec<_>>();
        let instances = model
            .optimized_form_ir
            .optimized
            .instances
            .values()
            .filter(|instance| instance.form == form.id)
            .map(|instance| FormInspectionInstance {
                form_instance: instance.id.to_string(),
                component_instance: instance.component_instance.to_string(),
                value_slots: instance
                    .storage
                    .value
                    .values()
                    .map(|id| id.as_str().to_string())
                    .collect(),
                dirty_slots: instance
                    .storage
                    .dirty
                    .values()
                    .map(|id| id.as_str().to_string())
                    .collect(),
                touched_slots: instance
                    .storage
                    .touched
                    .values()
                    .map(|id| id.as_str().to_string())
                    .collect(),
                validation_slots: instance
                    .storage
                    .validation
                    .values()
                    .map(|id| id.as_str().to_string())
                    .collect(),
                aggregate_slot: instance.storage.aggregate.as_str().to_string(),
                submission_slot: instance.storage.submission.as_str().to_string(),
                input_program_fields: instance.input.keys().map(ToString::to_string).collect(),
                blur_program_fields: instance.blur.keys().map(ToString::to_string).collect(),
            })
            .collect::<Vec<_>>();
        let base = FormInspection {
            role: "form",
            form: form.id.to_string(),
            owner_component: form.owner.entity_id().expect("Form owner").to_string(),
            field_order: fields.iter().map(|field| field.id.to_string()).collect(),
            field_semantic_type: None,
            initial_value: None,
            bindings: model
                .form_field_bindings
                .values()
                .filter(|binding| binding.form == form.id)
                .map(|binding| binding.id.to_string())
                .collect(),
            binding_channels: model
                .form_field_bindings
                .values()
                .filter(|binding| binding.form == form.id)
                .map(|binding| form_control_channel_text(binding.channel).to_string())
                .collect(),
            validation_rules: model
                .validation_rules
                .values()
                .filter(|rule| rule.owner_form == form.id)
                .map(|rule| rule.id.to_string())
                .collect(),
            source_rules: Vec::new(),
            dependent_rules: Vec::new(),
            dirty_tracking: model
                .form_tracking
                .dirty
                .plan(&form.id)
                .map(|plan| plan.id.as_str().to_string()),
            touched_tracking: model
                .form_tracking
                .touched
                .plan(&form.id)
                .map(|plan| plan.id.as_str().to_string()),
            submission_plan: model
                .submissions
                .plan(&form.id)
                .map(|plan| plan.id.as_str().to_string()),
            serialization_plan: model
                .serialization
                .plans
                .get(&crate::SerializationPlanId::for_form(&form.id))
                .map(|plan| plan.id.as_str().to_string()),
            reset_plan: model
                .reset
                .plans
                .get(&crate::ResetPlanId::for_form(&form.id))
                .map(|plan| plan.id.as_str().to_string()),
            instances,
            runtime_registry_member: model.runtime_forms.forms.contains_key(&form.id),
            runtime_artifact_member: artifact
                .forms
                .iter()
                .any(|record| record.id == form.id.to_string()),
            resume_planned: resume
                .form_instances
                .iter()
                .any(|record| record.form == form.id.to_string()),
            blocked_reasons: Vec::new(),
        };
        records.insert(form.id.as_semantic_id().clone(), base.clone());
        for field in fields {
            let mut record = base.clone();
            record.role = "field";
            record.field_order = vec![field.id.to_string()];
            record.field_semantic_type = Some(semantic_type_text(&field.semantic_type));
            record.initial_value = Some(field.initial_value.clone());
            record.bindings = model
                .form_field_bindings
                .values()
                .filter(|binding| binding.field == field.id)
                .map(|binding| binding.id.to_string())
                .collect();
            record.binding_channels = model
                .form_field_bindings
                .values()
                .filter(|binding| binding.field == field.id)
                .map(|binding| form_control_channel_text(binding.channel).to_string())
                .collect();
            record.validation_rules = model
                .validation_rules
                .values()
                .filter(|rule| rule.target_field == field.id)
                .map(|rule| rule.id.to_string())
                .collect();
            record.source_rules = model
                .validation_dependency_plans
                .dependencies
                .values()
                .filter(|dependency| dependency.source_field == field.id)
                .map(|dependency| dependency.dependent_rule.to_string())
                .collect();
            record.dependent_rules = model
                .validation_dependency_plans
                .dependencies
                .values()
                .filter(|dependency| dependency.target_field == field.id)
                .map(|dependency| dependency.dependent_rule.to_string())
                .collect();
            record.dirty_tracking = model
                .form_tracking
                .dirty
                .tracking(&field.id)
                .map(|tracking| tracking.id.as_str().to_string());
            record.touched_tracking = model
                .form_tracking
                .touched
                .tracking(&field.id)
                .map(|tracking| tracking.id.as_str().to_string());
            records.insert(field.id.as_semantic_id().clone(), record);
        }
    }
    for binding in model.form_field_bindings.values() {
        let mut record = records
            .get(binding.form.as_semantic_id())
            .expect("binding Form inspection")
            .clone();
        record.role = "binding";
        record.bindings = vec![binding.id.to_string()];
        record.binding_channels = vec![form_control_channel_text(binding.channel).to_string()];
        records.insert(binding.id.as_semantic_id().clone(), record);
    }
    for rule in model.validation_rules.values() {
        let mut record = records
            .get(rule.owner_form.as_semantic_id())
            .expect("rule Form inspection")
            .clone();
        record.role = "validation-rule";
        record.validation_rules = vec![rule.id.to_string()];
        records.insert(rule.id.as_semantic_id().clone(), record);
    }
    FormInspectionRegistry { records }
}

const fn form_control_channel_text(channel: FormControlChannel) -> &'static str {
    match channel {
        FormControlChannel::Value => "value",
        FormControlChannel::NumericValue => "numeric-value",
        FormControlChannel::Checked => "checked",
        FormControlChannel::RadioValue => "radio-value",
        FormControlChannel::SelectedValue => "selected-value",
        FormControlChannel::SelectedValues => "selected-values",
        FormControlChannel::Files => "files",
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn projects_canonical_form_field_binding_and_rule_records() {
        let model = crate::build_application_semantic_model(&presolve_parser::parse_file(
            "src/X.tsx",
            r#"@component("x") class X { @form() @serialize("json") form!: Form; @field(this.form) value = ""; @action() @submit(this.form) save(): void {} render() { return <form form={this.form}><input field={this.value}/></form>; } }"#,
        ));
        let registry = super::build_form_inspection_registry(&model);
        assert_eq!(registry.records.len(), 3);
        assert!(registry
            .records
            .values()
            .any(|record| record.role == "form" && record.runtime_artifact_member));
    }
}
