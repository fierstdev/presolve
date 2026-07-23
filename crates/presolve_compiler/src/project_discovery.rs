//! Deterministic ergonomic project discovery for Phase R.

use std::fs;
use std::path::{Path, PathBuf};

use crate::platform::Digest;
use crate::{
    parse_semantic_package_contract, SemanticPackageResolutionTable,
    SemanticPackageRuntimeModuleKey, SemanticPackageRuntimeModuleTable,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DiscoveredProjectSourceV1 {
    pub logical_path: PathBuf,
    pub source: String,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DiscoveredProjectV1 {
    pub root: PathBuf,
    pub source_root: PathBuf,
    pub fingerprint: String,
    pub sources: Vec<DiscoveredProjectSourceV1>,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProjectDiscoveryErrorV1 {
    pub code: &'static str,
    pub message: String,
}

/// Resolves published `presolve.contract.json` metadata from a declared npm
/// package. Package source stays opaque; only verified contract/runtime paths
/// become compiler inputs.
pub fn discover_semantic_packages_v1(
    root: &Path,
    specifiers: &[String],
) -> Result<
    (
        SemanticPackageResolutionTable,
        SemanticPackageRuntimeModuleTable,
    ),
    ProjectDiscoveryErrorV1,
> {
    let mut contracts = SemanticPackageResolutionTable::default();
    let mut modules = SemanticPackageRuntimeModuleTable::default();
    for specifier in specifiers {
        let package_root = root.join("node_modules").join(specifier);
        let contract_path = package_root.join("presolve.contract.json");
        if !contract_path.is_file() {
            continue;
        }
        let source =
            fs::read_to_string(&contract_path).map_err(|error| ProjectDiscoveryErrorV1 {
                code: "PSDISC1010_PACKAGE_CONTRACT_READ_FAILED",
                message: error.to_string(),
            })?;
        let contract =
            parse_semantic_package_contract(&source).map_err(|error| ProjectDiscoveryErrorV1 {
                code: "PSDISC1011_PACKAGE_CONTRACT_INVALID",
                message: format!("{error:?}"),
            })?;
        for export in contract.exports.values() {
            let location = package_root.join(&export.runtime_module);
            if !location.is_file() {
                return Err(ProjectDiscoveryErrorV1 {
                    code: "PSDISC1012_PACKAGE_RUNTIME_MISSING",
                    message: location.display().to_string(),
                });
            }
            modules
                .insert(
                    SemanticPackageRuntimeModuleKey {
                        package: contract.package.clone(),
                        version: contract.version.clone(),
                        integrity: contract.integrity.clone(),
                        runtime_module: export.runtime_module.clone(),
                    },
                    location.to_string_lossy().into_owned(),
                )
                .map_err(|error| ProjectDiscoveryErrorV1 {
                    code: "PSDISC1013_PACKAGE_RUNTIME_INVALID",
                    message: format!("{error:?}"),
                })?;
        }
        contracts
            .insert(specifier.clone(), contract)
            .map_err(|error| ProjectDiscoveryErrorV1 {
                code: "PSDISC1011_PACKAGE_CONTRACT_INVALID",
                message: format!("{error:?}"),
            })?;
    }
    Ok((contracts, modules))
}

/// Discovers the default `app/` root, falling back to `src/`, in deterministic path order.
///
/// # Errors
/// Returns a stable error for unavailable roots or unreadable supported files.
pub fn discover_project_v1(root: &Path) -> Result<DiscoveredProjectV1, ProjectDiscoveryErrorV1> {
    let root = fs::canonicalize(root).map_err(|error| ProjectDiscoveryErrorV1 {
        code: "PSDISC1001_PROJECT_ROOT_UNAVAILABLE",
        message: error.to_string(),
    })?;
    let source_root = [root.join("app"), root.join("src")]
        .into_iter()
        .find(|path| path.is_dir())
        .ok_or_else(|| ProjectDiscoveryErrorV1 {
            code: "PSDISC1002_SOURCE_ROOT_MISSING",
            message: "expected app/ or src/ below project root".into(),
        })?;
    let mut paths = Vec::new();
    collect_sources(&source_root, &mut paths)?;
    paths.sort();
    if paths.is_empty() {
        return Err(ProjectDiscoveryErrorV1 {
            code: "PSDISC1003_SOURCE_SET_EMPTY",
            message: "default source root contains no .ts or .tsx sources".into(),
        });
    }
    let sources = paths
        .into_iter()
        .map(|path| {
            let source = fs::read_to_string(&path).map_err(|error| ProjectDiscoveryErrorV1 {
                code: "PSDISC1004_SOURCE_READ_FAILED",
                message: format!("{}: {error}", path.display()),
            })?;
            let logical_path = path
                .strip_prefix(&root)
                .expect("discovered source is contained")
                .to_path_buf();
            Ok(DiscoveredProjectSourceV1 {
                logical_path,
                source,
            })
        })
        .collect::<Result<Vec<_>, ProjectDiscoveryErrorV1>>()?;
    let fingerprint = Digest::sha256(
        sources
            .iter()
            .map(|source| format!("{}\n{}\n", source.logical_path.display(), source.source))
            .collect::<String>(),
    )
    .to_string();
    Ok(DiscoveredProjectV1 {
        root,
        source_root,
        fingerprint,
        sources,
    })
}

fn collect_sources(
    directory: &Path,
    paths: &mut Vec<PathBuf>,
) -> Result<(), ProjectDiscoveryErrorV1> {
    for entry in fs::read_dir(directory).map_err(|error| ProjectDiscoveryErrorV1 {
        code: "PSDISC1001_PROJECT_ROOT_UNAVAILABLE",
        message: error.to_string(),
    })? {
        let path = entry
            .map_err(|error| ProjectDiscoveryErrorV1 {
                code: "PSDISC1001_PROJECT_ROOT_UNAVAILABLE",
                message: error.to_string(),
            })?
            .path();
        if path.is_dir() {
            collect_sources(&path, paths)?;
        } else if matches!(
            path.extension().and_then(|value| value.to_str()),
            Some("ts") | Some("tsx")
        ) {
            paths.push(path);
        }
    }
    Ok(())
}
