//! Deterministic ergonomic project discovery for Phase R.

use std::fs;
use std::path::{Component, Path, PathBuf};

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
        if !is_npm_package_specifier(specifier) {
            continue;
        }
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
        if contract.package != *specifier {
            return Err(ProjectDiscoveryErrorV1 {
                code: "PSDISC1014_PACKAGE_SPECIFIER_MISMATCH",
                message: format!("{} declares package {}", specifier, contract.package),
            });
        }
        for export in contract.exports.values() {
            if !is_contained_runtime_module_path(&export.runtime_module) {
                return Err(ProjectDiscoveryErrorV1 {
                    code: "PSDISC1015_PACKAGE_RUNTIME_PATH_INVALID",
                    message: format!("{}: {}", specifier, export.runtime_module),
                });
            }
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

fn is_npm_package_specifier(specifier: &str) -> bool {
    let segments = specifier.split('/').collect::<Vec<_>>();
    match segments.as_slice() {
        [name] => valid_package_name_segment(name, false),
        [scope, name] => {
            valid_package_name_segment(scope, true) && valid_package_name_segment(name, false)
        }
        _ => false,
    }
}

fn valid_package_name_segment(value: &str, scoped: bool) -> bool {
    let value = if scoped {
        let Some(value) = value.strip_prefix('@') else {
            return false;
        };
        value
    } else {
        value
    };
    !value.is_empty()
        && value.chars().all(|character| {
            character.is_ascii_alphanumeric() || matches!(character, '-' | '_' | '.')
        })
}

fn is_contained_runtime_module_path(value: &str) -> bool {
    let path = Path::new(value);
    !path.is_absolute()
        && path
            .components()
            .all(|component| matches!(component, Component::Normal(_)))
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

#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::PathBuf;
    use std::sync::atomic::{AtomicU64, Ordering};

    use super::{discover_project_v1, discover_semantic_packages_v1};
    use crate::SemanticPackageRuntimeModuleKey;

    static NEXT_TEMPORARY_PROJECT: AtomicU64 = AtomicU64::new(1);

    fn temporary_project_root(label: &str) -> PathBuf {
        let sequence = NEXT_TEMPORARY_PROJECT.fetch_add(1, Ordering::Relaxed);
        let root = std::env::temp_dir().join(format!(
            "presolve-project-discovery-{label}-{}-{sequence}",
            std::process::id()
        ));
        fs::create_dir_all(&root).unwrap();
        root
    }

    #[test]
    fn discovers_app_sources_in_deterministic_logical_path_order() {
        let root = temporary_project_root("sources");
        fs::create_dir_all(root.join("app/routes/blog")).unwrap();
        fs::create_dir_all(root.join("src")).unwrap();
        fs::write(root.join("app/routes/index.tsx"), "export class Home {}\n").unwrap();
        fs::write(
            root.join("app/routes/blog/post.tsx"),
            "export class Post {}\n",
        )
        .unwrap();
        fs::write(root.join("src/ignored.tsx"), "export class Ignored {}\n").unwrap();

        let discovered = discover_project_v1(&root).unwrap();

        assert_eq!(
            discovered
                .sources
                .iter()
                .map(|source| source.logical_path.to_string_lossy().into_owned())
                .collect::<Vec<_>>(),
            vec!["app/routes/blog/post.tsx", "app/routes/index.tsx"]
        );
        assert_eq!(
            discovered.source_root,
            std::fs::canonicalize(root.join("app")).unwrap()
        );
        assert!(!discovered.fingerprint.is_empty());
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn resolves_only_exactly_named_contained_package_runtime_modules() {
        let root = temporary_project_root("packages");
        let package_root = root.join("node_modules/@acme/analytics");
        fs::create_dir_all(package_root.join("dist")).unwrap();
        fs::write(
            package_root.join("dist/track.js"),
            "export function track() {}\n",
        )
        .unwrap();
        fs::write(
            package_root.join("presolve.contract.json"),
            r#"{"schema_version":1,"package":"@acme/analytics","version":"1.0.0","integrity":"sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa","exports":{"track":{"kind":"opaque","type_signature":"() -> void","runtime_module":"dist/track.js","resume_policy":"cold_fallback","opaque_terminal":{"execution_boundary":"client","resume":"cold_fallback"}}}}"#,
        )
        .unwrap();

        let specifier = "@acme/analytics".to_string();
        let (contracts, modules) =
            discover_semantic_packages_v1(&root, std::slice::from_ref(&specifier)).unwrap();

        let contract = contracts.contract(&specifier).unwrap();
        assert_eq!(contract.package, specifier);
        assert_eq!(
            modules.resolve(&SemanticPackageRuntimeModuleKey {
                package: "@acme/analytics".into(),
                version: "1.0.0".into(),
                integrity: "sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa".into(),
                runtime_module: "dist/track.js".into(),
            }),
            Some(package_root.join("dist/track.js").to_string_lossy().as_ref())
        );
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn rejects_contract_runtime_paths_that_escape_the_declared_package() {
        let root = temporary_project_root("escape");
        let package_root = root.join("node_modules/date-kit");
        fs::create_dir_all(&package_root).unwrap();
        fs::write(
            package_root.join("presolve.contract.json"),
            r#"{"schema_version":1,"package":"date-kit","version":"1.0.0","integrity":"sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa","exports":{"format":{"kind":"pure","type_signature":"(Date) -> string","runtime_module":"../outside.js","resume_policy":"input_only"}}}"#,
        )
        .unwrap();

        let error = discover_semantic_packages_v1(&root, &["date-kit".into()]).unwrap_err();

        assert_eq!(error.code, "PSDISC1015_PACKAGE_RUNTIME_PATH_INVALID");
        fs::remove_dir_all(root).unwrap();
    }
}
