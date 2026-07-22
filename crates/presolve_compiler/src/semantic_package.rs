use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

pub const SEMANTIC_PACKAGE_CONTRACT_SCHEMA_VERSION: u32 = 1;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SemanticPackageKind {
    Pure,
    Capability,
    Resource,
    Codec,
    Component,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SemanticPackageExport {
    pub kind: SemanticPackageKind,
    pub type_signature: String,
    pub runtime_module: String,
    pub resume_policy: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SemanticPackageContract {
    pub schema_version: u32,
    pub package: String,
    pub version: String,
    pub integrity: String,
    pub exports: BTreeMap<String, SemanticPackageExport>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SemanticPackageContractError {
    InvalidJson,
    UnsupportedSchema,
    InvalidIntegrity,
    EmptyPackage,
    EmptyExport,
    DuplicateSpecifier,
}

pub fn parse_semantic_package_contract(
    json: &str,
) -> Result<SemanticPackageContract, SemanticPackageContractError> {
    let contract = serde_json::from_str::<SemanticPackageContract>(json)
        .map_err(|_| SemanticPackageContractError::InvalidJson)?;
    if contract.schema_version != SEMANTIC_PACKAGE_CONTRACT_SCHEMA_VERSION {
        return Err(SemanticPackageContractError::UnsupportedSchema);
    }
    if contract.package.is_empty() {
        return Err(SemanticPackageContractError::EmptyPackage);
    }
    if !contract.integrity.starts_with("sha256:")
        || contract.integrity.len() != 71
        || !contract.integrity[7..]
            .chars()
            .all(|c| c.is_ascii_hexdigit())
    {
        return Err(SemanticPackageContractError::InvalidIntegrity);
    }
    if contract.exports.is_empty() || contract.exports.keys().any(|name| name.is_empty()) {
        return Err(SemanticPackageContractError::EmptyExport);
    }
    Ok(contract)
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct SemanticPackageResolutionTable {
    pub contracts: BTreeMap<String, SemanticPackageContract>,
}

impl SemanticPackageResolutionTable {
    pub fn insert(
        &mut self,
        specifier: String,
        contract: SemanticPackageContract,
    ) -> Result<(), SemanticPackageContractError> {
        if self.contracts.insert(specifier, contract).is_some() {
            return Err(SemanticPackageContractError::DuplicateSpecifier);
        }
        Ok(())
    }
    #[must_use]
    pub fn resolve(&self, specifier: &str, export: &str) -> Option<&SemanticPackageExport> {
        self.contracts.get(specifier)?.exports.get(export)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn parses_integrity_checked_contract_and_resolves_an_export() {
        let contract = parse_semantic_package_contract(r#"{"schema_version":1,"package":"date-kit","version":"1.2.3","integrity":"sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa","exports":{"format":{"kind":"pure","type_signature":"(Date) -> string","runtime_module":"dist/format.js","resume_policy":"input_only"}}}"#).unwrap();
        let mut table = SemanticPackageResolutionTable::default();
        table.insert("date-kit".into(), contract).unwrap();
        assert_eq!(
            table.resolve("date-kit", "format").unwrap().kind,
            SemanticPackageKind::Pure
        );
    }
}
