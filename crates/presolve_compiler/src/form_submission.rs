//! I9 compiler-owned declaration-level Form submission planning.

use std::collections::BTreeMap;

use crate::component_graph::AuthoredSubmissionDeclarationFact;
use crate::{
    BindingTable, ComponentNode, EffectTriggerPlan, FieldId, FormEntity, FormFieldEntity, FormId,
    ImportBindingTarget, SemanticId, SourceProvenance, SubmissionDeclarationCandidateId,
    SubmissionPlanId, ValidationRule, ValidationRuleId,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum SubmissionDeclarationViolation {
    InvalidOwner,
    InvalidDecoratorInvocation,
    InvalidDecoratorArity,
    InvalidFormDesignator,
    UnresolvedForm,
    InvalidAction,
    StaticMethod,
    AsyncMethod,
    ParameterizedMethod,
    InvalidReturnType,
    InvalidCapability,
    InheritedDeclaration,
    DuplicateFormSubmission,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SubmissionDeclarationCandidate {
    pub id: SubmissionDeclarationCandidateId,
    pub owner_component: Option<SemanticId>,
    pub method: Option<SemanticId>,
    pub form_designator: Option<String>,
    pub resolved_form: Option<FormId>,
    pub action_batch: Option<SemanticId>,
    pub capability: Option<FormSubmissionCapability>,
    pub provenance: SourceProvenance,
    pub decorator_provenance: SourceProvenance,
    pub form_designator_provenance: Option<SourceProvenance>,
    pub violations: Vec<SubmissionDeclarationViolation>,
}

impl SubmissionDeclarationCandidate {
    #[must_use]
    pub fn is_valid(&self) -> bool {
        self.violations.is_empty()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SubmitResetPolicy {
    Never,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FormSubmissionPlan {
    pub id: SubmissionPlanId,
    pub form: FormId,
    pub component: SemanticId,
    pub candidate: SubmissionDeclarationCandidateId,
    pub action_method: SemanticId,
    pub action_batch: SemanticId,
    pub capability: Option<FormSubmissionCapability>,
    pub validation_rules: Vec<ValidationRuleId>,
    pub blocks_action_on_invalid: bool,
    pub reset_policy: SubmitResetPolicy,
    pub provenance: SourceProvenance,
}

/// Exact integrity-bound client capability selected for one Form submission.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct FormSubmissionCapability {
    pub id: String,
    pub module_specifier: String,
    pub package: String,
    pub version: String,
    pub integrity: String,
    pub export: String,
    pub runtime_module: String,
    pub resume_policy: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct SubmissionProducts {
    pub candidates: Vec<SubmissionDeclarationCandidate>,
    pub plans: BTreeMap<SubmissionPlanId, FormSubmissionPlan>,
}

impl SubmissionProducts {
    #[must_use]
    pub fn plan(&self, form: &FormId) -> Option<&FormSubmissionPlan> {
        self.plans.get(&SubmissionPlanId::for_form(form))
    }
}

#[must_use]
#[allow(clippy::too_many_lines)]
/// # Panics
///
/// Panics when a candidate previously classified as valid lacks an exact Form,
/// method, or action-batch identity. That violates I9 staged lowering.
pub fn collect_submission_products(
    components: &[ComponentNode],
    forms: &BTreeMap<FormId, FormEntity>,
    fields: &BTreeMap<FieldId, FormFieldEntity>,
    rules: &BTreeMap<ValidationRuleId, ValidationRule>,
    action_batches: &EffectTriggerPlan,
    bindings: Option<&BindingTable>,
    retained_capabilities: Option<
        &BTreeMap<SubmissionDeclarationCandidateId, FormSubmissionCapability>,
    >,
) -> SubmissionProducts {
    let mut candidates = components
        .iter()
        .flat_map(|component| component.submission_declaration_facts.iter())
        .map(|fact| lower_candidate(fact, forms, action_batches, bindings, retained_capabilities))
        .collect::<Vec<_>>();
    candidates.sort_by(|left, right| {
        (
            left.provenance.path.as_path(),
            left.decorator_provenance.span.start,
            left.id.as_str(),
        )
            .cmp(&(
                right.provenance.path.as_path(),
                right.decorator_provenance.span.start,
                right.id.as_str(),
            ))
    });

    let mut groups = BTreeMap::<FormId, Vec<usize>>::new();
    for (index, candidate) in candidates
        .iter()
        .enumerate()
        .filter(|(_, candidate)| candidate.is_valid())
    {
        groups
            .entry(
                candidate
                    .resolved_form
                    .clone()
                    .expect("valid submission resolves Form"),
            )
            .or_default()
            .push(index);
    }
    for indexes in groups.values().filter(|indexes| indexes.len() > 1) {
        for index in indexes {
            add_violation(
                &mut candidates[*index],
                SubmissionDeclarationViolation::DuplicateFormSubmission,
            );
        }
    }
    let mut methods = BTreeMap::<SemanticId, Vec<usize>>::new();
    for (index, candidate) in candidates
        .iter()
        .enumerate()
        .filter(|(_, candidate)| candidate.is_valid())
    {
        methods
            .entry(
                candidate
                    .method
                    .clone()
                    .expect("valid submission has method"),
            )
            .or_default()
            .push(index);
    }
    for indexes in methods.values().filter(|indexes| indexes.len() > 1) {
        for index in indexes {
            add_violation(
                &mut candidates[*index],
                SubmissionDeclarationViolation::DuplicateFormSubmission,
            );
        }
    }

    let mut plans = BTreeMap::new();
    for candidate in candidates.iter().filter(|candidate| candidate.is_valid()) {
        let form = candidate
            .resolved_form
            .clone()
            .expect("valid submission resolves Form");
        let form_entity = forms.get(&form).expect("valid submission Form exists");
        let action_method = candidate
            .method
            .clone()
            .expect("valid submission has action method");
        let action_batch = candidate
            .action_batch
            .clone()
            .expect("valid submission has action batch");
        let mut validation_rules = rules
            .values()
            .filter(|rule| rule.owner_form == form)
            .collect::<Vec<_>>();
        validation_rules.sort_by(|left, right| {
            (
                left.field_authored_order,
                left.rule_authored_order,
                &left.id,
            )
                .cmp(&(
                    right.field_authored_order,
                    right.rule_authored_order,
                    &right.id,
                ))
        });
        plans.insert(
            SubmissionPlanId::for_form(&form),
            FormSubmissionPlan {
                id: SubmissionPlanId::for_form(&form),
                form,
                component: form_entity
                    .owner
                    .entity_id()
                    .expect("valid Form has Component owner")
                    .clone(),
                candidate: candidate.id.clone(),
                action_method,
                action_batch,
                capability: candidate.capability.clone(),
                validation_rules: validation_rules
                    .into_iter()
                    .map(|rule| rule.id.clone())
                    .collect(),
                blocks_action_on_invalid: true,
                reset_policy: SubmitResetPolicy::Never,
                provenance: candidate.provenance.clone(),
            },
        );
    }
    let _ = fields; // I3 authored order is already retained by each I6 Rule.
    SubmissionProducts { candidates, plans }
}

fn lower_candidate(
    fact: &AuthoredSubmissionDeclarationFact,
    forms: &BTreeMap<FormId, FormEntity>,
    action_batches: &EffectTriggerPlan,
    bindings: Option<&BindingTable>,
    retained_capabilities: Option<
        &BTreeMap<SubmissionDeclarationCandidateId, FormSubmissionCapability>,
    >,
) -> SubmissionDeclarationCandidate {
    let mut violations = Vec::new();
    if fact.owner_component.is_none() {
        violations.push(SubmissionDeclarationViolation::InvalidOwner);
    }
    if !fact.submit_invoked {
        violations.push(SubmissionDeclarationViolation::InvalidDecoratorInvocation);
    }
    if fact.submit_argument_count != 1 {
        violations.push(SubmissionDeclarationViolation::InvalidDecoratorArity);
    }
    if fact.form_designator.is_none() {
        violations.push(SubmissionDeclarationViolation::InvalidFormDesignator);
    }
    if !fact.has_action || !fact.action_invoked || fact.action_argument_count != 0 {
        violations.push(SubmissionDeclarationViolation::InvalidAction);
    }
    if fact.is_static {
        violations.push(SubmissionDeclarationViolation::StaticMethod);
    }
    if fact.native_inline {
        if fact.parameter_count != 1 {
            violations.push(SubmissionDeclarationViolation::ParameterizedMethod);
        }
    } else {
        if fact.is_async {
            violations.push(SubmissionDeclarationViolation::AsyncMethod);
        }
        if fact.parameter_count != 0 {
            violations.push(SubmissionDeclarationViolation::ParameterizedMethod);
        }
        if fact.return_type.as_deref() != Some("void") {
            violations.push(SubmissionDeclarationViolation::InvalidReturnType);
        }
    }
    if fact.inherited {
        violations.push(SubmissionDeclarationViolation::InheritedDeclaration);
    }
    let resolved_form = fact
        .owner_component
        .as_ref()
        .zip(fact.form_designator.as_ref())
        .and_then(|(component, name)| {
            forms
                .values()
                .find(|form| form.owner.entity_id() == Some(component) && form.name == *name)
                .map(|form| form.id.clone())
        });
    if fact.submit_invoked
        && fact.submit_argument_count == 1
        && fact.form_designator.is_some()
        && resolved_form.is_none()
    {
        violations.push(SubmissionDeclarationViolation::UnresolvedForm);
    }
    let action_batch = fact
        .owner_component
        .as_ref()
        .zip(fact.method_name.as_ref())
        .map(|(component, method)| component.action_batch(method))
        .filter(|id| action_batches.action_batches.contains_key(id));
    if fact.has_action && action_batch.is_none() {
        violations.push(SubmissionDeclarationViolation::InvalidAction);
    }
    let capability = retained_capabilities
        .and_then(|capabilities| capabilities.get(&fact.id))
        .cloned()
        .or_else(|| {
            fact.capability_local_name.as_ref().and_then(|local| {
                let binding = bindings?.resolve_import(&fact.method_provenance.path, local)?;
                if binding.imported_name == "default" {
                    return None;
                }
                let ImportBindingTarget::SemanticPackage {
                    package,
                    version,
                    integrity,
                    export,
                    runtime_module,
                    resume_policy,
                    form_submission: Some(_),
                    ..
                } = &binding.target
                else {
                    return None;
                };
                Some(FormSubmissionCapability {
                    id: format!(
                        "form-submission-capability:{package}@{version}:{export}:{integrity}"
                    ),
                    module_specifier: binding.source_module.to_string_lossy().into_owned(),
                    package: package.clone(),
                    version: version.clone(),
                    integrity: integrity.clone(),
                    export: export.clone(),
                    runtime_module: runtime_module.clone(),
                    resume_policy: resume_policy.clone(),
                })
            })
        });
    if fact.capability_local_name.is_some() && capability.is_none() {
        violations.push(SubmissionDeclarationViolation::InvalidCapability);
    }
    violations.sort();
    violations.dedup();
    SubmissionDeclarationCandidate {
        id: fact.id.clone(),
        owner_component: fact.owner_component.clone(),
        method: fact.method.clone(),
        form_designator: fact.form_designator.clone(),
        resolved_form,
        action_batch,
        capability,
        provenance: fact.method_provenance.clone(),
        decorator_provenance: fact.decorator_provenance.clone(),
        form_designator_provenance: fact.form_designator_provenance.clone(),
        violations,
    }
}

fn add_violation(
    candidate: &mut SubmissionDeclarationCandidate,
    violation: SubmissionDeclarationViolation,
) {
    if !candidate.violations.contains(&violation) {
        candidate.violations.push(violation);
        candidate.violations.sort();
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        build_application_semantic_model, FormId, SubmissionDeclarationViolation, SubmissionPlanId,
    };

    #[test]
    fn lowers_one_submit_plan_with_the_complete_form_rule_order() {
        let parsed = presolve_parser::parse_file(
            "src/Profile.tsx",
            r#"
@component("profile-editor")
class ProfileEditor {
  @form() profile!: Form;
  @validate(required()) @field("profile") name = "";
  @validate(min(0)) @field("profile") age = 0;
  @action() @submit("profile") save(): void {}
  render() { return <input field={this.name} />; }
}
"#,
        );
        let model = build_application_semantic_model(&parsed);
        let form = FormId::for_owner(&model.components[0].id, "profile");
        let plan = model.submissions.plan(&form).expect("submission plan");
        assert_eq!(plan.id, SubmissionPlanId::for_form(&form));
        assert_eq!(plan.validation_rules.len(), 2);
        assert_eq!(
            plan.action_batch,
            model.components[0].id.action_batch("save")
        );
        assert!(matches!(plan.reset_policy, super::SubmitResetPolicy::Never));
    }

    #[test]
    fn retains_invalid_submit_candidates_without_plan_or_winner() {
        let parsed = presolve_parser::parse_file(
            "src/Profile.tsx",
            r#"
@component("profile-editor")
class ProfileEditor {
  @form() profile!: Form;
  @field(this.profile) name = "";
  @submit(this.profile) missingAction(): void {}
  @action() @submit(this.profile) duplicate(value: string): void {}
  render() { return <input field={this.name} />; }
}
"#,
        );
        let model = build_application_semantic_model(&parsed);
        assert_eq!(model.submissions.candidates.len(), 2);
        assert!(model.submissions.plans.is_empty());
        assert!(model
            .submissions
            .candidates
            .iter()
            .any(|candidate| candidate
                .violations
                .contains(&SubmissionDeclarationViolation::InvalidAction)));
        assert!(model
            .submissions
            .candidates
            .iter()
            .any(|candidate| candidate
                .violations
                .contains(&SubmissionDeclarationViolation::ParameterizedMethod)));
    }
}
