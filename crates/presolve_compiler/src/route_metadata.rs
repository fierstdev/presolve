//! Compiler-owned metadata projection over canonical file routes.

use std::collections::{BTreeMap, BTreeSet};

use serde::Serialize;

use crate::FileRoutePublicationManifestV1;

pub const ROUTE_METADATA_SCHEMA_VERSION: u32 = 1;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RouteMetadataInputV1 {
    pub route_path: String,
    pub title: String,
    pub description: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RouteMetadataManifestV1 {
    pub schema_version: u32,
    pub routes: Vec<RouteMetadataRecordV1>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RouteMetadataRecordV1 {
    pub path: String,
    pub entry_component_id: String,
    pub title: String,
    pub description: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RouteMetadataErrorV1 {
    pub code: &'static str,
    pub message: String,
}

pub fn build_route_metadata_manifest_v1(
    routes: &FileRoutePublicationManifestV1,
    inputs: &[RouteMetadataInputV1],
) -> Result<RouteMetadataManifestV1, RouteMetadataErrorV1> {
    let canonical = routes
        .routes
        .iter()
        .map(|route| (route.path.as_str(), route.entry_component_id.as_str()))
        .collect::<BTreeMap<_, _>>();
    let mut seen = BTreeSet::new();
    let mut records = Vec::new();
    for input in inputs {
        if !seen.insert(&input.route_path) {
            return Err(RouteMetadataErrorV1 {
                code: "PSMETA1001_DUPLICATE_ROUTE",
                message: input.route_path.clone(),
            });
        }
        let Some(entry_component_id) = canonical.get(input.route_path.as_str()) else {
            return Err(RouteMetadataErrorV1 {
                code: "PSMETA1002_ROUTE_UNKNOWN",
                message: input.route_path.clone(),
            });
        };
        if input.title.trim().is_empty()
            || input
                .description
                .as_ref()
                .is_some_and(|value| value.trim().is_empty())
        {
            return Err(RouteMetadataErrorV1 {
                code: "PSMETA1003_FIELD_INVALID",
                message: input.route_path.clone(),
            });
        }
        records.push(RouteMetadataRecordV1 {
            path: input.route_path.clone(),
            entry_component_id: (*entry_component_id).into(),
            title: input.title.clone(),
            description: input.description.clone(),
        });
    }
    records.sort_by(|left, right| left.path.cmp(&right.path));
    Ok(RouteMetadataManifestV1 {
        schema_version: ROUTE_METADATA_SCHEMA_VERSION,
        routes: records,
    })
}

#[must_use]
pub fn route_metadata_manifest_json_v1(value: &RouteMetadataManifestV1) -> String {
    serde_json::to_string_pretty(value).expect("route metadata serializes") + "\n"
}

#[cfg(test)]
mod tests {
    use super::{build_route_metadata_manifest_v1, RouteMetadataInputV1};
    use crate::{FileRoutePublicationManifestV1, FileRoutePublicationRouteV1};
    fn routes() -> FileRoutePublicationManifestV1 {
        FileRoutePublicationManifestV1 {
            schema_version: 1,
            compiler_contract: "presolve-file-route-publication:1".into(),
            profile: "production".into(),
            routes: vec![FileRoutePublicationRouteV1 {
                path: "/docs".into(),
                entry_component_id: "component:docs".into(),
                artifact_root: "routes/segment-docs".into(),
                layout_component_ids: Vec::new(),
            }],
            artifacts: Vec::new(),
        }
    }
    #[test]
    fn joins_metadata_only_to_a_canonical_route() {
        let product = build_route_metadata_manifest_v1(
            &routes(),
            &[RouteMetadataInputV1 {
                route_path: "/docs".into(),
                title: "Docs".into(),
                description: None,
            }],
        )
        .unwrap();
        assert_eq!(product.routes[0].entry_component_id, "component:docs");
        assert_eq!(
            build_route_metadata_manifest_v1(
                &routes(),
                &[RouteMetadataInputV1 {
                    route_path: "/missing".into(),
                    title: "Missing".into(),
                    description: None
                }]
            )
            .unwrap_err()
            .code,
            "PSMETA1002_ROUTE_UNKNOWN"
        );
    }
}
