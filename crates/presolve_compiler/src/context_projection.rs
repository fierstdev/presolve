//! V2 Context token and provider inspection over canonical ASM entities.
use crate::{ContextEntity, ContextId, ProviderEntity, SemanticType, SerializationCompatibility};
use serde::Serialize;
pub const CONTEXT_PROJECTION_SCHEMA_VERSION: u32 = 1;
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ContextTokenRecordV1 {
    pub context: String,
    pub owner: String,
    pub has_default: bool,
    pub codec_eligible: bool,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ContextProviderRecordV1 {
    pub provider: String,
    pub context: String,
    pub owner: String,
    pub reactive_value_expression: String,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ContextProjectionV1 {
    pub schema_version: u32,
    pub tokens: Vec<ContextTokenRecordV1>,
    pub providers: Vec<ContextProviderRecordV1>,
}
#[must_use]
pub fn build_context_projection_v1(
    contexts: &std::collections::BTreeMap<ContextId, ContextEntity>,
    providers: &std::collections::BTreeMap<crate::ProviderId, ProviderEntity>,
    types: &std::collections::BTreeMap<crate::SemanticId, SemanticType>,
) -> ContextProjectionV1 {
    let mut tokens = contexts
        .values()
        .map(|c| ContextTokenRecordV1 {
            context: c.id.as_str().into(),
            owner: c
                .owner
                .entity_id()
                .map_or_else(String::new, ToString::to_string),
            has_default: c.default_expression.is_some(),
            codec_eligible: types.get(c.id.as_semantic_id()).is_some_and(|t| {
                crate::serialization_compatibility(t) == SerializationCompatibility::Serializable
            }),
        })
        .collect::<Vec<_>>();
    tokens.sort_by(|a, b| a.context.cmp(&b.context));
    let mut providers = providers
        .values()
        .map(|p| ContextProviderRecordV1 {
            provider: p.id.as_str().into(),
            context: p.context.as_str().into(),
            owner: p
                .owner
                .entity_id()
                .map_or_else(String::new, ToString::to_string),
            reactive_value_expression: p.value_expression.to_string(),
        })
        .collect::<Vec<_>>();
    providers.sort_by(|a, b| a.provider.cmp(&b.provider));
    ContextProjectionV1 {
        schema_version: CONTEXT_PROJECTION_SCHEMA_VERSION,
        tokens,
        providers,
    }
}
