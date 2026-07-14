use serde::Serialize;

use crate::{
    ApplicationSemanticModel, ConsumerEntity, ContextEntity, SemanticEntity, SemanticId,
    SemanticOwner, SemanticReferenceKind, SourceProvenance, TemplateSemanticKind,
};

pub const SEMANTIC_GRAPH_SCHEMA_VERSION: u32 = 5;

/// A stable, backend-independent graph projection of the canonical ASM.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct SemanticGraph {
    pub schema_version: u32,
    pub roots: Vec<SemanticId>,
    pub nodes: Vec<SemanticGraphNode>,
    pub edges: Vec<SemanticGraphEdge>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct SemanticGraphNode {
    pub id: SemanticId,
    pub kind: SemanticGraphNodeKind,
    pub provenance: SemanticGraphProvenance,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub context: Option<SemanticGraphContext>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub provider: Option<SemanticGraphProvider>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub consumer: Option<SemanticGraphConsumer>,
}

/// Context-specific facts projected from the canonical G1 ASM entity.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct SemanticGraphContext {
    pub owner: SemanticId,
    pub authored_field: SemanticId,
    pub declared_type_id: String,
    pub execution_boundary: &'static str,
    pub has_default_expression: bool,
}

/// Provider-specific facts projected from the canonical G2 ASM entity.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct SemanticGraphProvider {
    pub owner: SemanticId,
    pub authored_field: SemanticId,
    pub context: SemanticId,
    pub declared_type_id: String,
    pub value_expression: SemanticId,
    pub execution_boundary: &'static str,
}

