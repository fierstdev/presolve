//! I8 declaration-level dirty and touched tracking plans.
//!
//! These products describe compiler-owned tracking semantics only. They do not
//! allocate runtime state, execute transitions, schedule validation, or create
//! Form instances.

use std::collections::BTreeMap;

use crate::{
    DirtyTrackingPlanId, ExecutionBoundary, FieldBindingId, FieldId, FieldTrackingId, FormEntity,
    FormFieldBinding, FormFieldEntity, FormId, FormOwnershipGraph, SemanticId, SerializableValue,
    SourceProvenance, TouchedTrackingPlanId,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DirtyTransitionCause {
    CommittedFieldValueWrite,
    ResetField,
    ResetForm,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TouchedTransitionCause {
    BoundControlBlur,
    ResetField,
    ResetForm,
}

/// Immutable dirty transition facts for one canonical Field.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DirtyTransitionPlan {
    pub committed_write: DirtyTransitionCause,
    pub reset_field: DirtyTransitionCause,
    pub reset_form: DirtyTransitionCause,
    pub compares_with_initial_value: bool,
    pub may_handoff_to_direct_validation_dependencies: bool,
}

/// Immutable touched transition facts for one canonical Field.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TouchedTransitionPlan {
    pub blur: Option<TouchedTransitionCause>,
    pub reset_field: TouchedTransitionCause,
    pub reset_form: TouchedTransitionCause,
    pub may_handoff_blur_to_target_validation: bool,
}

/// I8 dirty facts for one valid I3 Form Field.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FieldDirtyTracking {
    pub id: FieldTrackingId,
    pub plan: DirtyTrackingPlanId,
    pub field: FieldId,
    pub form: FormId,
    pub component: SemanticId,
    pub initial_value: SerializableValue,
    pub initial_dirty: bool,
    pub declaration_order: usize,
    pub transitions: DirtyTransitionPlan,
    pub provenance: SourceProvenance,
    pub initializer_provenance: SourceProvenance,
    pub boundary: ExecutionBoundary,
}

