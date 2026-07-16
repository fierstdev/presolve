use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

use ezc_parser::ParsedFile;

use crate::compilation_unit::CompilationUnit;
use crate::component_composition::{analyze_component_composition, ComponentCompositionAnalysis};
use crate::component_graph::{
    build_component_graph_for_module, render_event_handlers, ComponentAction, ComponentDiagnostic,
    ComponentDiagnosticSeverity, ComponentMethod, ComponentNode, MethodLocalVariable,
    RenderEventHandler, StateField,
};
use crate::component_initialization::{plan_component_initialization, ComponentInitializationPlan};
use crate::component_instance::{
    plan_component_instances, BlockedComponentInstancePlan, ComponentInstance,
    ComponentInstancePlan,
};
use crate::component_instance_scope::{
    build_component_instance_scope_graph, ComponentInstanceScopeGraph,
};
use crate::component_invocation::{collect_component_invocations, ComponentInvocationEntity};
use crate::component_ir::{lower_component_ir, ComponentIrReport};
use crate::component_ir_optimization::{optimize_component_ir, OptimizedComponentIrReport};
use crate::component_scope::ComponentScopeGraph;
use crate::composition_typing::{collect_composition_type_products, CompositionTypeProducts};
use crate::computed_value::{collect_computed_values, ComputedDiagnosticCode, ComputedValue};
use crate::consumer::{collect_consumer_entities, ConsumerEntity, ContextResolutionState};
use crate::context::{collect_context_entities, ContextEntity};
use crate::context_declaration_candidate::{
    collect_context_declaration_candidates, ContextDeclarationCandidateRegistry,
};
use crate::context_dependency::{collect_context_dependency_graph, ContextDependencyGraph};
use crate::context_evaluation::{collect_context_evaluation_plan, ContextEvaluationPlan};
use crate::context_lifetime::{collect_context_lifetime_analysis, ContextLifetimeAnalysis};
use crate::context_ownership::{collect_context_ownership_graph, ContextOwnershipGraph};
use crate::context_resolution::{
    collect_context_resolutions, ContextResolution, ContextResolutionResult,
};
use crate::context_typing::{
    collect_context_type_products, ConsumerTypeRecord, ContextBindingCompatibility,
    ContextBindingTypeRecord, ContextTypeRecord, ProviderTypeRecord,
};
use crate::expression_graph::{ExpressionGraph, ExpressionNode, ExpressionNodeKind};
use crate::form::{collect_form_entities, FormEntity};
use crate::form_binding::{
    collect_form_field_binding_products, FormFieldBinding, FormFieldBindingCandidate,
};
use crate::form_field::{collect_form_field_products, FormFieldEntity};
use crate::form_ir::{lower_form_ir, FormIrReport};
use crate::form_ownership::{collect_form_ownership_graph, FormOwnershipGraph};
use crate::form_reset::{collect_reset_products, ResetProducts};
use crate::form_serialization::{collect_serialization_products, SerializationProducts};
use crate::form_submission::{collect_submission_products, SubmissionProducts};
use crate::form_tracking::{collect_form_tracking_products, FormTrackingProducts};
use crate::form_validation::{
    collect_validation_graph, collect_validation_products, ValidationGraph, ValidationRule,
    ValidationRuleCandidate,
};
use crate::form_validation_plan::{collect_validation_dependency_plans, ValidationDependencyPlans};
use crate::instance_context::{collect_instance_context_registry, InstanceContextRegistry};
use crate::intermediate_representation::{
    IrComputedEvaluationPlan, IrReactiveCycleAnalysis, IrReactiveGraph,
    IrReactiveTransitiveAnalysis,
};
use crate::provider::{collect_provider_entities, DuplicateProviderDeclaration, ProviderEntity};
use crate::semantic_id::{
    ComponentInvocationId, ConsumerId, ContextId, FieldBindingId, FieldId, FormId, ProviderId,
    SemanticId, SemanticOwner, SlotBindingId, SlotContentFragmentId, SlotId, SlotOutletId,
    ValidationRuleId,
};
use crate::semantic_provenance::SourceProvenance;
use crate::semantic_reference::{SemanticReference, SemanticReferenceKind};
use crate::semantic_type::{EffectStatementTypeRecord, SemanticTypeModel};
use crate::slot::{collect_slot_entities, SlotEntity};
use crate::slot_binding::{collect_slot_bindings, SlotBinding, SlotBindingRegistry};
use crate::slot_content::{collect_slot_composition, SlotContentFragment, SlotOutlet};
use crate::template_graph::{build_template_graph, TemplateNode};
use crate::template_semantics::{
    build_template_semantic_entities, TemplateSemanticEntity, TemplateSemanticKind,
    TemplateSemanticScope,
};
use crate::{
    analyze_effect_reactivity, collect_effects, derive_effect_trigger_plan, lower_effect_bodies,
    plan_effect_execution, validate_effects, Effect, EffectBody, EffectExecutionPlan,
    EffectReactiveAnalysis, EffectStatement, EffectTriggerPlan,
};

