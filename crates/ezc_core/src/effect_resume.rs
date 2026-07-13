use std::collections::BTreeSet;

use serde::{Deserialize, Serialize};

use crate::{
    ApplicationSemanticModel, Effect, EffectExecutionPolicy, EffectRenderBoundary,
    EffectValidation, ExecutionBoundary, RuntimeEffectRegistry, SemanticId, SourceProvenance,
};

/// A stable mutable-resume identity for an effect's initial activation lifecycle.
///
/// This identity is intentionally distinct from the effect semantic entity and
/// from executable function, action-batch, computed-cache, and dirty-flag IDs.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct EffectActivationSlotId(String);

impl EffectActivationSlotId {
    #[must_use]
    pub fn for_effect(effect: &Effect) -> Option<Self> {
        effect.owner.entity_id().map(|component| {
            Self(
                component
                    .effect_activation_slot(&effect.name)
                    .as_str()
                    .to_string(),
            )
        })
    }

    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// Compiler-owned lifecycle state for initial effect activation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EffectActivationStatus {
    Pending,
    Completed,
    Failed,
}

/// The exact F9 placement of an initially activatable effect.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EffectInitialResumeMembership {
    pub render_boundary: EffectRenderBoundary,
    pub batch_index: u32,
}

/// Immutable compiler-owned resumability metadata for one runtime effect.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EffectResumeRecord {
    pub effect: SemanticId,
    pub activation_slot: Option<EffectActivationSlotId>,
    pub initial_status: Option<EffectActivationStatus>,
    pub execution_policy: EffectExecutionPolicy,
    pub runtime_function: SemanticId,
    pub initial_plan_membership: Option<EffectInitialResumeMembership>,
    pub action_batches: Vec<SemanticId>,
    pub boundary: ExecutionBoundary,
    pub provenance: SourceProvenance,
}

/// Deterministic F16 projection over runtime-effect registry membership.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct EffectResumePlan {
    pub records: Vec<EffectResumeRecord>,
}

impl EffectResumePlan {
    #[must_use]
    pub fn record(&self, effect: &SemanticId) -> Option<&EffectResumeRecord> {
        self.records.iter().find(|record| &record.effect == effect)
    }

