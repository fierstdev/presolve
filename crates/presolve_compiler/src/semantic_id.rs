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

/// Stable identity for one compiler-owned Form semantic entity.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct FormId(SemanticId);

/// Stable identity for one compiler-owned Form instance.
///
/// A Form definition and each of its component-instance-qualified executions
/// are distinct identity domains.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct FormInstanceId(SemanticId);

/// Source-qualified identity for one recognized `@form` declaration candidate.
///
/// Candidate identity remains available even when no canonical Form identity
/// can be constructed from the authored target.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct FormDeclarationCandidateId(SemanticId);

/// Source-qualified identity for one recognized `@field` declaration
/// candidate. It is intentionally distinct from canonical `FieldId`.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct FormFieldDeclarationCandidateId(SemanticId);

/// Source-qualified identity for one recognized JSX `field` binding candidate.
/// It remains distinct from a valid template-occurrence binding identity.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct FormFieldBindingCandidateId(SemanticId);

/// Stable identity for one compiler-owned Field owned by a Form.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct FieldId(SemanticId);

/// Stable identity for one authored template control occurrence bound to a
/// canonical Form Field.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct FieldBindingId(SemanticId);

/// Source-qualified candidate identity for a compiler-owned submission-host
/// attribute. Invalid candidates never acquire a host semantic identity.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct SubmissionHostCandidateId(SemanticId);

/// Stable declaration-level identity for one explicit template submission host.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct SubmissionHostId(SemanticId);

/// Stable identity for the declaration-level Form ownership projection of one
/// canonical application/build-root set.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct FormOwnershipGraphId(SemanticId);

/// Source-qualified identity for every recognized `@validate` placement.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct ValidationRuleCandidateId(SemanticId);

/// Stable identity for one valid compiler-owned validation rule.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct ValidationRuleId(SemanticId);

/// Stable identity for the declaration-level validation graph of one build.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct ValidationGraphId(SemanticId);

/// Stable identity for the declaration-level validation dependency plan of one
/// canonical Form. This is deliberately distinct from both Form declarations
/// and future Form-instance-qualified execution plans.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct ValidationPlanId(SemanticId);

/// Stable identity for one direct I6 validation-rule read dependency.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct FieldDependencyId(SemanticId);

/// Stable identity for one canonical dependency-cycle field set.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct ValidationDependencyCycleId(SemanticId);

/// Stable identity for one declaration-level Form dirty-tracking plan.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct DirtyTrackingPlanId(SemanticId);

/// Stable identity for one declaration-level Form touched-tracking plan.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct TouchedTrackingPlanId(SemanticId);

/// Stable identity for the tracking facts associated with one declaration-level
/// Form Field. It is intentionally not a runtime storage identity.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct FieldTrackingId(SemanticId);

/// Source-qualified identity for an authored `@submit` declaration candidate.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct SubmissionDeclarationCandidateId(SemanticId);

/// Stable identity for the submission plan of one canonical Form.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct SubmissionPlanId(SemanticId);

/// Stable identity for the serialization plan of one canonical Form.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct SerializationPlanId(SemanticId);

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct ResetPlanId(SemanticId);

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct FieldResetOperationId(SemanticId);

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct FormFieldValueSlotId(SemanticId);
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct FormFieldDirtySlotId(SemanticId);
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct FormFieldTouchedSlotId(SemanticId);
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct FormFieldValidationSlotId(SemanticId);
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct FormValidationAggregateSlotId(SemanticId);
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct FormSubmissionStateSlotId(SemanticId);

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

/// Stable identity for one invocation- and callee-instance-specific Slot binding.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct SlotBindingId(SemanticId);

/// Stable identity for one compiler-owned build/page component root.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct ComponentRootId(SemanticId);

/// Stable identity for one component instance or blocked instance boundary.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct ComponentInstanceId(SemanticId);

/// Stable identity for a conditional or keyed-list component instance template region.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct ComponentStructuralRegionId(SemanticId);

/// Stable identity for one compiler-owned Resource declaration.
///
/// A Resource declaration describes the application-level operation. Its
/// per-component-instance activation has a distinct identity.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct ResourceId(SemanticId);

