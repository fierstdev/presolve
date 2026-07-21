//! L9-D public build/check request loading over the L9-C service adapter.
//!
//! The command layer loads only explicit `logical=relative-path` source
//! specifications. It never enumerates source roots or infers membership.

#![allow(clippy::missing_errors_doc, clippy::missing_panics_doc)]

use std::fmt;
use std::fs;
use std::path::{Component, Path, PathBuf};

use ezc_core::persistent_cache::CacheReportSelector;
use ezc_core::service::IncrementalReportSelector;

use crate::{
    compile_complete_candidate_v1, load_explicit_project_envelope_v1, CliCompilationCandidateV1,
    CliCompilationResultV1, CliSourceInputV1,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CliExplicitSourceSpecV1 {
    pub logical_path: String,
    pub relative_path: PathBuf,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CliBuildCheckErrorV1 {
    pub code: &'static str,
    pub message: String,
}
impl fmt::Display for CliBuildCheckErrorV1 {
    fn fmt(&self, output: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(output, "{}: {}", self.code, self.message)
    }
}
impl std::error::Error for CliBuildCheckErrorV1 {}

fn parse_source_spec(value: &str) -> Result<CliExplicitSourceSpecV1, CliBuildCheckErrorV1> {
    let (logical_path, relative_path) = value.split_once('=').ok_or(CliBuildCheckErrorV1 {
        code: "L9D001_INVALID_SOURCE_SPEC",
        message: "--source must use logical-path=relative-path".into(),
    })?;
    if logical_path.is_empty() || relative_path.is_empty() {
        return Err(CliBuildCheckErrorV1 {
            code: "L9D001_INVALID_SOURCE_SPEC",
            message: "--source must name non-empty logical and relative paths".into(),
        });
    }
    let relative_path = PathBuf::from(relative_path);
    if relative_path.is_absolute()
        || relative_path.components().any(|component| {
            matches!(
                component,
                Component::ParentDir | Component::RootDir | Component::Prefix(_)
            )
        })
    {
        return Err(CliBuildCheckErrorV1 {
            code: "L9D002_SOURCE_PATH_ESCAPE",
            message: "--source host paths must be project-relative and contained".into(),
        });
    }
    Ok(CliExplicitSourceSpecV1 {
        logical_path: logical_path.into(),
        relative_path,
    })
}

pub fn parse_explicit_source_spec_v1(
    value: &str,
) -> Result<CliExplicitSourceSpecV1, CliBuildCheckErrorV1> {
    parse_source_spec(value)
}

fn load_sources(
    project_root: &Path,
    source_specs: &[CliExplicitSourceSpecV1],
) -> Result<Vec<CliSourceInputV1>, CliBuildCheckErrorV1> {
    if source_specs.is_empty() {
        return Err(CliBuildCheckErrorV1 {
            code: "L9D003_MISSING_SOURCE_INPUT",
            message: "build and check require at least one explicit --source input".into(),
        });
    }
    let root = fs::canonicalize(project_root).map_err(|error| CliBuildCheckErrorV1 {
        code: "L9D004_PROJECT_ROOT_UNAVAILABLE",
        message: format!("failed to resolve project root: {error}"),
    })?;
    let mut sources = Vec::with_capacity(source_specs.len());
    for spec in source_specs {
        let host_path = fs::canonicalize(root.join(&spec.relative_path)).map_err(|error| {
            CliBuildCheckErrorV1 {
                code: "L9D005_SOURCE_READ_FAILED",
                message: format!(
                    "failed to resolve {}: {error}",
                    spec.relative_path.display()
                ),
            }
        })?;
        if !host_path.starts_with(&root) || !host_path.is_file() {
            return Err(CliBuildCheckErrorV1 {
                code: "L9D002_SOURCE_PATH_ESCAPE",
                message: "explicit source must be a regular file inside the project root".into(),
            });
        }
        let content = fs::read_to_string(&host_path).map_err(|error| CliBuildCheckErrorV1 {
            code: "L9D005_SOURCE_READ_FAILED",
            message: format!("failed to read {}: {error}", spec.relative_path.display()),
        })?;
        sources.push(CliSourceInputV1 {
            logical_path: spec.logical_path.clone(),
            content,
        });
    }
    sources.sort_by(|left, right| left.logical_path.cmp(&right.logical_path));
    if sources
        .windows(2)
        .any(|pair| pair[0].logical_path == pair[1].logical_path)
    {
        return Err(CliBuildCheckErrorV1 {
            code: "L9D006_DUPLICATE_LOGICAL_SOURCE",
            message: "explicit logical source paths must be unique".into(),
        });
    }
    Ok(sources)
}

/// Loads only explicit project/source inputs and delegates the complete
/// candidate through the L9-C L4 adapter.
pub fn run_explicit_build_or_check_v1(
    configuration_path: &Path,
    source_specs: &[CliExplicitSourceSpecV1],
    verify_clean_equivalence: bool,
) -> Result<CliCompilationResultV1, CliBuildCheckErrorV1> {
    let project_root = configuration_path.parent().ok_or(CliBuildCheckErrorV1 {
        code: "L9D007_CONFIGURATION_PATH_INVALID",
        message: "configuration path must have a parent directory".into(),
    })?;
    let envelope =
        load_explicit_project_envelope_v1(project_root, configuration_path).map_err(|error| {
            CliBuildCheckErrorV1 {
                code: error.code,
                message: error.message,
            }
        })?;
    let sources = load_sources(&envelope.project_root, source_specs)?;
    compile_complete_candidate_v1(
        &envelope.project_root.join(".presolve"),
        CliCompilationCandidateV1 {
            configuration: envelope.configuration,
            sources,
            verify_clean_equivalence,
            report: IncrementalReportSelector::Summary,
            cache_report: CacheReportSelector::Summary,
        },
    )
    .map_err(|error| CliBuildCheckErrorV1 {
        code: error.code,
        message: error.message,
    })
}

#[cfg(test)]
mod tests {
    use std::sync::atomic::{AtomicU64, Ordering};

    use super::*;

    static NEXT_ROOT: AtomicU64 = AtomicU64::new(1);

    fn setup() -> (PathBuf, PathBuf) {
        let root = std::env::temp_dir().join(format!(
            "presolve-l9d-{}",
            NEXT_ROOT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir_all(root.join("src")).unwrap();
        let config = root.join("presolve.json");
        fs::write(
            &config,
            include_bytes!("../fixtures/configuration/minimum-cli-v1.json"),
        )
        .unwrap();
        fs::write(root.join("src/main.ts"), "export const value = 1;\n").unwrap();
        (root, config)
    }

    #[test]
    fn l9d_build_check_loads_only_explicit_contained_sources() {
        let (root, config) = setup();
        let result = run_explicit_build_or_check_v1(
            &config,
            &[parse_explicit_source_spec_v1("src/main.ts=src/main.ts").unwrap()],
            true,
        )
        .unwrap();
        assert_eq!(result.commit_sequence, 1);
        assert!(root.join(".presolve/service").is_dir());
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn l9d_rejects_escaping_and_missing_source_authority() {
        assert!(parse_explicit_source_spec_v1("src/main.ts=../main.ts").is_err());
        let (root, config) = setup();
        assert!(run_explicit_build_or_check_v1(&config, &[], false).is_err());
        fs::remove_dir_all(root).unwrap();
    }
}
