use std::collections::{BTreeMap, BTreeSet};

use crate::{
    ComponentNode, ComputedPurity, ComputedValue, EffectStatementSyntaxKind, ExecutionBoundary,
    ExpressionGraph, IrComputedEvaluationPlan, IrReactiveGraph, IrReactiveNode, IrReactiveNodeKind,
    IrReactiveTransitiveAnalysis, IrUpdateScheduler, SemanticId, SemanticOwner, SemanticTypeModel,
    SourceProvenance, UnsupportedEffectStatementKind,
};

/// Compiler-owned execution contract for an effect.
///
/// The scheduler will consume this policy in later Phase F slices. It is
/// declared here so effect timing remains compiler semantics rather than a
/// runtime callback convention.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EffectExecutionPolicy {
    AfterInitialRenderAndCompletedActionBatch,
}

/// Compiler classification of an effect's language-semantic validity.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EffectValidation {
    Unvalidated,
    Valid,
    Invalid,
}

/// Unsupported behavior that makes an effect ineligible for later lowering.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum EffectSemanticViolationKind {
    Async,
    CleanupLifecycleUnavailable,
    ReactiveStateMutation,
    ActionInvocation,
    EffectInvocation,
    ComponentMethodInvocation,
    UnresolvedComponentCall,
    UnresolvedComponentAssignment,
    UnknownExternalCapability,
    CapabilitySignature,
    CapabilityBoundary,
    CapabilitySerialization,
    ValueReturn,
    UnsupportedStatement,
}

/// One compiler-owned effect semantic violation with canonical provenance.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EffectSemanticViolation {
    pub kind: EffectSemanticViolationKind,
    pub statement: Option<SemanticId>,
    pub provenance: SourceProvenance,
}

/// Transitive reactive topology for one valid terminal effect.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EffectReactiveAnalysis {
    pub effect: SemanticId,
    pub dependencies: Vec<SemanticId>,
    pub dependents: Vec<SemanticId>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ActionBatch {
    pub id: SemanticId,
    pub authored_action_endpoint: SemanticId,
    pub ordered_write_records: Vec<SemanticId>,
    pub written_states: Vec<SemanticId>,
    pub provenance: SourceProvenance,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ActionBatchEffectTrigger {
    pub action_batch: SemanticId,
    pub effects: Vec<SemanticId>,
    pub matched_states: BTreeMap<SemanticId, Vec<SemanticId>>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct EffectTriggerPlan {
    pub initial_effects: Vec<SemanticId>,
    pub action_batches: BTreeMap<SemanticId, ActionBatch>,
    pub action_batch_triggers: Vec<ActionBatchEffectTrigger>,
}

/// One filtered reference to an existing E9 computed-update batch.
///
/// `source_batch_index` identifies the canonical E9 batch; `computed` retains
/// only the effect-required members without changing E9's ordering or batch
/// boundaries.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EffectComputedPrerequisiteBatch {
    pub source_batch_index: u32,
    pub computed: Vec<SemanticId>,
}

/// The compiler-owned boundary separating initial rendering from effects.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EffectRenderBoundary {
    AfterInitialRender,
}

/// One scheduler-produced terminal batch of effects.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EffectExecutionBatch {
    pub index: u32,
    pub effects: Vec<SemanticId>,
}

/// The reason an otherwise eligible effect has no execution batch.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnplannedEffectReason {
    UnavailableComputedPrerequisite,
}

/// An F8-eligible effect whose required computed values cannot be executed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UnplannedEffect {
    pub effect: SemanticId,
    pub reason: UnplannedEffectReason,
    pub computed_dependencies: Vec<SemanticId>,
}

/// The initial effect schedule, explicitly separated from rendering.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct InitialEffectExecutionPlan {
    pub required_computed: Vec<SemanticId>,
    pub prerequisite_batches: Vec<EffectComputedPrerequisiteBatch>,
    pub render_boundary: Option<EffectRenderBoundary>,
    pub effect_batches: Vec<EffectExecutionBatch>,
    pub unplanned_effects: Vec<UnplannedEffect>,
}

/// The compiler-owned effect schedule for one completed action batch.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ActionEffectExecutionPlan {
    pub action_batch: SemanticId,
    pub required_computed: Vec<SemanticId>,
    pub prerequisite_batches: Vec<EffectComputedPrerequisiteBatch>,
    pub effect_batches: Vec<EffectExecutionBatch>,
    pub unplanned_effects: Vec<UnplannedEffect>,
}

/// Immutable F9 scheduling product derived from F7, F8, and E9 products.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct EffectExecutionPlan {
    pub initial: InitialEffectExecutionPlan,
    pub actions: Vec<ActionEffectExecutionPlan>,
}

/// The source declaration carrying one compiler effect body.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EffectDeclaration {
    LegacyMethod(SemanticId),
    V2Field,
}

/// A first-class compiler semantic entity for one legacy or V2 effect.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Effect {
    pub id: SemanticId,
    pub owner: SemanticOwner,
    pub declaration: EffectDeclaration,
    pub name: String,
    /// V2 field source order, when an effect originated from a class field.
    pub declaration_order: Option<u32>,
    pub execution_boundary: ExecutionBoundary,
    pub execution_policy: EffectExecutionPolicy,
    pub validation: EffectValidation,
    pub semantic_violations: Vec<EffectSemanticViolation>,
    pub provenance: SourceProvenance,
}

/// An ordered, compiler-owned effect body.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EffectBody {
    pub effect: SemanticId,
    pub statements: Vec<SemanticId>,
    pub cleanup: Option<EffectCleanupBody>,
    pub provenance: SourceProvenance,
}

/// The separately ordered cleanup program for one effect body.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EffectCleanupBody {
    pub statements: Vec<SemanticId>,
    pub provenance: SourceProvenance,
}

/// One compiler-owned statement belonging to an effect body.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EffectStatement {
    pub id: SemanticId,
    pub owner: SemanticId,
    pub kind: EffectStatementKind,
    pub provenance: SourceProvenance,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EffectStatementKind {
    ExternalMemberAssignment {
        target: SemanticId,
        value: SemanticId,
    },
    CapabilityCall {
        callee: SemanticId,
        arguments: Vec<SemanticId>,
    },
    EffectReturn {
        value: Option<SemanticId>,
    },
    Empty,
    Unsupported(UnsupportedEffectStatementKind),
}

