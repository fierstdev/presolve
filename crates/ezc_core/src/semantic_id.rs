use std::fmt;
use std::path::{Component, Path};

use serde::{Deserialize, Serialize};

/// Globally stable identity for a compiler semantic entity.
///
/// IDs use the component element name when available because it is the
/// application-facing component identity. Invalid components without an
/// element declaration fall back to their class name so diagnostics and later
/// ASM validation can still refer to them deterministically.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct SemanticId(String);

/// Stable semantic identity for the subject of an effect diagnostic.
///
/// Effects continue to use `SemanticId` throughout the compiler graph. This
/// wrapper exists only at the shared diagnostic boundary so consumers cannot
/// confuse an effect subject with an arbitrary semantic entity.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct EffectId(SemanticId);

/// Stable identity for an authored statement owned by an effect.
///
/// This deliberately remains distinct from `SemanticId` in diagnostic output:
/// statement identity is meaningful only together with its effect subject.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct EffectStatementId(SemanticId);

/// Stable identity for one compiler-owned Context semantic entity.
///
/// Context identity is intentionally distinct from the authored field and all
/// future provider, consumer, and runtime-slot identity domains.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct ContextId(SemanticId);

/// Stable identity for one compiler-owned Context Provider semantic entity.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct ProviderId(SemanticId);

/// Stable identity for one compiler-owned Context Consumer semantic entity.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct ConsumerId(SemanticId);

/// Stable identity for one compiler-owned component Slot semantic entity.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct SlotId(SemanticId);

/// Stable identity for one authored `@slot()` declaration candidate.
///
/// Candidate identity is source-qualified and remains distinct from `SlotId`,
/// so invalid declarations never acquire a valid Slot identity.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct SlotDeclarationCandidateId(SemanticId);

/// Stable identity for one authored component use in a caller template.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct ComponentInvocationId(SemanticId);

/// Typed identity for one canonical authored position in a template traversal.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct TemplatePositionId(SemanticId);

/// Stable identity for one caller-owned content fragment supplied to an invocation Slot.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct SlotContentFragmentId(SemanticId);

/// Stable identity for one callee-authored compile-time Slot outlet.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct SlotOutletId(SemanticId);

/// Stable identity for an authored Context-family declaration candidate.
///
/// Candidates are source-qualified compiler facts.  They intentionally do not
/// share an identity domain with valid Context, Provider, or Consumer entities.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct ContextDeclarationCandidateId(SemanticId);

/// Direct owner of a semantic entity within one compiled application.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SemanticOwner {
    Application,
    Entity(SemanticId),
}

impl SemanticOwner {
    #[must_use]
    pub fn entity(id: SemanticId) -> Self {
        Self::Entity(id)
    }

    #[must_use]
    pub fn entity_id(&self) -> Option<&SemanticId> {
        match self {
            Self::Application => None,
            Self::Entity(id) => Some(id),
        }
    }
}

impl SemanticId {
    #[must_use]
    pub fn component(element_name: Option<&str>, class_name: &str) -> Self {
        Self(format!("component:{}", element_name.unwrap_or(class_name)))
    }

    #[must_use]
    pub fn component_in_module(
        module_path: impl AsRef<Path>,
        element_name: Option<&str>,
        class_name: &str,
    ) -> Self {
        Self(format!(
            "module:{}/component:{}",
            normalized_module_path(module_path.as_ref()),
            element_name.unwrap_or(class_name)
        ))
    }

    #[must_use]
    pub fn type_alias_in_module(module_path: impl AsRef<Path>, name: &str) -> Self {
        Self(format!(
            "module:{}/type-alias:{name}",
            normalized_module_path(module_path.as_ref())
        ))
    }

    #[must_use]
    pub fn state_field(&self, name: &str) -> Self {
        self.child("state", name)
    }

    #[must_use]
    pub fn method(&self, name: &str) -> Self {
        self.child("method", name)
    }

    #[must_use]
    pub fn computed(&self, name: &str) -> Self {
        self.child("computed", name)
    }

    #[must_use]
    pub fn effect(&self, name: &str) -> Self {
        self.child("effect", name)
    }

    #[must_use]
    pub fn context(&self, name: &str) -> Self {
        self.child("context", name)
    }

    #[must_use]
    pub fn context_field(&self, name: &str) -> Self {
        self.child("context-field", name)
    }