    #[must_use]
    pub fn activation_slot(&self, effect: &SemanticId) -> Option<&EffectActivationSlotId> {
        self.record(effect)
            .and_then(|record| record.activation_slot.as_ref())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EffectResumeValidationDiagnostic {
    pub code: String,
    pub message: String,
}

/// Project F1 identity/policy, F9 membership, and F12 runtime records into
/// compiler-owned resumability metadata. This function never rereads effect
/// syntax, dependencies, capability operations, or runtime evidence.
#[must_use]
pub fn build_effect_resume_plan(
    model: &ApplicationSemanticModel,
    registry: &RuntimeEffectRegistry,
) -> EffectResumePlan {
    let records = registry
        .records
        .iter()
        .filter_map(|(effect_id, runtime)| {
            let effect = model.effects.get(effect_id)?;
            (effect.validation == EffectValidation::Valid).then_some((effect, runtime))
        })
        .filter_map(|(effect, runtime)| {
            let initial_plan_membership =
                runtime
                    .initial_trigger
                    .as_ref()
                    .map(|trigger| EffectInitialResumeMembership {
                        render_boundary: trigger.render_boundary,
                        batch_index: trigger.effect_batch_index,
                    });
            let activation_slot = initial_plan_membership
                .as_ref()
                .and_then(|_| EffectActivationSlotId::for_effect(effect));
            let action_batches = runtime
                .action_batch_triggers
                .iter()
                .map(|trigger| trigger.action_batch.clone())
                .collect::<BTreeSet<_>>()
                .into_iter()
                .collect::<Vec<_>>();
            (initial_plan_membership.is_some() || !action_batches.is_empty()).then_some(
                EffectResumeRecord {
                    effect: effect.id.clone(),
                    initial_status: activation_slot
                        .as_ref()
                        .map(|_| EffectActivationStatus::Pending),
                    activation_slot,
                    execution_policy: effect.execution_policy,
                    runtime_function: runtime.execution_function.clone(),
                    initial_plan_membership,
                    action_batches,
                    boundary: runtime.execution_boundary,
                    provenance: effect.provenance.clone(),
                },
            )
        })
        .collect();
    EffectResumePlan { records }
}

/// Validate that an F16 plan is a faithful projection of canonical inputs.
#[must_use]
#[allow(clippy::too_many_lines)]
pub fn validate_effect_resume_plan(
    model: &ApplicationSemanticModel,
    registry: &RuntimeEffectRegistry,
    plan: &EffectResumePlan,
) -> Vec<EffectResumeValidationDiagnostic> {
    let mut diagnostics = Vec::new();
    let mut effects = BTreeSet::new();
    let mut slots = BTreeSet::new();

    for record in &plan.records {
        if !effects.insert(record.effect.clone()) {
            diagnostics.push(diagnostic(
                "EZRSM1101",
                "effect resume record is duplicated",
            ));
        }
        let Some(effect) = model.effects.get(&record.effect) else {
            diagnostics.push(diagnostic(
                "EZRSM1102",
                "effect resume record has no canonical effect",
            ));
            continue;
        };
        let Some(runtime) = registry.record(&record.effect) else {
            diagnostics.push(diagnostic(
                "EZRSM1103",
                "effect resume record has no runtime effect record",
            ));
            continue;
        };
        if effect.validation != EffectValidation::Valid {
            diagnostics.push(diagnostic(
                "EZRSM1104",
                "invalid effect has a resume record",
            ));
        }
        if record.runtime_function != runtime.execution_function
            || record.runtime_function.as_str().is_empty()
        {
            diagnostics.push(diagnostic(
                "EZRSM1105",
                "effect resume record has no canonical runtime function",
            ));
        }
        if record.execution_policy != effect.execution_policy
            || record.boundary != runtime.execution_boundary
        {
            diagnostics.push(diagnostic(
                "EZRSM1106",
                "effect resume record mismatches canonical execution facts",
            ));
        }
        let expected_initial =
            runtime
                .initial_trigger
                .as_ref()
                .map(|trigger| EffectInitialResumeMembership {
                    render_boundary: trigger.render_boundary,
                    batch_index: trigger.effect_batch_index,
                });
        if record.initial_plan_membership != expected_initial {
            diagnostics.push(diagnostic(
                "EZRSM1107",
                "effect resume initial membership mismatches F9",
            ));
        }
        let expected_slots = expected_initial
            .as_ref()
            .and_then(|_| EffectActivationSlotId::for_effect(effect));
        if record.activation_slot != expected_slots {
            diagnostics.push(diagnostic(
                "EZRSM1108",
                "effect activation slot does not match initial membership",
            ));
        }
        if record.initial_status
            != record
                .activation_slot
                .as_ref()
                .map(|_| EffectActivationStatus::Pending)
        {
            diagnostics.push(diagnostic(
                "EZRSM1109",
                "effect activation status must be pending exactly when a slot exists",
            ));
        }
        if let Some(slot) = &record.activation_slot {
            if !slots.insert(slot.clone()) {
                diagnostics.push(diagnostic(
                    "EZRSM1110",
                    "effect activation slot is duplicated",
                ));
            }
        }
        let expected_batches = runtime
            .action_batch_triggers
            .iter()
            .map(|trigger| trigger.action_batch.clone())
            .collect::<BTreeSet<_>>()
            .into_iter()
            .collect::<Vec<_>>();
        if record.action_batches != expected_batches
            || record
                .action_batches
                .iter()
                .any(|batch| !model.effect_trigger_plan.action_batches.contains_key(batch))
        {
            diagnostics.push(diagnostic(
                "EZRSM1111",
                "effect resume action batches are not canonical F8 references",
            ));
        }
    }

    diagnostics
}

fn diagnostic(code: &str, message: &str) -> EffectResumeValidationDiagnostic {
    EffectResumeValidationDiagnostic {
        code: code.to_string(),
        message: message.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        build_application_semantic_model, build_effect_resume_plan, build_runtime_effect_registry,
        lower_components_to_ir, optimize_effect_ir, validate_effect_resume_plan,
        EffectActivationStatus,
    };

    #[test]
    fn plans_stable_initial_slots_and_canonical_action_batches() {
        let parsed = ezc_parser::parse_file(
            "src/EffectResume.tsx",
            r#"
@component("x-effect-resume")
class EffectResume extends Component {
  count = state(1);
  label = state("ready");

  @computed()
  get doubled() { return this.count * 2; }

  @action()
  increment() { this.count += 1; }

  @action()
  rename() { this.label = "done"; }

  @effect()
  sync() { console.log(this.doubled); console.log(this.label); }

  @effect()
  ready() { console.log("ready"); }

  @effect()
  invalid() { this.count = 0; }
}
"#,
        );
        let model = build_application_semantic_model(&parsed);
        let registry = build_runtime_effect_registry(
            &model,
            &optimize_effect_ir(&lower_components_to_ir(&model)).output,
        );
        let plan = build_effect_resume_plan(&model, &registry);
        let sync = model.components[0].id.effect("sync");
        let ready = model.components[0].id.effect("ready");
        let invalid = model.components[0].id.effect("invalid");
        let sync_record = plan.record(&sync).expect("sync resume record");
        let ready_record = plan.record(&ready).expect("ready resume record");

        assert_eq!(plan.records.len(), 2);
        assert_eq!(
            sync_record.initial_status,
            Some(EffectActivationStatus::Pending)
        );
        assert!(sync_record.initial_plan_membership.is_some());
        assert_eq!(sync_record.action_batches.len(), 2);
        assert!(ready_record.activation_slot.is_some());
        assert!(ready_record.action_batches.is_empty());
        assert_ne!(sync_record.activation_slot, ready_record.activation_slot);
        assert!(plan.record(&invalid).is_none());
        assert!(validate_effect_resume_plan(&model, &registry, &plan).is_empty());

        let repeated = build_effect_resume_plan(&model, &registry);
        assert_eq!(plan, repeated);

        let mut malformed = plan.clone();
        let sync_index = malformed
            .records
            .iter()
            .position(|record| record.effect == sync)
            .expect("sync record index");
        let ready_index = malformed
            .records
            .iter()
            .position(|record| record.effect == ready)
            .expect("ready record index");
        malformed.records[sync_index].runtime_function = ready.clone();
        malformed.records[sync_index]
            .action_batches
            .push(model.components[0].id.action_batch("missing"));
        malformed.records[ready_index].activation_slot =
            malformed.records[sync_index].activation_slot.clone();
        let codes = validate_effect_resume_plan(&model, &registry, &malformed)
            .into_iter()
            .map(|diagnostic| diagnostic.code)
            .collect::<Vec<_>>();
        assert!(codes.contains(&"EZRSM1105".to_string()));
        assert!(codes.contains(&"EZRSM1110".to_string()));
        assert!(codes.contains(&"EZRSM1111".to_string()));
    }
}
