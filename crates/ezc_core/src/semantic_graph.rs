use serde::Serialize;

use crate::{
    ApplicationSemanticModel, SemanticEntity, SemanticId, SemanticOwner, SemanticReferenceKind,
    SourceProvenance, TemplateSemanticKind,
};

pub const SEMANTIC_GRAPH_SCHEMA_VERSION: u32 = 1;

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
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum SemanticGraphNodeKind {
    Component,
    StateField,
    Method,
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
            }
        })
        .collect();
    let roots = asm.application_roots().into_iter().cloned().collect();
    let mut edges = asm
        .ownership
        .iter()
        .filter_map(|(target, owner)| match owner {
            SemanticOwner::Application => None,
            SemanticOwner::Entity(source) => Some(SemanticGraphEdge {
                kind: SemanticGraphEdgeKind::Ownership,
                source: source.clone(),
                target: target.clone(),
            }),
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

fn semantic_graph_edge_kind(kind: SemanticReferenceKind) -> SemanticGraphEdgeKind {
    match kind {
        SemanticReferenceKind::ActionState => SemanticGraphEdgeKind::ActionState,
        SemanticReferenceKind::ComputedState => SemanticGraphEdgeKind::ComputedState,
        SemanticReferenceKind::ComputedComputed => SemanticGraphEdgeKind::ComputedComputed,
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

        assert_eq!(graph.schema_version, 1);
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
        assert_eq!(document["schema_version"], 1);
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