/// Collect canonical effect entities in stable semantic-ID order.
///
#[must_use]
pub fn collect_effects(
    components: &[ComponentNode],
    provenance: &BTreeMap<SemanticId, SourceProvenance>,
) -> BTreeMap<SemanticId, Effect> {
    let mut effects = BTreeMap::new();
    for component in components {
        for method in component.methods.iter().filter(|method| method.is_effect()) {
            let id = component.id.effect(&method.name);
            let method_provenance = provenance
                .get(&method.id)
                .expect("effect methods should have canonical provenance")
                .clone();
            effects.insert(
                id.clone(),
                Effect {
                    id,
                    owner: SemanticOwner::entity(component.id.clone()),
                    declaration: EffectDeclaration::LegacyMethod(method.id.clone()),
                    name: method.name.clone(),
                    declaration_order: None,
                    execution_boundary: ExecutionBoundary::Client,
                    execution_policy:
                        EffectExecutionPolicy::AfterInitialRenderAndCompletedActionBatch,
                    validation: EffectValidation::Unvalidated,
                    semantic_violations: Vec::new(),
                    provenance: method_provenance,
                },
            );
        }
        for field in &component.effect_fields {
            let id = component.id.effect(&field.name);
            effects.insert(
                id.clone(),
                Effect {
                    id,
                    owner: field.owner.clone(),
                    declaration: EffectDeclaration::V2Field,
                    name: field.name.clone(),
                    declaration_order: Some(field.declaration_order),
                    execution_boundary: ExecutionBoundary::Client,
                    execution_policy:
                        EffectExecutionPolicy::AfterInitialRenderAndCompletedActionBatch,
                    validation: EffectValidation::Unvalidated,
                    semantic_violations: Vec::new(),
                    provenance: field.provenance.clone(),
                },
            );
        }
    }
    effects
}

