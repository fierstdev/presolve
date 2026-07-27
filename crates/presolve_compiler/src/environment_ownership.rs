//! Immutable environment and lifetime ownership analysis.

use std::collections::{BTreeMap, BTreeSet, VecDeque};

use serde::Serialize;

use crate::{ControlFlowProvenanceV1, SemanticId};

pub const ENVIRONMENT_OWNERSHIP_SCHEMA_VERSION: u32 = 1;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum EnvironmentClassV1 {
    CompileTime,
    Server,
    Browser,
    SharedSerializable,
    OpaqueExternal,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum LifetimeClassV1 {
    Application,
    Route,
    Request,
    ComponentInstance,
    ActionInvocation,
    EffectExecution,
    ResourceLoad,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct EnvironmentOwnershipNodeV1 {
    pub id: SemanticId,
    pub environment: EnvironmentClassV1,
    pub lifetime: LifetimeClassV1,
    pub provenance: ControlFlowProvenanceV1,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum EnvironmentOwnershipEdgeKindV1 {
    Owns,
    References,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct EnvironmentOwnershipEdgeV1 {
    pub source: SemanticId,
    pub target: SemanticId,
    pub kind: EnvironmentOwnershipEdgeKindV1,
    pub provenance: ControlFlowProvenanceV1,
}

/// Facts supplied by the canonical lowering that owns environment assignment.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct EnvironmentOwnershipFactsV1 {
    pub nodes: Vec<EnvironmentOwnershipNodeV1>,
    pub edges: Vec<EnvironmentOwnershipEdgeV1>,
    pub browser_artifact_roots: Vec<SemanticId>,
    pub shared_publication_roots: Vec<SemanticId>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum EnvironmentOwnershipViolationV1 {
    OwnershipCycle,
    BrowserReferencesServer,
    SharedPublicationReferencesRequest,
}

/// Every violation carries the complete deterministic route from its root.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct EnvironmentOwnershipDiagnosticV1 {
    pub violation: EnvironmentOwnershipViolationV1,
    pub path: Vec<SemanticId>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct EnvironmentOwnershipGraphV1 {
    pub schema_version: u32,
    pub nodes: Vec<EnvironmentOwnershipNodeV1>,
    pub edges: Vec<EnvironmentOwnershipEdgeV1>,
    pub diagnostics: Vec<EnvironmentOwnershipDiagnosticV1>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EnvironmentOwnershipErrorV1 {
    DuplicateNode(SemanticId),
    UnknownEndpoint(SemanticId),
    UnknownRoot(SemanticId),
}

impl std::fmt::Display for EnvironmentOwnershipErrorV1 {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::DuplicateNode(id) => write!(formatter, "duplicate environment node {id}"),
            Self::UnknownEndpoint(id) => {
                write!(formatter, "environment edge references unknown node {id}")
            }
            Self::UnknownRoot(id) => {
                write!(formatter, "environment root references unknown node {id}")
            }
        }
    }
}

impl std::error::Error for EnvironmentOwnershipErrorV1 {}

/// Validates ownership topology and finds publication leaks using supplied facts.
pub fn build_environment_ownership_graph_v1(
    facts: &EnvironmentOwnershipFactsV1,
) -> Result<EnvironmentOwnershipGraphV1, EnvironmentOwnershipErrorV1> {
    let mut nodes = BTreeMap::new();
    for node in &facts.nodes {
        if nodes.insert(node.id.clone(), node.clone()).is_some() {
            return Err(EnvironmentOwnershipErrorV1::DuplicateNode(node.id.clone()));
        }
    }
    for edge in &facts.edges {
        for endpoint in [&edge.source, &edge.target] {
            if !nodes.contains_key(endpoint) {
                return Err(EnvironmentOwnershipErrorV1::UnknownEndpoint(
                    endpoint.clone(),
                ));
            }
        }
    }
    for root in facts
        .browser_artifact_roots
        .iter()
        .chain(&facts.shared_publication_roots)
    {
        if !nodes.contains_key(root) {
            return Err(EnvironmentOwnershipErrorV1::UnknownRoot(root.clone()));
        }
    }

    let mut edges = facts.edges.clone();
    edges.sort_by(|left, right| {
        (&left.source, left.kind, &left.target).cmp(&(&right.source, right.kind, &right.target))
    });
    edges.dedup_by(|left, right| {
        left.source == right.source && left.target == right.target && left.kind == right.kind
    });
    let all_adjacency = adjacency(&edges, None);
    let ownership_adjacency = adjacency(&edges, Some(EnvironmentOwnershipEdgeKindV1::Owns));
    let mut diagnostics = Vec::new();

    for root in &facts.browser_artifact_roots {
        if let Some(path) = path_to(root, &all_adjacency, |id| {
            nodes[id].environment == EnvironmentClassV1::Server
        }) {
            diagnostics.push(EnvironmentOwnershipDiagnosticV1 {
                violation: EnvironmentOwnershipViolationV1::BrowserReferencesServer,
                path,
            });
        }
    }
    for root in &facts.shared_publication_roots {
        if let Some(path) = path_to(root, &all_adjacency, |id| {
            nodes[id].lifetime == LifetimeClassV1::Request
        }) {
            diagnostics.push(EnvironmentOwnershipDiagnosticV1 {
                violation: EnvironmentOwnershipViolationV1::SharedPublicationReferencesRequest,
                path,
            });
        }
    }
    for source in nodes.keys() {
        if let Some(path) = path_to(source, &ownership_adjacency, |target| target == source) {
            if path.len() > 1 {
                let cycle_nodes = &path[..path.len() - 1];
                if cycle_nodes.iter().min() == Some(source) {
                    diagnostics.push(EnvironmentOwnershipDiagnosticV1 {
                        violation: EnvironmentOwnershipViolationV1::OwnershipCycle,
                        path,
                    });
                }
            }
        }
    }
    diagnostics
        .sort_by(|left, right| (left.violation, &left.path).cmp(&(right.violation, &right.path)));
    diagnostics.dedup();
    Ok(EnvironmentOwnershipGraphV1 {
        schema_version: ENVIRONMENT_OWNERSHIP_SCHEMA_VERSION,
        nodes: nodes.into_values().collect(),
        edges,
        diagnostics,
    })
}

fn adjacency(
    edges: &[EnvironmentOwnershipEdgeV1],
    kind: Option<EnvironmentOwnershipEdgeKindV1>,
) -> BTreeMap<SemanticId, Vec<SemanticId>> {
    let mut result = BTreeMap::<SemanticId, Vec<SemanticId>>::new();
    for edge in edges
        .iter()
        .filter(|edge| kind.is_none_or(|kind| edge.kind == kind))
    {
        result
            .entry(edge.source.clone())
            .or_default()
            .push(edge.target.clone());
    }
    for targets in result.values_mut() {
        targets.sort();
        targets.dedup();
    }
    result
}

fn path_to(
    root: &SemanticId,
    adjacency: &BTreeMap<SemanticId, Vec<SemanticId>>,
    matches: impl Fn(&SemanticId) -> bool,
) -> Option<Vec<SemanticId>> {
    let mut queue = VecDeque::from([(root.clone(), vec![root.clone()])]);
    let mut visited = BTreeSet::new();
    while let Some((current, path)) = queue.pop_front() {
        if path.len() > 1 && matches(&current) {
            return Some(path);
        }
        if !visited.insert(current.clone()) {
            continue;
        }
        for target in adjacency.get(&current).into_iter().flatten() {
            let mut next_path = path.clone();
            next_path.push(target.clone());
            queue.push_back((target.clone(), next_path));
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    fn provenance() -> ControlFlowProvenanceV1 {
        ControlFlowProvenanceV1 {
            path: "src/App.tsx".into(),
            start: 0,
            end: 1,
            line: 1,
            column: 1,
        }
    }

    fn id(value: &str) -> SemanticId {
        SemanticId::component(None, value)
    }

    fn node(
        value: &str,
        environment: EnvironmentClassV1,
        lifetime: LifetimeClassV1,
    ) -> EnvironmentOwnershipNodeV1 {
        EnvironmentOwnershipNodeV1 {
            id: id(value),
            environment,
            lifetime,
            provenance: provenance(),
        }
    }

    #[test]
    fn reports_browser_and_request_leaks_with_reference_paths() {
        let graph = build_environment_ownership_graph_v1(&EnvironmentOwnershipFactsV1 {
            nodes: vec![
                node(
                    "browser",
                    EnvironmentClassV1::Browser,
                    LifetimeClassV1::Application,
                ),
                node(
                    "server",
                    EnvironmentClassV1::Server,
                    LifetimeClassV1::Application,
                ),
                node(
                    "publication",
                    EnvironmentClassV1::SharedSerializable,
                    LifetimeClassV1::Application,
                ),
                node(
                    "request",
                    EnvironmentClassV1::Server,
                    LifetimeClassV1::Request,
                ),
            ],
            edges: vec![
                EnvironmentOwnershipEdgeV1 {
                    source: id("browser"),
                    target: id("server"),
                    kind: EnvironmentOwnershipEdgeKindV1::References,
                    provenance: provenance(),
                },
                EnvironmentOwnershipEdgeV1 {
                    source: id("publication"),
                    target: id("request"),
                    kind: EnvironmentOwnershipEdgeKindV1::References,
                    provenance: provenance(),
                },
            ],
            browser_artifact_roots: vec![id("browser")],
            shared_publication_roots: vec![id("publication")],
        })
        .expect("valid ownership facts");
        assert_eq!(graph.schema_version, ENVIRONMENT_OWNERSHIP_SCHEMA_VERSION);
        assert_eq!(graph.diagnostics.len(), 2);
        assert!(graph
            .diagnostics
            .iter()
            .all(|diagnostic| diagnostic.path.len() == 2));
    }

    #[test]
    fn reports_ownership_cycles_without_reclassifying_existing_products() {
        let graph = build_environment_ownership_graph_v1(&EnvironmentOwnershipFactsV1 {
            nodes: vec![
                node(
                    "first",
                    EnvironmentClassV1::Browser,
                    LifetimeClassV1::Application,
                ),
                node(
                    "second",
                    EnvironmentClassV1::Browser,
                    LifetimeClassV1::ComponentInstance,
                ),
            ],
            edges: vec![
                EnvironmentOwnershipEdgeV1 {
                    source: id("first"),
                    target: id("second"),
                    kind: EnvironmentOwnershipEdgeKindV1::Owns,
                    provenance: provenance(),
                },
                EnvironmentOwnershipEdgeV1 {
                    source: id("second"),
                    target: id("first"),
                    kind: EnvironmentOwnershipEdgeKindV1::Owns,
                    provenance: provenance(),
                },
            ],
            browser_artifact_roots: Vec::new(),
            shared_publication_roots: Vec::new(),
        })
        .expect("valid ownership facts");
        assert!(graph.diagnostics.iter().any(|diagnostic| {
            diagnostic.violation == EnvironmentOwnershipViolationV1::OwnershipCycle
        }));
    }
}
