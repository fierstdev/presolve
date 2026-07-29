//! Deterministic runtime metadata for decorator-free terminal package calls.

use std::collections::BTreeSet;

use serde::{Deserialize, Serialize};

use crate::ApplicationSemanticModel;

pub const RUNTIME_PACKAGE_INVOCATION_ARTIFACT_SCHEMA_VERSION: u32 = 1;
pub const PACKAGE_INVOCATION_REGISTRY_PATH: &str = "/presolve.package-invocations.js";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RuntimePackageInvocationArtifact {
    pub schema_version: u32,
    pub registry_path: String,
    pub invocations: Vec<RuntimePackageInvocation>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RuntimePackageInvocation {
    pub id: String,
    pub owner_component: String,
    pub method: String,
    pub module_specifier: String,
    pub export: String,
    pub owner_source: String,
    pub declaration_modules: Vec<String>,
    pub execution_boundary: String,
    pub resume_policy: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RuntimePackageInvocationArtifactValidationError {
    UnsupportedSchemaVersion { actual: u32 },
    InvalidRegistryPath,
    DuplicateInvocation { id: String },
    MissingCoordinate { id: String },
    InvalidLifecycle { id: String },
}

#[must_use]
pub fn build_runtime_package_invocation_artifact(
    model: &ApplicationSemanticModel,
) -> RuntimePackageInvocationArtifact {
    let mut invocations = model
        .terminal_package_invocations
        .iter()
        .map(|fact| {
            let mut declaration_modules = fact.declaration_modules.clone();
            declaration_modules.sort();
            declaration_modules.dedup();
            RuntimePackageInvocation {
                id: fact.id.to_string(),
                owner_component: fact.owner_component.to_string(),
                method: fact.method.to_string(),
                module_specifier: fact.module_specifier.clone(),
                export: fact.export_name.clone(),
                owner_source: fact.provenance.path.to_string_lossy().into_owned(),
                declaration_modules,
                execution_boundary: "client".into(),
                resume_policy: "event_restore".into(),
            }
        })
        .collect::<Vec<_>>();
    invocations.sort_by(|left, right| left.id.cmp(&right.id));
    RuntimePackageInvocationArtifact {
        schema_version: RUNTIME_PACKAGE_INVOCATION_ARTIFACT_SCHEMA_VERSION,
        registry_path: PACKAGE_INVOCATION_REGISTRY_PATH.into(),
        invocations,
    }
}

#[must_use]
pub fn runtime_package_invocation_artifact_json(
    artifact: &RuntimePackageInvocationArtifact,
) -> String {
    serde_json::to_string_pretty(artifact).expect("package invocation artifact should serialize")
        + "\n"
}

#[must_use]
pub fn validate_runtime_package_invocation_artifact(
    artifact: &RuntimePackageInvocationArtifact,
) -> Vec<RuntimePackageInvocationArtifactValidationError> {
    let mut errors = Vec::new();
    if artifact.schema_version != RUNTIME_PACKAGE_INVOCATION_ARTIFACT_SCHEMA_VERSION {
        errors.push(
            RuntimePackageInvocationArtifactValidationError::UnsupportedSchemaVersion {
                actual: artifact.schema_version,
            },
        );
    }
    if artifact.registry_path != PACKAGE_INVOCATION_REGISTRY_PATH {
        errors.push(RuntimePackageInvocationArtifactValidationError::InvalidRegistryPath);
    }
    let mut ids = BTreeSet::new();
    for invocation in &artifact.invocations {
        if !ids.insert(invocation.id.clone()) {
            errors.push(
                RuntimePackageInvocationArtifactValidationError::DuplicateInvocation {
                    id: invocation.id.clone(),
                },
            );
        }
        if invocation.id.is_empty()
            || invocation.owner_component.is_empty()
            || invocation.method.is_empty()
            || invocation.module_specifier.is_empty()
            || invocation.export.is_empty()
            || invocation.owner_source.is_empty()
            || invocation.declaration_modules.is_empty()
        {
            errors.push(
                RuntimePackageInvocationArtifactValidationError::MissingCoordinate {
                    id: invocation.id.clone(),
                },
            );
        }
        if invocation.execution_boundary != "client" || invocation.resume_policy != "event_restore"
        {
            errors.push(
                RuntimePackageInvocationArtifactValidationError::InvalidLifecycle {
                    id: invocation.id.clone(),
                },
            );
        }
    }
    errors
}

#[cfg(test)]
mod tests {
    use super::{
        runtime_package_invocation_artifact_json, validate_runtime_package_invocation_artifact,
        RuntimePackageInvocation, RuntimePackageInvocationArtifact,
        RuntimePackageInvocationArtifactValidationError, PACKAGE_INVOCATION_REGISTRY_PATH,
        RUNTIME_PACKAGE_INVOCATION_ARTIFACT_SCHEMA_VERSION,
    };

    fn artifact() -> RuntimePackageInvocationArtifact {
        RuntimePackageInvocationArtifact {
            schema_version: RUNTIME_PACKAGE_INVOCATION_ARTIFACT_SCHEMA_VERSION,
            registry_path: PACKAGE_INVOCATION_REGISTRY_PATH.into(),
            invocations: vec![RuntimePackageInvocation {
                id: "component:Home/package-invocation:record".into(),
                owner_component: "component:Home".into(),
                method: "component:Home/action-endpoint:record".into(),
                module_specifier: "analytics-kit".into(),
                export: "recordVisit".into(),
                owner_source: "app/routes/index.tsx".into(),
                declaration_modules: vec!["node_modules/analytics-kit/index.d.ts".into()],
                execution_boundary: "client".into(),
                resume_policy: "event_restore".into(),
            }],
        }
    }

    #[test]
    fn validates_and_serializes_exact_package_invocation_coordinates() {
        let artifact = artifact();
        assert!(validate_runtime_package_invocation_artifact(&artifact).is_empty());
        let json = runtime_package_invocation_artifact_json(&artifact);
        assert!(json.contains("\"registry_path\": \"/presolve.package-invocations.js\""));
        assert!(json.contains("\"module_specifier\": \"analytics-kit\""));
    }

    #[test]
    fn rejects_duplicate_missing_and_noncanonical_package_invocations() {
        let mut artifact = artifact();
        artifact.schema_version += 1;
        artifact.registry_path = "relative.js".into();
        artifact.invocations[0].module_specifier.clear();
        artifact.invocations[0].resume_policy = "snapshot".into();
        artifact.invocations.push(artifact.invocations[0].clone());
        let errors = validate_runtime_package_invocation_artifact(&artifact);
        assert!(errors.iter().any(|error| matches!(
            error,
            RuntimePackageInvocationArtifactValidationError::UnsupportedSchemaVersion { .. }
        )));
        assert!(errors.iter().any(|error| matches!(
            error,
            RuntimePackageInvocationArtifactValidationError::InvalidRegistryPath
        )));
        assert!(errors.iter().any(|error| matches!(
            error,
            RuntimePackageInvocationArtifactValidationError::DuplicateInvocation { .. }
        )));
        assert!(errors.iter().any(|error| matches!(
            error,
            RuntimePackageInvocationArtifactValidationError::MissingCoordinate { .. }
        )));
        assert!(errors.iter().any(|error| matches!(
            error,
            RuntimePackageInvocationArtifactValidationError::InvalidLifecycle { .. }
        )));
    }
}
