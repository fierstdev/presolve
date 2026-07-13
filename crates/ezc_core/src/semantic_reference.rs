use crate::semantic_id::SemanticId;
use crate::semantic_provenance::SourceProvenance;

/// A resolved directed relationship between compiler semantic entities.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SemanticReference {
    pub kind: SemanticReferenceKind,
    pub source: SemanticId,
    pub target: SemanticId,
    pub provenance: SourceProvenance,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SemanticReferenceKind {
    ActionState,
    ComputedState,
    ComputedComputed,
    EffectState,
    EffectComputed,
    ProvidesContext,
    ConsumesContext,
    ResolvesToProvider,
    EventMethod,
    TemplateState,
    TemplateComputed,
    TemplateLocal,
}
