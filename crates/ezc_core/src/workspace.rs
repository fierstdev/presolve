//! L7 caller-owned workspace orchestration products.
#![allow(
    clippy::missing_errors_doc,
    clippy::missing_panics_doc,
    clippy::too_many_lines
)]

use sha2::{Digest as _, Sha256};
use std::collections::{BTreeMap, BTreeSet};

pub const WORKSPACE_MANIFEST_V1_SCHEMA: &str = "presolve.workspace-manifest";
pub const MAX_WORKSPACE_PACKAGES: usize = 256;
pub const MAX_WORKSPACE_EDGES: usize = 4096;
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorkspacePackageDescriptorV1 {
    pub package_id: String,
    pub session_id: String,
    pub display_name: Option<String>,
    pub configuration_identity_hint: Option<String>,
    pub metadata: BTreeMap<String, String>,
}
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct WorkspaceDependencyEdgeV1 {
    pub dependency_package_id: String,
    pub dependent_package_id: String,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorkspacePolicyV1 {
    pub failure_mode: String,
    pub execution_mode: String,
    pub result_detail: String,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorkspaceManifestV1 {
    pub schema: String,
    pub version: u32,
    pub workspace_id: String,
    pub packages: Vec<WorkspacePackageDescriptorV1>,
    pub dependencies: Vec<WorkspaceDependencyEdgeV1>,
    pub policy: WorkspacePolicyV1,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorkspacePackageGraphV1 {
    pub workspace_id: String,
    pub manifest_identity: String,
    pub packages: Vec<WorkspacePackageDescriptorV1>,
    pub edges: Vec<WorkspaceDependencyEdgeV1>,
    pub reverse_edges: Vec<WorkspaceDependencyEdgeV1>,
    pub roots: Vec<String>,
    pub leaves: Vec<String>,
    pub topological_order: Vec<String>,
    pub graph_identity: String,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorkspaceBuildStageV1 {
    pub stage_index: u32,
    pub packages: Vec<String>,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorkspaceBuildPlanV1 {
    pub manifest_identity: String,
    pub graph_identity: String,
    pub stages: Vec<WorkspaceBuildStageV1>,
    pub request_fingerprints: Vec<(String, String)>,
    pub plan_identity: String,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WorkspaceErrorV1 {
    Cycle { members: Vec<String> },
    Code(&'static str),
}
impl WorkspaceErrorV1 {
    #[must_use]
    pub const fn code(&self) -> &'static str {
        match self {
            Self::Cycle { .. } => "L7W001_WORKSPACE_DEPENDENCY_CYCLE",
            Self::Code(code) => code,
        }
    }
}
fn norm(value: &str) -> Result<String, WorkspaceErrorV1> {
    let value = value.trim();
    if value.is_empty() || value.bytes().any(|b| b.is_ascii_control()) {
        Err(WorkspaceErrorV1::Code(
            "L7W012_INVALID_WORKSPACE_IDENTIFIER",
        ))
    } else {
        Ok(value.into())
    }
}
fn digest(value: impl AsRef<[u8]>) -> String {
    format!("sha256:{:x}", Sha256::digest(value))
}
impl WorkspaceManifestV1 {
    pub fn normalize_validate(&self) -> Result<Self, WorkspaceErrorV1> {
        if self.schema != WORKSPACE_MANIFEST_V1_SCHEMA || self.version != 1 {
            return Err(WorkspaceErrorV1::Code(
                "L7W009_UNSUPPORTED_WORKSPACE_SCHEMA",
            ));
        }
        if self.packages.len() > MAX_WORKSPACE_PACKAGES
            || self.dependencies.len() > MAX_WORKSPACE_EDGES
        {
            return Err(WorkspaceErrorV1::Code(
                "L7W012_INVALID_WORKSPACE_IDENTIFIER",
            ));
        }
        if self.policy.failure_mode != "fail_fast"
            || self.policy.execution_mode != "deterministic_serial"
            || !matches!(self.policy.result_detail.as_str(), "summary" | "full")
        {
            return Err(WorkspaceErrorV1::Code("L7W010_INVALID_WORKSPACE_POLICY"));
        }
        let mut out = self.clone();
        out.workspace_id = norm(&out.workspace_id)?;
        for p in &mut out.packages {
            p.package_id = norm(&p.package_id)?;
            p.session_id = norm(&p.session_id)?;
            if !p.metadata.is_empty() {
                return Err(WorkspaceErrorV1::Code("L7W010_INVALID_WORKSPACE_POLICY"));
            }
        }
        out.packages.sort_by(|a, b| a.package_id.cmp(&b.package_id));
        if out
            .packages
            .windows(2)
            .any(|p| p[0].package_id == p[1].package_id)
        {
            return Err(WorkspaceErrorV1::Code("L7W002_DUPLICATE_PACKAGE_ID"));
        }
        let sessions = out
            .packages
            .iter()
            .map(|p| p.session_id.clone())
            .collect::<BTreeSet<_>>();
        if sessions.len() != out.packages.len() {
            return Err(WorkspaceErrorV1::Code("L7W003_DUPLICATE_SESSION_ID"));
        }
        let ids = out
            .packages
            .iter()
            .map(|p| p.package_id.clone())
            .collect::<BTreeSet<_>>();
        for e in &mut out.dependencies {
            e.dependency_package_id = norm(&e.dependency_package_id)?;
            e.dependent_package_id = norm(&e.dependent_package_id)?;
            if e.dependency_package_id == e.dependent_package_id {
                return Err(WorkspaceErrorV1::Code("L7W005_SELF_DEPENDENCY"));
            }
            if !ids.contains(&e.dependency_package_id) || !ids.contains(&e.dependent_package_id) {
                return Err(WorkspaceErrorV1::Code("L7W004_UNKNOWN_EDGE_PACKAGE"));
            }
        }
        out.dependencies.sort();
        if out.dependencies.windows(2).any(|e| e[0] == e[1]) {
            return Err(WorkspaceErrorV1::Code("L7W006_DUPLICATE_DEPENDENCY_EDGE"));
        }
        Ok(out)
    }
    #[must_use]
    pub fn identity(&self) -> String {
        digest(self.canonical_json())
    }
    #[must_use]
    pub fn canonical_json(&self) -> Vec<u8> {
        let q = |s: &str| serde_json::to_string(s).expect("strings serialize");
        let p = self
            .packages
            .iter()
            .map(|x| format!("{}:{}", x.package_id, x.session_id))
            .collect::<Vec<_>>()
            .join(",");
        let e = self
            .dependencies
            .iter()
            .map(|x| format!("{}>{}", x.dependency_package_id, x.dependent_package_id))
            .collect::<Vec<_>>()
            .join(",");
        format!("{{\"schema\":{},\"version\":1,\"workspace_id\":{},\"packages\":{},\"dependencies\":{},\"policy\":{}/{}/{}}}\n",q(&self.schema),q(&self.workspace_id),q(&p),q(&e),self.policy.failure_mode,self.policy.execution_mode,self.policy.result_detail).into_bytes()
    }
}
pub fn graph(manifest: &WorkspaceManifestV1) -> Result<WorkspacePackageGraphV1, WorkspaceErrorV1> {
    let m = manifest.normalize_validate()?;
    let mut indegree = m
        .packages
        .iter()
        .map(|p| (p.package_id.clone(), 0usize))
        .collect::<BTreeMap<_, _>>();
    let mut forward = BTreeMap::<String, Vec<String>>::new();
    for e in &m.dependencies {
        *indegree.get_mut(&e.dependent_package_id).expect("valid") += 1;
        forward
            .entry(e.dependency_package_id.clone())
            .or_default()
            .push(e.dependent_package_id.clone());
    }
    for v in forward.values_mut() {
        v.sort();
    }
    let mut remaining = indegree.clone();
    let mut order = Vec::new();
    while !remaining.is_empty() {
        let now = remaining
            .iter()
            .filter(|(_, n)| **n == 0)
            .map(|(k, _)| k.clone())
            .collect::<Vec<_>>();
        if now.is_empty() {
            return Err(WorkspaceErrorV1::Cycle {
                members: remaining.into_keys().collect(),
            });
        }
        for id in now {
            remaining.remove(&id);
            if let Some(next) = forward.get(&id) {
                for n in next {
                    if let Some(v) = remaining.get_mut(n) {
                        *v -= 1;
                    }
                }
            }
            order.push(id);
        }
    }
    let roots = m
        .packages
        .iter()
        .filter(|p| {
            !m.dependencies
                .iter()
                .any(|e| e.dependent_package_id == p.package_id)
        })
        .map(|p| p.package_id.clone())
        .collect();
    let leaves = m
        .packages
        .iter()
        .filter(|p| {
            !m.dependencies
                .iter()
                .any(|e| e.dependency_package_id == p.package_id)
        })
        .map(|p| p.package_id.clone())
        .collect();
    let mut reverse = m
        .dependencies
        .iter()
        .map(|e| WorkspaceDependencyEdgeV1 {
            dependency_package_id: e.dependent_package_id.clone(),
            dependent_package_id: e.dependency_package_id.clone(),
        })
        .collect::<Vec<_>>();
    reverse.sort();
    let manifest_identity = m.identity();
    let id = digest(format!("{}|{:?}", manifest_identity, m.dependencies));
    Ok(WorkspacePackageGraphV1 {
        workspace_id: m.workspace_id,
        manifest_identity,
        packages: m.packages,
        edges: m.dependencies,
        reverse_edges: reverse,
        roots,
        leaves,
        topological_order: order,
        graph_identity: id,
    })
}
#[must_use]
pub fn plan(
    g: &WorkspacePackageGraphV1,
    mut requests: Vec<(String, String)>,
) -> WorkspaceBuildPlanV1 {
    requests.sort();
    let mut indegree = g
        .packages
        .iter()
        .map(|p| (p.package_id.clone(), 0usize))
        .collect::<BTreeMap<_, _>>();
    let mut forward = BTreeMap::<String, Vec<String>>::new();
    for e in &g.edges {
        *indegree.get_mut(&e.dependent_package_id).expect("graph") += 1;
        forward
            .entry(e.dependency_package_id.clone())
            .or_default()
            .push(e.dependent_package_id.clone());
    }
    for v in forward.values_mut() {
        v.sort();
    }
    let mut stages = Vec::new();
    let mut index = 0;
    while !indegree.is_empty() {
        let now = indegree
            .iter()
            .filter(|(_, n)| **n == 0)
            .map(|(k, _)| k.clone())
            .collect::<Vec<_>>();
        for id in &now {
            indegree.remove(id);
            if let Some(next) = forward.get(id) {
                for n in next {
                    if let Some(v) = indegree.get_mut(n) {
                        *v -= 1;
                    }
                }
            }
        }
        stages.push(WorkspaceBuildStageV1 {
            stage_index: index,
            packages: now,
        });
        index += 1;
    }
    let id = digest(format!(
        "{}|{}|{:?}|{:?}",
        g.manifest_identity, g.graph_identity, stages, requests
    ));
    WorkspaceBuildPlanV1 {
        manifest_identity: g.manifest_identity.clone(),
        graph_identity: g.graph_identity.clone(),
        stages,
        request_fingerprints: requests,
        plan_identity: id,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn manifest(packages: Vec<&str>, edges: Vec<(&str, &str)>) -> WorkspaceManifestV1 {
        WorkspaceManifestV1 {
            schema: WORKSPACE_MANIFEST_V1_SCHEMA.into(),
            version: 1,
            workspace_id: "demo".into(),
            packages: packages
                .into_iter()
                .map(|id| WorkspacePackageDescriptorV1 {
                    package_id: id.into(),
                    session_id: format!("session-{id}"),
                    display_name: None,
                    configuration_identity_hint: None,
                    metadata: BTreeMap::new(),
                })
                .collect(),
            dependencies: edges
                .into_iter()
                .map(|(a, b)| WorkspaceDependencyEdgeV1 {
                    dependency_package_id: a.into(),
                    dependent_package_id: b.into(),
                })
                .collect(),
            policy: WorkspacePolicyV1 {
                failure_mode: "fail_fast".into(),
                execution_mode: "deterministic_serial".into(),
                result_detail: "full".into(),
            },
        }
    }
    #[test]
    fn l7_chain_and_permutations_are_deterministic() {
        let a = manifest(
            vec!["app", "foundation", "feature"],
            vec![("feature", "app"), ("foundation", "feature")],
        );
        let b = manifest(
            vec!["feature", "app", "foundation"],
            vec![("foundation", "feature"), ("feature", "app")],
        );
        let ga = graph(&a).unwrap();
        let gb = graph(&b).unwrap();
        assert_eq!(ga.graph_identity, gb.graph_identity);
        let plan = super::plan(
            &ga,
            vec![
                ("app".into(), "a".into()),
                ("foundation".into(), "f".into()),
                ("feature".into(), "x".into()),
            ],
        );
        assert_eq!(plan.stages[0].packages, vec!["foundation"]);
        assert_eq!(plan.stages[2].packages, vec!["app"]);
    }
    #[test]
    fn l7_cycle_is_canonical() {
        let error = graph(&manifest(vec!["a", "b"], vec![("a", "b"), ("b", "a")])).unwrap_err();
        assert_eq!(error.code(), "L7W001_WORKSPACE_DEPENDENCY_CYCLE");
    }
}