    #[must_use]
    pub fn provider(&self, name: &str) -> Self {
        self.child("provider", name)
    }

    #[must_use]
    pub fn provider_field(&self, name: &str) -> Self {
        self.child("provider-field", name)
    }

    #[must_use]
    pub fn consumer(&self, name: &str) -> Self {
        self.child("consumer", name)
    }

    #[must_use]
    pub fn consumer_field(&self, name: &str) -> Self {
        self.child("consumer-field", name)
    }

    #[must_use]
    pub fn slot(&self, name: &str) -> Self {
        self.child("slot", name)
    }

    #[must_use]
    pub fn slot_field(&self, name: &str) -> Self {
        self.child("slot-field", name)
    }

    #[must_use]
    pub fn slot_declaration_candidate(&self, position: usize) -> Self {
        self.child("slot-declaration", &position.to_string())
    }

    #[must_use]
    pub fn context_declaration_candidate(&self, position: usize) -> Self {
        self.child("context-declaration", &position.to_string())
    }

    /// Stable generated-function identity for a Provider Context source.
    #[must_use]
    pub fn context_provider_function(&self) -> Self {
        self.child("context-source", "evaluation")
    }

    /// Stable generated-function identity for a Context-default source.
    #[must_use]
    pub fn context_default_function(&self) -> Self {
        self.child("context-default-source", "evaluation")
    }

    /// Stable semantic origin used only to scope a retained Context Consumer load value.
    #[must_use]
    pub fn context_consumer_load(&self) -> Self {
        self.child("context-load", "value")
    }

    #[must_use]
    pub fn effect_activation_slot(&self, name: &str) -> Self {
        self.child("effect-activation", name)
    }

    #[must_use]
    pub fn effect_statement(&self, index: usize) -> Self {
        self.child("statement", &index.to_string())
    }

    #[must_use]
    pub fn action(&self, method: &str, index: usize) -> Self {
        self.child("action", &format!("{method}:{index}"))
    }

    #[must_use]
    pub fn action_batch(&self, method: &str) -> Self {
        self.child("action-batch", method)
    }

    #[must_use]
    pub fn local_variable(&self, name: &str, index: usize) -> Self {
        self.child("local", &format!("{name}:{index}"))
    }

    #[must_use]
    pub fn parameter(&self, name: &str, index: usize) -> Self {
        self.child("parameter", &format!("{name}:{index}"))
    }

    #[must_use]
    pub fn expression(&self, path: &str) -> Self {
        self.child("expression", path)
    }

    #[must_use]
    pub fn semantic_type(&self) -> Self {
        self.child("type", "semantic")
    }

    #[must_use]
    pub fn event_handler(&self, event: &str, index: usize) -> Self {
        self.child("event", &format!("{event}:{index}"))
    }

    #[must_use]
    pub fn template(&self) -> Self {
        self.child("template", "render")
    }

    #[must_use]
    pub fn template_entity(&self, kind: &str, path: &str) -> Self {
        self.child(kind, path)
    }

    #[must_use]
    pub fn component_invocation(&self, target: &str) -> Self {
        self.child("component-invocation", target)
    }

    #[must_use]
    pub fn template_position(&self) -> Self {
        self.child("template-position", "authored")
    }

    #[must_use]
    pub fn slot_content_fragment(&self, slot_name: &str) -> Self {
        self.child("slot-content", slot_name)
    }

    #[must_use]
    pub fn slot_outlet(&self, slot_name: &str) -> Self {
        self.child("slot-outlet", slot_name)
    }

    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }

    fn child(&self, kind: &str, name: &str) -> Self {
        Self(format!("{}/{kind}:{name}", self.0))
    }
}

impl EffectId {
    #[must_use]
    pub fn from_semantic(id: &SemanticId) -> Self {
        Self(id.clone())
    }

    #[must_use]
    pub fn as_str(&self) -> &str {
        self.0.as_str()
    }
}

impl EffectStatementId {
    #[must_use]
    pub fn from_semantic(id: &SemanticId) -> Self {
        Self(id.clone())
    }

    #[must_use]
    pub fn as_str(&self) -> &str {
        self.0.as_str()
    }
}

impl ContextId {
    #[must_use]
    pub fn for_component(component: &SemanticId, name: &str) -> Self {
        Self(component.context(name))
    }

    #[must_use]
    pub const fn as_semantic_id(&self) -> &SemanticId {
        &self.0
    }

