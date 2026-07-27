//! V2 inspection of canonical Form submission planning.
use crate::SubmissionProducts;
use serde::Serialize;
pub const FORM_SUBMISSION_PROJECTION_SCHEMA_VERSION: u32 = 1;
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct FormSubmissionProjectionV1 {
    pub schema_version: u32,
    pub valid_plan_count: usize,
    pub candidate_count: usize,
}
#[must_use]
pub fn build_form_submission_projection_v1(
    products: &SubmissionProducts,
) -> FormSubmissionProjectionV1 {
    FormSubmissionProjectionV1 {
        schema_version: FORM_SUBMISSION_PROJECTION_SCHEMA_VERSION,
        valid_plan_count: products.plans.len(),
        candidate_count: products.candidates.len(),
    }
}
