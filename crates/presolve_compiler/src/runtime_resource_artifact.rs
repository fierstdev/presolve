//! N6-C3 deterministic Resource declaration and activation artifact.
//!
//! This is a projection of already-resolved compiler products. Its
//! execution-facing form contains the exact host-provided runtime module
//! location consumed by the generated browser runtime.

use std::collections::BTreeSet;

use serde::{Deserialize, Serialize};

use crate::{
    resume_value_codec, semantic_type_text, ApplicationSemanticModel,
    ResourceEndpointResolutionOutcome, ResourceLifecycleState, ResumeValueCodec,
    SemanticPackageRuntimeModuleKey, SemanticPackageRuntimeModuleTable,
};

/// Version 4 adds exact compiler-owned server bootstrap descriptors for route
/// loader activations. Dynamic values remain request-owned Node output.
pub const RUNTIME_RESOURCE_ARTIFACT_SCHEMA_VERSION: u32 = 4;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RuntimeResourceArtifact {
    pub schema_version: u32,
    pub declarations: Vec<RuntimeResourceArtifactDeclaration>,
    pub activations: Vec<RuntimeResourceArtifactActivation>,
    pub server_bootstraps: Vec<RuntimeResourceArtifactServerBootstrap>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RuntimeResourceArtifactDeclaration {
    pub id: String,
    pub owner_component: String,
    pub key: String,
    pub data_type: String,
    pub error_type: String,
    pub data_codec: ResumeValueCodec,
    pub error_codec: ResumeValueCodec,
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
    #[serde(skip_serializing_if = "Option::is_none")]
    pub runtime_location: Option<String>,
    pub resume_policy: String,
    pub cancellation: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RuntimeResourceArtifactActivation {
    pub id: String,
    pub declaration: String,
    pub component_instance: String,
    pub state_slot: String,
    pub data_slot: String,
    pub error_slot: String,
    pub state: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub generation: Option<u64>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RuntimeResourceArtifactServerBootstrap {
    pub activation: String,
    pub declaration: String,
    pub component_instance: String,
    pub loader_capability_id: String,
    pub bootstrap_key: String,
    pub resume_policy: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RuntimeResourceArtifactValidationError {
    UnsupportedSchemaVersion {
        actual: u32,
    },
    DuplicateDeclarationId {
        id: String,
    },
    DuplicateActivationId {
        id: String,
    },
    MissingEndpointCoordinate {
        declaration: String,
    },
    InvalidLifecycleGeneration {
        activation: String,
    },
    UnknownActivationDeclaration {
        activation: String,
        declaration: String,
    },
    InvalidResumeSlotIdentity {
        activation: String,
    },
    InvalidValueCodec {
        declaration: String,
    },
    MissingRuntimeLocation {
        declaration: String,
    },
    InvalidServerBootstrap {
        activation: String,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RuntimeResourceArtifactBuildError {
    MissingRuntimeModuleLocation { declaration: String },
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
                data_codec: resume_value_codec(&declaration.data_type).expect(
                    "Resource declarations must use the compiler's closed runtime value codec vocabulary",
                ),
                error_codec: resume_value_codec(&declaration.error_type).expect(
                    "Resource declarations must use the compiler's closed runtime value codec vocabulary",
                ),
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
                    runtime_location: None,
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
                state_slot: activation.id.state_slot().as_str().to_string(),
                data_slot: activation.id.data_slot().as_str().to_string(),
                error_slot: activation.id.error_slot().as_str().to_string(),
                state: state.to_string(),
                generation,
            }
        })
        .collect();
    let server_bootstraps = model
        .resource_activations
        .values()
        .filter_map(|activation| {
            let declaration = model.resource_declarations.get(&activation.declaration)?;
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
                })?;
            endpoint.route_loader.as_ref()?;
            Some(RuntimeResourceArtifactServerBootstrap {
                activation: activation.id.as_str().to_owned(),
                declaration: declaration.id.as_str().to_owned(),
                component_instance: activation.component_instance.as_str().to_owned(),
                loader_capability_id: declaration
                    .owner_component
                    .route_loader(&declaration.name)
                    .to_string(),
                bootstrap_key: activation.id.as_str().to_owned(),
                resume_policy: "reload".into(),
            })
        })
        .collect();
    RuntimeResourceArtifact {
        schema_version: RUNTIME_RESOURCE_ARTIFACT_SCHEMA_VERSION,
        declarations,
        activations,
        server_bootstraps,
    }
}

/// Produces the execution-facing artifact only when every Resource endpoint
/// has an explicit host-supplied location for its exact package coordinate.
pub fn build_runtime_resource_artifact_with_modules(
    model: &ApplicationSemanticModel,
    modules: &SemanticPackageRuntimeModuleTable,
) -> Result<RuntimeResourceArtifact, RuntimeResourceArtifactBuildError> {
    let mut artifact = build_runtime_resource_artifact(model);
    for declaration in &mut artifact.declarations {
        if artifact
            .server_bootstraps
            .iter()
            .any(|bootstrap| bootstrap.declaration == declaration.id)
        {
            declaration.endpoint.runtime_location = None;
            continue;
        }
        let key = SemanticPackageRuntimeModuleKey {
            package: declaration.endpoint.package.clone(),
            version: declaration.endpoint.version.clone(),
            integrity: declaration.endpoint.integrity.clone(),
            runtime_module: declaration.endpoint.runtime_module.clone(),
        };
        declaration.endpoint.runtime_location = Some(
            modules
                .resolve(&key)
                .ok_or_else(
                    || RuntimeResourceArtifactBuildError::MissingRuntimeModuleLocation {
                        declaration: declaration.id.clone(),
                    },
                )?
                .to_string(),
        );
    }
    Ok(artifact)
}

#[must_use]
pub fn runtime_resource_artifact_json(artifact: &RuntimeResourceArtifact) -> String {
    serde_json::to_string_pretty(artifact).expect("resource artifact should serialize") + "\n"
}

/// Validates the exact identity and endpoint prerequisites required by the
/// Resource runtime boundary. Callers must reject the artifact on any error.
#[must_use]
pub fn validate_runtime_resource_artifact(
    artifact: &RuntimeResourceArtifact,
) -> Vec<RuntimeResourceArtifactValidationError> {
    let mut errors = Vec::new();
    if artifact.schema_version != RUNTIME_RESOURCE_ARTIFACT_SCHEMA_VERSION {
        errors.push(
            RuntimeResourceArtifactValidationError::UnsupportedSchemaVersion {
                actual: artifact.schema_version,
            },
        );
    }
    let mut declarations = BTreeSet::new();
    let bootstrap_declarations = artifact
        .server_bootstraps
        .iter()
        .map(|bootstrap| bootstrap.declaration.as_str())
        .collect::<BTreeSet<_>>();
    for declaration in &artifact.declarations {
        if !declarations.insert(declaration.id.clone()) {
            errors.push(
                RuntimeResourceArtifactValidationError::DuplicateDeclarationId {
                    id: declaration.id.clone(),
                },
            );
        }
        if declaration.endpoint.package.is_empty()
            || declaration.endpoint.version.is_empty()
            || declaration.endpoint.integrity.is_empty()
            || declaration.endpoint.export.is_empty()
            || declaration.endpoint.runtime_module.is_empty()
        {
            errors.push(
                RuntimeResourceArtifactValidationError::MissingEndpointCoordinate {
                    declaration: declaration.id.clone(),
                },
            );
        }
        if !is_valid_value_codec(&declaration.data_codec)
            || !is_valid_value_codec(&declaration.error_codec)
        {
            errors.push(RuntimeResourceArtifactValidationError::InvalidValueCodec {
                declaration: declaration.id.clone(),
            });
        }
        if bootstrap_declarations.contains(declaration.id.as_str())
            && declaration.endpoint.runtime_location.is_some()
        {
            errors.push(
                RuntimeResourceArtifactValidationError::MissingRuntimeLocation {
                    declaration: declaration.id.clone(),
                },
            );
        }
    }
    let mut activations = BTreeSet::new();
    for activation in &artifact.activations {
        if !activations.insert(activation.id.clone()) {
            errors.push(
                RuntimeResourceArtifactValidationError::DuplicateActivationId {
                    id: activation.id.clone(),
                },
            );
        }
        let generation_required = matches!(
            activation.state.as_str(),
            "pending" | "ready" | "failed" | "cancelled"
        );
        if generation_required != activation.generation.is_some() {
            errors.push(
                RuntimeResourceArtifactValidationError::InvalidLifecycleGeneration {
                    activation: activation.id.clone(),
                },
            );
        }
        if !declarations.contains(&activation.declaration) {
            errors.push(
                RuntimeResourceArtifactValidationError::UnknownActivationDeclaration {
                    activation: activation.id.clone(),
                    declaration: activation.declaration.clone(),
                },
            );
        }
        let expected_prefix = format!("{}/resource-slot:", activation.id);
        if activation.state_slot != format!("{expected_prefix}state")
            || activation.data_slot != format!("{expected_prefix}data")
            || activation.error_slot != format!("{expected_prefix}error")
            || activation.state_slot == activation.data_slot
            || activation.state_slot == activation.error_slot
            || activation.data_slot == activation.error_slot
        {
            errors.push(
                RuntimeResourceArtifactValidationError::InvalidResumeSlotIdentity {
                    activation: activation.id.clone(),
                },
            );
        }
    }
    let activation_ids = artifact
        .activations
        .iter()
        .map(|activation| activation.id.as_str())
        .collect::<BTreeSet<_>>();
    let mut bootstrap_activations = BTreeSet::new();
    for bootstrap in &artifact.server_bootstraps {
        let valid = bootstrap_activations.insert(bootstrap.activation.as_str())
            && activation_ids.contains(bootstrap.activation.as_str())
            && declarations.contains(bootstrap.declaration.as_str())
            && bootstrap.bootstrap_key == bootstrap.activation
            && !bootstrap.loader_capability_id.is_empty()
            && bootstrap.resume_policy == "reload"
            && artifact.activations.iter().any(|activation| {
                activation.id == bootstrap.activation
                    && activation.declaration == bootstrap.declaration
                    && activation.component_instance == bootstrap.component_instance
            });
        if !valid {
            errors.push(
                RuntimeResourceArtifactValidationError::InvalidServerBootstrap {
                    activation: bootstrap.activation.clone(),
                },
            );
        }
    }
    errors
}

fn is_valid_value_codec(codec: &ResumeValueCodec) -> bool {
    match codec {
        ResumeValueCodec::NullCodec
        | ResumeValueCodec::BooleanCodec
        | ResumeValueCodec::NumberCodec
        | ResumeValueCodec::StringCodec => true,
        ResumeValueCodec::ArrayCodec(element) | ResumeValueCodec::NullableCodec(element) => {
            is_valid_value_codec(element)
        }
        ResumeValueCodec::ObjectCodec(properties) => {
            let mut names = BTreeSet::new();
            properties.iter().all(|property| {
                !property.name.is_empty()
                    && names.insert(property.name.as_str())
                    && is_valid_value_codec(&property.codec)
            })
        }
    }
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
        build_runtime_resource_artifact_with_modules, parse_semantic_package_contract,
        runtime_resource_artifact_json, validate_runtime_resource_artifact, CompilationUnit,
        SemanticPackageResolutionTable, SemanticPackageRuntimeModuleKey,
        SemanticPackageRuntimeModuleTable, RUNTIME_RESOURCE_ARTIFACT_SCHEMA_VERSION,
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

    fn ordered_model(reverse: bool) -> crate::ApplicationSemanticModel {
        let first = (
            "src/Account.tsx",
            r#"import { loadAccount } from "profile-service";
@component("x-account") @route("/account") class Account extends Component {
  @resource("loadAccount") account!: Resource<string, string>;
  render() { return <main>Account</main>; }
}"#,
        );
        let second = (
            "src/Profile.tsx",
            r#"import { loadProfile } from "profile-service";
@component("x-profile") @route("/profile") class Profile extends Component {
  @resource("loadProfile") profile!: Resource<string, string>;
  render() { return <main>Profile</main>; }
}"#,
        );
        let files = if reverse {
            [second, first]
        } else {
            [first, second]
        };
        let unit = CompilationUnit::parse_sources(files);
        let contract = parse_semantic_package_contract(
            r#"{"schema_version":1,"package":"profile-service","version":"1.2.3","integrity":"sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa","exports":{"loadAccount":{"kind":"resource","type_signature":"() -> Resource<string, string>","runtime_module":"dist/load-account.js","resume_policy":"snapshot","resource_endpoint":{"execution_boundary":"shared","cancellation":"abort","resume":"snapshot"}},"loadProfile":{"kind":"resource","type_signature":"() -> Resource<string, string>","runtime_module":"dist/load-profile.js","resume_policy":"snapshot","resource_endpoint":{"execution_boundary":"shared","cancellation":"abort","resume":"snapshot"}}}}"#,
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
        assert_eq!(
            artifact.declarations[0].data_codec,
            crate::ResumeValueCodec::StringCodec
        );
        assert_eq!(
            artifact.declarations[0].error_codec,
            crate::ResumeValueCodec::StringCodec
        );
        assert_eq!(artifact.activations.len(), 1);
        assert_eq!(artifact.activations[0].state, "idle");
        assert!(validate_runtime_resource_artifact(&artifact).is_empty());
        assert_eq!(
            runtime_resource_artifact_json(&artifact),
            runtime_resource_artifact_json(&build_runtime_resource_artifact(&model))
        );
    }

    #[test]
    fn rejects_malformed_resource_artifact_identity_and_lifecycle_records() {
        let model = model();
        let mut artifact = build_runtime_resource_artifact(&model);
        artifact.schema_version = 9;
        artifact.declarations[0].endpoint.integrity.clear();
        artifact.declarations[0].data_codec = crate::ResumeValueCodec::ObjectCodec(vec![
            crate::ResumeObjectPropertyCodec {
                name: "duplicate".to_string(),
                codec: crate::ResumeValueCodec::StringCodec,
            },
            crate::ResumeObjectPropertyCodec {
                name: "duplicate".to_string(),
                codec: crate::ResumeValueCodec::StringCodec,
            },
        ]);
        artifact.activations[0].declaration = "resource:missing".to_string();
        artifact.activations[0].data_slot = "fabricated:slot".to_string();
        artifact.activations[0].state = "ready".to_string();

        let errors = validate_runtime_resource_artifact(&artifact);
        assert!(errors.iter().any(|error| matches!(
            error,
            crate::RuntimeResourceArtifactValidationError::UnsupportedSchemaVersion { actual: 9 }
        )));
        assert!(errors.iter().any(|error| matches!(
            error,
            crate::RuntimeResourceArtifactValidationError::MissingEndpointCoordinate { .. }
        )));
        assert!(errors.iter().any(|error| matches!(
            error,
            crate::RuntimeResourceArtifactValidationError::InvalidValueCodec { .. }
        )));
        assert!(errors.iter().any(|error| matches!(
            error,
            crate::RuntimeResourceArtifactValidationError::InvalidLifecycleGeneration { .. }
        )));
        assert!(errors.iter().any(|error| matches!(
            error,
            crate::RuntimeResourceArtifactValidationError::UnknownActivationDeclaration { .. }
        )));
        assert!(errors.iter().any(|error| matches!(
            error,
            crate::RuntimeResourceArtifactValidationError::InvalidResumeSlotIdentity { .. }
        )));
    }

    #[test]
    fn resource_artifact_and_resume_manifest_are_deterministic_under_source_reversal() {
        let forward = ordered_model(false);
        let reverse = ordered_model(true);
        assert_eq!(
            runtime_resource_artifact_json(&build_runtime_resource_artifact(&forward)),
            runtime_resource_artifact_json(&build_runtime_resource_artifact(&reverse))
        );
        assert_eq!(
            crate::resume_manifest_json(&crate::build_resume_manifest(&forward)),
            crate::resume_manifest_json(&crate::build_resume_manifest(&reverse))
        );
    }

    #[test]
    fn requires_exact_runtime_module_location_for_execution_facing_artifact() {
        let model = model();
        let mut modules = SemanticPackageRuntimeModuleTable::default();
        assert!(matches!(
            build_runtime_resource_artifact_with_modules(&model, &modules),
            Err(crate::RuntimeResourceArtifactBuildError::MissingRuntimeModuleLocation { .. })
        ));
        modules
            .insert(
                SemanticPackageRuntimeModuleKey {
                    package: "profile-service".into(),
                    version: "1.2.3".into(),
                    integrity:
                        "sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"
                            .into(),
                    runtime_module: "dist/load-profile.js".into(),
                },
                "./vendor/profile-service.js".into(),
            )
            .unwrap();
        let artifact = build_runtime_resource_artifact_with_modules(&model, &modules).unwrap();
        assert_eq!(
            artifact.declarations[0]
                .endpoint
                .runtime_location
                .as_deref(),
            Some("./vendor/profile-service.js")
        );
    }
}
