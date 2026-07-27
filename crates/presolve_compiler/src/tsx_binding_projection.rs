//! V2 projection of canonical typed template semantics.
use crate::TemplateSemanticEntity;
use serde::Serialize;
pub const TSX_BINDING_PROJECTION_SCHEMA_VERSION: u32 = 1;
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct TsxBindingProjectionV1 {
    pub schema_version: u32,
    pub entity_count: usize,
    pub binding_count: usize,
}
#[must_use]
pub fn build_tsx_binding_projection_v1(
    entities: &[TemplateSemanticEntity],
) -> TsxBindingProjectionV1 {
    TsxBindingProjectionV1 {
        schema_version: TSX_BINDING_PROJECTION_SCHEMA_VERSION,
        entity_count: entities.len(),
        binding_count: entities
            .iter()
            .filter(|e| {
                matches!(
                    e.kind,
                    crate::TemplateSemanticKind::Binding
                        | crate::TemplateSemanticKind::AttributeBinding
                        | crate::TemplateSemanticKind::EventAttribute
                )
            })
            .count(),
    }
}
