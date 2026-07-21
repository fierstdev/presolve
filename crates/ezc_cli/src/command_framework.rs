//! L9-B command metadata and explicit project-envelope loading.
//!
//! The loader reads only the exact configuration path passed by its caller. It
//! does not enumerate source roots, discover a workspace, or compile sources.
#![allow(clippy::missing_errors_doc, clippy::missing_panics_doc)]

use std::fmt;
use std::fs;
use std::path::{Path, PathBuf};

use ezc_core::platform::WorkspaceConfiguration;

use crate::decode_cli_workspace_configuration_bytes_v1;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CliExitCodeV1 {
    Success = 0,
    CompilationFailure = 1,
    ConfigurationError = 2,
    WorkspaceError = 3,
    CompilerInternalError = 4,
    CacheError = 5,
    ToolingError = 6,
    UnexpectedPlatformError = 7,
}
impl CliExitCodeV1 {
    #[must_use]
    pub const fn as_i32(self) -> i32 {
        self as i32
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CliCommandV1 {
    Create,
    Dev,
    Build,
    Watch,
    Check,
    Clean,
    Explain,
    Inspect,
    Graph,
    Trace,
    Profile,
    Benchmark,
    Doctor,
    Cache,
    Workspace,
    Version,
}
impl CliCommandV1 {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Create => "create",
            Self::Dev => "dev",
            Self::Build => "build",
            Self::Watch => "watch",
            Self::Check => "check",
            Self::Clean => "clean",
            Self::Explain => "explain",
            Self::Inspect => "inspect",
            Self::Graph => "graph",
            Self::Trace => "trace",
            Self::Profile => "profile",
            Self::Benchmark => "benchmark",
            Self::Doctor => "doctor",
            Self::Cache => "cache",
            Self::Workspace => "workspace",
            Self::Version => "version",
        }
    }
}

#[must_use]
pub fn parse_cli_command_v1(value: &str) -> Option<CliCommandV1> {
    match value {
        "create" => Some(CliCommandV1::Create),
        "dev" => Some(CliCommandV1::Dev),
        "build" => Some(CliCommandV1::Build),
        "watch" => Some(CliCommandV1::Watch),
        "check" => Some(CliCommandV1::Check),
        "clean" => Some(CliCommandV1::Clean),
        "explain" => Some(CliCommandV1::Explain),
        "inspect" => Some(CliCommandV1::Inspect),
        "graph" => Some(CliCommandV1::Graph),
        "trace" => Some(CliCommandV1::Trace),
        "profile" => Some(CliCommandV1::Profile),
        "benchmark" => Some(CliCommandV1::Benchmark),
        "doctor" => Some(CliCommandV1::Doctor),
        "cache" => Some(CliCommandV1::Cache),
        "workspace" => Some(CliCommandV1::Workspace),
        "version" => Some(CliCommandV1::Version),
        _ => None,
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CliProjectEnvelopeV1 {
    pub project_root: PathBuf,
    pub configuration_path: PathBuf,
    pub configuration: WorkspaceConfiguration,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CliProjectEnvelopeErrorV1 {
    pub code: &'static str,
    pub message: String,
}
impl fmt::Display for CliProjectEnvelopeErrorV1 {
    fn fmt(&self, output: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(output, "{}: {}", self.code, self.message)
    }
}
impl std::error::Error for CliProjectEnvelopeErrorV1 {}

/// Loads one caller-named public configuration document. This is intentionally
/// not configuration discovery: neither `project_root` nor `source_roots` are
/// opened, scanned, globbed, or resolved here.
pub fn load_explicit_project_envelope_v1(
    project_root: &Path,
    configuration_path: &Path,
) -> Result<CliProjectEnvelopeV1, CliProjectEnvelopeErrorV1> {
    let bytes = fs::read(configuration_path).map_err(|error| CliProjectEnvelopeErrorV1 {
        code: "L9P001_CONFIGURATION_READ_FAILED",
        message: format!("failed to read {}: {error}", configuration_path.display()),
    })?;
    let configuration = decode_cli_workspace_configuration_bytes_v1(&bytes).map_err(|error| {
        CliProjectEnvelopeErrorV1 {
            code: "L9P002_CONFIGURATION_DECODE_FAILED",
            message: error.to_string(),
        }
    })?;
    Ok(CliProjectEnvelopeV1 {
        project_root: project_root.into(),
        configuration_path: configuration_path.into(),
        configuration,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn l9b_command_names_and_exit_codes_are_stable() {
        assert_eq!(parse_cli_command_v1("version"), Some(CliCommandV1::Version));
        assert_eq!(parse_cli_command_v1("unknown"), None);
        assert_eq!(CliExitCodeV1::ConfigurationError.as_i32(), 2);
        assert_eq!(CliCommandV1::Workspace.as_str(), "workspace");
    }
    #[test]
    fn l9b_loader_reads_only_the_explicit_configuration_document() {
        let root = std::env::temp_dir().join("presolve-l9b-envelope");
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(&root).unwrap();
        let config = root.join("chosen.json");
        fs::write(
            &config,
            include_bytes!("../fixtures/configuration/minimum-cli-v1.json"),
        )
        .unwrap();
        let envelope = load_explicit_project_envelope_v1(&root, &config).unwrap();
        assert_eq!(envelope.configuration, WorkspaceConfiguration::default());
        assert_eq!(envelope.configuration_path, config);
        fs::remove_dir_all(root).unwrap();
    }
}