    #[must_use]
    pub fn as_str(&self) -> &str {
        self.0.as_str()
    }
}

impl ProviderId {
    #[must_use]
    pub fn for_component(component: &SemanticId, name: &str) -> Self {
        Self(component.provider(name))
    }

    #[must_use]
    pub const fn as_semantic_id(&self) -> &SemanticId {
        &self.0
    }

    #[must_use]
    pub fn as_str(&self) -> &str {
        self.0.as_str()
    }
}

impl ConsumerId {
    #[must_use]
    pub fn for_component(component: &SemanticId, name: &str) -> Self {
        Self(component.consumer(name))
    }

    #[must_use]
    pub const fn as_semantic_id(&self) -> &SemanticId {
        &self.0
    }

    #[must_use]
    pub fn as_str(&self) -> &str {
        self.0.as_str()
    }
}

impl SlotId {
    #[must_use]
    pub fn for_component(component: &SemanticId, name: &str) -> Self {
        Self(component.slot(name))
    }

    #[must_use]
    pub const fn as_semantic_id(&self) -> &SemanticId {
        &self.0
    }

    #[must_use]
    pub fn as_str(&self) -> &str {
        self.0.as_str()
    }
}

impl SlotDeclarationCandidateId {
    #[must_use]
    pub fn for_component_position(component: &SemanticId, position: usize) -> Self {
        Self(component.slot_declaration_candidate(position))
    }

    #[must_use]
    pub const fn as_semantic_id(&self) -> &SemanticId {
        &self.0
    }

    #[must_use]
    pub fn as_str(&self) -> &str {
        self.0.as_str()
    }
}

impl ComponentInvocationId {
    #[must_use]
    pub fn for_template_entity(template_entity: &SemanticId, target: &str) -> Self {
        Self(template_entity.component_invocation(target))
    }

    #[must_use]
    pub const fn as_semantic_id(&self) -> &SemanticId {
        &self.0
    }

    #[must_use]
    pub fn as_str(&self) -> &str {
        self.0.as_str()
    }
}

impl TemplatePositionId {
    #[must_use]
    pub fn for_template_entity(template_entity: &SemanticId) -> Self {
        Self(template_entity.template_position())
    }

    #[must_use]
    pub const fn as_semantic_id(&self) -> &SemanticId {
        &self.0
    }

    #[must_use]
    pub fn as_str(&self) -> &str {
        self.0.as_str()
    }
}

impl SlotContentFragmentId {
    #[must_use]
    pub fn for_invocation(invocation: &ComponentInvocationId, slot_name: &str) -> Self {
        Self(invocation.as_semantic_id().slot_content_fragment(slot_name))
    }

    #[must_use]
    pub const fn as_semantic_id(&self) -> &SemanticId {
        &self.0
    }

    #[must_use]
    pub fn as_str(&self) -> &str {
        self.0.as_str()
    }
}

impl SlotOutletId {
    #[must_use]
    pub fn for_template_entity(template_entity: &SemanticId, slot_name: &str) -> Self {
        Self(template_entity.slot_outlet(slot_name))
    }

    #[must_use]
    pub const fn as_semantic_id(&self) -> &SemanticId {
        &self.0
    }

    #[must_use]
    pub fn as_str(&self) -> &str {
        self.0.as_str()
    }
}

impl ContextDeclarationCandidateId {
    #[must_use]
    pub fn for_component_position(component: &SemanticId, position: usize) -> Self {
        Self(component.context_declaration_candidate(position))
    }

    #[must_use]
    pub const fn as_semantic_id(&self) -> &SemanticId {
        &self.0
    }

    #[must_use]
    pub fn as_str(&self) -> &str {
        self.0.as_str()
    }
}

fn normalized_module_path(path: &Path) -> String {
    let mut segments = Vec::new();
    let absolute = path.is_absolute();

    for component in path.components() {
        match component {
            Component::CurDir | Component::RootDir | Component::Prefix(_) => {}
            Component::ParentDir => {
                if segments.last().is_some_and(|segment| *segment != "..") {
                    segments.pop();
                } else {
                    segments.push("..".to_string());
                }
            }
            Component::Normal(segment) => segments.push(segment.to_string_lossy().into_owned()),
        }
    }

    let path = segments.join("/");
    if absolute {
        format!("/{path}")
    } else {
        path
    }
}

