use std::collections::BTreeMap;

use serde::Serialize;

use crate::{
    build_context_resume_plan, build_context_update_plan, build_runtime_context_registry,
    lower_components_to_ir, optimize_context_ir, ApplicationSemanticModel, SemanticId,
};

/// One shared compiler projection used by all Context inspection consumers.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ContextInspection {
    pub target_context: Option<String>,
    pub selected_source: Option<String>,
    pub source_plan_status: Option<String>,
    pub evaluation_batch: Option<u32>,
    pub slot_id: Option<String>,
    pub ir_function_id: Option<String>,
    pub load_identity: Option<String>,
    pub runtime_registration: bool,
    pub action_update_membership: Vec<String>,
    pub resumable: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ContextInspectionRegistry {
    pub records: BTreeMap<SemanticId, ContextInspection>,
}

#[must_use]
pub fn build_context_inspection_registry(
    model: &ApplicationSemanticModel,
) -> ContextInspectionRegistry {
    let ir = lower_components_to_ir(model);
    let optimized = optimize_context_ir(&ir);
    let runtime = build_runtime_context_registry(model, &optimized);
    let updates = build_context_update_plan(model, &optimized);
    let resume = build_context_resume_plan(&runtime, &updates);
    let mut records = BTreeMap::new();
    for context in model.contexts.values() {
        let source = runtime
            .sources
            .iter()
            .find(|source| source.context == context.id);
        records.insert(
            context.id.as_semantic_id().clone(),
            ContextInspection {
                target_context: Some(context.id.to_string()),
                selected_source: source.map(|source| source_id(&source.source)),
                source_plan_status: source
                    .and_then(|source| model.context_evaluation.context_source_plan(&source.source))
                    .map(|entry| format!("{:?}", entry.status)),
                evaluation_batch: source.map(|source| source.evaluation_batch.index),
                slot_id: source.map(|source| source.slot.as_str().to_string()),
                ir_function_id: source.map(|source| source.function.as_semantic_id().to_string()),
                load_identity: None,
                runtime_registration: source.is_some(),
                action_update_membership: Vec::new(),
                resumable: source.is_some(),
            },
        );
    }
    for provider in model.providers.values() {
        let source_id_value = crate::ContextValueSourceId::Provider(provider.id.clone());
        let source = runtime.source(&source_id_value);
        records.insert(
            provider.id.as_semantic_id().clone(),
            ContextInspection {
                target_context: Some(provider.context.to_string()),
                selected_source: source.map(|_| source_id(&source_id_value)),
                source_plan_status: model
                    .context_evaluation
                    .context_source_plan(&source_id_value)
                    .map(|entry| format!("{:?}", entry.status)),
                evaluation_batch: source.map(|source| source.evaluation_batch.index),
                slot_id: source.map(|source| source.slot.as_str().to_string()),
                ir_function_id: source.map(|source| source.function.as_semantic_id().to_string()),
                load_identity: None,
                runtime_registration: source.is_some(),
                action_update_membership: updates
                    .actions
                    .iter()
                    .filter(|action| action.invalidated_sources.contains(&source_id_value))
                    .map(|action| action.action_batch.to_string())
                    .collect(),
                resumable: resume
                    .records
                    .iter()
                    .any(|record| record.source == source_id_value),
            },
        );
    }
    for consumer in model.consumers.values() {
        let binding = optimized
            .optimized_module
            .context_ir
            .context_consumer_binding(&consumer.id);
        records.insert(
            consumer.id.as_semantic_id().clone(),
            ContextInspection {
                target_context: consumer.context().map(ToString::to_string),
                selected_source: binding.map(|binding| source_id(&binding.source)),
                source_plan_status: binding
                    .and_then(|binding| {
                        model
                            .context_evaluation
                            .context_source_plan(&binding.source)
                    })
                    .map(|entry| format!("{:?}", entry.status)),
                evaluation_batch: binding
                    .and_then(|binding| runtime.source(&binding.source))
                    .map(|source| source.evaluation_batch.index),
                slot_id: binding.map(|binding| binding.slot.as_str().to_string()),
                ir_function_id: None,
                load_identity: binding.map(|binding| binding.load.id.as_semantic_id().to_string()),
                runtime_registration: binding.is_some(),
                action_update_membership: Vec::new(),
                resumable: binding.is_some(),
            },
        );
    }
    ContextInspectionRegistry { records }
}

fn source_id(source: &crate::ContextValueSourceId) -> String {
    match source {
        crate::ContextValueSourceId::Provider(provider) => provider.to_string(),
        crate::ContextValueSourceId::ContextDefault(context) => format!("{context}/default"),
    }
}

#[cfg(test)]
mod tests {
    use crate::{build_application_semantic_model, build_context_inspection_registry, ProviderId};

    #[test]
    fn projects_context_provider_and_consumer_from_one_compiler_owned_registry() {
        let model = build_application_semantic_model(&presolve_parser::parse_file(
            "src/App.tsx",
            r#"
@component("x-app") class App extends Component {
  @context() theme: string = "dark";
  @provide(App.theme) providedTheme: string = "light";
  @consume(App.theme) theme!: string;
  render() { return <main />; }
}
"#,
        ));
        let component = &model.components[0].id;
        let registry = build_context_inspection_registry(&model);
        assert!(registry.records.contains_key(&component.context("theme")));
        let provider = ProviderId::for_component(component, "providedTheme");
        let record = registry
            .records
            .get(provider.as_semantic_id())
            .expect("Provider inspection");
        assert!(record.runtime_registration);
        assert!(record.slot_id.is_some());
    }
}
