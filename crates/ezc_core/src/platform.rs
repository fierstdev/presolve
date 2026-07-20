//! Deterministic, process-local compiler platform products (Phase L3).
//!
//! This module deliberately orchestrates the existing parser and application
//! semantic model.  It neither defines a second parser nor reproduces semantic
//! ownership, artifact, or runtime derivation.

#![allow(
    clippy::case_sensitive_file_extension_comparisons,
    clippy::cast_possible_truncation,
    clippy::missing_errors_doc,
    clippy::needless_pass_by_value,
    clippy::semicolon_if_nothing_returned,
    clippy::too_many_lines
)]

use std::collections::{BTreeMap, BTreeSet};
use std::fmt;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Arc;

use sha2::{Digest as _, Sha256};

use crate::{build_application_semantic_model_for_unit, CompilationUnit};

pub const WORKSPACE_GRAPH_SCHEMA_VERSION: u32 = 1;
pub const WORKSPACE_SNAPSHOT_SCHEMA_VERSION: u32 = 1;
pub const COMPILER_SESSION_SCHEMA_VERSION: u32 = 1;
pub const INCREMENTAL_PLAN_SCHEMA_VERSION: u32 = 1;
pub const PRODUCT_CACHE_INSPECTION_SCHEMA_VERSION: u32 = 1;

static NEXT_SESSION_ID: AtomicU64 = AtomicU64::new(1);

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Digest(String);