impl fmt::Display for SemanticId {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(formatter)
    }
}

impl fmt::Display for EffectId {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(formatter)
    }
}

impl fmt::Display for EffectStatementId {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(formatter)
    }
}

impl fmt::Display for ContextId {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(formatter)
    }
}

impl fmt::Display for ProviderId {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(formatter)
    }
}

impl fmt::Display for ConsumerId {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(formatter)
    }
}

impl fmt::Display for SlotId {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(formatter)
    }
}

impl fmt::Display for SlotDeclarationCandidateId {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(formatter)
    }
}

impl fmt::Display for ComponentInvocationId {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(formatter)
    }
}

impl fmt::Display for TemplatePositionId {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(formatter)
    }
}

impl fmt::Display for SlotContentFragmentId {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(formatter)
    }
}

impl fmt::Display for SlotOutletId {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(formatter)
    }
}

impl fmt::Display for ContextDeclarationCandidateId {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(formatter)
    }
}

#[cfg(test)]
mod tests {
    use super::{
        ComponentInvocationId, ConsumerId, ContextId, ProviderId, SemanticId, SemanticOwner,
        SlotId, TemplatePositionId,
    };

    #[test]
    fn derives_component_scoped_ids() {
        let component = SemanticId::component(Some("x-counter"), "Counter");

        assert_eq!(component.as_str(), "component:x-counter");
        assert_eq!(
            component.state_field("count").as_str(),
            "component:x-counter/state:count"
        );
        assert_eq!(
            component.method("increment").as_str(),
            "component:x-counter/method:increment"
        );
        assert_eq!(
            component.computed("remainingCount").as_str(),
            "component:x-counter/computed:remainingCount"
        );
        assert_eq!(
            component.effect("syncTitle").as_str(),
            "component:x-counter/effect:syncTitle"
        );
        assert_eq!(
            ContextId::for_component(&component, "theme").as_str(),
            "component:x-counter/context:theme"
        );
        assert_eq!(
            ProviderId::for_component(&component, "providedTheme").as_str(),
            "component:x-counter/provider:providedTheme"
        );
        assert_eq!(
            ConsumerId::for_component(&component, "theme").as_str(),
            "component:x-counter/consumer:theme"
        );
        assert_eq!(
            SlotId::for_component(&component, "children").as_str(),
            "component:x-counter/slot:children"
        );
        assert_eq!(
            component.effect_activation_slot("syncTitle").as_str(),
            "component:x-counter/effect-activation:syncTitle"
        );
        assert_eq!(
            component.action("increment", 0).as_str(),
            "component:x-counter/action:increment:0"
        );
        assert_eq!(
            component
                .method("render")
                .local_variable("title", 0)
                .as_str(),
            "component:x-counter/method:render/local:title:0"
        );
        assert_eq!(
            component.event_handler("click", 0).as_str(),
            "component:x-counter/event:click:0"
        );
        assert_eq!(
            component.template().as_str(),
            "component:x-counter/template:render"
        );
        let element = component.template().template_entity("element", "root.0");
        assert_eq!(
            TemplatePositionId::for_template_entity(&element).as_str(),
            "component:x-counter/template:render/element:root.0/template-position:authored"
        );
        assert_eq!(
            ComponentInvocationId::for_template_entity(&element, "component:x-card").as_str(),
            "component:x-counter/template:render/element:root.0/component-invocation:component:x-card"
        );
    }

    #[test]
    fn derives_module_qualified_component_ids() {
        let component =
            SemanticId::component_in_module("src/../src/Counter.tsx", Some("x-counter"), "Counter");

        assert_eq!(
            component.as_str(),
            "module:src/Counter.tsx/component:x-counter"
        );
        assert_eq!(
            component.state_field("count").as_str(),
            "module:src/Counter.tsx/component:x-counter/state:count"
        );
    }

    #[test]
    fn falls_back_to_class_name_for_invalid_components() {
        assert_eq!(
            SemanticId::component(None, "MissingDecorator").as_str(),
            "component:MissingDecorator"
        );
    }

    #[test]
    fn distinguishes_application_roots_from_entity_owners() {
        let component = SemanticId::component(Some("x-counter"), "Counter");

        assert_eq!(SemanticOwner::Application.entity_id(), None);
        assert_eq!(
            SemanticOwner::entity(component.clone()).entity_id(),
            Some(&component)
        );
    }
}