/// Classifies effect legality from F4 statement facts without adding diagnostics.
#[allow(clippy::too_many_lines)]
#[must_use]
pub fn validate_effects(
    components: &[ComponentNode],
    mut effects: BTreeMap<SemanticId, Effect>,
    statements: &BTreeMap<SemanticId, EffectStatement>,
    semantic_types: &SemanticTypeModel,
) -> BTreeMap<SemanticId, Effect> {
    for effect in effects.values_mut() {
        let mut violations = Vec::new();
        if effect_is_async(components, effect) {
            violations.push(EffectSemanticViolation {
                kind: EffectSemanticViolationKind::Async,
                statement: None,
                provenance: effect.provenance.clone(),
            });
        }
        if effect_has_cleanup(components, effect) {
            violations.push(EffectSemanticViolation {
                kind: EffectSemanticViolationKind::CleanupLifecycleUnavailable,
                statement: None,
                provenance: effect.provenance.clone(),
            });
        }
        for statement in statements
            .values()
            .filter(|statement| statement.owner == effect.id)
        {
            let Some(record) = semantic_types.effect_statements.get(&statement.id) else {
                continue;
            };
            let statement_violation = match record.operation_classification {
                crate::EffectOperationClassification::ReactiveStateAssignment => {
                    Some(EffectSemanticViolationKind::ReactiveStateMutation)
                }
                crate::EffectOperationClassification::ComponentActionCall => {
                    Some(EffectSemanticViolationKind::ActionInvocation)
                }
                crate::EffectOperationClassification::ComponentEffectCall => {
                    Some(EffectSemanticViolationKind::EffectInvocation)
                }
                crate::EffectOperationClassification::ComponentMethodCall => {
                    Some(EffectSemanticViolationKind::ComponentMethodInvocation)
                }
                crate::EffectOperationClassification::UnresolvedComponentCall => {
                    Some(EffectSemanticViolationKind::UnresolvedComponentCall)
                }
                crate::EffectOperationClassification::UnresolvedComponentAssignment => {
                    Some(EffectSemanticViolationKind::UnresolvedComponentAssignment)
                }
                crate::EffectOperationClassification::UnknownExternalCapability => {
                    Some(EffectSemanticViolationKind::UnknownExternalCapability)
                }
                crate::EffectOperationClassification::ValueReturn => {
                    Some(EffectSemanticViolationKind::ValueReturn)
                }
                crate::EffectOperationClassification::UnsupportedStatement => {
                    Some(EffectSemanticViolationKind::UnsupportedStatement)
                }
                crate::EffectOperationClassification::RecognizedCapability
                | crate::EffectOperationClassification::BareReturn
                | crate::EffectOperationClassification::Empty => None,
            };
            if let Some(kind) = statement_violation {
                violations.push(EffectSemanticViolation {
                    kind,
                    statement: Some(statement.id.clone()),
                    provenance: statement.provenance.clone(),
                });
            }
            for (compatibility, kind) in [
                (
                    record.signature_compatibility,
                    EffectSemanticViolationKind::CapabilitySignature,
                ),
                (
                    record.boundary_compatibility,
                    EffectSemanticViolationKind::CapabilityBoundary,
                ),
                (
                    record.serialization_compatibility,
                    EffectSemanticViolationKind::CapabilitySerialization,
                ),
            ] {
                if compatibility != crate::EffectCompatibility::Compatible
                    && record.operation_classification
                        == crate::EffectOperationClassification::RecognizedCapability
                {
                    violations.push(EffectSemanticViolation {
                        kind,
                        statement: Some(statement.id.clone()),
                        provenance: statement.provenance.clone(),
                    });
                }
            }
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
        violations.dedup_by(|left, right| {
            left.kind == right.kind
                && left.statement == right.statement
                && left.provenance == right.provenance
        });
        effect.validation = if violations.is_empty() {
            EffectValidation::Valid
        } else {
            EffectValidation::Invalid
        };
        effect.semantic_violations = violations;
    }
    effects
}

/// Projects existing reactive transitive analysis into effect-keyed records.
#[must_use]
pub fn analyze_effect_reactivity(
    components: &[ComponentNode],
    computed_values: &BTreeMap<SemanticId, crate::ComputedValue>,
    effects: &BTreeMap<SemanticId, Effect>,
    transitive: &IrReactiveTransitiveAnalysis,
) -> BTreeMap<SemanticId, EffectReactiveAnalysis> {
    let identities = components
        .iter()
        .flat_map(|component| component.state_fields.iter().map(|field| field.id.clone()))
        .chain(computed_values.keys().cloned())
        .chain(effects.keys().cloned())
        .map(|id| (id.as_str().to_string(), id))
        .collect::<BTreeMap<_, _>>();
    effects
        .values()
        .filter(|effect| effect.validation == EffectValidation::Valid)
        .map(|effect| {
            let map_ids = |ids: Option<&Vec<String>>| {
                ids.into_iter()
                    .flatten()
                    .filter_map(|id| identities.get(id).cloned())
                    .collect()
            };
            (
                effect.id.clone(),
                EffectReactiveAnalysis {
                    effect: effect.id.clone(),
                    dependencies: map_ids(transitive.dependencies.get(effect.id.as_str())),
                    dependents: map_ids(transitive.dependents.get(effect.id.as_str())),
                },
            )
        })
        .collect()
}

/// # Panics
///
/// Panics when an authored action method has no canonical source provenance.
#[must_use]
pub fn derive_effect_trigger_plan(
    components: &[ComponentNode],
    effects: &BTreeMap<SemanticId, Effect>,
    analysis: &BTreeMap<SemanticId, EffectReactiveAnalysis>,
    provenance: &BTreeMap<SemanticId, SourceProvenance>,
) -> EffectTriggerPlan {
    let initial_effects = effects
        .values()
        .filter(|effect| effect.validation == EffectValidation::Valid)
        .map(|effect| effect.id.clone())
        .collect::<Vec<_>>();
    let mut action_batches = BTreeMap::new();
    for component in components {
        for (name, endpoint) in component.action_endpoint_ids() {
            let writes = component
                .actions
                .iter()
                .filter(|action| action.method == name)
                .collect::<Vec<_>>();
            let id = component.id.action_batch(&name);
            action_batches.insert(
                id.clone(),
                ActionBatch {
                    id,
                    authored_action_endpoint: endpoint.clone(),
                    ordered_write_records: writes.iter().map(|action| action.id.clone()).collect(),
                    written_states: writes
                        .iter()
                        .map(|action| component.id.state_field(&action.field))
                        .collect::<std::collections::BTreeSet<_>>()
                        .into_iter()
                        .collect(),
                    provenance: provenance
                        .get(&endpoint)
                        .expect("action endpoints should have canonical provenance")
                        .clone(),
                },
            );
        }
    }
    let action_batch_triggers = action_batches
        .values()
        .filter_map(|batch| {
            let matched_states = initial_effects
                .iter()
                .filter_map(|effect| {
                    let matches = analysis
                        .get(effect)?
                        .dependencies
                        .iter()
                        .filter(|dependency| batch.written_states.contains(dependency))
                        .cloned()
                        .collect::<Vec<_>>();
                    (!matches.is_empty()).then(|| (effect.clone(), matches))
                })
                .collect::<BTreeMap<_, _>>();
            (!matched_states.is_empty()).then(|| ActionBatchEffectTrigger {
                action_batch: batch.id.clone(),
                effects: matched_states.keys().cloned().collect(),
                matched_states,
            })
        })
        .collect();
    EffectTriggerPlan {
        initial_effects,
        action_batches,
        action_batch_triggers,
    }
}

/// Derive minimal, dependency-complete computed prerequisites and terminal
/// effect batches from existing F7, F8, and E9 compiler products.
///
/// This function never inspects effect or computed source expressions. E9
/// remains the authority for computed ordering; F9 only filters its batches
/// and asks the existing Phase D scheduler to form terminal effect batches.
#[must_use]
pub fn plan_effect_execution(
    computed_values: &BTreeMap<SemanticId, ComputedValue>,
    effects: &BTreeMap<SemanticId, Effect>,
    effect_analysis: &BTreeMap<SemanticId, EffectReactiveAnalysis>,
    trigger_plan: &EffectTriggerPlan,
    reactive_transitive_analysis: &IrReactiveTransitiveAnalysis,
    computed_evaluation_plan: &IrComputedEvaluationPlan,
) -> EffectExecutionPlan {
    let computed_by_text = computed_values
        .keys()
        .map(|id| (id.as_str().to_string(), id.clone()))
        .collect::<BTreeMap<_, _>>();
    let executable = computed_evaluation_plan
        .evaluation_order
        .iter()
        .filter_map(|id| computed_by_text.get(id))
        .filter(|id| {
            computed_values
                .get(*id)
                .is_some_and(|computed| computed.purity == ComputedPurity::Pure)
        })
        .cloned()
        .collect::<BTreeSet<_>>();
    let initial = build_initial_effect_execution_plan(
        &trigger_plan.initial_effects,
        effects,
        effect_analysis,
        computed_values,
        &executable,
        computed_evaluation_plan,
    );
    let actions = trigger_plan
        .action_batch_triggers
        .iter()
        .filter_map(|trigger| {
            let batch = trigger_plan.action_batches.get(&trigger.action_batch)?;
            let invalidated = batch
                .written_states
                .iter()
                .flat_map(|state| reactive_transitive_analysis.dependents_of(state.as_str()))
                .filter_map(|id| computed_by_text.get(id).cloned())
                .collect::<BTreeSet<_>>();
            Some(build_action_effect_execution_plan(
                &batch.id,
                &trigger.effects,
                effects,
                effect_analysis,
                computed_values,
                &executable,
                &invalidated,
                computed_evaluation_plan,
            ))
        })
        .collect();

    EffectExecutionPlan { initial, actions }
}

fn build_initial_effect_execution_plan(
    eligible_effects: &[SemanticId],
    effects: &BTreeMap<SemanticId, Effect>,
    effect_analysis: &BTreeMap<SemanticId, EffectReactiveAnalysis>,
    computed_values: &BTreeMap<SemanticId, ComputedValue>,
    executable: &BTreeSet<SemanticId>,
    computed_evaluation_plan: &IrComputedEvaluationPlan,
) -> InitialEffectExecutionPlan {
    let membership = effect_execution_membership(
        eligible_effects,
        effects,
        effect_analysis,
        computed_values,
        executable,
    );
    let required_computed = membership
        .schedulable_effects
        .iter()
        .flat_map(|effect| required_computed_for_effect(effect, effect_analysis, computed_values))
        .collect::<BTreeSet<_>>();
    InitialEffectExecutionPlan {
        prerequisite_batches: select_prerequisite_batches(
            &required_computed,
            computed_evaluation_plan,
            computed_values,
        ),
        required_computed: required_computed.into_iter().collect(),
        render_boundary: Some(EffectRenderBoundary::AfterInitialRender),
        effect_batches: schedule_terminal_effects(&membership.schedulable_effects, effects),
        unplanned_effects: membership.unplanned_effects,
    }
}

#[allow(clippy::too_many_arguments)]
fn build_action_effect_execution_plan(
    action_batch: &SemanticId,
    eligible_effects: &[SemanticId],
    effects: &BTreeMap<SemanticId, Effect>,
    effect_analysis: &BTreeMap<SemanticId, EffectReactiveAnalysis>,
    computed_values: &BTreeMap<SemanticId, ComputedValue>,
    executable: &BTreeSet<SemanticId>,
    invalidated: &BTreeSet<SemanticId>,
    computed_evaluation_plan: &IrComputedEvaluationPlan,
) -> ActionEffectExecutionPlan {
    let membership = effect_execution_membership(
        eligible_effects,
        effects,
        effect_analysis,
        computed_values,
        executable,
    );
    let required_computed = membership
        .schedulable_effects
        .iter()
        .flat_map(|effect| required_computed_for_effect(effect, effect_analysis, computed_values))
        .filter(|computed| invalidated.contains(computed))
        .collect::<BTreeSet<_>>();
    ActionEffectExecutionPlan {
        action_batch: action_batch.clone(),
        prerequisite_batches: select_prerequisite_batches(
            &required_computed,
            computed_evaluation_plan,
            computed_values,
        ),
        required_computed: required_computed.into_iter().collect(),
        effect_batches: schedule_terminal_effects(&membership.schedulable_effects, effects),
        unplanned_effects: membership.unplanned_effects,
    }
}

#[derive(Debug, Default)]
struct EffectExecutionMembership {
    schedulable_effects: Vec<SemanticId>,
    unplanned_effects: Vec<UnplannedEffect>,
}

fn effect_execution_membership(
    eligible_effects: &[SemanticId],
    effects: &BTreeMap<SemanticId, Effect>,
    effect_analysis: &BTreeMap<SemanticId, EffectReactiveAnalysis>,
    computed_values: &BTreeMap<SemanticId, ComputedValue>,
    executable: &BTreeSet<SemanticId>,
) -> EffectExecutionMembership {
    let mut membership = EffectExecutionMembership::default();
    for effect in eligible_effects {
        if !effects.contains_key(effect) {
            continue;
        }
        let unavailable = required_computed_for_effect(effect, effect_analysis, computed_values)
            .into_iter()
            .filter(|computed| !executable.contains(computed))
            .collect::<Vec<_>>();
        if unavailable.is_empty() {
            membership.schedulable_effects.push(effect.clone());
        } else {
            membership.unplanned_effects.push(UnplannedEffect {
                effect: effect.clone(),
                reason: UnplannedEffectReason::UnavailableComputedPrerequisite,
                computed_dependencies: unavailable,
            });
        }
    }
    membership.schedulable_effects.sort();
    membership.schedulable_effects.dedup();
    membership
        .unplanned_effects
        .sort_by(|left, right| left.effect.cmp(&right.effect));
    membership
}

fn required_computed_for_effect(
    effect: &SemanticId,
    effect_analysis: &BTreeMap<SemanticId, EffectReactiveAnalysis>,
    computed_values: &BTreeMap<SemanticId, ComputedValue>,
) -> Vec<SemanticId> {
    effect_analysis
        .get(effect)
        .into_iter()
        .flat_map(|analysis| &analysis.dependencies)
        .filter(|dependency| computed_values.contains_key(*dependency))
        .cloned()
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect()
}

fn select_prerequisite_batches(
    required_computed: &BTreeSet<SemanticId>,
    computed_evaluation_plan: &IrComputedEvaluationPlan,
    computed_values: &BTreeMap<SemanticId, ComputedValue>,
) -> Vec<EffectComputedPrerequisiteBatch> {
    computed_evaluation_plan
        .update_batches
        .iter()
        .enumerate()
        .filter_map(|(index, batch)| {
            let computed = batch
                .iter()
                .filter_map(|id| {
                    computed_values
                        .keys()
                        .find(|computed| computed.as_str() == id)
                })
                .filter(|computed| required_computed.contains(*computed))
                .cloned()
                .collect::<Vec<_>>();
            (!computed.is_empty()).then_some(EffectComputedPrerequisiteBatch {
                source_batch_index: u32::try_from(index)
                    .expect("computed scheduler batch index should fit u32"),
                computed,
            })
        })
        .collect()
}

fn schedule_terminal_effects(
    effects: &[SemanticId],
    canonical_effects: &BTreeMap<SemanticId, Effect>,
) -> Vec<EffectExecutionBatch> {
    let nodes = effects
        .iter()
        .filter_map(|effect| {
            canonical_effects.get(effect).map(|canonical| {
                (
                    effect.as_str().to_string(),
                    IrReactiveNode {
                        id: effect.as_str().to_string(),
                        kind: IrReactiveNodeKind::Effect,
                        provenance: canonical.provenance.clone(),
                    },
                )
            })
        })
        .collect();
    let inspection = IrUpdateScheduler::new(IrReactiveGraph {
        nodes,
        edges: Vec::new(),
    })
    .inspect();
    inspection
        .batches
        .into_iter()
        .enumerate()
        .filter_map(|(index, batch)| {
            let mut effects = batch
                .into_iter()
                .filter_map(|id| {
                    canonical_effects
                        .keys()
                        .find(|effect| effect.as_str() == id)
                        .cloned()
                })
                .collect::<Vec<_>>();
            effects.sort_by(|left, right| {
                let left_effect = canonical_effects
                    .get(left)
                    .expect("scheduled effect should remain canonical");
                let right_effect = canonical_effects
                    .get(right)
                    .expect("scheduled effect should remain canonical");
                left_effect
                    .declaration_order
                    .cmp(&right_effect.declaration_order)
                    .then_with(|| left.cmp(right))
            });
            (!effects.is_empty()).then_some(EffectExecutionBatch {
                index: u32::try_from(index).expect("effect scheduler batch index should fit u32"),
                effects,
            })
        })
        .collect()
}

/// Lower all authored effect bodies to ordered statement records.
#[must_use]
pub fn lower_effect_bodies(
    components: &[ComponentNode],
    effects: &BTreeMap<SemanticId, Effect>,
    expression_graph: &ExpressionGraph,
) -> (
    BTreeMap<SemanticId, EffectBody>,
    BTreeMap<SemanticId, EffectStatement>,
) {
    let mut bodies = BTreeMap::new();
    let mut statements = BTreeMap::new();
    for effect in effects.values() {
        let Some(syntax) = effect_body_syntax(components, effect) else {
            continue;
        };
        let mut body_statement_ids = Vec::new();
        for (index, statement) in syntax.statements.iter().enumerate() {
            let id = effect.id.effect_statement(index);
            let path = format!("statement:{index}");
            let expression = |suffix: &str| effect.id.expression(&format!("{path}/{suffix}"));
            let kind = effect_statement_kind(statement, &expression);
            assert_effect_statement_expressions_exist(&kind, expression_graph);
            body_statement_ids.push(id.clone());
            statements.insert(
                id.clone(),
                EffectStatement {
                    id,
                    owner: effect.id.clone(),
                    kind,
                    provenance: SourceProvenance::new(&effect.provenance.path, statement.span),
                },
            );
        }
        let cleanup = syntax.cleanup.as_ref().map(|cleanup| {
            let mut cleanup_statement_ids = Vec::new();
            for (index, statement) in cleanup.body.statements.iter().enumerate() {
                let id = effect.id.effect_cleanup_statement(index);
                let path = format!("cleanup/statement:{index}");
                let expression = |suffix: &str| effect.id.expression(&format!("{path}/{suffix}"));
                let kind = effect_statement_kind(statement, &expression);
                assert_effect_statement_expressions_exist(&kind, expression_graph);
                cleanup_statement_ids.push(id.clone());
                statements.insert(
                    id.clone(),
                    EffectStatement {
                        id,
                        owner: effect.id.clone(),
                        kind,
                        provenance: SourceProvenance::new(&effect.provenance.path, statement.span),
                    },
                );
            }
            EffectCleanupBody {
                statements: cleanup_statement_ids,
                provenance: SourceProvenance::new(&effect.provenance.path, cleanup.span),
            }
        });
        bodies.insert(
            effect.id.clone(),
            EffectBody {
                effect: effect.id.clone(),
                statements: body_statement_ids,
                cleanup,
                provenance: effect.provenance.clone(),
            },
        );
    }
    (bodies, statements)
}

fn effect_statement_kind(
    statement: &crate::EffectStatementSyntax,
    expression: &impl Fn(&str) -> SemanticId,
) -> EffectStatementKind {
    match &statement.kind {
        EffectStatementSyntaxKind::StaticMemberAssignment { .. } => {
            EffectStatementKind::ExternalMemberAssignment {
                target: expression("target"),
                value: expression("value"),
            }
        }
        EffectStatementSyntaxKind::CapabilityCall { arguments, .. } => {
            EffectStatementKind::CapabilityCall {
                callee: expression("callee"),
                arguments: (0..arguments.len())
                    .map(|argument| expression(&format!("argument:{argument}")))
                    .collect(),
            }
        }
        EffectStatementSyntaxKind::EffectReturn { value } => EffectStatementKind::EffectReturn {
            value: value.as_ref().map(|_| expression("return")),
        },
        EffectStatementSyntaxKind::Empty => EffectStatementKind::Empty,
        EffectStatementSyntaxKind::Unsupported(kind) => EffectStatementKind::Unsupported(*kind),
    }
}

fn effect_is_async(components: &[ComponentNode], effect: &Effect) -> bool {
    let Some(component_id) = effect.owner.entity_id() else {
        return false;
    };
    let Some(component) = components
        .iter()
        .find(|component| component.id == *component_id)
    else {
        return false;
    };
    match &effect.declaration {
        EffectDeclaration::LegacyMethod(method_id) => component
            .methods
            .iter()
            .find(|method| method.id == *method_id)
            .is_some_and(|method| method.is_async),
        EffectDeclaration::V2Field => component
            .effect_fields
            .iter()
            .find(|field| field.name == effect.name)
            .is_some_and(|field| field.is_async),
    }
}

fn effect_has_cleanup(components: &[ComponentNode], effect: &Effect) -> bool {
    effect_body_syntax(components, effect).is_some_and(|body| body.cleanup.is_some())
}

fn effect_body_syntax<'a>(
    components: &'a [ComponentNode],
    effect: &Effect,
) -> Option<&'a crate::EffectBodySyntax> {
    let component_id = effect.owner.entity_id()?;
    let component = components
        .iter()
        .find(|component| component.id == *component_id)?;
    match &effect.declaration {
        EffectDeclaration::LegacyMethod(method_id) => component
            .methods
            .iter()
            .find(|method| method.id == *method_id)
            .and_then(|method| method.effect_body.as_ref()),
        EffectDeclaration::V2Field => component
            .effect_fields
            .iter()
            .find(|field| field.name == effect.name)
            .map(|field| &field.body),
    }
}

