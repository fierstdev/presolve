//! I12 instance-qualified projection of canonical Form plans.
use crate::{
    ComponentInstanceId, ComponentInstancePlan, FieldId, FormEntity, FormFieldDirtySlotId,
    FormFieldEntity, FormFieldTouchedSlotId, FormFieldValidationSlotId, FormFieldValueSlotId,
    FormId, FormInstanceId, FormSubmissionStateSlotId, FormValidationAggregateSlotId,
};
use std::collections::BTreeMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FormIrOperation {
    InitializeFormInstance,
    InitializeFieldValue,
    InitializeFieldDirty,
    InitializeFieldTouched,
    InitializeFieldValidation,
    ReadControlChannel,
    NormalizeControlValue,
    WriteFieldValue,
    WriteControlChannel,
    ComputeDirty,
    MarkTouched,
    EvaluateValidationRule,
    StoreValidationResult,
    ComputeFormValidity,
    SerializeForm,
    BeginSubmission,
    InvokeSubmissionAction,
    CompleteSubmission,
    FailSubmission,
    ResetField,
    ResetForm,
    ClearValidation,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FormRuntimeStorage {
    pub value: BTreeMap<FieldId, FormFieldValueSlotId>,
    pub dirty: BTreeMap<FieldId, FormFieldDirtySlotId>,
    pub touched: BTreeMap<FieldId, FormFieldTouchedSlotId>,
    pub validation: BTreeMap<FieldId, FormFieldValidationSlotId>,
    pub aggregate: FormValidationAggregateSlotId,
    pub submission: FormSubmissionStateSlotId,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FormInstanceIr {
    pub id: FormInstanceId,
    pub form: FormId,
    pub component_instance: ComponentInstanceId,
    pub storage: FormRuntimeStorage,
    pub initialize: Vec<FormIrOperation>,
    pub input: BTreeMap<FieldId, Vec<FormIrOperation>>,
    pub blur: BTreeMap<FieldId, Vec<FormIrOperation>>,
    pub reset: Vec<FormIrOperation>,
}
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct FormIrReport {
    pub instances: BTreeMap<FormInstanceId, FormInstanceIr>,
}

#[must_use]
#[allow(clippy::too_many_lines)]
pub fn lower_form_ir(
    instances: &ComponentInstancePlan,
    forms: &BTreeMap<FormId, FormEntity>,
    fields: &BTreeMap<FieldId, FormFieldEntity>,
) -> FormIrReport {
    let mut result = BTreeMap::new();
    for component in instances.instances.values() {
        for form in forms
            .values()
            .filter(|form| form.owner.entity_id() == Some(&component.component))
        {
            let id = FormInstanceId::for_component_instance(&component.id, &form.id);
            let mut form_fields = fields
                .values()
                .filter(|field| field.owner_form == form.id)
                .collect::<Vec<_>>();
            form_fields
                .sort_by(|a, b| (a.declaration_order, &a.id).cmp(&(b.declaration_order, &b.id)));
            let storage = FormRuntimeStorage {
                value: form_fields
                    .iter()
                    .map(|f| {
                        (
                            f.id.clone(),
                            FormFieldValueSlotId::for_instance(&id, f.id.as_str()),
                        )
                    })
                    .collect(),
                dirty: form_fields
                    .iter()
                    .map(|f| {
                        (
                            f.id.clone(),
                            FormFieldDirtySlotId::for_instance(&id, f.id.as_str()),
                        )
                    })
                    .collect(),
                touched: form_fields
                    .iter()
                    .map(|f| {
                        (
                            f.id.clone(),
                            FormFieldTouchedSlotId::for_instance(&id, f.id.as_str()),
                        )
                    })
                    .collect(),
                validation: form_fields
                    .iter()
                    .map(|f| {
                        (
                            f.id.clone(),
                            FormFieldValidationSlotId::for_instance(&id, f.id.as_str()),
                        )
                    })
                    .collect(),
                aggregate: FormValidationAggregateSlotId::for_instance(&id, "aggregate"),
                submission: FormSubmissionStateSlotId::for_instance(&id, "submission"),
            };
            let input = form_fields
                .iter()
                .map(|f| {
                    (
                        f.id.clone(),
                        vec![
                            FormIrOperation::ReadControlChannel,
                            FormIrOperation::NormalizeControlValue,
                            FormIrOperation::WriteFieldValue,
                            FormIrOperation::ComputeDirty,
                            FormIrOperation::EvaluateValidationRule,
                            FormIrOperation::StoreValidationResult,
                            FormIrOperation::ComputeFormValidity,
                            FormIrOperation::WriteControlChannel,
                        ],
                    )
                })
                .collect();
            let blur = form_fields
                .iter()
                .map(|f| {
                    (
                        f.id.clone(),
                        vec![
                            FormIrOperation::MarkTouched,
                            FormIrOperation::EvaluateValidationRule,
                            FormIrOperation::ComputeFormValidity,
                        ],
                    )
                })
                .collect();
            result.insert(
                id.clone(),
                FormInstanceIr {
                    id,
                    form: form.id.clone(),
                    component_instance: component.id.clone(),
                    storage,
                    initialize: vec![FormIrOperation::InitializeFormInstance],
                    input,
                    blur,
                    reset: vec![
                        FormIrOperation::ResetField,
                        FormIrOperation::ClearValidation,
                        FormIrOperation::ResetForm,
                    ],
                },
            );
        }
    }
    FormIrReport { instances: result }
}

#[cfg(test)]
mod tests {
    #[test]
    fn repeats_form_storage_per_component_instance() {
        let parsed = ezc_parser::parse_file(
            "src/X.tsx",
            r#"@component("x") class X { @form() form!: Form; @field(this.form) value=""; render(){return <input field={this.value}/>;} }"#,
        );
        let model = crate::build_application_semantic_model(&parsed);
        assert_eq!(
            model.form_ir.instances.len(),
            model.component_instance_plan.instances.len()
        );
        assert!(model
            .form_ir
            .instances
            .values()
            .all(|instance| !instance.storage.value.is_empty()));
    }
}