/// I8 touched facts for one valid I3 Form Field. All radio bindings for a
/// Field intentionally share this one record and touched flag.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FieldTouchedTracking {
    pub id: FieldTrackingId,
    pub plan: TouchedTrackingPlanId,
    pub field: FieldId,
    pub form: FormId,
    pub component: SemanticId,
    pub initial_touched: bool,
    pub declaration_order: usize,
    pub blur_bindings: Vec<FieldBindingId>,
    pub transitions: TouchedTransitionPlan,
    pub provenance: SourceProvenance,
    pub boundary: ExecutionBoundary,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DirtyTrackingPlan {
    pub id: DirtyTrackingPlanId,
    pub form: FormId,
    pub component: SemanticId,
    pub fields: Vec<FieldTrackingId>,
    pub validation: FormTrackingValidation,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TouchedTrackingPlan {
    pub id: TouchedTrackingPlanId,
    pub form: FormId,
    pub component: SemanticId,
    pub fields: Vec<FieldTrackingId>,
    pub validation: FormTrackingValidation,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FormTrackingIntegrityDiagnostic {
    pub code: String,
    pub kind: FormTrackingIntegrityKind,
    pub message: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum FormTrackingIntegrityKind {
    MissingForm,
    MissingField,
    PlanIdentityMismatch,
    TrackingIdentityMismatch,
    FieldFormMismatch,
    ComponentMismatch,
    InitialValueMismatch,
    BindingMismatch,
    DuplicateTrackingRecord,
    MissingTrackingRecord,
    MissingProvenance,
    NonCanonicalOrdering,
}

impl FormTrackingIntegrityKind {
    #[must_use]
    pub const fn code(self) -> &'static str {
        match self {
            Self::MissingForm => "PSASM1273",
            Self::MissingField => "PSASM1274",
            Self::PlanIdentityMismatch => "PSASM1275",
            Self::TrackingIdentityMismatch => "PSASM1276",
            Self::FieldFormMismatch => "PSASM1277",
            Self::ComponentMismatch => "PSASM1278",
            Self::InitialValueMismatch => "PSASM1279",
            Self::BindingMismatch => "PSASM1280",
            Self::DuplicateTrackingRecord => "PSASM1281",
            Self::MissingTrackingRecord => "PSASM1282",
            Self::MissingProvenance => "PSASM1283",
            Self::NonCanonicalOrdering => "PSASM1284",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FormTrackingValidation {
    pub diagnostics: Vec<FormTrackingIntegrityDiagnostic>,
    pub is_valid: bool,
}

impl Default for FormTrackingValidation {
    fn default() -> Self {
        Self {
            diagnostics: Vec::new(),
            is_valid: true,
        }
    }
}

/// Complete immutable I8 product. This is declaration-level only and is not
/// an additional ASM ownership authority or public schema.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DirtyTrackingGraph {
    pub plans: BTreeMap<DirtyTrackingPlanId, DirtyTrackingPlan>,
    pub fields: BTreeMap<FieldTrackingId, FieldDirtyTracking>,
    pub validation: FormTrackingValidation,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TouchedTrackingGraph {
    pub plans: BTreeMap<TouchedTrackingPlanId, TouchedTrackingPlan>,
    pub fields: BTreeMap<FieldTrackingId, FieldTouchedTracking>,
    pub validation: FormTrackingValidation,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FormTrackingProducts {
    pub dirty: DirtyTrackingGraph,
    pub touched: TouchedTrackingGraph,
}

impl DirtyTrackingGraph {
    #[must_use]
    pub fn plan(&self, form: &FormId) -> Option<&DirtyTrackingPlan> {
        self.plans.get(&DirtyTrackingPlanId::for_form(form))
    }

    #[must_use]
    pub fn tracking(&self, field: &FieldId) -> Option<&FieldDirtyTracking> {
        self.fields.get(&FieldTrackingId::for_field(field))
    }

    /// Pure I8 transition query. It computes the recorded structural-equality
    /// rule; it does not mutate runtime state or schedule validation.
    #[must_use]
    pub fn dirty_after_committed_write(
        &self,
        field: &FieldId,
        current: &SerializableValue,
    ) -> Option<bool> {
        self.tracking(field).map(|tracking| {
            !structurally_equal_serializable_values(current, &tracking.initial_value)
        })
    }
}

impl TouchedTrackingGraph {
    #[must_use]
    pub fn plan(&self, form: &FormId) -> Option<&TouchedTrackingPlan> {
        self.plans.get(&TouchedTrackingPlanId::for_form(form))
    }

    #[must_use]
    pub fn tracking(&self, field: &FieldId) -> Option<&FieldTouchedTracking> {
        self.fields.get(&FieldTrackingId::for_field(field))
    }

    #[must_use]
    pub fn blur_bindings(&self, field: &FieldId) -> Vec<&FieldBindingId> {
        self.tracking(field)
            .map_or_else(Vec::new, |tracking| tracking.blur_bindings.iter().collect())
    }
}

/// Canonical recursive equality is `SerializableValue` structural equality.
#[must_use]
pub fn structurally_equal_serializable_values(
    left: &SerializableValue,
    right: &SerializableValue,
) -> bool {
    left == right
}

#[must_use]
#[allow(clippy::too_many_lines)]
/// # Panics
///
/// Panics when a supposedly valid I2 Form lacks its required canonical
/// Component owner. That is a violated pre-I8 invariant.
pub fn collect_form_tracking_products(
    forms: &BTreeMap<FormId, FormEntity>,
    fields: &BTreeMap<FieldId, FormFieldEntity>,
    bindings: &BTreeMap<FieldBindingId, FormFieldBinding>,
    ownership: &FormOwnershipGraph,
) -> FormTrackingProducts {
    let mut dirty = DirtyTrackingGraph {
        plans: BTreeMap::new(),
        fields: BTreeMap::new(),
        validation: FormTrackingValidation::default(),
    };
    let mut touched = TouchedTrackingGraph {
        plans: BTreeMap::new(),
        fields: BTreeMap::new(),
        validation: FormTrackingValidation::default(),
    };

    for form in forms.values() {
        let dirty_id = DirtyTrackingPlanId::for_form(&form.id);
        let touched_id = TouchedTrackingPlanId::for_form(&form.id);
        let fields_for_form = ordered_fields(fields, &form.id);
        let mut tracking_ids = Vec::with_capacity(fields_for_form.len());

        for field in fields_for_form {
            let tracking_id = FieldTrackingId::for_field(&field.id);
            tracking_ids.push(tracking_id.clone());
            let blur_bindings = ordered_bindings_for_field(bindings, &field.id)
                .into_iter()
                .filter(|binding| {
                    binding.form == form.id
                        && binding.component == field.owner_component
                        && binding.field == field.id
                })
                .map(|binding| binding.id.clone())
                .collect::<Vec<_>>();
            dirty.fields.insert(
                tracking_id.clone(),
                FieldDirtyTracking {
                    id: tracking_id.clone(),
                    plan: dirty_id.clone(),
                    field: field.id.clone(),
                    form: form.id.clone(),
                    component: field.owner_component.clone(),
                    initial_value: field.initial_value.clone(),
                    initial_dirty: false,
                    declaration_order: field.declaration_order,
                    transitions: DirtyTransitionPlan {
                        committed_write: DirtyTransitionCause::CommittedFieldValueWrite,
                        reset_field: DirtyTransitionCause::ResetField,
                        reset_form: DirtyTransitionCause::ResetForm,
                        compares_with_initial_value: true,
                        may_handoff_to_direct_validation_dependencies: true,
                    },
                    provenance: field.provenance.clone(),
                    initializer_provenance: field.initializer_provenance.clone(),
                    boundary: ExecutionBoundary::Client,
                },
            );
            touched.fields.insert(
                tracking_id.clone(),
                FieldTouchedTracking {
                    id: tracking_id,
                    plan: touched_id.clone(),
                    field: field.id.clone(),
                    form: form.id.clone(),
                    component: field.owner_component.clone(),
                    initial_touched: false,
                    declaration_order: field.declaration_order,
                    blur_bindings: blur_bindings.clone(),
                    transitions: TouchedTransitionPlan {
                        blur: (!blur_bindings.is_empty())
                            .then_some(TouchedTransitionCause::BoundControlBlur),
                        reset_field: TouchedTransitionCause::ResetField,
                        reset_form: TouchedTransitionCause::ResetForm,
                        may_handoff_blur_to_target_validation: !blur_bindings.is_empty(),
                    },
                    provenance: field.provenance.clone(),
                    boundary: ExecutionBoundary::Client,
                },
            );
        }

        dirty.plans.insert(
            dirty_id.clone(),
            DirtyTrackingPlan {
                id: dirty_id,
                form: form.id.clone(),
                component: form
                    .owner
                    .entity_id()
                    .expect("valid Form has component owner")
                    .clone(),
                fields: tracking_ids.clone(),
                validation: FormTrackingValidation::default(),
            },
        );
        touched.plans.insert(
            touched_id.clone(),
            TouchedTrackingPlan {
                id: touched_id,
                form: form.id.clone(),
                component: form
                    .owner
                    .entity_id()
                    .expect("valid Form has component owner")
                    .clone(),
                fields: tracking_ids,
                validation: FormTrackingValidation::default(),
            },
        );
    }

    dirty.validation = validate_dirty_tracking_graph(&dirty, forms, fields, bindings, ownership);
    touched.validation =
        validate_touched_tracking_graph(&touched, forms, fields, bindings, ownership);
    for plan in dirty.plans.values_mut() {
        plan.validation = dirty.validation.clone();
    }
    for plan in touched.plans.values_mut() {
        plan.validation = touched.validation.clone();
    }
    FormTrackingProducts { dirty, touched }
}

#[must_use]
#[allow(clippy::too_many_lines)]
/// # Panics
///
/// Panics when a supposedly valid I2 Form lacks its required canonical
/// Component owner. That is a violated pre-I8 invariant.
pub fn validate_dirty_tracking_graph(
    graph: &DirtyTrackingGraph,
    forms: &BTreeMap<FormId, FormEntity>,
    fields: &BTreeMap<FieldId, FormFieldEntity>,
    _bindings: &BTreeMap<FieldBindingId, FormFieldBinding>,
    ownership: &FormOwnershipGraph,
) -> FormTrackingValidation {
    let mut validation = FormTrackingValidation::default();
    for form in forms.values() {
        let plan_id = DirtyTrackingPlanId::for_form(&form.id);
        let Some(plan) = graph.plans.get(&plan_id) else {
            push(
                &mut validation,
                FormTrackingIntegrityKind::MissingForm,
                format!("Form `{}` has no dirty tracking plan", form.id.as_str()),
            );
            continue;
        };
        if plan.id != plan_id
            || plan.form != form.id
            || plan.component
                != *form
                    .owner
                    .entity_id()
                    .expect("valid Form has component owner")
        {
            push(
                &mut validation,
                FormTrackingIntegrityKind::PlanIdentityMismatch,
                format!(
                    "dirty tracking plan for `{}` has non-canonical identity or owner",
                    form.id.as_str()
                ),
            );
        }
        if ownership.component_of_form(&form.id) != form.owner.entity_id() {
            push(
                &mut validation,
                FormTrackingIntegrityKind::FieldFormMismatch,
                format!(
                    "dirty tracking Form `{}` is absent from canonical I5 ownership",
                    form.id.as_str()
                ),
            );
        }
        let expected = ordered_fields(fields, &form.id)
            .into_iter()
            .map(|field| FieldTrackingId::for_field(&field.id))
            .collect::<Vec<_>>();
        if plan.fields != expected {
            push(
                &mut validation,
                FormTrackingIntegrityKind::NonCanonicalOrdering,
                format!(
                    "dirty tracking plan `{}` does not retain I3 field order",
                    plan.id.as_str()
                ),
            );
        }
    }
    for field in fields.values() {
        let id = FieldTrackingId::for_field(&field.id);
        let Some(record) = graph.fields.get(&id) else {
            push(
                &mut validation,
                FormTrackingIntegrityKind::MissingTrackingRecord,
                format!("Field `{}` has no dirty tracking record", field.id.as_str()),
            );
            continue;
        };
        if record.id != id || record.plan != DirtyTrackingPlanId::for_form(&field.owner_form) {
            push(
                &mut validation,
                FormTrackingIntegrityKind::TrackingIdentityMismatch,
                format!(
                    "dirty tracking for `{}` has non-canonical identity",
                    field.id.as_str()
                ),
            );
        }
        if record.field != field.id || record.form != field.owner_form {
            push(
                &mut validation,
                FormTrackingIntegrityKind::FieldFormMismatch,
                format!(
                    "dirty tracking `{}` disagrees with I3 Field ownership",
                    record.id.as_str()
                ),
            );
        }
        if record.component != field.owner_component {
            push(
                &mut validation,
                FormTrackingIntegrityKind::ComponentMismatch,
                format!(
                    "dirty tracking `{}` disagrees with I3 Field Component",
                    record.id.as_str()
                ),
            );
        }
        if ownership.component_of_field(&field.id) != Some(&field.owner_component) {
            push(
                &mut validation,
                FormTrackingIntegrityKind::FieldFormMismatch,
                format!(
                    "dirty tracking `{}` is absent from canonical I5 Field ownership",
                    record.id.as_str()
                ),
            );
        }
        if record.initial_value != field.initial_value || record.initial_dirty {
            push(&mut validation, FormTrackingIntegrityKind::InitialValueMismatch, format!("dirty tracking `{}` does not retain the I3 initial value or clean initial state", record.id.as_str()));
        }
        if record.provenance != field.provenance
            || record.initializer_provenance != field.initializer_provenance
        {
            push(
                &mut validation,
                FormTrackingIntegrityKind::MissingProvenance,
                format!(
                    "dirty tracking `{}` lacks canonical field provenance",
                    record.id.as_str()
                ),
            );
        }
    }
    for record in graph.fields.values() {
        if !fields.contains_key(&record.field) {
            push(
                &mut validation,
                FormTrackingIntegrityKind::MissingField,
                format!(
                    "dirty tracking `{}` references unknown Field",
                    record.id.as_str()
                ),
            );
        }
    }
    validation
}

#[must_use]
#[allow(clippy::too_many_lines)]
/// # Panics
///
/// Panics when a supposedly valid I2 Form lacks its required canonical
/// Component owner. That is a violated pre-I8 invariant.
pub fn validate_touched_tracking_graph(
    graph: &TouchedTrackingGraph,
    forms: &BTreeMap<FormId, FormEntity>,
    fields: &BTreeMap<FieldId, FormFieldEntity>,
    bindings: &BTreeMap<FieldBindingId, FormFieldBinding>,
    ownership: &FormOwnershipGraph,
) -> FormTrackingValidation {
    let mut validation = FormTrackingValidation::default();
    for form in forms.values() {
        let plan_id = TouchedTrackingPlanId::for_form(&form.id);
        let Some(plan) = graph.plans.get(&plan_id) else {
            push(
                &mut validation,
                FormTrackingIntegrityKind::MissingForm,
                format!("Form `{}` has no touched tracking plan", form.id.as_str()),
            );
            continue;
        };
        if plan.id != plan_id
            || plan.form != form.id
            || plan.component
                != *form
                    .owner
                    .entity_id()
                    .expect("valid Form has component owner")
        {
            push(
                &mut validation,
                FormTrackingIntegrityKind::PlanIdentityMismatch,
                format!(
                    "touched tracking plan for `{}` has non-canonical identity or owner",
                    form.id.as_str()
                ),
            );
        }
        if ownership.component_of_form(&form.id) != form.owner.entity_id() {
            push(
                &mut validation,
                FormTrackingIntegrityKind::FieldFormMismatch,
                format!(
                    "touched tracking Form `{}` is absent from canonical I5 ownership",
                    form.id.as_str()
                ),
            );
        }
        let expected = ordered_fields(fields, &form.id)
            .into_iter()
            .map(|field| FieldTrackingId::for_field(&field.id))
            .collect::<Vec<_>>();
        if plan.fields != expected {
            push(
                &mut validation,
                FormTrackingIntegrityKind::NonCanonicalOrdering,
                format!(
                    "touched tracking plan `{}` does not retain I3 field order",
                    plan.id.as_str()
                ),
            );
        }
    }
    for field in fields.values() {
        let id = FieldTrackingId::for_field(&field.id);
        let Some(record) = graph.fields.get(&id) else {
            push(
                &mut validation,
                FormTrackingIntegrityKind::MissingTrackingRecord,
                format!(
                    "Field `{}` has no touched tracking record",
                    field.id.as_str()
                ),
            );
            continue;
        };
        if record.id != id || record.plan != TouchedTrackingPlanId::for_form(&field.owner_form) {
            push(
                &mut validation,
                FormTrackingIntegrityKind::TrackingIdentityMismatch,
                format!(
                    "touched tracking for `{}` has non-canonical identity",
                    field.id.as_str()
                ),
            );
        }
        if record.field != field.id || record.form != field.owner_form {
            push(
                &mut validation,
                FormTrackingIntegrityKind::FieldFormMismatch,
                format!(
                    "touched tracking `{}` disagrees with I3 Field ownership",
                    record.id.as_str()
                ),
            );
        }
        if record.component != field.owner_component || record.initial_touched {
            push(
                &mut validation,
                FormTrackingIntegrityKind::ComponentMismatch,
                format!(
                    "touched tracking `{}` disagrees with I3 owner or clean initial state",
                    record.id.as_str()
                ),
            );
        }
        if ownership.component_of_field(&field.id) != Some(&field.owner_component) {
            push(
                &mut validation,
                FormTrackingIntegrityKind::FieldFormMismatch,
                format!(
                    "touched tracking `{}` is absent from canonical I5 Field ownership",
                    record.id.as_str()
                ),
            );
        }
        if record.provenance != field.provenance {
            push(
                &mut validation,
                FormTrackingIntegrityKind::MissingProvenance,
                format!(
                    "touched tracking `{}` lacks canonical field provenance",
                    record.id.as_str()
                ),
            );
        }
    }
    for tracking in graph.fields.values() {
        let expected = ordered_bindings_for_field(bindings, &tracking.field)
            .into_iter()
            .filter(|binding| {
                binding.form == tracking.form && binding.component == tracking.component
            })
            .map(|binding| binding.id.clone())
            .collect::<Vec<_>>();
        if tracking.blur_bindings != expected {
            push(
                &mut validation,
                FormTrackingIntegrityKind::BindingMismatch,
                format!(
                    "touched tracking `{}` does not retain the canonical bound-control blur set",
                    tracking.id.as_str()
                ),
            );
        }
        if tracking
            .blur_bindings
            .windows(2)
            .any(|pair| pair[0] >= pair[1])
            && tracking.blur_bindings.len() > 1
        {
            push(
                &mut validation,
                FormTrackingIntegrityKind::NonCanonicalOrdering,
                format!(
                    "touched tracking `{}` blur bindings are not canonical",
                    tracking.id.as_str()
                ),
            );
        }
    }
    for record in graph.fields.values() {
        if !fields.contains_key(&record.field) {
            push(
                &mut validation,
                FormTrackingIntegrityKind::MissingField,
                format!(
                    "touched tracking `{}` references unknown Field",
                    record.id.as_str()
                ),
            );
        }
    }
    validation
}

fn ordered_fields<'a>(
    fields: &'a BTreeMap<FieldId, FormFieldEntity>,
    form: &FormId,
) -> Vec<&'a FormFieldEntity> {
    let mut result = fields
        .values()
        .filter(|field| &field.owner_form == form)
        .collect::<Vec<_>>();
    result.sort_by(|left, right| {
        (left.declaration_order, &left.id).cmp(&(right.declaration_order, &right.id))
    });
    result
}

fn ordered_bindings_for_field<'a>(
    bindings: &'a BTreeMap<FieldBindingId, FormFieldBinding>,
    field: &FieldId,
) -> Vec<&'a FormFieldBinding> {
    let mut result = bindings
        .values()
        .filter(|binding| &binding.field == field)
        .collect::<Vec<_>>();
    result.sort_by(|left, right| {
        (left.authored_order, &left.id).cmp(&(right.authored_order, &right.id))
    });
    result
}

fn push(validation: &mut FormTrackingValidation, kind: FormTrackingIntegrityKind, message: String) {
    validation
        .diagnostics
        .push(FormTrackingIntegrityDiagnostic {
            code: kind.code().to_string(),
            kind,
            message,
        });
    validation.diagnostics.sort_by(|left, right| {
        (left.code.as_str(), left.message.as_str())
            .cmp(&(right.code.as_str(), right.message.as_str()))
    });
    validation.diagnostics.dedup();
    validation.is_valid = false;
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use crate::{
        build_application_semantic_model, DirtyTrackingPlanId, FieldId, FormId, SerializableValue,
        TouchedTrackingPlanId,
    };

    fn model() -> crate::ApplicationSemanticModel {
        let parsed = ezc_parser::parse_file(
            "src/Profile.tsx",
            r#"
@component("profile-editor")
class ProfileEditor {
  @form() profile!: Form;
  @field(this.profile) name = "Austin";
  @field(this.profile) tags: string[] = ["compiler"];
  @field(this.profile) contact: "email" | "phone" = "email";
  @field(this.profile) unbound = false;
  render() {
    return <div>
      <input field={this.name} />
      <input type="radio" value="email" field={this.contact} />
      <input type="radio" value="phone" field={this.contact} />
    </div>;
  }
}
"#,
        );
        build_application_semantic_model(&parsed)
    }

    #[test]
    fn plans_every_field_and_records_dirty_and_blur_facts() {
        let model = model();
        assert!(model.form_tracking.dirty.validation.is_valid);
        assert!(model.form_tracking.touched.validation.is_valid);
        let form = FormId::for_owner(&model.components[0].id, "profile");
        assert!(model
            .form_tracking
            .dirty
            .plans
            .contains_key(&DirtyTrackingPlanId::for_form(&form)));
        assert!(model
            .form_tracking
            .touched
            .plans
            .contains_key(&TouchedTrackingPlanId::for_form(&form)));

        let name = FieldId::for_form(&form, "name");
        let tags = FieldId::for_form(&form, "tags");
        let contact = FieldId::for_form(&form, "contact");
        let unbound = FieldId::for_form(&form, "unbound");
        assert_eq!(
            model.form_tracking.dirty.dirty_after_committed_write(
                &name,
                &SerializableValue::String("Austin".to_string())
            ),
            Some(false)
        );
        assert_eq!(
            model
                .form_tracking
                .dirty
                .dirty_after_committed_write(&name, &SerializableValue::String("Ada".to_string())),
            Some(true)
        );
        assert_eq!(
            model.form_tracking.dirty.dirty_after_committed_write(
                &tags,
                &SerializableValue::Array(vec![SerializableValue::String("compiler".to_string())])
            ),
            Some(false)
        );
        assert_eq!(model.form_tracking.touched.blur_bindings(&contact).len(), 2);
        assert!(model
            .form_tracking
            .touched
            .blur_bindings(&unbound)
            .is_empty());
    }

    #[test]
    fn structural_equality_is_recursive_and_key_order_independent() {
        let mut left = BTreeMap::new();
        left.insert(
            "city".to_string(),
            SerializableValue::String("Austin".to_string()),
        );
        left.insert(
            "zip".to_string(),
            SerializableValue::String("78701".to_string()),
        );
        let mut right = BTreeMap::new();
        right.insert(
            "zip".to_string(),
            SerializableValue::String("78701".to_string()),
        );
        right.insert(
            "city".to_string(),
            SerializableValue::String("Austin".to_string()),
        );
        assert!(super::structurally_equal_serializable_values(
            &SerializableValue::Object(left),
            &SerializableValue::Object(right)
        ));
    }

    #[test]
    fn malformed_tracking_is_reported_without_reconstructing_it() {
        let model = model();
        let mut dirty = model.form_tracking.dirty.clone();
        let record = dirty.fields.values_mut().next().expect("dirty record");
        record.initial_dirty = true;
        let validation = super::validate_dirty_tracking_graph(
            &dirty,
            &model.forms,
            &model.form_fields,
            &model.form_field_bindings,
            &model.form_ownership,
        );
        assert!(!validation.is_valid);
        assert!(validation
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.kind
                == super::FormTrackingIntegrityKind::InitialValueMismatch));
    }

    #[test]
    fn remains_internal_and_does_not_change_the_frozen_public_graph() {
        let model = model();
        let graph = crate::build_semantic_graph(&model);
        let json = crate::semantic_graph_json(&graph);
        assert!(!json.contains("dirty-plan"));
        assert!(!json.contains("touched-plan"));
        assert!(!json.contains("/tracking"));
    }
}