fn assert_effect_statement_expressions_exist(kind: &EffectStatementKind, graph: &ExpressionGraph) {
    let expressions = match kind {
        EffectStatementKind::ExternalMemberAssignment { target, value } => vec![target, value],
        EffectStatementKind::CapabilityCall { callee, arguments } => {
            let mut expressions = vec![callee];
            expressions.extend(arguments);
            expressions
        }
        EffectStatementKind::EffectReturn { value } => value.iter().collect(),
        EffectStatementKind::Empty | EffectStatementKind::Unsupported(_) => Vec::new(),
    };
    assert!(expressions.iter().all(|id| graph.node(id).is_some()));
}

#[cfg(test)]
mod tests {
    use super::{schedule_terminal_effects, Effect, EffectExecutionBatch};

    use crate::{
        build_application_semantic_model, build_component_graph, build_semantic_graph,
        collect_effects, validate_application_semantic_model, EffectDeclaration,
        EffectExecutionPolicy, EffectSemanticViolationKind, EffectStatementKind, EffectValidation,
        ExecutionBoundary, ExpressionNodeKind, SemanticEntity, SemanticEntityKind, SemanticId,
        SemanticOwner, SemanticReferenceKind, SourceProvenance, UnsupportedEffectStatementKind,
    };

    #[test]
    fn collects_stable_effect_entities_from_decorated_methods() {
        let parsed = presolve_parser::parse_file(
            "src/Effects.tsx",
            r#"
@component("x-effects")
class Effects extends Component {
  @effect()
  syncTitle() {
    document.title = "Presolve";
  }
}
"#,
        );
        let graph = build_component_graph(&parsed);
        let component = &graph.components[0];
        let effect = collect_effects(&graph.components, &graph.provenance)
            .into_values()
            .next()
            .expect("effect entity");

        assert_eq!(effect.id.as_str(), "component:x-effects/effect:syncTitle");
        assert_eq!(
            effect.declaration,
            EffectDeclaration::LegacyMethod(component.methods[0].id.clone())
        );
        assert_eq!(effect.owner, component.methods[0].owner);
        assert_eq!(effect.execution_boundary, ExecutionBoundary::Client);
        assert_eq!(
            effect.execution_policy,
            EffectExecutionPolicy::AfterInitialRenderAndCompletedActionBatch
        );
    }

