use crate::application_semantic_model::ApplicationSemanticModel;
use crate::component_graph::render_event_handlers;
use crate::semantic_id::SemanticId;
use crate::{
    build_context_resume_plan, build_context_update_plan, build_effect_resume_plan,
    build_runtime_computed_registry, build_runtime_context_registry, build_runtime_effect_registry,
    lower_components_to_ir, optimize_context_ir, optimize_effect_ir, ContextResumePlan,
    EffectResumePlan,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResumePlan {
    pub components: Vec<ResumeComponentPlan>,
    pub effects: EffectResumePlan,
    pub contexts: ContextResumePlan,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ResumeComponentPlan {
    pub component: SemanticId,
    pub state: Vec<SemanticId>,
    pub computed: Vec<ResumeComputedPlan>,
    pub events: Vec<SemanticId>,
}

/// One serializable, compiler-lowered computed cache available to resumability.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ResumeComputedPlan {
    pub computed: SemanticId,
    pub cache_slot: String,
    pub dirty_flag: String,
    pub initial_dirty: bool,
}

#[must_use]
pub fn build_resume_plan(model: &ApplicationSemanticModel) -> ResumePlan {
    let ir = lower_components_to_ir(model);
    let registry = build_runtime_computed_registry(model, &ir);
    let effect_registry = build_runtime_effect_registry(model, &optimize_effect_ir(&ir).output);
    let context_ir = optimize_context_ir(&ir);
    let context_registry = build_runtime_context_registry(model, &context_ir);
    let context_updates = build_context_update_plan(model, &context_ir);

    ResumePlan {
        components: model
            .components
            .iter()
            .map(|component| ResumeComponentPlan {
                component: component.id.clone(),
                state: component
                    .state_fields
                    .iter()
                    .map(|field| field.id.clone())
                    .collect(),
                computed: registry
                    .records
                    .values()
                    .filter(|record| {
                        record.computed.as_str().starts_with(component.id.as_str())
                            && record.serialization
                                == crate::SerializationCompatibility::Serializable
                    })
                    .map(|record| ResumeComputedPlan {
                        computed: record.computed.clone(),
                        cache_slot: record.cache_slot.as_str().to_string(),
                        dirty_flag: record.dirty_flag.id.clone(),
                        initial_dirty: record.dirty_flag.initial_value,
                    })
                    .collect(),
                events: component.render.as_ref().map_or_else(Vec::new, |render| {
                    render_event_handlers(render)
                        .into_iter()
                        .map(|handler| handler.id.clone())
                        .collect()
                }),
            })
            .collect(),
        effects: build_effect_resume_plan(model, &effect_registry),
        contexts: build_context_resume_plan(&context_registry, &context_updates),
    }
}

#[cfg(test)]
mod tests {
    use crate::{build_application_semantic_model, build_resume_plan};

    #[test]
    fn plans_only_serializable_lowered_computed_caches_for_resumption() {
        let parsed = ezc_parser::parse_file(
            "src/ResumeComputed.tsx",
            r#"
@component("x-resume-computed")
class ResumeComputed extends Component {
  count = state(1);

  @computed()
  get doubled() { return this.count * 2; }

  @computed()
  get unresolved() { return this.missing; }
}
"#,
        );
        let model = build_application_semantic_model(&parsed);
        let component = &model.components[0];
        let doubled = component.id.computed("doubled");
        let plan = build_resume_plan(&model);
        let component_plan = &plan.components[0];

        assert_eq!(component_plan.computed.len(), 1);
        assert_eq!(component_plan.computed[0].computed, doubled);
        assert_eq!(
            component_plan.computed[0].cache_slot,
            format!("{doubled}/runtime:cache")
        );
        assert_eq!(
            component_plan.computed[0].dirty_flag,
            format!("{doubled}/runtime:dirty")
        );
        assert!(component_plan.computed[0].initial_dirty);
    }
}
