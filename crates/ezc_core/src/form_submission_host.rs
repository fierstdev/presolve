//! I15-S canonical explicit template submission-host binding.
//!
//! A host is never inferred from DOM structure.  It is the one explicit
//! compiler-only `form={this.<Form>}` attribute on an intrinsic `<form>`.

use std::collections::{BTreeMap, BTreeSet};

use crate::{
    ElementNode, FieldBindingId, FormEntity, FormFieldBinding, FormId, FormIrReport,
    FormSerializationPlan, FormSubmissionPlan, RenderAttribute, RenderAttributeValue, SemanticId,
    SerializationPlanId, SourceProvenance, SubmissionHostCandidateId, SubmissionHostId,
    SubmissionPlanId, TemplateChild, TemplateNode,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum SubmissionHostViolation {
    InvalidHostElement,
    DuplicateHostAttribute,
    InvalidHostExpression,
    UnresolvedForm,
    CrossComponentForm,
    MissingSubmissionPlan,
    MissingSerializationPlan,
    MissingFormIr,
    DuplicateFormHost,
    NestedHost,
    ContainedControlForDifferentForm,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SubmissionHostCandidate {
    pub id: SubmissionHostCandidateId,
    pub host_id: Option<SubmissionHostId>,
    pub owner_component: Option<SemanticId>,
    pub owner_template: Option<SemanticId>,
    /// The exact template `<form>` element which owns this use site.
    pub owner_template_element: Option<SemanticId>,
    pub template_path: String,
    pub resolved_form: Option<FormId>,
    pub submission_plan: Option<SubmissionPlanId>,
    pub submit_action: Option<SemanticId>,
    pub action_batch: Option<SemanticId>,
    pub serialization_plan: Option<SerializationPlanId>,
    pub provenance: SourceProvenance,
    pub attribute_provenance: SourceProvenance,
    pub violations: Vec<SubmissionHostViolation>,
}

impl SubmissionHostCandidate {
    #[must_use]
    pub fn is_valid(&self) -> bool {
        self.violations.is_empty()
    }

    fn add(&mut self, violation: SubmissionHostViolation) {
        if !self.violations.contains(&violation) {
            self.violations.push(violation);
            self.violations.sort();
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SubmissionHost {
    pub id: SubmissionHostId,
    pub owner_template: SemanticId,
    pub owner_template_element: SemanticId,
    pub component: SemanticId,
    pub form: FormId,
    pub submission_plan: SubmissionPlanId,
    pub submit_action: SemanticId,
    pub action_batch: SemanticId,
    pub serialization_plan: SerializationPlanId,
    pub event: &'static str,
    pub prevent_default: bool,
    pub provenance: SourceProvenance,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct SubmissionHostProducts {
    pub candidates: Vec<SubmissionHostCandidate>,
    pub hosts: BTreeMap<SubmissionHostId, SubmissionHost>,
}

#[must_use]
#[allow(clippy::too_many_arguments, clippy::too_many_lines)]
/// # Panics
///
/// Panics only when a candidate retained as valid is missing a required
/// compiler product, which indicates a staged lowering invariant violation.
pub fn collect_submission_host_products(
    templates: &[TemplateNode],
    components: &[crate::ComponentNode],
    forms: &BTreeMap<FormId, FormEntity>,
    bindings: &BTreeMap<FieldBindingId, FormFieldBinding>,
    submissions: &BTreeMap<SubmissionPlanId, FormSubmissionPlan>,
    serialization: &BTreeMap<SerializationPlanId, FormSerializationPlan>,
    ir: &FormIrReport,
) -> SubmissionHostProducts {
    let components = components
        .iter()
        .map(|component| (component.id.clone(), component))
        .collect::<BTreeMap<_, _>>();
    let mut candidates = Vec::new();
    for template in templates {
        let component = template
            .owner
            .entity_id()
            .and_then(|id| components.get(id).copied());
        if let Some(root) = &template.root {
            collect_element(
                root,
                template,
                "root",
                component,
                forms,
                bindings,
                submissions,
                serialization,
                ir,
                &mut candidates,
            );
        }
        if let Some(fragment) = &template.root_fragment {
            collect_children(
                &fragment.children,
                template,
                "root",
                component,
                forms,
                bindings,
                submissions,
                serialization,
                ir,
                &mut candidates,
            );
        }
    }
    candidates.sort_by_key(|candidate| {
        (
            candidate.provenance.path.clone(),
            candidate.attribute_provenance.span.start,
        )
    });

    let positions = candidates
        .iter()
        .enumerate()
        .map(|(index, candidate)| {
            (
                (
                    candidate.owner_template.clone(),
                    candidate.template_path.clone(),
                ),
                index,
            )
        })
        .collect::<BTreeMap<_, _>>();
    let mut nested = Vec::new();
    for (index, candidate) in candidates.iter().enumerate() {
        for ancestor in ancestor_paths(&candidate.template_path) {
            if let Some(parent) = positions.get(&(candidate.owner_template.clone(), ancestor)) {
                nested.push((index, *parent));
            }
        }
    }
    for (child, parent) in nested {
        candidates[child].add(SubmissionHostViolation::NestedHost);
        candidates[parent].add(SubmissionHostViolation::NestedHost);
    }
    let mut by_form = BTreeMap::<FormId, Vec<usize>>::new();
    for (index, candidate) in candidates
        .iter()
        .enumerate()
        .filter(|(_, candidate)| candidate.is_valid())
    {
        if let Some(form) = &candidate.resolved_form {
            by_form.entry(form.clone()).or_default().push(index);
        }
    }
    for indexes in by_form.values().filter(|indexes| indexes.len() > 1) {
        for index in indexes {
            candidates[*index].add(SubmissionHostViolation::DuplicateFormHost);
        }
    }
    let hosts = candidates
        .iter_mut()
        .filter(|candidate| candidate.is_valid())
        .map(|candidate| {
            let id = SubmissionHostId::for_element(
                candidate
                    .owner_template_element
                    .as_ref()
                    .expect("valid host element"),
                candidate.resolved_form.as_ref().expect("valid host form"),
            );
            candidate.host_id = Some(id.clone());
            (
                id.clone(),
                SubmissionHost {
                    id,
                    owner_template: candidate
                        .owner_template
                        .clone()
                        .expect("valid host template"),
                    owner_template_element: candidate
                        .owner_template_element
                        .clone()
                        .expect("valid host element"),
                    component: candidate
                        .owner_component
                        .clone()
                        .expect("valid host component"),
                    form: candidate.resolved_form.clone().expect("valid host form"),
                    submission_plan: candidate
                        .submission_plan
                        .clone()
                        .expect("valid host submission"),
                    submit_action: candidate.submit_action.clone().expect("valid host action"),
                    action_batch: candidate.action_batch.clone().expect("valid host batch"),
                    serialization_plan: candidate
                        .serialization_plan
                        .clone()
                        .expect("valid host serialization"),
                    event: "submit",
                    prevent_default: true,
                    provenance: candidate.provenance.clone(),
                },
            )
        })
        .collect();
    SubmissionHostProducts { candidates, hosts }
}

#[allow(clippy::too_many_arguments)]
fn collect_element(
    element: &ElementNode,
    template: &TemplateNode,
    path: &str,
    component: Option<&crate::ComponentNode>,
    forms: &BTreeMap<FormId, FormEntity>,
    bindings: &BTreeMap<FieldBindingId, FormFieldBinding>,
    submissions: &BTreeMap<SubmissionPlanId, FormSubmissionPlan>,
    serialization: &BTreeMap<SerializationPlanId, FormSerializationPlan>,
    ir: &FormIrReport,
    candidates: &mut Vec<SubmissionHostCandidate>,
) {
    let element_id = template.id.template_entity("element", path);
    let attributes = element
        .authored_attributes
        .iter()
        .filter(|attribute| attribute.name == "form")
        .collect::<Vec<_>>();
    for attribute in &attributes {
        let mut candidate =
            candidate_for(element, template, path, &element_id, component, attribute);
        if attributes.len() > 1 {
            candidate.add(SubmissionHostViolation::DuplicateHostAttribute);
        }
        if element.tag_name != "form" {
            candidate.add(SubmissionHostViolation::InvalidHostElement);
        }
        resolve(&mut candidate, forms, submissions, serialization, ir);
        if candidate.is_valid()
            && contains_binding_for_other_form(
                element,
                template,
                path,
                &candidate.resolved_form.clone().expect("valid host form"),
                bindings,
            )
        {
            candidate.add(SubmissionHostViolation::ContainedControlForDifferentForm);
        }
        candidates.push(candidate);
    }
    collect_children(
        &element.children,
        template,
        path,
        component,
        forms,
        bindings,
        submissions,
        serialization,
        ir,
        candidates,
    );
}

#[allow(clippy::too_many_arguments)]
fn collect_children(
    children: &[TemplateChild],
    template: &TemplateNode,
    path: &str,
    component: Option<&crate::ComponentNode>,
    forms: &BTreeMap<FormId, FormEntity>,
    bindings: &BTreeMap<FieldBindingId, FormFieldBinding>,
    submissions: &BTreeMap<SubmissionPlanId, FormSubmissionPlan>,
    serialization: &BTreeMap<SerializationPlanId, FormSerializationPlan>,
    ir: &FormIrReport,
    candidates: &mut Vec<SubmissionHostCandidate>,
) {
    for (index, child) in children.iter().enumerate() {
        let path = format!("{path}.{index}");
        match child {
            TemplateChild::Element(element) => collect_element(
                element,
                template,
                &path,
                component,
                forms,
                bindings,
                submissions,
                serialization,
                ir,
                candidates,
            ),
            TemplateChild::Fragment(fragment) => collect_children(
                &fragment.children,
                template,
                &path,
                component,
                forms,
                bindings,
                submissions,
                serialization,
                ir,
                candidates,
            ),
            TemplateChild::Conditional(conditional) => {
                collect_children(
                    &conditional.when_true,
                    template,
                    &format!("{path}.true"),
                    component,
                    forms,
                    bindings,
                    submissions,
                    serialization,
                    ir,
                    candidates,
                );
                collect_children(
                    &conditional.when_false,
                    template,
                    &format!("{path}.false"),
                    component,
                    forms,
                    bindings,
                    submissions,
                    serialization,
                    ir,
                    candidates,
                );
            }
            TemplateChild::List(list) => collect_children(
                &list.item_template,
                template,
                &format!("{path}.item"),
                component,
                forms,
                bindings,
                submissions,
                serialization,
                ir,
                candidates,
            ),
            TemplateChild::Text { .. } | TemplateChild::Binding { .. } => {}
        }
    }
}

fn candidate_for(
    element: &ElementNode,
    template: &TemplateNode,
    template_path: &str,
    element_id: &SemanticId,
    component: Option<&crate::ComponentNode>,
    attribute: &RenderAttribute,
) -> SubmissionHostCandidate {
    let path = &template.provenance.path;
    let name = match &attribute.value {
        RenderAttributeValue::Expression(_) => attribute
            .this_member
            .as_ref()
            .map(|member| member.member.clone()),
        _ => None,
    };
    let mut candidate = SubmissionHostCandidate {
        id: SubmissionHostCandidateId::for_source_position(path, attribute.span.start),
        host_id: None,
        owner_component: component.map(|component| component.id.clone()),
        owner_template: Some(template.id.clone()),
        owner_template_element: Some(element_id.clone()),
        template_path: template_path.to_string(),
        resolved_form: None,
        submission_plan: None,
        submit_action: None,
        action_batch: None,
        serialization_plan: None,
        provenance: SourceProvenance::new(path, element.span),
        attribute_provenance: SourceProvenance::new(path, attribute.span),
        violations: Vec::new(),
    };
    let Some(name) = name else {
        candidate.add(SubmissionHostViolation::InvalidHostExpression);
        return candidate;
    };
    candidate.resolved_form = Some(FormId::for_owner(
        candidate
            .owner_component
            .as_ref()
            .expect("template component"),
        &name,
    ));
    candidate
}

fn resolve(
    candidate: &mut SubmissionHostCandidate,
    forms: &BTreeMap<FormId, FormEntity>,
    submissions: &BTreeMap<SubmissionPlanId, FormSubmissionPlan>,
    serialization: &BTreeMap<SerializationPlanId, FormSerializationPlan>,
    ir: &FormIrReport,
) {
    let Some(form) = candidate.resolved_form.clone() else {
        return;
    };
    if !forms.contains_key(&form) {
        candidate.add(SubmissionHostViolation::UnresolvedForm);
        return;
    }
    let submission = SubmissionPlanId::for_form(&form);
    if let Some(plan) = submissions.get(&submission) {
        candidate.submission_plan = Some(submission);
        candidate.submit_action = Some(plan.action_method.clone());
        candidate.action_batch = Some(plan.action_batch.clone());
    } else {
        candidate.add(SubmissionHostViolation::MissingSubmissionPlan);
    }
    let serialization_plan = SerializationPlanId::for_form(&form);
    if serialization.contains_key(&serialization_plan) {
        candidate.serialization_plan = Some(serialization_plan);
    } else {
        candidate.add(SubmissionHostViolation::MissingSerializationPlan);
    }
    if !ir.instances.values().any(|instance| instance.form == form) {
        candidate.add(SubmissionHostViolation::MissingFormIr);
    }
}

fn contains_binding_for_other_form(
    element: &ElementNode,
    template: &TemplateNode,
    path: &str,
    form: &FormId,
    bindings: &BTreeMap<FieldBindingId, FormFieldBinding>,
) -> bool {
    let element_id = template.id.template_entity("element", path);
    if bindings
        .values()
        .any(|binding| binding.control_entity == element_id && binding.form != *form)
    {
        return true;
    }
    element
        .children
        .iter()
        .enumerate()
        .any(|(index, child)| match child {
            TemplateChild::Element(child) => contains_binding_for_other_form(
                child,
                template,
                &format!("{path}.{index}"),
                form,
                bindings,
            ),
            TemplateChild::Fragment(fragment) => {
                fragment.children.iter().enumerate().any(|(nested, child)| {
                    contained_child_other_form(
                        child,
                        template,
                        &format!("{path}.{index}.{nested}"),
                        form,
                        bindings,
                    )
                })
            }
            _ => false,
        })
}
fn contained_child_other_form(
    child: &TemplateChild,
    template: &TemplateNode,
    path: &str,
    form: &FormId,
    bindings: &BTreeMap<FieldBindingId, FormFieldBinding>,
) -> bool {
    match child {
        TemplateChild::Element(element) => {
            contains_binding_for_other_form(element, template, path, form, bindings)
        }
        TemplateChild::Fragment(fragment) => {
            fragment.children.iter().enumerate().any(|(index, child)| {
                contained_child_other_form(
                    child,
                    template,
                    &format!("{path}.{index}"),
                    form,
                    bindings,
                )
            })
        }
        TemplateChild::Conditional(conditional) => conditional
            .when_true
            .iter()
            .chain(&conditional.when_false)
            .enumerate()
            .any(|(index, child)| {
                contained_child_other_form(
                    child,
                    template,
                    &format!("{path}.{index}"),
                    form,
                    bindings,
                )
            }),
        TemplateChild::List(list) => list.item_template.iter().enumerate().any(|(index, child)| {
            contained_child_other_form(child, template, &format!("{path}.{index}"), form, bindings)
        }),
        TemplateChild::Text { .. } | TemplateChild::Binding { .. } => false,
    }
}
fn ancestor_paths(path: &str) -> BTreeSet<String> {
    let mut result = BTreeSet::new();
    let mut parts = path.split('.').collect::<Vec<_>>();
    while parts.len() > 1 {
        parts.pop();
        result.insert(parts.join("."));
    }
    result
}

#[cfg(test)]
mod tests {
    use crate::{
        build_application_semantic_model, build_runtime_forms_artifact,
        build_template_manifest_from_asm, generate_static_html, SubmissionHostViolation,
    };

    const VALID: &str = r#"
@component("profile") class Profile {
  @form() @serialize("json") profile!: Form;
  @field(this.profile) name = "";
  @action() @submit(this.profile) save(): void {}
  render() { return <form form={this.profile}><input field={this.name} /></form>; }
}"#;

    #[test]
    fn projects_one_explicit_host_through_manifest_artifact_and_html_anchor() {
        let model =
            build_application_semantic_model(&ezc_parser::parse_file("src/Profile.tsx", VALID));
        assert_eq!(
            model.submission_hosts.len(),
            1,
            "{:?}",
            model.submission_host_candidates
        );
        assert!(model.submission_host_candidates[0].is_valid());
        let manifest = build_template_manifest_from_asm(&model);
        assert_eq!(manifest.form_hosts.len(), 1);
        assert_eq!(manifest.form_hosts[0].event, "submit");
        assert!(manifest.form_hosts[0].prevent_default);
        let artifact = build_runtime_forms_artifact(&model);
        assert_eq!(artifact.hosts.len(), 1);
        assert_eq!(artifact.hosts[0].event, "submit");
        let html = generate_static_html(&crate::TemplateGraph {
            templates: model.templates.clone(),
        });
        assert!(html.contains("<form data-presolve-node="));
        assert!(!html.contains(" form="));
    }

    #[test]
    fn rejects_multiple_nested_and_cross_form_control_hosts_without_changing_field_ownership() {
        let source = r#"
@component("profile") class Profile {
  @form() @serialize("json") one!: Form; @form() @serialize("json") two!: Form;
  @field(this.one) first = ""; @field(this.two) second = "";
  @action() @submit(this.one) saveOne(): void {} @action() @submit(this.two) saveTwo(): void {}
  render() { return <form form={this.one}><input field={this.second}/><form form={this.one}><input field={this.first}/></form></form>; }
}"#;
        let model =
            build_application_semantic_model(&ezc_parser::parse_file("src/Profile.tsx", source));
        assert!(model.submission_hosts.is_empty());
        assert_eq!(model.form_field_bindings.len(), 2);
        assert!(model
            .submission_host_candidates
            .iter()
            .any(|candidate| candidate
                .violations
                .contains(&SubmissionHostViolation::NestedHost)));
        assert!(model
            .submission_host_candidates
            .iter()
            .any(|candidate| candidate
                .violations
                .contains(&SubmissionHostViolation::ContainedControlForDifferentForm)));
    }
}
