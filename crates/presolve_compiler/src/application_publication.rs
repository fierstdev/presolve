//! Phase P explicit multi-source application-publication request validation.
//!
//! This module deliberately stops before artifact lowering or filesystem
//! publication. It establishes one compiler-owned request/entry authority for
//! the later publication product.

use std::collections::BTreeSet;
use std::fmt;
use std::path::{Component, PathBuf};

use crate::platform::WorkspaceConfiguration;
use crate::semantic_package::SemanticPackageResolutionTable;
use crate::semantic_package_runtime::SemanticPackageRuntimeModuleTable;
use crate::{build_application_semantic_model_for_unit_with_packages, CompilationUnit, SemanticId};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ApplicationPublicationRequestV1 {
    pub configuration: WorkspaceConfiguration,
    pub unit: CompilationUnit,
    pub entry_path: PathBuf,
    pub package_contracts: SemanticPackageResolutionTable,
    pub package_runtime_modules: SemanticPackageRuntimeModuleTable,
    pub output_root: PathBuf,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ValidatedApplicationPublicationRequestV1 {
    pub request: ApplicationPublicationRequestV1,
    pub entry_component: SemanticId,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ApplicationPublicationRequestErrorV1 {
    pub code: &'static str,
    pub message: String,
}

impl fmt::Display for ApplicationPublicationRequestErrorV1 {
    fn fmt(&self, output: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(output, "{}: {}", self.code, self.message)
    }
}

impl std::error::Error for ApplicationPublicationRequestErrorV1 {}

/// Validates only caller-owned request identity and explicit entry selection.
/// Artifact generation is intentionally deferred to P2.
pub fn validate_application_publication_request_v1(
    request: ApplicationPublicationRequestV1,
) -> Result<ValidatedApplicationPublicationRequestV1, ApplicationPublicationRequestErrorV1> {
    if request.unit.is_empty() {
        return Err(ApplicationPublicationRequestErrorV1 {
            code: "PSAPP1001_EMPTY_SOURCE_SET",
            message: "application publication requires at least one explicit source".into(),
        });
    }
    if request.entry_path.as_os_str().is_empty()
        || request.entry_path.is_absolute()
        || request.entry_path.components().any(|component| {
            matches!(
                component,
                Component::ParentDir | Component::RootDir | Component::Prefix(_)
            )
        })
    {
        return Err(ApplicationPublicationRequestErrorV1 {
            code: "PSAPP1002_INVALID_ENTRY_PATH",
            message: "application entry path must be a non-empty relative logical path".into(),
        });
    }
    let paths = request
        .unit
        .files()
        .iter()
        .map(|file| file.path.clone())
        .collect::<BTreeSet<_>>();
    if paths.len() != request.unit.files().len() {
        return Err(ApplicationPublicationRequestErrorV1 {
            code: "PSAPP1003_DUPLICATE_LOGICAL_SOURCE",
            message: "application publication source logical paths must be unique".into(),
        });
    }
    if !paths.contains(&request.entry_path) {
        return Err(ApplicationPublicationRequestErrorV1 {
            code: "PSAPP1004_ENTRY_NOT_IN_SOURCE_SET",
            message: "application entry path must name one explicit source".into(),
        });
    }
    let model = build_application_semantic_model_for_unit_with_packages(
        &request.unit,
        &request.package_contracts,
    );
    let entries = model
        .components
        .iter()
        .filter(|component| {
            component.module_path == request.entry_path
                && component.element_name.is_some()
                && component.render.is_some()
        })
        .map(|component| component.id.clone())
        .collect::<Vec<_>>();
    let [entry_component] = entries.as_slice() else {
        return Err(ApplicationPublicationRequestErrorV1 {
            code: if entries.is_empty() {
                "PSAPP1005_ENTRY_APPLICATION_ROOT_MISSING"
            } else {
                "PSAPP1006_ENTRY_APPLICATION_ROOT_AMBIGUOUS"
            },
            message:
                "application entry source must declare exactly one supported rendered component"
                    .into(),
        });
    };
    Ok(ValidatedApplicationPublicationRequestV1 {
        request,
        entry_component: entry_component.clone(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn request(sources: Vec<(&str, &str)>, entry: &str) -> ApplicationPublicationRequestV1 {
        ApplicationPublicationRequestV1 {
            configuration: WorkspaceConfiguration::default(),
            unit: CompilationUnit::parse_sources(sources),
            entry_path: PathBuf::from(entry),
            package_contracts: SemanticPackageResolutionTable::default(),
            package_runtime_modules: SemanticPackageRuntimeModuleTable::default(),
            output_root: PathBuf::from("dist"),
        }
    }

    #[test]
    fn validates_one_explicit_rendered_component_entry_independent_of_source_order() {
        let source =
            r#"@component("x-app") class App extends Component { render() { return <main />; } }"#;
        let first = validate_application_publication_request_v1(request(
            vec![
                ("src/Utility.ts", "export const value = 1;"),
                ("src/App.tsx", source),
            ],
            "src/App.tsx",
        ))
        .unwrap();
        let second = validate_application_publication_request_v1(request(
            vec![
                ("src/App.tsx", source),
                ("src/Utility.ts", "export const value = 1;"),
            ],
            "src/App.tsx",
        ))
        .unwrap();
        assert_eq!(first.entry_component, second.entry_component);
    }

    #[test]
    fn rejects_missing_ambiguous_and_non_member_entries() {
        let missing = validate_application_publication_request_v1(request(
            vec![("src/Entry.tsx", "export const value = 1;")],
            "src/Entry.tsx",
        ))
        .unwrap_err();
        assert_eq!(missing.code, "PSAPP1005_ENTRY_APPLICATION_ROOT_MISSING");
        let ambiguous = validate_application_publication_request_v1(request(
            vec![("src/Entry.tsx", r#"@component("x-a") class A extends Component { render() { return <main />; } } @component("x-b") class B extends Component { render() { return <main />; } }"#)],
            "src/Entry.tsx",
        ))
        .unwrap_err();
        assert_eq!(ambiguous.code, "PSAPP1006_ENTRY_APPLICATION_ROOT_AMBIGUOUS");
        let non_member = validate_application_publication_request_v1(request(
            vec![("src/App.tsx", r#"@component("x-app") class App extends Component { render() { return <main />; } }"#)],
            "src/Missing.tsx",
        ))
        .unwrap_err();
        assert_eq!(non_member.code, "PSAPP1004_ENTRY_NOT_IN_SOURCE_SET");
    }
}
