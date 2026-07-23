//! Compiler-owned static request and deployment handoffs for Phase Q.

use std::collections::BTreeMap;

use serde::Serialize;

use crate::platform::Digest;
use crate::RouteManifestV1;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct StaticRequestHandoffV1 {
    pub schema_version: u32,
    pub routes: Vec<StaticRequestRouteV1>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct StaticRequestRouteV1 {
    pub method: String,
    pub path: String,
    pub artifact: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct DeployableReleaseManifestV1 {
    pub schema_version: u32,
    pub release_id: String,
    pub route_manifest_digest: String,
    pub artifacts: Vec<DeployableArtifactV1>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct DeployableArtifactV1 {
    pub path: String,
    pub digest: String,
    pub size: u64,
}

/// Creates static GET request ownership from the compiler route manifest.
/// It intentionally does not create handlers, SSR, loaders, or server actions.
#[must_use]
pub fn build_static_request_handoff_v1(routes: &RouteManifestV1) -> StaticRequestHandoffV1 {
    StaticRequestHandoffV1 {
        schema_version: 1,
        routes: routes
            .routes
            .iter()
            .map(|route| StaticRequestRouteV1 {
                method: "GET".into(),
                path: route.path.clone(),
                artifact: format!("{}/index.html", route.artifact_root),
            })
            .collect(),
    }
}

/// Derives a provider-neutral immutable release inventory from compiler bytes.
#[must_use]
pub fn build_deployable_release_manifest_v1(
    routes: &RouteManifestV1,
    artifacts: &BTreeMap<std::path::PathBuf, Vec<u8>>,
) -> DeployableReleaseManifestV1 {
    let inventory = artifacts
        .iter()
        .map(|(path, bytes)| DeployableArtifactV1 {
            path: path.to_string_lossy().replace('\\', "/"),
            digest: Digest::sha256(bytes).to_string(),
            size: u64::try_from(bytes.len()).expect("artifact size exceeds u64"),
        })
        .collect::<Vec<_>>();
    let route_manifest = serde_json::to_vec(routes).expect("route manifest serializes");
    let route_manifest_digest = Digest::sha256(&route_manifest).to_string();
    let release_id = Digest::sha256(
        inventory
            .iter()
            .map(|artifact| format!("{}:{}:{}\n", artifact.path, artifact.digest, artifact.size))
            .collect::<String>(),
    )
    .to_string();
    DeployableReleaseManifestV1 {
        schema_version: 1,
        release_id,
        route_manifest_digest,
        artifacts: inventory,
    }
}

#[must_use]
pub fn static_request_handoff_json_v1(value: &StaticRequestHandoffV1) -> String {
    serde_json::to_string_pretty(value).expect("request handoff serializes") + "\n"
}

#[must_use]
pub fn deployable_release_manifest_json_v1(value: &DeployableReleaseManifestV1) -> String {
    serde_json::to_string_pretty(value).expect("release manifest serializes") + "\n"
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::RouteManifestEntryV1;

    #[test]
    fn derives_static_requests_and_stable_release_inventory() {
        let routes = RouteManifestV1 {
            schema_version: 1,
            routes: vec![RouteManifestEntryV1 {
                path: "/".into(),
                component_id: "component:home".into(),
                artifact_root: "routes/root".into(),
                parent_path: None,
            }],
        };
        let request = build_static_request_handoff_v1(&routes);
        assert_eq!(request.routes[0].artifact, "routes/root/index.html");
        let mut artifacts = BTreeMap::new();
        artifacts.insert("routes/root/index.html".into(), b"<main />".to_vec());
        let first = build_deployable_release_manifest_v1(&routes, &artifacts);
        let second = build_deployable_release_manifest_v1(&routes, &artifacts);
        assert_eq!(first.release_id, second.release_id);
        assert!(deployable_release_manifest_json_v1(&first).contains("release_id"));
    }
}
