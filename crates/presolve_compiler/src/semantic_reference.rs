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

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum SemanticReferenceKind {
    ActionState,
    ComputedState,
    ComputedComputed,
    /// A direct, compiler-recognized projection from one Computed value onto
    /// the lifecycle record of an exact Resource declaration.
    ComputedResource,
    EffectState,
    EffectComputed,
    ProvidesContext,
    ConsumesContext,
    ResolvesToProvider,
    EventMethod,
    TemplateState,
    TemplateComputed,
    TemplateLocal,
    FieldBindingField,
    FieldBindingForm,
    ValidationRuleField,
}