    #[test]
    fn schedules_v2_effects_by_field_declaration_order() {
        let component = SemanticId::component(Some("x-v2-order"), "V2Order");
        let early = component.effect("zEarly");
        let late = component.effect("aLate");
        let provenance = SourceProvenance::new(
            "src/V2Order.tsx",
            presolve_parser::SourceSpan {
                start: 0,
                end: 0,
                line: 1,
                column: 1,
            },
        );
        let effect = |id: SemanticId, name: &str, declaration_order| Effect {
            id,
            owner: SemanticOwner::entity(component.clone()),
            declaration: EffectDeclaration::V2Field,
            name: name.to_owned(),
            declaration_order: Some(declaration_order),
            execution_boundary: ExecutionBoundary::Client,
            execution_policy: EffectExecutionPolicy::AfterInitialRenderAndCompletedActionBatch,
            validation: EffectValidation::Valid,
            semantic_violations: Vec::new(),
            provenance: provenance.clone(),
        };
        let effects = [
            (late.clone(), effect(late.clone(), "aLate", 4)),
            (early.clone(), effect(early.clone(), "zEarly", 1)),
        ]
        .into_iter()
        .collect();

        assert_eq!(
            schedule_terminal_effects(&[late, early.clone()], &effects),
            vec![EffectExecutionBatch {
                index: 0,
                effects: vec![early, component.effect("aLate")],
            }]
        );
    }