/// Application-level semantic data assembled from the compiler's existing graphs.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ApplicationSemanticModel {
    pub expression_graph: ExpressionGraph,
    pub semantic_types: SemanticTypeModel,
    pub components: Vec<ComponentNode>,
    pub contexts: BTreeMap<ContextId, ContextEntity>,
    pub providers: BTreeMap<ProviderId, ProviderEntity>,
    pub consumers: BTreeMap<ConsumerId, ConsumerEntity>,
    pub forms: BTreeMap<FormId, FormEntity>,
    pub form_field_declaration_candidates: Vec<crate::FormFieldDeclarationCandidate>,
    pub form_fields: BTreeMap<FieldId, FormFieldEntity>,
    pub form_field_binding_candidates: Vec<FormFieldBindingCandidate>,
    pub form_field_bindings: BTreeMap<FieldBindingId, FormFieldBinding>,
    pub form_ownership: FormOwnershipGraph,
    pub validation_rule_candidates: Vec<ValidationRuleCandidate>,
    pub validation_rules: BTreeMap<ValidationRuleId, ValidationRule>,
    pub validation_graph: ValidationGraph,
    pub validation_dependency_plans: ValidationDependencyPlans,
    pub form_tracking: FormTrackingProducts,
    pub submissions: SubmissionProducts,
    pub serialization: SerializationProducts,
    pub reset: ResetProducts,
    pub form_ir: FormIrReport,
    pub slots: BTreeMap<SlotId, SlotEntity>,
    pub component_invocations: BTreeMap<ComponentInvocationId, ComponentInvocationEntity>,
    pub component_instance_plan: ComponentInstancePlan,
    pub component_instance_scope: ComponentInstanceScopeGraph,
    pub component_composition: ComponentCompositionAnalysis,
    pub component_initialization: ComponentInitializationPlan,
    pub component_ir: ComponentIrReport,
    pub component_ir_optimization: OptimizedComponentIrReport,
    pub instance_context: InstanceContextRegistry,
    pub slot_content_fragments: BTreeMap<SlotContentFragmentId, SlotContentFragment>,
    pub slot_outlets: BTreeMap<SlotOutletId, SlotOutlet>,
    pub slot_bindings: SlotBindingRegistry,
    pub composition_types: CompositionTypeProducts,
    pub context_declaration_candidates: ContextDeclarationCandidateRegistry,
    pub context_ownership: ContextOwnershipGraph,
    pub context_dependency: ContextDependencyGraph,
    pub context_lifetime: ContextLifetimeAnalysis,
    pub context_evaluation: ContextEvaluationPlan,
    pub component_scope: ComponentScopeGraph,
    pub context_resolutions: BTreeMap<ConsumerId, ContextResolution>,
    pub context_types: BTreeMap<ContextId, ContextTypeRecord>,
    pub provider_types: BTreeMap<ProviderId, ProviderTypeRecord>,
    pub consumer_types: BTreeMap<ConsumerId, ConsumerTypeRecord>,
    pub context_binding_types: BTreeMap<ConsumerId, ContextBindingTypeRecord>,
    pub duplicate_provider_declarations: Vec<DuplicateProviderDeclaration>,
    pub computed_values: BTreeMap<SemanticId, ComputedValue>,
    pub effects: BTreeMap<SemanticId, Effect>,
    pub effect_reactive_analysis: BTreeMap<SemanticId, EffectReactiveAnalysis>,
    pub effect_trigger_plan: EffectTriggerPlan,
    pub effect_execution_plan: EffectExecutionPlan,
    pub effect_bodies: BTreeMap<SemanticId, EffectBody>,
    pub effect_statements: BTreeMap<SemanticId, EffectStatement>,
    pub reactive_graph: IrReactiveGraph,
    pub reactive_transitive_analysis: IrReactiveTransitiveAnalysis,
    pub reactive_cycle_analysis: IrReactiveCycleAnalysis,
    pub computed_evaluation_plan: IrComputedEvaluationPlan,
    pub templates: Vec<TemplateNode>,
    pub template_entities: Vec<TemplateSemanticEntity>,
    pub diagnostics: Vec<ComponentDiagnostic>,
    pub ownership: BTreeMap<SemanticId, SemanticOwner>,
    pub references: Vec<SemanticReference>,
    pub provenance: BTreeMap<SemanticId, SourceProvenance>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SemanticEntity<'a> {
    Component(&'a ComponentNode),
    StateField(&'a StateField),
    Method(&'a ComponentMethod),
    Context(&'a ContextEntity),
    Provider(&'a ProviderEntity),
    Consumer(&'a ConsumerEntity),
    Form(&'a FormEntity),
    FormField(&'a FormFieldEntity),
    FormFieldBinding(&'a FormFieldBinding),
    ValidationRule(&'a ValidationRule),
    Slot(&'a SlotEntity),
    ComponentInvocation(&'a ComponentInvocationEntity),
    ComponentInstance(&'a ComponentInstance),
    BlockedComponentInstance(&'a BlockedComponentInstancePlan),
    SlotContentFragment(&'a SlotContentFragment),
    SlotOutlet(&'a SlotOutlet),
    Computed(&'a ComputedValue),
    Effect(&'a Effect),
    Parameter(&'a crate::MethodParameter),
    LocalVariable(&'a MethodLocalVariable),
    Action(&'a ComponentAction),
    EventHandler(&'a RenderEventHandler),
    Template(&'a TemplateNode),
    TemplateEntity(&'a TemplateSemanticEntity),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SemanticEntityKind {
    Component,
    StateField,
    Method,
    Context,
    Provider,
    Consumer,
    Form,
    FormField,
    FormFieldBinding,
    ValidationRule,
    Slot,
    ComponentInvocation,
    ComponentInstance,
    BlockedComponentInstance,
    SlotContentFragment,
    SlotOutlet,
    Computed,
    Effect,
    Parameter,
    LocalVariable,
    Action,
    EventHandler,
    Template,
    TemplateEntity,
}

impl SemanticEntity<'_> {
    #[must_use]
    pub fn kind(self) -> SemanticEntityKind {
        match self {
            Self::Component(_) => SemanticEntityKind::Component,
            Self::StateField(_) => SemanticEntityKind::StateField,
            Self::Method(_) => SemanticEntityKind::Method,
            Self::Context(_) => SemanticEntityKind::Context,
            Self::Provider(_) => SemanticEntityKind::Provider,
            Self::Consumer(_) => SemanticEntityKind::Consumer,
            Self::Form(_) => SemanticEntityKind::Form,
            Self::FormField(_) => SemanticEntityKind::FormField,
            Self::FormFieldBinding(_) => SemanticEntityKind::FormFieldBinding,
            Self::ValidationRule(_) => SemanticEntityKind::ValidationRule,
            Self::Slot(_) => SemanticEntityKind::Slot,
            Self::ComponentInvocation(_) => SemanticEntityKind::ComponentInvocation,
            Self::ComponentInstance(_) => SemanticEntityKind::ComponentInstance,
            Self::BlockedComponentInstance(_) => SemanticEntityKind::BlockedComponentInstance,
            Self::SlotContentFragment(_) => SemanticEntityKind::SlotContentFragment,
            Self::SlotOutlet(_) => SemanticEntityKind::SlotOutlet,
            Self::Computed(_) => SemanticEntityKind::Computed,
            Self::Effect(_) => SemanticEntityKind::Effect,
            Self::Parameter(_) => SemanticEntityKind::Parameter,
            Self::LocalVariable(_) => SemanticEntityKind::LocalVariable,
            Self::Action(_) => SemanticEntityKind::Action,
            Self::EventHandler(_) => SemanticEntityKind::EventHandler,
            Self::Template(_) => SemanticEntityKind::Template,
            Self::TemplateEntity(_) => SemanticEntityKind::TemplateEntity,
        }
    }
}

impl ApplicationSemanticModel {
    #[allow(clippy::too_many_lines)]
    #[must_use]
    pub fn entity(&self, id: &SemanticId) -> Option<SemanticEntity<'_>> {
        for component in &self.components {
            if component.id == *id {
                return Some(SemanticEntity::Component(component));
            }
            if let Some(field) = component.state_fields.iter().find(|field| field.id == *id) {
                return Some(SemanticEntity::StateField(field));
            }
            if let Some(method) = component.methods.iter().find(|method| method.id == *id) {
                return Some(SemanticEntity::Method(method));
            }
            if let Some(parameter) = component
                .methods
                .iter()
                .flat_map(|method| method.parameters.iter())
                .find(|parameter| parameter.id == *id)
            {
                return Some(SemanticEntity::Parameter(parameter));
            }
            if let Some(local) = component
                .methods
                .iter()
                .flat_map(|method| method.local_variables.iter())
                .find(|local| local.id == *id)
            {
                return Some(SemanticEntity::LocalVariable(local));
            }
            if let Some(action) = component.actions.iter().find(|action| action.id == *id) {
                return Some(SemanticEntity::Action(action));
            }
            if let Some(render) = &component.render {
                if let Some(handler) = render_event_handlers(render)
                    .into_iter()
                    .find(|handler| handler.id == *id)
                {
                    return Some(SemanticEntity::EventHandler(handler));
                }
            }
        }

        if let Some(computed) = self.computed_values.get(id) {
            return Some(SemanticEntity::Computed(computed));
        }
        if let Some(context) = self
            .contexts
            .values()
            .find(|context| context.id.as_semantic_id() == id)
        {
            return Some(SemanticEntity::Context(context));
        }
        if let Some(provider) = self
            .providers
            .values()
            .find(|provider| provider.id.as_semantic_id() == id)
        {
            return Some(SemanticEntity::Provider(provider));
        }
        if let Some(consumer) = self
            .consumers
            .values()
            .find(|consumer| consumer.id.as_semantic_id() == id)
        {
            return Some(SemanticEntity::Consumer(consumer));
        }
        if let Some(form) = self
            .forms
            .values()
            .find(|form| form.id.as_semantic_id() == id)
        {
            return Some(SemanticEntity::Form(form));
        }
        if let Some(field) = self
            .form_fields
            .values()
            .find(|field| field.id.as_semantic_id() == id)
        {
            return Some(SemanticEntity::FormField(field));
        }
        if let Some(binding) = self
            .form_field_bindings
            .values()
            .find(|binding| binding.id.as_semantic_id() == id)
        {
            return Some(SemanticEntity::FormFieldBinding(binding));
        }
        if let Some(rule) = self
            .validation_rules
            .values()
            .find(|rule| rule.id.as_semantic_id() == id)
        {
            return Some(SemanticEntity::ValidationRule(rule));
        }
        if let Some(slot) = self
            .slots
            .values()
            .find(|slot| slot.id.as_semantic_id() == id)
        {
            return Some(SemanticEntity::Slot(slot));
        }
        if let Some(invocation) = self
            .component_invocations
            .values()
            .find(|invocation| invocation.id.as_semantic_id() == id)
        {
            return Some(SemanticEntity::ComponentInvocation(invocation));
        }
        if let Some(instance) = self
            .component_instance_plan
            .instances
            .values()
            .find(|instance| instance.id.as_semantic_id() == id)
        {
            return Some(SemanticEntity::ComponentInstance(instance));
        }
        if let Some(blocked) = self
            .component_instance_plan
            .blocked
            .values()
            .find(|blocked| blocked.id.as_semantic_id() == id)
        {
            return Some(SemanticEntity::BlockedComponentInstance(blocked));
        }
        if let Some(fragment) = self
            .slot_content_fragments
            .values()
            .find(|fragment| fragment.id.as_semantic_id() == id)
        {
            return Some(SemanticEntity::SlotContentFragment(fragment));
        }
        if let Some(outlet) = self
            .slot_outlets
            .values()
            .find(|outlet| outlet.id.as_semantic_id() == id)
        {
            return Some(SemanticEntity::SlotOutlet(outlet));
        }
        if let Some(effect) = self.effects.get(id) {
            return Some(SemanticEntity::Effect(effect));
        }

        if let Some(template) = self.templates.iter().find(|template| template.id == *id) {
            return Some(SemanticEntity::Template(template));
        }

        self.template_entities
            .iter()
            .find(|entity| entity.id == *id)
            .map(SemanticEntity::TemplateEntity)
    }

    #[must_use]
    pub fn component(&self, id: &SemanticId) -> Option<&ComponentNode> {
        self.components.iter().find(|component| component.id == *id)
    }

    #[must_use]
    pub fn template(&self, id: &SemanticId) -> Option<&TemplateNode> {
        self.templates.iter().find(|template| template.id == *id)
    }

    #[must_use]
    pub fn computed_value(&self, id: &SemanticId) -> Option<&ComputedValue> {
        self.computed_values.get(id)
    }

    #[must_use]
    pub fn contexts(&self) -> Vec<&ContextEntity> {
        self.contexts.values().collect()
    }

    #[must_use]
    pub fn context(&self, id: &ContextId) -> Option<&ContextEntity> {
        self.contexts.get(id)
    }

    #[must_use]
    pub fn forms(&self) -> Vec<&FormEntity> {
        self.forms.values().collect()
    }

    #[must_use]
    pub fn form(&self, id: &FormId) -> Option<&FormEntity> {
        self.forms.get(id)
    }

    #[must_use]
    pub fn form_fields(&self) -> Vec<&FormFieldEntity> {
        let mut fields = self.form_fields.values().collect::<Vec<_>>();
        fields.sort_by(|left, right| {
            (&left.owner_form, left.declaration_order, &left.id).cmp(&(
                &right.owner_form,
                right.declaration_order,
                &right.id,
            ))
        });
        fields
    }

    #[must_use]
    pub fn form_field(&self, id: &FieldId) -> Option<&FormFieldEntity> {
        self.form_fields.get(id)
    }

    #[must_use]
    pub fn form_field_declaration_candidates(&self) -> &[crate::FormFieldDeclarationCandidate] {
        &self.form_field_declaration_candidates
    }

    #[must_use]
    pub fn form_field_binding_candidates(&self) -> &[FormFieldBindingCandidate] {
        &self.form_field_binding_candidates
    }

    #[must_use]
    pub fn form_field_bindings(&self) -> Vec<&FormFieldBinding> {
        let mut bindings = self.form_field_bindings.values().collect::<Vec<_>>();
        bindings.sort_by(|left, right| {
            (&left.owner_template, left.authored_order, &left.id).cmp(&(
                &right.owner_template,
                right.authored_order,
                &right.id,
            ))
        });
        bindings
    }

    #[must_use]
    pub fn form_field_binding(&self, id: &FieldBindingId) -> Option<&FormFieldBinding> {
        self.form_field_bindings.get(id)
    }

    #[must_use]
    pub const fn form_ownership(&self) -> &FormOwnershipGraph {
        &self.form_ownership
    }

    #[must_use]
    pub fn validation_rule_candidates(&self) -> &[ValidationRuleCandidate] {
        &self.validation_rule_candidates
    }

    #[must_use]
    pub fn validation_candidates_of(&self, field: &FieldId) -> Vec<&ValidationRuleCandidate> {
        self.validation_rule_candidates
            .iter()
            .filter(|candidate| candidate.target_field.as_ref() == Some(field))
            .collect()
    }

    #[must_use]
    pub fn validation_rule(&self, id: &ValidationRuleId) -> Option<&ValidationRule> {
        self.validation_rules.get(id)
    }

    #[must_use]
    pub fn validation_rules(&self) -> Vec<&ValidationRule> {
        let mut rules = self.validation_rules.values().collect::<Vec<_>>();
        rules.sort_by(|left, right| {
            (
                &left.owner_form,
                left.field_authored_order,
                left.rule_authored_order,
                &left.id,
            )
                .cmp(&(
                    &right.owner_form,
                    right.field_authored_order,
                    right.rule_authored_order,
                    &right.id,
                ))
        });
        rules
    }

    #[must_use]
    pub const fn validation_graph(&self) -> &ValidationGraph {
        &self.validation_graph
    }

    #[must_use]
    pub const fn validation_dependency_plans(&self) -> &ValidationDependencyPlans {
        &self.validation_dependency_plans
    }

    #[must_use]
    pub const fn form_tracking(&self) -> &FormTrackingProducts {
        &self.form_tracking
    }

    #[must_use]
    pub const fn submissions(&self) -> &SubmissionProducts {
        &self.submissions
    }

    #[must_use]
    pub const fn serialization(&self) -> &SerializationProducts {
        &self.serialization
    }

    #[must_use]
    pub const fn reset(&self) -> &ResetProducts {
        &self.reset
    }

    #[must_use]
    pub fn form_declaration_candidates(&self) -> Vec<&crate::FormDeclarationCandidate> {
        let mut candidates = self
            .components
            .iter()
            .flat_map(|component| component.form_declaration_candidates.iter())
            .collect::<Vec<_>>();
        candidates.sort_by(|left, right| {
            (
                left.provenance.path.as_path(),
                left.provenance.span.start,
                left.provenance.span.end,
                left.id.as_str(),
            )
                .cmp(&(
                    right.provenance.path.as_path(),
                    right.provenance.span.start,
                    right.provenance.span.end,
                    right.id.as_str(),
                ))
        });
        candidates
    }

    #[must_use]
    pub const fn context_declaration_candidates(&self) -> &ContextDeclarationCandidateRegistry {
        &self.context_declaration_candidates
    }

    #[must_use]
    pub fn contexts_owned_by(&self, component: &SemanticId) -> Vec<&ContextId> {
        self.contexts
            .iter()
            .filter_map(|(id, context)| {
                (context.owner.entity_id() == Some(component)).then_some(id)
            })
            .collect()
    }

    #[must_use]
    pub fn context_for_authored_field(&self, field: &SemanticId) -> Option<&ContextId> {
        self.contexts
            .iter()
            .find_map(|(id, context)| (&context.authored_field == field).then_some(id))
    }

    #[must_use]
    pub fn providers(&self) -> Vec<&ProviderEntity> {
        self.providers.values().collect()
    }

    #[must_use]
    pub fn provider(&self, id: &ProviderId) -> Option<&ProviderEntity> {
        self.providers.get(id)
    }

    #[must_use]
    pub fn providers_owned_by(&self, component: &SemanticId) -> Vec<&ProviderId> {
        self.providers
            .iter()
            .filter_map(|(id, provider)| {
                (provider.owner.entity_id() == Some(component)).then_some(id)
            })
            .collect()
    }

    #[must_use]
    pub fn providers_for_context(&self, context: &ContextId) -> Vec<&ProviderId> {
        self.providers
            .iter()
            .filter_map(|(id, provider)| (&provider.context == context).then_some(id))
            .collect()
    }

    #[must_use]
    pub fn provider_for_authored_field(&self, field: &SemanticId) -> Option<&ProviderId> {
        self.providers
            .iter()
            .find_map(|(id, provider)| (&provider.authored_field == field).then_some(id))
    }

    #[must_use]
    pub fn consumers(&self) -> Vec<&ConsumerEntity> {
        self.consumers.values().collect()
    }

    #[must_use]
    pub fn consumer(&self, id: &ConsumerId) -> Option<&ConsumerEntity> {
        self.consumers.get(id)
    }

    pub(crate) fn consumer_for_semantic_id(&self, id: &SemanticId) -> Option<&ConsumerEntity> {
        self.consumers
            .values()
            .find(|consumer| consumer.id.as_semantic_id() == id)
    }

    #[must_use]
    pub fn consumers_owned_by(&self, component: &SemanticId) -> Vec<&ConsumerId> {
        self.consumers
            .iter()
            .filter_map(|(id, consumer)| {
                (consumer.owner.entity_id() == Some(component)).then_some(id)
            })
            .collect()
    }

    #[must_use]
    pub fn consumers_for_context(&self, context: &ContextId) -> Vec<&ConsumerId> {
        self.consumers
            .iter()
            .filter_map(|(id, consumer)| (consumer.context() == Some(context)).then_some(id))
            .collect()
    }

    #[must_use]
    pub fn consumer_for_authored_field(&self, field: &SemanticId) -> Option<&ConsumerId> {
        self.consumers
            .iter()
            .find_map(|(id, consumer)| (&consumer.authored_field == field).then_some(id))
    }

    #[must_use]
    pub fn slots(&self) -> Vec<&SlotEntity> {
        self.slots.values().collect()
    }

    #[must_use]
    pub fn slot(&self, id: &SlotId) -> Option<&SlotEntity> {
        self.slots.get(id)
    }

    #[must_use]
    pub fn slots_owned_by(&self, component: &SemanticId) -> Vec<&SlotId> {
        self.slots
            .iter()
            .filter_map(|(id, slot)| (&slot.owner == component).then_some(id))
            .collect()
    }

    #[must_use]
    pub fn slot_for_authored_field(&self, field: &SemanticId) -> Option<&SlotId> {
        self.slots
            .iter()
            .find_map(|(id, slot)| (&slot.authored_field == field).then_some(id))
    }

    #[must_use]
    pub fn component_invocations(&self) -> Vec<&ComponentInvocationEntity> {
        self.component_invocations.values().collect()
    }

    #[must_use]
    pub fn component_invocation(
        &self,
        id: &ComponentInvocationId,
    ) -> Option<&ComponentInvocationEntity> {
        self.component_invocations.get(id)
    }

    #[must_use]
    pub fn component_invocations_owned_by(
        &self,
        component: &SemanticId,
    ) -> Vec<&ComponentInvocationId> {
        self.component_invocations
            .iter()
            .filter_map(|(id, invocation)| (&invocation.owner_component == component).then_some(id))
            .collect()
    }

    #[must_use]
    pub fn component_instance(
        &self,
        id: &crate::ComponentInstanceId,
    ) -> Option<&ComponentInstance> {
        self.component_instance_plan.instances.get(id)
    }

    #[must_use]
    pub fn component_build_roots(&self) -> Vec<&crate::ComponentBuildRoot> {
        self.component_instance_plan.roots.values().collect()
    }

    #[must_use]
    pub fn component_instances(&self) -> Vec<&ComponentInstance> {
        self.component_instance_plan.instances.values().collect()
    }

    #[must_use]
    pub fn blocked_component_instances(&self) -> Vec<&BlockedComponentInstancePlan> {
        self.component_instance_plan.blocked.values().collect()
    }

    #[must_use]
    pub fn blocked_component_instance(
        &self,
        id: &crate::ComponentInstanceId,
    ) -> Option<&BlockedComponentInstancePlan> {
        self.component_instance_plan.blocked.get(id)
    }

    #[must_use]
    pub fn component_instances_for_definition(
        &self,
        component: &SemanticId,
    ) -> Vec<&ComponentInstance> {
        self.component_instance_plan
            .instances
            .values()
            .filter(|instance| &instance.component == component)
            .collect()
    }

    #[must_use]
    pub const fn component_instance_scope_graph(&self) -> &ComponentInstanceScopeGraph {
        &self.component_instance_scope
    }

    #[must_use]
    pub const fn component_composition_analysis(&self) -> &ComponentCompositionAnalysis {
        &self.component_composition
    }

    #[must_use]
    pub const fn component_initialization_plan(&self) -> &ComponentInitializationPlan {
        &self.component_initialization
    }

    #[must_use]
    pub const fn component_ir_report(&self) -> &ComponentIrReport {
        &self.component_ir
    }

    #[must_use]
    pub const fn optimized_component_ir_report(&self) -> &OptimizedComponentIrReport {
        &self.component_ir_optimization
    }

    #[must_use]
    pub const fn instance_context_registry(&self) -> &InstanceContextRegistry {
        &self.instance_context
    }

    #[must_use]
    pub fn slot_content_fragments(&self) -> Vec<&SlotContentFragment> {
        self.slot_content_fragments.values().collect()
    }

    #[must_use]
    pub const fn slot_binding_registry(&self) -> &SlotBindingRegistry {
        &self.slot_bindings
    }

    #[must_use]
    pub const fn composition_type_products(&self) -> &CompositionTypeProducts {
        &self.composition_types
    }

    #[must_use]
    pub fn slot_bindings(&self) -> Vec<&SlotBinding> {
        self.slot_bindings.bindings.values().collect()
    }

    #[must_use]
    pub fn slot_binding(&self, id: &SlotBindingId) -> Option<&SlotBinding> {
        self.slot_bindings.binding(id)
    }

    #[must_use]
    pub fn slot_bindings_for_callee(
        &self,
        callee: &crate::ComponentInstanceId,
    ) -> Vec<&SlotBinding> {
        self.slot_bindings.for_callee(callee)
    }

    #[must_use]
    pub fn slot_content_fragment(
        &self,
        id: &SlotContentFragmentId,
    ) -> Option<&SlotContentFragment> {
        self.slot_content_fragments.get(id)
    }

    #[must_use]
    pub fn slot_content_fragments_for(
        &self,
        invocation: &ComponentInvocationId,
    ) -> Vec<&SlotContentFragment> {
        self.slot_content_fragments
            .values()
            .filter(|fragment| &fragment.invocation == invocation)
            .collect()
    }

    #[must_use]
    pub fn slot_outlets(&self) -> Vec<&SlotOutlet> {
        self.slot_outlets.values().collect()
    }

    #[must_use]
    pub fn slot_outlet(&self, id: &SlotOutletId) -> Option<&SlotOutlet> {
        self.slot_outlets.get(id)
    }

    #[must_use]
    pub fn slot_outlets_owned_by(&self, component: &SemanticId) -> Vec<&SlotOutletId> {
        self.slot_outlets
            .iter()
            .filter_map(|(id, outlet)| (&outlet.owner_component == component).then_some(id))
            .collect()
    }

    #[must_use]
    pub fn context_resolution(&self, consumer: &ConsumerId) -> Option<&ContextResolution> {
        self.context_resolutions.get(consumer)
    }

    #[must_use]
    pub const fn context_ownership_graph(&self) -> &ContextOwnershipGraph {
        &self.context_ownership
    }

    #[must_use]
    pub const fn context_dependency_graph(&self) -> &ContextDependencyGraph {
        &self.context_dependency
    }

    #[must_use]
    pub const fn context_lifetime_analysis(&self) -> &ContextLifetimeAnalysis {
        &self.context_lifetime
    }

    #[must_use]
    pub const fn context_evaluation_plan(&self) -> &ContextEvaluationPlan {
        &self.context_evaluation
    }

    #[must_use]
    pub fn resolved_provider(&self, consumer: &ConsumerId) -> Option<&ProviderId> {
        let ContextResolutionResult::Provider { provider, .. } =
            &self.context_resolution(consumer)?.result
        else {
            return None;
        };
        Some(provider)
    }

    #[must_use]
    pub fn consumers_resolved_to(&self, provider: &ProviderId) -> Vec<&ConsumerId> {
        self.context_resolutions
            .iter()
            .filter_map(|(consumer, resolution)| {
                matches!(
                    &resolution.result,
                    ContextResolutionResult::Provider {
                        provider: resolved, ..
                    } if resolved == provider
                )
                .then_some(consumer)
            })
            .collect()
    }

    #[must_use]
    pub fn consumers_using_default(&self, context: &ContextId) -> Vec<&ConsumerId> {
        self.context_resolutions
            .iter()
            .filter_map(|(consumer, resolution)| {
                matches!(
                    &resolution.result,
                    ContextResolutionResult::ContextDefault {
                        context: resolved, ..
                    } if resolved == context
                )
                .then_some(consumer)
            })
            .collect()
    }

    #[must_use]
    pub fn unresolved_context_consumers(&self) -> Vec<&ConsumerId> {
        self.context_resolutions
            .iter()
            .filter_map(|(consumer, resolution)| {
                matches!(resolution.result, ContextResolutionResult::Unresolved).then_some(consumer)
            })
            .collect()
    }

    #[must_use]
    pub fn context_type(&self, context: &ContextId) -> Option<&ContextTypeRecord> {
        self.context_types.get(context)
    }

    #[must_use]
    pub fn provider_type(&self, provider: &ProviderId) -> Option<&ProviderTypeRecord> {
        self.provider_types.get(provider)
    }

    #[must_use]
    pub fn consumer_type(&self, consumer: &ConsumerId) -> Option<&ConsumerTypeRecord> {
        self.consumer_types.get(consumer)
    }

    #[must_use]
    pub fn context_binding_type(&self, consumer: &ConsumerId) -> Option<&ContextBindingTypeRecord> {
        self.context_binding_types.get(consumer)
    }

    #[must_use]
    pub fn runtime_eligible_context_binding(&self, consumer: &ConsumerId) -> bool {
        self.context_binding_type(consumer)
            .is_some_and(|binding| binding.overall == ContextBindingCompatibility::Compatible)
    }

    #[must_use]
    pub fn effect(&self, id: &SemanticId) -> Option<&Effect> {
        self.effects.get(id)
    }

    #[must_use]
    pub fn effect_body(&self, effect: &SemanticId) -> Option<&EffectBody> {
        self.effect_bodies.get(effect)
    }

    #[must_use]
    pub fn effect_reactive_analysis(&self, effect: &SemanticId) -> Option<&EffectReactiveAnalysis> {
        self.effect_reactive_analysis.get(effect)
    }

    #[must_use]
    pub const fn effect_trigger_plan(&self) -> &EffectTriggerPlan {
        &self.effect_trigger_plan
    }

    #[must_use]
    pub const fn effect_execution_plan(&self) -> &EffectExecutionPlan {
        &self.effect_execution_plan
    }

    /// Returns F4 typing facts for one canonical effect statement.
    #[must_use]
    pub fn effect_statement_type(
        &self,
        statement: &SemanticId,
    ) -> Option<&EffectStatementTypeRecord> {
        self.semantic_types.effect_statements.get(statement)
    }

    #[must_use]
    pub fn effect_statement(&self, id: &SemanticId) -> Option<&EffectStatement> {
        self.effect_statements.get(id)
    }

    #[must_use]
    pub fn template_entity(&self, id: &SemanticId) -> Option<&TemplateSemanticEntity> {
        self.template_entities
            .iter()
            .find(|entity| entity.id == *id)
    }

    #[must_use]
    pub fn template_entities_for(&self, template: &SemanticId) -> Vec<&TemplateSemanticEntity> {
        self.children_of(template)
            .into_iter()
            .filter_map(|id| self.template_entity(id))
            .collect()
    }

    #[must_use]
    pub fn owner(&self, id: &SemanticId) -> Option<&SemanticOwner> {
        self.ownership.get(id)
    }

    #[must_use]
    pub fn parent_of(&self, id: &SemanticId) -> Option<&SemanticId> {
        self.owner(id).and_then(SemanticOwner::entity_id)
    }

    #[must_use]
    pub fn ancestors_of(&self, id: &SemanticId) -> Vec<&SemanticId> {
        let mut ancestors = Vec::new();
        let mut seen = BTreeSet::from([id.clone()]);
        let mut current = id;

        while let Some(parent) = self.parent_of(current) {
            if !seen.insert(parent.clone()) {
                break;
            }
            ancestors.push(parent);
            current = parent;
        }

        ancestors
    }

    #[must_use]
    pub fn provenance(&self, id: &SemanticId) -> Option<&SourceProvenance> {
        self.provenance.get(id)
    }

    #[must_use]
    pub fn expression(&self, id: &SemanticId) -> Option<&ExpressionNode> {
        self.expression_graph.node(id)
    }

    #[must_use]
    pub fn expression_root(&self, owner: &SemanticId) -> Option<&SemanticId> {
        self.expression_graph.root_for(owner)
    }

    #[must_use]
    pub fn expressions_for(&self, owner: &SemanticId) -> Vec<&ExpressionNode> {
        self.expression_graph.nodes_for(owner)
    }

    #[must_use]
    pub fn expression_dependencies(&self, id: &SemanticId) -> Vec<&SemanticId> {
        self.expression_graph.dependencies_of(id)
    }

    #[must_use]
    pub fn expression_dependents(&self, id: &SemanticId) -> Vec<&ExpressionNode> {
        self.expression_graph.dependents_of(id)
    }

    #[must_use]
    pub fn expression_owner(&self, id: &SemanticId) -> Option<&SemanticId> {
        self.expression_graph.owner_of(id)
    }

    #[must_use]
    pub fn expression_provenance(&self, id: &SemanticId) -> Option<&SourceProvenance> {
        self.expression_graph.provenance_of(id)
    }

    #[must_use]
    pub fn semantic_type_of(&self, id: &SemanticId) -> Option<&crate::SemanticType> {
        self.semantic_types
            .assignments
            .get(id)
            .map(|assignment| &assignment.semantic_type)
    }

    #[must_use]
    pub fn expression_type(&self, id: &SemanticId) -> Option<&crate::SemanticType> {
        self.expression_graph
            .node(id)
            .and_then(|_| self.semantic_type_of(id))
    }

    #[must_use]
    pub fn type_declarations(
        &self,
        semantic_type: &crate::SemanticType,
    ) -> Vec<&crate::SemanticTypeAssignment> {
        self.semantic_types
            .assignments
            .values()
            .filter(|assignment| {
                assignment.status == crate::SemanticTypeStatus::Declared
                    && assignment.semantic_type == *semantic_type
            })
            .collect()
    }

    #[must_use]
    pub fn type_usages(
        &self,
        semantic_type: &crate::SemanticType,
    ) -> Vec<&crate::SemanticTypeAssignment> {
        self.semantic_types
            .assignments
            .values()
            .filter(|assignment| assignment.semantic_type == *semantic_type)
            .collect()
    }

    #[must_use]
    pub fn serialization_compatibility_of(
        &self,
        id: &SemanticId,
    ) -> Option<crate::SerializationCompatibility> {
        self.semantic_type_of(id)
            .map(crate::serialization_compatibility)
    }

    #[must_use]
    pub fn is_type_assignable(&self, source: &SemanticId, target: &SemanticId) -> Option<bool> {
        Some(crate::is_assignable(
            self.semantic_type_of(source)?,
            self.semantic_type_of(target)?,
        ))
    }

    #[must_use]
    pub fn expressions_in_file(&self, path: &Path) -> Vec<&ExpressionNode> {
        self.expression_graph.nodes_in_file(path)
    }

    #[must_use]
    pub fn expressions_at(&self, path: &Path, offset: usize) -> Vec<&ExpressionNode> {
        self.expression_graph.nodes_at(path, offset)
    }

    #[must_use]
    pub fn application_roots(&self) -> Vec<&SemanticId> {
        self.ownership
            .iter()
            .filter_map(|(id, owner)| matches!(owner, SemanticOwner::Application).then_some(id))
            .collect()
    }

    #[must_use]
    pub fn children_of(&self, owner: &SemanticId) -> Vec<&SemanticId> {
        self.ownership
            .iter()
            .filter_map(|(id, entity_owner)| {
                (entity_owner.entity_id() == Some(owner)).then_some(id)
            })
            .collect()
    }

    #[must_use]
    pub fn descendants_of(&self, owner: &SemanticId) -> Vec<&SemanticId> {
        let mut descendants = Vec::new();
        self.collect_descendants(owner, &mut descendants);
        descendants
    }

    #[must_use]
    pub fn entities_of_kind(&self, kind: SemanticEntityKind) -> Vec<&SemanticId> {
        self.ownership
            .keys()
            .filter(|id| self.entity(id).is_some_and(|entity| entity.kind() == kind))
            .collect()
    }

    #[must_use]
    pub fn entities_in_file(&self, path: &Path) -> Vec<&SemanticId> {
        self.provenance
            .iter()
            .filter_map(|(id, provenance)| (provenance.path == path).then_some(id))
            .collect()
    }

    #[must_use]
    pub fn entities_at(&self, path: &Path, offset: usize) -> Vec<&SemanticId> {
        self.provenance
            .iter()
            .filter_map(|(id, provenance)| {
                (provenance.path == path
                    && provenance.span.start <= offset
                    && offset < provenance.span.end)
                    .then_some(id)
            })
            .collect()
    }

    #[must_use]
    pub fn references_of_kind(&self, kind: SemanticReferenceKind) -> Vec<&SemanticReference> {
        let mut references = self
            .references
            .iter()
            .filter(|reference| reference.kind == kind)
            .collect::<Vec<_>>();
        references.sort_by(|left, right| {
            (left.source.as_str(), left.target.as_str())
                .cmp(&(right.source.as_str(), right.target.as_str()))
        });
        references
    }

    #[must_use]
    pub fn references_in_file(&self, path: &Path) -> Vec<&SemanticReference> {
        let mut references = self
            .references
            .iter()
            .filter(|reference| reference.provenance.path == path)
            .collect::<Vec<_>>();
        references.sort_by(|left, right| {
            (left.source.as_str(), left.target.as_str())
                .cmp(&(right.source.as_str(), right.target.as_str()))
        });
        references
    }

    #[must_use]
    pub fn references_at(&self, path: &Path, offset: usize) -> Vec<&SemanticReference> {
        let mut references = self
            .references
            .iter()
            .filter(|reference| {
                reference.provenance.path == path
                    && reference.provenance.span.start <= offset
                    && offset < reference.provenance.span.end
            })
            .collect::<Vec<_>>();
        references.sort_by(|left, right| {
            (left.source.as_str(), left.target.as_str())
                .cmp(&(right.source.as_str(), right.target.as_str()))
        });
        references
    }

    fn collect_descendants<'a>(
        &'a self,
        owner: &SemanticId,
        descendants: &mut Vec<&'a SemanticId>,
    ) {
        for child in self.children_of(owner) {
            descendants.push(child);
            self.collect_descendants(child, descendants);
        }
    }

    #[must_use]
    pub fn references_from(&self, id: &SemanticId) -> Vec<&SemanticReference> {
        self.references
            .iter()
            .filter(move |reference| reference.source == *id)
            .collect()
    }

    #[must_use]
    pub fn references_to(&self, id: &SemanticId) -> Vec<&SemanticReference> {
        self.references
            .iter()
            .filter(move |reference| reference.target == *id)
            .collect()
    }
}

#[must_use]
pub fn build_application_semantic_model(parsed: &ParsedFile) -> ApplicationSemanticModel {
    build_application_semantic_model_from_files(std::slice::from_ref(parsed))
}

/// Assemble canonical ASM from an existing component graph while preserving its identity mode.
#[allow(clippy::too_many_lines)]
#[must_use]
pub fn build_application_semantic_model_from_component_graph(
    component_graph: &crate::component_graph::ComponentGraph,
) -> ApplicationSemanticModel {
    let templates = build_template_graph(component_graph).templates;
    let template_entities = build_template_semantic_entities(&templates);
    let component_invocations = collect_component_invocations(
        &component_graph.components,
        &templates,
        &template_entities,
        &[],
        None,
        &component_graph.provenance,
    );
    let component_instance_plan = plan_component_instances(
        &component_graph.components,
        &component_invocations,
        &template_entities,
        &component_graph.provenance,
    );
    let component_instance_scope = build_component_instance_scope_graph(&component_instance_plan);
    let component_composition = analyze_component_composition(
        &component_graph
            .components
            .iter()
            .map(|component| component.id.clone())
            .collect(),
        &component_invocations,
        &component_instance_plan,
    );
    let (computed_values, computed_diagnostics) = classify_computed_values(
        &component_graph.components,
        collect_computed_values(&component_graph.components, &component_graph.provenance),
        &component_graph.provenance,
    );
    let effects = collect_effects(&component_graph.components, &component_graph.provenance);
    let expression_graph =
        ExpressionGraph::from_components(&component_graph.components, &component_graph.provenance);
    let contexts = collect_context_entities(&component_graph.components, &expression_graph);
    let forms = collect_form_entities(&component_graph.components);
    let base_semantic_types = SemanticTypeModel::from_components(
        &component_graph.components,
        &component_graph.provenance,
    );
    let form_field_products = collect_form_field_products(
        &component_graph.components,
        &forms,
        &base_semantic_types,
        None,
    );
    let form_field_declaration_candidates = form_field_products.candidates;
    let form_fields = form_field_products.fields;
    let form_field_binding_products = collect_form_field_binding_products(
        &component_graph.components,
        &templates,
        &forms,
        &form_fields,
        &form_field_declaration_candidates,
    );
    let form_field_binding_candidates = form_field_binding_products.candidates;
    let form_field_bindings = form_field_binding_products.bindings;
    let slots = collect_slot_entities(&component_graph.components);
    let slot_composition = collect_slot_composition(&templates, &component_invocations, &slots);
    let slot_bindings = collect_slot_bindings(
        &component_instance_plan,
        &component_invocations,
        &slots,
        &slot_composition.fragments,
        &slot_composition.outlets,
    );
    let (providers, duplicate_provider_declarations) = collect_provider_entities(
        &component_graph.components,
        &contexts,
        &expression_graph,
        None,
    );
    let consumers = collect_consumer_entities(&component_graph.components, &contexts, None);
    let instance_context = collect_instance_context_registry(
        &component_instance_scope,
        &contexts,
        &providers,
        &consumers,
    );
    let context_declaration_candidates = collect_context_declaration_candidates(
        &component_graph.components,
        &contexts,
        &providers,
        &consumers,
    );
    let component_scope = ComponentScopeGraph::reflexive(&component_graph.components);
    let context_resolutions = collect_context_resolutions(
        &consumers,
        &contexts,
        &providers,
        &expression_graph,
        &component_scope,
    );
    let mut provenance = component_graph.provenance.clone();
    extend_template_entity_provenance(&mut provenance, &template_entities);
    extend_derived_entity_provenance(
        &mut provenance,
        &contexts,
        &forms,
        &form_fields,
        &form_field_bindings,
        &providers,
        &consumers,
        &slots,
        &component_invocations,
        &slot_composition.fragments,
        &slot_composition.outlets,
        &component_instance_plan,
        &computed_values,
        &effects,
    );
    let mut ownership = collect_ownership(
        &component_graph.components,
        &contexts,
        &forms,
        &form_fields,
        &form_field_bindings,
        &providers,
        &consumers,
        &slots,
        &component_invocations,
        &slot_composition.fragments,
        &slot_composition.outlets,
        &component_instance_plan,
        &computed_values,
        &effects,
        &templates,
        &template_entities,
    );
    let context_ownership = collect_context_ownership_graph(
        &component_graph.components,
        &contexts,
        &providers,
        &consumers,
        &expression_graph,
        &provenance,
    );
    let (effect_bodies, effect_statements) =
        lower_effect_bodies(&component_graph.components, &effects, &expression_graph);
    let mut references = component_graph.references.clone();
    references.extend(build_computed_references(
        &component_graph.components,
        &computed_values,
        &expression_graph,
    ));
    references.extend(build_effect_references(
        &component_graph.components,
        &effects,
        &computed_values,
        &expression_graph,
    ));
    references.extend(build_provider_references(&providers));
    references.extend(build_consumer_references(&consumers));
    references.extend(build_context_resolution_references(&context_resolutions));
    extend_template_references(
        &mut references,
        &component_graph.components,
        &computed_values,
        &template_entities,
        &ownership,
    );
    references.extend(build_form_field_binding_references(&form_field_bindings));
    let form_ownership = collect_form_ownership_graph(
        &component_instance_plan.roots,
        &component_graph.components,
        &forms,
        &form_fields,
        &form_field_bindings,
        &template_entities,
        &ownership,
        &references,
        &provenance,
    );
    let validation_products =
        collect_validation_products(&component_graph.components, &forms, &form_fields);
    let validation_rule_candidates = validation_products.candidates;
    let validation_rules = validation_products.rules;
    extend_validation_products(
        &mut provenance,
        &mut ownership,
        &mut references,
        &validation_rules,
    );
    let validation_graph = collect_validation_graph(
        &component_instance_plan.roots,
        &form_ownership,
        &forms,
        &form_fields,
        &validation_rules,
        &validation_rule_candidates,
        &validation_products.cycles,
        &ownership,
        &references,
    );
    let validation_dependency_plans = collect_validation_dependency_plans(
        &forms,
        &form_fields,
        &validation_rules,
        &form_ownership,
        &validation_graph,
    );
    let form_tracking =
        collect_form_tracking_products(&forms, &form_fields, &form_field_bindings, &form_ownership);
    let semantic_types = finalize_semantic_types(
        base_semantic_types,
        &component_graph.components,
        &contexts,
        &forms,
        &form_fields,
        &providers,
        &consumers,
        &slots,
        &computed_values,
        &effects,
        &effect_statements,
        &expression_graph,
        &references,
        &template_entities,
    );
    let context_type_products = collect_context_type_products(
        &contexts,
        &providers,
        &consumers,
        &context_resolutions,
        &expression_graph,
        &semantic_types,
    );
    let context_dependency = collect_context_dependency_graph(
        &component_graph.components,
        &contexts,
        &providers,
        &consumers,
        &context_resolutions,
        &context_type_products.contexts,
        &context_type_products.providers,
        &context_type_products.bindings,
        &computed_values,
        &expression_graph,
    );
    let context_lifetime = collect_context_lifetime_analysis(
        &component_graph.components,
        &contexts,
        &providers,
        &consumers,
        &computed_values,
        &context_ownership,
        &component_scope,
        &context_resolutions,
        &context_dependency,
        &provenance,
    );
    let effects = validate_effects(
        &component_graph.components,
        effects,
        &effect_statements,
        &semantic_types,
    );
    let (
        reactive_graph,
        reactive_transitive_analysis,
        reactive_cycle_analysis,
        computed_evaluation_plan,
    ) = build_computed_reactive_products(
        &component_graph.components,
        &computed_values,
        &effects,
        &references,
        &provenance,
    );
    let context_evaluation = collect_context_evaluation_plan(
        &contexts,
        &providers,
        &context_resolutions,
        &context_type_products.contexts,
        &context_type_products.providers,
        &context_type_products.bindings,
        &context_lifetime,
        &context_dependency,
        &computed_evaluation_plan,
        &component_scope,
    );
    let effect_reactive_analysis = analyze_effect_reactivity(
        &component_graph.components,
        &computed_values,
        &effects,
        &reactive_transitive_analysis,
    );
    let effect_trigger_plan = derive_effect_trigger_plan(
        &component_graph.components,
        &effects,
        &effect_reactive_analysis,
        &provenance,
    );
    let submissions = collect_submission_products(
        &component_graph.components,
        &forms,
        &form_fields,
        &validation_rules,
        &effect_trigger_plan,
    );
    let serialization = collect_serialization_products(
        &component_graph.components,
        &forms,
        &form_fields,
        &submissions.plans,
    );
    let reset = collect_reset_products(&forms, &form_fields, &form_field_bindings, &form_tracking);
    let form_ir = lower_form_ir(&component_instance_plan, &forms, &form_fields);
    let effect_execution_plan = plan_effect_execution(
        &computed_values,
        &effects,
        &effect_reactive_analysis,
        &effect_trigger_plan,
        &reactive_transitive_analysis,
        &computed_evaluation_plan,
    );
    let mut diagnostics = component_graph.diagnostics.clone();
    diagnostics.extend(computed_diagnostics);
    extend_computed_diagnostics(
        &mut diagnostics,
        &component_graph.components,
        &computed_values,
        &expression_graph,
        &semantic_types,
        &reactive_cycle_analysis,
        &provenance,
    );
    let component_ids = component_graph
        .components
        .iter()
        .map(|component| component.id.clone())
        .collect::<BTreeSet<_>>();
    let composition_types = collect_composition_type_products(
        &component_ids,
        &component_invocations,
        &slot_bindings,
        &slots,
        &slot_composition.fragments,
        &slot_composition.outlets,
        &instance_context,
        &context_type_products.bindings,
        &context_type_products.contexts,
        &context_type_products.providers,
        &context_lifetime,
        &ownership,
        &references,
    );
    let component_initialization = plan_component_initialization(
        &component_instance_plan,
        &slot_bindings,
        &composition_types,
        &instance_context,
    );
    let mut model = ApplicationSemanticModel {
        expression_graph,
        semantic_types,
        components: components_with_resolved_form_field_candidates(
            &component_graph.components,
            &form_field_declaration_candidates,
        ),
        contexts,
        providers,
        consumers,
        forms,
        form_field_declaration_candidates,
        form_fields,
        form_field_binding_candidates,
        form_field_bindings,
        form_ownership,
        validation_rule_candidates,
        validation_rules,
        validation_graph,
        validation_dependency_plans,
        form_tracking,
        submissions,
        serialization,
        reset,
        form_ir,
        slots,
        component_invocations,
        component_instance_plan,
        component_instance_scope,
        component_composition,
        component_initialization,
        component_ir: ComponentIrReport::default(),
        component_ir_optimization: OptimizedComponentIrReport::default(),
        instance_context,
        slot_content_fragments: slot_composition.fragments,
        slot_outlets: slot_composition.outlets,
        slot_bindings,
        composition_types,
        context_declaration_candidates,
        context_ownership,
        context_dependency,
        context_lifetime,
        context_evaluation,
        component_scope,
        context_resolutions,
        context_types: context_type_products.contexts,
        provider_types: context_type_products.providers,
        consumer_types: context_type_products.consumers,
        context_binding_types: context_type_products.bindings,
        duplicate_provider_declarations,
        computed_values,
        effects,
        effect_reactive_analysis,
        effect_trigger_plan,
        effect_execution_plan,
        effect_bodies,
        effect_statements,
        reactive_graph,
        reactive_transitive_analysis,
        reactive_cycle_analysis,
        computed_evaluation_plan,
        templates,
        template_entities,
        diagnostics,
        ownership,
        references,
        provenance,
    };
    model
        .diagnostics
        .extend(crate::collect_effect_diagnostics(&model));
    model
        .diagnostics
        .extend(crate::collect_context_diagnostics(&model));
    model.component_ir = lower_component_ir(&model);
    model.component_ir_optimization = optimize_component_ir(&model.component_ir);
    model
        .diagnostics
        .extend(crate::collect_component_diagnostics(&model));
    model
}

#[must_use]
pub fn build_application_semantic_model_for_unit(
    unit: &CompilationUnit,
) -> ApplicationSemanticModel {
    let symbols = crate::build_symbol_table(unit);
    let modules = crate::build_module_graph(unit);
    let bindings = crate::build_binding_table(unit, &symbols, &modules);
    build_application_semantic_model_from_files_with_bindings(unit.files(), Some(&bindings))
}

fn build_application_semantic_model_from_files(files: &[ParsedFile]) -> ApplicationSemanticModel {
    build_application_semantic_model_from_files_with_bindings(files, None)
}

#[allow(clippy::too_many_lines)]
fn build_application_semantic_model_from_files_with_bindings(
    files: &[ParsedFile],
    bindings: Option<&crate::BindingTable>,
) -> ApplicationSemanticModel {
    let (mut components, mut templates, mut diagnostics, mut references) =
        (Vec::new(), Vec::new(), Vec::new(), Vec::new());
    let (mut provenance, mut template_entities, mut type_aliases) =
        (BTreeMap::new(), Vec::new(), Vec::new());
    for parsed in files {
        let component_graph = build_component_graph_for_module(parsed);
        let template_graph = build_template_graph(&component_graph);
        let file_template_entities = build_template_semantic_entities(&template_graph.templates);

        components.extend(component_graph.components);
        templates.extend(template_graph.templates);
        diagnostics.extend(component_graph.diagnostics);
        references.extend(component_graph.references);
        provenance.extend(component_graph.provenance);
        extend_template_entity_provenance(&mut provenance, &file_template_entities);
        template_entities.extend(file_template_entities);
        type_aliases.extend(
            parsed
                .type_aliases
                .iter()
                .cloned()
                .map(|alias| (parsed.path.clone(), alias)),
        );
    }

    let component_invocations = collect_component_invocations(
        &components,
        &templates,
        &template_entities,
        files,
        bindings,
        &provenance,
    );
    let component_instance_plan = plan_component_instances(
        &components,
        &component_invocations,
        &template_entities,
        &provenance,
    );
    let component_instance_scope = build_component_instance_scope_graph(&component_instance_plan);
    let component_composition = analyze_component_composition(
        &components
            .iter()
            .map(|component| component.id.clone())
            .collect(),
        &component_invocations,
        &component_instance_plan,
    );

    let (computed_values, computed_diagnostics) = classify_computed_values(
        &components,
        collect_computed_values(&components, &provenance),
        &provenance,
    );
    let effects = collect_effects(&components, &provenance);
    let expression_graph = ExpressionGraph::from_components(&components, &provenance);
    let contexts = collect_context_entities(&components, &expression_graph);
    let forms = collect_form_entities(&components);
    let base_semantic_types = SemanticTypeModel::from_components_with_aliases_and_bindings(
        &components,
        &provenance,
        &type_aliases,
        bindings,
    );
    let form_field_products =
        collect_form_field_products(&components, &forms, &base_semantic_types, bindings);
    let form_field_declaration_candidates = form_field_products.candidates;
    let form_fields = form_field_products.fields;
    let form_field_binding_products = collect_form_field_binding_products(
        &components,
        &templates,
        &forms,
        &form_fields,
        &form_field_declaration_candidates,
    );
    let form_field_binding_candidates = form_field_binding_products.candidates;
    let form_field_bindings = form_field_binding_products.bindings;
    let slots = collect_slot_entities(&components);
    let slot_composition = collect_slot_composition(&templates, &component_invocations, &slots);
    let slot_bindings = collect_slot_bindings(
        &component_instance_plan,
        &component_invocations,
        &slots,
        &slot_composition.fragments,
        &slot_composition.outlets,
    );
    let (providers, duplicate_provider_declarations) =
        collect_provider_entities(&components, &contexts, &expression_graph, bindings);
    let consumers = collect_consumer_entities(&components, &contexts, bindings);
    let instance_context = collect_instance_context_registry(
        &component_instance_scope,
        &contexts,
        &providers,
        &consumers,
    );
    let context_declaration_candidates =
        collect_context_declaration_candidates(&components, &contexts, &providers, &consumers);
    let component_scope = ComponentScopeGraph::reflexive(&components);
    let context_resolutions = collect_context_resolutions(
        &consumers,
        &contexts,
        &providers,
        &expression_graph,
        &component_scope,
    );
    diagnostics.extend(computed_diagnostics);
    extend_derived_entity_provenance(
        &mut provenance,
        &contexts,
        &forms,
        &form_fields,
        &form_field_bindings,
        &providers,
        &consumers,
        &slots,
        &component_invocations,
        &slot_composition.fragments,
        &slot_composition.outlets,
        &component_instance_plan,
        &computed_values,
        &effects,
    );
    let mut ownership = collect_ownership(
        &components,
        &contexts,
        &forms,
        &form_fields,
        &form_field_bindings,
        &providers,
        &consumers,
        &slots,
        &component_invocations,
        &slot_composition.fragments,
        &slot_composition.outlets,
        &component_instance_plan,
        &computed_values,
        &effects,
        &templates,
        &template_entities,
    );
    let context_ownership = collect_context_ownership_graph(
        &components,
        &contexts,
        &providers,
        &consumers,
        &expression_graph,
        &provenance,
    );
    let (effect_bodies, effect_statements) =
        lower_effect_bodies(&components, &effects, &expression_graph);
    references.extend(build_computed_references(
        &components,
        &computed_values,
        &expression_graph,
    ));
    references.extend(build_effect_references(
        &components,
        &effects,
        &computed_values,
        &expression_graph,
    ));
    references.extend(build_provider_references(&providers));
    references.extend(build_consumer_references(&consumers));
    references.extend(build_context_resolution_references(&context_resolutions));
    extend_template_references(
        &mut references,
        &components,
        &computed_values,
        &template_entities,
        &ownership,
    );
    references.extend(build_form_field_binding_references(&form_field_bindings));
    let form_ownership = collect_form_ownership_graph(
        &component_instance_plan.roots,
        &components,
        &forms,
        &form_fields,
        &form_field_bindings,
        &template_entities,
        &ownership,
        &references,
        &provenance,
    );
    let validation_products = collect_validation_products(&components, &forms, &form_fields);
    let validation_rule_candidates = validation_products.candidates;
    let validation_rules = validation_products.rules;
    extend_validation_products(
        &mut provenance,
        &mut ownership,
        &mut references,
        &validation_rules,
    );
    let validation_graph = collect_validation_graph(
        &component_instance_plan.roots,
        &form_ownership,
        &forms,
        &form_fields,
        &validation_rules,
        &validation_rule_candidates,
        &validation_products.cycles,
        &ownership,
        &references,
    );
    let validation_dependency_plans = collect_validation_dependency_plans(
        &forms,
        &form_fields,
        &validation_rules,
        &form_ownership,
        &validation_graph,
    );
    let form_tracking =
        collect_form_tracking_products(&forms, &form_fields, &form_field_bindings, &form_ownership);
    let semantic_types = finalize_semantic_types(
        base_semantic_types,
        &components,
        &contexts,
        &forms,
        &form_fields,
        &providers,
        &consumers,
        &slots,
        &computed_values,
        &effects,
        &effect_statements,
        &expression_graph,
        &references,
        &template_entities,
    );
    let context_type_products = collect_context_type_products(
        &contexts,
        &providers,
        &consumers,
        &context_resolutions,
        &expression_graph,
        &semantic_types,
    );
    let context_dependency = collect_context_dependency_graph(
        &components,
        &contexts,
        &providers,
        &consumers,
        &context_resolutions,
        &context_type_products.contexts,
        &context_type_products.providers,
        &context_type_products.bindings,
        &computed_values,
        &expression_graph,
    );
    let context_lifetime = collect_context_lifetime_analysis(
        &components,
        &contexts,
        &providers,
        &consumers,
        &computed_values,
        &context_ownership,
        &component_scope,
        &context_resolutions,
        &context_dependency,
        &provenance,
    );
    let effects = validate_effects(&components, effects, &effect_statements, &semantic_types);
    let (
        reactive_graph,
        reactive_transitive_analysis,
        reactive_cycle_analysis,
        computed_evaluation_plan,
    ) = build_computed_reactive_products(
        &components,
        &computed_values,
        &effects,
        &references,
        &provenance,
    );
    let context_evaluation = collect_context_evaluation_plan(
        &contexts,
        &providers,
        &context_resolutions,
        &context_type_products.contexts,
        &context_type_products.providers,
        &context_type_products.bindings,
        &context_lifetime,
        &context_dependency,
        &computed_evaluation_plan,
        &component_scope,
    );
    let effect_reactive_analysis = analyze_effect_reactivity(
        &components,
        &computed_values,
        &effects,
        &reactive_transitive_analysis,
    );
    let effect_trigger_plan = derive_effect_trigger_plan(
        &components,
        &effects,
        &effect_reactive_analysis,
        &provenance,
    );
    let submissions = collect_submission_products(
        &components,
        &forms,
        &form_fields,
        &validation_rules,
        &effect_trigger_plan,
    );
    let serialization =
        collect_serialization_products(&components, &forms, &form_fields, &submissions.plans);
    let reset = collect_reset_products(&forms, &form_fields, &form_field_bindings, &form_tracking);
    let form_ir = lower_form_ir(&component_instance_plan, &forms, &form_fields);
    let effect_execution_plan = plan_effect_execution(
        &computed_values,
        &effects,
        &effect_reactive_analysis,
        &effect_trigger_plan,
        &reactive_transitive_analysis,
        &computed_evaluation_plan,
    );
    extend_computed_diagnostics(
        &mut diagnostics,
        &components,
        &computed_values,
        &expression_graph,
        &semantic_types,
        &reactive_cycle_analysis,
        &provenance,
    );
    let component_ids = components
        .iter()
        .map(|component| component.id.clone())
        .collect::<BTreeSet<_>>();
    let composition_types = collect_composition_type_products(
        &component_ids,
        &component_invocations,
        &slot_bindings,
        &slots,
        &slot_composition.fragments,
        &slot_composition.outlets,
        &instance_context,
        &context_type_products.bindings,
        &context_type_products.contexts,
        &context_type_products.providers,
        &context_lifetime,
        &ownership,
        &references,
    );
    let component_initialization = plan_component_initialization(
        &component_instance_plan,
        &slot_bindings,
        &composition_types,
        &instance_context,
    );
    let components = components_with_resolved_form_field_candidates(
        &components,
        &form_field_declaration_candidates,
    );
    let mut model = ApplicationSemanticModel {
        expression_graph,
        semantic_types,
        components,
        contexts,
        providers,
        consumers,
        forms,
        form_field_declaration_candidates,
        form_fields,
        form_field_binding_candidates,
        form_field_bindings,
        form_ownership,
        validation_rule_candidates,
        validation_rules,
        validation_graph,
        validation_dependency_plans,
        form_tracking,
        submissions,
        serialization,
        reset,
        form_ir,
        slots,
        component_invocations,
        component_instance_plan,
        component_instance_scope,
        component_composition,
        component_initialization,
        component_ir: ComponentIrReport::default(),
        component_ir_optimization: OptimizedComponentIrReport::default(),
        instance_context,
        slot_content_fragments: slot_composition.fragments,
        slot_outlets: slot_composition.outlets,
        slot_bindings,
        composition_types,
        context_declaration_candidates,
        context_ownership,
        context_dependency,
        context_lifetime,
        context_evaluation,
        component_scope,
        context_resolutions,
        context_types: context_type_products.contexts,
        provider_types: context_type_products.providers,
        consumer_types: context_type_products.consumers,
        context_binding_types: context_type_products.bindings,
        duplicate_provider_declarations,
        computed_values,
        effects,
        effect_reactive_analysis,
        effect_trigger_plan,
        effect_execution_plan,
        effect_bodies,
        effect_statements,
        reactive_graph,
        reactive_transitive_analysis,
        reactive_cycle_analysis,
        computed_evaluation_plan,
        templates,
        template_entities,
        diagnostics,
        ownership,
        references,
        provenance,
    };
    model
        .diagnostics
        .extend(crate::collect_effect_diagnostics(&model));
    model
        .diagnostics
        .extend(crate::collect_context_diagnostics(&model));
    model.component_ir = lower_component_ir(&model);
    model.component_ir_optimization = optimize_component_ir(&model.component_ir);
    model
        .diagnostics
        .extend(crate::collect_component_diagnostics(&model));
    model
}

fn components_with_resolved_form_field_candidates(
    components: &[ComponentNode],
    candidates: &[crate::FormFieldDeclarationCandidate],
) -> Vec<ComponentNode> {
    let candidates = candidates
        .iter()
        .map(|candidate| (candidate.id.clone(), candidate))
        .collect::<BTreeMap<_, _>>();
    let mut components = components.to_vec();
    for component in &mut components {
        for candidate in &mut component.form_field_declaration_candidates {
            if let Some(resolved) = candidates.get(&candidate.id) {
                *candidate = (*resolved).clone();
            }
        }
    }
    components
}

fn extend_template_entity_provenance(
    provenance: &mut BTreeMap<SemanticId, SourceProvenance>,
    template_entities: &[TemplateSemanticEntity],
) {
    provenance.extend(
        template_entities
            .iter()
            .map(|entity| (entity.id.clone(), entity.provenance.clone())),
    );
}

fn extend_validation_products(
    provenance: &mut BTreeMap<SemanticId, SourceProvenance>,
    ownership: &mut BTreeMap<SemanticId, SemanticOwner>,
    references: &mut Vec<SemanticReference>,
    rules: &BTreeMap<ValidationRuleId, ValidationRule>,
) {
    for rule in rules.values() {
        provenance.insert(rule.id.as_semantic_id().clone(), rule.provenance.clone());
        ownership.insert(
            rule.id.as_semantic_id().clone(),
            SemanticOwner::entity(rule.target_field.as_semantic_id().clone()),
        );
        if let Some(dependency) = &rule.dependency {
            references.push(SemanticReference {
                kind: SemanticReferenceKind::ValidationRuleField,
                source: rule.id.as_semantic_id().clone(),
                target: dependency.as_semantic_id().clone(),
                provenance: rule
                    .argument_provenance
                    .clone()
                    .unwrap_or_else(|| rule.decorator_provenance.clone()),
            });
        }
    }
    references.sort_by(|left, right| {
        (left.source.as_str(), left.kind, left.target.as_str()).cmp(&(
            right.source.as_str(),
            right.kind,
            right.target.as_str(),
        ))
    });
    references.dedup();
}

#[allow(clippy::too_many_arguments)]
fn extend_derived_entity_provenance(
    provenance: &mut BTreeMap<SemanticId, SourceProvenance>,
    contexts: &BTreeMap<ContextId, ContextEntity>,
    forms: &BTreeMap<FormId, FormEntity>,
    form_fields: &BTreeMap<FieldId, FormFieldEntity>,
    form_field_bindings: &BTreeMap<FieldBindingId, FormFieldBinding>,
    providers: &BTreeMap<ProviderId, ProviderEntity>,
    consumers: &BTreeMap<ConsumerId, ConsumerEntity>,
    slots: &BTreeMap<SlotId, SlotEntity>,
    component_invocations: &BTreeMap<ComponentInvocationId, ComponentInvocationEntity>,
    slot_content_fragments: &BTreeMap<SlotContentFragmentId, SlotContentFragment>,
    slot_outlets: &BTreeMap<SlotOutletId, SlotOutlet>,
    component_instance_plan: &ComponentInstancePlan,
    computed_values: &BTreeMap<SemanticId, ComputedValue>,
    effects: &BTreeMap<SemanticId, Effect>,
) {
    provenance.extend(contexts.values().map(|context| {
        (
            context.id.as_semantic_id().clone(),
            context.provenance.clone(),
        )
    }));
    provenance.extend(
        forms
            .values()
            .map(|form| (form.id.as_semantic_id().clone(), form.provenance.clone())),
    );
    provenance.extend(
        form_fields
            .values()
            .map(|field| (field.id.as_semantic_id().clone(), field.provenance.clone())),
    );
    provenance.extend(form_field_bindings.values().map(|binding| {
        (
            binding.id.as_semantic_id().clone(),
            binding.provenance.clone(),
        )
    }));
    provenance.extend(providers.values().map(|provider| {
        (
            provider.id.as_semantic_id().clone(),
            provider.provenance.clone(),
        )
    }));
    provenance.extend(consumers.values().map(|consumer| {
        (
            consumer.id.as_semantic_id().clone(),
            consumer.provenance.clone(),
        )
    }));
    provenance.extend(
        slots
            .values()
            .map(|slot| (slot.id.as_semantic_id().clone(), slot.provenance.clone())),
    );
    provenance.extend(component_invocations.values().map(|invocation| {
        (
            invocation.id.as_semantic_id().clone(),
            invocation.provenance.clone(),
        )
    }));
    provenance.extend(slot_content_fragments.values().map(|fragment| {
        (
            fragment.id.as_semantic_id().clone(),
            fragment.provenance.clone(),
        )
    }));
    provenance.extend(slot_outlets.values().map(|outlet| {
        (
            outlet.id.as_semantic_id().clone(),
            outlet.provenance.clone(),
        )
    }));
    provenance.extend(component_instance_plan.instances.values().map(|instance| {
        (
            instance.id.as_semantic_id().clone(),
            instance.provenance.clone(),
        )
    }));
    provenance.extend(component_instance_plan.blocked.values().map(|blocked| {
        (
            blocked.id.as_semantic_id().clone(),
            blocked.provenance.clone(),
        )
    }));
    provenance.extend(
        computed_values
            .iter()
            .map(|(id, computed)| (id.clone(), computed.provenance.clone())),
    );
    provenance.extend(
        effects
            .iter()
            .map(|(id, effect)| (id.clone(), effect.provenance.clone())),
    );
}

#[allow(clippy::too_many_arguments)]
fn finalize_semantic_types(
    semantic_types: SemanticTypeModel,
    components: &[ComponentNode],
    contexts: &BTreeMap<ContextId, ContextEntity>,
    forms: &BTreeMap<FormId, FormEntity>,
    form_fields: &BTreeMap<FieldId, FormFieldEntity>,
    providers: &BTreeMap<ProviderId, ProviderEntity>,
    consumers: &BTreeMap<ConsumerId, ConsumerEntity>,
    slots: &BTreeMap<SlotId, SlotEntity>,
    computed_values: &BTreeMap<SemanticId, ComputedValue>,
    effects: &BTreeMap<SemanticId, Effect>,
    effect_statements: &BTreeMap<SemanticId, EffectStatement>,
    expression_graph: &ExpressionGraph,
    references: &[SemanticReference],
    template_entities: &[TemplateSemanticEntity],
) -> SemanticTypeModel {
    semantic_types
        .with_slot_types(slots)
        .with_form_types(forms)
        .with_form_field_types(form_fields)
        .with_context_types(contexts)
        .with_provider_types(providers)
        .with_consumer_types(consumers)
        .with_expression_types(expression_graph, components, contexts, providers)
        .with_computed_value_types(components, computed_values, expression_graph, references)
        .with_context_source_expression_types(components, contexts, providers, expression_graph)
        .with_effect_statement_types(components, effects, effect_statements, expression_graph)
        .with_template_binding_types(template_entities, references)
        .normalized()
}

fn extend_computed_diagnostics(
    diagnostics: &mut Vec<ComponentDiagnostic>,
    components: &[ComponentNode],
    computed_values: &BTreeMap<SemanticId, ComputedValue>,
    expression_graph: &ExpressionGraph,
    semantic_types: &SemanticTypeModel,
    reactive_cycle_analysis: &IrReactiveCycleAnalysis,
    provenance: &BTreeMap<SemanticId, SourceProvenance>,
) {
    diagnostics.extend(collect_computed_cycle_diagnostics(
        reactive_cycle_analysis,
        computed_values,
    ));
    diagnostics.extend(collect_computed_semantic_diagnostics(
        components,
        computed_values,
        expression_graph,
        semantic_types,
        provenance,
    ));
}

#[allow(clippy::too_many_lines)]
fn classify_computed_values(
    components: &[ComponentNode],
    mut computed_values: BTreeMap<SemanticId, ComputedValue>,
    provenance: &BTreeMap<SemanticId, SourceProvenance>,
) -> (
    BTreeMap<SemanticId, ComputedValue>,
    Vec<ComponentDiagnostic>,
) {
    let mut diagnostics = Vec::new();

    for computed in computed_values.values_mut() {
        let Some(component_id) = computed.owner.entity_id() else {
            continue;
        };
        let Some(component) = components
            .iter()
            .find(|component| component.id == *component_id)
        else {
            continue;
        };
        let Some(method) = component
            .methods
            .iter()
            .find(|method| method.id == computed.method)
        else {
            continue;
        };
        let mut violations = Vec::new();

        if method.is_async {
            violations.push(crate::ComputedPurityViolation {
                kind: crate::ComputedPurityViolationKind::Async,
                provenance: computed.provenance.clone(),
            });
        }
        if method
            .decorators
            .iter()
            .any(|decorator| decorator == "action")
        {
            violations.push(crate::ComputedPurityViolation {
                kind: crate::ComputedPurityViolationKind::Action,
                provenance: computed.provenance.clone(),
            });
        }
        if method
            .decorators
            .iter()
            .any(|decorator| decorator == "effect")
        {
            violations.push(crate::ComputedPurityViolation {
                kind: crate::ComputedPurityViolationKind::Effect,
                provenance: computed.provenance.clone(),
            });
        }
        for action in component
            .actions
            .iter()
            .filter(|action| action.method == method.name)
        {
            violations.push(crate::ComputedPurityViolation {
                kind: crate::ComputedPurityViolationKind::StateMutation,
                provenance: provenance
                    .get(&action.id)
                    .expect("computed state update should have canonical provenance")
                    .clone(),
            });
        }
        for call in &method.calls {
            let kind = computed_call_purity_kind(component, &call.callee);
            violations.push(crate::ComputedPurityViolation {
                kind,
                provenance: SourceProvenance::new(&computed.provenance.path, call.span),
            });
        }

        violations.sort_by(|left, right| {
            (
                left.kind,
                left.provenance.path.as_path(),
                left.provenance.span.start,
                left.provenance.span.end,
            )
                .cmp(&(
                    right.kind,
                    right.provenance.path.as_path(),
                    right.provenance.span.start,
                    right.provenance.span.end,
                ))
        });
        violations
            .dedup_by(|left, right| left.kind == right.kind && left.provenance == right.provenance);
        computed.purity = if violations.is_empty() {
            crate::ComputedPurity::Pure
        } else {
            crate::ComputedPurity::Impure
        };
        computed.purity_violations = violations;
        diagnostics.extend(computed.purity_violations.iter().map(|violation| {
            ComponentDiagnostic {
                severity: ComponentDiagnosticSeverity::Error,
                effect_id: None,
                statement_id: None,
                context_declaration_candidate_id: None,
                context_id: None,
                provider_id: None,
                consumer_id: None,
                slot_id: None,
                invocation_id: None,
                component_instance_id: None,
                slot_binding_id: None,
                structural_region_id: None,
                component_id: None,
                provider_instance_id: None,
                consumer_instance_id: None,
                secondary_labels: Vec::new(),
                code: ComputedDiagnosticCode::PurityViolation.as_str().to_string(),
                message: format!(
                    "computed getter `{}` is impure: {}",
                    computed.name,
                    violation.kind.description()
                ),
                provenance: Some(violation.provenance.clone()),
            }
        }));
    }

    (computed_values, diagnostics)
}

fn collect_computed_cycle_diagnostics(
    analysis: &IrReactiveCycleAnalysis,
    computed_values: &BTreeMap<SemanticId, ComputedValue>,
) -> Vec<ComponentDiagnostic> {
    analysis
        .cycles
        .iter()
        .filter_map(|cycle| {
            let first = cycle.nodes.first()?;
            let computed = computed_values
                .values()
                .find(|computed| computed.id.as_str() == first)?;
            Some(ComponentDiagnostic {
                severity: ComponentDiagnosticSeverity::Error,
                effect_id: None,
                statement_id: None,
                context_declaration_candidate_id: None,
                context_id: None,
                provider_id: None,
                consumer_id: None,
                slot_id: None,
                invocation_id: None,
                component_instance_id: None,
                slot_binding_id: None,
                structural_region_id: None,
                component_id: None,
                provider_instance_id: None,
                consumer_instance_id: None,
                secondary_labels: Vec::new(),
                code: ComputedDiagnosticCode::DependencyCycle.as_str().to_string(),
                message: format!(
                    "computed dependency cycle detected among: {}",
                    cycle.nodes.join(", ")
                ),
                provenance: Some(computed.provenance.clone()),
            })
        })
        .collect()
}

fn collect_computed_semantic_diagnostics(
    components: &[ComponentNode],
    computed_values: &BTreeMap<SemanticId, ComputedValue>,
    expression_graph: &ExpressionGraph,
    semantic_types: &SemanticTypeModel,
    provenance: &BTreeMap<SemanticId, SourceProvenance>,
) -> Vec<ComponentDiagnostic> {
    let mut diagnostics = collect_invalid_computed_declaration_diagnostics(components, provenance);
    diagnostics.extend(collect_computed_body_and_read_diagnostics(
        components,
        computed_values,
        expression_graph,
    ));
    diagnostics.extend(collect_computed_type_diagnostics(
        computed_values,
        semantic_types,
    ));
    sort_computed_diagnostics(&mut diagnostics);
    diagnostics
}

fn collect_invalid_computed_declaration_diagnostics(
    components: &[ComponentNode],
    provenance: &BTreeMap<SemanticId, SourceProvenance>,
) -> Vec<ComponentDiagnostic> {
    components
        .iter()
        .flat_map(|component| &component.methods)
        .filter(|method| {
            method
                .decorators
                .iter()
                .any(|decorator| decorator == "computed")
                && !method.is_getter
        })
        .map(|method| ComponentDiagnostic {
            severity: ComponentDiagnosticSeverity::Error,
            effect_id: None,
            statement_id: None,
            context_declaration_candidate_id: None,
            context_id: None,
            provider_id: None,
            consumer_id: None,
            slot_id: None,
            invocation_id: None,
            component_instance_id: None,
            slot_binding_id: None,
            structural_region_id: None,
            component_id: None,
            provider_instance_id: None,
            consumer_instance_id: None,
            secondary_labels: Vec::new(),
            code: ComputedDiagnosticCode::InvalidDeclaration
                .as_str()
                .to_string(),
            message: format!(
                "@computed() declaration `{}` must decorate a getter",
                method.name
            ),
            provenance: Some(
                provenance
                    .get(&method.id)
                    .expect("component methods should have canonical provenance")
                    .clone(),
            ),
        })
        .collect()
}

fn collect_computed_body_and_read_diagnostics(
    components: &[ComponentNode],
    computed_values: &BTreeMap<SemanticId, ComputedValue>,
    expression_graph: &ExpressionGraph,
) -> Vec<ComponentDiagnostic> {
    let mut diagnostics = Vec::new();

    for computed in computed_values.values() {
        let component_id = computed
            .owner
            .entity_id()
            .expect("computed values should have component owners");
        let component = components
            .iter()
            .find(|component| component.id == *component_id)
            .expect("computed values should have owning components");
        let method = component
            .methods
            .iter()
            .find(|method| method.id == computed.method)
            .expect("computed values should have authored methods");

        if method.computed_expression.is_none() {
            diagnostics.push(ComponentDiagnostic {
                severity: ComponentDiagnosticSeverity::Error,
                effect_id: None,
                statement_id: None,
                context_declaration_candidate_id: None,
                context_id: None,
                provider_id: None,
                consumer_id: None,
                slot_id: None,
                invocation_id: None,
                component_instance_id: None,
                slot_binding_id: None,
                structural_region_id: None,
                component_id: None,
                provider_instance_id: None,
                consumer_instance_id: None,
                secondary_labels: Vec::new(),
                code: ComputedDiagnosticCode::UnsupportedBody.as_str().to_string(),
                message: format!(
                    "computed getter `{}` has an unsupported body",
                    computed.name
                ),
                provenance: Some(computed.provenance.clone()),
            });
        }

        diagnostics.extend(
            expression_graph
                .nodes_for(&computed.id)
                .into_iter()
                .filter_map(|node| {
                    unresolved_computed_read_diagnostic(component, computed_values, computed, node)
                }),
        );
    }

    diagnostics
}

fn unresolved_computed_read_diagnostic(
    component: &ComponentNode,
    computed_values: &BTreeMap<SemanticId, ComputedValue>,
    computed: &ComputedValue,
    node: &ExpressionNode,
) -> Option<ComponentDiagnostic> {
    let ExpressionNodeKind::ThisMember { name } = &node.kind else {
        return None;
    };
    let resolved = component
        .state_fields
        .iter()
        .any(|field| field.name == *name)
        || computed_values.contains_key(&component.id.computed(name));
    (!resolved).then(|| ComponentDiagnostic {
        severity: ComponentDiagnosticSeverity::Error,
        effect_id: None,
        statement_id: None,
        context_declaration_candidate_id: None,
        context_id: None,
        provider_id: None,
        consumer_id: None,
        slot_id: None,
        invocation_id: None,
        component_instance_id: None,
        slot_binding_id: None,
        structural_region_id: None,
        component_id: None,
        provider_instance_id: None,
        consumer_instance_id: None,
        secondary_labels: Vec::new(),
        code: ComputedDiagnosticCode::UnresolvedRead.as_str().to_string(),
        message: format!(
            "computed getter `{}` reads unresolved member `this.{name}`",
            computed.name
        ),
        provenance: Some(node.provenance.clone()),
    })
}

fn collect_computed_type_diagnostics(
    computed_values: &BTreeMap<SemanticId, ComputedValue>,
    semantic_types: &SemanticTypeModel,
) -> Vec<ComponentDiagnostic> {
    let mut diagnostics = Vec::new();

    for computed_type in semantic_types.computed_values.values() {
        let computed = computed_values
            .get(&computed_type.computed)
            .expect("computed type records should have computed entities");
        if computed_type.declared_return_compatible == Some(false) {
            let declared = computed_type
                .declared_return_type
                .as_ref()
                .expect("incompatible computed return types should be declared");
            diagnostics.push(ComponentDiagnostic {
                severity: ComponentDiagnosticSeverity::Error,
                effect_id: None,
                statement_id: None,
                context_declaration_candidate_id: None,
                context_id: None,
                provider_id: None,
                consumer_id: None,
                slot_id: None,
                invocation_id: None,
                component_instance_id: None,
                slot_binding_id: None,
                structural_region_id: None,
                component_id: None,
                provider_instance_id: None,
                consumer_instance_id: None,
                secondary_labels: Vec::new(),
                code: ComputedDiagnosticCode::TypeMismatch.as_str().to_string(),
                message: format!(
                    "computed getter `{}` returns `{}` but declares `{}`",
                    computed.name,
                    crate::semantic_type_text(&computed_type.semantic_type),
                    crate::semantic_type_text(declared)
                ),
                provenance: Some(computed_type.provenance.clone()),
            });
        }
        if computed_type.serialization == crate::SerializationCompatibility::NotSerializable {
            diagnostics.push(ComponentDiagnostic {
                severity: ComponentDiagnosticSeverity::Error,
                effect_id: None,
                statement_id: None,
                context_declaration_candidate_id: None,
                context_id: None,
                provider_id: None,
                consumer_id: None,
                slot_id: None,
                invocation_id: None,
                component_instance_id: None,
                slot_binding_id: None,
                structural_region_id: None,
                component_id: None,
                provider_instance_id: None,
                consumer_instance_id: None,
                secondary_labels: Vec::new(),
                code: ComputedDiagnosticCode::SerializationViolation
                    .as_str()
                    .to_string(),
                message: format!(
                    "computed getter `{}` has a non-serializable result",
                    computed.name
                ),
                provenance: Some(computed_type.provenance.clone()),
            });
        }
    }

    diagnostics
}

fn sort_computed_diagnostics(diagnostics: &mut [ComponentDiagnostic]) {
    diagnostics.sort_by(|left, right| {
        (
            left.code.as_str(),
            left.provenance.as_ref().map(|provenance| &provenance.path),
            left.provenance
                .as_ref()
                .map(|provenance| provenance.span.start),
            left.provenance
                .as_ref()
                .map(|provenance| provenance.span.end),
            left.message.as_str(),
        )
            .cmp(&(
                right.code.as_str(),
                right.provenance.as_ref().map(|provenance| &provenance.path),
                right
                    .provenance
                    .as_ref()
                    .map(|provenance| provenance.span.start),
                right
                    .provenance
                    .as_ref()
                    .map(|provenance| provenance.span.end),
                right.message.as_str(),
            ))
    });
}

fn build_computed_reactive_products(
    components: &[ComponentNode],
    computed_values: &BTreeMap<SemanticId, ComputedValue>,
    effects: &BTreeMap<SemanticId, Effect>,
    references: &[SemanticReference],
    provenance: &BTreeMap<SemanticId, SourceProvenance>,
) -> (
    IrReactiveGraph,
    IrReactiveTransitiveAnalysis,
    IrReactiveCycleAnalysis,
    IrComputedEvaluationPlan,
) {
    let graph =
        crate::build_reactive_graph(components, computed_values, effects, references, provenance);
    let transitive_analysis = crate::analyze_reactive_transitive_graph(&graph);
    let cycle_analysis = crate::analyze_reactive_cycles(&graph);
    let evaluation_plan = crate::plan_computed_evaluation(&graph);
    (graph, transitive_analysis, cycle_analysis, evaluation_plan)
}

fn extend_template_references(
    references: &mut Vec<SemanticReference>,
    components: &[ComponentNode],
    computed_values: &BTreeMap<SemanticId, ComputedValue>,
    template_entities: &[TemplateSemanticEntity],
    ownership: &BTreeMap<SemanticId, SemanticOwner>,
) {
    references.extend(build_template_state_references(
        components,
        template_entities,
        ownership,
    ));
    references.extend(build_template_computed_references(
        components,
        computed_values,
        template_entities,
        ownership,
    ));
    references.extend(build_template_event_references(
        components,
        template_entities,
        ownership,
    ));
    references.extend(build_template_local_references(
        components,
        template_entities,
        ownership,
    ));
}

fn build_form_field_binding_references(
    bindings: &BTreeMap<FieldBindingId, FormFieldBinding>,
) -> Vec<SemanticReference> {
    bindings
        .values()
        .flat_map(|binding| {
            [
                SemanticReference {
                    kind: SemanticReferenceKind::FieldBindingField,
                    source: binding.id.as_semantic_id().clone(),
                    target: binding.field.as_semantic_id().clone(),
                    provenance: binding.expression_provenance.clone(),
                },
                SemanticReference {
                    kind: SemanticReferenceKind::FieldBindingForm,
                    source: binding.id.as_semantic_id().clone(),
                    target: binding.form.as_semantic_id().clone(),
                    provenance: binding.expression_provenance.clone(),
                },
            ]
        })
        .collect()
}

fn computed_call_purity_kind(
    component: &ComponentNode,
    callee: &str,
) -> crate::ComputedPurityViolationKind {
    if matches!(callee, "resource" | "this.resource") {
        return crate::ComputedPurityViolationKind::Resource;
    }
    if matches!(
        callee,
        "Math.random"
            | "Date.now"
            | "performance.now"
            | "crypto.randomUUID"
            | "crypto.getRandomValues"
    ) {
        return crate::ComputedPurityViolationKind::NondeterministicOperation;
    }
    if callee.starts_with("console.")
        || matches!(
            callee,
            "fetch" | "setTimeout" | "setInterval" | "queueMicrotask"
        )
    {
        return crate::ComputedPurityViolationKind::Effect;
    }
    if let Some(name) = callee.strip_prefix("this.") {
        if let Some(method) = component.methods.iter().find(|method| method.name == name) {
            if method.is_action()
                || method
                    .decorators
                    .iter()
                    .any(|decorator| decorator == "action")
            {
                return crate::ComputedPurityViolationKind::Action;
            }
            if method
                .decorators
                .iter()
                .any(|decorator| decorator == "effect")
            {
                return crate::ComputedPurityViolationKind::Effect;
            }
            if method
                .decorators
                .iter()
                .any(|decorator| decorator == "resource")
            {
                return crate::ComputedPurityViolationKind::Resource;
            }
        }
    }
    crate::ComputedPurityViolationKind::ArbitraryMethodCall
}

fn build_computed_references(
    components: &[ComponentNode],
    computed_values: &BTreeMap<SemanticId, ComputedValue>,
    expression_graph: &ExpressionGraph,
) -> Vec<SemanticReference> {
    let mut references = BTreeMap::new();

    for computed in computed_values.values() {
        let Some(component_id) = computed.owner.entity_id() else {
            continue;
        };
        let Some(component) = components
            .iter()
            .find(|component| component.id == *component_id)
        else {
            continue;
        };

        for node in expression_graph.nodes_for(&computed.id) {
            let crate::ExpressionNodeKind::ThisMember { name } = &node.kind else {
                continue;
            };
            let reference = if let Some(field) = component
                .state_fields
                .iter()
                .find(|field| field.name == *name)
            {
                SemanticReference {
                    kind: SemanticReferenceKind::ComputedState,
                    source: computed.id.clone(),
                    target: field.id.clone(),
                    provenance: computed.provenance.clone(),
                }
            } else if let Some(target) = computed_values.get(&component.id.computed(name)) {
                SemanticReference {
                    kind: SemanticReferenceKind::ComputedComputed,
                    source: computed.id.clone(),
                    target: target.id.clone(),
                    provenance: computed.provenance.clone(),
                }
            } else {
                continue;
            };
            references.insert(
                (reference.source.clone(), reference.target.clone()),
                reference,
            );
        }
    }

    references.into_values().collect()
}

fn build_effect_references(
    components: &[ComponentNode],
    effects: &BTreeMap<SemanticId, Effect>,
    computed_values: &BTreeMap<SemanticId, ComputedValue>,
    expression_graph: &ExpressionGraph,
) -> Vec<SemanticReference> {
    let mut references = Vec::new();
    for effect in effects.values() {
        let Some(component_id) = effect.owner.entity_id() else {
            continue;
        };
        let Some(component) = components
            .iter()
            .find(|component| component.id == *component_id)
        else {
            continue;
        };
        for node in expression_graph.nodes_for(&effect.id) {
            let crate::ExpressionNodeKind::ThisMember { name } = &node.kind else {
                continue;
            };
            let reference = if let Some(field) = component
                .state_fields
                .iter()
                .find(|field| field.name == *name)
            {
                SemanticReference {
                    kind: SemanticReferenceKind::EffectState,
                    source: effect.id.clone(),
                    target: field.id.clone(),
                    provenance: node.provenance.clone(),
                }
            } else if let Some(computed) = computed_values.get(&component.id.computed(name)) {
                SemanticReference {
                    kind: SemanticReferenceKind::EffectComputed,
                    source: effect.id.clone(),
                    target: computed.id.clone(),
                    provenance: node.provenance.clone(),
                }
            } else {
                continue;
            };
            references.push(reference);
        }
    }
    references.sort_by(|left, right| {
        (
            left.source.as_str(),
            left.target.as_str(),
            left.provenance.span.start,
        )
            .cmp(&(
                right.source.as_str(),
                right.target.as_str(),
                right.provenance.span.start,
            ))
    });
    references.dedup_by(|left, right| {
        left.kind == right.kind && left.source == right.source && left.target == right.target
    });
    references
}

fn build_provider_references(
    providers: &BTreeMap<ProviderId, ProviderEntity>,
) -> Vec<SemanticReference> {
    providers
        .values()
        .map(|provider| SemanticReference {
            kind: SemanticReferenceKind::ProvidesContext,
            source: provider.id.as_semantic_id().clone(),
            target: provider.context.as_semantic_id().clone(),
            provenance: provider.provenance.clone(),
        })
        .collect()
}

fn build_consumer_references(
    consumers: &BTreeMap<ConsumerId, ConsumerEntity>,
) -> Vec<SemanticReference> {
    consumers
        .values()
        .filter_map(|consumer| {
            let ContextResolutionState::Resolved(context) = &consumer.context_resolution else {
                return None;
            };
            Some(SemanticReference {
                kind: SemanticReferenceKind::ConsumesContext,
                source: consumer.id.as_semantic_id().clone(),
                target: context.as_semantic_id().clone(),
                provenance: consumer.provenance.clone(),
            })
        })
        .collect()
}

fn build_context_resolution_references(
    resolutions: &BTreeMap<ConsumerId, ContextResolution>,
) -> Vec<SemanticReference> {
    resolutions
        .values()
        .filter_map(|resolution| {
            let ContextResolutionResult::Provider { provider, .. } = &resolution.result else {
                return None;
            };
            Some(SemanticReference {
                kind: SemanticReferenceKind::ResolvesToProvider,
                source: resolution.consumer.as_semantic_id().clone(),
                target: provider.as_semantic_id().clone(),
                provenance: resolution.provenance.clone(),
            })
        })
        .collect()
}

fn build_template_state_references(
    components: &[ComponentNode],
    template_entities: &[TemplateSemanticEntity],
    ownership: &BTreeMap<SemanticId, SemanticOwner>,
) -> Vec<SemanticReference> {
    template_entities
        .iter()
        .filter(|entity| {
            matches!(
                entity.kind,
                TemplateSemanticKind::Binding
                    | TemplateSemanticKind::AttributeBinding
                    | TemplateSemanticKind::Conditional
                    | TemplateSemanticKind::List
            )
        })
        .filter_map(|entity| {
            let field_name = entity.expression.as_deref().and_then(this_member_name)?;
            let component = template_entity_component(components, ownership, entity)?;
            let field = component
                .state_fields
                .iter()
                .find(|field| field.name == field_name)?;

            Some(SemanticReference {
                kind: SemanticReferenceKind::TemplateState,
                source: entity.id.clone(),
                target: field.id.clone(),
                provenance: entity.provenance.clone(),
            })
        })
        .collect()
}

fn build_template_computed_references(
    components: &[ComponentNode],
    computed_values: &BTreeMap<SemanticId, ComputedValue>,
    template_entities: &[TemplateSemanticEntity],
    ownership: &BTreeMap<SemanticId, SemanticOwner>,
) -> Vec<SemanticReference> {
    template_entities
        .iter()
        .filter(|entity| {
            matches!(
                entity.kind,
                TemplateSemanticKind::Binding
                    | TemplateSemanticKind::AttributeBinding
                    | TemplateSemanticKind::Conditional
                    | TemplateSemanticKind::List
            )
        })
        .filter_map(|entity| {
            let computed_name = entity.expression.as_deref().and_then(this_member_name)?;
            let component = template_entity_component(components, ownership, entity)?;
            if component
                .state_fields
                .iter()
                .any(|field| field.name == computed_name)
            {
                return None;
            }
            let computed = computed_values.get(&component.id.computed(computed_name))?;

            Some(SemanticReference {
                kind: SemanticReferenceKind::TemplateComputed,
                source: entity.id.clone(),
                target: computed.id.clone(),
                provenance: entity.provenance.clone(),
            })
        })
        .collect()
}

fn build_template_event_references(
    components: &[ComponentNode],
    template_entities: &[TemplateSemanticEntity],
    ownership: &BTreeMap<SemanticId, SemanticOwner>,
) -> Vec<SemanticReference> {
    template_entities
        .iter()
        .filter(|entity| entity.kind == TemplateSemanticKind::EventAttribute)
        .filter_map(|entity| {
            let method_name = entity.expression.as_deref().and_then(this_member_name)?;
            let component = template_entity_component(components, ownership, entity)?;
            let method = component
                .methods
                .iter()
                .find(|method| method.name == method_name)?;

            Some(SemanticReference {
                kind: SemanticReferenceKind::EventMethod,
                source: entity.id.clone(),
                target: method.id.clone(),
                provenance: entity.provenance.clone(),
            })
        })
        .collect()
}

fn build_template_local_references(
    components: &[ComponentNode],
    template_entities: &[TemplateSemanticEntity],
    ownership: &BTreeMap<SemanticId, SemanticOwner>,
) -> Vec<SemanticReference> {
    let mut references = template_entities
        .iter()
        .filter(|entity| {
            matches!(
                entity.kind,
                TemplateSemanticKind::Binding | TemplateSemanticKind::AttributeBinding
            ) && entity.scope == TemplateSemanticScope::Render
        })
        .filter_map(|entity| {
            let name = entity.expression.as_deref()?;
            let component = template_entity_component(components, ownership, entity)?;
            let render = component
                .methods
                .iter()
                .find(|method| method.name == "render")?;
            let local = unique_local_variable(render, name)?;

            Some(SemanticReference {
                kind: SemanticReferenceKind::TemplateLocal,
                source: entity.id.clone(),
                target: local.id.clone(),
                provenance: entity.provenance.clone(),
            })
        })
        .collect::<Vec<_>>();
    references.sort_by(|left, right| {
        (left.source.as_str(), left.target.as_str())
            .cmp(&(right.source.as_str(), right.target.as_str()))
    });
    references
}

fn unique_local_variable<'a>(
    method: &'a ComponentMethod,
    name: &str,
) -> Option<&'a MethodLocalVariable> {
    let mut locals = method
        .local_variables
        .iter()
        .filter(|local| local.name == name);
    let local = locals.next()?;
    locals.next().is_none().then_some(local)
}

fn template_entity_component<'a>(
    components: &'a [ComponentNode],
    ownership: &BTreeMap<SemanticId, SemanticOwner>,
    entity: &TemplateSemanticEntity,
) -> Option<&'a ComponentNode> {
    let template_id = ownership.get(&entity.id)?.entity_id()?;
    let component_id = ownership.get(template_id)?.entity_id()?;

    components
        .iter()
        .find(|component| component.id == *component_id)
}

fn this_member_name(expression: &str) -> Option<&str> {
    expression.strip_prefix("this.").filter(|name| {
        !name.is_empty()
            && name
                .bytes()
                .all(|byte| byte.is_ascii_alphanumeric() || byte == b'_')
    })
}

#[allow(clippy::too_many_arguments, clippy::too_many_lines)]
fn collect_ownership(
    components: &[ComponentNode],
    contexts: &BTreeMap<ContextId, ContextEntity>,
    forms: &BTreeMap<FormId, FormEntity>,
    form_fields: &BTreeMap<FieldId, FormFieldEntity>,
    form_field_bindings: &BTreeMap<FieldBindingId, FormFieldBinding>,
    providers: &BTreeMap<ProviderId, ProviderEntity>,
    consumers: &BTreeMap<ConsumerId, ConsumerEntity>,
    slots: &BTreeMap<SlotId, SlotEntity>,
    component_invocations: &BTreeMap<ComponentInvocationId, ComponentInvocationEntity>,
    slot_content_fragments: &BTreeMap<SlotContentFragmentId, SlotContentFragment>,
    slot_outlets: &BTreeMap<SlotOutletId, SlotOutlet>,
    component_instance_plan: &ComponentInstancePlan,
    computed_values: &BTreeMap<SemanticId, ComputedValue>,
    effects: &BTreeMap<SemanticId, Effect>,
    templates: &[TemplateNode],
    template_entities: &[TemplateSemanticEntity],
) -> BTreeMap<SemanticId, SemanticOwner> {
    let mut ownership = BTreeMap::new();

    for component in components {
        ownership.insert(component.id.clone(), SemanticOwner::Application);

        for field in &component.state_fields {
            ownership.insert(
                field.id.clone(),
                SemanticOwner::entity(component.id.clone()),
            );
        }
        for method in &component.methods {
            ownership.insert(
                method.id.clone(),
                SemanticOwner::entity(component.id.clone()),
            );
            for parameter in &method.parameters {
                ownership.insert(
                    parameter.id.clone(),
                    SemanticOwner::entity(method.id.clone()),
                );
            }
            for local in &method.local_variables {
                ownership.insert(local.id.clone(), SemanticOwner::entity(method.id.clone()));
            }
        }
        for computed in computed_values
            .values()
            .filter(|computed| computed.owner.entity_id() == Some(&component.id))
        {
            ownership.insert(computed.id.clone(), computed.owner.clone());
        }
        for context in contexts
            .values()
            .filter(|context| context.owner.entity_id() == Some(&component.id))
        {
            ownership.insert(context.id.as_semantic_id().clone(), context.owner.clone());
        }
        for form in forms
            .values()
            .filter(|form| form.owner.entity_id() == Some(&component.id))
        {
            ownership.insert(form.id.as_semantic_id().clone(), form.owner.clone());
        }
        for field in form_fields
            .values()
            .filter(|field| field.owner_component == component.id)
        {
            ownership.insert(
                field.id.as_semantic_id().clone(),
                SemanticOwner::entity(field.owner_form.as_semantic_id().clone()),
            );
        }
        for provider in providers
            .values()
            .filter(|provider| provider.owner.entity_id() == Some(&component.id))
        {
            ownership.insert(provider.id.as_semantic_id().clone(), provider.owner.clone());
        }
        for consumer in consumers
            .values()
            .filter(|consumer| consumer.owner.entity_id() == Some(&component.id))
        {
            ownership.insert(consumer.id.as_semantic_id().clone(), consumer.owner.clone());
        }
        for slot in slots.values().filter(|slot| slot.owner == component.id) {
            ownership.insert(slot.id.as_semantic_id().clone(), slot.semantic_owner());
        }
        for invocation in component_invocations
            .values()
            .filter(|invocation| invocation.owner_component == component.id)
        {
            ownership.insert(
                invocation.id.as_semantic_id().clone(),
                SemanticOwner::entity(component.id.clone()),
            );
        }
        for fragment in slot_content_fragments
            .values()
            .filter(|fragment| fragment.owner_component == component.id)
        {
            ownership.insert(
                fragment.id.as_semantic_id().clone(),
                SemanticOwner::entity(component.id.clone()),
            );
        }
        for outlet in slot_outlets
            .values()
            .filter(|outlet| outlet.owner_component == component.id)
        {
            ownership.insert(
                outlet.id.as_semantic_id().clone(),
                SemanticOwner::entity(component.id.clone()),
            );
        }
        for effect in effects
            .values()
            .filter(|effect| effect.owner.entity_id() == Some(&component.id))
        {
            ownership.insert(effect.id.clone(), effect.owner.clone());
        }
        for action in &component.actions {
            ownership.insert(
                action.id.clone(),
                SemanticOwner::entity(component.id.method(&action.method)),
            );
        }
        if let Some(render) = &component.render {
            for handler in render_event_handlers(render) {
                ownership.insert(
                    handler.id.clone(),
                    SemanticOwner::entity(component.id.template()),
                );
            }
        }
    }

    for instance in component_instance_plan.instances.values() {
        ownership.insert(
            instance.id.as_semantic_id().clone(),
            instance
                .parent_instance
                .as_ref()
                .map_or(SemanticOwner::Application, |parent| {
                    SemanticOwner::entity(parent.as_semantic_id().clone())
                }),
        );
    }
    for blocked in component_instance_plan.blocked.values() {
        ownership.insert(
            blocked.id.as_semantic_id().clone(),
            SemanticOwner::entity(blocked.parent_instance.as_semantic_id().clone()),
        );
    }

    for template in templates {
        let component = components
            .iter()
            .find(|component| component.id.template() == template.id)
            .expect("template graph should only contain component templates");
        ownership.insert(
            template.id.clone(),
            SemanticOwner::entity(component.id.clone()),
        );
    }

    for binding in form_field_bindings.values() {
        ownership.insert(
            binding.id.as_semantic_id().clone(),
            SemanticOwner::entity(binding.control_entity.clone()),
        );
    }

    for entity in template_entities {
        ownership.insert(entity.id.clone(), entity.owner.clone());
    }

    ownership
}

#[cfg(test)]
mod tests {
    use super::{
        build_application_semantic_model, build_application_semantic_model_for_unit,
        collect_ownership,
    };
    use crate::{
        build_component_graph_for_module, build_template_graph, build_template_semantic_entities,
        CompilationUnit, ComponentInstancePlan, SemanticEntity, SemanticEntityKind, SemanticOwner,
        SemanticReferenceKind, TemplateSemanticKind,
    };

    #[test]
    fn lowers_supported_primitive_state_annotations_into_canonical_types() {
        let parsed = ezc_parser::parse_file(
            "src/TypedState.tsx",
            r#"
@component("x-typed-state")
class TypedState extends Component {
  count: number = state(0);
}
"#,
        );

        let asm = build_application_semantic_model(&parsed);

        let field = &asm.components[0].state_fields[0];
        let assignment = asm
            .semantic_types
            .assignments
            .get(&field.id)
            .expect("canonical declared type");

        assert_eq!(assignment.semantic_type, crate::SemanticType::Number);
        assert_eq!(assignment.status, crate::SemanticTypeStatus::Declared);
        assert_eq!(
            assignment.provenance,
            field.declared_type.as_ref().unwrap().provenance
        );
    }

    #[test]
    fn queries_canonical_type_information_from_the_asm() {
        let parsed = ezc_parser::parse_file(
            "src/TypeQueries.tsx",
            r#"
@component("x-type-queries")
class TypeQueries extends Component {
  count: number = state(0);
  label = state("EdgeZero");
}
"#,
        );
        let asm = build_application_semantic_model(&parsed);
        let count = &asm.components[0].state_fields[0];
        let label = &asm.components[0].state_fields[1];

        assert_eq!(
            asm.semantic_type_of(&count.id),
            Some(&crate::SemanticType::Number)
        );
        assert_eq!(asm.type_declarations(&crate::SemanticType::Number).len(), 1);
        assert_eq!(asm.type_usages(&crate::SemanticType::String).len(), 1);
        assert_eq!(
            asm.serialization_compatibility_of(&count.id),
            Some(crate::SerializationCompatibility::Serializable)
        );
        assert_eq!(asm.is_type_assignable(&label.id, &count.id), Some(false));
    }

    #[test]
    fn lowers_typed_method_parameters_into_canonical_entities() {
        let parsed = ezc_parser::parse_file(
            "src/Parameters.tsx",
            r#"
@component("x-parameters")
class Parameters extends Component {
  save(title: string, retries?: number) {}
}
"#,
        );

        let asm = build_application_semantic_model(&parsed);
        let method = &asm.components[0].methods[0];
        let parameters = &method.parameters;

        assert_eq!(parameters.len(), 2);
        assert_eq!(
            asm.semantic_types.assignments[&parameters[0].id].semantic_type,
            crate::SemanticType::String
        );
        assert_eq!(
            asm.semantic_types.assignments[&parameters[1].id].semantic_type,
            crate::SemanticType::Number
        );
        assert!(parameters.iter().all(|parameter| {
            asm.owner(&parameter.id) == Some(&SemanticOwner::entity(method.id.clone()))
                && asm
                    .entity(&parameter.id)
                    .is_some_and(|entity| entity.kind() == SemanticEntityKind::Parameter)
        }));
    }

    #[test]
    fn lowers_declared_and_inferred_method_return_types() {
        let parsed = ezc_parser::parse_file(
            "src/Returns.tsx",
            r#"
@component("x-returns")
class Returns extends Component {
  declared(): string { return "EdgeZero"; }
  inferred() { return 1; }
}
"#,
        );

        let asm = build_application_semantic_model(&parsed);
        let methods = &asm.components[0].methods;
        let declared = &methods[0];
        let inferred = &methods[1];

        assert_eq!(
            asm.semantic_types.assignments[&declared.id].semantic_type,
            crate::SemanticType::String
        );
        assert_eq!(
            asm.semantic_types.assignments[&declared.id].status,
            crate::SemanticTypeStatus::Declared
        );
        assert_eq!(
            asm.semantic_types.assignments[&inferred.id].semantic_type,
            crate::SemanticType::Number
        );
        assert_eq!(
            asm.semantic_types.assignments[&inferred.id].status,
            crate::SemanticTypeStatus::Inferred
        );
    }

    #[test]
    fn establishes_typed_computed_getter_contracts() {
        let parsed = ezc_parser::parse_file(
            "src/Computed.tsx",
            r#"
@component("x-computed")
class Computed extends Component {
  @computed()
  get remainingCount(): number { return 1; }

  render() { return <p />; }
}
"#,
        );

        let asm = build_application_semantic_model(&parsed);
        let method = &asm.components[0].methods[0];
        let computed_id = asm.components[0].id.computed("remainingCount");
        let computed = asm
            .semantic_types
            .computed_values
            .get(&computed_id)
            .expect("computed type contract");
        let computed_entity = asm
            .computed_value(&computed_id)
            .expect("first-class computed entity");

        assert!(method.is_getter);
        assert!(method.is_computed());
        assert_eq!(
            computed.semantic_type,
            crate::SemanticType::NumberLiteral("1".to_string())
        );
        assert_eq!(
            computed.declared_return_type,
            Some(crate::SemanticType::Number)
        );
        assert_eq!(computed.declared_return_compatible, Some(true));
        assert_eq!(computed_entity.method, method.id);
        assert_eq!(
            computed_entity.owner,
            crate::SemanticOwner::entity(asm.components[0].id.clone())
        );
        assert_eq!(
            computed_entity.cache_policy,
            crate::ComputedCachePolicy::Memoized
        );
        assert_eq!(computed_entity.purity, crate::ComputedPurity::Pure);
        assert!(computed_entity.purity_violations.is_empty());
        assert_eq!(
            computed_entity.execution_boundary,
            crate::ExecutionBoundary::Client
        );
        assert_eq!(
            asm.entity(&computed_id),
            Some(SemanticEntity::Computed(computed_entity))
        );
        assert_eq!(
            asm.owner(&computed_id),
            Some(&crate::SemanticOwner::entity(asm.components[0].id.clone()))
        );
        assert_eq!(asm.provenance(&computed_id), asm.provenance(&method.id));
        assert_eq!(
            asm.entities_of_kind(SemanticEntityKind::Computed),
            vec![&computed_id]
        );
    }

    #[test]
    fn resolves_computed_reads_to_canonical_state_and_computed_references() {
        let parsed = ezc_parser::parse_file(
            "src/ComputedReads.tsx",
            r#"
@component("x-computed-reads")
class ComputedReads extends Component {
  count = state(1);
  profile = state({ hidden: false });

  @computed()
  get doubled() { return this.count * 2; }

  @computed()
  get visible() { return this.doubled + this.count + this.count; }

  @computed()
  get profileHidden() { return this.profile.hidden; }

  @computed()
  get unresolved() { return this.missing; }
}
"#,
        );
        let asm = build_application_semantic_model(&parsed);
        let component = &asm.components[0];
        let count = component.id.state_field("count");
        let profile = component.id.state_field("profile");
        let doubled = component.id.computed("doubled");
        let visible = component.id.computed("visible");
        let profile_hidden = component.id.computed("profileHidden");
        let unresolved = component.id.computed("unresolved");

        let state_references = asm.references_of_kind(SemanticReferenceKind::ComputedState);
        assert_eq!(state_references.len(), 3);
        assert!(state_references
            .iter()
            .any(|reference| reference.source == doubled && reference.target == count));
        assert!(state_references
            .iter()
            .any(|reference| reference.source == visible && reference.target == count));
        assert!(state_references.iter().any(|reference| {
            reference.source == profile_hidden && reference.target == profile
        }));

        let computed_references = asm.references_of_kind(SemanticReferenceKind::ComputedComputed);
        assert_eq!(computed_references.len(), 1);
        assert_eq!(computed_references[0].source, visible);
        assert_eq!(computed_references[0].target, doubled);
        assert!(!asm
            .references
            .iter()
            .any(|reference| reference.source == unresolved));
        assert!(asm
            .references
            .iter()
            .all(|reference| { asm.provenance(&reference.source) == Some(&reference.provenance) }));
    }

    #[test]
    fn populates_reactive_graph_from_direct_computed_reads() {
        let parsed = ezc_parser::parse_file(
            "src/ComputedReactiveGraph.tsx",
            r#"
@component("x-computed-reactive-graph")
class ComputedReactiveGraph extends Component {
  count = state(1);

  @computed()
  get doubled() { return this.count * 2; }

  @computed()
  get label() { return this.doubled + 1; }
}
"#,
        );
        let asm = build_application_semantic_model(&parsed);
        let component = &asm.components[0];
        let count = component.id.state_field("count");
        let doubled = component.id.computed("doubled");
        let label = component.id.computed("label");
        let graph = &asm.reactive_graph;

        assert_eq!(graph.nodes.len(), 3);
        assert_eq!(
            graph.nodes[count.as_str()].kind,
            crate::IrReactiveNodeKind::State
        );
        assert_eq!(
            graph.nodes[doubled.as_str()].kind,
            crate::IrReactiveNodeKind::Computed
        );
        assert_eq!(
            graph.nodes[label.as_str()].kind,
            crate::IrReactiveNodeKind::Computed
        );

        let doubled_dependencies = graph.computed_dependencies(doubled.as_str());
        assert_eq!(doubled_dependencies.len(), 1);
        assert_eq!(doubled_dependencies[0].target, count.as_str());
        let label_dependencies = graph.computed_dependencies(label.as_str());
        assert_eq!(label_dependencies.len(), 1);
        assert_eq!(label_dependencies[0].target, doubled.as_str());

        let count_invalidations = graph.invalidations_from(count.as_str());
        assert_eq!(count_invalidations.len(), 1);
        assert_eq!(count_invalidations[0].target, doubled.as_str());
        let doubled_invalidations = graph.invalidations_from(doubled.as_str());
        assert_eq!(doubled_invalidations.len(), 1);
        assert_eq!(doubled_invalidations[0].target, label.as_str());
        assert!(graph
            .invalidations_from(count.as_str())
            .iter()
            .all(|edge| { edge.target != label.as_str() }));
    }

    #[test]
    fn computes_deterministic_transitive_reactive_dependencies_and_dependents() {
        let parsed = ezc_parser::parse_file(
            "src/ComputedTransitiveReactiveGraph.tsx",
            r#"
@component("x-computed-transitive-reactive-graph")
class ComputedTransitiveReactiveGraph extends Component {
  count = state(1);

  @computed()
  get doubled() { return this.count * 2; }

  @computed()
  get label() { return this.doubled + 1; }
}
"#,
        );
        let asm = build_application_semantic_model(&parsed);
        let component = &asm.components[0];
        let count = component.id.state_field("count");
        let doubled = component.id.computed("doubled");
        let label = component.id.computed("label");
        let analysis = &asm.reactive_transitive_analysis;

        assert_eq!(analysis.dependencies.len(), 3);
        assert_eq!(analysis.dependents.len(), 3);
        assert_eq!(
            analysis.dependencies_of(count.as_str()),
            Vec::<String>::new()
        );
        assert_eq!(
            analysis.dependencies_of(doubled.as_str()),
            vec![count.as_str().to_string()]
        );
        assert_eq!(
            analysis.dependencies_of(label.as_str()),
            vec![doubled.as_str().to_string(), count.as_str().to_string()]
        );
        assert_eq!(
            analysis.dependents_of(count.as_str()),
            vec![doubled.as_str().to_string(), label.as_str().to_string()]
        );
        assert_eq!(
            analysis.dependents_of(doubled.as_str()),
            vec![label.as_str().to_string()]
        );
        assert_eq!(analysis.dependents_of(label.as_str()), Vec::<String>::new());
    }

    #[test]
    fn detects_computed_dependency_cycles_with_stable_diagnostics() {
        let parsed = ezc_parser::parse_file(
            "src/ComputedDependencyCycles.tsx",
            r#"
@component("x-computed-dependency-cycles")
class ComputedDependencyCycles extends Component {
  @computed()
  get alpha() { return this.beta; }

  @computed()
  get beta() { return this.alpha; }

  @computed()
  get selfLoop() { return this.selfLoop; }
}
"#,
        );
        let asm = build_application_semantic_model(&parsed);
        let component = &asm.components[0];
        let alpha = component.id.computed("alpha");
        let beta = component.id.computed("beta");
        let self_loop = component.id.computed("selfLoop");

        assert_eq!(
            asm.reactive_cycle_analysis.cycles,
            vec![
                crate::IrReactiveCycle {
                    nodes: vec![alpha.as_str().to_string(), beta.as_str().to_string()],
                },
                crate::IrReactiveCycle {
                    nodes: vec![self_loop.as_str().to_string()],
                },
            ]
        );
        let diagnostics = asm
            .diagnostics
            .iter()
            .filter(|diagnostic| diagnostic.code == "EZC1035")
            .collect::<Vec<_>>();
        assert_eq!(diagnostics.len(), 2);
        assert_eq!(
            diagnostics[0].message,
            format!("computed dependency cycle detected among: {alpha}, {beta}")
        );
        assert_eq!(
            diagnostics[0].provenance,
            Some(
                asm.computed_value(&alpha)
                    .expect("alpha computed value")
                    .provenance
                    .clone()
            )
        );
        assert_eq!(
            diagnostics[1].message,
            format!("computed dependency cycle detected among: {self_loop}")
        );
        assert_eq!(
            diagnostics[1].provenance,
            Some(
                asm.computed_value(&self_loop)
                    .expect("self-loop computed value")
                    .provenance
                    .clone()
            )
        );
    }

    #[test]
    fn plans_deterministic_computed_evaluation_order_and_batches() {
        let parsed = ezc_parser::parse_file(
            "src/ComputedEvaluationPlan.tsx",
            r#"
@component("x-computed-evaluation-plan")
class ComputedEvaluationPlan extends Component {
  count = state(1);
  other = state(2);

  @computed()
  get alpha() { return this.count; }

  @computed()
  get beta() { return this.other; }

  @computed()
  get gamma() { return this.alpha; }
}
"#,
        );
        let asm = build_application_semantic_model(&parsed);
        let component = &asm.components[0];
        let alpha = component.id.computed("alpha");
        let beta = component.id.computed("beta");
        let gamma = component.id.computed("gamma");

        assert_eq!(
            asm.computed_evaluation_plan.evaluation_order,
            vec![
                alpha.as_str().to_string(),
                beta.as_str().to_string(),
                gamma.as_str().to_string(),
            ]
        );
        assert_eq!(
            asm.computed_evaluation_plan.update_batches,
            vec![
                vec![alpha.as_str().to_string(), beta.as_str().to_string()],
                vec![gamma.as_str().to_string()],
            ]
        );
        assert!(asm.computed_evaluation_plan.unplanned.is_empty());
    }

    #[test]
    fn leaves_cyclic_computed_values_unplanned() {
        let parsed = ezc_parser::parse_file(
            "src/CyclicComputedEvaluationPlan.tsx",
            r#"
@component("x-cyclic-computed-evaluation-plan")
class CyclicComputedEvaluationPlan extends Component {
  @computed()
  get alpha() { return this.beta; }

  @computed()
  get beta() { return this.alpha; }
}
"#,
        );
        let asm = build_application_semantic_model(&parsed);
        let component = &asm.components[0];
        let alpha = component.id.computed("alpha");
        let beta = component.id.computed("beta");

        assert!(asm.computed_evaluation_plan.evaluation_order.is_empty());
        assert!(asm.computed_evaluation_plan.update_batches.is_empty());
        assert_eq!(
            asm.computed_evaluation_plan.unplanned,
            vec![alpha.as_str().to_string(), beta.as_str().to_string()]
        );
    }

    #[test]
    fn assigns_inferred_computed_types_and_validates_declared_returns() {
        let parsed = ezc_parser::parse_file(
            "src/ComputedTypes.tsx",
            r#"
@component("x-computed-types")
class ComputedTypes extends Component {
  count: number = state(1);
  profile = state({ label: "EdgeZero" });

  @computed()
  get doubled(): number { return this.count * 2; }

  @computed()
  get chained() { return this.doubled + 1; }

  @computed()
  get label(): string { return this.profile.label; }

  @computed()
  get invalid(): number { return "wrong"; }

  @computed()
  get unresolved() { return this.missing; }
}
"#,
        );
        let asm = build_application_semantic_model(&parsed);
        let component = &asm.components[0];
        let doubled = component.id.computed("doubled");
        let chained = component.id.computed("chained");
        let label = component.id.computed("label");
        let invalid = component.id.computed("invalid");
        let unresolved = component.id.computed("unresolved");

        let types = &asm.semantic_types.computed_values;
        assert_eq!(types[&doubled].semantic_type, crate::SemanticType::Number);
        assert_eq!(types[&chained].semantic_type, crate::SemanticType::Number);
        assert_eq!(types[&label].semantic_type, crate::SemanticType::String);
        assert_eq!(
            types[&invalid].semantic_type,
            crate::SemanticType::StringLiteral("wrong".to_string())
        );
        assert_eq!(types[&invalid].declared_return_compatible, Some(false));
        assert_eq!(types[&doubled].declared_return_compatible, Some(true));
        assert_eq!(
            types[&unresolved].semantic_type,
            crate::SemanticType::Unknown
        );
        assert_eq!(
            types[&unresolved].serialization,
            crate::SerializationCompatibility::NotSerializable
        );
        assert_eq!(
            types[&doubled].boundary_compatibility,
            crate::BoundaryCompatibility::Compatible
        );
        assert_eq!(
            types[&doubled].execution_boundary,
            crate::ExecutionBoundary::Client
        );
        assert_eq!(
            asm.semantic_type_of(&doubled),
            Some(&crate::SemanticType::Number)
        );
        assert_eq!(
            asm.serialization_compatibility_of(&chained),
            Some(crate::SerializationCompatibility::Serializable)
        );
    }

    #[test]
    fn reports_stable_diagnostics_for_computed_semantics() {
        let parsed = ezc_parser::parse_file(
            "src/ComputedDiagnostics.tsx",
            r#"
@component("x-computed-diagnostics")
class ComputedDiagnostics extends Component {
  count = state(1);

  @computed()
  invalidDeclaration() { return 1; }

  @computed()
  get unsupportedBody() {
    const value = this.count;
    return value;
  }

  @computed()
  get unresolvedRead() { return this.missing; }

  @computed()
  get mismatch(): number { return "wrong"; }

  @computed()
  get impure() { return helper(); }

  @computed()
  get cycleA() { return this.cycleB; }

  @computed()
  get cycleB() { return this.cycleA; }

  render() { return <div />; }
}
"#,
        );

        let asm = build_application_semantic_model(&parsed);
        let repeated = build_application_semantic_model(&parsed);
        assert_eq!(asm.diagnostics, repeated.diagnostics);

        let component_graph = crate::build_component_graph(&parsed);
        let asm_from_component_graph =
            super::build_application_semantic_model_from_component_graph(&component_graph);

        let codes = asm
            .diagnostics
            .iter()
            .map(|diagnostic| diagnostic.code.as_str())
            .collect::<std::collections::BTreeSet<_>>();
        assert_eq!(
            codes,
            std::collections::BTreeSet::from([
                "EZC1034", "EZC1035", "EZC1036", "EZC1037", "EZC1038", "EZC1039", "EZC1040",
            ])
        );
        assert_eq!(
            asm_from_component_graph
                .diagnostics
                .iter()
                .map(|diagnostic| diagnostic.code.as_str())
                .collect::<std::collections::BTreeSet<_>>(),
            codes
        );
        assert!(asm
            .diagnostics
            .iter()
            .all(|diagnostic| diagnostic.provenance.is_some()));
        assert!(asm.diagnostics.iter().any(|diagnostic| {
            diagnostic.code == crate::ComputedDiagnosticCode::InvalidDeclaration.as_str()
                && diagnostic.message
                    == "@computed() declaration `invalidDeclaration` must decorate a getter"
        }));
        assert!(asm.diagnostics.iter().any(|diagnostic| {
            diagnostic.code == crate::ComputedDiagnosticCode::UnsupportedBody.as_str()
                && diagnostic.message == "computed getter `unsupportedBody` has an unsupported body"
        }));
        assert!(asm.diagnostics.iter().any(|diagnostic| {
            diagnostic.code == crate::ComputedDiagnosticCode::UnresolvedRead.as_str()
                && diagnostic.message
                    == "computed getter `unresolvedRead` reads unresolved member `this.missing`"
        }));
        assert!(asm.diagnostics.iter().any(|diagnostic| {
            diagnostic.code == crate::ComputedDiagnosticCode::TypeMismatch.as_str()
                && diagnostic.message
                    == "computed getter `mismatch` returns `\"wrong\"` but declares `number`"
        }));
        assert!(asm.diagnostics.iter().any(|diagnostic| {
            diagnostic.code == crate::ComputedDiagnosticCode::SerializationViolation.as_str()
                && diagnostic.message
                    == "computed getter `unresolvedRead` has a non-serializable result"
        }));
    }

    #[test]
    fn phase_e_audit_preserves_canonical_computed_products() {
        let path = "fixtures/0047-computed-diamond/input/ComputedDiamond.tsx";
        let parsed = ezc_parser::parse_file(
            path,
            include_str!("../../../fixtures/0047-computed-diamond/input/ComputedDiamond.tsx"),
        );
        let asm = build_application_semantic_model(&parsed);
        let component_graph = crate::build_component_graph(&parsed);
        let asm_from_component_graph =
            super::build_application_semantic_model_from_component_graph(&component_graph);
        let component = &asm.components[0];
        let count = component.id.state_field("count");
        let doubled = component.id.computed("doubled");
        let tripled = component.id.computed("tripled");
        let total = component.id.computed("total");
        let expected_owner = crate::SemanticOwner::entity(component.id.clone());

        assert!(crate::validate_application_semantic_model(&asm).is_empty());
        assert!(crate::validate_application_semantic_model(&asm_from_component_graph).is_empty());
        for computed in [&doubled, &tripled, &total] {
            assert!(matches!(
                asm.entity(computed),
                Some(crate::SemanticEntity::Computed(_))
            ));
            assert_eq!(asm.owner(computed), Some(&expected_owner));
            assert_eq!(
                asm.semantic_types
                    .computed_values
                    .get(computed)
                    .map(|computed_type| &computed_type.computed),
                Some(computed)
            );
        }

        let reactive_references = asm
            .references
            .iter()
            .filter(|reference| {
                matches!(
                    reference.kind,
                    crate::SemanticReferenceKind::ComputedState
                        | crate::SemanticReferenceKind::ComputedComputed
                )
            })
            .collect::<Vec<_>>();
        assert_eq!(reactive_references.len(), 4);
        for reference in reactive_references {
            assert!(asm.reactive_graph.edges.iter().any(|edge| {
                edge.kind == crate::IrReactiveEdgeKind::Reads
                    && edge.source == reference.source.as_str()
                    && edge.target == reference.target.as_str()
                    && edge.provenance == reference.provenance
            }));
            assert!(asm.reactive_graph.edges.iter().any(|edge| {
                edge.kind == crate::IrReactiveEdgeKind::Invalidates
                    && edge.source == reference.target.as_str()
                    && edge.target == reference.source.as_str()
                    && edge.provenance == reference.provenance
            }));
        }
        assert!(asm.reactive_graph.nodes.contains_key(count.as_str()));
        assert_eq!(
            asm.computed_evaluation_plan,
            crate::plan_computed_evaluation(&asm.reactive_graph)
        );
        assert_eq!(
            asm.computed_evaluation_plan.evaluation_order,
            vec![
                doubled.as_str().to_string(),
                tripled.as_str().to_string(),
                total.as_str().to_string(),
            ]
        );

        let ir = crate::lower_components_to_ir(&asm);
        assert!(crate::validate_intermediate_representation(&ir).is_empty());
        let evaluations = ir
            .modules
            .iter()
            .flat_map(|module| &module.computed_evaluations)
            .map(|evaluation| evaluation.computed.clone())
            .collect::<Vec<_>>();
        assert_eq!(evaluations, vec![doubled, tripled, total]);
    }

    #[test]
    fn classifies_computed_purity_and_reports_unsupported_behavior() {
        let mut parsed = ezc_parser::parse_file(
            "src/ComputedPurity.tsx",
            r#"
@component("x-computed-purity")
class ComputedPurity extends Component {
  count = state(1);

  @action()
  save() {}

  @effect()
  sync() {}

  @computed()
  get pure() { return this.count; }

  @computed()
  get mutates() { this.count++; return 1; }

  @computed()
  get actionCall() { return this.save(); }

  @computed()
  get effectCall() { console.log("effect"); return 1; }

  @computed()
  get resourceCall() { return resource(); }

  @computed()
  get arbitraryCall() { return helper(); }

  @computed()
  get random() { return Math.random(); }

  @computed()
  get asyncValue() { return 1; }
}
"#,
        );
        parsed.classes[0]
            .methods
            .iter_mut()
            .find(|method| method.name == "asyncValue")
            .expect("async test getter")
            .is_async = true;

        let asm = build_application_semantic_model(&parsed);
        let component = &asm.components[0];
        let pure = component.id.computed("pure");
        let mutates = component.id.computed("mutates");
        let action_call = component.id.computed("actionCall");
        let effect_call = component.id.computed("effectCall");
        let resource_call = component.id.computed("resourceCall");
        let arbitrary_call = component.id.computed("arbitraryCall");
        let random = component.id.computed("random");
        let async_value = component.id.computed("asyncValue");

        assert_eq!(
            asm.computed_value(&pure).expect("pure value").purity,
            crate::ComputedPurity::Pure
        );
        for (computed, kind) in [
            (mutates, crate::ComputedPurityViolationKind::StateMutation),
            (action_call, crate::ComputedPurityViolationKind::Action),
            (effect_call, crate::ComputedPurityViolationKind::Effect),
            (resource_call, crate::ComputedPurityViolationKind::Resource),
            (
                arbitrary_call,
                crate::ComputedPurityViolationKind::ArbitraryMethodCall,
            ),
            (
                random,
                crate::ComputedPurityViolationKind::NondeterministicOperation,
            ),
            (async_value, crate::ComputedPurityViolationKind::Async),
        ] {
            let value = asm.computed_value(&computed).expect("computed value");
            assert_eq!(value.purity, crate::ComputedPurity::Impure);
            assert!(value
                .purity_violations
                .iter()
                .any(|violation| violation.kind == kind));
        }
        let diagnostics = asm
            .diagnostics
            .iter()
            .filter(|diagnostic| diagnostic.code == "EZC1034")
            .collect::<Vec<_>>();
        assert_eq!(diagnostics.len(), 7);
        assert!(diagnostics
            .iter()
            .all(|diagnostic| diagnostic.provenance.is_some()));
    }

    #[test]
    fn establishes_typed_action_input_and_output_contracts() {
        let parsed = ezc_parser::parse_file(
            "src/Action.tsx",
            r#"
type ActionResult = number;

@component("x-action")
class Action extends Component {
  @action()
  async addTodo(input: string): ActionResult { return 1; }
}
"#,
        );

        let asm = build_application_semantic_model(&parsed);
        let method = &asm.components[0].methods[0];
        let signature = asm
            .semantic_types
            .action_signatures
            .get(&method.id)
            .expect("action signature");

        assert!(method.is_action());
        assert!(signature.is_async);
        assert_eq!(signature.input.len(), 1);
        assert_eq!(signature.input[0].1, crate::SemanticType::String);
        assert_eq!(signature.output, Some(crate::SemanticType::Number));
    }

    #[test]
    fn lowers_supported_literal_state_annotations_into_canonical_types() {
        let parsed = ezc_parser::parse_file(
            "src/LiteralTypes.tsx",
            r#"
@component("x-literal-types")
class LiteralTypes extends Component {
  filter: "all" = state("all");
  step: 42 = state(42);
  enabled: true = state(true);
}
"#,
        );

        let asm = build_application_semantic_model(&parsed);
        let types = asm.components[0]
            .state_fields
            .iter()
            .map(|field| {
                (
                    field.name.as_str(),
                    &asm.semantic_types.assignments[&field.id].semantic_type,
                )
            })
            .collect::<Vec<_>>();

        assert_eq!(
            types,
            vec![
                (
                    "filter",
                    &crate::SemanticType::StringLiteral("all".to_string())
                ),
                (
                    "step",
                    &crate::SemanticType::NumberLiteral("42".to_string())
                ),
                ("enabled", &crate::SemanticType::BooleanLiteral(true)),
            ]
        );
    }

    #[test]
    fn lowers_array_and_tuple_state_annotations_into_canonical_types() {
        let parsed = ezc_parser::parse_file(
            "src/CollectionTypes.tsx",
            r#"
@component("x-collection-types")
class CollectionTypes extends Component {
  names: string[] = state([]);
  todos: Todo[] = state([]);
  pair: [string, number] = state(["EdgeZero", 1]);
}
"#,
        );

        let asm = build_application_semantic_model(&parsed);
        let types = asm.components[0]
            .state_fields
            .iter()
            .map(|field| {
                (
                    field.name.as_str(),
                    &asm.semantic_types.assignments[&field.id].semantic_type,
                )
            })
            .collect::<Vec<_>>();

        assert_eq!(
            types,
            vec![
                (
                    "names",
                    &crate::SemanticType::Array(Box::new(crate::SemanticType::String)),
                ),
                (
                    "todos",
                    &crate::SemanticType::Array(Box::new(crate::SemanticType::Unknown)),
                ),
                (
                    "pair",
                    &crate::SemanticType::Tuple(vec![
                        crate::SemanticType::String,
                        crate::SemanticType::Number,
                    ]),
                ),
            ]
        );
    }

    #[test]
    fn lowers_structural_object_state_annotations_into_canonical_types() {
        let parsed = ezc_parser::parse_file(
            "src/ObjectTypes.tsx",
            r#"
@component("x-object-types")
class ObjectTypes extends Component {
  todo: { id: string; title: string; completed: boolean } = state({ id: "1", title: "EdgeZero", completed: false });
}
"#,
        );

        let asm = build_application_semantic_model(&parsed);
        let field = &asm.components[0].state_fields[0];
        let assignment = &asm.semantic_types.assignments[&field.id];
        let crate::SemanticType::Object(object) = &assignment.semantic_type else {
            panic!("expected structural object type");
        };

        assert_eq!(
            object.properties,
            std::collections::BTreeMap::from([
                ("completed".to_string(), crate::SemanticType::Boolean),
                ("id".to_string(), crate::SemanticType::String),
                ("title".to_string(), crate::SemanticType::String),
            ])
        );
    }

    #[test]
    fn lowers_union_and_nullable_state_annotations_into_canonical_types() {
        let parsed = ezc_parser::parse_file(
            "src/UnionTypes.tsx",
            r#"
@component("x-union-types")
class UnionTypes extends Component {
  filter: "all" | "active" | "completed" = state("all");
  user: { id: string } | null = state(null);
}
"#,
        );

        let asm = build_application_semantic_model(&parsed);
        let fields = &asm.components[0].state_fields;
        assert_eq!(
            asm.semantic_types.assignments[&fields[0].id].semantic_type,
            crate::SemanticType::Union(vec![
                crate::SemanticType::StringLiteral("active".to_string()),
                crate::SemanticType::StringLiteral("all".to_string()),
                crate::SemanticType::StringLiteral("completed".to_string()),
            ])
        );
        assert_eq!(
            asm.semantic_types.assignments[&fields[1].id].semantic_type,
            crate::SemanticType::Union(vec![
                crate::SemanticType::Null,
                crate::SemanticType::Object(crate::ObjectType {
                    properties: std::collections::BTreeMap::from([(
                        "id".to_string(),
                        crate::SemanticType::String,
                    )]),
                }),
            ])
        );
    }

    #[test]
    fn resolves_local_type_aliases_with_canonical_alias_identity() {
        let parsed = ezc_parser::parse_file(
            "src/Aliases.tsx",
            r#"
type TodoId = string;
type Filter = "all" | "active" | "completed";

@component("x-aliases")
class Aliases extends Component {
  id: TodoId = state("todo-1");
  filter: Filter = state("all");
}
"#,
        );

        let asm = build_application_semantic_model(&parsed);
        let fields = &asm.components[0].state_fields;
        let todo_id = asm
            .semantic_types
            .aliases
            .values()
            .find(|alias| alias.name == "TodoId")
            .expect("TodoId alias");
        let filter = asm
            .semantic_types
            .aliases
            .values()
            .find(|alias| alias.name == "Filter")
            .expect("Filter alias");

        assert_eq!(todo_id.semantic_type, crate::SemanticType::String);
        assert_eq!(
            filter.semantic_type,
            crate::SemanticType::Union(vec![
                crate::SemanticType::StringLiteral("active".to_string()),
                crate::SemanticType::StringLiteral("all".to_string()),
                crate::SemanticType::StringLiteral("completed".to_string()),
            ])
        );
        assert_eq!(
            asm.semantic_types.assignments[&fields[0].id].origin,
            todo_id.id
        );
        assert_eq!(
            asm.semantic_types.assignments[&fields[1].id].origin,
            filter.id
        );
    }

    #[test]
    fn resolves_imported_type_aliases_through_named_reexports() {
        let unit = CompilationUnit::parse_sources([
            (
                "src/types.ts",
                r#"export type Filter = "all" | "active" | "completed";"#,
            ),
            ("src/index.ts", r#"export { Filter } from "./types";"#),
            (
                "src/App.tsx",
                r#"
import { Filter } from "./index";

@component("x-app")
class App extends Component {
  filter: Filter = state("all");
}
"#,
            ),
        ]);

        let asm = build_application_semantic_model_for_unit(&unit);
        let field = &asm
            .components
            .iter()
            .find(|component| component.class_name == "App")
            .expect("App component")
            .state_fields[0];
        let assignment = &asm.semantic_types.assignments[&field.id];

        assert_eq!(
            assignment.semantic_type,
            crate::SemanticType::Union(vec![
                crate::SemanticType::StringLiteral("active".to_string()),
                crate::SemanticType::StringLiteral("all".to_string()),
                crate::SemanticType::StringLiteral("completed".to_string()),
            ])
        );
        assert_eq!(
            assignment.origin,
            crate::SemanticId::type_alias_in_module("src/types.ts", "Filter")
        );
    }

    #[test]
    fn infers_state_types_from_direct_serializable_initializers() {
        let parsed = ezc_parser::parse_file(
            "src/InferredState.tsx",
            r#"
@component("x-inferred-state")
class InferredState extends Component {
  count = state(0);
  todos = state([]);
  tags = state(["EdgeZero"]);
  todo = state({ id: "1", completed: false });
}
"#,
        );

        let asm = build_application_semantic_model(&parsed);
        let fields = &asm.components[0].state_fields;
        let types = fields
            .iter()
            .map(|field| {
                let assignment = &asm.semantic_types.assignments[&field.id];
                assert_eq!(assignment.status, crate::SemanticTypeStatus::Inferred);
                (field.name.as_str(), assignment.semantic_type.clone())
            })
            .collect::<Vec<_>>();

        assert_eq!(
            types,
            vec![
                ("count", crate::SemanticType::Number),
                (
                    "todos",
                    crate::SemanticType::Array(Box::new(crate::SemanticType::Unknown)),
                ),
                (
                    "tags",
                    crate::SemanticType::Array(Box::new(crate::SemanticType::String)),
                ),
                (
                    "todo",
                    crate::SemanticType::Object(crate::ObjectType {
                        properties: std::collections::BTreeMap::from([
                            ("completed".to_string(), crate::SemanticType::Boolean),
                            ("id".to_string(), crate::SemanticType::String),
                        ]),
                    }),
                ),
            ]
        );
    }

    #[test]
    fn propagates_canonical_types_to_expression_graph_nodes() {
        let parsed = ezc_parser::parse_file(
            "src/ExpressionTypes.tsx",
            r#"
@component("x-expression-types")
class ExpressionTypes extends Component {
  total = state((1 + 2) * 3);
  ready = state(1 < 2);
}
"#,
        );

        let asm = build_application_semantic_model(&parsed);
        let total = &asm.components[0].state_fields[0];
        let ready = &asm.components[0].state_fields[1];
        let total_root = asm
            .expression_root(&total.id)
            .expect("total expression root");
        let ready_root = asm
            .expression_root(&ready.id)
            .expect("ready expression root");

        assert_eq!(
            asm.semantic_types.assignments[total_root].semantic_type,
            crate::SemanticType::Number
        );
        assert_eq!(
            asm.semantic_types.assignments[ready_root].semantic_type,
            crate::SemanticType::Boolean
        );
        assert!(asm
            .expression_dependencies(total_root)
            .iter()
            .all(|id| asm.semantic_types.assignments.contains_key(*id)));
    }

    #[test]
    fn derives_component_ownership_without_legacy_owner_fields() {
        let parsed = ezc_parser::parse_file(
            "src/Counter.tsx",
            r#"
@component("x-counter")
class Counter extends Component {
  count = state(0);

  increment() {
    this.count++;
  }

  render() {
    return <button onClick={() => this.increment()}>{this.count}</button>;
  }
}
"#,
        );
        let mut component_graph = build_component_graph_for_module(&parsed);
        let mut templates = build_template_graph(&component_graph).templates;
        let template_entities = build_template_semantic_entities(&templates);
        let component = component_graph
            .components
            .first_mut()
            .expect("component graph should contain Counter");
        let component_id = component.id.clone();
        let method_id = component.methods[0].id.clone();
        let action_id = component.actions[0].id.clone();
        let state_id = component.state_fields[0].id.clone();
        let event_id = component
            .render
            .as_ref()
            .expect("Counter should render")
            .event_handlers[0]
            .id
            .clone();

        component.owner = SemanticOwner::entity(component_id.clone());
        component.state_fields[0].owner = SemanticOwner::Application;
        component.methods[0].owner = SemanticOwner::Application;
        component.actions[0].owner = SemanticOwner::Application;
        component
            .render
            .as_mut()
            .expect("Counter should render")
            .event_handlers[0]
            .owner = SemanticOwner::Application;
        templates[0].owner = SemanticOwner::Application;

        let ownership = collect_ownership(
            &component_graph.components,
            &std::collections::BTreeMap::new(),
            &std::collections::BTreeMap::new(),
            &std::collections::BTreeMap::new(),
            &std::collections::BTreeMap::new(),
            &std::collections::BTreeMap::new(),
            &std::collections::BTreeMap::new(),
            &std::collections::BTreeMap::new(),
            &std::collections::BTreeMap::new(),
            &std::collections::BTreeMap::new(),
            &std::collections::BTreeMap::new(),
            &ComponentInstancePlan::default(),
            &std::collections::BTreeMap::new(),
            &std::collections::BTreeMap::new(),
            &templates,
            &template_entities,
        );
        assert_eq!(ownership[&component_id], SemanticOwner::Application);
        assert_eq!(
            ownership[&state_id],
            SemanticOwner::entity(component_id.clone())
        );
        assert_eq!(
            ownership[&method_id],
            SemanticOwner::entity(component_id.clone())
        );
        assert_eq!(ownership[&action_id], SemanticOwner::entity(method_id));
        assert_eq!(
            ownership[&event_id],
            SemanticOwner::entity(component_id.clone().template())
        );
        assert_eq!(
            ownership[&templates[0].id],
            SemanticOwner::entity(component_id)
        );
    }

    #[test]
    #[allow(clippy::too_many_lines)]
    fn traverses_application_ownership_in_semantic_id_order() {
        let parsed = ezc_parser::parse_file(
            "src/Counter.tsx",
            r#"
@component("x-counter")
class Counter extends Component {
  count = state(0);

  increment() {
    this.count++;
  }

  render() {
    return <button onClick={this.increment}>{this.count}</button>;
  }
}
"#,
        );

        let asm = build_application_semantic_model(&parsed);
        let component = &asm.components[0];
        let roots = asm.application_roots();

        assert_eq!(
            roots,
            vec![
                &component.id,
                asm.component_instance_plan
                    .instances
                    .values()
                    .find(|instance| instance.depth == 0)
                    .expect("build root instance")
                    .id
                    .as_semantic_id(),
            ]
        );
        assert_eq!(
            asm.children_of(&component.id),
            vec![
                &component.methods[0].id,
                &component.methods[1].id,
                &component.state_fields[0].id,
                &asm.templates[0].id,
            ]
        );
        assert_eq!(
            asm.children_of(&component.methods[0].id),
            vec![&component.actions[0].id]
        );
        assert_eq!(asm.parent_of(&component.id), None);
        assert_eq!(
            asm.parent_of(&component.actions[0].id),
            Some(&component.methods[0].id)
        );
        assert_eq!(
            asm.ancestors_of(&component.actions[0].id),
            vec![&component.methods[0].id, &component.id]
        );
        let descendants = asm.descendants_of(&component.id);
        assert_eq!(descendants[0], &component.methods[0].id);
        assert_eq!(descendants[1], &component.actions[0].id);
        assert_eq!(descendants[2], &component.methods[1].id);
        assert_eq!(
            descendants.len(),
            asm.ownership.len() - asm.application_roots().len()
        );
        assert_eq!(
            asm.entities_of_kind(SemanticEntityKind::Method),
            vec![&component.methods[0].id, &component.methods[1].id]
        );
        assert_eq!(
            asm.entities_of_kind(SemanticEntityKind::Action),
            vec![&component.actions[0].id]
        );
        assert_eq!(
            asm.entities_of_kind(SemanticEntityKind::StateField),
            vec![&component.state_fields[0].id]
        );
        let state_id = &component.state_fields[0].id;
        let state_provenance = asm.provenance(state_id).expect("state provenance");
        assert_eq!(
            asm.entities_in_file(state_provenance.path.as_path()).len(),
            asm.ownership.len()
        );
        let at_state =
            asm.entities_at(state_provenance.path.as_path(), state_provenance.span.start);
        assert!(at_state.contains(&state_id));
        assert!(at_state.iter().all(|id| {
            let provenance = asm.provenance(id).expect("entity provenance");
            provenance.span.start <= state_provenance.span.start
                && state_provenance.span.start < provenance.span.end
        }));
        let action_references = asm.references_of_kind(SemanticReferenceKind::ActionState);
        assert_eq!(action_references.len(), 1);
        assert_eq!(action_references[0].source, component.actions[0].id);
        assert_eq!(action_references[0].target, component.state_fields[0].id);
        let action_provenance = &action_references[0].provenance;
        assert_eq!(
            asm.references_in_file(action_provenance.path.as_path())
                .len(),
            asm.references.len()
        );
        let at_action = asm.references_at(
            action_provenance.path.as_path(),
            action_provenance.span.start,
        );
        assert!(at_action.iter().any(|reference| {
            reference.source == component.actions[0].id
                && reference.target == component.state_fields[0].id
        }));
    }

    #[test]
    fn resolves_template_state_dependencies() {
        let parsed = ezc_parser::parse_file(
            "src/Panel.tsx",
            r#"
@component("x-panel")
class Panel extends Component {
  enabled = state(true);

  render() {
    return <section hidden={this.enabled}>{this.enabled}{this.enabled ? <span>On</span> : <span>Off</span>}</section>;
  }
}
"#,
        );

        let asm = build_application_semantic_model(&parsed);
        let component = &asm.components[0];
        let state = &component.state_fields[0];
        let state_references = asm.references_to(&state.id);

        assert_eq!(state_references.len(), 3);
        assert!(state_references.iter().all(|reference| {
            reference.kind == SemanticReferenceKind::TemplateState
                && reference.target == state.id
                && reference.provenance.path.as_path() == std::path::Path::new("src/Panel.tsx")
        }));
        assert!(state_references.iter().any(|reference| {
            asm.template_entity(&reference.source)
                .is_some_and(|entity| entity.kind == TemplateSemanticKind::Binding)
        }));
        assert!(state_references.iter().any(|reference| {
            asm.template_entity(&reference.source)
                .is_some_and(|entity| entity.kind == TemplateSemanticKind::AttributeBinding)
        }));
        assert!(state_references.iter().any(|reference| {
            asm.template_entity(&reference.source)
                .is_some_and(|entity| entity.kind == TemplateSemanticKind::Conditional)
        }));
    }

    #[test]
    fn resolves_template_bindings_to_canonical_computed_entities() {
        let parsed = ezc_parser::parse_file(
            "src/ComputedTemplate.tsx",
            r#"
@component("x-computed-template")
class ComputedTemplate extends Component {
  @computed()
  get label(): string { return "Ready"; }

  @computed()
  get visible(): boolean { return true; }

  render() {
    return <section title={this.label}>{this.label}{this.visible ? <span>Visible</span> : <span>Hidden</span>}</section>;
  }
}
"#,
        );

        let asm = build_application_semantic_model(&parsed);
        let component = &asm.components[0];
        let label = component.id.computed("label");
        let visible = component.id.computed("visible");
        let references = asm.references_of_kind(SemanticReferenceKind::TemplateComputed);

        assert_eq!(references.len(), 3);
        assert_eq!(
            references
                .iter()
                .filter(|reference| reference.target == label)
                .count(),
            2
        );
        assert!(references.iter().any(|reference| {
            reference.target == visible
                && asm
                    .template_entity(&reference.source)
                    .is_some_and(|entity| entity.kind == TemplateSemanticKind::Conditional)
        }));
        assert!(references.iter().all(|reference| {
            asm.provenance(&reference.source) == Some(&reference.provenance)
                && asm.semantic_type_of(&reference.source)
                    == asm.semantic_type_of(&reference.target)
        }));
        assert_eq!(
            asm.references_of_kind(SemanticReferenceKind::TemplateState)
                .into_iter()
                .filter(|reference| reference.target == label || reference.target == visible)
                .count(),
            0
        );

        let graph = crate::build_semantic_graph(&asm);
        assert_eq!(
            graph
                .edges
                .iter()
                .filter(|edge| edge.kind == crate::SemanticGraphEdgeKind::TemplateComputed)
                .count(),
            3
        );
    }

    #[test]
    fn navigates_template_entities_from_canonical_ownership() {
        let parsed = ezc_parser::parse_file(
            "src/Panel.tsx",
            r#"
@component("x-panel")
class Panel extends Component {
  count = state(0);

  render() {
    return <section>{this.count}</section>;
  }
}
"#,
        );
        let mut asm = build_application_semantic_model(&parsed);
        let template_id = asm.templates[0].id.clone();
        let expected = asm.template_entities_for(&template_id).len();

        for entity in &mut asm.template_entities {
            entity.owner = SemanticOwner::Application;
        }

        assert_eq!(asm.template_entities_for(&template_id).len(), expected);
    }

    #[test]
    fn leaves_member_expressions_unresolved_without_expression_evaluation() {
        let parsed = ezc_parser::parse_file(
            "src/Panel.tsx",
            r#"
@component("x-panel")
class Panel extends Component {
  enabled = state(true);

  render() {
    return <section data-value={this.enabled.value}>Panel</section>;
  }
}
"#,
        );

        let asm = build_application_semantic_model(&parsed);
        assert_eq!(
            asm.references
                .iter()
                .filter(|reference| reference.kind == SemanticReferenceKind::TemplateState)
                .count(),
            0
        );
    }

    #[test]
    fn resolves_keyed_list_iterable_to_component_state() {
        let parsed = ezc_parser::parse_file(
            "src/KeyedList.tsx",
            r#"
@component("x-keyed-list")
class KeyedList extends Component {
  items = state([]);

  render() {
    return <ul>{this.items.map((item) => <li key={item}>{item}</li>)}</ul>;
  }
}
"#,
        );

        let asm = build_application_semantic_model(&parsed);
        let component = &asm.components[0];
        let state = &component.state_fields[0];
        let reference = asm
            .references_to(&state.id)
            .into_iter()
            .find(|reference| reference.kind == SemanticReferenceKind::TemplateState)
            .expect("list iterable state reference");
        let list = asm
            .template_entity(&reference.source)
            .expect("list semantic entity");

        assert_eq!(list.kind, TemplateSemanticKind::List);
        assert_eq!(list.expression.as_deref(), Some("this.items"));
        assert_eq!(reference.provenance, list.provenance);
    }

    #[test]
    fn resolves_template_event_attribute_to_component_method() {
        let parsed = ezc_parser::parse_file(
            "src/Counter.tsx",
            r#"
@component("x-counter")
class Counter extends Component {
  count = state(0);

  increment() {
    this.count += 1;
  }

  render() {
    return <button onClick={() => this.increment()}>Count</button>;
  }
}
"#,
        );

        let asm = build_application_semantic_model(&parsed);
        let component = &asm.components[0];
        let method = component
            .methods
            .iter()
            .find(|method| method.name == "increment")
            .expect("increment method");
        let reference = asm
            .references_to(&method.id)
            .into_iter()
            .find(|reference| {
                reference.kind == SemanticReferenceKind::EventMethod
                    && asm
                        .template_entity(&reference.source)
                        .is_some_and(|entity| entity.kind == TemplateSemanticKind::EventAttribute)
            })
            .expect("template event method reference");

        assert_eq!(
            reference.provenance.path,
            std::path::Path::new("src/Counter.tsx")
        );
        assert_eq!(reference.provenance.span.line, 11);
    }

    #[test]
    fn resolves_render_template_bindings_to_unique_method_locals() {
        let parsed = ezc_parser::parse_file(
            "src/LocalResolution.tsx",
            r#"
@component("x-local-resolution")
class LocalResolution extends Component {
  render() {
    const title = "EdgeZero";
    return <output title={title}>{title}</output>;
  }
}
"#,
        );

        let asm = build_application_semantic_model(&parsed);
        let component = &asm.components[0];
        let render = component
            .methods
            .iter()
            .find(|method| method.name == "render")
            .expect("render method");
        let local = &render.local_variables[0];
        let references = asm.references_of_kind(SemanticReferenceKind::TemplateLocal);
        let assignment = asm
            .semantic_types
            .assignments
            .get(&local.id)
            .expect("canonical inferred local type");

        assert_eq!(
            asm.owner(&local.id),
            Some(&SemanticOwner::entity(render.id.clone()))
        );
        assert_eq!(
            asm.entities_of_kind(SemanticEntityKind::LocalVariable),
            vec![&local.id]
        );
        assert_eq!(references.len(), 2);
        assert!(references.iter().all(|reference| {
            reference.target == local.id
                && reference.provenance.path == std::path::Path::new("src/LocalResolution.tsx")
        }));
        assert_eq!(assignment.semantic_type, crate::SemanticType::String);
        assert_eq!(assignment.status, crate::SemanticTypeStatus::Inferred);
        assert_eq!(assignment.provenance.span, local.span);

        let semantic_graph = crate::build_semantic_graph(&asm);
        assert!(semantic_graph.nodes.iter().any(|node| {
            node.kind == crate::SemanticGraphNodeKind::LocalVariable && node.id == local.id
        }));
        assert_eq!(
            semantic_graph
                .edges
                .iter()
                .filter(|edge| {
                    edge.kind == crate::SemanticGraphEdgeKind::TemplateLocal
                        && edge.target == local.id
                })
                .count(),
            2
        );

        let template_graph = build_template_graph(&build_component_graph_for_module(&parsed));
        assert_eq!(
            crate::generate_static_html(&template_graph),
            "<output data-ez-node=\"n0\" title=\"EdgeZero\" data-ez-bindings=\"title\"><!-- ez-binding:n2:title -->EdgeZero</output>\n"
        );
    }

    #[test]
    fn queries_canonical_expression_graphs_by_dependency_provenance_and_owner() {
        let parsed = ezc_parser::parse_file(
            "src/ExpressionQueries.tsx",
            r#"
@component("x-expression-queries")
class ExpressionQueries extends Component {
  total = state((1 + 2) * 3);
}
"#,
        );

        let asm = build_application_semantic_model(&parsed);
        let field = &asm.components[0].state_fields[0];
        let root = asm.expression_root(&field.id).expect("expression root");
        let left = field.id.expression("root.0");
        let right = field.id.expression("root.1");

        assert_eq!(asm.expression(root).map(|node| &node.id), Some(root));
        assert_eq!(asm.expression_owner(root), Some(&field.id));
        assert_eq!(asm.expression_dependencies(root), vec![&left, &right]);
        assert_eq!(
            asm.expression_dependents(&left)
                .into_iter()
                .map(|node| &node.id)
                .collect::<Vec<_>>(),
            vec![root]
        );
        assert_eq!(asm.expressions_for(&field.id).len(), 5);

        let provenance = asm
            .expression_provenance(root)
            .expect("expression provenance");
        assert_eq!(
            provenance.path,
            std::path::Path::new("src/ExpressionQueries.tsx")
        );
        assert_eq!(asm.expressions_in_file(&provenance.path).len(), 5);
        assert_eq!(
            asm.expressions_at(&provenance.path, provenance.span.start)
                .into_iter()
                .map(|node| &node.id)
                .collect::<Vec<_>>(),
            vec![root]
        );
    }
}
