//! I15 public compiler-generated Forms runtime artifact.
//!
//! This module is deliberately a projection of immutable compiler products.
//! It does not inspect source syntax or create runtime state.

use std::collections::BTreeSet;

use serde::Serialize;

use crate::{
    semantic_type_text, ApplicationSemanticModel, FormControlCompatibility, FormIrOperation,
    RUNTIME_FORM_REGISTRY_VERSION,
};

pub const RUNTIME_FORM_ARTIFACT_SCHEMA_VERSION: u32 = 2;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct RuntimeFormsArtifact {
    pub schema_version: u32,
    pub registry_version: u32,
    pub forms: Vec<RuntimeFormsArtifactForm>,
    pub instances: Vec<RuntimeFormsArtifactInstance>,
    /// Instance-qualified executable submit-host records. These are the only
    /// runtime authority for locating and handling a native submit event.
    pub hosts: Vec<RuntimeFormsArtifactHost>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct RuntimeFormsArtifactForm {
    pub id: String,
    /// Non-authoritative authored name for developer tooling only.
    pub debug_name: String,
    pub fields: Vec<RuntimeFormsArtifactField>,
    pub bindings: Vec<RuntimeFormsArtifactBinding>,
    pub validation_rules: Vec<RuntimeFormsArtifactRule>,
    pub validation_dependencies: Vec<RuntimeFormsArtifactDependency>,
    pub submission: Option<RuntimeFormsArtifactSubmission>,
    pub serialization: RuntimeFormsArtifactSerialization,
    pub reset: RuntimeFormsArtifactReset,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct RuntimeFormsArtifactField {
    pub id: String,
    /// Non-authoritative authored name for developer tooling only.
    pub debug_name: String,
    /// Exact compiler-issued serialized leaf path.
    pub path: Vec<String>,
    pub semantic_type: String,
    pub initial_value: crate::SerializableValue,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct RuntimeFormsArtifactBinding {
    pub id: String,
    pub control_anchor: String,
    pub field: String,
    pub channel: String,
    pub normalization: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct RuntimeFormsArtifactRule {
    pub id: String,
    pub target_field: String,
    pub kind: String,
    pub argument: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dependency: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct RuntimeFormsArtifactDependency {
    pub id: String,
    pub source_field: String,
    pub target_field: String,
    pub rule: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct RuntimeFormsArtifactSubmission {
    pub plan: String,
    pub action_batch: String,
    pub validation_rules: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct RuntimeFormsArtifactSerialization {
    pub plan: String,
    pub format: String,
    pub fields: Vec<String>,
    pub status: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct RuntimeFormsArtifactReset {
    pub plan: String,
    pub fields: Vec<String>,
    pub schedule_validation: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct RuntimeFormsArtifactInstance {
    pub id: String,
    pub form: String,
    pub component_instance: String,
    pub field_slots: Vec<RuntimeFormsArtifactFieldSlots>,
    pub aggregate_validation_slot: String,
    pub submission_slot: String,
    pub programs: RuntimeFormsArtifactPrograms,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct RuntimeFormsArtifactHost {
    pub id: String,
    pub host_anchor: String,
    pub form: String,
    pub form_instance: String,
    pub submission_plan: String,
    pub submit_action: String,
    pub action_batch: String,
    pub serialization_plan: String,
    pub event: String,
    pub prevent_default: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct RuntimeFormsArtifactFieldSlots {
    pub field: String,
    pub value: String,
    pub dirty: String,
    pub touched: String,
    pub validation: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct RuntimeFormsArtifactPrograms {
    pub initialize: Vec<String>,
    pub input: Vec<RuntimeFormsArtifactFieldProgram>,
    pub blur: Vec<RuntimeFormsArtifactFieldProgram>,
    pub reset: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct RuntimeFormsArtifactFieldProgram {
    pub field: String,
    pub operations: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeFormsArtifactValidation {
    pub diagnostics: Vec<String>,
    pub is_valid: bool,
}

/// Project every executable I0-I14 Forms product into the versioned public
/// artifact. All identity-bearing fields are canonical IDs; authored names are
/// kept in explicit non-authoritative debug fields only.
///
/// # Panics
///
/// Panics when a prior I10 or I11 product failed to provide its mandatory
/// per-Form plan. That is an internal staged-compiler invariant violation.
#[must_use]
#[allow(clippy::too_many_lines)]
pub fn build_runtime_forms_artifact(model: &ApplicationSemanticModel) -> RuntimeFormsArtifact {
    let forms = model
        .forms
        .values()
        .map(|form| {
            let fields = model
                .form_fields
                .values()
                .filter(|field| field.owner_form == form.id)
                .map(|field| RuntimeFormsArtifactField {
                    id: field.id.to_string(),
                    debug_name: field.name.clone(),
                    path: field.path.clone(),
                    semantic_type: semantic_type_text(&field.semantic_type),
                    initial_value: field.initial_value.clone(),
                })
                .collect::<Vec<_>>();
            let bindings = model
                .form_field_bindings
                .values()
                .filter(|binding| binding.form == form.id)
                .map(|binding| RuntimeFormsArtifactBinding {
                    id: binding.id.to_string(),
                    control_anchor: binding.control_entity.to_string(),
                    field: binding.field.to_string(),
                    channel: format!("{:?}", binding.channel),
                    normalization: normalization_name(binding.compatibility),
                })
                .collect();
            let validation_rules = model
                .validation_rules
                .values()
                .filter(|rule| rule.owner_form == form.id)
                .map(|rule| RuntimeFormsArtifactRule {
                    id: rule.id.to_string(),
                    target_field: rule.target_field.to_string(),
                    kind: format!("{:?}", rule.kind),
                    argument: format!("{:?}", rule.argument),
                    dependency: rule.dependency.as_ref().map(ToString::to_string),
                })
                .collect();
            let validation_dependencies = model
                .validation_dependency_plans
                .dependencies
                .values()
                .filter(|dependency| dependency.form == form.id)
                .map(|dependency| RuntimeFormsArtifactDependency {
                    id: dependency.id.to_string(),
                    source_field: dependency.source_field.to_string(),
                    target_field: dependency.target_field.to_string(),
                    rule: dependency.dependent_rule.to_string(),
                })
                .collect();
            let submission =
                model
                    .submissions
                    .plan(&form.id)
                    .map(|plan| RuntimeFormsArtifactSubmission {
                        plan: plan.id.as_str().to_string(),
                        action_batch: plan.action_batch.to_string(),
                        validation_rules: plan
                            .validation_rules
                            .iter()
                            .map(ToString::to_string)
                            .collect(),
                    });
            let serialization = model
                .serialization
                .plans
                .get(&crate::SerializationPlanId::for_form(&form.id))
                .expect("I10 creates one serialization plan per valid Form");
            let reset = model
                .reset
                .plans
                .get(&crate::ResetPlanId::for_form(&form.id))
                .expect("I11 creates one reset plan per valid Form");
            RuntimeFormsArtifactForm {
                id: form.id.to_string(),
                debug_name: form.name.clone(),
                fields,
                bindings,
                validation_rules,
                validation_dependencies,
                submission,
                serialization: RuntimeFormsArtifactSerialization {
                    plan: serialization.id.as_str().to_string(),
                    format: format!("{:?}", serialization.format),
                    fields: serialization
                        .fields
                        .iter()
                        .map(|field| field.field.to_string())
                        .collect(),
                    status: format!("{:?}", serialization.status),
                },
                reset: RuntimeFormsArtifactReset {
                    plan: reset.id.as_str().to_string(),
                    fields: reset
                        .operations
                        .iter()
                        .map(|operation| operation.field.to_string())
                        .collect(),
                    schedule_validation: reset.schedule_validation,
                },
            }
        })
        .collect();
    let instances = model
        .optimized_form_ir
        .optimized
        .instances
        .values()
        .map(|instance| RuntimeFormsArtifactInstance {
            id: instance.id.to_string(),
            form: instance.form.to_string(),
            component_instance: instance.component_instance.to_string(),
            field_slots: instance
                .storage
                .value
                .iter()
                .map(|(field, value)| RuntimeFormsArtifactFieldSlots {
                    field: field.to_string(),
                    value: value.as_str().to_string(),
                    dirty: instance.storage.dirty[field].as_str().to_string(),
                    touched: instance.storage.touched[field].as_str().to_string(),
                    validation: instance.storage.validation[field].as_str().to_string(),
                })
                .collect(),
            aggregate_validation_slot: instance.storage.aggregate.as_str().to_string(),
            submission_slot: instance.storage.submission.as_str().to_string(),
            programs: RuntimeFormsArtifactPrograms {
                initialize: operations(&instance.initialize),
                input: field_programs(&instance.input),
                blur: field_programs(&instance.blur),
                reset: operations(&instance.reset),
            },
        })
        .collect();
    let hosts = model
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
                .map(move |instance| RuntimeFormsArtifactHost {
                    id: host.id.to_string(),
                    host_anchor: runtime_anchor_for_element(
                        model,
                        &host.owner_template,
                        &host.owner_template_element,
                    )
                    .expect("valid host has exact template anchor"),
                    form: host.form.to_string(),
                    form_instance: instance.id.to_string(),
                    submission_plan: host.submission_plan.as_str().to_string(),
                    submit_action: host.submit_action.to_string(),
                    action_batch: host.action_batch.to_string(),
                    serialization_plan: host.serialization_plan.as_str().to_string(),
                    event: host.event.to_string(),
                    prevent_default: host.prevent_default,
                })
        })
        .collect();
    let artifact = RuntimeFormsArtifact {
        schema_version: RUNTIME_FORM_ARTIFACT_SCHEMA_VERSION,
        registry_version: RUNTIME_FORM_REGISTRY_VERSION,
        forms,
        instances,
        hosts,
    };
    debug_assert!(validate_runtime_forms_artifact(&artifact).is_valid);
    artifact
}

#[allow(clippy::items_after_statements)]
fn runtime_anchor_for_element(
    model: &ApplicationSemanticModel,
    template_id: &crate::SemanticId,
    element: &crate::SemanticId,
) -> Option<String> {
    let template = model
        .templates
        .iter()
        .find(|template| &template.id == template_id)?;
    fn find(
        element: &crate::ElementNode,
        template: &crate::TemplateNode,
        path: &str,
        target: &crate::SemanticId,
    ) -> Option<String> {
        if template.id.template_entity("element", path) == *target {
            return Some(element.id.0.clone());
        }
        for (index, child) in element.children.iter().enumerate() {
            if let crate::TemplateChild::Element(child) = child {
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
        .and_then(|root| find(root, template, "root", element))
}

#[must_use]
pub fn validate_runtime_forms_artifact(
    artifact: &RuntimeFormsArtifact,
) -> RuntimeFormsArtifactValidation {
    let mut diagnostics = Vec::new();
    if artifact.schema_version != RUNTIME_FORM_ARTIFACT_SCHEMA_VERSION {
        diagnostics.push("unsupported Forms artifact schema".to_string());
    }
    let forms = artifact
        .forms
        .iter()
        .map(|form| form.id.as_str())
        .collect::<BTreeSet<_>>();
    if forms.len() != artifact.forms.len() {
        diagnostics.push("duplicate Form definition".to_string());
    }
    let instances = artifact
        .instances
        .iter()
        .map(|instance| instance.id.as_str())
        .collect::<BTreeSet<_>>();
    if instances.len() != artifact.instances.len() {
        diagnostics.push("duplicate Form instance".to_string());
    }
    for instance in &artifact.instances {
        if !forms.contains(instance.form.as_str()) {
            diagnostics.push(format!("unknown Form for instance {}", instance.id));
        }
    }
    for host in &artifact.hosts {
        if !instances.contains(host.form_instance.as_str())
            || !forms.contains(host.form.as_str())
            || host.event != "submit"
        {
            diagnostics.push(format!("invalid submission host {}", host.id));
        }
    }
    RuntimeFormsArtifactValidation {
        is_valid: diagnostics.is_empty(),
        diagnostics,
    }
}

#[must_use]
/// # Panics
///
/// Panics only if an internal Forms artifact cannot serialize, which indicates
/// a compiler implementation defect.
pub fn runtime_forms_artifact_json(artifact: &RuntimeFormsArtifact) -> String {
    serde_json::to_string_pretty(artifact).expect("Forms artifact serializes deterministically")
        + "\n"
}

fn normalization_name(compatibility: FormControlCompatibility) -> String {
    match compatibility {
        FormControlCompatibility::Compatible(normalization) => format!("{normalization:?}"),
        FormControlCompatibility::Incompatible => "Incompatible".to_string(),
    }
}

fn operations(operations: &[FormIrOperation]) -> Vec<String> {
    operations
        .iter()
        .map(|operation| format!("{operation:?}"))
        .collect()
}

fn field_programs(
    programs: &std::collections::BTreeMap<crate::FieldId, Vec<FormIrOperation>>,
) -> Vec<RuntimeFormsArtifactFieldProgram> {
    programs
        .iter()
        .map(
            |(field, program_operations)| RuntimeFormsArtifactFieldProgram {
                field: field.to_string(),
                operations: operations(program_operations),
            },
        )
        .collect()
}

#[cfg(test)]
mod tests {
    #[test]
    fn emits_schema_v2_with_only_canonical_execution_references_and_field_paths() {
        let parsed = presolve_parser::parse_file(
            "src/X.tsx",
            r#"@component("x")class X{@form()form!:Form;@field(this.form)value="";render(){return <input field={this.value}/>;}}"#,
        );
        let asm = crate::build_application_semantic_model(&parsed);
        let artifact = super::build_runtime_forms_artifact(&asm);
        assert_eq!(artifact.schema_version, 2);
        assert_eq!(artifact.registry_version, 1);
        assert_eq!(artifact.forms.len(), 1);
        assert_eq!(artifact.instances.len(), 1);
        assert!(super::validate_runtime_forms_artifact(&artifact).is_valid);
        assert!(super::runtime_forms_artifact_json(&artifact).contains("field-binding"));
    }

    #[test]
    fn forms_products_are_byte_deterministic_when_input_files_are_reversed() {
        let a = r#"@component("a") class A { @form() @serialize("json") profile!: Form; @field(this.profile) name = ""; @action() @submit(this.profile) save(): void {} render() { return <form form={this.profile}><input field={this.name}/></form>; } }"#;
        let b = r#"@component("b") class B { @form() @serialize("url-encoded") search!: Form; @field(this.search) query = ""; @action() @submit(this.search) save(): void {} render() { return <form form={this.search}><input field={this.query}/></form>; } }"#;
        let first = crate::CompilationUnit::parse_sources([("src/A.tsx", a), ("src/B.tsx", b)]);
        let second = crate::CompilationUnit::parse_sources([("src/B.tsx", b), ("src/A.tsx", a)]);
        let first = crate::build_application_semantic_model_for_unit(&first);
        let second = crate::build_application_semantic_model_for_unit(&second);

        assert_eq!(
            super::runtime_forms_artifact_json(&super::build_runtime_forms_artifact(&first)),
            super::runtime_forms_artifact_json(&super::build_runtime_forms_artifact(&second)),
        );
        assert_eq!(
            crate::template_manifest_json(&crate::build_template_manifest_from_asm(&first)),
            crate::template_manifest_json(&crate::build_template_manifest_from_asm(&second)),
        );
        assert_eq!(
            crate::resume_manifest_json(&crate::build_resume_manifest(&first)),
            crate::resume_manifest_json(&crate::build_resume_manifest(&second)),
        );
    }
}