impl Digest {
    #[must_use]
    pub fn sha256(bytes: impl AsRef<[u8]>) -> Self {
        Self(format!("sha256:{:x}", Sha256::digest(bytes.as_ref())))
    }

    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for Digest {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ContractVersion(String);
impl ContractVersion {
    #[must_use]
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}
impl fmt::Display for ContractVersion {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

macro_rules! typed_id {
    ($name:ident) => {
        #[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
        pub struct $name(String);
        impl $name {
            #[must_use]
            pub fn as_str(&self) -> &str {
                &self.0
            }
        }
        impl fmt::Display for $name {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                self.0.fmt(f)
            }
        }
    };
}
typed_id!(WorkspaceId);
typed_id!(SourceUnitId);
typed_id!(SourceRevisionId);
typed_id!(WorkspaceSnapshotId);
typed_id!(IncrementalPlanId);
typed_id!(ProductKey);
typed_id!(WorkspaceProductKey);
typed_id!(CompilerSessionId);
typed_id!(CompilationAttemptId);

fn hashed_id(prefix: &str, domain: &str, fields: &[&[u8]]) -> String {
    let mut bytes = Vec::new();
    bytes.extend_from_slice(domain.as_bytes());
    for field in fields {
        bytes.push(0);
        bytes.extend_from_slice(field);
    }
    format!("{prefix}:{}", Digest::sha256(bytes))
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct WorkspacePath(String);

impl WorkspacePath {
    pub fn new(path: impl AsRef<str>) -> Result<Self, PlatformFailure> {
        let raw = path.as_ref();
        if raw.is_empty()
            || raw.contains('\0')
            || raw.starts_with('/')
            || raw.starts_with('\\')
            || is_windows_absolute(raw)
        {
            return Err(PlatformFailure::path(
                "workspace path must be non-empty and relative",
                raw,
            ));
        }
        let mut parts = Vec::new();
        let separated = raw.replace('\\', "/");
        for part in separated.split('/') {
            match part {
                "" | "." => {}
                ".." => {
                    if parts.pop().is_none() {
                        return Err(PlatformFailure::path(
                            "workspace path escapes its root",
                            raw,
                        ));
                    }
                }
                value => parts.push(value),
            }
        }
        if parts.is_empty() {
            return Err(PlatformFailure::path(
                "workspace path must name a file",
                raw,
            ));
        }
        Ok(Self(parts.join("/")))
    }
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}
impl fmt::Display for WorkspacePath {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}
fn is_windows_absolute(path: &str) -> bool {
    path.len() > 2 && path.as_bytes()[1] == b':' && path.as_bytes()[2] == b'\\'
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum SourceLanguage {
    TypeScript,
    TypeScriptJsx,
    JavaScript,
    JavaScriptJsx,
}
impl SourceLanguage {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::TypeScript => "typescript",
            Self::TypeScriptJsx => "typescript_jsx",
            Self::JavaScript => "javascript",
            Self::JavaScriptJsx => "javascript_jsx",
        }
    }
    fn from_path(path: &WorkspacePath) -> Result<Self, PlatformFailure> {
        if path.0.ends_with(".tsx") {
            Ok(Self::TypeScriptJsx)
        } else if path.0.ends_with(".ts") {
            Ok(Self::TypeScript)
        } else if path.0.ends_with(".jsx") {
            Ok(Self::JavaScriptJsx)
        } else if path.0.ends_with(".js") {
            Ok(Self::JavaScript)
        } else {
            Err(PlatformFailure::new(
                PlatformFailureCode::UnsupportedSourceLanguage,
                "only .ts, .tsx, .js, and .jsx workspace inputs are supported",
            ))
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum ProductKind {
    Parse,
    SourceSemantic,
    ComponentGraph,
    ApplicationSemanticModel,
    LoweredProgram,
    HtmlArtifact,
    RuntimeArtifact,
    TemplateManifest,
    ResumeManifest,
    InspectionArtifact,
    DiagnosticSet,
}
impl ProductKind {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Parse => "parse",
            Self::SourceSemantic => "source_semantic",
            Self::ComponentGraph => "component_graph",
            Self::ApplicationSemanticModel => "application_semantic_model",
            Self::LoweredProgram => "lowered_program",
            Self::HtmlArtifact => "html_artifact",
            Self::RuntimeArtifact => "runtime_artifact",
            Self::TemplateManifest => "template_manifest",
            Self::ResumeManifest => "resume_manifest",
            Self::InspectionArtifact => "inspection_artifact",
            Self::DiagnosticSet => "diagnostic_set",
        }
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum ApplicationProductKind {
    ApplicationSemanticModel,
    ApplicationDiagnostics,
    BuildManifest,
}
impl ApplicationProductKind {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::ApplicationSemanticModel => "application_semantic_model",
            Self::ApplicationDiagnostics => "application_diagnostics",
            Self::BuildManifest => "build_manifest",
        }
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum WorkspaceDependencyKind {
    Compile,
    Semantic,
    Artifact,
}
impl WorkspaceDependencyKind {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Compile => "compile",
            Self::Semantic => "semantic",
            Self::Artifact => "artifact",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorkspaceConfiguration {
    pub source_roots: Vec<WorkspacePath>,
    pub feature_flags: Vec<String>,
    pub target_profile: String,
    pub platform_options: Vec<(String, String)>,
}
impl Default for WorkspaceConfiguration {
    fn default() -> Self {
        Self {
            source_roots: vec![WorkspacePath("src".into())],
            feature_flags: Vec::new(),
            target_profile: "default".into(),
            platform_options: Vec::new(),
        }
    }
}
impl WorkspaceConfiguration {
    fn canonical(&self) -> String {
        let roots = self
            .source_roots
            .iter()
            .map(ToString::to_string)
            .collect::<Vec<_>>();
        let mut flags = self.feature_flags.clone();
        flags.sort();
        flags.dedup();
        let mut options = self.platform_options.clone();
        options.sort();
        options.dedup();
        format!(
            "roots={}\\nflags={}\\ntarget={}\\noptions={}",
            roots.join(","),
            flags.join(","),
            self.target_profile,
            options
                .into_iter()
                .map(|(k, v)| format!("{k}={v}"))
                .collect::<Vec<_>>()
                .join(",")
        )
    }
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorkspaceSource {
    pub path: String,
    pub source: String,
    pub language: Option<SourceLanguage>,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorkspaceInput {
    pub configuration: WorkspaceConfiguration,
    pub sources: Vec<WorkspaceSource>,
    pub compiler_contract: ContractVersion,
}
impl WorkspaceInput {
    #[must_use]
    pub fn new(sources: Vec<WorkspaceSource>) -> Self {
        Self {
            configuration: WorkspaceConfiguration::default(),
            sources,
            compiler_contract: ContractVersion::new(format!(
                "presolve-compiler:{}",
                env!("CARGO_PKG_VERSION")
            )),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct NormalizedInput {
    path: WorkspacePath,
    source: String,
    language: SourceLanguage,
    source_unit_id: SourceUnitId,
    source_revision_id: SourceRevisionId,
    source_digest: Digest,
}
fn workspace_id(configuration: &WorkspaceConfiguration) -> WorkspaceId {
    WorkspaceId(hashed_id(
        "workspace",
        "workspace-config-v1",
        &[configuration.canonical().as_bytes()],
    ))
}
fn source_unit_id(workspace: &WorkspaceId, path: &WorkspacePath) -> SourceUnitId {
    SourceUnitId(hashed_id(
        "source",
        "source-unit-v1",
        &[workspace.as_str().as_bytes(), path.as_str().as_bytes()],
    ))
}
fn source_revision_id(
    unit: &SourceUnitId,
    source: &str,
    language: SourceLanguage,
) -> SourceRevisionId {
    SourceRevisionId(hashed_id(
        "revision",
        "source-revision-v1",
        &[
            unit.as_str().as_bytes(),
            source.len().to_string().as_bytes(),
            source.as_bytes(),
            language.as_str().as_bytes(),
        ],
    ))
}
fn normalize(
    input: &WorkspaceInput,
) -> Result<(WorkspaceId, Digest, Vec<NormalizedInput>), PlatformFailure> {
    let workspace = workspace_id(&input.configuration);
    let configuration_fingerprint = Digest::sha256(input.configuration.canonical());
    let mut seen = BTreeSet::new();
    let mut sources = Vec::new();
    for source in &input.sources {
        let path = WorkspacePath::new(&source.path)?;
        if !seen.insert(path.clone()) {
            return Err(PlatformFailure {
                code: PlatformFailureCode::DuplicateWorkspacePath,
                message: format!("duplicate normalized workspace path: {path}"),
                workspace_path: Some(path),
                source_unit_id: None,
                product_key: None,
            });
        }
        let language = match source.language {
            Some(language) => language,
            None => SourceLanguage::from_path(&path)?,
        };
        let unit = source_unit_id(&workspace, &path);
        let revision = source_revision_id(&unit, &source.source, language);
        sources.push(NormalizedInput {
            path,
            source: source.source.clone(),
            language,
            source_unit_id: unit,
            source_revision_id: revision,
            source_digest: Digest::sha256(source.source.as_bytes()),
        });
    }
    sources.sort_by(|a, b| a.path.cmp(&b.path));
    Ok((workspace, configuration_fingerprint, sources))
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SnapshotUnit {
    pub source_unit_id: SourceUnitId,
    pub path: WorkspacePath,
    pub source_revision_id: SourceRevisionId,
    pub source_digest: Digest,
    pub source_length: u64,
    pub language: SourceLanguage,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorkspaceSnapshot {
    pub schema_version: u32,
    pub workspace_id: WorkspaceId,
    pub snapshot_id: WorkspaceSnapshotId,
    pub compiler_contract: ContractVersion,
    pub configuration_fingerprint: Digest,
    pub units: Vec<SnapshotUnit>,
}
impl WorkspaceSnapshot {
    pub fn from_input(input: &WorkspaceInput) -> Result<Self, PlatformFailure> {
        let (workspace, fingerprint, inputs) = normalize(input)?;
        Ok(Self::from_normalized(
            workspace,
            fingerprint,
            input.compiler_contract.clone(),
            &inputs,
        ))
    }
    fn from_normalized(
        workspace_id: WorkspaceId,
        configuration_fingerprint: Digest,
        compiler_contract: ContractVersion,
        inputs: &[NormalizedInput],
    ) -> Self {
        let units = inputs
            .iter()
            .map(|input| SnapshotUnit {
                source_unit_id: input.source_unit_id.clone(),
                path: input.path.clone(),
                source_revision_id: input.source_revision_id.clone(),
                source_digest: input.source_digest.clone(),
                source_length: input.source.len() as u64,
                language: input.language,
            })
            .collect::<Vec<_>>();
        let identity = canonical_snapshot_identity(&workspace_id, &compiler_contract, &units);
        Self {
            schema_version: WORKSPACE_SNAPSHOT_SCHEMA_VERSION,
            workspace_id,
            snapshot_id: WorkspaceSnapshotId(hashed_id(
                "snapshot",
                "workspace-snapshot-v1",
                &[identity.as_bytes()],
            )),
            compiler_contract,
            configuration_fingerprint,
            units,
        }
    }
    pub fn validate(&self) -> Result<(), PlatformFailure> {
        if self.schema_version != WORKSPACE_SNAPSHOT_SCHEMA_VERSION {
            return Err(PlatformFailure::new(
                PlatformFailureCode::UnsupportedSchemaVersion,
                "unsupported workspace snapshot schema version",
            ));
        }
        let mut previous = None;
        for unit in &self.units {
            if previous
                .as_ref()
                .is_some_and(|value: &WorkspacePath| value >= &unit.path)
            {
                return Err(PlatformFailure::new(
                    PlatformFailureCode::InvalidWorkspaceSnapshot,
                    "snapshot units are not uniquely ordered by path",
                ));
            }
            if source_unit_id(&self.workspace_id, &unit.path) != unit.source_unit_id {
                return Err(PlatformFailure::new(
                    PlatformFailureCode::InvalidWorkspaceSnapshot,
                    "snapshot source-unit identity mismatch",
                ));
            }
            previous = Some(unit.path.clone());
        }
        let expected = WorkspaceSnapshotId(hashed_id(
            "snapshot",
            "workspace-snapshot-v1",
            &[canonical_snapshot_identity(
                &self.workspace_id,
                &self.compiler_contract,
                &self.units,
            )
            .as_bytes()],
        ));
        if expected != self.snapshot_id {
            return Err(PlatformFailure::new(
                PlatformFailureCode::InvalidWorkspaceSnapshot,
                "snapshot identity mismatch",
            ));
        }
        Ok(())
    }
    pub fn to_canonical_json(&self) -> Result<Vec<u8>, PlatformSerializationError> {
        self.validate()
            .map_err(PlatformSerializationError::validation)?;
        Ok(json_document(snapshot_json(self)))
    }
}
fn canonical_snapshot_identity(
    workspace: &WorkspaceId,
    contract: &ContractVersion,
    units: &[SnapshotUnit],
) -> String {
    let units = units
        .iter()
        .map(|unit| {
            format!(
                "{{\"source_unit_id\":{},\"source_revision_id\":{}}}",
                quote(unit.source_unit_id.as_str()),
                quote(unit.source_revision_id.as_str())
            )
        })
        .collect::<Vec<_>>()
        .join(",");
    format!(
        "{{\"workspace_id\":{},\"compiler_contract\":{},\"units\":[{units}]}}",
        quote(workspace.as_str()),
        quote(contract.as_str())
    )
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UnitProductRef {
    pub product_kind: ProductKind,
    pub product_key: ProductKey,
    pub producer_contract: ContractVersion,
    pub content_digest: Digest,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorkspaceUnit {
    pub source_unit_id: SourceUnitId,
    pub path: WorkspacePath,
    pub source_revision_id: SourceRevisionId,
    pub source_digest: Digest,
    pub source_length: u64,
    pub language: SourceLanguage,
    pub products: Vec<UnitProductRef>,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorkspaceDependencyEdge {
    pub source: SourceUnitId,
    pub kind: WorkspaceDependencyKind,
    pub target: SourceUnitId,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ApplicationProductRef {
    pub product_kind: ApplicationProductKind,
    pub product_key: WorkspaceProductKey,
    pub producer_contract: ContractVersion,
    pub content_digest: Digest,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorkspaceGraph {
    pub schema_version: u32,
    pub workspace_id: WorkspaceId,
    pub snapshot_id: WorkspaceSnapshotId,
    pub compiler_contract: ContractVersion,
    pub units: Vec<WorkspaceUnit>,
    pub dependency_edges: Vec<WorkspaceDependencyEdge>,
    pub application_products: Vec<ApplicationProductRef>,
}
impl WorkspaceGraph {
    #[must_use]
    pub fn unit(&self, id: &SourceUnitId) -> Option<&WorkspaceUnit> {
        self.units.iter().find(|unit| &unit.source_unit_id == id)
    }
    #[must_use]
    pub fn unit_by_path(&self, path: &WorkspacePath) -> Option<&WorkspaceUnit> {
        self.units.iter().find(|unit| &unit.path == path)
    }
    pub fn direct_dependencies<'a>(
        &'a self,
        id: &'a SourceUnitId,
    ) -> impl Iterator<Item = &'a WorkspaceDependencyEdge> + 'a {
        self.dependency_edges
            .iter()
            .filter(move |edge| &edge.source == id)
    }
    pub fn direct_dependents<'a>(
        &'a self,
        id: &'a SourceUnitId,
    ) -> impl Iterator<Item = &'a WorkspaceDependencyEdge> + 'a {
        self.dependency_edges
            .iter()
            .filter(move |edge| &edge.target == id)
    }
    pub fn validate(&self) -> Result<(), PlatformFailure> {
        if self.schema_version != WORKSPACE_GRAPH_SCHEMA_VERSION {
            return Err(PlatformFailure::new(
                PlatformFailureCode::UnsupportedSchemaVersion,
                "unsupported workspace graph schema version",
            ));
        }
        let mut paths = BTreeSet::new();
        let mut ids = BTreeSet::new();
        let mut prior = None;
        for unit in &self.units {
            if !paths.insert(unit.path.clone())
                || !ids.insert(unit.source_unit_id.clone())
                || prior
                    .as_ref()
                    .is_some_and(|path: &WorkspacePath| path >= &unit.path)
            {
                return Err(PlatformFailure::new(
                    PlatformFailureCode::InvalidWorkspaceGraph,
                    "workspace graph units must be unique and ordered",
                ));
            }
            if source_unit_id(&self.workspace_id, &unit.path) != unit.source_unit_id {
                return Err(PlatformFailure::new(
                    PlatformFailureCode::InvalidWorkspaceGraph,
                    "workspace graph source-unit identity mismatch",
                ));
            }
            let mut kinds = BTreeSet::new();
            for product in &unit.products {
                if !kinds.insert(product.product_kind)
                    || !product.product_key.as_str().starts_with("product:sha256:")
                {
                    return Err(PlatformFailure::new(
                        PlatformFailureCode::InvalidWorkspaceGraph,
                        "workspace graph product key mismatch",
                    ));
                }
            }
            prior = Some(unit.path.clone());
        }
        let mut edges = BTreeSet::new();
        let mut previous = None;
        for edge in &self.dependency_edges {
            if !ids.contains(&edge.source)
                || !ids.contains(&edge.target)
                || !edges.insert((edge.source.clone(), edge.kind, edge.target.clone()))
                || previous.as_ref().is_some_and(
                    |value: &(SourceUnitId, WorkspaceDependencyKind, SourceUnitId)| {
                        value >= &(edge.source.clone(), edge.kind, edge.target.clone())
                    },
                )
            {
                return Err(PlatformFailure::new(
                    PlatformFailureCode::InvalidWorkspaceGraph,
                    "workspace graph edges must be valid, unique, and ordered",
                ));
            }
            previous = Some((edge.source.clone(), edge.kind, edge.target.clone()));
        }
        Ok(())
    }
    pub fn to_canonical_json(&self) -> Result<Vec<u8>, PlatformSerializationError> {
        self.validate()
            .map_err(PlatformSerializationError::validation)?;
        Ok(json_document(graph_json(self)))
    }
}
fn product_key(
    unit: &SourceUnitId,
    revision: &SourceRevisionId,
    kind: ProductKind,
    contract: &ContractVersion,
    configuration: &Digest,
) -> ProductKey {
    ProductKey(hashed_id(
        "product",
        "product-key-v1",
        &[
            unit.as_str().as_bytes(),
            revision.as_str().as_bytes(),
            kind.as_str().as_bytes(),
            contract.as_str().as_bytes(),
            configuration.as_str().as_bytes(),
        ],
    ))
}
fn workspace_product_key(
    workspace: &WorkspaceId,
    snapshot: &WorkspaceSnapshotId,
    kind: ApplicationProductKind,
    contract: &ContractVersion,
    configuration: &Digest,
) -> WorkspaceProductKey {
    WorkspaceProductKey(hashed_id(
        "workspace-product",
        "workspace-product-key-v1",
        &[
            workspace.as_str().as_bytes(),
            snapshot.as_str().as_bytes(),
            kind.as_str().as_bytes(),
            contract.as_str().as_bytes(),
            configuration.as_str().as_bytes(),
        ],
    ))
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorkspaceChangeSet {
    pub baseline_snapshot_id: WorkspaceSnapshotId,
    pub candidate_snapshot_id: WorkspaceSnapshotId,
    pub configuration_changed: bool,
    pub added: Vec<SourceUnitId>,
    pub removed: Vec<SourceUnitId>,
    pub modified: Vec<SourceUnitId>,
    pub unchanged: Vec<SourceUnitId>,
}
#[must_use]
pub fn compare_snapshots(
    baseline: &WorkspaceSnapshot,
    candidate: &WorkspaceSnapshot,
) -> WorkspaceChangeSet {
    let old = baseline
        .units
        .iter()
        .map(|unit| (unit.source_unit_id.clone(), unit))
        .collect::<BTreeMap<_, _>>();
    let new = candidate
        .units
        .iter()
        .map(|unit| (unit.source_unit_id.clone(), unit))
        .collect::<BTreeMap<_, _>>();
    let mut added = Vec::new();
    let mut removed = Vec::new();
    let mut modified = Vec::new();
    let mut unchanged = Vec::new();
    for unit in &candidate.units {
        match old.get(&unit.source_unit_id) {
            None => added.push(unit.source_unit_id.clone()),
            Some(previous) if previous.source_revision_id != unit.source_revision_id => {
                modified.push(unit.source_unit_id.clone())
            }
            Some(_) => unchanged.push(unit.source_unit_id.clone()),
        }
    }
    for unit in &baseline.units {
        if !new.contains_key(&unit.source_unit_id) {
            removed.push(unit.source_unit_id.clone());
        }
    }
    WorkspaceChangeSet {
        baseline_snapshot_id: baseline.snapshot_id.clone(),
        candidate_snapshot_id: candidate.snapshot_id.clone(),
        configuration_changed: baseline.workspace_id != candidate.workspace_id
            || baseline.configuration_fingerprint != candidate.configuration_fingerprint
            || baseline.compiler_contract != candidate.compiler_contract,
        added,
        removed,
        modified,
        unchanged,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IncrementalMode {
    NoOp,
    Incremental,
    Full,
}
impl IncrementalMode {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::NoOp => "no_op",
            Self::Incremental => "incremental",
            Self::Full => "full",
        }
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum InvalidationReason {
    Added,
    Modified,
    RemovedDependency,
    CompileDependent,
    SemanticDependent,
    ArtifactDependent,
    ConfigurationChanged,
    CompilerContractChanged,
}
impl InvalidationReason {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Added => "added",
            Self::Modified => "modified",
            Self::RemovedDependency => "removed_dependency",
            Self::CompileDependent => "compile_dependent",
            Self::SemanticDependent => "semantic_dependent",
            Self::ArtifactDependent => "artifact_dependent",
            Self::ConfigurationChanged => "configuration_changed",
            Self::CompilerContractChanged => "compiler_contract_changed",
        }
    }
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InvalidatedUnit {
    pub source_unit_id: SourceUnitId,
    pub reason: InvalidationReason,
    pub caused_by: Vec<SourceUnitId>,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IncrementalStageKind {
    Remove,
    Parse,
    Analyze,
    AssembleApplicationModel,
    Lower,
    Emit,
    Validate,
    Commit,
}
impl IncrementalStageKind {
    const fn as_str(self) -> &'static str {
        match self {
            Self::Remove => "remove",
            Self::Parse => "parse",
            Self::Analyze => "analyze",
            Self::AssembleApplicationModel => "assemble_application_model",
            Self::Lower => "lower",
            Self::Emit => "emit",
            Self::Validate => "validate",
            Self::Commit => "commit",
        }
    }
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IncrementalStage {
    pub stage_index: u32,
    pub kind: IncrementalStageKind,
    pub units: Vec<SourceUnitId>,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExpectedCommit {
    pub snapshot_id: WorkspaceSnapshotId,
    pub workspace_graph_schema_version: u32,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IncrementalPlan {
    pub schema_version: u32,
    pub plan_id: IncrementalPlanId,
    pub baseline_snapshot_id: WorkspaceSnapshotId,
    pub candidate_snapshot_id: WorkspaceSnapshotId,
    pub mode: IncrementalMode,
    pub changes: WorkspaceChangeSet,
    pub invalidated_units: Vec<InvalidatedUnit>,
    pub stages: Vec<IncrementalStage>,
    pub retained_products: Vec<ProductKey>,
    pub discarded_products: Vec<ProductKey>,
    pub expected_commit: ExpectedCommit,
}
impl IncrementalPlan {
    pub fn to_canonical_json(&self) -> Result<Vec<u8>, PlatformSerializationError> {
        if self.schema_version != INCREMENTAL_PLAN_SCHEMA_VERSION {
            return Err(PlatformSerializationError {
                message: "unsupported incremental plan schema version".into(),
            });
        }
        Ok(json_document(incremental_plan_json(self)))
    }
}
#[must_use]
pub fn plan_incremental(
    baseline: Option<(&WorkspaceSnapshot, &WorkspaceGraph)>,
    candidate: &WorkspaceSnapshot,
    forced_full: bool,
) -> IncrementalPlan {
    let empty_id = WorkspaceSnapshotId(
        "snapshot:sha256:0000000000000000000000000000000000000000000000000000000000000000".into(),
    );
    let Some((before, graph)) = baseline else {
        return build_full_plan(
            empty_id.clone(),
            candidate,
            WorkspaceChangeSet {
                baseline_snapshot_id: empty_id.clone(),
                candidate_snapshot_id: candidate.snapshot_id.clone(),
                configuration_changed: true,
                added: candidate
                    .units
                    .iter()
                    .map(|u| u.source_unit_id.clone())
                    .collect(),
                removed: Vec::new(),
                modified: Vec::new(),
                unchanged: Vec::new(),
            },
            candidate,
        );
    };
    let changes = compare_snapshots(before, candidate);
    if forced_full || changes.configuration_changed || graph.validate().is_err() {
        return build_full_plan(before.snapshot_id.clone(), candidate, changes, candidate);
    }
    if before.snapshot_id == candidate.snapshot_id {
        return build_plan(
            before.snapshot_id.clone(),
            candidate,
            changes,
            IncrementalMode::NoOp,
            Vec::new(),
            Vec::new(),
            Vec::new(),
        );
    }
    let candidate_paths = candidate
        .units
        .iter()
        .map(|u| (u.source_unit_id.clone(), u.path.clone()))
        .collect::<BTreeMap<_, _>>();
    let mut causes = BTreeMap::<SourceUnitId, BTreeSet<SourceUnitId>>::new();
    let mut reasons = BTreeMap::<SourceUnitId, InvalidationReason>::new();
    for id in &changes.added {
        causes.entry(id.clone()).or_default().insert(id.clone());
        reasons.insert(id.clone(), InvalidationReason::Added);
    }
    for id in &changes.modified {
        causes.entry(id.clone()).or_default().insert(id.clone());
        reasons.insert(id.clone(), InvalidationReason::Modified);
    }
    let mut frontier = changes.modified.clone();
    frontier.extend(changes.removed.clone());
    while let Some(changed) = frontier.pop() {
        for edge in graph.direct_dependents(&changed) {
            if !candidate_paths.contains_key(&edge.source) {
                continue;
            }
            let reason = match edge.kind {
                WorkspaceDependencyKind::Compile => InvalidationReason::CompileDependent,
                WorkspaceDependencyKind::Semantic => InvalidationReason::SemanticDependent,
                WorkspaceDependencyKind::Artifact => InvalidationReason::ArtifactDependent,
            };
            let added = causes
                .entry(edge.source.clone())
                .or_default()
                .insert(changed.clone());
            reasons.entry(edge.source.clone()).or_insert(reason);
            if added {
                frontier.push(edge.source.clone());
            }
        }
    }
    let mut invalidated = causes
        .into_iter()
        .map(|(id, causes)| InvalidatedUnit {
            reason: reasons[&id],
            source_unit_id: id,
            caused_by: causes.into_iter().collect(),
        })
        .collect::<Vec<_>>();
    invalidated.sort_by_key(|item| candidate_paths.get(&item.source_unit_id).cloned());
    let impacted = invalidated
        .iter()
        .map(|item| item.source_unit_id.clone())
        .collect::<Vec<_>>();
    let discarded = graph
        .units
        .iter()
        .filter(|unit| {
            impacted.contains(&unit.source_unit_id)
                || changes.removed.contains(&unit.source_unit_id)
        })
        .flat_map(|unit| {
            unit.products
                .iter()
                .map(|product| product.product_key.clone())
        })
        .collect::<Vec<_>>();
    build_plan(
        before.snapshot_id.clone(),
        candidate,
        changes,
        IncrementalMode::Incremental,
        invalidated,
        impacted,
        discarded,
    )
}
fn build_full_plan(
    baseline: WorkspaceSnapshotId,
    candidate: &WorkspaceSnapshot,
    changes: WorkspaceChangeSet,
    _snapshot: &WorkspaceSnapshot,
) -> IncrementalPlan {
    let invalidated = candidate
        .units
        .iter()
        .map(|unit| InvalidatedUnit {
            source_unit_id: unit.source_unit_id.clone(),
            reason: InvalidationReason::ConfigurationChanged,
            caused_by: Vec::new(),
        })
        .collect();
    build_plan(
        baseline,
        candidate,
        changes,
        IncrementalMode::Full,
        invalidated,
        candidate
            .units
            .iter()
            .map(|u| u.source_unit_id.clone())
            .collect(),
        Vec::new(),
    )
}
fn build_plan(
    baseline: WorkspaceSnapshotId,
    candidate: &WorkspaceSnapshot,
    changes: WorkspaceChangeSet,
    mode: IncrementalMode,
    invalidated: Vec<InvalidatedUnit>,
    impacted: Vec<SourceUnitId>,
    mut discarded: Vec<ProductKey>,
) -> IncrementalPlan {
    let mut stages = Vec::new();
    let mut push = |kind, units: Vec<SourceUnitId>| {
        if !units.is_empty()
            || matches!(
                kind,
                IncrementalStageKind::Validate
                    | IncrementalStageKind::Commit
                    | IncrementalStageKind::AssembleApplicationModel
            )
        {
            let index = stages.len() as u32;
            stages.push(IncrementalStage {
                stage_index: index,
                kind,
                units,
            });
        }
    };
    push(IncrementalStageKind::Remove, changes.removed.clone());
    if !matches!(mode, IncrementalMode::NoOp) {
        push(IncrementalStageKind::Parse, impacted.clone());
        push(IncrementalStageKind::Analyze, impacted.clone());
        push(IncrementalStageKind::AssembleApplicationModel, Vec::new());
        push(IncrementalStageKind::Lower, impacted.clone());
        push(IncrementalStageKind::Emit, impacted.clone());
    }
    push(IncrementalStageKind::Validate, Vec::new());
    push(IncrementalStageKind::Commit, Vec::new());
    discarded.sort();
    discarded.dedup();
    let identity = format!(
        "{}|{}|{}|{:?}",
        baseline,
        candidate.snapshot_id,
        mode.as_str(),
        stages
    );
    let plan_id = IncrementalPlanId(hashed_id(
        "incremental-plan",
        "incremental-plan-v1",
        &[identity.as_bytes()],
    ));
    IncrementalPlan {
        schema_version: INCREMENTAL_PLAN_SCHEMA_VERSION,
        plan_id,
        baseline_snapshot_id: baseline,
        candidate_snapshot_id: candidate.snapshot_id.clone(),
        mode,
        changes,
        invalidated_units: invalidated,
        stages,
        retained_products: Vec::new(),
        discarded_products: discarded,
        expected_commit: ExpectedCommit {
            snapshot_id: candidate.snapshot_id.clone(),
            workspace_graph_schema_version: WORKSPACE_GRAPH_SCHEMA_VERSION,
        },
    }
}

pub trait CachedProduct: Send + Sync {
    fn kind(&self) -> ProductKind;
    fn digest(&self) -> &Digest;
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CacheEntryState {
    AttemptLocal,
    Committed,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CacheLimits {
    pub maximum_entries: usize,
    pub maximum_weight: u64,
}
impl Default for CacheLimits {
    fn default() -> Self {
        Self {
            maximum_entries: 1024,
            maximum_weight: 268_435_456,
        }
    }
}
pub struct CacheEntry {
    pub key: ProductKey,
    pub content_digest: Digest,
    pub state: CacheEntryState,
    pub weight: u64,
    pub last_access: u64,
    pub product: Arc<dyn CachedProduct>,
}
pub struct ProductCache {
    entries: BTreeMap<ProductKey, CacheEntry>,
    total_weight: u64,
    limits: CacheLimits,
    next_access: u64,
}
impl ProductCache {
    #[must_use]
    pub fn new(limits: CacheLimits) -> Self {
        Self {
            entries: BTreeMap::new(),
            total_weight: 0,
            limits,
            next_access: 0,
        }
    }
    pub fn get(&mut self, key: &ProductKey) -> Option<Arc<dyn CachedProduct>> {
        let entry = self.entries.get_mut(key)?;
        if entry.product.kind() != product_kind_from_key(key, &entry.key)
            || entry.product.digest() != &entry.content_digest
        {
            self.total_weight -= entry.weight;
            self.entries.remove(key);
            return None;
        }
        self.next_access += 1;
        entry.last_access = self.next_access;
        Some(Arc::clone(&entry.product))
    }
    pub fn insert(&mut self, entry: CacheEntry) {
        if let Some(old) = self.entries.insert(entry.key.clone(), entry) {
            self.total_weight -= old.weight;
        }
        self.total_weight = self.entries.values().map(|entry| entry.weight).sum();
    }
    fn promote_and_evict(&mut self) {
        for entry in self.entries.values_mut() {
            if entry.state == CacheEntryState::AttemptLocal {
                entry.state = CacheEntryState::Committed;
            }
        }
        while self.entries.len() > self.limits.maximum_entries
            || self.total_weight > self.limits.maximum_weight
        {
            let candidate = self
                .entries
                .iter()
                .filter(|(_, entry)| entry.state == CacheEntryState::Committed)
                .min_by_key(|(key, entry)| (entry.last_access, (*key).clone()))
                .map(|(key, _)| key.clone());
            let Some(key) = candidate else { break };
            if let Some(entry) = self.entries.remove(&key) {
                self.total_weight -= entry.weight;
            }
        }
    }
    #[must_use]
    pub fn inspection(&self) -> ProductCacheInspection {
        let mut entries = self
            .entries
            .values()
            .map(|entry| CacheInspectionEntry {
                product_key: entry.key.clone(),
                product_kind: entry.product.kind(),
                content_digest: entry.content_digest.clone(),
                state: entry.state,
                weight: entry.weight,
            })
            .collect::<Vec<_>>();
        entries.sort_by_key(|e| e.product_key.clone());
        ProductCacheInspection {
            schema_version: PRODUCT_CACHE_INSPECTION_SCHEMA_VERSION,
            entry_count: entries.len(),
            total_weight: self.total_weight,
            limits: self.limits.clone(),
            entries,
        }
    }
}
fn product_kind_from_key(_: &ProductKey, _: &ProductKey) -> ProductKind {
    ProductKind::Parse
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CacheInspectionEntry {
    pub product_key: ProductKey,
    pub product_kind: ProductKind,
    pub content_digest: Digest,
    pub state: CacheEntryState,
    pub weight: u64,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProductCacheInspection {
    pub schema_version: u32,
    pub entry_count: usize,
    pub total_weight: u64,
    pub limits: CacheLimits,
    pub entries: Vec<CacheInspectionEntry>,
}
impl ProductCacheInspection {
    #[must_use]
    pub fn to_canonical_json(&self) -> Vec<u8> {
        json_document(cache_json(self))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CompilerSessionStatus {
    Empty,
    Ready,
    Compiling,
    Failed,
    Closed,
}
#[derive(Debug, Clone)]
pub struct CommittedWorkspaceState {
    pub snapshot: Arc<WorkspaceSnapshot>,
    pub graph: Arc<WorkspaceGraph>,
    pub committed_product_keys: Vec<ProductKey>,
    pub committed_workspace_product_keys: Vec<WorkspaceProductKey>,
}
#[derive(Debug, Clone)]
pub struct CompilationAttemptState {
    pub attempt_id: CompilationAttemptId,
    pub baseline_snapshot_id: Option<WorkspaceSnapshotId>,
    pub candidate_snapshot: Arc<WorkspaceSnapshot>,
    pub plan: Arc<IncrementalPlan>,
    pub phase: AttemptPhase,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AttemptPhase {
    Planned,
    Executing,
    Validating,
    Committing,
}
pub struct CompilerSessionState {
    pub session_id: CompilerSessionId,
    pub workspace_id: WorkspaceId,
    pub compiler_contract: ContractVersion,
    pub status: CompilerSessionStatus,
    pub committed: Option<CommittedWorkspaceState>,
    pub active_attempt: Option<CompilationAttemptState>,
    pub cache: ProductCache,
    pub next_attempt: u64,
}
#[derive(Debug, Clone)]
pub struct CancellationToken(Arc<AtomicBool>);
impl CancellationToken {
    #[must_use]
    pub fn new() -> Self {
        Self(Arc::new(AtomicBool::new(false)))
    }
    pub fn cancel(&self) {
        self.0.store(true, Ordering::Release);
    }
    #[must_use]
    pub fn is_cancelled(&self) -> bool {
        self.0.load(Ordering::Acquire)
    }
}
impl Default for CancellationToken {
    fn default() -> Self {
        Self::new()
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RequestedCompilationMode {
    Automatic,
    Full,
}
#[derive(Debug, Clone)]
pub struct CompileWorkspaceRequest {
    pub workspace: WorkspaceInput,
    pub mode: RequestedCompilationMode,
    pub cancellation: CancellationToken,
}
#[derive(Debug, Clone)]
pub struct CommittedCompilation {
    pub snapshot: Arc<WorkspaceSnapshot>,
    pub graph: Arc<WorkspaceGraph>,
    pub plan: Arc<IncrementalPlan>,
}
#[derive(Debug, Clone)]
pub struct RejectedCompilation {
    pub message: String,
}
#[derive(Debug, Clone)]
pub struct CancelledCompilation {
    pub message: String,
}
#[derive(Debug, Clone)]
pub enum CompilationOutcome {
    Committed(CommittedCompilation),
    Rejected(RejectedCompilation),
    Cancelled(CancelledCompilation),
    PlatformFailure(PlatformFailure),
}
impl CompilerSessionState {
    #[must_use]
    pub fn new(
        workspace: WorkspaceId,
        compiler_contract: ContractVersion,
        limits: CacheLimits,
    ) -> Self {
        let session = NEXT_SESSION_ID.fetch_add(1, Ordering::Relaxed);
        Self {
            session_id: CompilerSessionId(format!("session:{session}")),
            workspace_id: workspace,
            compiler_contract,
            status: CompilerSessionStatus::Empty,
            committed: None,
            active_attempt: None,
            cache: ProductCache::new(limits),
            next_attempt: 1,
        }
    }
    #[must_use]
    pub fn committed_snapshot(&self) -> Option<&WorkspaceSnapshot> {
        self.committed.as_ref().map(|state| state.snapshot.as_ref())
    }
    #[must_use]
    pub fn committed_graph(&self) -> Option<&WorkspaceGraph> {
        self.committed.as_ref().map(|state| state.graph.as_ref())
    }
    #[must_use]
    pub const fn status(&self) -> CompilerSessionStatus {
        self.status
    }
    pub fn close(&mut self) {
        self.status = CompilerSessionStatus::Closed;
        self.active_attempt = None;
        self.committed = None;
        self.cache = ProductCache::new(CacheLimits::default());
    }
    pub fn compile_workspace(&mut self, request: CompileWorkspaceRequest) -> CompilationOutcome {
        if self.status == CompilerSessionStatus::Closed {
            return CompilationOutcome::PlatformFailure(PlatformFailure::new(
                PlatformFailureCode::SessionClosed,
                "compiler session is closed",
            ));
        }
        if self.active_attempt.is_some() {
            return CompilationOutcome::PlatformFailure(PlatformFailure::new(
                PlatformFailureCode::ConcurrentAttempt,
                "compiler session already has an active attempt",
            ));
        }
        let snapshot = match WorkspaceSnapshot::from_input(&request.workspace) {
            Ok(snapshot) => snapshot,
            Err(error) => return self.fail(error),
        };
        if request.cancellation.is_cancelled() {
            return self.cancel();
        }
        let plan = Arc::new(plan_incremental(
            self.committed
                .as_ref()
                .map(|state| (state.snapshot.as_ref(), state.graph.as_ref())),
            &snapshot,
            request.mode == RequestedCompilationMode::Full,
        ));
        let attempt_id = CompilationAttemptId(format!(
            "attempt:{}:{}",
            self.session_id.as_str().trim_start_matches("session:"),
            self.next_attempt
        ));
        self.next_attempt += 1;
        self.status = CompilerSessionStatus::Compiling;
        self.active_attempt = Some(CompilationAttemptState {
            attempt_id,
            baseline_snapshot_id: self
                .committed
                .as_ref()
                .map(|state| state.snapshot.snapshot_id.clone()),
            candidate_snapshot: Arc::new(snapshot.clone()),
            plan: Arc::clone(&plan),
            phase: AttemptPhase::Executing,
        });
        let graph = match build_graph(&request.workspace, &snapshot) {
            Ok(graph) => graph,
            Err(error) => return self.fail(error),
        };
        if request.cancellation.is_cancelled() {
            return self.cancel();
        }
        if let Err(error) = snapshot.validate().and_then(|()| graph.validate()) {
            return self.fail(error);
        }
        if let Some(attempt) = self.active_attempt.as_mut() {
            attempt.phase = AttemptPhase::Committing;
        } else {
            return self.fail(PlatformFailure::new(
                PlatformFailureCode::CommitInvariantFailed,
                "active compiler attempt disappeared before commit",
            ));
        }
        let committed = CommittedWorkspaceState {
            snapshot: Arc::new(snapshot),
            graph: Arc::new(graph),
            committed_product_keys: Vec::new(),
            committed_workspace_product_keys: Vec::new(),
        };
        self.workspace_id = committed.snapshot.workspace_id.clone();
        self.compiler_contract = committed.snapshot.compiler_contract.clone();
        self.committed = Some(committed.clone());
        self.active_attempt = None;
        self.status = CompilerSessionStatus::Ready;
        self.cache.promote_and_evict();
        CompilationOutcome::Committed(CommittedCompilation {
            snapshot: committed.snapshot,
            graph: committed.graph,
            plan,
        })
    }
    fn fail(&mut self, error: PlatformFailure) -> CompilationOutcome {
        self.active_attempt = None;
        self.status = CompilerSessionStatus::Failed;
        CompilationOutcome::PlatformFailure(error)
    }
    fn cancel(&mut self) -> CompilationOutcome {
        self.active_attempt = None;
        self.status = CompilerSessionStatus::Failed;
        CompilationOutcome::Cancelled(CancelledCompilation {
            message: "compilation cancelled".into(),
        })
    }
    #[must_use]
    pub fn inspect(&self) -> CompilerSessionInspection {
        CompilerSessionInspection {
            schema_version: COMPILER_SESSION_SCHEMA_VERSION,
            session_id: self.session_id.clone(),
            workspace_id: self.workspace_id.clone(),
            compiler_contract: self.compiler_contract.clone(),
            status: self.status,
            committed_snapshot_id: self
                .committed
                .as_ref()
                .map(|value| value.snapshot.snapshot_id.clone()),
            active_attempt: self
                .active_attempt
                .as_ref()
                .map(|value| value.attempt_id.clone()),
            committed_product_count: self
                .committed
                .as_ref()
                .map_or(0, |value| value.committed_product_keys.len()),
            cache: CacheSummary {
                entry_count: self.cache.entries.len(),
                total_weight: self.cache.total_weight,
            },
        }
    }
}
fn build_graph(
    input: &WorkspaceInput,
    snapshot: &WorkspaceSnapshot,
) -> Result<WorkspaceGraph, PlatformFailure> {
    let (_, fingerprint, inputs) = normalize(input)?;
    let parsed = inputs
        .iter()
        .map(|source| ezc_parser::parse_file(source.path.as_str(), &source.source))
        .collect::<Vec<_>>();
    let unit = CompilationUnit::from_parsed_files(parsed.clone());
    let model = build_application_semantic_model_for_unit(&unit);
    let model_digest = Digest::sha256(format!(
        "components:{};diagnostics:{}",
        model.components.len(),
        model.diagnostics.len()
    ));
    let parser_contract = ContractVersion::new("parser:1");
    let units = inputs
        .iter()
        .map(|source| {
            let key = product_key(
                &source.source_unit_id,
                &source.source_revision_id,
                ProductKind::Parse,
                &parser_contract,
                &fingerprint,
            );
            WorkspaceUnit {
                source_unit_id: source.source_unit_id.clone(),
                path: source.path.clone(),
                source_revision_id: source.source_revision_id.clone(),
                source_digest: source.source_digest.clone(),
                source_length: source.source.len() as u64,
                language: source.language,
                products: vec![UnitProductRef {
                    product_kind: ProductKind::Parse,
                    product_key: key,
                    producer_contract: parser_contract.clone(),
                    content_digest: source.source_digest.clone(),
                }],
            }
        })
        .collect::<Vec<_>>();
    let by_path = inputs
        .iter()
        .map(|source| (source.path.clone(), source.source_unit_id.clone()))
        .collect::<BTreeMap<_, _>>();
    let mut edges = Vec::new();
    for (source, file) in inputs.iter().zip(parsed.iter()) {
        for import in &file.imports {
            if let Some(target) = resolve_import(&source.path, &import.source, &by_path) {
                edges.push(WorkspaceDependencyEdge {
                    source: source.source_unit_id.clone(),
                    kind: WorkspaceDependencyKind::Compile,
                    target,
                });
            }
        }
    }
    edges.sort_by_key(|edge| (edge.source.clone(), edge.kind, edge.target.clone()));
    edges.dedup();
    let asm_contract = ContractVersion::new("asm:1");
    let application_products = vec![ApplicationProductRef {
        product_kind: ApplicationProductKind::ApplicationSemanticModel,
        product_key: workspace_product_key(
            &snapshot.workspace_id,
            &snapshot.snapshot_id,
            ApplicationProductKind::ApplicationSemanticModel,
            &asm_contract,
            &fingerprint,
        ),
        producer_contract: asm_contract,
        content_digest: model_digest,
    }];
    Ok(WorkspaceGraph {
        schema_version: WORKSPACE_GRAPH_SCHEMA_VERSION,
        workspace_id: snapshot.workspace_id.clone(),
        snapshot_id: snapshot.snapshot_id.clone(),
        compiler_contract: snapshot.compiler_contract.clone(),
        units,
        dependency_edges: edges,
        application_products,
    })
}
fn resolve_import(
    source: &WorkspacePath,
    raw: &str,
    known: &BTreeMap<WorkspacePath, SourceUnitId>,
) -> Option<SourceUnitId> {
    if !raw.starts_with('.') {
        return None;
    }
    let mut parts = source.0.split('/').collect::<Vec<_>>();
    parts.pop();
    let separated = raw.replace('\\', "/");
    for part in separated.split('/') {
        match part {
            "." => {}
            ".." => {
                parts.pop()?;
            }
            value => parts.push(value),
        }
    }
    let base = parts.join("/");
    for candidate in [
        base.clone(),
        format!("{base}.ts"),
        format!("{base}.tsx"),
        format!("{base}.js"),
        format!("{base}.jsx"),
        format!("{base}/index.ts"),
        format!("{base}/index.tsx"),
    ] {
        if let Ok(path) = WorkspacePath::new(candidate) {
            if let Some(id) = known.get(&path) {
                return Some(id.clone());
            }
        }
    }
    None
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CacheSummary {
    pub entry_count: usize,
    pub total_weight: u64,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CompilerSessionInspection {
    pub schema_version: u32,
    pub session_id: CompilerSessionId,
    pub workspace_id: WorkspaceId,
    pub compiler_contract: ContractVersion,
    pub status: CompilerSessionStatus,
    pub committed_snapshot_id: Option<WorkspaceSnapshotId>,
    pub active_attempt: Option<CompilationAttemptId>,
    pub committed_product_count: usize,
    pub cache: CacheSummary,
}
impl CompilerSessionInspection {
    #[must_use]
    pub fn to_canonical_json(&self) -> Vec<u8> {
        json_document(session_json(self))
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlatformFailureCode {
    InvalidWorkspacePath,
    DuplicateWorkspacePath,
    UnsupportedSourceLanguage,
    InputReadFailed,
    InvalidWorkspaceGraph,
    InvalidWorkspaceSnapshot,
    InvalidIncrementalPlan,
    UnsupportedSchemaVersion,
    UnsupportedProducerContract,
    CacheCorruption,
    ProductDigestMismatch,
    DuplicateProductMismatch,
    MissingRequiredProduct,
    DependencyAuthorityUnavailable,
    ConcurrentAttempt,
    SessionClosed,
    CommitInvariantFailed,
    IncrementalEquivalenceFailed,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlatformFailure {
    pub code: PlatformFailureCode,
    pub message: String,
    pub workspace_path: Option<WorkspacePath>,
    pub source_unit_id: Option<SourceUnitId>,
    pub product_key: Option<ProductKey>,
}
impl PlatformFailure {
    fn new(code: PlatformFailureCode, message: impl Into<String>) -> Self {
        Self {
            code,
            message: message.into(),
            workspace_path: None,
            source_unit_id: None,
            product_key: None,
        }
    }
    fn path(message: impl Into<String>, path: &str) -> Self {
        Self {
            code: PlatformFailureCode::InvalidWorkspacePath,
            message: message.into(),
            workspace_path: Some(WorkspacePath(path.replace('\\', "/"))),
            source_unit_id: None,
            product_key: None,
        }
    }
}
#[derive(Debug, Clone)]
pub struct PlatformSerializationError {
    pub message: String,
}
impl PlatformSerializationError {
    fn validation(error: PlatformFailure) -> Self {
        Self {
            message: error.message,
        }
    }
}

fn quote(value: &str) -> String {
    serde_json::to_string(value).expect("strings serialize")
}
fn json_document(value: String) -> Vec<u8> {
    let mut value = value.into_bytes();
    value.push(b'\n');
    value
}
fn snapshot_json(snapshot: &WorkspaceSnapshot) -> String {
    format!("{{\"schema_version\":{},\"workspace_id\":{},\"snapshot_id\":{},\"compiler_contract\":{},\"configuration_fingerprint\":{},\"units\":[{}]}}",snapshot.schema_version,quote(snapshot.workspace_id.as_str()),quote(snapshot.snapshot_id.as_str()),quote(snapshot.compiler_contract.as_str()),quote(snapshot.configuration_fingerprint.as_str()),snapshot.units.iter().map(|unit|format!("{{\"source_unit_id\":{},\"path\":{},\"source_revision_id\":{},\"source_digest\":{},\"source_length\":{},\"language\":{}}}",quote(unit.source_unit_id.as_str()),quote(unit.path.as_str()),quote(unit.source_revision_id.as_str()),quote(unit.source_digest.as_str()),unit.source_length,quote(unit.language.as_str()))).collect::<Vec<_>>().join(","))
}
fn graph_json(graph: &WorkspaceGraph) -> String {
    format!("{{\"schema_version\":{},\"workspace_id\":{},\"snapshot_id\":{},\"compiler_contract\":{},\"units\":[{}],\"dependency_edges\":[{}],\"application_products\":[{}]}}",graph.schema_version,quote(graph.workspace_id.as_str()),quote(graph.snapshot_id.as_str()),quote(graph.compiler_contract.as_str()),graph.units.iter().map(|unit|format!("{{\"source_unit_id\":{},\"path\":{},\"source_revision_id\":{},\"source_digest\":{},\"source_length\":{},\"language\":{},\"products\":[{}]}}",quote(unit.source_unit_id.as_str()),quote(unit.path.as_str()),quote(unit.source_revision_id.as_str()),quote(unit.source_digest.as_str()),unit.source_length,quote(unit.language.as_str()),unit.products.iter().map(|product|format!("{{\"product_kind\":{},\"product_key\":{},\"producer_contract\":{},\"content_digest\":{}}}",quote(product.product_kind.as_str()),quote(product.product_key.as_str()),quote(product.producer_contract.as_str()),quote(product.content_digest.as_str()))).collect::<Vec<_>>().join(","))).collect::<Vec<_>>().join(","),graph.dependency_edges.iter().map(|edge|format!("{{\"source\":{},\"kind\":{},\"target\":{}}}",quote(edge.source.as_str()),quote(edge.kind.as_str()),quote(edge.target.as_str()))).collect::<Vec<_>>().join(","),graph.application_products.iter().map(|product|format!("{{\"product_kind\":{},\"product_key\":{},\"producer_contract\":{},\"content_digest\":{}}}",quote(product.product_kind.as_str()),quote(product.product_key.as_str()),quote(product.producer_contract.as_str()),quote(product.content_digest.as_str()))).collect::<Vec<_>>().join(","))
}

fn incremental_plan_json(plan: &IncrementalPlan) -> String {
    let ids = |values: &[SourceUnitId]| {
        values
            .iter()
            .map(|value| quote(value.as_str()))
            .collect::<Vec<_>>()
            .join(",")
    };
    let changes = format!("{{\"baseline_snapshot_id\":{},\"candidate_snapshot_id\":{},\"configuration_changed\":{},\"added\":[{}],\"removed\":[{}],\"modified\":[{}],\"unchanged\":[{}]}}", quote(plan.changes.baseline_snapshot_id.as_str()), quote(plan.changes.candidate_snapshot_id.as_str()), plan.changes.configuration_changed, ids(&plan.changes.added), ids(&plan.changes.removed), ids(&plan.changes.modified), ids(&plan.changes.unchanged));
    let invalidated = plan
        .invalidated_units
        .iter()
        .map(|item| {
            format!(
                "{{\"source_unit_id\":{},\"reason\":{},\"caused_by\":[{}]}}",
                quote(item.source_unit_id.as_str()),
                quote(item.reason.as_str()),
                ids(&item.caused_by)
            )
        })
        .collect::<Vec<_>>()
        .join(",");
    let stages = plan
        .stages
        .iter()
        .map(|stage| {
            format!(
                "{{\"stage_index\":{},\"kind\":{},\"units\":[{}]}}",
                stage.stage_index,
                quote(stage.kind.as_str()),
                ids(&stage.units)
            )
        })
        .collect::<Vec<_>>()
        .join(",");
    format!("{{\"schema_version\":{},\"plan_id\":{},\"baseline_snapshot_id\":{},\"candidate_snapshot_id\":{},\"mode\":{},\"changes\":{},\"invalidated_units\":[{}],\"stages\":[{}],\"retained_products\":[{}],\"discarded_products\":[{}],\"expected_commit\":{{\"snapshot_id\":{},\"workspace_graph_schema_version\":{}}}}}", plan.schema_version, quote(plan.plan_id.as_str()), quote(plan.baseline_snapshot_id.as_str()), quote(plan.candidate_snapshot_id.as_str()), quote(plan.mode.as_str()), changes, invalidated, stages, plan.retained_products.iter().map(|key|quote(key.as_str())).collect::<Vec<_>>().join(","), plan.discarded_products.iter().map(|key|quote(key.as_str())).collect::<Vec<_>>().join(","), quote(plan.expected_commit.snapshot_id.as_str()), plan.expected_commit.workspace_graph_schema_version)
}
fn cache_json(cache: &ProductCacheInspection) -> String {
    let entries = cache.entries.iter().map(|entry| format!("{{\"product_key\":{},\"product_kind\":{},\"content_digest\":{},\"state\":{},\"weight\":{}}}", quote(entry.product_key.as_str()), quote(entry.product_kind.as_str()), quote(entry.content_digest.as_str()), quote(match entry.state { CacheEntryState::AttemptLocal => "attempt_local", CacheEntryState::Committed => "committed" }), entry.weight)).collect::<Vec<_>>().join(",");
    format!("{{\"schema_version\":{},\"entry_count\":{},\"total_weight\":{},\"limits\":{{\"maximum_entries\":{},\"maximum_weight\":{}}},\"entries\":[{}]}}", cache.schema_version, cache.entry_count, cache.total_weight, cache.limits.maximum_entries, cache.limits.maximum_weight, entries)
}
fn session_json(session: &CompilerSessionInspection) -> String {
    format!("{{\"schema_version\":{},\"session_id\":{},\"workspace_id\":{},\"compiler_contract\":{},\"status\":{},\"committed_snapshot_id\":{},\"active_attempt\":{},\"committed_product_count\":{},\"cache\":{{\"entry_count\":{},\"total_weight\":{}}}}}", session.schema_version, quote(session.session_id.as_str()), quote(session.workspace_id.as_str()), quote(session.compiler_contract.as_str()), quote(match session.status { CompilerSessionStatus::Empty => "empty", CompilerSessionStatus::Ready => "ready", CompilerSessionStatus::Compiling => "compiling", CompilerSessionStatus::Failed => "failed", CompilerSessionStatus::Closed => "closed" }), session.committed_snapshot_id.as_ref().map_or_else(|| "null".into(), |id|quote(id.as_str())), session.active_attempt.as_ref().map_or_else(|| "null".into(), |id|quote(id.as_str())), session.committed_product_count, session.cache.entry_count, session.cache.total_weight)
}

/// Public submodule names are stable platform contracts; their values live in
/// this file to keep the first platform implementation compact.
pub mod workspace {
    pub use super::{
        ApplicationProductKind, ApplicationProductRef, ContractVersion, Digest, ProductKey,
        ProductKind, SourceLanguage, SourceRevisionId, SourceUnitId, UnitProductRef,
        WorkspaceConfiguration, WorkspaceDependencyEdge, WorkspaceDependencyKind, WorkspaceGraph,
        WorkspaceId, WorkspaceInput, WorkspacePath, WorkspaceProductKey, WorkspaceSource,
        WorkspaceUnit,
    };
}
pub mod snapshot {
    pub use super::{
        compare_snapshots, SnapshotUnit, WorkspaceChangeSet, WorkspaceSnapshot, WorkspaceSnapshotId,
    };
}
pub mod incremental {
    pub use super::{
        plan_incremental, ExpectedCommit, IncrementalMode, IncrementalPlan, IncrementalPlanId,
        IncrementalStage, IncrementalStageKind, InvalidatedUnit, InvalidationReason,
    };
}
pub mod cache {
    pub use super::{
        CacheEntry, CacheEntryState, CacheInspectionEntry, CacheLimits, CachedProduct,
        ProductCache, ProductCacheInspection,
    };
}
pub mod session {
    pub use super::{
        CancellationToken, CommittedCompilation, CompilationAttemptId, CompilationOutcome,
        CompileWorkspaceRequest, CompilerSessionId, CompilerSessionInspection,
        CompilerSessionState, CompilerSessionStatus, RequestedCompilationMode,
    };
}
pub mod inspection {
    pub use super::{CompilerSessionInspection, ProductCacheInspection};
}

#[cfg(test)]
mod tests {
    use super::*;
    fn source(path: &str, body: &str) -> WorkspaceSource {
        WorkspaceSource {
            path: path.into(),
            source: body.into(),
            language: None,
        }
    }
    fn input(sources: Vec<WorkspaceSource>) -> WorkspaceInput {
        WorkspaceInput::new(sources)
    }
    #[test]
    fn paths_normalize_and_reject_escape() {
        assert_eq!(
            WorkspacePath::new("src\\./Counter.tsx").unwrap().as_str(),
            "src/Counter.tsx"
        );
        assert!(WorkspacePath::new("../Counter.tsx").is_err());
        assert!(WorkspacePath::new("/Counter.tsx").is_err());
    }
    #[test]
    fn identities_are_content_and_path_sensitive() {
        let a =
            WorkspaceSnapshot::from_input(&input(vec![source("src/A.tsx", "class A {}")])).unwrap();
        let b = WorkspaceSnapshot::from_input(&input(vec![source("src/A.tsx", "class A { x=1 }")]))
            .unwrap();
        let c =
            WorkspaceSnapshot::from_input(&input(vec![source("src/B.tsx", "class A {}")])).unwrap();
        assert_eq!(a.units[0].source_unit_id, b.units[0].source_unit_id);
        assert_ne!(a.units[0].source_revision_id, b.units[0].source_revision_id);
        assert_ne!(a.units[0].source_unit_id, c.units[0].source_unit_id);
    }
    #[test]
    fn snapshots_and_graphs_are_enumeration_independent() {
        let left = input(vec![
            source("src/B.tsx", "class B {}"),
            source("src/A.tsx", "class A {}"),
        ]);
        let right = input(vec![
            source("src/A.tsx", "class A {}"),
            source("src/B.tsx", "class B {}"),
        ]);
        let l = WorkspaceSnapshot::from_input(&left).unwrap();
        let r = WorkspaceSnapshot::from_input(&right).unwrap();
        assert_eq!(
            l.to_canonical_json().unwrap(),
            r.to_canonical_json().unwrap()
        );
        let mut session = CompilerSessionState::new(
            l.workspace_id.clone(),
            l.compiler_contract.clone(),
            CacheLimits::default(),
        );
        let outcome = session.compile_workspace(CompileWorkspaceRequest {
            workspace: left,
            mode: RequestedCompilationMode::Full,
            cancellation: CancellationToken::new(),
        });
        let CompilationOutcome::Committed(done) = outcome else {
            panic!("expected commit")
        };
        assert!(done.graph.to_canonical_json().unwrap().ends_with(b"\n"));
    }
    #[test]
    fn changes_classify_rename_as_removal_and_addition() {
        let first =
            WorkspaceSnapshot::from_input(&input(vec![source("src/A.tsx", "class A {}")])).unwrap();
        let second =
            WorkspaceSnapshot::from_input(&input(vec![source("src/B.tsx", "class A {}")])).unwrap();
        let changes = compare_snapshots(&first, &second);
        assert_eq!(changes.added.len(), 1);
        assert_eq!(changes.removed.len(), 1);
    }
    #[test]
    fn session_cancellation_keeps_baseline() {
        let initial = input(vec![source("src/A.tsx", "class A {}")]);
        let snapshot = WorkspaceSnapshot::from_input(&initial).unwrap();
        let mut session = CompilerSessionState::new(
            snapshot.workspace_id.clone(),
            snapshot.compiler_contract.clone(),
            CacheLimits::default(),
        );
        assert!(matches!(
            session.compile_workspace(CompileWorkspaceRequest {
                workspace: initial,
                mode: RequestedCompilationMode::Full,
                cancellation: CancellationToken::new()
            }),
            CompilationOutcome::Committed(_)
        ));
        let baseline = session.committed_snapshot().unwrap().snapshot_id.clone();
        let cancellation = CancellationToken::new();
        cancellation.cancel();
        assert!(matches!(
            session.compile_workspace(CompileWorkspaceRequest {
                workspace: input(vec![source("src/A.tsx", "class A { value=1 }")]),
                mode: RequestedCompilationMode::Automatic,
                cancellation
            }),
            CompilationOutcome::Cancelled(_)
        ));
        assert_eq!(session.committed_snapshot().unwrap().snapshot_id, baseline);
    }

    #[test]
    fn clean_full_equivalence_for_incremental_workspace_changes() {
        let baseline = input(vec![
            source("src/Dependency.tsx", "export class Dependency {}"),
            source(
                "src/App.tsx",
                "import { Dependency } from './Dependency'; export class App {}",
            ),
        ]);
        let candidate = input(vec![
            source(
                "src/Dependency.tsx",
                "export class Dependency { value = 1; }",
            ),
            source(
                "src/App.tsx",
                "import { Dependency } from './Dependency'; export class App {}",
            ),
            source("src/Added.tsx", "export class Added {}"),
        ]);
        let initial = WorkspaceSnapshot::from_input(&baseline).unwrap();
        let mut incremental = CompilerSessionState::new(
            initial.workspace_id.clone(),
            initial.compiler_contract.clone(),
            CacheLimits::default(),
        );
        assert!(matches!(
            incremental.compile_workspace(CompileWorkspaceRequest {
                workspace: baseline,
                mode: RequestedCompilationMode::Full,
                cancellation: CancellationToken::new(),
            }),
            CompilationOutcome::Committed(_)
        ));
        let incremental_result = incremental.compile_workspace(CompileWorkspaceRequest {
            workspace: candidate.clone(),
            mode: RequestedCompilationMode::Automatic,
            cancellation: CancellationToken::new(),
        });
        let CompilationOutcome::Committed(incremental_result) = incremental_result else {
            panic!("incremental candidate should commit");
        };
        assert_eq!(incremental_result.plan.mode, IncrementalMode::Incremental);

        let mut clean = CompilerSessionState::new(
            initial.workspace_id,
            initial.compiler_contract,
            CacheLimits::default(),
        );
        let clean_result = clean.compile_workspace(CompileWorkspaceRequest {
            workspace: candidate,
            mode: RequestedCompilationMode::Full,
            cancellation: CancellationToken::new(),
        });
        let CompilationOutcome::Committed(clean_result) = clean_result else {
            panic!("clean candidate should commit");
        };
        assert_eq!(
            incremental_result.snapshot.to_canonical_json().unwrap(),
            clean_result.snapshot.to_canonical_json().unwrap()
        );
        assert_eq!(
            incremental_result.graph.to_canonical_json().unwrap(),
            clean_result.graph.to_canonical_json().unwrap()
        );
    }
}