/// Stable identity for one component-instance-qualified Resource activation.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct ResourceActivationId(SemanticId);

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
    pub fn component_root(component: &Self) -> Self {
        Self(format!("root:{}", component.as_str()))
    }
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
    pub fn form(&self, name: &str) -> Self {
        self.child("form", name)
    }

    #[must_use]
    pub fn resource(&self, name: &str) -> Self {
        self.child("resource", name)
    }

    #[must_use]
    pub fn form_field(&self, name: &str) -> Self {
        self.child("form-field", name)
    }

    #[must_use]
    pub fn form_declaration_candidate_in_module(
        module_path: impl AsRef<Path>,
        position: usize,
    ) -> Self {
        Self(format!(
            "module:{}/form-declaration:{position}",
            normalized_module_path(module_path.as_ref())
        ))
    }

    #[must_use]
    pub fn form_field_declaration_candidate_in_module(
        module_path: impl AsRef<Path>,
        position: usize,
    ) -> Self {
        Self(format!(
            "module:{}/form-field-declaration:{position}",
            normalized_module_path(module_path.as_ref())
        ))
    }

    #[must_use]
    pub fn form_field_binding_candidate_in_module(
        module_path: impl AsRef<Path>,
        position: usize,
    ) -> Self {
        Self(format!(
            "module:{}/form-field-binding-candidate:{position}",
            normalized_module_path(module_path.as_ref())
        ))
    }

    #[must_use]
    pub fn validation_rule_candidate_in_module(
        module_path: impl AsRef<Path>,
        position: usize,
    ) -> Self {
        Self(format!(
            "module:{}/validation-rule-candidate:{position}",
            normalized_module_path(module_path.as_ref())
        ))
    }

    #[must_use]
    pub fn submission_declaration_candidate_in_module(
        module_path: impl AsRef<Path>,
        position: usize,
    ) -> Self {
        Self(format!(
            "module:{}/submission-declaration-candidate:{position}",
            normalized_module_path(module_path.as_ref())
        ))
    }

    #[must_use]
    pub fn field(&self, name: &str) -> Self {
        self.child("field", name)
    }

    #[must_use]
    pub fn validation_rule(&self, ordinal: usize) -> Self {
        self.child("validation-rule", &ordinal.to_string())
    }

    #[must_use]
    pub fn validation_plan(&self) -> Self {
        Self(format!("{}/validation-plan", self.as_str()))
    }

    #[must_use]
    pub fn field_dependency(&self, source_field: &Self) -> Self {
        self.child("field-dependency", source_field.as_str())
    }

    #[must_use]
    pub fn dirty_tracking_plan(&self) -> Self {
        Self(format!("{}/dirty-plan", self.as_str()))
    }

    #[must_use]
    pub fn touched_tracking_plan(&self) -> Self {
        Self(format!("{}/touched-plan", self.as_str()))
    }

    #[must_use]
    pub fn field_tracking(&self) -> Self {
        Self(format!("{}/tracking", self.as_str()))
    }

    #[must_use]
    pub fn submission_plan(&self) -> Self {
        Self(format!("{}/submission-plan", self.as_str()))
    }

    #[must_use]
    pub fn serialization_plan(&self) -> Self {
        Self(format!("{}/serialization-plan", self.as_str()))
    }

    #[must_use]
    pub fn reset_plan(&self) -> Self {
        Self(format!("{}/reset-plan", self.as_str()))
    }

    #[must_use]
    pub fn field_reset_operation(&self, field: &Self) -> Self {
        self.child("field-reset", field.as_str())
    }

    #[must_use]
    pub fn form_instance(&self, form: &str) -> Self {
        self.child("form-instance", form)
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
    pub fn opaque_activation(&self, method: &str) -> Self {
        self.child("opaque-activation", method)
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

    /// Compiler-issued identity for one automatic file-route layout edge.
    #[must_use]
    pub fn layout_composition_invocation(&self, route: &Self, position: usize) -> Self {
        self.child(
            "layout-composition",
            &format!("{}:{position}", route.as_str()),
        )
    }

    #[must_use]
    pub fn template_position(&self) -> Self {
        self.child("template-position", "authored")
    }

    #[must_use]
    pub fn field_binding(&self, field: &str) -> Self {
        self.child("field-binding", field)
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
    pub fn slot_binding(&self, binding_key: &str) -> Self {
        self.child("slot-binding", binding_key)
    }

    #[must_use]
    pub fn component_instance_invocation(&self, invocation: &str) -> Self {
        self.child("invocation", invocation)
    }

    #[must_use]
    pub fn component_structural_region(&self, kind: &str) -> Self {
        self.child("component-structural-region", kind)
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

impl FormId {
    #[must_use]
    pub fn for_owner(owner: &SemanticId, name: &str) -> Self {
        Self(owner.form(name))
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

impl ResourceId {
    #[must_use]
    pub fn for_owner(owner: &SemanticId, name: &str) -> Self {
        Self(owner.resource(name))
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

impl ResourceActivationId {
    #[must_use]
    pub fn for_component_instance(instance: &ComponentInstanceId, resource: &ResourceId) -> Self {
        Self(
            instance
                .as_semantic_id()
                .child("resource-activation", resource.as_str()),
        )
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

impl FormInstanceId {
    #[must_use]
    pub fn for_component_instance(instance: &ComponentInstanceId, form: &FormId) -> Self {
        Self(instance.as_semantic_id().form_instance(form.as_str()))
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

impl FormDeclarationCandidateId {
    #[must_use]
    pub fn for_source_position(module_path: impl AsRef<Path>, position: usize) -> Self {
        Self(SemanticId::form_declaration_candidate_in_module(
            module_path,
            position,
        ))
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

impl FormFieldDeclarationCandidateId {
    #[must_use]
    pub fn for_source_position(module_path: impl AsRef<Path>, position: usize) -> Self {
        Self(SemanticId::form_field_declaration_candidate_in_module(
            module_path,
            position,
        ))
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

impl FormFieldBindingCandidateId {
    #[must_use]
    pub fn for_source_position(module_path: impl AsRef<Path>, position: usize) -> Self {
        Self(SemanticId::form_field_binding_candidate_in_module(
            module_path,
            position,
        ))
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

impl FieldId {
    #[must_use]
    pub fn for_form(form: &FormId, name: &str) -> Self {
        Self(form.as_semantic_id().field(name))
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

impl FieldBindingId {
    #[must_use]
    pub fn for_control(control: &SemanticId, field: &FieldId) -> Self {
        Self(control.field_binding(field.as_str()))
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

impl SubmissionHostCandidateId {
    #[must_use]
    pub fn for_source_position(module_path: impl AsRef<Path>, position: usize) -> Self {
        Self(
            SemanticId::form_field_binding_candidate_in_module(module_path, position)
                .child("submission-host-candidate", "host"),
        )
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

impl SubmissionHostId {
    #[must_use]
    pub fn for_element(element: &SemanticId, form: &FormId) -> Self {
        Self(element.child("submission-host", form.as_str()))
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

impl FormOwnershipGraphId {
    /// Derives one product identity from the existing canonical build-root
    /// identities. The root set is sorted and deduplicated so file input order
    /// cannot affect the result.
    #[must_use]
    pub fn for_build_roots<'a>(roots: impl IntoIterator<Item = &'a ComponentRootId>) -> Self {
        let mut roots = roots
            .into_iter()
            .map(ComponentRootId::as_str)
            .collect::<Vec<_>>();
        roots.sort_unstable();
        roots.dedup();
        let authority = if roots.is_empty() {
            "application".to_string()
        } else {
            roots.join("+")
        };
        Self(SemanticId(format!("form-ownership-graph:{authority}")))
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

impl ValidationRuleCandidateId {
    #[must_use]
    pub fn for_source_position(module_path: impl AsRef<Path>, position: usize) -> Self {
        Self(SemanticId::validation_rule_candidate_in_module(
            module_path,
            position,
        ))
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

impl ValidationRuleId {
    #[must_use]
    pub fn for_field(field: &FieldId, authored_ordinal: usize) -> Self {
        Self(field.as_semantic_id().validation_rule(authored_ordinal))
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

impl ValidationGraphId {
    #[must_use]
    pub fn for_build_roots<'a>(roots: impl IntoIterator<Item = &'a ComponentRootId>) -> Self {
        let mut roots = roots
            .into_iter()
            .map(ComponentRootId::as_str)
            .collect::<Vec<_>>();
        roots.sort_unstable();
        roots.dedup();
        let authority = if roots.is_empty() {
            "application".to_string()
        } else {
            roots.join("+")
        };
        Self(SemanticId(format!("validation-graph:{authority}")))
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

impl ValidationPlanId {
    #[must_use]
    pub fn for_form(form: &FormId) -> Self {
        Self(form.as_semantic_id().validation_plan())
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

impl FieldDependencyId {
    #[must_use]
    pub fn for_rule_and_source(rule: &ValidationRuleId, source_field: &FieldId) -> Self {
        Self(
            rule.as_semantic_id()
                .field_dependency(source_field.as_semantic_id()),
        )
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

impl DirtyTrackingPlanId {
    #[must_use]
    pub fn for_form(form: &FormId) -> Self {
        Self(form.as_semantic_id().dirty_tracking_plan())
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

impl TouchedTrackingPlanId {
    #[must_use]
    pub fn for_form(form: &FormId) -> Self {
        Self(form.as_semantic_id().touched_tracking_plan())
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

impl FieldTrackingId {
    #[must_use]
    pub fn for_field(field: &FieldId) -> Self {
        Self(field.as_semantic_id().field_tracking())
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

impl SubmissionDeclarationCandidateId {
    #[must_use]
    pub fn for_source_position(module_path: impl AsRef<Path>, position: usize) -> Self {
        Self(SemanticId::submission_declaration_candidate_in_module(
            module_path,
            position,
        ))
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

impl SubmissionPlanId {
    #[must_use]
    pub fn for_form(form: &FormId) -> Self {
        Self(form.as_semantic_id().submission_plan())
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

impl SerializationPlanId {
    #[must_use]
    pub fn for_form(form: &FormId) -> Self {
        Self(form.as_semantic_id().serialization_plan())
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

impl ResetPlanId {
    #[must_use]
    pub fn for_form(form: &FormId) -> Self {
        Self(form.as_semantic_id().reset_plan())
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

impl FieldResetOperationId {
    #[must_use]
    pub fn for_plan_and_field(plan: &ResetPlanId, field: &FieldId) -> Self {
        Self(
            plan.as_semantic_id()
                .field_reset_operation(field.as_semantic_id()),
        )
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

macro_rules! impl_form_slot_id {
    ($name:ident) => {
        impl $name {
            #[must_use]
            pub fn for_instance(instance: &FormInstanceId, name: &str) -> Self {
                Self(
                    instance
                        .as_semantic_id()
                        .child("form-slot", &format!("{}:{name}", stringify!($name))),
                )
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
    };
}
impl_form_slot_id!(FormFieldValueSlotId);
impl_form_slot_id!(FormFieldDirtySlotId);
impl_form_slot_id!(FormFieldTouchedSlotId);
impl_form_slot_id!(FormFieldValidationSlotId);
impl_form_slot_id!(FormValidationAggregateSlotId);
impl_form_slot_id!(FormSubmissionStateSlotId);

impl ValidationDependencyCycleId {
    #[must_use]
    pub fn for_fields(form: &FormId, fields: &[FieldId]) -> Self {
        let mut fields = fields.iter().map(FieldId::as_str).collect::<Vec<_>>();
        fields.sort_unstable();
        fields.dedup();
        Self(SemanticId(format!(
            "validation-cycle:{}/{}",
            form.as_str(),
            fields.join("+")
        )))
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

    /// Builds a virtual compiler-only invocation ID for automatic file-route
    /// layout composition. It is intentionally distinct from authored JSX.
    #[must_use]
    pub fn for_layout_composition(
        layout: &SemanticId,
        route_component: &SemanticId,
        position: usize,
    ) -> Self {
        Self(layout.layout_composition_invocation(route_component, position))
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

impl SlotBindingId {
    #[must_use]
    pub fn for_instance(instance: &ComponentInstanceId, binding_key: &str) -> Self {
        Self(instance.as_semantic_id().slot_binding(binding_key))
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

impl ComponentRootId {
    #[must_use]
    pub fn for_component(component: &SemanticId) -> Self {
        Self(SemanticId::component_root(component))
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

impl ComponentInstanceId {
    #[must_use]
    pub fn for_root(root: &ComponentRootId) -> Self {
        Self(root.as_semantic_id().clone())
    }

    #[must_use]
    pub fn for_invocation(parent: &Self, invocation: &ComponentInvocationId) -> Self {
        Self(
            parent
                .as_semantic_id()
                .component_instance_invocation(invocation.as_str()),
        )
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

impl ComponentStructuralRegionId {
    #[must_use]
    pub fn for_template_entity(entity: &SemanticId, kind: &str) -> Self {
        Self(entity.component_structural_region(kind))
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

impl fmt::Display for FormId {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(formatter)
    }
}

impl fmt::Display for FormInstanceId {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(formatter)
    }
}

impl fmt::Display for FormDeclarationCandidateId {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(formatter)
    }
}

impl fmt::Display for FormFieldDeclarationCandidateId {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(formatter)
    }
}

impl fmt::Display for FormFieldBindingCandidateId {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(formatter)
    }
}

impl fmt::Display for FieldId {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(formatter)
    }
}

impl fmt::Display for FieldBindingId {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(formatter)
    }
}

impl fmt::Display for SubmissionHostCandidateId {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(formatter)
    }
}
impl fmt::Display for SubmissionHostId {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(formatter)
    }
}

impl fmt::Display for FormOwnershipGraphId {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(formatter)
    }
}

impl fmt::Display for ValidationRuleCandidateId {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(formatter)
    }
}

impl fmt::Display for ValidationRuleId {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(formatter)
    }
}

impl fmt::Display for ValidationGraphId {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(formatter)
    }
}

impl fmt::Display for ValidationPlanId {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(formatter)
    }
}

impl fmt::Display for FieldDependencyId {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(formatter)
    }
}

impl fmt::Display for ValidationDependencyCycleId {
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

impl fmt::Display for SlotBindingId {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(formatter)
    }
}

impl fmt::Display for ComponentRootId {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(formatter)
    }
}

impl fmt::Display for ComponentInstanceId {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(formatter)
    }
}

impl fmt::Display for ComponentStructuralRegionId {
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
        ComponentInstanceId, ComponentInvocationId, ConsumerId, ContextId, FieldDependencyId,
        FieldId, FormId, FormInstanceId, ProviderId, SemanticId, SemanticOwner, SlotId,
        TemplatePositionId, ValidationPlanId, ValidationRuleId,
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
    fn derives_distinct_form_definition_instance_and_field_identities() {
        let component =
            SemanticId::component_in_module("src/Profile.tsx", Some("x-profile"), "Profile");
        let component_instance = serde_json::from_str::<ComponentInstanceId>(
            r#""root:module:src/Profile.tsx/component:x-profile""#,
        )
        .expect("canonical component instance id");
        let form = FormId::for_owner(&component, "profile");
        let instance = FormInstanceId::for_component_instance(&component_instance, &form);
        let nested_component_instance = serde_json::from_str::<ComponentInstanceId>(
            r#""root:module:src/Profile.tsx/component:x-profile/invocation:nested""#,
        )
        .expect("canonical nested component instance id");
        let nested_instance =
            FormInstanceId::for_component_instance(&nested_component_instance, &form);
        let field = FieldId::for_form(&form, "email");
        let rule = ValidationRuleId::for_field(&field, 2);
        let plan = ValidationPlanId::for_form(&form);
        let dependency = FieldDependencyId::for_rule_and_source(&rule, &field);

        assert_eq!(
            form.as_str(),
            "module:src/Profile.tsx/component:x-profile/form:profile"
        );
        assert_eq!(
            field.as_str(),
            "module:src/Profile.tsx/component:x-profile/form:profile/field:email"
        );
        assert_eq!(
            instance.as_str(),
            "root:module:src/Profile.tsx/component:x-profile/form-instance:module:src/Profile.tsx/component:x-profile/form:profile"
        );
        assert_ne!(form.as_semantic_id(), field.as_semantic_id());
        assert_ne!(form.as_semantic_id(), instance.as_semantic_id());
        assert_ne!(field.as_semantic_id(), instance.as_semantic_id());
        assert_ne!(instance, nested_instance);
        assert_eq!(
            plan.as_str(),
            "module:src/Profile.tsx/component:x-profile/form:profile/validation-plan"
        );
        assert_eq!(
            dependency.as_str(),
            "module:src/Profile.tsx/component:x-profile/form:profile/field:email/validation-rule:2/field-dependency:module:src/Profile.tsx/component:x-profile/form:profile/field:email"
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