    #[test]
    fn assembles_effects_into_canonical_asm_without_reactive_products() {
        let parsed = presolve_parser::parse_file(
            "src/Effects.tsx",
            r#"
@component("x-effects")
class Effects extends Component {
  title = state("Presolve");

  @effect()
  syncTitle() {
    document.title = this.title;
  }

  render() {
    return <p>{this.title}</p>;
  }
}
"#,
        );
        let asm = build_application_semantic_model(&parsed);
        let component = &asm.components[0];
        let effect_id = component.id.effect("syncTitle");
        let effect = asm.effect(&effect_id).expect("effect entity");

        assert_eq!(
            effect.declaration,
            EffectDeclaration::LegacyMethod(component.id.method("syncTitle"))
        );
        assert_eq!(effect.owner, SemanticOwner::entity(component.id.clone()));
        assert_eq!(asm.owner(&effect_id), Some(&effect.owner));
        assert_eq!(
            asm.provenance(&effect_id),
            asm.provenance(&component.id.method("syncTitle"))
        );
        assert_eq!(
            asm.entity(&effect_id).map(SemanticEntity::kind),
            Some(SemanticEntityKind::Effect)
        );
        assert!(asm.references_from(&effect_id).iter().any(|reference| {
            reference.kind == SemanticReferenceKind::EffectState
                && reference.target == component.id.state_field("title")
        }));
        assert!(asm.semantic_type_of(&effect_id).is_none());
        assert_eq!(validate_application_semantic_model(&asm), Vec::new());
        assert!(build_semantic_graph(&asm)
            .nodes
            .iter()
            .any(|node| node.id == effect_id));
    }

    #[test]
    fn lowers_ordered_effect_statements_and_expression_operands_without_resolution() {
        let parsed = presolve_parser::parse_file(
            "src/Effects.tsx",
            include_str!("../../../fixtures/0052-effect-body-lowering/input/Effects.tsx"),
        );
        let asm = build_application_semantic_model(&parsed);
        let component = &asm.components[0];
        let sync = component.id.effect("sync");
        let body = asm.effect_body(&sync).expect("effect body");

        assert_eq!(body.statements.len(), 3);
        let assignment = asm
            .effect_statement(&body.statements[0])
            .expect("assignment");
        let call = asm.effect_statement(&body.statements[1]).expect("call");
        let completion = asm.effect_statement(&body.statements[2]).expect("return");
        let EffectStatementKind::ExternalMemberAssignment { target, value } = &assignment.kind
        else {
            panic!("expected static member assignment");
        };
        assert!(matches!(
            asm.expression(target).map(|node| &node.kind),
            Some(ExpressionNodeKind::MemberAccess { property, .. }) if property == "title"
        ));
        assert!(matches!(
            asm.expression(value).map(|node| &node.kind),
            Some(ExpressionNodeKind::ThisMember { name }) if name == "title"
        ));
        let EffectStatementKind::CapabilityCall { callee, arguments } = &call.kind else {
            panic!("expected capability call");
        };
        assert_eq!(arguments.len(), 2);
        assert!(matches!(
            asm.expression(callee).map(|node| &node.kind),
            Some(ExpressionNodeKind::MemberAccess { property, .. }) if property == "track"
        ));
        assert!(matches!(
            asm.expression(&arguments[1]).map(|node| &node.kind),
            Some(ExpressionNodeKind::Arithmetic { .. })
        ));
        assert!(matches!(
            completion.kind,
            EffectStatementKind::EffectReturn { value: None }
        ));
        assert!(asm.references_from(&sync).iter().any(|reference| {
            reference.kind == SemanticReferenceKind::EffectState
                && reference.target == component.id.state_field("title")
        }));
        assert!(asm.semantic_types.assignments.keys().any(|id| id == target));
        assert_eq!(
            asm.semantic_type_of(value),
            Some(&crate::SemanticType::String)
        );

        let invalid = asm
            .effect_body(&component.id.effect("invalid"))
            .expect("invalid body");
        assert!(matches!(
            asm.effect_statement(&invalid.statements[0])
                .map(|statement| &statement.kind),
            Some(EffectStatementKind::ExternalMemberAssignment { .. })
        ));
        assert!(matches!(
            asm.effect_statement(&invalid.statements[1])
                .map(|statement| &statement.kind),
            Some(EffectStatementKind::CapabilityCall { .. })
        ));
        assert!(matches!(
            asm.effect_statement(&invalid.statements[2])
                .map(|statement| &statement.kind),
            Some(EffectStatementKind::Unsupported(
                UnsupportedEffectStatementKind::CleanupReturnCandidate
            ))
        ));
        assert!(matches!(
            asm.effect_statement(&invalid.statements[3])
                .map(|statement| &statement.kind),
            Some(EffectStatementKind::Unsupported(
                UnsupportedEffectStatementKind::LocalDeclaration
            ))
        ));
    }

