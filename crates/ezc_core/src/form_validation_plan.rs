use std::collections::{BTreeMap, BTreeSet};

use crate::{
    ExecutionBoundary, FieldDependencyId, FieldId, FormEntity, FormFieldEntity, FormId,
    FormOwnershipGraph, FormOwnershipNodeKey, SemanticId, SourceProvenance, ValidationGraph,
    ValidationGraphEdgeKind, ValidationGraphNodeKey, ValidationPlanId, ValidationRule,
    ValidationRuleId, ValidationRuleKind,
};

/// I7 deliberately retains no initial- or submission-validation policy.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ValidationPlanningStatus {
    Deferred,
}

/// One direct I6 `ValidationRule -> source Field` read relation, projected into
/// I7 invalidation planning. It is declaration-level and immutable.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FieldValidationDependency {
    pub id: FieldDependencyId,
    pub plan: ValidationPlanId,
    pub form: FormId,
    pub component: SemanticId,
    pub source_field: FieldId,
    pub target_field: FieldId,
    pub dependent_rule: ValidationRuleId,
    pub rule_kind: ValidationRuleKind,
    pub execution_boundary: ExecutionBoundary,
    pub source_field_order: usize,
    pub target_field_order: usize,
    pub rule_order: usize,
    pub provenance: SourceProvenance,
    pub source_field_provenance: SourceProvenance,
    pub target_field_provenance: SourceProvenance,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FieldValidationSourceEntry {
    pub source_field: FieldId,
    pub form: FormId,
    pub authored_field_order: usize,
    pub directly_invalidated_rules: Vec<ValidationRuleId>,
    pub directly_invalidated_target_fields: Vec<FieldId>,
    pub dependencies: Vec<FieldDependencyId>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FieldValidationTargetEntry {
    pub target_field: FieldId,
    pub form: FormId,
    pub authored_field_order: usize,
    pub cross_field_rules: Vec<ValidationRuleId>,
    pub source_fields: Vec<FieldId>,
    pub dependencies: Vec<FieldDependencyId>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum FieldDependencyBlockReason {
    MissingRule,
    MissingSourceField,
    MissingTargetField,
    MissingForm,
    RuleNotInValidationGraph,
    RuleHasNoDependency,
    RuleCycleExcluded,
    RuleCandidateOnly,
    SourceTargetFormMismatch,
    ComponentMismatch,
    UnsupportedBoundary,
    MissingProvenance,
    IdentityMismatch,
}

/// Retained only for malformed or stale canonical I6/I5 products. Normal I6
/// candidates remain solely in I6 candidate registries.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BlockedFieldValidationDependency {
    pub rule: Option<ValidationRuleId>,
    pub source_field: Option<FieldId>,
    pub target_field: Option<FieldId>,
    pub form: Option<FormId>,
    pub reason: FieldDependencyBlockReason,
    pub provenance: Option<SourceProvenance>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ValidationDependencyPlanIntegrityDiagnostic {
    pub code: String,
    pub kind: ValidationDependencyPlanIntegrityKind,
    pub message: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum ValidationDependencyPlanIntegrityKind {
    DuplicatePlan,
    MissingPlanForm,
    UnknownForm,
    UnknownSourceField,
    UnknownTargetField,
    UnknownRule,
    RuleNotInValidationGraph,
    MissingI6Dependency,
    DuplicateDependencyProjection,
    MissingDependencyProjection,
    PlanFormMismatch,
    FieldFormMismatch,
    RuleTargetMismatch,
    RuleFormMismatch,
    ComponentMismatch,
    SelfDependency,
    CycleRulePromoted,
    UnsupportedBoundary,
    SourceIndexMismatch,
    TargetIndexMismatch,
    InvalidationIndexMismatch,
    DuplicateScheduledRule,
    DuplicateScheduledTarget,
    TransitiveInvalidationLeak,
    CandidateIdentityPromoted,
    InstanceIdentityLeak,
    MissingProvenance,
    NonCanonicalOrdering,
    PlanIdentityDrift,
}

impl ValidationDependencyPlanIntegrityKind {
    #[must_use]
    pub const fn code(self) -> &'static str {
        match self {
            Self::DuplicatePlan => "EZASM1242",
            Self::MissingPlanForm => "EZASM1243",
            Self::UnknownForm => "EZASM1244",
            Self::UnknownSourceField => "EZASM1245",
            Self::UnknownTargetField => "EZASM1246",
            Self::UnknownRule => "EZASM1247",
            Self::RuleNotInValidationGraph => "EZASM1248",
            Self::MissingI6Dependency => "EZASM1249",
            Self::DuplicateDependencyProjection => "EZASM1250",
            Self::MissingDependencyProjection => "EZASM1251",
            Self::PlanFormMismatch => "EZASM1252",
            Self::FieldFormMismatch => "EZASM1253",
            Self::RuleTargetMismatch => "EZASM1254",
            Self::RuleFormMismatch => "EZASM1255",
            Self::ComponentMismatch => "EZASM1256",
            Self::SelfDependency => "EZASM1257",
            Self::CycleRulePromoted => "EZASM1258",
            Self::UnsupportedBoundary => "EZASM1259",
            Self::SourceIndexMismatch => "EZASM1260",
            Self::TargetIndexMismatch => "EZASM1261",
            Self::InvalidationIndexMismatch => "EZASM1262",
            Self::DuplicateScheduledRule => "EZASM1263",
            Self::DuplicateScheduledTarget => "EZASM1264",
            Self::TransitiveInvalidationLeak => "EZASM1265",
            Self::CandidateIdentityPromoted => "EZASM1266",
            Self::InstanceIdentityLeak => "EZASM1267",
            Self::MissingProvenance => "EZASM1268",
            Self::NonCanonicalOrdering => "EZASM1269",
            Self::PlanIdentityDrift => "EZASM1270",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ValidationDependencyPlanValidation {
    pub diagnostics: Vec<ValidationDependencyPlanIntegrityDiagnostic>,
    pub is_valid: bool,
}

impl Default for ValidationDependencyPlanValidation {
    fn default() -> Self {
        Self {
            diagnostics: Vec::new(),
            is_valid: true,
        }
    }
}

/// Exactly one declaration-level plan exists for every valid Form, including
/// Forms with no Fields or no cross-field rules.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FormValidationDependencyPlan {
    pub id: ValidationPlanId,
    pub form: FormId,
    pub component: SemanticId,
    pub fields: Vec<FieldId>,
    pub dependencies: Vec<FieldDependencyId>,
    pub source_entries: Vec<FieldValidationSourceEntry>,
    pub target_entries: Vec<FieldValidationTargetEntry>,
    pub initial_validation: ValidationPlanningStatus,
    pub submission_validation: ValidationPlanningStatus,
    pub validation: ValidationDependencyPlanValidation,
}

/// Complete I7 internal planning product. It is not a public schema, runtime
/// artifact, or new ASM ownership authority.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ValidationDependencyPlans {
    pub plans: BTreeMap<ValidationPlanId, FormValidationDependencyPlan>,
    pub dependencies: BTreeMap<FieldDependencyId, FieldValidationDependency>,
    pub blocked: Vec<BlockedFieldValidationDependency>,
    pub validation: ValidationDependencyPlanValidation,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FieldChangeSet {
    pub form: FormId,
    pub changed_fields: Vec<FieldId>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FieldValidationChangePlan {
    pub plan: ValidationPlanId,
    pub form: FormId,
    pub changed_field: FieldId,
    pub scheduled_rules: Vec<ValidationRuleId>,
    pub scheduled_target_fields: Vec<FieldId>,
    pub dependencies: Vec<FieldDependencyId>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FieldChangeValidationSchedule {
    pub plan: ValidationPlanId,
    pub form: FormId,
    pub changed_fields: Vec<FieldId>,
    pub scheduled_rules: Vec<ValidationRuleId>,
    pub scheduled_target_fields: Vec<FieldId>,
    pub triggering_dependencies: Vec<FieldDependencyId>,
}

impl ValidationDependencyPlans {
    #[must_use]
    pub fn validation_plan(&self, form: &FormId) -> Option<&FormValidationDependencyPlan> {
        self.plans.get(&ValidationPlanId::for_form(form))
    }

    #[must_use]
    pub fn validation_plan_by_id(
        &self,
        id: &ValidationPlanId,
    ) -> Option<&FormValidationDependencyPlan> {
        self.plans.get(id)
    }

    #[must_use]
    pub fn dependency(&self, id: &FieldDependencyId) -> Option<&FieldValidationDependency> {
        self.dependencies.get(id)
    }

    #[must_use]
    pub fn dependencies_of_plan(&self, plan: &ValidationPlanId) -> Vec<&FieldValidationDependency> {
        let mut dependencies = self
            .dependencies
            .values()
            .filter(|dependency| &dependency.plan == plan)
            .collect::<Vec<_>>();
        dependencies.sort_by(|left, right| dependency_order(left, right));
        dependencies
    }

    #[must_use]
    pub fn source_entry(&self, field: &FieldId) -> Option<&FieldValidationSourceEntry> {
        self.plans.values().find_map(|plan| {
            plan.source_entries
                .iter()
                .find(|entry| &entry.source_field == field)
        })
    }

    #[must_use]
    pub fn target_entry(&self, field: &FieldId) -> Option<&FieldValidationTargetEntry> {
        self.plans.values().find_map(|plan| {
            plan.target_entries
                .iter()
                .find(|entry| &entry.target_field == field)
        })
    }

    #[must_use]
    pub fn directly_invalidated_rules(&self, field: &FieldId) -> Vec<&ValidationRuleId> {
        self.source_entry(field).map_or_else(Vec::new, |entry| {
            entry.directly_invalidated_rules.iter().collect()
        })
    }

    #[must_use]
    pub fn directly_invalidated_target_fields(&self, field: &FieldId) -> Vec<&FieldId> {
        self.source_entry(field).map_or_else(Vec::new, |entry| {
            entry.directly_invalidated_target_fields.iter().collect()
        })
    }

    #[must_use]
    pub fn dependencies_from_field(&self, field: &FieldId) -> Vec<&FieldValidationDependency> {
        let mut dependencies = self
            .dependencies
            .values()
            .filter(|dependency| &dependency.source_field == field)
            .collect::<Vec<_>>();
        dependencies.sort_by(|left, right| dependency_order(left, right));
        dependencies
    }

    #[must_use]
    pub fn dependencies_to_field(&self, field: &FieldId) -> Vec<&FieldValidationDependency> {
        let mut dependencies = self
            .dependencies
            .values()
            .filter(|dependency| &dependency.target_field == field)
            .collect::<Vec<_>>();
        dependencies.sort_by(|left, right| dependency_order(left, right));
        dependencies
    }

    #[must_use]
    pub fn source_field_of_dependency(&self, id: &FieldDependencyId) -> Option<&FieldId> {
        self.dependency(id)
            .map(|dependency| &dependency.source_field)
    }

    #[must_use]
    pub fn target_field_of_dependency(&self, id: &FieldDependencyId) -> Option<&FieldId> {
        self.dependency(id)
            .map(|dependency| &dependency.target_field)
    }

    #[must_use]
    pub fn rule_of_dependency(&self, id: &FieldDependencyId) -> Option<&ValidationRuleId> {
        self.dependency(id)
            .map(|dependency| &dependency.dependent_rule)
    }

    #[must_use]
    pub fn change_plan(&self, field: &FieldId) -> Option<FieldValidationChangePlan> {
        let plan = self
            .plans
            .values()
            .find(|plan| plan.fields.contains(field))?;
        let schedule = self.schedule_change_set(&plan.form, std::slice::from_ref(field))?;
        Some(FieldValidationChangePlan {
            plan: schedule.plan,
            form: schedule.form,
            changed_field: field.clone(),
            scheduled_rules: schedule.scheduled_rules,
            scheduled_target_fields: schedule.scheduled_target_fields,
            dependencies: schedule.triggering_dependencies,
        })
    }

    /// Schedules only directly read dependencies. An unknown Form or Field is
    /// rejected by returning no schedule; this query never substitutes a Form.
    #[must_use]
    pub fn schedule_change_set(
        &self,
        form: &FormId,
        changed_fields: &[FieldId],
    ) -> Option<FieldChangeValidationSchedule> {
        let plan = self.validation_plan(form)?;
        if changed_fields
            .iter()
            .any(|field| !plan.fields.contains(field))
        {
            return None;
        }

        let orders = plan_field_orders(plan);
        let mut normalized = changed_fields
            .iter()
            .cloned()
            .collect::<BTreeSet<_>>()
            .into_iter()
            .collect::<Vec<_>>();
        normalized.sort_by_key(|field| {
            (
                orders.get(field).copied().unwrap_or(usize::MAX),
                field.clone(),
            )
        });

        let triggered = normalized
            .iter()
            .flat_map(|field| self.dependencies_from_field(field).into_iter())
            .collect::<Vec<_>>();
        let mut dependency_ids = triggered
            .iter()
            .map(|dependency| dependency.id.clone())
            .collect::<Vec<_>>();
        dependency_ids.sort_by_key(|id| {
            self.dependencies
                .get(id)
                .map(dependency_sort_key)
                .unwrap_or((usize::MAX, usize::MAX, usize::MAX, id.clone()))
        });

        let mut rules = triggered
            .iter()
            .map(|dependency| dependency.dependent_rule.clone())
            .collect::<BTreeSet<_>>()
            .into_iter()
            .collect::<Vec<_>>();
        let rule_keys = triggered
            .iter()
            .map(|dependency| {
                (
                    dependency.dependent_rule.clone(),
                    rule_schedule_key(dependency),
                )
            })
            .collect::<BTreeMap<_, _>>();
        rules.sort_by_key(|rule| {
            rule_keys
                .get(rule)
                .cloned()
                .unwrap_or((usize::MAX, usize::MAX, rule.clone()))
        });

        let mut target_fields = triggered
            .iter()
            .map(|dependency| dependency.target_field.clone())
            .collect::<BTreeSet<_>>()
            .into_iter()
            .collect::<Vec<_>>();
        target_fields.sort_by_key(|field| {
            (
                orders.get(field).copied().unwrap_or(usize::MAX),
                field.clone(),
            )
        });

        Some(FieldChangeValidationSchedule {
            plan: plan.id.clone(),
            form: form.clone(),
            changed_fields: normalized,
            scheduled_rules: rules,
            scheduled_target_fields: target_fields,
            triggering_dependencies: dependency_ids,
        })
    }

    #[must_use]
    pub fn plans_of_component(&self, component: &SemanticId) -> Vec<&FormValidationDependencyPlan> {
        self.plans
            .values()
            .filter(|plan| &plan.component == component)
            .collect()
    }

    #[must_use]
    pub fn blocked_dependencies(&self) -> &[BlockedFieldValidationDependency] {
        &self.blocked
    }
}

/// Builds I7 exclusively from I3/I5/I6 canonical products. No parser facts,
/// JSX controls, runtime state, or declaration-name resolution is consulted.
///
/// # Panics
///
/// Panics only when an I3-valid Form lacks its canonical Component owner, or
/// when a dependency accepted after I5/I6 reciprocity checks is absent from
/// the plan it canonically names.
#[must_use]
pub fn collect_validation_dependency_plans(
    forms: &BTreeMap<FormId, FormEntity>,
    fields: &BTreeMap<FieldId, FormFieldEntity>,
    rules: &BTreeMap<ValidationRuleId, ValidationRule>,
    form_ownership: &FormOwnershipGraph,
    validation_graph: &ValidationGraph,
) -> ValidationDependencyPlans {
    let mut plans = initialize_validation_plans(forms, fields);
    let (dependencies, mut blocked) =
        collect_direct_dependencies(forms, fields, rules, validation_graph);
    populate_dependency_entries(&mut plans, &dependencies);
    canonicalize_plans(&mut plans, &dependencies);
    blocked.sort_by(blocked_order);
    let mut product = ValidationDependencyPlans {
        plans,
        dependencies,
        blocked,
        validation: ValidationDependencyPlanValidation::default(),
    };
    product.validation = validate_validation_dependency_plans(
        &product,
        forms,
        fields,
        rules,
        form_ownership,
        validation_graph,
    );
    for plan in product.plans.values_mut() {
        plan.validation = product.validation.clone();
    }
    product
}

fn initialize_validation_plans(
    forms: &BTreeMap<FormId, FormEntity>,
    fields: &BTreeMap<FieldId, FormFieldEntity>,
) -> BTreeMap<ValidationPlanId, FormValidationDependencyPlan> {
    forms
        .values()
        .map(|form| {
            let form_fields = ordered_fields_for_form(fields, &form.id);
            let plan = FormValidationDependencyPlan {
                id: ValidationPlanId::for_form(&form.id),
                form: form.id.clone(),
                component: form
                    .owner
                    .entity_id()
                    .cloned()
                    .expect("valid Form has a canonical Component owner"),
                fields: form_fields.iter().map(|field| field.id.clone()).collect(),
                dependencies: Vec::new(),
                source_entries: form_fields
                    .iter()
                    .map(|field| FieldValidationSourceEntry {
                        source_field: field.id.clone(),
                        form: form.id.clone(),
                        authored_field_order: field.declaration_order,
                        directly_invalidated_rules: Vec::new(),
                        directly_invalidated_target_fields: Vec::new(),
                        dependencies: Vec::new(),
                    })
                    .collect(),
                target_entries: form_fields
                    .iter()
                    .map(|field| FieldValidationTargetEntry {
                        target_field: field.id.clone(),
                        form: form.id.clone(),
                        authored_field_order: field.declaration_order,
                        cross_field_rules: Vec::new(),
                        source_fields: Vec::new(),
                        dependencies: Vec::new(),
                    })
                    .collect(),
                initial_validation: ValidationPlanningStatus::Deferred,
                submission_validation: ValidationPlanningStatus::Deferred,
                validation: ValidationDependencyPlanValidation::default(),
            };
            (plan.id.clone(), plan)
        })
        .collect()
}

fn collect_direct_dependencies(
    forms: &BTreeMap<FormId, FormEntity>,
    fields: &BTreeMap<FieldId, FormFieldEntity>,
    rules: &BTreeMap<ValidationRuleId, ValidationRule>,
    validation_graph: &ValidationGraph,
) -> (
    BTreeMap<FieldDependencyId, FieldValidationDependency>,
    Vec<BlockedFieldValidationDependency>,
) {
    let mut dependencies = BTreeMap::new();
    let mut blocked = Vec::new();
    for rule in rules.values() {
        let Some(source_id) = rule.dependency.clone() else {
            continue;
        };
        match project_rule_dependency(rule, &source_id, forms, fields, validation_graph) {
            Ok(dependency) => {
                if dependencies
                    .insert(dependency.id.clone(), dependency)
                    .is_some()
                {
                    blocked.push(blocked_for_rule(
                        rule,
                        Some(source_id),
                        FieldDependencyBlockReason::IdentityMismatch,
                        None,
                    ));
                }
            }
            Err(blocked_dependency) => blocked.push(*blocked_dependency),
        }
    }
    (dependencies, blocked)
}

fn project_rule_dependency(
    rule: &ValidationRule,
    source_id: &FieldId,
    forms: &BTreeMap<FormId, FormEntity>,
    fields: &BTreeMap<FieldId, FormFieldEntity>,
    validation_graph: &ValidationGraph,
) -> Result<FieldValidationDependency, Box<BlockedFieldValidationDependency>> {
    let Some(source) = fields.get(source_id) else {
        return Err(Box::new(blocked_for_rule(
            rule,
            Some(source_id.clone()),
            FieldDependencyBlockReason::MissingSourceField,
            None,
        )));
    };
    let Some(target) = fields.get(&rule.target_field) else {
        return Err(Box::new(blocked_for_rule(
            rule,
            Some(source.id.clone()),
            FieldDependencyBlockReason::MissingTargetField,
            None,
        )));
    };
    let Some(form) = forms.get(&rule.owner_form) else {
        return Err(Box::new(blocked_for_rule(
            rule,
            Some(source.id.clone()),
            FieldDependencyBlockReason::MissingForm,
            None,
        )));
    };
    let edges = matching_i6_edges(validation_graph, rule, source);
    if let Some(reason) =
        dependency_block_reason(rule, source, target, form, validation_graph, &edges)
    {
        return Err(Box::new(blocked_for_rule(
            rule,
            Some(source.id.clone()),
            reason,
            edges.first(),
        )));
    }
    let id = FieldDependencyId::for_rule_and_source(&rule.id, &source.id);
    Ok(FieldValidationDependency {
        id,
        plan: ValidationPlanId::for_form(&rule.owner_form),
        form: rule.owner_form.clone(),
        component: rule.owner_component.clone(),
        source_field: source.id.clone(),
        target_field: target.id.clone(),
        dependent_rule: rule.id.clone(),
        rule_kind: rule.kind,
        execution_boundary: rule.boundary,
        source_field_order: source.declaration_order,
        target_field_order: target.declaration_order,
        rule_order: rule.rule_authored_order,
        provenance: edges[0].provenance.clone(),
        source_field_provenance: source.provenance.clone(),
        target_field_provenance: target.provenance.clone(),
    })
}

fn matching_i6_edges<'a>(
    graph: &'a ValidationGraph,
    rule: &ValidationRule,
    source: &FormFieldEntity,
) -> Vec<&'a crate::ValidationGraphEdge> {
    graph
        .edges
        .iter()
        .filter(|edge| {
            edge.kind == ValidationGraphEdgeKind::RuleDependsOnField
                && edge.source == ValidationGraphNodeKey::ValidationRule(rule.id.clone())
                && edge.target == ValidationGraphNodeKey::FormField(source.id.clone())
        })
        .collect()
}

fn dependency_block_reason(
    rule: &ValidationRule,
    source: &FormFieldEntity,
    target: &FormFieldEntity,
    form: &FormEntity,
    graph: &ValidationGraph,
    edges: &[&crate::ValidationGraphEdge],
) -> Option<FieldDependencyBlockReason> {
    if !graph
        .nodes
        .contains_key(&ValidationGraphNodeKey::ValidationRule(rule.id.clone()))
    {
        return Some(FieldDependencyBlockReason::RuleNotInValidationGraph);
    }
    if edges.len() != 1 || source.id == target.id {
        return Some(FieldDependencyBlockReason::IdentityMismatch);
    }
    if graph
        .cycles
        .iter()
        .any(|cycle| cycle.candidates.contains(&rule.candidate_id))
    {
        return Some(FieldDependencyBlockReason::RuleCycleExcluded);
    }
    if source.owner_form != rule.owner_form || target.owner_form != rule.owner_form {
        return Some(FieldDependencyBlockReason::SourceTargetFormMismatch);
    }
    if source.owner_component != rule.owner_component
        || target.owner_component != rule.owner_component
        || form.owner.entity_id() != Some(&rule.owner_component)
    {
        return Some(FieldDependencyBlockReason::ComponentMismatch);
    }
    if rule.boundary != ExecutionBoundary::Client {
        return Some(FieldDependencyBlockReason::UnsupportedBoundary);
    }
    (!has_provenance(&rule.provenance)
        || !has_provenance(&source.provenance)
        || !has_provenance(&target.provenance)
        || !has_provenance(&edges[0].provenance))
    .then_some(FieldDependencyBlockReason::MissingProvenance)
}

fn populate_dependency_entries(
    plans: &mut BTreeMap<ValidationPlanId, FormValidationDependencyPlan>,
    dependencies: &BTreeMap<FieldDependencyId, FieldValidationDependency>,
) {
    for dependency in dependencies.values() {
        let plan = plans
            .get_mut(&dependency.plan)
            .expect("a valid dependency has a canonical Form plan");
        plan.dependencies.push(dependency.id.clone());
        let source = plan
            .source_entries
            .iter_mut()
            .find(|entry| entry.source_field == dependency.source_field)
            .expect("dependency source belongs to its plan");
        source.dependencies.push(dependency.id.clone());
        source
            .directly_invalidated_rules
            .push(dependency.dependent_rule.clone());
        source
            .directly_invalidated_target_fields
            .push(dependency.target_field.clone());
        let target = plan
            .target_entries
            .iter_mut()
            .find(|entry| entry.target_field == dependency.target_field)
            .expect("dependency target belongs to its plan");
        target.dependencies.push(dependency.id.clone());
        target
            .cross_field_rules
            .push(dependency.dependent_rule.clone());
        target.source_fields.push(dependency.source_field.clone());
    }
}

#[allow(clippy::too_many_lines)]
#[must_use]
pub fn validate_validation_dependency_plans(
    products: &ValidationDependencyPlans,
    forms: &BTreeMap<FormId, FormEntity>,
    fields: &BTreeMap<FieldId, FormFieldEntity>,
    rules: &BTreeMap<ValidationRuleId, ValidationRule>,
    form_ownership: &FormOwnershipGraph,
    validation_graph: &ValidationGraph,
) -> ValidationDependencyPlanValidation {
    let mut diagnostics = Vec::new();
    let expected_plan_ids = forms
        .keys()
        .map(ValidationPlanId::for_form)
        .collect::<BTreeSet<_>>();
    let actual_plan_ids = products.plans.keys().cloned().collect::<BTreeSet<_>>();
    if expected_plan_ids != actual_plan_ids {
        push_integrity(
            &mut diagnostics,
            ValidationDependencyPlanIntegrityKind::DuplicatePlan,
            "Validation Plans do not contain exactly one canonical plan per Form",
        );
    }

    for plan in products.plans.values() {
        if plan.id != ValidationPlanId::for_form(&plan.form) {
            push_integrity(
                &mut diagnostics,
                ValidationDependencyPlanIntegrityKind::PlanIdentityDrift,
                "Validation Plan identity does not derive from its Form",
            );
        }
        let Some(form) = forms.get(&plan.form) else {
            push_integrity(
                &mut diagnostics,
                ValidationDependencyPlanIntegrityKind::UnknownForm,
                "Validation Plan references an unknown Form",
            );
            continue;
        };
        if form.owner.entity_id() != Some(&plan.component) {
            push_integrity(
                &mut diagnostics,
                ValidationDependencyPlanIntegrityKind::ComponentMismatch,
                "Validation Plan component does not match its Form owner",
            );
        }
        let expected_fields = ordered_fields_for_form(fields, &plan.form)
            .into_iter()
            .map(|field| field.id.clone())
            .collect::<Vec<_>>();
        if plan.fields != expected_fields {
            push_integrity(
                &mut diagnostics,
                ValidationDependencyPlanIntegrityKind::NonCanonicalOrdering,
                "Validation Plan Fields do not preserve I3 authored Field order",
            );
        }
        if plan.source_entries.len() != expected_fields.len()
            || plan.target_entries.len() != expected_fields.len()
        {
            push_integrity(
                &mut diagnostics,
                ValidationDependencyPlanIntegrityKind::SourceIndexMismatch,
                "Validation Plan must retain one source and target entry per Field",
            );
        }
        for field in &expected_fields {
            if !plan
                .source_entries
                .iter()
                .any(|entry| &entry.source_field == field)
            {
                push_integrity(
                    &mut diagnostics,
                    ValidationDependencyPlanIntegrityKind::SourceIndexMismatch,
                    "Validation Plan is missing a source Field entry",
                );
            }
            if !plan
                .target_entries
                .iter()
                .any(|entry| &entry.target_field == field)
            {
                push_integrity(
                    &mut diagnostics,
                    ValidationDependencyPlanIntegrityKind::TargetIndexMismatch,
                    "Validation Plan is missing a target Field entry",
                );
            }
        }
    }

    let mut pairs = BTreeSet::new();
    for dependency in products.dependencies.values() {
        if !pairs.insert((
            dependency.dependent_rule.clone(),
            dependency.source_field.clone(),
        )) {
            push_integrity(
                &mut diagnostics,
                ValidationDependencyPlanIntegrityKind::DuplicateDependencyProjection,
                "multiple I7 dependencies share one Rule/source Field tuple",
            );
        }
        validate_dependency(
            dependency,
            products,
            forms,
            fields,
            rules,
            form_ownership,
            validation_graph,
            &mut diagnostics,
        );
    }

    for rule in rules.values() {
        let Some(source) = rule.dependency.as_ref() else {
            continue;
        };
        let exact_edges = validation_graph
            .edges
            .iter()
            .filter(|edge| {
                edge.kind == ValidationGraphEdgeKind::RuleDependsOnField
                    && edge.source == ValidationGraphNodeKey::ValidationRule(rule.id.clone())
                    && edge.target == ValidationGraphNodeKey::FormField(source.clone())
            })
            .count();
        if exact_edges == 0 {
            push_integrity(
                &mut diagnostics,
                ValidationDependencyPlanIntegrityKind::MissingI6Dependency,
                "valid I6 cross-Field Rule has no exact RuleDependsOnField edge",
            );
        } else if exact_edges > 1 {
            push_integrity(
                &mut diagnostics,
                ValidationDependencyPlanIntegrityKind::DuplicateDependencyProjection,
                "I6 cross-Field Rule has multiple exact dependency edges",
            );
        } else if !products
            .dependencies
            .contains_key(&FieldDependencyId::for_rule_and_source(&rule.id, source))
        {
            push_integrity(
                &mut diagnostics,
                ValidationDependencyPlanIntegrityKind::MissingDependencyProjection,
                "valid I6 cross-Field edge is absent from I7 planning",
            );
        }
    }
    for edge in validation_graph
        .edges
        .iter()
        .filter(|edge| edge.kind == ValidationGraphEdgeKind::RuleDependsOnField)
    {
        let (
            ValidationGraphNodeKey::ValidationRule(rule),
            ValidationGraphNodeKey::FormField(source),
        ) = (&edge.source, &edge.target)
        else {
            push_integrity(
                &mut diagnostics,
                ValidationDependencyPlanIntegrityKind::RuleNotInValidationGraph,
                "I6 dependency edge does not have Rule-to-Field endpoints",
            );
            continue;
        };
        if let Some(rule_record) = rules.get(rule) {
            if rule_record.dependency.as_ref() != Some(source) {
                push_integrity(
                    &mut diagnostics,
                    ValidationDependencyPlanIntegrityKind::RuleTargetMismatch,
                    "I6 dependency edge does not match its Rule dependency Field",
                );
            }
        } else {
            push_integrity(
                &mut diagnostics,
                ValidationDependencyPlanIntegrityKind::UnknownRule,
                "I6 dependency edge references an unknown Rule",
            );
        }
    }

    validate_entry_reciprocity(products, &mut diagnostics);
    validate_product_ordering(products, &mut diagnostics);
    diagnostics
        .sort_by(|left, right| (&left.code, &left.message).cmp(&(&right.code, &right.message)));
    ValidationDependencyPlanValidation {
        is_valid: diagnostics.is_empty(),
        diagnostics,
    }
}

#[allow(clippy::too_many_arguments)]
fn validate_dependency(
    dependency: &FieldValidationDependency,
    products: &ValidationDependencyPlans,
    forms: &BTreeMap<FormId, FormEntity>,
    fields: &BTreeMap<FieldId, FormFieldEntity>,
    rules: &BTreeMap<ValidationRuleId, ValidationRule>,
    form_ownership: &FormOwnershipGraph,
    validation_graph: &ValidationGraph,
    diagnostics: &mut Vec<ValidationDependencyPlanIntegrityDiagnostic>,
) {
    let Some(plan) = products.plans.get(&dependency.plan) else {
        push_integrity(
            diagnostics,
            ValidationDependencyPlanIntegrityKind::MissingPlanForm,
            "Field dependency references a missing Validation Plan",
        );
        return;
    };
    let Some(source) = fields.get(&dependency.source_field) else {
        push_integrity(
            diagnostics,
            ValidationDependencyPlanIntegrityKind::UnknownSourceField,
            "Field dependency references an unknown source Field",
        );
        return;
    };
    let Some(target) = fields.get(&dependency.target_field) else {
        push_integrity(
            diagnostics,
            ValidationDependencyPlanIntegrityKind::UnknownTargetField,
            "Field dependency references an unknown target Field",
        );
        return;
    };
    let Some(rule) = rules.get(&dependency.dependent_rule) else {
        push_integrity(
            diagnostics,
            ValidationDependencyPlanIntegrityKind::UnknownRule,
            "Field dependency references an unknown Validation Rule",
        );
        return;
    };
    validate_dependency_scope(dependency, plan, source, target, rule, diagnostics);
    validate_dependency_backing(
        dependency,
        source,
        rule,
        forms,
        form_ownership,
        validation_graph,
        diagnostics,
    );
    validate_dependency_provenance(dependency, diagnostics);
}

fn validate_dependency_scope(
    dependency: &FieldValidationDependency,
    plan: &FormValidationDependencyPlan,
    source: &FormFieldEntity,
    target: &FormFieldEntity,
    rule: &ValidationRule,
    diagnostics: &mut Vec<ValidationDependencyPlanIntegrityDiagnostic>,
) {
    if dependency.id != FieldDependencyId::for_rule_and_source(&rule.id, &source.id) {
        push_integrity(
            diagnostics,
            ValidationDependencyPlanIntegrityKind::PlanIdentityDrift,
            "Field dependency identity does not derive from Rule and source Field",
        );
    }
    if dependency.form != plan.form
        || dependency.form != source.owner_form
        || dependency.form != target.owner_form
    {
        push_integrity(
            diagnostics,
            ValidationDependencyPlanIntegrityKind::PlanFormMismatch,
            "Field dependency source, target, and plan do not share one Form",
        );
    }
    if dependency.form != rule.owner_form {
        push_integrity(
            diagnostics,
            ValidationDependencyPlanIntegrityKind::RuleFormMismatch,
            "Field dependency Rule does not belong to its Form",
        );
    }
    if dependency.component != source.owner_component
        || dependency.component != target.owner_component
        || dependency.component != rule.owner_component
    {
        push_integrity(
            diagnostics,
            ValidationDependencyPlanIntegrityKind::ComponentMismatch,
            "Field dependency does not share one Component owner",
        );
    }
    if source.id == target.id {
        push_integrity(
            diagnostics,
            ValidationDependencyPlanIntegrityKind::SelfDependency,
            "Field dependency cannot read its target Field",
        );
    }
    if dependency.target_field != rule.target_field {
        push_integrity(
            diagnostics,
            ValidationDependencyPlanIntegrityKind::RuleTargetMismatch,
            "Field dependency target does not match its Rule target",
        );
    }
    if rule.dependency.as_ref() != Some(&dependency.source_field) {
        push_integrity(
            diagnostics,
            ValidationDependencyPlanIntegrityKind::MissingI6Dependency,
            "Field dependency does not match its I6 Rule dependency",
        );
    }
    if dependency.execution_boundary != ExecutionBoundary::Client
        || rule.boundary != ExecutionBoundary::Client
    {
        push_integrity(
            diagnostics,
            ValidationDependencyPlanIntegrityKind::UnsupportedBoundary,
            "I7 plans accept only Client-boundary Rules",
        );
    }
}

fn validate_dependency_backing(
    dependency: &FieldValidationDependency,
    source: &FormFieldEntity,
    rule: &ValidationRule,
    forms: &BTreeMap<FormId, FormEntity>,
    form_ownership: &FormOwnershipGraph,
    validation_graph: &ValidationGraph,
    diagnostics: &mut Vec<ValidationDependencyPlanIntegrityDiagnostic>,
) {
    if validation_graph
        .cycles
        .iter()
        .any(|cycle| cycle.candidates.contains(&rule.candidate_id))
    {
        push_integrity(
            diagnostics,
            ValidationDependencyPlanIntegrityKind::CycleRulePromoted,
            "cycle-excluded I6 Rule entered I7 planning",
        );
    }
    if !validation_graph
        .nodes
        .contains_key(&ValidationGraphNodeKey::ValidationRule(rule.id.clone()))
        || matching_i6_edges(validation_graph, rule, source).len() != 1
    {
        push_integrity(
            diagnostics,
            ValidationDependencyPlanIntegrityKind::RuleNotInValidationGraph,
            "I7 dependency is not backed by an exact I6 Validation Graph edge",
        );
    }
    let owns_field = |field: &FieldId| {
        form_ownership.ownership_edges.iter().any(|edge| {
            edge.owner == FormOwnershipNodeKey::Form(dependency.form.clone())
                && edge.child == FormOwnershipNodeKey::FormField(field.clone())
        })
    };
    if !owns_field(&dependency.source_field) || !owns_field(&dependency.target_field) {
        push_integrity(
            diagnostics,
            ValidationDependencyPlanIntegrityKind::FieldFormMismatch,
            "I7 dependency Fields are not canonical I5 children of its Form",
        );
    }
    if !forms.contains_key(&dependency.form) {
        push_integrity(
            diagnostics,
            ValidationDependencyPlanIntegrityKind::UnknownForm,
            "I7 dependency references an unknown Form",
        );
    }
}

fn validate_dependency_provenance(
    dependency: &FieldValidationDependency,
    diagnostics: &mut Vec<ValidationDependencyPlanIntegrityDiagnostic>,
) {
    if !has_provenance(&dependency.provenance)
        || !has_provenance(&dependency.source_field_provenance)
        || !has_provenance(&dependency.target_field_provenance)
    {
        push_integrity(
            diagnostics,
            ValidationDependencyPlanIntegrityKind::MissingProvenance,
            "I7 dependency lacks canonical source provenance",
        );
    }
    if dependency.id.as_str().contains("form-instance")
        || dependency.plan.as_str().contains("form-instance")
        || dependency.form.as_str().contains("form-instance")
    {
        push_integrity(
            diagnostics,
            ValidationDependencyPlanIntegrityKind::InstanceIdentityLeak,
            "I7 declaration planning contains an instance-qualified identity",
        );
    }
}

fn validate_entry_reciprocity(
    products: &ValidationDependencyPlans,
    diagnostics: &mut Vec<ValidationDependencyPlanIntegrityDiagnostic>,
) {
    for plan in products.plans.values() {
        for source in &plan.source_entries {
            let expected = products
                .dependencies
                .values()
                .filter(|dependency| {
                    dependency.plan == plan.id && dependency.source_field == source.source_field
                })
                .collect::<Vec<_>>();
            let expected_ids = expected
                .iter()
                .map(|dependency| dependency.id.clone())
                .collect::<BTreeSet<_>>();
            let actual_ids = source.dependencies.iter().cloned().collect::<BTreeSet<_>>();
            if expected_ids != actual_ids {
                push_integrity(
                    diagnostics,
                    ValidationDependencyPlanIntegrityKind::InvalidationIndexMismatch,
                    "source entry does not contain exactly its direct I7 dependencies",
                );
            }
            let expected_rules = expected
                .iter()
                .map(|dependency| dependency.dependent_rule.clone())
                .collect::<BTreeSet<_>>();
            let actual_rules = source
                .directly_invalidated_rules
                .iter()
                .cloned()
                .collect::<BTreeSet<_>>();
            if expected_rules != actual_rules {
                push_integrity(
                    diagnostics,
                    ValidationDependencyPlanIntegrityKind::TransitiveInvalidationLeak,
                    "source entry contains non-direct invalidated Rules",
                );
            }
        }
        for target in &plan.target_entries {
            let expected = products
                .dependencies
                .values()
                .filter(|dependency| {
                    dependency.plan == plan.id && dependency.target_field == target.target_field
                })
                .map(|dependency| dependency.id.clone())
                .collect::<BTreeSet<_>>();
            let actual = target.dependencies.iter().cloned().collect::<BTreeSet<_>>();
            if expected != actual {
                push_integrity(
                    diagnostics,
                    ValidationDependencyPlanIntegrityKind::TargetIndexMismatch,
                    "target entry does not contain exactly its direct I7 dependencies",
                );
            }
        }
    }
    for dependency in products.dependencies.values() {
        let Some(plan) = products.plans.get(&dependency.plan) else {
            continue;
        };
        let source = plan
            .source_entries
            .iter()
            .find(|entry| entry.source_field == dependency.source_field);
        if !source.is_some_and(|entry| {
            entry.dependencies.contains(&dependency.id)
                && entry
                    .directly_invalidated_rules
                    .contains(&dependency.dependent_rule)
                && entry
                    .directly_invalidated_target_fields
                    .contains(&dependency.target_field)
        }) {
            push_integrity(
                diagnostics,
                ValidationDependencyPlanIntegrityKind::SourceIndexMismatch,
                "I7 source index omits its dependency projection",
            );
        }
        let target = plan
            .target_entries
            .iter()
            .find(|entry| entry.target_field == dependency.target_field);
        if !target.is_some_and(|entry| {
            entry.dependencies.contains(&dependency.id)
                && entry.cross_field_rules.contains(&dependency.dependent_rule)
                && entry.source_fields.contains(&dependency.source_field)
        }) {
            push_integrity(
                diagnostics,
                ValidationDependencyPlanIntegrityKind::TargetIndexMismatch,
                "I7 target index omits its dependency projection",
            );
        }
    }
}

fn validate_product_ordering(
    products: &ValidationDependencyPlans,
    diagnostics: &mut Vec<ValidationDependencyPlanIntegrityDiagnostic>,
) {
    for plan in products.plans.values() {
        let orders = plan_field_orders(plan);
        let mut ordered_dependencies = plan.dependencies.clone();
        ordered_dependencies.sort_by(|left, right| {
            dependency_order(
                products
                    .dependencies
                    .get(left)
                    .expect("plan dependency exists"),
                products
                    .dependencies
                    .get(right)
                    .expect("plan dependency exists"),
            )
        });
        if plan.dependencies != ordered_dependencies {
            push_integrity(
                diagnostics,
                ValidationDependencyPlanIntegrityKind::NonCanonicalOrdering,
                "I7 plan dependencies are not canonically ordered",
            );
        }
        for source in &plan.source_entries {
            if has_duplicates(&source.directly_invalidated_rules) {
                push_integrity(
                    diagnostics,
                    ValidationDependencyPlanIntegrityKind::DuplicateScheduledRule,
                    "source entry schedules one Rule more than once",
                );
            }
            if has_duplicates(&source.directly_invalidated_target_fields) {
                push_integrity(
                    diagnostics,
                    ValidationDependencyPlanIntegrityKind::DuplicateScheduledTarget,
                    "source entry schedules one target Field more than once",
                );
            }
            let mut rules = source.directly_invalidated_rules.clone();
            rules.sort_by(|left, right| {
                let left = products
                    .dependencies
                    .values()
                    .find(|dependency| {
                        &dependency.dependent_rule == left
                            && dependency.source_field == source.source_field
                    })
                    .expect("source Rule has dependency");
                let right = products
                    .dependencies
                    .values()
                    .find(|dependency| {
                        &dependency.dependent_rule == right
                            && dependency.source_field == source.source_field
                    })
                    .expect("source Rule has dependency");
                rule_schedule_order(left, right)
            });
            if source.directly_invalidated_rules != rules {
                push_integrity(
                    diagnostics,
                    ValidationDependencyPlanIntegrityKind::InvalidationIndexMismatch,
                    "source invalidation Rules are not in schedule order",
                );
            }
        }
        if plan
            .source_entries
            .iter()
            .map(|entry| entry.authored_field_order)
            .collect::<Vec<_>>()
            != plan
                .source_entries
                .iter()
                .map(|entry| orders[&entry.source_field])
                .collect::<Vec<_>>()
        {
            push_integrity(
                diagnostics,
                ValidationDependencyPlanIntegrityKind::NonCanonicalOrdering,
                "source entries do not preserve authored Field order",
            );
        }
    }
}

fn canonicalize_plans(
    plans: &mut BTreeMap<ValidationPlanId, FormValidationDependencyPlan>,
    dependencies: &BTreeMap<FieldDependencyId, FieldValidationDependency>,
) {
    for plan in plans.values_mut() {
        plan.fields.sort_by_key(|field| {
            plan.source_entries
                .iter()
                .find(|entry| entry.source_field == *field)
                .map_or(usize::MAX, |entry| entry.authored_field_order)
        });
        plan.dependencies.sort_by(|left, right| {
            dependency_order(
                dependencies.get(left).expect("plan dependency exists"),
                dependencies.get(right).expect("plan dependency exists"),
            )
        });
        let orders = plan_field_orders(plan);
        for source in &mut plan.source_entries {
            source.dependencies.sort_by(|left, right| {
                dependency_order(
                    dependencies.get(left).expect("source dependency exists"),
                    dependencies.get(right).expect("source dependency exists"),
                )
            });
            source.directly_invalidated_rules.sort_by(|left, right| {
                let left = dependencies
                    .values()
                    .find(|dependency| {
                        &dependency.dependent_rule == left
                            && dependency.source_field == source.source_field
                    })
                    .expect("source Rule has dependency");
                let right = dependencies
                    .values()
                    .find(|dependency| {
                        &dependency.dependent_rule == right
                            && dependency.source_field == source.source_field
                    })
                    .expect("source Rule has dependency");
                rule_schedule_order(left, right)
            });
            source.directly_invalidated_rules.dedup();
            source
                .directly_invalidated_target_fields
                .sort_by_key(|field| {
                    (
                        orders.get(field).copied().unwrap_or(usize::MAX),
                        field.clone(),
                    )
                });
            source.directly_invalidated_target_fields.dedup();
        }
        for target in &mut plan.target_entries {
            target.dependencies.sort_by(|left, right| {
                dependency_order(
                    dependencies.get(left).expect("target dependency exists"),
                    dependencies.get(right).expect("target dependency exists"),
                )
            });
            target.cross_field_rules.sort_by(|left, right| {
                let left = dependencies
                    .values()
                    .find(|dependency| {
                        &dependency.dependent_rule == left
                            && dependency.target_field == target.target_field
                    })
                    .expect("target Rule has dependency");
                let right = dependencies
                    .values()
                    .find(|dependency| {
                        &dependency.dependent_rule == right
                            && dependency.target_field == target.target_field
                    })
                    .expect("target Rule has dependency");
                rule_schedule_order(left, right)
            });
            target.cross_field_rules.dedup();
            target.source_fields.sort_by_key(|field| {
                (
                    orders.get(field).copied().unwrap_or(usize::MAX),
                    field.clone(),
                )
            });
            target.source_fields.dedup();
        }
        plan.source_entries
            .sort_by_key(|entry| (entry.authored_field_order, entry.source_field.clone()));
        plan.target_entries
            .sort_by_key(|entry| (entry.authored_field_order, entry.target_field.clone()));
    }
}

fn ordered_fields_for_form<'a>(
    fields: &'a BTreeMap<FieldId, FormFieldEntity>,
    form: &FormId,
) -> Vec<&'a FormFieldEntity> {
    let mut result = fields
        .values()
        .filter(|field| &field.owner_form == form)
        .collect::<Vec<_>>();
    result.sort_by(|left, right| field_order(left, right));
    result
}

fn field_order(left: &FormFieldEntity, right: &FormFieldEntity) -> std::cmp::Ordering {
    (left.declaration_order, &left.id).cmp(&(right.declaration_order, &right.id))
}

fn plan_field_orders(plan: &FormValidationDependencyPlan) -> BTreeMap<FieldId, usize> {
    plan.source_entries
        .iter()
        .map(|entry| (entry.source_field.clone(), entry.authored_field_order))
        .collect()
}

fn dependency_order(
    left: &FieldValidationDependency,
    right: &FieldValidationDependency,
) -> std::cmp::Ordering {
    (
        left.target_field_order,
        left.rule_order,
        left.source_field_order,
        &left.id,
    )
        .cmp(&(
            right.target_field_order,
            right.rule_order,
            right.source_field_order,
            &right.id,
        ))
}

fn dependency_sort_key(
    dependency: &FieldValidationDependency,
) -> (usize, usize, usize, FieldDependencyId) {
    (
        dependency.target_field_order,
        dependency.rule_order,
        dependency.source_field_order,
        dependency.id.clone(),
    )
}

fn rule_schedule_order(
    left: &FieldValidationDependency,
    right: &FieldValidationDependency,
) -> std::cmp::Ordering {
    (
        left.target_field_order,
        left.rule_order,
        &left.dependent_rule,
    )
        .cmp(&(
            right.target_field_order,
            right.rule_order,
            &right.dependent_rule,
        ))
}

fn rule_schedule_key(dependency: &FieldValidationDependency) -> (usize, usize, ValidationRuleId) {
    (
        dependency.target_field_order,
        dependency.rule_order,
        dependency.dependent_rule.clone(),
    )
}

fn blocked_for_rule(
    rule: &ValidationRule,
    source: Option<FieldId>,
    reason: FieldDependencyBlockReason,
    edge: Option<&&crate::ValidationGraphEdge>,
) -> BlockedFieldValidationDependency {
    BlockedFieldValidationDependency {
        rule: Some(rule.id.clone()),
        source_field: source,
        target_field: Some(rule.target_field.clone()),
        form: Some(rule.owner_form.clone()),
        reason,
        provenance: edge
            .map(|edge| edge.provenance.clone())
            .or_else(|| Some(rule.provenance.clone())),
    }
}

fn blocked_order(
    left: &BlockedFieldValidationDependency,
    right: &BlockedFieldValidationDependency,
) -> std::cmp::Ordering {
    (
        left.form.as_ref().map_or("", FormId::as_str),
        left.target_field.as_ref().map_or("", FieldId::as_str),
        left.rule.as_ref().map_or("", ValidationRuleId::as_str),
        left.source_field.as_ref().map_or("", FieldId::as_str),
        left.reason,
        left.provenance
            .as_ref()
            .map_or("", |provenance| provenance.path.to_str().unwrap_or("")),
        left.provenance
            .as_ref()
            .map_or(0, |provenance| provenance.span.start),
    )
        .cmp(&(
            right.form.as_ref().map_or("", FormId::as_str),
            right.target_field.as_ref().map_or("", FieldId::as_str),
            right.rule.as_ref().map_or("", ValidationRuleId::as_str),
            right.source_field.as_ref().map_or("", FieldId::as_str),
            right.reason,
            right
                .provenance
                .as_ref()
                .map_or("", |provenance| provenance.path.to_str().unwrap_or("")),
            right
                .provenance
                .as_ref()
                .map_or(0, |provenance| provenance.span.start),
        ))
}

fn has_provenance(provenance: &SourceProvenance) -> bool {
    !provenance.path.as_os_str().is_empty() && provenance.span.start <= provenance.span.end
}

fn has_duplicates<T: Ord + Clone>(values: &[T]) -> bool {
    values.iter().cloned().collect::<BTreeSet<_>>().len() != values.len()
}

fn push_integrity(
    diagnostics: &mut Vec<ValidationDependencyPlanIntegrityDiagnostic>,
    kind: ValidationDependencyPlanIntegrityKind,
    message: &str,
) {
    diagnostics.push(ValidationDependencyPlanIntegrityDiagnostic {
        code: kind.code().to_string(),
        kind,
        message: message.to_string(),
    });
}

#[cfg(test)]
mod tests {
    use super::{
        collect_validation_dependency_plans, validate_validation_dependency_plans,
        FieldDependencyBlockReason, ValidationDependencyPlanIntegrityKind,
        ValidationPlanningStatus,
    };
    use crate::{
        build_application_semantic_model, build_application_semantic_model_for_unit,
        build_semantic_graph, semantic_graph_json, validate_application_semantic_model,
        CompilationUnit, FormInstanceId, SemanticGraph, SEMANTIC_GRAPH_SCHEMA_VERSION,
    };

    fn build(source: &str) -> crate::ApplicationSemanticModel {
        build_application_semantic_model(&ezc_parser::parse_file("src/Profile.tsx", source))
    }

    #[test]
    fn plans_direct_cross_field_dependencies_without_unary_or_transitive_propagation() {
        let asm = build(
            r#"
@component("profile")
class Profile {
  @form() form!: Form;
  @field(this.form) c = "";
  @validate(required()) @validate(equals(this.c)) @field(this.form) b = "";
  @validate(equals(this.b)) @field(this.form) a = "";
  render() { return <div />; }
}
"#,
        );
        let form = asm.forms.keys().next().unwrap();
        let fields = &asm.form_fields;
        let c = fields
            .values()
            .find(|field| field.name == "c")
            .unwrap()
            .id
            .clone();
        let b = fields
            .values()
            .find(|field| field.name == "b")
            .unwrap()
            .id
            .clone();
        let a = fields
            .values()
            .find(|field| field.name == "a")
            .unwrap()
            .id
            .clone();
        let plan = asm
            .validation_dependency_plans
            .validation_plan(form)
            .unwrap();
        assert_eq!(plan.dependencies.len(), 2);
        assert!(plan
            .target_entries
            .iter()
            .find(|entry| entry.target_field == b)
            .is_some_and(|entry| entry.cross_field_rules.len() == 1));
        let c_schedule = asm
            .validation_dependency_plans
            .schedule_change_set(form, std::slice::from_ref(&c))
            .unwrap();
        assert_eq!(c_schedule.scheduled_rules.len(), 1);
        assert_eq!(c_schedule.scheduled_target_fields, vec![b.clone()]);
        let b_schedule = asm
            .validation_dependency_plans
            .schedule_change_set(form, std::slice::from_ref(&b))
            .unwrap();
        assert_eq!(b_schedule.scheduled_target_fields, vec![a]);
        assert_eq!(plan.initial_validation, ValidationPlanningStatus::Deferred);
        assert_eq!(
            plan.submission_validation,
            ValidationPlanningStatus::Deferred
        );
    }

    #[test]
    fn schedules_change_sets_by_target_order_deduplicates_rules_and_keeps_evidence() {
        let asm = build(
            r#"
@component("profile")
class Profile {
  @form() form!: Form;
  @field(this.form) first = "";
  @field(this.form) second = "";
  @validate(equals(this.second)) @field(this.form) alpha = "";
  @validate(equals(this.first)) @field(this.form) beta = "";
  render() { return <div />; }
}
"#,
        );
        let form = asm.forms.keys().next().unwrap();
        let first = asm
            .form_fields
            .values()
            .find(|field| field.name == "first")
            .unwrap()
            .id
            .clone();
        let second = asm
            .form_fields
            .values()
            .find(|field| field.name == "second")
            .unwrap()
            .id
            .clone();
        let alpha = asm
            .form_fields
            .values()
            .find(|field| field.name == "alpha")
            .unwrap()
            .id
            .clone();
        let beta = asm
            .form_fields
            .values()
            .find(|field| field.name == "beta")
            .unwrap()
            .id
            .clone();
        let schedule = asm
            .validation_dependency_plans
            .schedule_change_set(form, &[second.clone(), first.clone(), second])
            .unwrap();
        assert_eq!(
            schedule.changed_fields,
            vec![
                first,
                asm.form_fields
                    .values()
                    .find(|field| field.name == "second")
                    .unwrap()
                    .id
                    .clone()
            ]
        );
        assert_eq!(schedule.scheduled_target_fields, vec![alpha, beta]);
        assert_eq!(schedule.scheduled_rules.len(), 2);
        assert_eq!(schedule.triggering_dependencies.len(), 2);
    }

    #[test]
    fn creates_empty_plans_and_rejects_cross_form_change_sets() {
        let asm = build(
            r#"
@component("profile")
class Profile {
  @form() empty!: Form;
  @form() active!: Form;
  @field(this.active) value = "";
  render() { return <div />; }
}
"#,
        );
        assert_eq!(asm.validation_dependency_plans.plans.len(), 2);
        let empty = asm
            .forms
            .values()
            .find(|form| form.name == "empty")
            .unwrap();
        assert!(asm
            .validation_dependency_plans
            .validation_plan(&empty.id)
            .is_some_and(|plan| plan.fields.is_empty() && plan.dependencies.is_empty()));
        let active = asm
            .forms
            .values()
            .find(|form| form.name == "active")
            .unwrap();
        let field = asm.form_fields.values().next().unwrap();
        assert!(asm
            .validation_dependency_plans
            .schedule_change_set(&empty.id, std::slice::from_ref(&field.id))
            .is_none());
        assert!(asm
            .validation_dependency_plans
            .schedule_change_set(&active.id, &[])
            .is_some());
    }

    #[test]
    fn retains_blocked_records_and_detects_missing_projection_and_index_drift() {
        let asm = build(
            r#"
@component("profile")
class Profile {
  @form() form!: Form;
  @field(this.form) source = "";
  @validate(equals(this.source)) @field(this.form) target = "";
  render() { return <div />; }
}
"#,
        );
        let mut missing = asm.validation_dependency_plans.clone();
        let dependency = missing.dependencies.keys().next().unwrap().clone();
        missing.dependencies.remove(&dependency);
        missing
            .plans
            .values_mut()
            .next()
            .unwrap()
            .dependencies
            .clear();
        let validation = validate_validation_dependency_plans(
            &missing,
            &asm.forms,
            &asm.form_fields,
            &asm.validation_rules,
            &asm.form_ownership,
            &asm.validation_graph,
        );
        assert!(validation
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.kind
                == ValidationDependencyPlanIntegrityKind::MissingDependencyProjection));

        let mut graph = asm.validation_graph.clone();
        graph.edges.retain(|edge| {
            !matches!(
                edge.kind,
                crate::ValidationGraphEdgeKind::RuleDependsOnField
            )
        });
        let blocked = collect_validation_dependency_plans(
            &asm.forms,
            &asm.form_fields,
            &asm.validation_rules,
            &asm.form_ownership,
            &graph,
        );
        assert!(blocked
            .blocked
            .iter()
            .any(|blocked| blocked.reason == FieldDependencyBlockReason::IdentityMismatch));

        let mut stale = asm.clone();
        stale.validation_dependency_plans.dependencies.clear();
        stale
            .validation_dependency_plans
            .plans
            .values_mut()
            .next()
            .unwrap()
            .dependencies
            .clear();
        assert!(validate_application_semantic_model(&stale)
            .iter()
            .any(|diagnostic| diagnostic.code == "EZASM1272"));
    }

    #[test]
    fn remains_declaration_only_deterministic_and_hidden_from_public_schema() {
        let first = ezc_parser::parse_file(
            "src/A.tsx",
            r#"@component("a-x") class A { @form() form!: Form; @field(this.form) left = ""; @validate(equals(this.left)) @field(this.form) right = ""; render() { return <div />; } }"#,
        );
        let second = ezc_parser::parse_file(
            "src/B.tsx",
            r#"@component("b-x") class B { @form() form!: Form; render() { return <div />; } }"#,
        );
        let forward =
            build_application_semantic_model_for_unit(&CompilationUnit::from_parsed_files(vec![
                first.clone(),
                second.clone(),
            ]));
        let reversed =
            build_application_semantic_model_for_unit(&CompilationUnit::from_parsed_files(vec![
                second, first,
            ]));
        assert_eq!(
            forward.validation_dependency_plans,
            reversed.validation_dependency_plans
        );
        assert!(forward
            .validation_dependency_plans
            .dependencies
            .values()
            .all(|dependency| !dependency.id.as_str().contains("form-instance")));
        let form = forward.forms.values().next().unwrap();
        let instance = FormInstanceId::for_component_instance(
            &forward
                .component_instance_plan
                .instances
                .values()
                .next()
                .unwrap()
                .id,
            &form.id,
        );
        assert!(!forward
            .validation_dependency_plans
            .plans
            .keys()
            .any(|plan| plan.as_str() == instance.as_str()));
        assert_eq!(SEMANTIC_GRAPH_SCHEMA_VERSION, 6);
        let graph: SemanticGraph = build_semantic_graph(&forward);
        let json = semantic_graph_json(&graph);
        assert!(!json.contains("validation-plan"));
        assert!(!json.contains("field-dependency"));
    }
}
