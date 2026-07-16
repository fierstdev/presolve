use serde::{Deserialize, Serialize};

use crate::{
    ContextValueSlotId, ContextValueSourceId, ExecutionBoundary, RuntimeContextRegistry,
    RuntimeContextSourceKind, SourceProvenance,
};

/// Stable serialized-state identity distinct from the live runtime slot.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct ContextResumeSlotId(String);

impl ContextResumeSlotId {
    #[must_use]
    pub fn for_source(source: &ContextValueSourceId) -> Self {
        let source = match source {
            ContextValueSourceId::Provider(provider) => provider.as_str().to_string(),
            ContextValueSourceId::ContextDefault(context) => {
                format!("{}/default", context.as_str())
            }
        };
        Self(format!("{source}/resume:context-slot"))
    }

    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ContextSlotResumeStatus {
    Uninitialized,
    Initialized,
    Failed,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ContextResumeRecord {
    pub source: ContextValueSourceId,
    pub context: crate::ContextId,
    pub runtime_slot: ContextValueSlotId,
    pub resume_slot: ContextResumeSlotId,
    pub semantic_type: crate::SemanticTypeId,
    pub source_kind: RuntimeContextSourceKind,
    pub initial_status: ContextSlotResumeStatus,
    pub action_batches: Vec<crate::SemanticId>,
    pub consumers: Vec<crate::ConsumerId>,
    pub boundary: ExecutionBoundary,
    pub provenance: SourceProvenance,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ContextResumePlan {
    pub records: Vec<ContextResumeRecord>,
}

#[must_use]
pub fn build_context_resume_plan(
    registry: &RuntimeContextRegistry,
    updates: &crate::ContextUpdatePlan,
) -> ContextResumePlan {
    ContextResumePlan {
        records: registry
            .sources
            .iter()
            .map(|source| ContextResumeRecord {
                source: source.source.clone(),
                context: source.context.clone(),
                runtime_slot: source.slot.clone(),
                resume_slot: ContextResumeSlotId::for_source(&source.source),
                semantic_type: source.semantic_type.clone(),
                source_kind: source.source_kind,
                initial_status: ContextSlotResumeStatus::Uninitialized,
                action_batches: updates
                    .actions
                    .iter()
                    .filter(|action| action.invalidated_sources.contains(&source.source))
                    .map(|action| action.action_batch.clone())
                    .collect(),
                consumers: registry
                    .consumers
                    .iter()
                    .filter(|consumer| consumer.selected_source == source.source)
                    .map(|consumer| consumer.consumer.clone())
                    .collect(),
                boundary: source.boundary,
                provenance: source.provenance.clone(),
            })
            .collect(),
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        build_application_semantic_model, build_resume_manifest, build_resume_plan,
        ContextSlotResumeStatus,
    };

    #[test]
    fn plans_distinct_uninitialized_resume_slots_for_runtime_context_sources() {
        let model = build_application_semantic_model(&ezc_parser::parse_file(
            "src/App.tsx",
            r#"
@component("x-app")
class App extends Component {
  @context() locale: string = "en";
  @consume(App.locale) locale!: string;
  render() { return <main />; }
}
"#,
        ));
        let plan = build_resume_plan(&model);
        let record = plan
            .contexts
            .records
            .first()
            .expect("Context resume record");
        assert_eq!(
            record.initial_status,
            ContextSlotResumeStatus::Uninitialized
        );
        assert_ne!(record.resume_slot.as_str(), record.runtime_slot.as_str());
        let manifest = build_resume_manifest(&plan);
        assert_eq!(manifest.schema_version, 5);
        assert_eq!(manifest.context_slots.len(), 1);
    }
}
