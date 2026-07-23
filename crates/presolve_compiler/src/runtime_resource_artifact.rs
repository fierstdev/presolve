//! N6-C3 deterministic Resource declaration and activation artifact.
//!
//! This is a projection of already-resolved compiler products. It deliberately
//! has no endpoint transport or executable runtime behavior.

use serde::{Deserialize, Serialize};

use crate::{
    semantic_type_text, ApplicationSemanticModel, ResourceEndpointResolutionOutcome,
    ResourceLifecycleState,
};

pub const RUNTIME_RESOURCE_ARTIFACT_SCHEMA_VERSION: u32 = 1;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RuntimeResourceArtifact {
    pub schema_version: u32,
    pub declarations: Vec<RuntimeResourceArtifactDeclaration>,
    pub activations: Vec<RuntimeResourceArtifactActivation>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RuntimeResourceArtifactDeclaration {
    pub id: String,
    pub owner_component: String,
    pub key: String,
    pub data_type: String,
    pub error_type: String,
    pub execution_boundary: String,
    pub input_dependencies: Vec<String>,
    pub retry_policy: String,
    pub invalidation_policy: String,
    pub endpoint: RuntimeResourceArtifactEndpoint,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RuntimeResourceArtifactEndpoint {
    pub package: String,
    pub version: String,
    pub integrity: String,
    pub export: String,
    pub type_signature: String,
    pub runtime_module: String,
    pub resume_policy: String,
    pub cancellation: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RuntimeResourceArtifactActivation {
    pub id: String,
    pub declaration: String,
    pub component_instance: String,
    pub state: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub generation: Option<u64>,
}

/// Projects immutable Resource declaration and instance-activation records.
///
/// # Panics
///
/// Panics when an internal declaration has no resolved resource endpoint. That
/// would violate the N6-C declaration-projection invariant.
#[must_use]
pub fn build_runtime_resource_artifact(
    model: &ApplicationSemanticModel,
) -> RuntimeResourceArtifact {
    let declarations = model
        .resource_declarations
        .values()
        .map(|declaration| {
            let endpoint = model
                .resource_endpoint_resolutions
                .iter()
                .find(|resolution| {
                    resolution.owner_component == declaration.owner_component
                        && resolution.field == declaration.name
                })
                .and_then(|resolution| match &resolution.outcome {
                    ResourceEndpointResolutionOutcome::Resolved(endpoint) => Some(endpoint),
                    _ => None,
                })
                .expect("projected Resource declaration must have resolved endpoint");
            RuntimeResourceArtifactDeclaration {
                id: declaration.id.as_str().to_string(),
                owner_component: declaration.owner_component.as_str().to_string(),
                key: declaration.key.clone(),
                data_type: semantic_type_text(&declaration.data_type),
                error_type: semantic_type_text(&declaration.error_type),
                execution_boundary: format!("{:?}", declaration.execution_boundary),
                input_dependencies: declaration
                    .input_dependencies
                    .iter()
                    .map(|dependency| dependency.as_str().to_string())
                    .collect(),
                retry_policy: format!("{:?}", declaration.retry_policy),
                invalidation_policy: format!("{:?}", declaration.invalidation_policy),
                endpoint: RuntimeResourceArtifactEndpoint {
                    package: endpoint.package.clone(),
                    version: endpoint.version.clone(),
                    integrity: endpoint.integrity.clone(),
                    export: endpoint.export.clone(),
                    type_signature: endpoint.type_signature.clone(),
                    runtime_module: endpoint.runtime_module.clone(),
                    resume_policy: endpoint.resume_policy.clone(),
                    cancellation: format!("{:?}", endpoint.endpoint.cancellation),
                },
            }
        })
        .collect();
    let activations = model
        .resource_activations
        .values()
        .map(|activation| {
            let (state, generation) = resource_lifecycle_artifact_state(activation.state);
            RuntimeResourceArtifactActivation {
                id: activation.id.as_str().to_string(),
                declaration: activation.declaration.as_str().to_string(),
                component_instance: activation.component_instance.as_str().to_string(),
                state: state.to_string(),
                generation,
            }
        })
        .collect();
    RuntimeResourceArtifact {
        schema_version: RUNTIME_RESOURCE_ARTIFACT_SCHEMA_VERSION,
        declarations,
        activations,
    }
}

#[must_use]
pub fn runtime_resource_artifact_json(artifact: &RuntimeResourceArtifact) -> String {
    serde_json::to_string_pretty(artifact).expect("resource artifact should serialize") + "\n"
}

fn resource_lifecycle_artifact_state(state: ResourceLifecycleState) -> (&'static str, Option<u64>) {
    match state {
        ResourceLifecycleState::Idle => ("idle", None),
        ResourceLifecycleState::Pending { generation } => ("pending", Some(generation)),
        ResourceLifecycleState::Ready { generation } => ("ready", Some(generation)),
        ResourceLifecycleState::Failed { generation } => ("failed", Some(generation)),
        ResourceLifecycleState::Cancelled { generation } => ("cancelled", Some(generation)),
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        build_application_semantic_model_for_unit_with_packages, build_runtime_resource_artifact,
        parse_semantic_package_contract, runtime_resource_artifact_json, CompilationUnit,
        SemanticPackageResolutionTable, RUNTIME_RESOURCE_ARTIFACT_SCHEMA_VERSION,
    };

    fn model() -> crate::ApplicationSemanticModel {
        let unit = CompilationUnit::parse_sources([(
            "src/Profile.tsx",
            r#"
import { loadProfile } from "profile-service";
@component("x-profile")
class Profile extends Component {
  @resource("loadProfile") profile!: Resource<string, string>;
  render() { return <div>Profile</div>; }
}
"#,
        )]);
        let contract = parse_semantic_package_contract(
            r#"{"schema_version":1,"package":"profile-service","version":"1.2.3","integrity":"sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa","exports":{"loadProfile":{"kind":"resource","type_signature":"(ProfileKey) -> Resource<Profile, ProfileError>","runtime_module":"dist/load-profile.js","resume_policy":"snapshot","resource_endpoint":{"execution_boundary":"shared","cancellation":"abort","resume":"snapshot"}}}}"#,
        )
        .expect("resource contract");
        let mut packages = SemanticPackageResolutionTable::default();
        packages.insert("profile-service".into(), contract).unwrap();
        build_application_semantic_model_for_unit_with_packages(&unit, &packages)
    }

    #[test]
    fn projects_resolved_resource_declaration_and_idle_activation_deterministically() {
        let model = model();
        let artifact = build_runtime_resource_artifact(&model);
        assert_eq!(
            artifact.schema_version,
            RUNTIME_RESOURCE_ARTIFACT_SCHEMA_VERSION
        );
        assert_eq!(artifact.declarations.len(), 1);
        assert_eq!(artifact.declarations[0].endpoint.package, "profile-service");
        assert_eq!(artifact.declarations[0].endpoint.export, "loadProfile");
        assert_eq!(artifact.activations.len(), 1);
        assert_eq!(artifact.activations[0].state, "idle");
        assert_eq!(
            runtime_resource_artifact_json(&artifact),
            runtime_resource_artifact_json(&build_runtime_resource_artifact(&model))
        );
    }
}