/// Consumer-specific facts projected from the canonical G3 ASM entity.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct SemanticGraphConsumer {
    pub owner: SemanticId,
    pub authored_field: SemanticId,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub context: Option<SemanticId>,
    pub requested_type_id: String,
    pub execution_boundary: &'static str,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum SemanticGraphNodeKind {
    Component,
    StateField,
    Method,
    Context,
    Provider,
    Consumer,
    Computed,
    Effect,
    Parameter,
    LocalVariable,
    Action,
    EventHandler,
    Template,
    TemplateElement,
    TemplateFragment,
    TemplateText,
    TemplateBinding,
    TemplateAttribute,
    TemplateAttributeBinding,
    TemplateEventAttribute,
    TemplateConditional,
    TemplateList,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct SemanticGraphProvenance {
    pub path: String,
    pub start: usize,
    pub end: usize,
    pub line: usize,
    pub column: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct SemanticGraphEdge {
    pub kind: SemanticGraphEdgeKind,
    pub source: SemanticId,
    pub target: SemanticId,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum SemanticGraphEdgeKind {
    Ownership,
    ActionState,
    ComputedState,
    ComputedComputed,
    EffectState,
    EffectComputed,
    ProvidesContext,
    ConsumesContext,
    ResolvesToProvider,
    EventMethod,
    TemplateState,
    TemplateComputed,
    TemplateLocal,
}

/// Build a deterministic graph export from canonical ASM semantics only.
///
/// # Panics
///
/// Panics if the ASM ownership map references a missing semantic entity or one
/// without source provenance, which violates the canonical ASM invariant.
#[must_use]
pub fn build_semantic_graph(asm: &ApplicationSemanticModel) -> SemanticGraph {
    let nodes = asm
        .ownership
        .keys()
        .filter(|id| {
            !matches!(
                asm.entity(id),
                Some(
                    SemanticEntity::Slot(_)
                        | SemanticEntity::ComponentInvocation(_)
                        | SemanticEntity::SlotContentFragment(_)
                        | SemanticEntity::SlotOutlet(_)
                )
            )
        })
        .map(|id| {
            let entity = asm
                .entity(id)
                .expect("ASM ownership should only contain semantic entities");
            let provenance = asm
                .provenance(id)
                .expect("ASM ownership should have source provenance");

            SemanticGraphNode {
                id: id.clone(),
                kind: semantic_graph_node_kind(entity),
                provenance: provenance.into(),
                context: semantic_graph_context(entity),
                provider: semantic_graph_provider(entity),
                consumer: semantic_graph_consumer(entity),
            }
        })
        .collect();
    let roots = asm.application_roots().into_iter().cloned().collect();
    let mut edges = asm
        .ownership
        .iter()
        .filter_map(|(target, owner)| {
            if matches!(
                asm.entity(target),
                Some(
                    SemanticEntity::Slot(_)
                        | SemanticEntity::ComponentInvocation(_)
                        | SemanticEntity::SlotContentFragment(_)
                        | SemanticEntity::SlotOutlet(_)
                )
            ) {
                return None;
            }
            match owner {
                SemanticOwner::Application => None,
                SemanticOwner::Entity(source) => Some(SemanticGraphEdge {
                    kind: SemanticGraphEdgeKind::Ownership,
                    source: source.clone(),
                    target: target.clone(),
                }),
            }
        })
        .chain(asm.references.iter().map(|reference| SemanticGraphEdge {
            kind: semantic_graph_edge_kind(reference.kind),
            source: reference.source.clone(),
            target: reference.target.clone(),
        }))
        .collect::<Vec<_>>();
    edges.sort_by(|left, right| {
        (
            left.kind.as_str(),
            left.source.as_str(),
            left.target.as_str(),
        )
            .cmp(&(
                right.kind.as_str(),
                right.source.as_str(),
                right.target.as_str(),
            ))
    });

    SemanticGraph {
        schema_version: SEMANTIC_GRAPH_SCHEMA_VERSION,
        roots,
        nodes,
        edges,
    }
}

/// Serialize a semantic graph as deterministic, pretty JSON.
///
/// # Panics
///
/// Panics if the compiler-owned graph cannot serialize to JSON.
#[must_use]
pub fn semantic_graph_json(graph: &SemanticGraph) -> String {
    serde_json::to_string_pretty(graph).expect("semantic graph should serialize") + "\n"
}

impl From<&SourceProvenance> for SemanticGraphProvenance {
    fn from(provenance: &SourceProvenance) -> Self {
        Self {
            path: provenance.path.display().to_string(),
            start: provenance.span.start,
            end: provenance.span.end,
            line: provenance.span.line,
            column: provenance.span.column,
        }
    }
}

impl SemanticGraphEdgeKind {
    const fn as_str(self) -> &'static str {
        match self {
            Self::Ownership => "ownership",
            Self::ActionState => "action-state",
            Self::ComputedState => "computed-state",
            Self::ComputedComputed => "computed-computed",
            Self::EffectState => "effect-state",
            Self::EffectComputed => "effect-computed",
            Self::ProvidesContext => "provides-context",
            Self::ConsumesContext => "consumes-context",
            Self::ResolvesToProvider => "resolves-to-provider",
            Self::EventMethod => "event-method",
            Self::TemplateState => "template-state",
            Self::TemplateComputed => "template-computed",
            Self::TemplateLocal => "template-local",
        }
    }
}

fn semantic_graph_node_kind(entity: SemanticEntity<'_>) -> SemanticGraphNodeKind {
    match entity {
        SemanticEntity::Component(_) => SemanticGraphNodeKind::Component,
        SemanticEntity::StateField(_) => SemanticGraphNodeKind::StateField,
        SemanticEntity::Method(_) => SemanticGraphNodeKind::Method,
        SemanticEntity::Context(_) => SemanticGraphNodeKind::Context,
        SemanticEntity::Provider(_) => SemanticGraphNodeKind::Provider,
        SemanticEntity::Consumer(_) => SemanticGraphNodeKind::Consumer,
        SemanticEntity::Slot(_) => {
            unreachable!("Slots are not projected into frozen semantic graph schema v5")
        }
        SemanticEntity::ComponentInvocation(_) => {
            unreachable!("Component invocations are not projected into semantic graph schema v5")
        }
        SemanticEntity::SlotContentFragment(_) => {
            unreachable!("Slot fragments are not projected into semantic graph schema v5")
        }
        SemanticEntity::SlotOutlet(_) => {
            unreachable!("Slot outlets are not projected into semantic graph schema v5")
        }
        SemanticEntity::Computed(_) => SemanticGraphNodeKind::Computed,
        SemanticEntity::Effect(_) => SemanticGraphNodeKind::Effect,
        SemanticEntity::Parameter(_) => SemanticGraphNodeKind::Parameter,
        SemanticEntity::LocalVariable(_) => SemanticGraphNodeKind::LocalVariable,
        SemanticEntity::Action(_) => SemanticGraphNodeKind::Action,
        SemanticEntity::EventHandler(_) => SemanticGraphNodeKind::EventHandler,
        SemanticEntity::Template(_) => SemanticGraphNodeKind::Template,
        SemanticEntity::TemplateEntity(entity) => match entity.kind {
            TemplateSemanticKind::Element => SemanticGraphNodeKind::TemplateElement,
            TemplateSemanticKind::Fragment => SemanticGraphNodeKind::TemplateFragment,
            TemplateSemanticKind::Text => SemanticGraphNodeKind::TemplateText,
            TemplateSemanticKind::Binding => SemanticGraphNodeKind::TemplateBinding,
            TemplateSemanticKind::Attribute => SemanticGraphNodeKind::TemplateAttribute,
            TemplateSemanticKind::AttributeBinding => {
                SemanticGraphNodeKind::TemplateAttributeBinding
            }
            TemplateSemanticKind::EventAttribute => SemanticGraphNodeKind::TemplateEventAttribute,
            TemplateSemanticKind::Conditional => SemanticGraphNodeKind::TemplateConditional,
            TemplateSemanticKind::List => SemanticGraphNodeKind::TemplateList,
        },
    }
}

fn semantic_graph_context(entity: SemanticEntity<'_>) -> Option<SemanticGraphContext> {
    let SemanticEntity::Context(context) = entity else {
        return None;
    };
    Some(context_metadata(context))
}

fn semantic_graph_provider(entity: SemanticEntity<'_>) -> Option<SemanticGraphProvider> {
    let SemanticEntity::Provider(provider) = entity else {
        return None;
    };
    Some(SemanticGraphProvider {
        owner: provider
            .owner
            .entity_id()
            .expect("provider entities should be component-owned")
            .clone(),
        authored_field: provider.authored_field.clone(),
        context: provider.context.as_semantic_id().clone(),
        declared_type_id: provider.declared_type_id.to_string(),
        value_expression: provider.value_expression.clone(),
        execution_boundary: match provider.execution_boundary {
            crate::ExecutionBoundary::Client => "client",
            crate::ExecutionBoundary::Server => "server",
        },
    })
}

fn semantic_graph_consumer(entity: SemanticEntity<'_>) -> Option<SemanticGraphConsumer> {
    let SemanticEntity::Consumer(consumer) = entity else {
        return None;
    };
    Some(consumer_metadata(consumer))
}

fn consumer_metadata(consumer: &ConsumerEntity) -> SemanticGraphConsumer {
    SemanticGraphConsumer {
        owner: consumer
            .owner
            .entity_id()
            .expect("consumer entities should be component-owned")
            .clone(),
        authored_field: consumer.authored_field.clone(),
        context: consumer
            .context()
            .map(|context| context.as_semantic_id().clone()),
        requested_type_id: consumer.requested_type_id.to_string(),
        execution_boundary: match consumer.execution_boundary {
            crate::ExecutionBoundary::Client => "client",
            crate::ExecutionBoundary::Server => "server",
        },
    }
}

fn context_metadata(context: &ContextEntity) -> SemanticGraphContext {
    SemanticGraphContext {
        owner: context
            .owner
            .entity_id()
            .expect("context entities should be component-owned")
            .clone(),
        authored_field: context.authored_field.clone(),
        declared_type_id: context.declared_type_id.to_string(),
        execution_boundary: match context.execution_boundary {
            crate::ExecutionBoundary::Client => "client",
            crate::ExecutionBoundary::Server => "server",
        },
        has_default_expression: context.default_expression.is_some(),
    }
}

fn semantic_graph_edge_kind(kind: SemanticReferenceKind) -> SemanticGraphEdgeKind {
    match kind {
        SemanticReferenceKind::ActionState => SemanticGraphEdgeKind::ActionState,
        SemanticReferenceKind::ComputedState => SemanticGraphEdgeKind::ComputedState,
        SemanticReferenceKind::ComputedComputed => SemanticGraphEdgeKind::ComputedComputed,
        SemanticReferenceKind::EffectState => SemanticGraphEdgeKind::EffectState,
        SemanticReferenceKind::EffectComputed => SemanticGraphEdgeKind::EffectComputed,
        SemanticReferenceKind::ProvidesContext => SemanticGraphEdgeKind::ProvidesContext,
        SemanticReferenceKind::ConsumesContext => SemanticGraphEdgeKind::ConsumesContext,
        SemanticReferenceKind::ResolvesToProvider => SemanticGraphEdgeKind::ResolvesToProvider,
        SemanticReferenceKind::EventMethod => SemanticGraphEdgeKind::EventMethod,
        SemanticReferenceKind::TemplateState => SemanticGraphEdgeKind::TemplateState,
        SemanticReferenceKind::TemplateComputed => SemanticGraphEdgeKind::TemplateComputed,
        SemanticReferenceKind::TemplateLocal => SemanticGraphEdgeKind::TemplateLocal,
    }
}

#[cfg(test)]
mod tests {
    use super::{build_semantic_graph, semantic_graph_json, SemanticGraphEdgeKind};
    use crate::build_application_semantic_model;

    #[test]
    fn exports_context_nodes_with_compiler_owned_metadata() {
        let parsed = ezc_parser::parse_file(
            "src/AppShell.tsx",
            r#"
@component("x-app-shell")
class AppShell extends Component {
  @context()
  locale: string = "en";
  render() { return <main />; }
}
"#,
        );
        let asm = build_application_semantic_model(&parsed);
        let graph = build_semantic_graph(&asm);
        let context_id = asm.contexts()[0].id.as_semantic_id();
        let node = graph
            .nodes
            .iter()
            .find(|node| node.id == *context_id)
            .unwrap();

        assert_eq!(node.kind, super::SemanticGraphNodeKind::Context);
        assert_eq!(node.context.as_ref().unwrap().owner, asm.components[0].id);
        assert!(node.context.as_ref().unwrap().has_default_expression);
        assert!(graph.edges.iter().any(|edge| {
            edge.kind == SemanticGraphEdgeKind::Ownership
                && edge.source == asm.components[0].id
                && edge.target == *context_id
        }));
    }

    #[test]
    fn exports_provider_nodes_and_compiler_resolved_context_edges() {
        let parsed = ezc_parser::parse_file(
            "src/AppShell.tsx",
            r#"
@component("x-app-shell")
class AppShell extends Component {
  @context()
  theme: string;
  @provide(AppShell.theme)
  providedTheme: string = this.theme;
  render() { return <main />; }
}
"#,
        );
        let asm = build_application_semantic_model(&parsed);
        let graph = build_semantic_graph(&asm);
        let provider = asm.providers()[0];
        let node = graph
            .nodes
            .iter()
            .find(|node| node.id == *provider.id.as_semantic_id())
            .unwrap();

        assert_eq!(node.kind, super::SemanticGraphNodeKind::Provider);
        assert_eq!(
            node.provider.as_ref().unwrap().context,
            *provider.context.as_semantic_id()
        );
        assert!(graph.edges.iter().any(|edge| {
            edge.kind == SemanticGraphEdgeKind::ProvidesContext
                && edge.source == *provider.id.as_semantic_id()
                && edge.target == *provider.context.as_semantic_id()
        }));
    }

    #[test]
    fn exports_consumer_nodes_and_compiler_resolved_context_edges() {
        let parsed = ezc_parser::parse_file(
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
  @consume(AppShell.theme)
  theme!: Theme;
  render() { return <main />; }
}
"#,
        );
        let asm = build_application_semantic_model(&parsed);
        let graph = build_semantic_graph(&asm);
        let consumer = asm.consumers()[0];
        let node = graph
            .nodes
            .iter()
            .find(|node| node.id == *consumer.id.as_semantic_id())
            .unwrap();

        assert_eq!(node.kind, super::SemanticGraphNodeKind::Consumer);
        assert_eq!(
            node.consumer.as_ref().unwrap().context,
            consumer
                .context()
                .map(|context| context.as_semantic_id().clone())
        );
        assert!(graph.edges.iter().any(|edge| {
            edge.kind == SemanticGraphEdgeKind::ConsumesContext
                && edge.source == *consumer.id.as_semantic_id()
                && edge.target == *consumer.context().unwrap().as_semantic_id()
        }));
    }

    #[test]
    fn exports_compiler_resolved_consumer_provider_edges() {
        let parsed = ezc_parser::parse_file(
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
        );
        let asm = build_application_semantic_model(&parsed);
        let graph = build_semantic_graph(&asm);
        let consumer = asm.consumers()[0];
        let provider = asm.resolved_provider(&consumer.id).unwrap();

        assert!(graph.edges.iter().any(|edge| {
            edge.kind == SemanticGraphEdgeKind::ResolvesToProvider
                && edge.source == *consumer.id.as_semantic_id()
                && edge.target == *provider.as_semantic_id()
        }));
    }

    #[test]
    fn exports_a_deterministic_canonical_semantic_graph() {
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
        let graph = build_semantic_graph(&asm);
        let component = &asm.components[0];

        assert_eq!(graph.schema_version, 5);
        assert_eq!(graph.roots, vec![component.id.clone()]);
        assert_eq!(graph.nodes.len(), asm.ownership.len());
        assert!(graph
            .nodes
            .windows(2)
            .all(|nodes| nodes[0].id <= nodes[1].id));
        assert!(graph.edges.windows(2).all(|edges| {
            let left = &edges[0];
            let right = &edges[1];
            (
                left.kind.as_str(),
                left.source.as_str(),
                left.target.as_str(),
            ) <= (
                right.kind.as_str(),
                right.source.as_str(),
                right.target.as_str(),
            )
        }));
        assert!(graph.edges.iter().any(|edge| {
            edge.kind == SemanticGraphEdgeKind::Ownership
                && edge.source == component.id
                && edge.target == component.methods[0].id
        }));
        assert!(graph.edges.iter().any(|edge| {
            edge.kind == SemanticGraphEdgeKind::ActionState
                && edge.source == component.actions[0].id
                && edge.target == component.state_fields[0].id
        }));

        let first = semantic_graph_json(&graph);
        let second = semantic_graph_json(&build_semantic_graph(&asm));
        assert_eq!(first, second);
        let document: serde_json::Value =
            serde_json::from_str(&first).expect("semantic graph JSON should parse");
        assert_eq!(document["schema_version"], 5);
        assert_eq!(document["nodes"][0]["kind"], "component");
    }

    #[test]
    fn exports_first_class_computed_nodes() {
        let parsed = ezc_parser::parse_file(
            "src/Computed.tsx",
            r#"
@component("x-computed")
class Computed extends Component {
  count = state(1);

  @computed()
  get remainingCount(): number { return this.count; }

  render() { return <p />; }
}
"#,
        );
        let asm = build_application_semantic_model(&parsed);
        let computed_id = asm.components[0].id.computed("remainingCount");
        let graph = build_semantic_graph(&asm);

        assert!(graph.nodes.iter().any(|node| {
            node.id == computed_id && node.kind == super::SemanticGraphNodeKind::Computed
        }));
        assert!(graph.edges.iter().any(|edge| {
            edge.kind == SemanticGraphEdgeKind::Ownership
                && edge.source == asm.components[0].id
                && edge.target == computed_id
        }));
        assert!(graph.edges.iter().any(|edge| {
            edge.kind == SemanticGraphEdgeKind::ComputedState
                && edge.source == computed_id
                && edge.target == asm.components[0].id.state_field("count")
        }));
    }
}
