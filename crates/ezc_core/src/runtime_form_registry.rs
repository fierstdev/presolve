//! I14 versioned compiler-generated Form runtime registry.
use crate::{
    FieldId, FormEntity, FormFieldBinding, FormFieldEntity, FormId, FormInstanceId, FormIrReport,
    FormSerializationPlan, FormSubmissionPlan, ResetProducts, SerializationProducts,
    SubmissionProducts, ValidationRule, ValidationRuleId,
};
use std::collections::BTreeMap;
pub const RUNTIME_FORM_REGISTRY_VERSION: u32 = 1;
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeFormRecord {
    pub form: FormId,
    pub fields: Vec<FieldId>,
    pub submission: Option<FormSubmissionPlan>,
    pub serialization: Option<FormSerializationPlan>,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeFormInstanceRecord {
    pub instance: FormInstanceId,
    pub form: FormId,
    pub programs: usize,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeFormRegistry {
    pub version: u32,
    pub forms: BTreeMap<FormId, RuntimeFormRecord>,
    pub instances: BTreeMap<FormInstanceId, RuntimeFormInstanceRecord>,
    pub bindings: Vec<FormFieldBinding>,
    pub rules: BTreeMap<ValidationRuleId, ValidationRule>,
    pub reset: ResetProducts,
}
#[must_use]
#[allow(clippy::too_many_arguments)]
pub fn build_runtime_form_registry(
    forms: &BTreeMap<FormId, FormEntity>,
    fields: &BTreeMap<FieldId, FormFieldEntity>,
    bindings: &BTreeMap<crate::FieldBindingId, FormFieldBinding>,
    rules: &BTreeMap<ValidationRuleId, ValidationRule>,
    ir: &FormIrReport,
    submissions: &SubmissionProducts,
    serialization: &SerializationProducts,
    reset: &ResetProducts,
) -> RuntimeFormRegistry {
    let forms = forms
        .values()
        .map(|form| {
            let mut ids = fields
                .values()
                .filter(|f| f.owner_form == form.id)
                .collect::<Vec<_>>();
            ids.sort_by_key(|f| f.declaration_order);
            (
                form.id.clone(),
                RuntimeFormRecord {
                    form: form.id.clone(),
                    fields: ids.into_iter().map(|f| f.id.clone()).collect(),
                    submission: submissions
                        .plans
                        .get(&crate::SubmissionPlanId::for_form(&form.id))
                        .cloned(),
                    serialization: serialization
                        .plans
                        .get(&crate::SerializationPlanId::for_form(&form.id))
                        .cloned(),
                },
            )
        })
        .collect();
    let instances = ir
        .instances
        .values()
        .map(|instance| {
            (
                instance.id.clone(),
                RuntimeFormInstanceRecord {
                    instance: instance.id.clone(),
                    form: instance.form.clone(),
                    programs: instance.input.len() + instance.blur.len() + 1,
                },
            )
        })
        .collect();
    RuntimeFormRegistry {
        version: RUNTIME_FORM_REGISTRY_VERSION,
        forms,
        instances,
        bindings: bindings.values().cloned().collect(),
        rules: rules.clone(),
        reset: reset.clone(),
    }
}
#[cfg(test)]
mod tests {
    #[test]
    fn is_versioned_and_instance_qualified() {
        let parsed = ezc_parser::parse_file(
            "src/X.tsx",
            r#"@component("x")class X{@form()form!:Form;@field(this.form)value="";render(){return <input field={this.value}/>;}}"#,
        );
        let asm = crate::build_application_semantic_model(&parsed);
        assert_eq!(asm.runtime_forms.version, 1);
        assert!(asm
            .runtime_forms
            .instances
            .values()
            .all(|r| asm.runtime_forms.forms.contains_key(&r.form)));
    }
}
