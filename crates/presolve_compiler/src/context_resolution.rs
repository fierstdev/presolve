use std::collections::BTreeMap;

use crate::{
    ComponentScopeGraph, ConsumerEntity, ConsumerId, ContextEntity, ContextId, ExpressionGraph,
    ProviderEntity, ProviderId, SemanticId, SourceProvenance,
};

/// Immutable compiler-owned Context binding result for one canonical Consumer.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ContextResolution {
    pub consumer: ConsumerId,
    pub context: Option<ContextId>,
    pub result: ContextResolutionResult,
    pub searched_scopes: Vec<SemanticId>,
    pub provenance: SourceProvenance,
}

/// The complete G4 result domain. No variant contains a runtime value, lookup,
/// slot, or component instance.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ContextResolutionResult {
    Provider {
        provider: ProviderId,
        provider_owner: SemanticId,
        distance: u32,
    },
    ContextDefault {
        context: ContextId,
        expression: SemanticId,
    },
    Unresolved,
    Ambiguous {
        providers: Vec<ProviderId>,
        distance: u32,
    },
    InvalidContextReference,
}

/// Resolves Consumers using only the supplied immutable scope graph and
/// canonical Provider/Context products.
#[must_use]
pub fn collect_context_resolutions(
    consumers: &BTreeMap<ConsumerId, ConsumerEntity>,
    contexts: &BTreeMap<ContextId, ContextEntity>,
    providers: &BTreeMap<ProviderId, ProviderEntity>,
    expression_graph: &ExpressionGraph,
    component_scope: &ComponentScopeGraph,
) -> BTreeMap<ConsumerId, ContextResolution> {
    let provider_index = provider_index(providers);
    consumers
        .iter()
        .map(|(id, consumer)| {
            (
                id.clone(),
                resolve_consumer(
                    consumer,
                    contexts,
                    expression_graph,
                    component_scope,
                    &provider_index,
                ),
            )
        })
        .collect()
}

fn provider_index(
    providers: &BTreeMap<ProviderId, ProviderEntity>,
) -> BTreeMap<(SemanticId, ContextId), Vec<ProviderId>> {
    let mut index = BTreeMap::<(SemanticId, ContextId), Vec<ProviderId>>::new();
    for provider in providers.values() {
        let Some(owner) = provider.owner.entity_id() else {
            continue;
        };
        index
            .entry((owner.clone(), provider.context.clone()))
            .or_default()
            .push(provider.id.clone());
    }
    for ids in index.values_mut() {
        ids.sort();
    }
    index
}

