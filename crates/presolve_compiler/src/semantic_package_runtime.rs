//! Explicit host-owned locations for semantic-package runtime modules.

use std::collections::BTreeMap;

use crate::SemanticPackageContract;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct SemanticPackageRuntimeModuleKey {
    pub package: String,
    pub version: String,
    pub integrity: String,
    pub runtime_module: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SemanticPackageRuntimeModuleError {
    EmptyLocation,
    DuplicateRuntimeModule,
    ContractMismatch,
}

/// The application/metaframework-provided location of exact package runtime
/// bytes. This table is a compiler input; it performs no package discovery.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct SemanticPackageRuntimeModuleTable {
    modules: BTreeMap<SemanticPackageRuntimeModuleKey, String>,
}

impl SemanticPackageRuntimeModuleTable {
    pub fn insert(
        &mut self,
        key: SemanticPackageRuntimeModuleKey,
        location: String,
    ) -> Result<(), SemanticPackageRuntimeModuleError> {
        if location.is_empty() {
            return Err(SemanticPackageRuntimeModuleError::EmptyLocation);
        }
        if self.modules.contains_key(&key) {
            return Err(SemanticPackageRuntimeModuleError::DuplicateRuntimeModule);
        }
        self.modules.insert(key, location);
        Ok(())
    }

    #[must_use]
    pub fn resolve(&self, key: &SemanticPackageRuntimeModuleKey) -> Option<&str> {
        self.modules.get(key).map(String::as_str)
    }

    pub fn resolve_contract_module(
        &self,
        contract: &SemanticPackageContract,
        runtime_module: &str,
    ) -> Result<&str, SemanticPackageRuntimeModuleError> {
        let key = SemanticPackageRuntimeModuleKey {
            package: contract.package.clone(),
            version: contract.version.clone(),
            integrity: contract.integrity.clone(),
            runtime_module: runtime_module.to_string(),
        };
        self.resolve(&key)
            .ok_or(SemanticPackageRuntimeModuleError::ContractMismatch)
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        parse_semantic_package_contract, SemanticPackageRuntimeModuleError,
        SemanticPackageRuntimeModuleKey, SemanticPackageRuntimeModuleTable,
    };

    #[test]
    fn resolves_only_the_exact_integrity_checked_runtime_module_coordinate() {
        let contract = parse_semantic_package_contract(
            r#"{"schema_version":1,"package":"profile-service","version":"1.2.3","integrity":"sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa","exports":{"loadProfile":{"kind":"resource","type_signature":"() -> Resource<string, string>","runtime_module":"dist/load-profile.js","resume_policy":"snapshot","resource_endpoint":{"execution_boundary":"shared","cancellation":"abort","resume":"snapshot"}}}}"#,
        )
        .unwrap();
        let mut modules = SemanticPackageRuntimeModuleTable::default();
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
        assert_eq!(
            modules
                .resolve_contract_module(&contract, "dist/load-profile.js")
                .unwrap(),
            "./vendor/profile-service.js"
        );
        assert_eq!(
            modules.resolve_contract_module(&contract, "other.js"),
            Err(SemanticPackageRuntimeModuleError::ContractMismatch)
        );
    }
}