    #[test]
    #[allow(clippy::too_many_lines)]
    fn types_effect_statements_against_the_versioned_capability_registry() {
        let parsed = presolve_parser::parse_file(
            "src/Effects.tsx",
            r#"
@component("x-effects")
class Effects extends Component {
  title = state("Presolve");
  theme = state("dark");
  total = state(3);
  profile = state({ name: "Ada" });

  @action()
  increment() { this.total += 1; this.total += 1; }

  helper() {}

  @effect()
  sync() {
    document.title = this.title;
    console.log("total", this.total);
    console.info(this.title);
    console.warn(this.title);
    console.error(this.title);
    localStorage.setItem("theme", this.theme);
    localStorage.removeItem("theme");
    sessionStorage.setItem("theme", this.theme);
    sessionStorage.removeItem("theme");
  }

  @effect()
  facts() {
    document.title = this.profile;
    analytics.track(this.total);
    this.total = 1;
    this.increment();
    this.sync();
    this.helper();
    this.unknown();
    return;
  }

  render() { return <p />; }
}
"#,
        );
        let asm = build_application_semantic_model(&parsed);
        let component = &asm.components[0];
        let sync = asm
            .effect_body(&component.id.effect("sync"))
            .expect("sync body");
        let facts = asm
            .effect_body(&component.id.effect("facts"))
            .expect("facts body");
        let record = |statement: &crate::SemanticId| {
            asm.effect_statement_type(statement)
                .expect("statement type")
        };

        assert_eq!(crate::EFFECT_CAPABILITY_REGISTRY.version(), 1);
        assert_eq!(
            record(&sync.statements[0]).capability_operation,
            Some(crate::CapabilityOperationId(
                "builtin.browser.document.title.assign"
            ))
        );
        for statement in &sync.statements[1..5] {
            assert_eq!(
                record(statement).operation_classification,
                crate::EffectOperationClassification::RecognizedCapability
            );
            assert_eq!(
                record(statement).serialization_compatibility,
                crate::EffectCompatibility::Compatible
            );
        }
        for statement in &sync.statements[5..] {
            assert_eq!(
                record(statement).signature_compatibility,
                crate::EffectCompatibility::Compatible
            );
        }
        assert_eq!(
            record(&facts.statements[0]).signature_compatibility,
            crate::EffectCompatibility::Incompatible
        );
        assert_eq!(
            record(&facts.statements[1]).operation_classification,
            crate::EffectOperationClassification::UnknownExternalCapability
        );
        assert_eq!(
            record(&facts.statements[1]).signature_compatibility,
            crate::EffectCompatibility::Incompatible
        );
        assert_eq!(
            record(&facts.statements[1]).operand_types,
            vec![crate::SemanticType::Number]
        );
        assert_eq!(
            record(&facts.statements[2]).operation_classification,
            crate::EffectOperationClassification::ReactiveStateAssignment
        );
        assert_eq!(
            record(&facts.statements[3]).operation_classification,
            crate::EffectOperationClassification::ComponentActionCall
        );
        assert_eq!(
            record(&facts.statements[4]).operation_classification,
            crate::EffectOperationClassification::ComponentEffectCall
        );
        assert_eq!(
            record(&facts.statements[5]).operation_classification,
            crate::EffectOperationClassification::ComponentMethodCall
        );
        assert_eq!(
            record(&facts.statements[6]).operation_classification,
            crate::EffectOperationClassification::UnresolvedComponentCall
        );
        assert_eq!(
            record(&facts.statements[7]).operation_classification,
            crate::EffectOperationClassification::BareReturn
        );
        assert_eq!(
            asm.effect(&component.id.effect("sync"))
                .expect("valid effect")
                .validation,
            EffectValidation::Valid
        );
        let violations = &asm
            .effect(&component.id.effect("facts"))
            .expect("invalid effect")
            .semantic_violations;
        assert_eq!(
            asm.effect(&component.id.effect("facts"))
                .expect("invalid effect")
                .validation,
            EffectValidation::Invalid
        );
        for kind in [
            EffectSemanticViolationKind::CapabilitySignature,
            EffectSemanticViolationKind::UnknownExternalCapability,
            EffectSemanticViolationKind::ReactiveStateMutation,
            EffectSemanticViolationKind::ActionInvocation,
            EffectSemanticViolationKind::EffectInvocation,
            EffectSemanticViolationKind::ComponentMethodInvocation,
            EffectSemanticViolationKind::UnresolvedComponentCall,
        ] {
            assert!(violations.iter().any(|violation| violation.kind == kind));
        }
        let sync_id = component.id.effect("sync");
        let facts_id = component.id.effect("facts");
        assert_eq!(
            asm.reactive_graph.nodes[sync_id.as_str()].kind,
            crate::IrReactiveNodeKind::Effect
        );
        assert!(!asm.reactive_graph.nodes.contains_key(facts_id.as_str()));
        assert!(asm.reactive_graph.edges.iter().any(|edge| {
            edge.source == sync_id.as_str()
                && edge.target == component.id.state_field("title").as_str()
                && edge.kind == crate::IrReactiveEdgeKind::Reads
        }));
        assert!(asm.reactive_graph.edges.iter().any(|edge| {
            edge.source == component.id.state_field("title").as_str()
                && edge.target == sync_id.as_str()
                && edge.kind == crate::IrReactiveEdgeKind::Invalidates
        }));
        let analysis = asm
            .effect_reactive_analysis(&sync_id)
            .expect("effect reactive analysis");
        assert!(analysis
            .dependencies
            .contains(&component.id.state_field("title")));
        assert!(analysis
            .dependencies
            .contains(&component.id.state_field("total")));
        assert!(analysis.dependents.is_empty());
        assert!(asm.effect_reactive_analysis(&facts_id).is_none());
        let trigger_plan = asm.effect_trigger_plan();
        assert!(trigger_plan.initial_effects.contains(&sync_id));
        assert!(!trigger_plan.initial_effects.contains(&facts_id));
        let batch = trigger_plan
            .action_batches
            .get(&component.id.action_batch("increment"))
            .expect("increment action batch");
        assert_eq!(batch.ordered_write_records.len(), 2);
        assert_eq!(
            batch.written_states,
            vec![component.id.state_field("total")]
        );
        let trigger = trigger_plan
            .action_batch_triggers
            .iter()
            .find(|trigger| trigger.action_batch == batch.id)
            .expect("batch trigger");
        assert_eq!(trigger.effects, vec![sync_id.clone()]);
        assert_eq!(
            trigger.matched_states[&sync_id],
            vec![component.id.state_field("total")]
        );
        assert_eq!(validate_application_semantic_model(&asm), Vec::new());
    }

