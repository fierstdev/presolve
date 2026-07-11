use crate::semantic_id::SemanticId;

/// A resolved directed relationship between compiler semantic entities.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SemanticReference {
    pub kind: SemanticReferenceKind,
    pub source: SemanticId,
    pub target: SemanticId,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SemanticReferenceKind {
    ActionState,
    EventMethod,
}
