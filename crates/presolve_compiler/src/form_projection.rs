//! V2 Form and Field inspection over canonical declarations.
use crate::{FormEntity, FormFieldEntity, FormId};
use serde::Serialize;
pub const FORM_PROJECTION_SCHEMA_VERSION: u32 = 1;
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct FormProjectionRecordV1 {
    pub form: String,
    pub owner: String,
    pub fields: Vec<String>,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct FormProjectionV1 {
    pub schema_version: u32,
    pub forms: Vec<FormProjectionRecordV1>,
}
#[must_use]
pub fn build_form_projection_v1(
    forms: &std::collections::BTreeMap<FormId, FormEntity>,
    fields: &std::collections::BTreeMap<crate::FieldId, FormFieldEntity>,
) -> FormProjectionV1 {
    let mut output = forms
        .values()
        .map(|f| {
            let mut names = fields
                .values()
                .filter(|x| x.owner_form == f.id)
                .map(|x| x.id.as_str().into())
                .collect::<Vec<String>>();
            names.sort();
            FormProjectionRecordV1 {
                form: f.id.as_str().into(),
                owner: f
                    .owner
                    .entity_id()
                    .map_or_else(String::new, ToString::to_string),
                fields: names,
            }
        })
        .collect::<Vec<_>>();
    output.sort_by(|a, b| a.form.cmp(&b.form));
    FormProjectionV1 {
        schema_version: FORM_PROJECTION_SCHEMA_VERSION,
        forms: output,
    }
}