    #[test]
    #[allow(clippy::too_many_lines)]
    fn schedules_minimal_effect_prerequisites_from_existing_computed_batches() {
        let parsed = presolve_parser::parse_file(
            "src/EffectExecutionPlan.tsx",
            r#"
@component("x-effect-execution-plan")
class EffectExecutionPlan extends Component {
  price = state(1);
  locale = state("en-US");
  theme = state("light");

  @computed()
  get subtotal() { return this.price * 2; }

  @computed()
  get total() { return this.subtotal + 1; }

  @computed()
  get currentLocale() { return this.locale; }

  @computed()
  get unrelated() { return this.theme; }

  @action()
  increment() { this.price += 1; }

  @action()
  restyle() { this.theme = "dark"; }

  @effect()
  report() { console.log(this.total, this.currentLocale); }

  @effect()
  audit() { console.log(this.price); }

  @effect()
  bootLog() { console.log("ready"); }

  render() { return <p />; }
}
"#,
        );
        let asm = build_application_semantic_model(&parsed);
        let component = &asm.components[0];
        let subtotal = component.id.computed("subtotal");
        let total = component.id.computed("total");
        let current_locale = component.id.computed("currentLocale");
        let unrelated = component.id.computed("unrelated");
        let report = component.id.effect("report");
        let audit = component.id.effect("audit");
        let boot_log = component.id.effect("bootLog");
        let execution = asm.effect_execution_plan();

        assert_eq!(
            execution.initial.required_computed,
            vec![current_locale.clone(), subtotal.clone(), total.clone()]
        );
        assert_eq!(
            execution.initial.render_boundary,
            Some(crate::EffectRenderBoundary::AfterInitialRender)
        );
        assert_eq!(
            execution
                .initial
                .prerequisite_batches
                .iter()
                .map(|batch| batch.computed.clone())
                .collect::<Vec<_>>(),
            vec![vec![current_locale, subtotal.clone()], vec![total.clone()]]
        );
        assert!(!execution.initial.required_computed.contains(&unrelated));
        assert_eq!(
            execution.initial.effect_batches,
            vec![crate::EffectExecutionBatch {
                index: 0,
                effects: vec![audit.clone(), boot_log.clone(), report.clone()],
            }]
        );
        assert!(execution.initial.unplanned_effects.is_empty());

        assert_eq!(execution.actions.len(), 1);
        let action = &execution.actions[0];
        assert_eq!(action.action_batch, component.id.action_batch("increment"));
        assert_eq!(action.required_computed, vec![subtotal, total.clone()]);
        assert_eq!(
            action
                .prerequisite_batches
                .iter()
                .map(|batch| batch.computed.clone())
                .collect::<Vec<_>>(),
            vec![
                vec![component.id.computed("subtotal")],
                vec![component.id.computed("total")],
            ]
        );
        assert_eq!(
            action.effect_batches,
            vec![crate::EffectExecutionBatch {
                index: 0,
                effects: vec![audit, report.clone()],
            }]
        );
        assert!(action.unplanned_effects.is_empty());

        let unavailable_plan = crate::IrComputedEvaluationPlan {
            evaluation_order: asm
                .computed_evaluation_plan
                .evaluation_order
                .iter()
                .filter(|computed| computed.as_str() != total.as_str())
                .cloned()
                .collect(),
            update_batches: asm
                .computed_evaluation_plan
                .update_batches
                .iter()
                .map(|batch| {
                    batch
                        .iter()
                        .filter(|computed| computed.as_str() != total.as_str())
                        .cloned()
                        .collect()
                })
                .collect(),
            unplanned: vec![total.as_str().to_string()],
        };
        let unavailable = crate::plan_effect_execution(
            &asm.computed_values,
            &asm.effects,
            &asm.effect_reactive_analysis,
            asm.effect_trigger_plan(),
            &asm.reactive_transitive_analysis,
            &unavailable_plan,
        );
        assert_eq!(
            unavailable.initial.unplanned_effects,
            vec![crate::UnplannedEffect {
                effect: report,
                reason: crate::UnplannedEffectReason::UnavailableComputedPrerequisite,
                computed_dependencies: vec![total],
            }]
        );
        assert_eq!(validate_application_semantic_model(&asm), Vec::new());
    }

    #[test]
    fn rejects_async_effects_without_runtime_execution() {
        let parsed = presolve_parser::parse_file(
            "src/AsyncEffect.tsx",
            r#"
@component("x-async-effect")
class AsyncEffect extends Component {
  title = state("Presolve");

  @effect()
  async syncTitle() {
    document.title = this.title;
  }

  @effect()
  invalidValueReturn() {
    return this.title;
  }

  @effect()
  invalidCleanupReturn() {
    return () => unsubscribe();
  }

  render() { return <p />; }
}
"#,
        );
        let asm = build_application_semantic_model(&parsed);
        let effect = asm
            .effect(&asm.components[0].id.effect("syncTitle"))
            .expect("effect");

        assert_eq!(effect.validation, EffectValidation::Invalid);
        assert!(effect
            .semantic_violations
            .iter()
            .any(|violation| violation.kind == EffectSemanticViolationKind::Async));
        for (name, kind) in [
            (
                "invalidValueReturn",
                EffectSemanticViolationKind::ValueReturn,
            ),
            (
                "invalidCleanupReturn",
                EffectSemanticViolationKind::UnsupportedStatement,
            ),
        ] {
            let effect = asm
                .effect(&asm.components[0].id.effect(name))
                .expect("invalid return effect");
            assert_eq!(effect.validation, EffectValidation::Invalid);
            assert!(effect
                .semantic_violations
                .iter()
                .any(|violation| violation.kind == kind));
        }
        let codes = asm
            .diagnostics
            .iter()
            .map(|diagnostic| diagnostic.code.as_str())
            .collect::<Vec<_>>();
        assert_eq!(codes, vec!["PSC1042", "PSC1046", "PSC1046"]);
    }
}
