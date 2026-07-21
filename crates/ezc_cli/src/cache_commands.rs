//! L9-E cache operations delegated to the frozen L6/L4 service boundary.
//!
//! Cache locations are derived only from an explicit configuration path. This
//! module neither searches for projects nor reads source files.

#![allow(clippy::missing_errors_doc, clippy::missing_panics_doc)]

use std::fmt;
use std::path::Path;

use ezc_core::persistent_cache::CacheInspectionReportV1;
use ezc_core::platform::ContractVersion;
use ezc_core::service::CompilerServiceHost;

use crate::load_explicit_project_envelope_v1;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CliCacheOperationV1 {
    Inspect,
    Verify,
    Clean,
}

impl CliCacheOperationV1 {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Inspect => "inspect",
            Self::Verify => "verify",
            Self::Clean => "clean",
        }
    }
}

#[derive(Debug, Clone)]
pub struct CliCacheOperationResultV1 {
    pub operation: CliCacheOperationV1,
    pub report: Option<CacheInspectionReportV1>,
    pub removed_keys: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CliCacheOperationErrorV1 {
    pub code: &'static str,
    pub message: String,
}

impl fmt::Display for CliCacheOperationErrorV1 {
    fn fmt(&self, output: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(output, "{}: {}", self.code, self.message)
    }
}

impl std::error::Error for CliCacheOperationErrorV1 {}

fn compiler_contract() -> ContractVersion {
    ContractVersion::new(format!("presolve-compiler:{}", env!("CARGO_PKG_VERSION")))
}

/// Operates on the sole L6 cache root associated with one caller-named public
/// configuration file. L6 owns cache validation and deletion semantics.
pub fn run_project_cache_operation_v1(
    configuration_path: &Path,
    operation: CliCacheOperationV1,
) -> Result<CliCacheOperationResultV1, CliCacheOperationErrorV1> {
    let project_root = configuration_path
        .parent()
        .ok_or(CliCacheOperationErrorV1 {
            code: "L9E001_CONFIGURATION_PATH_INVALID",
            message: "configuration path must have a parent directory".into(),
        })?;
    let envelope =
        load_explicit_project_envelope_v1(project_root, configuration_path).map_err(|error| {
            CliCacheOperationErrorV1 {
                code: error.code,
                message: error.message,
            }
        })?;
    let service_root = envelope.project_root.join(".presolve");
    let cache_root = service_root.join("cache");
    let service = CompilerServiceHost::start_with_cache(
        &service_root,
        Some(&cache_root),
        compiler_contract(),
    )
    .map_err(|error| CliCacheOperationErrorV1 {
        code: error.code,
        message: error.message,
    })?;
    match operation {
        CliCacheOperationV1::Inspect => {
            service
                .inspect_cache(&cache_root)
                .map(|report| CliCacheOperationResultV1 {
                    operation,
                    report: Some(report),
                    removed_keys: Vec::new(),
                })
        }
        CliCacheOperationV1::Verify => {
            service
                .verify_cache(&cache_root)
                .map(|report| CliCacheOperationResultV1 {
                    operation,
                    report: Some(report),
                    removed_keys: Vec::new(),
                })
        }
        CliCacheOperationV1::Clean => {
            service
                .clean_cache(&cache_root)
                .map(|removed_keys| CliCacheOperationResultV1 {
                    operation,
                    report: None,
                    removed_keys,
                })
        }
    }
    .map_err(|error| CliCacheOperationErrorV1 {
        code: error.code,
        message: error.message,
    })
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::sync::atomic::{AtomicU64, Ordering};

    use super::*;

    static NEXT_ROOT: AtomicU64 = AtomicU64::new(1);

    fn setup() -> (std::path::PathBuf, std::path::PathBuf) {
        let root = std::env::temp_dir().join(format!(
            "presolve-l9e-{}",
            NEXT_ROOT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir_all(&root).unwrap();
        let config = root.join("presolve.json");
        fs::write(
            &config,
            include_bytes!("../fixtures/configuration/minimum-cli-v1.json"),
        )
        .unwrap();
        (root, config)
    }

    #[test]
    fn l9e_inspects_and_verifies_only_the_configured_l6_cache_root() {
        let (root, config) = setup();
        fs::write(root.join("outside.txt"), "must survive").unwrap();
        let inspection = run_project_cache_operation_v1(&config, CliCacheOperationV1::Inspect)
            .unwrap()
            .report
            .unwrap();
        assert!(inspection.enabled);
        assert!(inspection.manifest_valid);
        let verification = run_project_cache_operation_v1(&config, CliCacheOperationV1::Verify)
            .unwrap()
            .report
            .unwrap();
        assert_eq!(
            inspection.to_canonical_json(),
            verification.to_canonical_json()
        );
        assert_eq!(
            fs::read_to_string(root.join("outside.txt")).unwrap(),
            "must survive"
        );
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn l9e_clean_delegates_to_l6_and_never_removes_project_siblings() {
        let (root, config) = setup();
        fs::write(root.join("outside.txt"), "must survive").unwrap();
        run_project_cache_operation_v1(&config, CliCacheOperationV1::Inspect).unwrap();
        let result = run_project_cache_operation_v1(&config, CliCacheOperationV1::Clean).unwrap();
        assert!(result.removed_keys.is_empty());
        assert!(root.join(".presolve/cache/entries").is_dir());
        assert_eq!(
            fs::read_to_string(root.join("outside.txt")).unwrap(),
            "must survive"
        );
        fs::remove_dir_all(root).unwrap();
    }
}
