//! I10 deterministic declaration-level Form serialization planning.

use std::collections::BTreeMap;

use crate::{
    serialization_compatibility, ComponentNode, FieldId, FormEntity, FormFieldEntity, FormId,
    FormSubmissionPlan, SerializableValue, SerializationCompatibility, SerializationPlanId,
    SourceProvenance, SubmissionPlanId,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FormSerializationFormat {
    Json,
    FormData,
    UrlEncoded,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FormFieldSerializationConversion {
    JsonValue,
    FormDataScalar,
    UrlEncodedScalar,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SerializedFieldPlan {
    pub field: FieldId,
    pub key: String,
    pub declaration_order: usize,
    pub conversion: FormFieldSerializationConversion,
    pub initial_value: SerializableValue,
    pub provenance: SourceProvenance,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SerializationDeclarationFact {
    pub form: Option<FormId>,
    pub invoked: bool,
    pub argument_count: usize,
    pub format: Option<String>,
    pub provenance: SourceProvenance,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SerializationPlanStatus {
    Valid,
    InvalidDecorator,
    NonSerializableField,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FormSerializationPlan {
    pub id: SerializationPlanId,
    pub form: FormId,
    pub format: FormSerializationFormat,
    pub fields: Vec<SerializedFieldPlan>,
    pub linked_submission: Option<SubmissionPlanId>,
    pub status: SerializationPlanStatus,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct SerializationProducts {
    pub declarations: Vec<SerializationDeclarationFact>,
    pub plans: BTreeMap<SerializationPlanId, FormSerializationPlan>,
}

#[must_use]
pub fn collect_serialization_products(
    components: &[ComponentNode],
    forms: &BTreeMap<FormId, FormEntity>,
    fields: &BTreeMap<FieldId, FormFieldEntity>,
    submissions: &BTreeMap<SubmissionPlanId, FormSubmissionPlan>,
) -> SerializationProducts {
    let declarations = components
        .iter()
        .flat_map(|component| component.serialization_declaration_facts.iter())
        .map(|fact| SerializationDeclarationFact {
            form: fact.declaration_field.as_ref().and_then(|authored| {
                forms
                    .values()
                    .find(|form| &form.authored_field == authored)
                    .map(|form| form.id.clone())
            }),
            invoked: fact.invoked,
            argument_count: fact.argument_count,
            format: fact.format.clone(),
            provenance: fact.decorator_provenance.clone(),
        })
        .collect::<Vec<_>>();
    let mut plans = BTreeMap::new();
    for form in forms.values() {
        let matching = declarations
            .iter()
            .filter(|declaration| declaration.form.as_ref() == Some(&form.id))
            .collect::<Vec<_>>();
        let decorator_valid = matching.len() <= 1
            && matching.iter().all(|declaration| {
                declaration.invoked
                    && declaration.argument_count == 1
                    && matches!(
                        declaration.format.as_deref(),
                        Some("json" | "form-data" | "url-encoded")
                    )
            });
        let format = matching
            .first()
            .and_then(|declaration| declaration.format.as_deref())
            .map_or(FormSerializationFormat::Json, |format| match format {
                "form-data" => FormSerializationFormat::FormData,
                "url-encoded" => FormSerializationFormat::UrlEncoded,
                _ => FormSerializationFormat::Json,
            });
        let conversion = match format {
            FormSerializationFormat::Json => FormFieldSerializationConversion::JsonValue,
            FormSerializationFormat::FormData => FormFieldSerializationConversion::FormDataScalar,
            FormSerializationFormat::UrlEncoded => {
                FormFieldSerializationConversion::UrlEncodedScalar
            }
        };
        let mut ordered = fields
            .values()
            .filter(|field| field.owner_form == form.id)
            .collect::<Vec<_>>();
        ordered.sort_by(|left, right| {
            (left.declaration_order, &left.id).cmp(&(right.declaration_order, &right.id))
        });
        let serializable = ordered.iter().all(|field| {
            serialization_compatibility(&field.semantic_type)
                == SerializationCompatibility::Serializable
        });
        let status = if !decorator_valid {
            SerializationPlanStatus::InvalidDecorator
        } else if !serializable {
            SerializationPlanStatus::NonSerializableField
        } else {
            SerializationPlanStatus::Valid
        };
        plans.insert(
            SerializationPlanId::for_form(&form.id),
            FormSerializationPlan {
                id: SerializationPlanId::for_form(&form.id),
                form: form.id.clone(),
                format,
                fields: ordered
                    .into_iter()
                    .map(|field| SerializedFieldPlan {
                        field: field.id.clone(),
                        key: field.name.clone(),
                        declaration_order: field.declaration_order,
                        conversion,
                        initial_value: field.initial_value.clone(),
                        provenance: field.provenance.clone(),
                    })
                    .collect(),
                linked_submission: submissions
                    .contains_key(&SubmissionPlanId::for_form(&form.id))
                    .then(|| SubmissionPlanId::for_form(&form.id)),
                status,
            },
        );
    }
    SerializationProducts {
        declarations,
        plans,
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        build_application_semantic_model, FormId, FormSerializationFormat, SerializationPlanStatus,
    };

    #[test]
    fn plans_implicit_json_and_explicit_scalar_formats_in_i3_order() {
        let parsed = presolve_parser::parse_file(
            "src/Forms.tsx",
            r#"
@component("forms") class Forms {
  @form() implicit!: Form;
  @form() @serialize("url-encoded") encoded!: Form;
  @field(this.implicit) object = { city: "Austin" };
  @field(this.implicit) value = false;
  @field(this.encoded) name = "Ada";
  render() { return <input field={this.name} />; }
}
"#,
        );
        let model = build_application_semantic_model(&parsed);
        let implicit = FormId::for_owner(&model.components[0].id, "implicit");
        let encoded = FormId::for_owner(&model.components[0].id, "encoded");
        let implicit_plan = model
            .serialization
            .plans
            .get(&crate::SerializationPlanId::for_form(&implicit))
            .expect("implicit plan");
        let encoded_plan = model
            .serialization
            .plans
            .get(&crate::SerializationPlanId::for_form(&encoded))
            .expect("encoded plan");
        assert_eq!(implicit_plan.format, FormSerializationFormat::Json);
        assert_eq!(
            implicit_plan
                .fields
                .iter()
                .map(|field| field.key.as_str())
                .collect::<Vec<_>>(),
            vec!["object", "value"]
        );
        assert_eq!(encoded_plan.format, FormSerializationFormat::UrlEncoded);
        assert_eq!(encoded_plan.status, SerializationPlanStatus::Valid);
    }

    #[test]
    fn retains_invalid_format_as_non_executable_plan_status() {
        let parsed = presolve_parser::parse_file(
            "src/Bad.tsx",
            r#"
@component("bad") class Bad {
  @form() @serialize("xml") value!: Form;
  @field(this.value) name = "";
  render() { return <input field={this.name} />; }
}
"#,
        );
        let model = build_application_semantic_model(&parsed);
        let form = FormId::for_owner(&model.components[0].id, "value");
        assert_eq!(
            model.serialization.plans[&crate::SerializationPlanId::for_form(&form)].status,
            SerializationPlanStatus::InvalidDecorator
        );
    }
}
