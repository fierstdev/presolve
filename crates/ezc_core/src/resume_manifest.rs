use std::collections::BTreeSet;

use serde::{Deserialize, Serialize};

use crate::effect::{EffectExecutionPolicy, EffectRenderBoundary};
use crate::effect_resume::{EffectActivationSlotId, EffectActivationStatus};
use crate::resume_plan::ResumePlan;
use crate::semantic_type::ExecutionBoundary;

pub const RESUME_MANIFEST_SCHEMA_VERSION: u32 = 4;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ResumeManifest {
    pub schema_version: u32,
    pub components: Vec<crate::resume_plan::ResumeComponentPlan>,
    /// Compiler-owned initial-effect activation metadata. This stores no live
    /// browser, capability, DOM, or interpreter state.
    #[serde(default)]
    pub effects: Vec<ResumeManifestEffectRecord>,
    #[serde(default)]
    pub context_slots: Vec<ResumeManifestContextSlotRecord>,
    #[serde(default)]
    pub component_instances: Vec<crate::resume_plan::ComponentInstanceResumePlan>,
    #[serde(default)]
    pub structural_regions: Vec<crate::resume_plan::StructuralRegionResumePlan>,
    #[serde(default)]
    pub slot_bindings: Vec<crate::resume_plan::SlotBindingResumePlan>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ResumeManifestContextSlotRecord {
    pub source: String,
    pub context_id: String,
    pub runtime_slot_id: String,
    pub resume_slot_id: crate::ContextResumeSlotId,
    pub semantic_type: String,
    pub source_kind: ResumeManifestContextSourceKind,
    pub initial_status: crate::ContextSlotResumeStatus,
    pub action_batch_ids: Vec<String>,
    pub consumer_ids: Vec<String>,
    pub execution_boundary: ResumeManifestEffectExecutionBoundary,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ResumeManifestContextSourceKind {
    Provider,
    ContextDefault,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ResumeManifestEffectRecord {
    pub effect_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub activation_slot_id: Option<EffectActivationSlotId>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub initial_status: Option<EffectActivationStatus>,
    pub execution_policy: ResumeManifestEffectExecutionPolicy,
    pub runtime_function_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub initial_plan: Option<ResumeManifestInitialEffectPlan>,
    pub action_batch_ids: Vec<String>,
    pub execution_boundary: ResumeManifestEffectExecutionBoundary,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ResumeManifestInitialEffectPlan {
    pub render_boundary: ResumeManifestEffectRenderBoundary,
    pub batch_index: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ResumeManifestEffectExecutionPolicy {
    AfterInitialRenderAndCompletedActionBatch,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ResumeManifestEffectRenderBoundary {
    AfterInitialRender,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ResumeManifestEffectExecutionBoundary {
    Client,
    Server,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResumeManifestValidationDiagnostic {
    pub code: String,
    pub message: String,
}

#[must_use]
pub fn build_resume_manifest(plan: &ResumePlan) -> ResumeManifest {
    ResumeManifest {
        schema_version: RESUME_MANIFEST_SCHEMA_VERSION,
        components: plan.components.clone(),
        effects: plan
            .effects
            .records
            .iter()
            .map(|record| ResumeManifestEffectRecord {
                effect_id: record.effect.to_string(),
                activation_slot_id: record.activation_slot.clone(),
                initial_status: record.initial_status,
                execution_policy: execution_policy(record.execution_policy),
                runtime_function_id: record.runtime_function.to_string(),
                initial_plan: record.initial_plan_membership.as_ref().map(|membership| {
                    ResumeManifestInitialEffectPlan {
                        render_boundary: render_boundary(membership.render_boundary),
                        batch_index: membership.batch_index,
                    }
                }),
                action_batch_ids: record
                    .action_batches
                    .iter()
                    .map(ToString::to_string)
                    .collect(),
                execution_boundary: execution_boundary(record.boundary),
            })
            .collect(),
        context_slots: plan
            .contexts
            .records
            .iter()
            .map(|record| ResumeManifestContextSlotRecord {
                source: match &record.source {
                    crate::ContextValueSourceId::Provider(provider) => {
                        provider.as_str().to_string()
                    }
                    crate::ContextValueSourceId::ContextDefault(context) => {
                        format!("{}/default", context.as_str())
                    }
                },
                context_id: record.context.as_str().to_string(),
                runtime_slot_id: record.runtime_slot.as_str().to_string(),
                resume_slot_id: record.resume_slot.clone(),
                semantic_type: record.semantic_type.to_string(),
                source_kind: match record.source_kind {
                    crate::RuntimeContextSourceKind::Provider => {
                        ResumeManifestContextSourceKind::Provider
                    }
                    crate::RuntimeContextSourceKind::ContextDefault => {
                        ResumeManifestContextSourceKind::ContextDefault
                    }
                },
                initial_status: record.initial_status,
                action_batch_ids: record
                    .action_batches
                    .iter()
                    .map(ToString::to_string)
                    .collect(),
                consumer_ids: record.consumers.iter().map(ToString::to_string).collect(),
                execution_boundary: execution_boundary(record.boundary),
            })
            .collect(),
        component_instances: plan.component_instances.clone(),
        structural_regions: plan.structural_regions.clone(),
        slot_bindings: plan.slot_bindings.clone(),
    }
}

#[must_use]
///
/// # Panics
///
/// Panics when the compiler-owned resume manifest cannot serialize.
pub fn resume_manifest_json(manifest: &ResumeManifest) -> String {
    serde_json::to_string_pretty(manifest).expect("resume manifest should serialize")
}

/// Validate schema-v3 compiler-owned effect activation and Context slot metadata.
///
/// Legacy v1 manifests remain deserializable for consumers that do not attempt
/// effect restoration, but this validation rejects them because they cannot
/// carry the required activation state.
#[must_use]
pub fn validate_resume_manifest(
    manifest: &ResumeManifest,
) -> Vec<ResumeManifestValidationDiagnostic> {
    let mut diagnostics = Vec::new();
    if manifest.schema_version != RESUME_MANIFEST_SCHEMA_VERSION {
        diagnostics.push(diagnostic(
            "EZRSM1201",
            "resume manifest schema does not contain compiler-owned effect activation state",
        ));
        return diagnostics;
    }
    let mut effects = BTreeSet::new();
    let mut slots = BTreeSet::new();
    for effect in &manifest.effects {
        if !effects.insert(effect.effect_id.clone()) {
            diagnostics.push(diagnostic(
                "EZRSM1202",
                "resume manifest effect ID is duplicated",
            ));
        }
        if effect.runtime_function_id.is_empty() {
            diagnostics.push(diagnostic(
                "EZRSM1203",
                "resume manifest effect has no runtime function ID",
            ));
        }
        if effect.activation_slot_id.is_some() != effect.initial_plan.is_some() {
            diagnostics.push(diagnostic(
                "EZRSM1204",
                "activation slot and initial plan must appear together",
            ));
        }
        if effect.initial_status.is_some() != effect.activation_slot_id.is_some() {
            diagnostics.push(diagnostic(
                "EZRSM1205",
                "initial status must appear exactly when an activation slot exists",
            ));
        }
        if let Some(slot) = &effect.activation_slot_id {
            if !slots.insert(slot.clone()) {
                diagnostics.push(diagnostic(
                    "EZRSM1206",
                    "resume manifest activation slot is duplicated",
                ));
            }
        }
        if effect
            .action_batch_ids
            .windows(2)
            .any(|pair| pair[0] >= pair[1])
        {
            diagnostics.push(diagnostic(
                "EZRSM1207",
                "resume manifest action batches are not deterministic",
            ));
        }
    }
    diagnostics
}

fn execution_policy(policy: EffectExecutionPolicy) -> ResumeManifestEffectExecutionPolicy {
    match policy {
        EffectExecutionPolicy::AfterInitialRenderAndCompletedActionBatch => {
            ResumeManifestEffectExecutionPolicy::AfterInitialRenderAndCompletedActionBatch
        }
    }
}

fn render_boundary(boundary: EffectRenderBoundary) -> ResumeManifestEffectRenderBoundary {
    match boundary {
        EffectRenderBoundary::AfterInitialRender => {
            ResumeManifestEffectRenderBoundary::AfterInitialRender
        }
    }
}

fn execution_boundary(boundary: ExecutionBoundary) -> ResumeManifestEffectExecutionBoundary {
    match boundary {
        ExecutionBoundary::Client => ResumeManifestEffectExecutionBoundary::Client,
        ExecutionBoundary::Server => ResumeManifestEffectExecutionBoundary::Server,
    }
}

fn diagnostic(code: &str, message: &str) -> ResumeManifestValidationDiagnostic {
    ResumeManifestValidationDiagnostic {
        code: code.to_string(),
        message: message.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::ResumeManifest;
    use crate::{
        build_application_semantic_model, build_resume_manifest, build_resume_plan,
        resume_manifest_json, validate_resume_manifest, EffectActivationStatus,
    };

    #[test]
    fn serializes_planned_computed_cache_metadata() {
        let parsed = ezc_parser::parse_file(
            "src/ResumeManifestComputed.tsx",
            r#"
@component("x-resume-manifest-computed")
class ResumeManifestComputed extends Component {
  count = state(1);

  @computed()
  get doubled() { return this.count * 2; }
}
"#,
        );
        let model = build_application_semantic_model(&parsed);
        let doubled = model.components[0].id.computed("doubled");
        let manifest = build_resume_manifest(&build_resume_plan(&model));
        let json: serde_json::Value =
            serde_json::from_str(&resume_manifest_json(&manifest)).expect("resume manifest JSON");

        assert_eq!(json["schema_version"], 4);
        assert_eq!(json["effects"], serde_json::json!([]));
        assert_eq!(
            json["components"][0]["computed"][0]["computed"],
            doubled.as_str()
        );
        assert_eq!(
            json["components"][0]["computed"][0]["cache_slot"],
            format!("{doubled}/runtime:cache")
        );
        assert_eq!(json["components"][0]["computed"][0]["initial_dirty"], true);
        assert!(validate_resume_manifest(&manifest).is_empty());
    }

    #[test]
    fn serializes_deterministic_effect_activation_metadata() {
        let parsed = ezc_parser::parse_file(
            "src/ResumeManifestEffect.tsx",
            r#"
@component("x-resume-manifest-effect")
class ResumeManifestEffect extends Component {
  count = state(1);

  @action()
  increment() { this.count += 1; }

  @effect()
  sync() { console.log(this.count); }
}
"#,
        );
        let model = build_application_semantic_model(&parsed);
        let manifest = build_resume_manifest(&build_resume_plan(&model));
        let json = resume_manifest_json(&manifest);
        let repeated = resume_manifest_json(&build_resume_manifest(&build_resume_plan(&model)));
        let value: serde_json::Value = serde_json::from_str(&json).expect("resume JSON");
        let effect = &value["effects"][0];

        assert_eq!(json, repeated);
        assert_eq!(value["schema_version"], 4);
        assert_eq!(effect["initial_status"], "pending");
        assert_eq!(
            effect["activation_slot_id"],
            "module:src/ResumeManifestEffect.tsx/component:x-resume-manifest-effect/effect-activation:sync"
        );
        assert_eq!(
            effect["initial_plan"]["render_boundary"],
            "after_initial_render"
        );
        assert_eq!(effect["action_batch_ids"].as_array().map(Vec::len), Some(1));
        assert!(validate_resume_manifest(&manifest).is_empty());

        let mut malformed = manifest.clone();
        malformed.effects[0].initial_status = Some(EffectActivationStatus::Completed);
        assert!(validate_resume_manifest(&malformed).is_empty());
    }

    #[test]
    fn rejects_legacy_schema_for_effect_restoration_validation() {
        let legacy: ResumeManifest =
            serde_json::from_str(r#"{"schema_version":1,"components":[]}"#)
                .expect("legacy manifest remains readable");
        let diagnostics = validate_resume_manifest(&legacy);

        assert_eq!(diagnostics[0].code, "EZRSM1201");
    }
}