fn resolve_consumer(
    consumer: &ConsumerEntity,
    contexts: &BTreeMap<ContextId, ContextEntity>,
    expression_graph: &ExpressionGraph,
    component_scope: &ComponentScopeGraph,
    provider_index: &BTreeMap<(SemanticId, ContextId), Vec<ProviderId>>,
) -> ContextResolution {
    let provenance = consumer.context_designator.provenance.clone();
    let Some(context) = consumer.context().cloned() else {
        return ContextResolution {
            consumer: consumer.id.clone(),
            context: None,
            result: ContextResolutionResult::InvalidContextReference,
            searched_scopes: Vec::new(),
            provenance,
        };
    };
    let Some(owner) = consumer.owner.entity_id() else {
        return ContextResolution {
            consumer: consumer.id.clone(),
            context: Some(context),
            result: ContextResolutionResult::Unresolved,
            searched_scopes: Vec::new(),
            provenance,
        };
    };
    let searched_scopes = component_scope.ancestor_chain(owner);

    for (distance, scope) in searched_scopes.iter().enumerate() {
        let candidates = provider_index
            .get(&(scope.clone(), context.clone()))
            .cloned()
            .unwrap_or_default();
        match candidates.as_slice() {
            [] => {}
            [provider] => {
                return ContextResolution {
                    consumer: consumer.id.clone(),
                    context: Some(context),
                    result: ContextResolutionResult::Provider {
                        provider: provider.clone(),
                        provider_owner: scope.clone(),
                        distance: u32::try_from(distance)
                            .expect("component scope depth should fit in u32"),
                    },
                    searched_scopes,
                    provenance,
                };
            }
            _ => {
                return ContextResolution {
                    consumer: consumer.id.clone(),
                    context: Some(context),
                    result: ContextResolutionResult::Ambiguous {
                        providers: candidates,
                        distance: u32::try_from(distance)
                            .expect("component scope depth should fit in u32"),
                    },
                    searched_scopes,
                    provenance,
                };
            }
        }
    }

    let result = contexts
        .get(&context)
        .filter(|context_entity| context_entity.default_expression.is_some())
        .and_then(|_| expression_graph.root_for(context.as_semantic_id()))
        .cloned()
        .map_or(ContextResolutionResult::Unresolved, |expression| {
            ContextResolutionResult::ContextDefault {
                context: context.clone(),
                expression,
            }
        });
    ContextResolution {
        consumer: consumer.id.clone(),
        context: Some(context),
        result,
        searched_scopes,
        provenance,
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use super::{collect_context_resolutions, ContextResolutionResult};
    use crate::{
        build_application_semantic_model, validate_application_semantic_model, ComponentScopeGraph,
        ConsumerId, ProviderId,
    };

    #[test]
    fn resolves_same_component_provider_with_a_canonical_relation() {
        let asm = build_application_semantic_model(&presolve_parser::parse_file(
            "src/toolbar.tsx",
            r#"
@component("x-app-shell")
class AppShell extends Component {
  @context()
  theme!: Theme;
  render() { return <main />; }
}
@component("x-toolbar")
class Toolbar extends Component {
  @provide(AppShell.theme)
  providedTheme: Theme = this.localTheme;
  @consume(AppShell.theme)
  theme!: Theme;
  render() { return <main />; }
}
"#,
        ));
        let toolbar = &asm.components[1];
        let consumer = ConsumerId::for_component(&toolbar.id, "theme");
        let provider = ProviderId::for_component(&toolbar.id, "providedTheme");
        let resolution = asm.context_resolution(&consumer).unwrap();

        assert!(matches!(
            resolution.result,
            ContextResolutionResult::Provider {
                provider: ref resolved,
                distance: 0,
                ..
            } if *resolved == provider
        ));
        assert_eq!(asm.resolved_provider(&consumer), Some(&provider));
        assert_eq!(asm.consumers_resolved_to(&provider), vec![&consumer]);
        assert!(asm.references.iter().any(|reference| {
            reference.kind == crate::SemanticReferenceKind::ResolvesToProvider
                && reference.source == *consumer.as_semantic_id()
                && reference.target == *provider.as_semantic_id()
                && reference.provenance == resolution.provenance
        }));
        assert!(validate_application_semantic_model(&asm).is_empty());
    }

    #[test]
    fn uses_context_default_only_when_no_visible_provider_exists() {
        let asm = build_application_semantic_model(&presolve_parser::parse_file(
            "src/toolbar.tsx",
            r#"
@component("x-app-shell")
class AppShell extends Component {
  @context()
  locale: string = "en";
  render() { return <main />; }
}
@component("x-toolbar")
class Toolbar extends Component {
  @consume(AppShell.locale)
  locale!: string;
  render() { return <main />; }
}
"#,
        ));
        let consumer = ConsumerId::for_component(&asm.components[1].id, "locale");
        let context = asm.consumers()[0].context().unwrap().clone();

        assert!(matches!(
            asm.context_resolution(&consumer).unwrap().result,
            ContextResolutionResult::ContextDefault {
                context: ref resolved, ..
            } if *resolved == context
        ));
        assert_eq!(asm.consumers_using_default(&context), vec![&consumer]);
        assert!(asm.resolved_provider(&consumer).is_none());
        assert!(validate_application_semantic_model(&asm).is_empty());
    }

    #[test]
    fn leaves_cross_component_providers_invisible_without_a_scope_edge() {
        let asm = build_application_semantic_model(&presolve_parser::parse_file(
            "src/components.tsx",
            r#"
@component("x-app-shell")
class AppShell extends Component {
  @context()
  theme!: Theme;
  render() { return <main />; }
}
@component("x-boundary")
class ThemeBoundary extends Component {
  @provide(AppShell.theme)
  providedTheme: Theme = this.localTheme;
  render() { return <main />; }
}
@component("x-toolbar")
class Toolbar extends Component {
  @consume(AppShell.theme)
  theme!: Theme;
  render() { return <main />; }
}
"#,
        ));
        let consumer = ConsumerId::for_component(&asm.components[2].id, "theme");

        assert!(matches!(
            asm.context_resolution(&consumer).unwrap().result,
            ContextResolutionResult::Unresolved
        ));
        assert_eq!(asm.unresolved_context_consumers(), vec![&consumer]);
    }

    #[test]
    fn future_parent_scope_edges_use_nearest_provider_without_source_inference() {
        let asm = build_application_semantic_model(&presolve_parser::parse_file(
            "src/components.tsx",
            r#"
@component("x-app-shell")
class AppShell extends Component {
  @context()
  theme!: Theme;
  @provide(AppShell.theme)
  rootTheme: Theme = this.rootTheme;
  render() { return <main />; }
}
@component("x-settings")
class SettingsArea extends Component {
  @provide(AppShell.theme)
  settingsTheme: Theme = this.settingsTheme;
  render() { return <main />; }
}
@component("x-toolbar")
class Toolbar extends Component {
  @consume(AppShell.theme)
  theme!: Theme;
  render() { return <main />; }
}
"#,
        ));
        let root = &asm.components[0].id;
        let settings = &asm.components[1].id;
        let toolbar = &asm.components[2].id;
        let scope = ComponentScopeGraph::with_parent_relations(
            &asm.components,
            BTreeMap::from([
                (toolbar.clone(), settings.clone()),
                (settings.clone(), root.clone()),
            ]),
        );
        let resolutions = collect_context_resolutions(
            &asm.consumers,
            &asm.contexts,
            &asm.providers,
            &asm.expression_graph,
            &scope,
        );
        let consumer = ConsumerId::for_component(toolbar, "theme");
        let expected = ProviderId::for_component(settings, "settingsTheme");

        assert!(matches!(
            resolutions[&consumer].result,
            ContextResolutionResult::Provider {
                ref provider,
                distance: 1,
                ..
            } if *provider == expected
        ));
    }

    #[test]
    fn retains_ambiguity_in_stable_provider_id_order() {
        let asm = build_application_semantic_model(&presolve_parser::parse_file(
            "src/toolbar.tsx",
            r#"
@component("x-app-shell")
class AppShell extends Component {
  @context()
  theme!: Theme;
  render() { return <main />; }
}
@component("x-toolbar")
class Toolbar extends Component {
  @provide(AppShell.theme)
  firstTheme: Theme = this.firstTheme;
  @consume(AppShell.theme)
  theme!: Theme;
  render() { return <main />; }
}
"#,
        ));
        let toolbar = &asm.components[1].id;
        let mut providers = asm.providers.clone();
        let mut duplicate = providers.values().next().unwrap().clone();
        duplicate.id = ProviderId::for_component(toolbar, "secondTheme");
        duplicate.name = "secondTheme".to_string();
        duplicate.authored_field = toolbar.provider_field("secondTheme");
        providers.insert(duplicate.id.clone(), duplicate);
        let resolutions = collect_context_resolutions(
            &asm.consumers,
            &asm.contexts,
            &providers,
            &asm.expression_graph,
            &asm.component_scope,
        );
        let consumer = ConsumerId::for_component(toolbar, "theme");

        assert!(matches!(
            resolutions[&consumer].result,
            ContextResolutionResult::Ambiguous {
                ref providers,
                distance: 0,
            } if providers.windows(2).all(|pair| pair[0] < pair[1])
        ));
    }

    #[test]
    fn excludes_invalid_context_references_from_resolution() {
        let asm = build_application_semantic_model(&presolve_parser::parse_file(
            "src/toolbar.tsx",
            r#"
@component("x-toolbar")
class Toolbar extends Component {
  @consume(AppShell.theme)
  theme!: Theme;
  render() { return <main />; }
}
"#,
        ));
        let consumer = ConsumerId::for_component(&asm.components[0].id, "theme");

        assert!(asm.context_resolution(&consumer).is_none());
        assert_eq!(
            asm.context_declaration_candidates()
                .invalid_candidates()
                .len(),
            1
        );
    }
}
