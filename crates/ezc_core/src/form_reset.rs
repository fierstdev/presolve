//! I11 immutable declaration-level Form reset programs.

use std::collections::BTreeMap;

use crate::{
    FieldBindingId, FieldId, FieldResetOperationId, FormEntity, FormFieldBinding, FormFieldEntity,
    FormId, FormTrackingProducts, ResetPlanId, SerializableValue, SourceProvenance,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FieldResetStep {
    RestoreInitialValue,
    WriteBoundControls,
    ClearDirty,
    ClearTouched,
    ClearValidation,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FieldResetOperation {
    pub id: FieldResetOperationId,
    pub field: FieldId,
    pub initial_value: SerializableValue,
    pub declaration_order: usize,
    pub bound_controls: Vec<FieldBindingId>,
    pub steps: Vec<FieldResetStep>,
    pub provenance: SourceProvenance,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FormResetPlan {
    pub id: ResetPlanId,
    pub form: FormId,
    pub operations: Vec<FieldResetOperation>,
    pub clear_aggregate_validity: bool,
    pub clear_submission_state: bool,
    pub schedule_validation: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ResetProducts {
    pub plans: BTreeMap<ResetPlanId, FormResetPlan>,
}

#[must_use]
/// # Panics
///
/// Panics when a valid I3 Field does not have both mandatory I8 tracking
/// records. That is a violated staged-product invariant.
pub fn collect_reset_products(
    forms: &BTreeMap<FormId, FormEntity>,
    fields: &BTreeMap<FieldId, FormFieldEntity>,
    bindings: &BTreeMap<FieldBindingId, FormFieldBinding>,
    tracking: &FormTrackingProducts,
) -> ResetProducts {
    let mut plans = BTreeMap::new();
    for form in forms.values() {
        let id = ResetPlanId::for_form(&form.id);
        let mut form_fields = fields
            .values()
            .filter(|field| field.owner_form == form.id)
            .collect::<Vec<_>>();
        form_fields.sort_by(|left, right| {
            (left.declaration_order, &left.id).cmp(&(right.declaration_order, &right.id))
        });
        let operations = form_fields
            .into_iter()
            .map(|field| {
                let mut bound_controls = bindings
                    .values()
                    .filter(|binding| binding.field == field.id)
                    .collect::<Vec<_>>();
                bound_controls.sort_by(|left, right| {
                    (left.authored_order, &left.id).cmp(&(right.authored_order, &right.id))
                });
                assert!(
                    tracking.dirty.tracking(&field.id).is_some()
                        && tracking.touched.tracking(&field.id).is_some(),
                    "I11 requires exact I8 tracking records"
                );
                FieldResetOperation {
                    id: FieldResetOperationId::for_plan_and_field(&id, &field.id),
                    field: field.id.clone(),
                    initial_value: field.initial_value.clone(),
                    declaration_order: field.declaration_order,
                    bound_controls: bound_controls
                        .into_iter()
                        .map(|binding| binding.id.clone())
                        .collect(),
                    steps: vec![
                        FieldResetStep::RestoreInitialValue,
                        FieldResetStep::WriteBoundControls,
                        FieldResetStep::ClearDirty,
                        FieldResetStep::ClearTouched,
                        FieldResetStep::ClearValidation,
                    ],
                    provenance: field.provenance.clone(),
                }
            })
            .collect();
        plans.insert(
            id.clone(),
            FormResetPlan {
                id,
                form: form.id.clone(),
                operations,
                clear_aggregate_validity: true,
                clear_submission_state: true,
                schedule_validation: false,
            },
        );
    }
    ResetProducts { plans }
}

#[cfg(test)]
mod tests {
    #[test]
    fn creates_ordered_non_revalidating_reset_operations() {
        let parsed = ezc_parser::parse_file(
            "src/Form.tsx",
            r#"@component("x") class X { @form() form!: Form; @field(this.form) first = "a"; @field(this.form) second = false; render() { return <input field={this.first} />; } }"#,
        );
        let model = crate::build_application_semantic_model(&parsed);
        let form = crate::FormId::for_owner(&model.components[0].id, "form");
        let plan = &model.reset.plans[&crate::ResetPlanId::for_form(&form)];
        assert!(!plan.schedule_validation);
        assert_eq!(plan.operations.len(), 2);
        assert_eq!(plan.operations[0].bound_controls.len(), 1);
        assert!(plan.operations[1].bound_controls.is_empty());
    }
}
