//! Deterministic runtime artifact for resolved opaque terminal Actions.

use std::collections::BTreeSet;

use serde::{Deserialize, Serialize};

use crate::{
    ApplicationSemanticModel, OpaqueActionResolutionOutcome, SemanticPackageRuntimeModuleKey,
    SemanticPackageRuntimeModuleTable,
};

pub const RUNTIME_OPAQUE_ARTIFACT_SCHEMA_VERSION: u32 = 1;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RuntimeOpaqueArtifact {
    pub schema_version: u32,
    pub activations: Vec<RuntimeOpaqueArtifactActivation>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RuntimeOpaqueArtifactActivation {
    pub id: String,
    pub owner_component: String,
    pub method: String,
    pub package: String,
    pub version: String,
    pub integrity: String,
    pub export: String,
    pub type_signature: String,
    pub runtime_module: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub runtime_location: Option<String>,
    pub execution_boundary: String,
    pub resume_policy: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RuntimeOpaqueArtifactBuildError {
    MissingRuntimeModuleLocation { activation: String },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RuntimeOpaqueArtifactValidationError {
    UnsupportedSchemaVersion { actual: u32 },
    DuplicateActivationId { id: String },
    MissingTerminalCoordinate { activation: String },
    InvalidTerminalContract { activation: String },
}

/// Projects only resolved, compiler-owned opaque terminal selections. Invalid
/// or unresolved source declarations cannot reach this artifact.
#[must_use]
pub fn build_runtime_opaque_artifact(model: &ApplicationSemanticModel) -> RuntimeOpaqueArtifact {
    let activations = model
        .opaque_action_resolutions
        .iter()
        .filter_map(|resolution| {
            let OpaqueActionResolutionOutcome::Resolved(binding) = &resolution.outcome else {
                return None;
            };
            Some(RuntimeOpaqueArtifactActivation {
                id: resolution.activation.as_str().to_string(),
                owner_component: resolution.owner_component.as_str().to_string(),
                method: resolution.method.as_str().to_string(),
                package: binding.package.clone(),
                version: binding.version.clone(),
                integrity: binding.integrity.clone(),
                export: binding.export.clone(),
                type_signature: binding.type_signature.clone(),
                runtime_module: binding.runtime_module.clone(),
                runtime_location: None,
                execution_boundary: "client".to_string(),
                resume_policy: binding.resume_policy.clone(),
            })
        })
        .collect();
    RuntimeOpaqueArtifact {
        schema_version: RUNTIME_OPAQUE_ARTIFACT_SCHEMA_VERSION,
        activations,
    }
}

/// Binds runtime locations only through the exact package/version/integrity/
/// module coordinate already selected by semantic resolution.
pub fn build_runtime_opaque_artifact_with_modules(
    model: &ApplicationSemanticModel,
    modules: &SemanticPackageRuntimeModuleTable,
) -> Result<RuntimeOpaqueArtifact, RuntimeOpaqueArtifactBuildError> {
    let mut artifact = build_runtime_opaque_artifact(model);
    for activation in &mut artifact.activations {
        let key = SemanticPackageRuntimeModuleKey {
            package: activation.package.clone(),
            version: activation.version.clone(),
            integrity: activation.integrity.clone(),
            runtime_module: activation.runtime_module.clone(),
        };
        activation.runtime_location = Some(
            modules
                .resolve(&key)
                .ok_or_else(
                    || RuntimeOpaqueArtifactBuildError::MissingRuntimeModuleLocation {
                        activation: activation.id.clone(),
                    },
                )?
                .to_string(),
        );
    }
    Ok(artifact)
}

#[must_use]
pub fn runtime_opaque_artifact_json(artifact: &RuntimeOpaqueArtifact) -> String {
    serde_json::to_string_pretty(artifact).expect("opaque artifact should serialize") + "\n"
}

#[must_use]
pub fn validate_runtime_opaque_artifact(
    artifact: &RuntimeOpaqueArtifact,
) -> Vec<RuntimeOpaqueArtifactValidationError> {
    let mut errors = Vec::new();
    if artifact.schema_version != RUNTIME_OPAQUE_ARTIFACT_SCHEMA_VERSION {
        errors.push(
            RuntimeOpaqueArtifactValidationError::UnsupportedSchemaVersion {
                actual: artifact.schema_version,
            },
        );
    }
    let mut ids = BTreeSet::new();
    for activation in &artifact.activations {
        if !ids.insert(activation.id.clone()) {
            errors.push(
                RuntimeOpaqueArtifactValidationError::DuplicateActivationId {
                    id: activation.id.clone(),
                },
            );
        }
        if activation.id.is_empty()
            || activation.owner_component.is_empty()
            || activation.method.is_empty()
            || activation.package.is_empty()
            || activation.version.is_empty()
            || activation.integrity.is_empty()
            || activation.export.is_empty()
            || activation.runtime_module.is_empty()
        {
            errors.push(
                RuntimeOpaqueArtifactValidationError::MissingTerminalCoordinate {
                    activation: activation.id.clone(),
                },
            );
        }
        if activation.type_signature != "() -> void"
            || activation.execution_boundary != "client"
            || activation.resume_policy != "cold_fallback"
        {
            errors.push(
                RuntimeOpaqueArtifactValidationError::InvalidTerminalContract {
                    activation: activation.id.clone(),
                },
            );
        }
    }
    errors
}

#[cfg(test)]
mod tests {
    use crate::{
        build_application_semantic_model_for_unit_with_packages, build_runtime_opaque_artifact,
        build_runtime_opaque_artifact_with_modules, parse_semantic_package_contract,
        validate_runtime_opaque_artifact, CompilationUnit, RuntimeOpaqueArtifactBuildError,
        RuntimeOpaqueArtifactValidationError, SemanticPackageResolutionTable,
        SemanticPackageRuntimeModuleKey, SemanticPackageRuntimeModuleTable,
    };

    fn model() -> crate::ApplicationSemanticModel {
        let unit = CompilationUnit::parse_sources([(
            "src/Checkout.tsx",
            r#"
import { trackPurchase } from "@acme/analytics";
@component("x-checkout")
class Checkout extends Component {
  @action() @opaque("@acme/analytics", "trackPurchase") track(): void {}
  render() { return <button onClick={this.track}>Buy</button>; }
}
"#,
        )]);
        let contract = parse_semantic_package_contract(
            r#"{"schema_version":1,"package":"@acme/analytics","version":"1.2.3","integrity":"sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa","exports":{"trackPurchase":{"kind":"opaque","type_signature":"() -> void","runtime_module":"dist/track.js","resume_policy":"cold_fallback","opaque_terminal":{"execution_boundary":"client","resume":"cold_fallback"}}}}"#,
        )
        .unwrap();
        let mut packages = SemanticPackageResolutionTable::default();
        packages.insert("@acme/analytics".into(), contract).unwrap();
        build_application_semantic_model_for_unit_with_packages(&unit, &packages)
    }

    #[test]
    fn projects_and_validates_resolved_opaque_terminal_artifact() {
        let artifact = build_runtime_opaque_artifact(&model());
        assert_eq!(artifact.activations.len(), 1);
        assert_eq!(artifact.activations[0].export, "trackPurchase");
        assert_eq!(artifact.activations[0].execution_boundary, "client");
        assert!(validate_runtime_opaque_artifact(&artifact).is_empty());
    }

    #[test]
    fn requires_an_exact_host_bound_runtime_module_location() {
        let model = model();
        let modules = SemanticPackageRuntimeModuleTable::default();
        assert!(matches!(
            build_runtime_opaque_artifact_with_modules(&model, &modules),
            Err(RuntimeOpaqueArtifactBuildError::MissingRuntimeModuleLocation { .. })
        ));

        let mut modules = SemanticPackageRuntimeModuleTable::default();
        modules
            .insert(
                SemanticPackageRuntimeModuleKey {
                    package: "@acme/analytics".into(),
                    version: "1.2.3".into(),
                    integrity:
                        "sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"
                            .into(),
                    runtime_module: "dist/track.js".into(),
                },
                "./vendor/track.js".into(),
            )
            .unwrap();
        let artifact = build_runtime_opaque_artifact_with_modules(&model, &modules).unwrap();
        assert_eq!(
            artifact.activations[0].runtime_location.as_deref(),
            Some("./vendor/track.js")
        );
    }

    #[test]
    fn rejects_malformed_opaque_terminal_artifacts() {
        let mut artifact = build_runtime_opaque_artifact(&model());
        artifact.schema_version = 2;
        artifact.activations[0].type_signature = "(value: string) -> void".into();
        artifact.activations[0].integrity.clear();
        let errors = validate_runtime_opaque_artifact(&artifact);
        assert!(errors.iter().any(|error| matches!(
            error,
            RuntimeOpaqueArtifactValidationError::UnsupportedSchemaVersion { actual: 2 }
        )));
        assert!(errors.iter().any(|error| matches!(
            error,
            RuntimeOpaqueArtifactValidationError::InvalidTerminalContract { .. }
        )));
        assert!(errors.iter().any(|error| matches!(
            error,
            RuntimeOpaqueArtifactValidationError::MissingTerminalCoordinate { .. }
        )));
    }
}
