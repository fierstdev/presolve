//! Local, durable compiler-service host (Phase L4).
//!
//! The host accepts complete request-owned workspace inputs and delegates every
//! compilation to `platform::CompilerSessionState`; it never reads workspace
//! files or persists source text.

#![allow(
    clippy::missing_errors_doc,
    clippy::missing_panics_doc,
    clippy::needless_pass_by_value,
    clippy::single_match_else,
    clippy::too_many_lines
)]

use std::collections::{BTreeMap, BTreeSet};
use std::fs::{self, File, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

use sha2::{Digest as _, Sha256};

use crate::persistent_cache::{
    CacheInspectionReportV1, CacheKeyInputV1, CacheOutcomeV1, CacheReportSelector,
    CacheTelemetryV1, CachedCompileResultV1, PersistentArtifactCacheV1,
};
use crate::platform::{
    self, CacheLimits, CancellationToken, CanonicalReusableProductV1, CompilationOutcome,
    CompileWorkspaceRequest, CompilerSessionState, ContractVersion, IncrementalCompilationModeV1,
    IncrementalCompileWorkspaceRequestV1, IncrementalFallbackReasonV1, RequestedCompilationMode,
    SourceRevisionId, SourceUnitId, WorkspaceConfiguration, WorkspaceGraph, WorkspaceId,
    WorkspaceInput, WorkspaceSnapshot, WorkspaceSource,
};
use crate::workspace::{self, WorkspaceBuildPlanV1, WorkspaceManifestV1, WorkspacePackageGraphV1};

pub const COMPILER_SERVICE_PROTOCOL_VERSION: u32 = 1;
pub const COMPILER_SERVICE_DESCRIPTOR_SCHEMA_VERSION: u32 = 1;
pub const COMPILER_SERVICE_REQUEST_SCHEMA_VERSION: u32 = 1;
pub const COMPILER_SERVICE_RESPONSE_SCHEMA_VERSION: u32 = 1;
pub const DURABLE_SESSION_SCHEMA_VERSION: u32 = 1;
pub const DURABLE_COMMIT_SCHEMA_VERSION: u32 = 1;
pub const SESSION_JOURNAL_SCHEMA_VERSION: u32 = 1;
pub const PERSISTENCE_MANIFEST_SCHEMA_VERSION: u32 = 1;
pub const SERVICE_INSPECTION_SCHEMA_VERSION: u32 = 1;
pub const SESSION_INSPECTION_SCHEMA_VERSION: u32 = 1;
pub const MAXIMUM_FRAME_BYTES: usize = 16 * 1024 * 1024;

static NEXT_SERVICE: AtomicU64 = AtomicU64::new(1);

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ServiceError {
    pub code: &'static str,
    pub message: String,
    pub retryable: bool,
}
impl ServiceError {
    fn new(code: &'static str, message: impl Into<String>) -> Self {
        Self {
            code,
            message: message.into(),
            retryable: false,
        }
    }
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CompilerServiceDescriptor {
    pub schema_version: u32,
    pub protocol_version: u32,
    pub service_instance_id: String,
    pub compiler_contract: ContractVersion,
    pub platform_contract: String,
    pub persistence_version: u32,
    pub capabilities: Vec<String>,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ServiceOperation {
    Handshake,
    ServiceStatus,
    OpenSession,
    ResumeSession,
    Compile,
    Cancel,
    SessionStatus,
    CloseSession,
    Shutdown,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ServiceResponseStatus {
    Ok,
    Rejected,
    Cancelled,
    Failed,
}
#[derive(Debug, Clone)]
pub struct CompleteSource {
    pub path: String,
    pub source: String,
    pub language: Option<platform::SourceLanguage>,
}
#[derive(Debug, Clone)]
pub struct CompileRequest {
    pub configuration: WorkspaceConfiguration,
    pub candidate_snapshot: WorkspaceSnapshot,
    pub sources: Vec<CompleteSource>,
    pub mode: RequestedCompilationMode,
    /// Optional L5 inspection data. `None` preserves the L4 response surface.
    pub incremental_report: IncrementalReportSelector,
    /// Explicit test-only proof mode. It never enables production semantics.
    pub verify_exact_equivalence: bool,
    /// Optional L6 cache telemetry. `None` preserves the L4/L5 response surface.
    pub cache_report: CacheReportSelector,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IncrementalReportSelector {
    None,
    Summary,
    Full,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IncrementalExecutionReportV1 {
    pub schema: &'static str,
    pub plan_fingerprint: platform::Digest,
    pub mode: IncrementalCompilationModeV1,
    pub changed_inputs: Vec<platform::IncrementalInputChangeV1>,
    pub invalidated_identities: Vec<SourceUnitId>,
    pub reused_product_identities: Vec<platform::ProductKey>,
    pub recomputed_work_unit_identities: Vec<SourceUnitId>,
    pub fallback_reasons: Vec<IncrementalFallbackReasonV1>,
    pub publication_outcome: &'static str,
    pub exact_equivalence_verified: Option<bool>,
}
impl IncrementalExecutionReportV1 {
    /// # Panics
    ///
    /// Panics only if serializing an owned Rust string fails, which `serde_json`
    /// guarantees for strings.
    #[must_use]
    pub fn to_canonical_json(&self) -> Vec<u8> {
        let quote = |value: &str| serde_json::to_string(value).expect("strings serialize");
        let changes = self
            .changed_inputs
            .iter()
            .map(|change| {
                format!(
                    "{{\"kind\":{},\"identity\":{}}}",
                    quote(change.kind.as_str()),
                    quote(&change.identity)
                )
            })
            .collect::<Vec<_>>()
            .join(",");
        let ids = |values: &[SourceUnitId]| {
            values
                .iter()
                .map(|value| quote(value.as_str()))
                .collect::<Vec<_>>()
                .join(",")
        };
        let reused = self
            .reused_product_identities
            .iter()
            .map(|value| quote(value.as_str()))
            .collect::<Vec<_>>()
            .join(",");
        let fallback = self
            .fallback_reasons
            .iter()
            .map(|reason| quote(reason.code()))
            .collect::<Vec<_>>()
            .join(",");
        format!("{{\"schema\":{},\"plan_fingerprint\":{},\"mode\":{},\"changed_inputs\":[{}],\"invalidated_identities\":[{}],\"reused_product_identities\":[{}],\"recomputed_work_unit_identities\":[{}],\"fallback_reasons\":[{}],\"publication_outcome\":{},\"exact_equivalence_verified\":{}}}\n",quote(self.schema),quote(self.plan_fingerprint.as_str()),quote(self.mode.as_str()),changes,ids(&self.invalidated_identities),reused,ids(&self.recomputed_work_unit_identities),fallback,quote(self.publication_outcome),self.exact_equivalence_verified.map_or_else(|| "null".into(), |value| value.to_string())).into_bytes()
    }
}
#[derive(Debug, Clone)]
pub struct CompileResponse {
    pub commit_sequence: u64,
    pub snapshot: Arc<WorkspaceSnapshot>,
    pub graph: Arc<WorkspaceGraph>,
    pub mode: String,
    pub incremental_report: Option<IncrementalExecutionReportV1>,
    pub cache_report: Option<CacheTelemetryV1>,
}
#[derive(Debug, Clone)]
pub struct WorkspacePackageCompileRequestV1 {
    pub package_id: String,
    pub expected_commit_sequence: u64,
    pub request: CompileRequest,
}
#[derive(Debug, Clone)]
pub struct WorkspaceCompileRequestV1 {
    pub manifest: WorkspaceManifestV1,
    pub packages: Vec<WorkspacePackageCompileRequestV1>,
    pub operation_id: String,
}
#[derive(Debug, Clone)]
pub struct WorkspacePackageResultV1 {
    pub package_id: String,
    pub status: String,
    pub snapshot_id: Option<String>,
}
#[derive(Debug, Clone)]
pub struct WorkspaceBuildResultV1 {
    pub workspace_id: String,
    pub status: String,
    pub manifest_identity: String,
    pub graph_identity: String,
    pub plan_identity: String,
    pub package_results: Vec<WorkspacePackageResultV1>,
}
#[derive(Debug, Clone)]
struct DurableWorkspaceStateV1 {
    manifest: WorkspaceManifestV1,
    graph: WorkspacePackageGraphV1,
    plan: WorkspaceBuildPlanV1,
    result: WorkspaceBuildResultV1,
}

pub fn encode_frame(payload: &[u8]) -> Result<Vec<u8>, ServiceError> {
    if payload.is_empty() {
        return Err(ServiceError::new("invalid_frame", "empty payload"));
    }
    if payload.len() > MAXIMUM_FRAME_BYTES {
        return Err(ServiceError::new(
            "frame_too_large",
            "payload exceeds service limit",
        ));
    }
    let mut frame = format!("{:08x}:", payload.len()).into_bytes();
    frame.extend_from_slice(payload);
    frame.push(b'\n');
    Ok(frame)
}
pub fn decode_frame(frame: &[u8]) -> Result<Vec<u8>, ServiceError> {
    if frame.len() < 10 || frame[8] != b':' || *frame.last().unwrap_or(&0) != b'\n' {
        return Err(ServiceError::new("invalid_frame", "malformed frame"));
    }
    let header = std::str::from_utf8(&frame[..8])
        .map_err(|_| ServiceError::new("invalid_frame", "non-ASCII length"))?;
    if !header
        .bytes()
        .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
    {
        return Err(ServiceError::new(
            "invalid_frame",
            "length is not lowercase hexadecimal",
        ));
    }
    let length = usize::from_str_radix(header, 16)
        .map_err(|_| ServiceError::new("invalid_frame", "invalid length"))?;
    if length == 0 || length > MAXIMUM_FRAME_BYTES || frame.len() != length + 10 {
        return Err(ServiceError::new(
            "invalid_frame",
            "truncated or trailing frame",
        ));
    }
    Ok(frame[9..frame.len() - 1].to_vec())
}

pub struct CompilerServiceHost {
    root: PathBuf,
    descriptor: CompilerServiceDescriptor,
    sessions: BTreeMap<String, DurableSession>,
    cache: PersistentArtifactCacheV1,
    workspaces: BTreeMap<String, DurableWorkspaceStateV1>,
}
struct DurableSession {
    workspace_id: WorkspaceId,
    configuration: WorkspaceConfiguration,
    compiler_contract: ContractVersion,
    commit_sequence: u64,
    closed: bool,
    l3: CompilerSessionState,
    incremental_baseline: Option<IncrementalBaseline>,
}
#[derive(Clone)]
struct IncrementalBaseline {
    publication_identity: String,
    configuration: WorkspaceConfiguration,
    source_fingerprints: BTreeMap<SourceUnitId, SourceRevisionId>,
    snapshot: Arc<WorkspaceSnapshot>,
    graph: Arc<WorkspaceGraph>,
    reusable_products: Vec<CanonicalReusableProductV1>,
    compiler_contract: ContractVersion,
}
impl IncrementalBaseline {
    fn is_consistent(&self) -> bool {
        self.publication_identity == self.snapshot.snapshot_id.as_str()
            && self.compiler_contract == self.snapshot.compiler_contract
            && platform::workspace_configuration_fingerprint_v1(&self.configuration)
                .is_ok_and(|fingerprint| fingerprint == self.snapshot.configuration_fingerprint)
            && self.source_fingerprints
                == self
                    .snapshot
                    .units
                    .iter()
                    .map(|unit| (unit.source_unit_id.clone(), unit.source_revision_id.clone()))
                    .collect()
    }
}
impl CompilerServiceHost {
    pub fn start(
        root: impl AsRef<Path>,
        compiler_contract: ContractVersion,
    ) -> Result<Self, ServiceError> {
        Self::start_with_cache(root, None, compiler_contract)
    }
    pub fn start_with_cache(
        root: impl AsRef<Path>,
        cache_root: Option<&Path>,
        compiler_contract: ContractVersion,
    ) -> Result<Self, ServiceError> {
        let root = root.as_ref().join("service");
        fs::create_dir_all(root.join("sessions")).map_err(io_error)?;
        let service_id = format!(
            "service-instance:{:032x}",
            NEXT_SERVICE.fetch_add(1, Ordering::Relaxed)
        );
        let descriptor = CompilerServiceDescriptor {
            schema_version: 1,
            protocol_version: 1,
            service_instance_id: service_id,
            compiler_contract: compiler_contract.clone(),
            platform_contract: "presolve-platform-l3:1".into(),
            persistence_version: 1,
            capabilities: vec![
                "canonical_inspection".into(),
                "durable_sessions".into(),
                "incremental_compile".into(),
                "request_cancellation".into(),
                "session_recovery".into(),
            ],
        };
        let host = Self {
            root,
            descriptor,
            sessions: BTreeMap::new(),
            cache: PersistentArtifactCacheV1::open(cache_root, &compiler_contract),
            workspaces: BTreeMap::new(),
        };
        host.write_manifest()?;
        Ok(host)
    }
    #[must_use]
    pub fn descriptor(&self) -> &CompilerServiceDescriptor {
        &self.descriptor
    }
    pub fn open_session(
        &mut self,
        configuration: WorkspaceConfiguration,
        claimed: &WorkspaceId,
    ) -> Result<String, ServiceError> {
        let workspace = platform::derive_workspace_id_v1(&configuration).map_err(platform_error)?;
        if &workspace != claimed {
            return Err(ServiceError::new(
                "workspace_mismatch",
                "workspace identity does not match configuration",
            ));
        }
        let id = format!(
            "session:sha256:{:x}",
            Sha256::digest(
                format!(
                    "presolve-durable-session-v1\0{}\0{}",
                    workspace.as_str(),
                    NEXT_SERVICE.fetch_add(1, Ordering::Relaxed)
                )
                .as_bytes()
            )
        );
        let l3 = CompilerSessionState::new(
            workspace.clone(),
            self.descriptor.compiler_contract.clone(),
            CacheLimits::default(),
        );
        let session = DurableSession {
            workspace_id: workspace,
            configuration,
            compiler_contract: self.descriptor.compiler_contract.clone(),
            commit_sequence: 0,
            closed: false,
            l3,
            incremental_baseline: None,
        };
        self.write_session(&id, &session)?;
        append_journal(&self.root, &id, 1, "session_created", 0, None)?;
        self.sessions.insert(id.clone(), session);
        Ok(id)
    }
    /// Closes a live session and releases its non-durable L5 baseline before
    /// recording the existing L4 durable closed-state marker.
    pub fn close_session(&mut self, session_id: &str) -> Result<(), ServiceError> {
        let root = self.root.clone();
        let session = self
            .sessions
            .get_mut(session_id)
            .ok_or_else(|| ServiceError::new("session_not_found", "session not loaded"))?;
        if session.closed {
            return Ok(());
        }
        session.incremental_baseline = None;
        session.l3.close();
        session.closed = true;
        write_session_at(&root, session_id, session)?;
        append_journal(
            &root,
            session_id,
            session.commit_sequence.saturating_mul(2).saturating_add(2),
            "session_closed",
            session.commit_sequence,
            None,
        )
    }
    pub fn verify_cache(
        &self,
        _explicit_root: &Path,
    ) -> Result<CacheInspectionReportV1, ServiceError> {
        self.cache.verify().map_err(cache_operation_error)
    }
    pub fn inspect_cache(
        &self,
        _explicit_root: &Path,
    ) -> Result<CacheInspectionReportV1, ServiceError> {
        self.cache.inspect().map_err(cache_operation_error)
    }
    pub fn clean_cache(&self, _explicit_root: &Path) -> Result<Vec<String>, ServiceError> {
        self.cache.clean().map_err(cache_operation_error)
    }
    pub fn compile_workspace_v1(
        &mut self,
        request: WorkspaceCompileRequestV1,
    ) -> Result<WorkspaceBuildResultV1, ServiceError> {
        let manifest = request
            .manifest
            .normalize_validate()
            .map_err(workspace_error)?;
        let graph = workspace::graph(&manifest).map_err(workspace_error)?;
        let expected = manifest
            .packages
            .iter()
            .map(|p| p.package_id.clone())
            .collect::<BTreeSet<_>>();
        let supplied = request
            .packages
            .iter()
            .map(|p| p.package_id.clone())
            .collect::<BTreeSet<_>>();
        if expected != supplied || supplied.len() != request.packages.len() {
            return Err(ServiceError::new(
                "L7W007_PACKAGE_REQUEST_SET_MISMATCH",
                "workspace request keys do not match manifest",
            ));
        }
        let mut requests = request
            .packages
            .into_iter()
            .map(|p| (p.package_id.clone(), p))
            .collect::<BTreeMap<_, _>>();
        for package in &manifest.packages {
            let item = requests
                .get(&package.package_id)
                .expect("validated request");
            if package.session_id != item.request.candidate_snapshot.workspace_id.as_str()
                && !self.sessions.contains_key(&package.session_id)
            {
                return Err(ServiceError::new(
                    "L7W011_WORKSPACE_SESSION_OWNERSHIP_CONFLICT",
                    "package session does not exist",
                ));
            }
            if let Some(hint) = &package.configuration_identity_hint {
                let actual =
                    platform::workspace_configuration_fingerprint_v1(&item.request.configuration)
                        .map_err(platform_error)?;
                if hint != actual.as_str() {
                    return Err(ServiceError::new(
                        "L7W008_CONFIGURATION_IDENTITY_HINT_MISMATCH",
                        "configuration identity hint mismatch",
                    ));
                }
            }
        }
        let fingerprints = requests
            .iter()
            .map(|(id, item)| {
                (
                    id.clone(),
                    item.request
                        .candidate_snapshot
                        .snapshot_id
                        .as_str()
                        .to_owned(),
                )
            })
            .collect();
        let plan = workspace::plan(&graph, fingerprints);
        let mut results = Vec::new();
        let mut failed = false;
        for stage in &plan.stages {
            for package_id in &stage.packages {
                let item = requests.remove(package_id).expect("plan package");
                let session_id = manifest
                    .packages
                    .iter()
                    .find(|p| p.package_id == *package_id)
                    .expect("descriptor")
                    .session_id
                    .clone();
                if failed {
                    results.push(WorkspacePackageResultV1 {
                        package_id: package_id.clone(),
                        status: "skipped_fail_fast".into(),
                        snapshot_id: None,
                    });
                    continue;
                }
                match self.compile(&session_id, item.expected_commit_sequence, item.request) {
                    Ok(response) => results.push(WorkspacePackageResultV1 {
                        package_id: package_id.clone(),
                        status: "succeeded".into(),
                        snapshot_id: Some(response.snapshot.snapshot_id.to_string()),
                    }),
                    Err(_) => {
                        failed = true;
                        results.push(WorkspacePackageResultV1 {
                            package_id: package_id.clone(),
                            status: "failed".into(),
                            snapshot_id: None,
                        });
                    }
                }
            }
        }
        let status = if failed { "failed" } else { "succeeded" }.to_owned();
        let result = WorkspaceBuildResultV1 {
            workspace_id: manifest.workspace_id.clone(),
            status,
            manifest_identity: graph.manifest_identity.clone(),
            graph_identity: graph.graph_identity.clone(),
            plan_identity: plan.plan_identity.clone(),
            package_results: results,
        };
        if !failed {
            let state = DurableWorkspaceStateV1 {
                manifest: manifest.clone(),
                graph: graph.clone(),
                plan: plan.clone(),
                result: result.clone(),
            };
            self.publish_workspace_state(&state)?;
            self.workspaces.insert(manifest.workspace_id.clone(), state);
        }
        Ok(result)
    }
    pub fn inspect_workspace(
        &self,
        workspace_id: &str,
    ) -> Result<WorkspaceBuildResultV1, ServiceError> {
        self.workspaces
            .get(workspace_id)
            .map(|state| state.result.clone())
            .ok_or_else(|| ServiceError::new("workspace_not_found", "workspace state not found"))
    }
    pub fn verify_workspace(&self, workspace_id: &str) -> Result<(), ServiceError> {
        let state = self
            .workspaces
            .get(workspace_id)
            .ok_or_else(|| ServiceError::new("workspace_not_found", "workspace state not found"))?;
        if state.graph.manifest_identity != state.manifest.identity()
            || state.plan.manifest_identity != state.graph.manifest_identity
        {
            return Err(ServiceError::new(
                "workspace_state_invalid",
                "workspace state integrity mismatch",
            ));
        }
        Ok(())
    }
    pub fn remove_workspace_state(&mut self, workspace_id: &str) -> Result<(), ServiceError> {
        self.workspaces.remove(workspace_id);
        let path = self
            .root
            .join("workspaces")
            .join(format!("{workspace_id}.json"));
        if path.exists() {
            fs::remove_file(path).map_err(io_error)?;
        }
        Ok(())
    }
    pub fn compile(
        &mut self,
        session_id: &str,
        expected: u64,
        request: CompileRequest,
    ) -> Result<CompileResponse, ServiceError> {
        let root = self.root.clone();
        let session = self
            .sessions
            .get_mut(session_id)
            .ok_or_else(|| ServiceError::new("session_not_found", "session not loaded"))?;
        if session.closed {
            return Err(ServiceError::new("session_closed", "session is closed"));
        }
        if expected != session.commit_sequence {
            return Err(ServiceError::new(
                "stale_session_state",
                "commit sequence is stale",
            ));
        }
        platform::canonical_workspace_configuration_json_v1(&request.configuration)
            .map_err(|error| ServiceError::new("invalid_request", error.message))?;
        let workspace = WorkspaceInput {
            configuration: request.configuration.clone(),
            sources: request
                .sources
                .iter()
                .map(|source| WorkspaceSource {
                    path: source.path.clone(),
                    source: source.source.clone(),
                    language: source.language,
                })
                .collect(),
            compiler_contract: session.compiler_contract.clone(),
        };
        let derived = WorkspaceSnapshot::from_input(&workspace).map_err(platform_failure)?;
        if derived
            .to_canonical_json()
            .map_err(|error| ServiceError::new("invalid_request", error.message))?
            != request
                .candidate_snapshot
                .to_canonical_json()
                .map_err(|error| ServiceError::new("invalid_request", error.message))?
        {
            return Err(ServiceError::new(
                "source_revision_mismatch",
                "complete source input does not match candidate snapshot",
            ));
        }
        let raw_baseline = session.incremental_baseline.as_ref();
        let baseline_is_malformed = raw_baseline.is_some_and(|baseline| !baseline.is_consistent());
        let baseline = raw_baseline.filter(|_| !baseline_is_malformed);
        let mut plan = platform::plan_incremental_compilation_v1(
            baseline.map(|value| (value.snapshot.as_ref(), value.graph.as_ref())),
            &derived,
        );
        if baseline_is_malformed {
            platform::force_clean_fallback_v1(
                &mut plan,
                &derived,
                IncrementalFallbackReasonV1::MalformedBaselineGraph,
            );
        }
        let cache_input = CacheKeyInputV1 {
            compiler_contract: session.compiler_contract.clone(),
            configuration_fingerprint: derived.configuration_fingerprint.as_str().into(),
            source_universe_fingerprint: platform::source_universe_fingerprint_v1(&derived)
                .as_str()
                .into(),
            compile_mode: match request.mode {
                RequestedCompilationMode::Automatic => "automatic",
                RequestedCompilationMode::Full => "full",
            },
        };
        if plan.mode == IncrementalCompilationModeV1::NoChange {
            let Some(baseline) = baseline else {
                return Err(ServiceError::new(
                    "internal_invariant_failed",
                    "L5 no-change plan has no baseline",
                ));
            };
            baseline.snapshot.validate().map_err(platform_failure)?;
            baseline.graph.validate().map_err(platform_failure)?;
            let report = selected_report(
                request.incremental_report,
                &plan,
                Vec::new(),
                Vec::new(),
                "published",
                Some(true),
            );
            return Ok(CompileResponse {
                commit_sequence: session.commit_sequence,
                snapshot: Arc::clone(&baseline.snapshot),
                graph: Arc::clone(&baseline.graph),
                mode: plan.mode.as_str().into(),
                incremental_report: report,
                cache_report: selected_cache_report(
                    request.cache_report,
                    CacheTelemetryV1 {
                        enabled: self.cache.enabled(),
                        outcome: CacheOutcomeV1::NotChecked,
                        reasons: Vec::new(),
                        cache_key: cache_input.key(),
                        payload_length: None,
                        result_fingerprint: None,
                        entry_published: false,
                        entry_replaced: false,
                    },
                ),
            });
        }
        // A restored L6 baseline deliberately has no parser products. Do not
        // pretend it can support L5 durable partial reuse.
        if plan.mode == IncrementalCompilationModeV1::Incremental
            && baseline.is_some_and(|value| value.reusable_products.is_empty())
        {
            platform::force_clean_fallback_v1(
                &mut plan,
                &derived,
                IncrementalFallbackReasonV1::ReuseProductRejected,
            );
        }
        let (cache_hit, mut cache_telemetry) = self.cache.lookup(&cache_input);
        if let Some(hit) = cache_hit {
            let previous_configuration = session.configuration.clone();
            let previous_workspace_id = session.workspace_id.clone();
            let previous_sequence = session.commit_sequence;
            session.configuration = request.configuration.clone();
            session.workspace_id = derived.workspace_id.clone();
            session.commit_sequence += 1;
            if publish_commit(
                &root,
                session_id,
                session,
                session.commit_sequence,
                &hit.snapshot,
                &hit.graph,
            )
            .is_ok()
            {
                session.incremental_baseline = Some(baseline_from_result(
                    &session.configuration,
                    Arc::new(hit.snapshot.clone()),
                    Arc::new(hit.graph.clone()),
                    Vec::new(),
                    &session.compiler_contract,
                ));
                return Ok(CompileResponse {
                    commit_sequence: session.commit_sequence,
                    snapshot: Arc::new(hit.snapshot),
                    graph: Arc::new(hit.graph),
                    mode: hit.response_mode,
                    incremental_report: selected_report(
                        request.incremental_report,
                        &plan,
                        Vec::new(),
                        Vec::new(),
                        "published",
                        None,
                    ),
                    cache_report: selected_cache_report(request.cache_report, cache_telemetry),
                });
            }
            session.configuration = previous_configuration;
            session.workspace_id = previous_workspace_id;
            session.commit_sequence = previous_sequence;
            cache_telemetry.outcome = CacheOutcomeV1::Miss;
            cache_telemetry.reasons.push(
                crate::persistent_cache::CacheReasonCodeV1::CanonicalProductValidationFailure,
            );
        }
        let verification_workspace = request.verify_exact_equivalence.then(|| workspace.clone());
        let (outcome, reused, recomputed) =
            if plan.mode == IncrementalCompilationModeV1::Incremental {
                let reusable_products = baseline
                    .map(|value| value.reusable_products.clone())
                    .unwrap_or_default();
                let execution = session.l3.compile_workspace_incremental_v1(
                    IncrementalCompileWorkspaceRequestV1 {
                        workspace,
                        cancellation: CancellationToken::new(),
                        plan: plan.clone(),
                        reusable_products,
                    },
                );
                (
                    execution.outcome,
                    execution.reused_product_identities,
                    execution.recomputed_work_units,
                )
            } else {
                let outcome = session.l3.compile_workspace(CompileWorkspaceRequest {
                    workspace,
                    // A fallback is intentionally a clean canonical L3 compile.
                    mode: RequestedCompilationMode::Full,
                    cancellation: CancellationToken::new(),
                });
                (outcome, Vec::new(), plan.recompute_work_units.clone())
            };
        let CompilationOutcome::Committed(committed) = outcome else {
            return Err(ServiceError::new(
                "compiler_platform_failed",
                "compiler did not commit",
            ));
        };
        let equivalence = if let Some(clean_workspace) = verification_workspace {
            let mut clean = CompilerSessionState::new(
                derived.workspace_id.clone(),
                session.compiler_contract.clone(),
                CacheLimits::default(),
            );
            let clean = clean.compile_workspace(CompileWorkspaceRequest {
                workspace: clean_workspace,
                mode: RequestedCompilationMode::Full,
                cancellation: CancellationToken::new(),
            });
            let CompilationOutcome::Committed(clean) = clean else {
                return Err(ServiceError::new(
                    "incremental_equivalence_failed",
                    "isolated clean L3 compilation failed",
                ));
            };
            if clean
                .snapshot
                .to_canonical_json()
                .map_err(platform_serialization)?
                != committed
                    .snapshot
                    .to_canonical_json()
                    .map_err(platform_serialization)?
                || clean
                    .graph
                    .to_canonical_json()
                    .map_err(platform_serialization)?
                    != committed
                        .graph
                        .to_canonical_json()
                        .map_err(platform_serialization)?
            {
                return Err(ServiceError::new(
                    "incremental_equivalence_failed",
                    "first canonical mismatch is workspace snapshot or graph",
                ));
            }
            Some(true)
        } else {
            None
        };
        let previous_configuration = session.configuration.clone();
        let previous_workspace_id = session.workspace_id.clone();
        let previous_sequence = session.commit_sequence;
        session.configuration = request.configuration;
        session.workspace_id = derived.workspace_id.clone();
        session.commit_sequence += 1;
        if let Err(error) = publish_commit(
            &root,
            session_id,
            session,
            session.commit_sequence,
            &committed.snapshot,
            &committed.graph,
        ) {
            session.configuration = previous_configuration;
            session.workspace_id = previous_workspace_id;
            session.commit_sequence = previous_sequence;
            return Err(error);
        }
        session.incremental_baseline = Some(baseline_from_result(
            &session.configuration,
            Arc::clone(&committed.snapshot),
            Arc::clone(&committed.graph),
            committed.reusable_products,
            &session.compiler_contract,
        ));
        let cache_telemetry = self.cache.publish(
            &cache_input,
            &CachedCompileResultV1 {
                snapshot: (*committed.snapshot).clone(),
                graph: (*committed.graph).clone(),
                response_mode: plan.mode.as_str().into(),
            },
        );
        Ok(CompileResponse {
            commit_sequence: session.commit_sequence,
            snapshot: committed.snapshot,
            graph: committed.graph,
            mode: plan.mode.as_str().into(),
            incremental_report: selected_report(
                request.incremental_report,
                &plan,
                reused,
                recomputed,
                "published",
                equivalence,
            ),
            cache_report: selected_cache_report(request.cache_report, cache_telemetry),
        })
    }
    fn write_manifest(&self) -> Result<(), ServiceError> {
        atomic_write(&self.root.join("manifest.json"),format!("{{\"schema_version\":1,\"protocol_version\":1,\"persistence_version\":1,\"compiler_contract\":{},\"platform_contract\":\"presolve-platform-l3:1\"}}\n",json(&self.descriptor.compiler_contract.to_string())).as_bytes())
    }
    fn write_session(&self, id: &str, session: &DurableSession) -> Result<(), ServiceError> {
        let config = platform::canonical_workspace_configuration_json_v1(&session.configuration)
            .map_err(|error| ServiceError::new("persistence_io_failed", error.message))?;
        let config = String::from_utf8(config).map_err(|_| {
            ServiceError::new(
                "internal_invariant_failed",
                "non-UTF8 canonical configuration",
            )
        })?;
        let config = config.trim_end();
        let dir = self.session_dir(id);
        fs::create_dir_all(dir.join("commits")).map_err(io_error)?;
        atomic_write(&dir.join("session.json"),format!("{{\"schema_version\":1,\"session_id\":{},\"workspace_id\":{},\"workspace_configuration\":{},\"compiler_contract\":{},\"state\":{},\"current_commit_sequence\":{}}}\n",json(id),json(session.workspace_id.as_str()),config,json(session.compiler_contract.as_str()),json(if session.closed{"closed"}else{"open"}),session.commit_sequence).as_bytes())
    }
    fn publish_workspace_state(&self, state: &DurableWorkspaceStateV1) -> Result<(), ServiceError> {
        let directory = self.root.join("workspaces");
        fs::create_dir_all(&directory).map_err(io_error)?;
        let packages = state
            .result
            .package_results
            .iter()
            .map(|p| {
                format!(
                    "{{\"package_id\":{},\"status\":{},\"snapshot_id\":{}}}",
                    json(&p.package_id),
                    json(&p.status),
                    p.snapshot_id
                        .as_ref()
                        .map_or_else(|| "null".into(), |v| json(v))
                )
            })
            .collect::<Vec<_>>()
            .join(",");
        let bytes=format!("{{\"schema\":\"presolve.durable-workspace-state\",\"version\":1,\"workspace_id\":{},\"manifest_identity\":{},\"graph_identity\":{},\"plan_identity\":{},\"package_results\":[{}]}}\n",json(&state.result.workspace_id),json(&state.result.manifest_identity),json(&state.result.graph_identity),json(&state.result.plan_identity),packages);
        atomic_write(
            &directory.join(format!("{}.json", state.result.workspace_id)),
            bytes.as_bytes(),
        )
    }
}
fn selected_report(
    selector: IncrementalReportSelector,
    plan: &platform::IncrementalCompilationPlanV1,
    reused_product_identities: Vec<platform::ProductKey>,
    recomputed_work_unit_identities: Vec<SourceUnitId>,
    publication_outcome: &'static str,
    exact_equivalence_verified: Option<bool>,
) -> Option<IncrementalExecutionReportV1> {
    if selector == IncrementalReportSelector::None {
        return None;
    }
    let mut report = IncrementalExecutionReportV1 {
        schema: platform::INCREMENTAL_EXECUTION_REPORT_V1_SCHEMA,
        plan_fingerprint: plan.plan_fingerprint.clone(),
        mode: plan.mode,
        changed_inputs: plan.input_changes.clone(),
        invalidated_identities: plan.invalidation_closure.clone(),
        reused_product_identities,
        recomputed_work_unit_identities,
        fallback_reasons: plan.fallback_reasons.clone(),
        publication_outcome,
        exact_equivalence_verified,
    };
    if selector == IncrementalReportSelector::Summary {
        report.changed_inputs.clear();
        report.invalidated_identities.clear();
        report.reused_product_identities.clear();
        report.recomputed_work_unit_identities.clear();
        report.fallback_reasons.clear();
    }
    Some(report)
}
fn selected_cache_report(
    selector: CacheReportSelector,
    mut telemetry: CacheTelemetryV1,
) -> Option<CacheTelemetryV1> {
    if selector == CacheReportSelector::None {
        return None;
    }
    if selector == CacheReportSelector::Summary {
        telemetry.payload_length = None;
        telemetry.result_fingerprint = None;
        telemetry.entry_published = false;
        telemetry.entry_replaced = false;
    }
    Some(telemetry)
}
fn baseline_from_result(
    configuration: &WorkspaceConfiguration,
    snapshot: Arc<WorkspaceSnapshot>,
    graph: Arc<WorkspaceGraph>,
    reusable_products: Vec<CanonicalReusableProductV1>,
    compiler_contract: &ContractVersion,
) -> IncrementalBaseline {
    IncrementalBaseline {
        publication_identity: snapshot.snapshot_id.to_string(),
        configuration: configuration.clone(),
        source_fingerprints: snapshot
            .units
            .iter()
            .map(|unit| (unit.source_unit_id.clone(), unit.source_revision_id.clone()))
            .collect(),
        snapshot,
        graph,
        reusable_products,
        compiler_contract: compiler_contract.clone(),
    }
}
fn publish_commit(
    root: &Path,
    id: &str,
    session: &DurableSession,
    sequence: u64,
    snapshot: &WorkspaceSnapshot,
    graph: &WorkspaceGraph,
) -> Result<(), ServiceError> {
    let dir = session_directory(root, id);
    let tmp = dir.join(format!("commits/{sequence:020}.tmp"));
    let final_dir = dir.join(format!("commits/{sequence:020}"));
    fs::create_dir_all(&tmp).map_err(io_error)?;
    let snapshot = snapshot
        .to_canonical_json()
        .map_err(|error| ServiceError::new("commit_publication_failed", error.message))?;
    let graph = graph
        .to_canonical_json()
        .map_err(|error| ServiceError::new("commit_publication_failed", error.message))?;
    atomic_write(&tmp.join("workspace-snapshot.json"), &snapshot)?;
    atomic_write(&tmp.join("workspace-graph.json"), &graph)?;
    atomic_write(
        &tmp.join("products.json"),
        b"{\"schema_version\":1,\"products\":[]}\n",
    )?;
    let snapshot_id = snapshot_id(&snapshot)?;
    atomic_write(&tmp.join("commit.json"),format!("{{\"schema_version\":1,\"session_id\":{},\"commit_sequence\":{},\"workspace_id\":{},\"snapshot_id\":{},\"workspace_graph_schema_version\":1,\"compiler_contract\":{}}}\n",json(id),sequence,json(session.workspace_id.as_str()),json(&snapshot_id),json(session.compiler_contract.as_str())).as_bytes())?;
    fs::rename(&tmp, &final_dir).map_err(io_error)?;
    append_journal(
        root,
        id,
        sequence * 2,
        "commit_prepared",
        sequence,
        Some(&snapshot_id),
    )?;
    atomic_write(&dir.join("current"), format!("{sequence:020}\n").as_bytes())?;
    write_session_at(root, id, session)?;
    append_journal(
        root,
        id,
        sequence * 2 + 1,
        "commit_published",
        sequence,
        Some(&snapshot_id),
    )
}
fn append_journal(
    root: &Path,
    id: &str,
    sequence: u64,
    kind: &str,
    commit: u64,
    snapshot: Option<&str>,
) -> Result<(), ServiceError> {
    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(session_directory(root, id).join("journal.ndjson"))
        .map_err(io_error)?;
    writeln!(file,"{{\"schema_version\":1,\"journal_sequence\":{},\"operation_id\":\"operation:service\",\"kind\":{},\"commit_sequence\":{},\"snapshot_id\":{}}}",sequence,json(kind),commit,snapshot.map_or("null".into(),json)).map_err(io_error)?;
    file.sync_all().map_err(io_error)
}
impl CompilerServiceHost {
    fn session_dir(&self, id: &str) -> PathBuf {
        self.root
            .join("sessions")
            .join(id.strip_prefix("session:sha256:").unwrap_or(id))
    }
}
fn session_directory(root: &Path, id: &str) -> PathBuf {
    root.join("sessions")
        .join(id.strip_prefix("session:sha256:").unwrap_or(id))
}
fn write_session_at(root: &Path, id: &str, session: &DurableSession) -> Result<(), ServiceError> {
    let config = platform::canonical_workspace_configuration_json_v1(&session.configuration)
        .map_err(|error| ServiceError::new("persistence_io_failed", error.message))?;
    let config = String::from_utf8(config).map_err(|_| {
        ServiceError::new(
            "internal_invariant_failed",
            "non-UTF8 canonical configuration",
        )
    })?;
    let dir = session_directory(root, id);
    fs::create_dir_all(dir.join("commits")).map_err(io_error)?;
    atomic_write(&dir.join("session.json"),format!("{{\"schema_version\":1,\"session_id\":{},\"workspace_id\":{},\"workspace_configuration\":{},\"compiler_contract\":{},\"state\":{},\"current_commit_sequence\":{}}}\n",json(id),json(session.workspace_id.as_str()),config.trim_end(),json(session.compiler_contract.as_str()),json(if session.closed{"closed"}else{"open"}),session.commit_sequence).as_bytes())
}
fn atomic_write(path: &Path, bytes: &[u8]) -> Result<(), ServiceError> {
    let tmp = path.with_extension("tmp");
    let mut file = File::create(&tmp).map_err(io_error)?;
    file.write_all(bytes).map_err(io_error)?;
    file.sync_all().map_err(io_error)?;
    fs::rename(tmp, path).map_err(io_error)
}
fn json(value: &str) -> String {
    serde_json::to_string(value).expect("strings serialize")
}
fn snapshot_id(bytes: &[u8]) -> Result<String, ServiceError> {
    let value: serde_json::Value = serde_json::from_slice(bytes)
        .map_err(|_| ServiceError::new("commit_publication_failed", "invalid snapshot bytes"))?;
    value
        .get("snapshot_id")
        .and_then(serde_json::Value::as_str)
        .map(str::to_owned)
        .ok_or_else(|| ServiceError::new("commit_publication_failed", "missing snapshot identity"))
}
fn io_error(error: std::io::Error) -> ServiceError {
    ServiceError::new("persistence_io_failed", error.to_string())
}
fn platform_error(error: platform::PlatformValidationError) -> ServiceError {
    ServiceError::new(error.code, error.message)
}
fn platform_failure(error: platform::PlatformFailure) -> ServiceError {
    ServiceError::new("compiler_platform_failed", error.message)
}
fn platform_serialization(error: platform::PlatformSerializationError) -> ServiceError {
    ServiceError::new("compiler_platform_failed", error.message)
}
fn cache_operation_error(reason: crate::persistent_cache::CacheReasonCodeV1) -> ServiceError {
    ServiceError::new("cache_operation_failed", reason.code())
}
fn workspace_error(error: workspace::WorkspaceErrorV1) -> ServiceError {
    ServiceError::new(error.code(), "workspace manifest validation failed")
}

pub mod protocol {
    pub use super::{
        decode_frame, encode_frame, CompilerServiceDescriptor, ServiceError, ServiceOperation,
        ServiceResponseStatus,
    };
}
pub mod host {
    pub use super::{
        CompileRequest, CompileResponse, CompilerServiceHost, CompleteSource,
        IncrementalExecutionReportV1, IncrementalReportSelector,
    };
}
pub mod session_store {
    pub use super::CompilerServiceHost;
}
pub mod journal {
    pub use super::CompilerServiceHost;
}
pub mod transport {
    pub use super::{decode_frame, encode_frame};
}
pub mod inspection {
    pub use super::CompilerServiceDescriptor;
}

#[cfg(test)]
mod tests {
    use super::*;

    fn contract() -> ContractVersion {
        ContractVersion::new("presolve-compiler:0.1.0-alpha")
    }
    fn request(
        configuration: WorkspaceConfiguration,
        sources: Vec<(&str, &str)>,
        report: IncrementalReportSelector,
        verify: bool,
    ) -> CompileRequest {
        let input = WorkspaceInput {
            configuration: configuration.clone(),
            sources: sources
                .iter()
                .map(|(path, source)| WorkspaceSource {
                    path: (*path).into(),
                    source: (*source).into(),
                    language: None,
                })
                .collect(),
            compiler_contract: contract(),
        };
        CompileRequest {
            configuration,
            candidate_snapshot: WorkspaceSnapshot::from_input(&input).unwrap(),
            sources: sources
                .into_iter()
                .map(|(path, source)| CompleteSource {
                    path: path.into(),
                    source: source.into(),
                    language: None,
                })
                .collect(),
            mode: RequestedCompilationMode::Automatic,
            incremental_report: report,
            verify_exact_equivalence: verify,
            cache_report: CacheReportSelector::None,
        }
    }
    fn root(label: &str) -> std::path::PathBuf {
        let root = std::env::temp_dir().join(format!(
            "presolve-service-l5-{label}-{}",
            NEXT_SERVICE.fetch_add(1, Ordering::Relaxed)
        ));
        let _ = std::fs::remove_dir_all(&root);
        root
    }
    fn opened_host(
        root: &std::path::Path,
        configuration: &WorkspaceConfiguration,
    ) -> (CompilerServiceHost, String) {
        let workspace = platform::derive_workspace_id_v1(configuration).unwrap();
        let mut host = CompilerServiceHost::start(root, contract()).unwrap();
        let session = host
            .open_session(configuration.clone(), &workspace)
            .unwrap();
        (host, session)
    }
    fn read_tree(path: &Path) -> Vec<u8> {
        if path.is_file() {
            return std::fs::read(path).unwrap();
        }
        std::fs::read_dir(path)
            .unwrap()
            .flat_map(|entry| read_tree(&entry.unwrap().path()))
            .collect()
    }

    #[test]
    fn frames_are_exactly_length_delimited() {
        let frame = encode_frame(br#"{"protocol_version":1}"#).unwrap();
        assert_eq!(decode_frame(&frame).unwrap(), br#"{"protocol_version":1}"#);
        assert!(decode_frame(b"00000000:\n").is_err());
        assert!(decode_frame(b"00000002:{}x\n").is_err());
    }

    #[test]
    fn complete_candidate_commit_is_durable_without_source_persistence() {
        let root = std::env::temp_dir().join(format!(
            "presolve-service-test-{}",
            NEXT_SERVICE.fetch_add(1, Ordering::Relaxed)
        ));
        let configuration = WorkspaceConfiguration::default();
        let workspace = platform::derive_workspace_id_v1(&configuration).unwrap();
        let contract = ContractVersion::new("presolve-compiler:0.1.0-alpha");
        let mut host = CompilerServiceHost::start(&root, contract.clone()).unwrap();
        let session = host
            .open_session(configuration.clone(), &workspace)
            .unwrap();
        let input = WorkspaceInput {
            configuration: configuration.clone(),
            sources: vec![WorkspaceSource {
                path: "src/App.tsx".into(),
                source: "export class App {}".into(),
                language: None,
            }],
            compiler_contract: contract,
        };
        let snapshot = WorkspaceSnapshot::from_input(&input).unwrap();
        let result = host
            .compile(
                &session,
                0,
                CompileRequest {
                    configuration,
                    candidate_snapshot: snapshot,
                    sources: vec![CompleteSource {
                        path: "src/App.tsx".into(),
                        source: "export class App {}".into(),
                        language: None,
                    }],
                    mode: RequestedCompilationMode::Full,
                    incremental_report: IncrementalReportSelector::None,
                    verify_exact_equivalence: false,
                    cache_report: CacheReportSelector::None,
                },
            )
            .unwrap();
        assert_eq!(result.commit_sequence, 1);
        let persisted = root
            .join("service/sessions")
            .join(session.strip_prefix("session:sha256:").unwrap())
            .join("commits/00000000000000000001");
        assert!(persisted.join("workspace-snapshot.json").is_file());
        assert!(persisted.join("workspace-graph.json").is_file());
        assert!(
            std::fs::metadata(persisted.join("commit.json"))
                .unwrap()
                .len()
                > 0
        );
        assert!(!String::from_utf8_lossy(&read_tree(&persisted)).contains("export class App {}"));
        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn l5_content_edit_reuses_validated_parse_products_and_equals_clean() {
        let root = root("content");
        let configuration = WorkspaceConfiguration::default();
        let (mut host, session) = opened_host(&root, &configuration);
        let baseline = vec![
            ("src/Dependency.ts", "export class Dependency {}"),
            (
                "src/App.ts",
                "import { Dependency } from './Dependency'; export class App {}",
            ),
            ("src/Stable.ts", "export class Stable {}"),
        ];
        let first = host
            .compile(
                &session,
                0,
                request(
                    configuration.clone(),
                    baseline.clone(),
                    IncrementalReportSelector::Full,
                    true,
                ),
            )
            .unwrap();
        assert_eq!(first.mode, "cold");
        let report = first.incremental_report.unwrap();
        assert_eq!(report.mode, IncrementalCompilationModeV1::Cold);
        assert_eq!(report.exact_equivalence_verified, Some(true));
        let candidate = vec![
            (
                "src/Dependency.ts",
                "export class Dependency { value = 1; }",
            ),
            (
                "src/App.ts",
                "import { Dependency } from './Dependency'; export class App {}",
            ),
            ("src/Stable.ts", "export class Stable {}"),
        ];
        let second = host
            .compile(
                &session,
                1,
                request(
                    configuration.clone(),
                    candidate.clone(),
                    IncrementalReportSelector::Full,
                    true,
                ),
            )
            .unwrap();
        let report = second.incremental_report.unwrap();
        assert_eq!(second.mode, "incremental");
        assert!(!report.reused_product_identities.is_empty());
        assert_eq!(report.exact_equivalence_verified, Some(true));
        let no_change = host
            .compile(
                &session,
                2,
                request(
                    configuration,
                    candidate,
                    IncrementalReportSelector::Full,
                    true,
                ),
            )
            .unwrap();
        assert_eq!(no_change.commit_sequence, 2);
        assert_eq!(no_change.mode, "no_change");
        assert_eq!(
            no_change.snapshot.to_canonical_json().unwrap(),
            second.snapshot.to_canonical_json().unwrap()
        );
        host.close_session(&session).unwrap();
        let closed = host.sessions.get(&session).unwrap();
        assert!(closed.closed);
        assert!(closed.incremental_baseline.is_none());
        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn l5_service_restart_has_no_durable_baseline() {
        let root = root("restart");
        let configuration = WorkspaceConfiguration::default();
        let baseline = vec![("src/App.ts", "export class App {}")];
        let (mut first, first_session) = opened_host(&root, &configuration);
        first
            .compile(
                &first_session,
                0,
                request(
                    configuration.clone(),
                    baseline.clone(),
                    IncrementalReportSelector::None,
                    false,
                ),
            )
            .unwrap();
        drop(first);
        let (mut restarted, session) = opened_host(&root, &configuration);
        let response = restarted
            .compile(
                &session,
                0,
                request(
                    configuration,
                    baseline,
                    IncrementalReportSelector::Full,
                    true,
                ),
            )
            .unwrap();
        assert_eq!(response.mode, "cold");
        assert!(response
            .incremental_report
            .unwrap()
            .reused_product_identities
            .is_empty());
        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn l6_persistent_complete_result_cache_hits_after_restart_without_source_text() {
        let root = root("l6-service");
        let cache_root = root.join("explicit-cache");
        let configuration = WorkspaceConfiguration::default();
        let workspace = platform::derive_workspace_id_v1(&configuration).unwrap();
        let sentinel = "L6_SOURCE_SENTINEL_IDENTIFIER_COMMENT_STRING";
        let source = format!("// {sentinel}\nexport class App {{ value = '{sentinel}'; }}");
        let mut first =
            CompilerServiceHost::start_with_cache(&root, Some(&cache_root), contract()).unwrap();
        let session = first
            .open_session(configuration.clone(), &workspace)
            .unwrap();
        let mut clean_request = request(
            configuration.clone(),
            vec![("src/App.ts", source.as_str())],
            IncrementalReportSelector::None,
            true,
        );
        clean_request.cache_report = CacheReportSelector::Full;
        let clean = first.compile(&session, 0, clean_request).unwrap();
        assert!(clean.cache_report.as_ref().unwrap().entry_published);
        assert!(cache_root.join("manifest.json").is_file());
        drop(first);

        let mut restarted =
            CompilerServiceHost::start_with_cache(&root, Some(&cache_root), contract()).unwrap();
        let session = restarted
            .open_session(configuration.clone(), &workspace)
            .unwrap();
        let mut hit_request = request(
            configuration,
            vec![("src/App.ts", source.as_str())],
            IncrementalReportSelector::None,
            false,
        );
        hit_request.cache_report = CacheReportSelector::Full;
        let hit = restarted.compile(&session, 0, hit_request).unwrap();
        assert_eq!(
            hit.cache_report.as_ref().unwrap().outcome,
            CacheOutcomeV1::Hit
        );
        assert_eq!(
            clean.snapshot.to_canonical_json().unwrap(),
            hit.snapshot.to_canonical_json().unwrap()
        );
        assert_eq!(
            clean.graph.to_canonical_json().unwrap(),
            hit.graph.to_canonical_json().unwrap()
        );
        let cache_bytes = read_tree(&cache_root);
        assert!(!String::from_utf8_lossy(&cache_bytes).contains(sentinel));
        let report = restarted.inspect_cache(&cache_root).unwrap();
        assert_eq!(report.valid_keys.len(), 1);
        assert_eq!(restarted.clean_cache(&cache_root).unwrap().len(), 1);
        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn l6_corruption_and_disabled_cache_fall_back_to_l5() {
        let root = root("l6-corrupt");
        let cache_root = root.join("explicit-cache");
        let configuration = WorkspaceConfiguration::default();
        let workspace = platform::derive_workspace_id_v1(&configuration).unwrap();
        let sources = vec![("src/App.ts", "export class App {}")];
        let mut host =
            CompilerServiceHost::start_with_cache(&root, Some(&cache_root), contract()).unwrap();
        let session = host
            .open_session(configuration.clone(), &workspace)
            .unwrap();
        host.compile(
            &session,
            0,
            request(
                configuration.clone(),
                sources.clone(),
                IncrementalReportSelector::None,
                false,
            ),
        )
        .unwrap();
        drop(host);
        let payload = cache_root.join("entries");
        let prefix = std::fs::read_dir(&payload)
            .unwrap()
            .next()
            .unwrap()
            .unwrap()
            .path();
        let entry = std::fs::read_dir(prefix)
            .unwrap()
            .next()
            .unwrap()
            .unwrap()
            .path()
            .join("payload.bin");
        std::fs::write(&entry, b"corrupt").unwrap();
        let mut restarted =
            CompilerServiceHost::start_with_cache(&root, Some(&cache_root), contract()).unwrap();
        let session = restarted
            .open_session(configuration.clone(), &workspace)
            .unwrap();
        let mut corrupt_request = request(
            configuration.clone(),
            sources.clone(),
            IncrementalReportSelector::None,
            true,
        );
        corrupt_request.cache_report = CacheReportSelector::Full;
        let result = restarted.compile(&session, 0, corrupt_request).unwrap();
        assert_ne!(result.cache_report.unwrap().outcome, CacheOutcomeV1::Hit);
        drop(restarted);
        let mut disabled = CompilerServiceHost::start(&root, contract()).unwrap();
        let session = disabled.open_session(configuration, &workspace).unwrap();
        let mut request = request(
            WorkspaceConfiguration::default(),
            sources,
            IncrementalReportSelector::None,
            true,
        );
        request.cache_report = CacheReportSelector::Full;
        assert_eq!(
            disabled
                .compile(&session, 0, request)
                .unwrap()
                .cache_report
                .unwrap()
                .outcome,
            CacheOutcomeV1::Miss
        );
        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn l5_add_delete_configuration_and_malformed_baselines_clean_fallback() {
        let root = root("fallback");
        let configuration = WorkspaceConfiguration::default();
        let (mut host, session) = opened_host(&root, &configuration);
        let baseline = vec![("src/App.ts", "export class App {}")];
        host.compile(
            &session,
            0,
            request(
                configuration.clone(),
                baseline.clone(),
                IncrementalReportSelector::None,
                false,
            ),
        )
        .unwrap();
        let added = host
            .compile(
                &session,
                1,
                request(
                    configuration.clone(),
                    vec![
                        ("src/App.ts", "export class App {}"),
                        ("src/Added.ts", "export class Added {}"),
                    ],
                    IncrementalReportSelector::Full,
                    true,
                ),
            )
            .unwrap();
        let report = added.incremental_report.unwrap();
        assert_eq!(added.mode, "clean_fallback");
        assert!(report
            .fallback_reasons
            .contains(&IncrementalFallbackReasonV1::SourceUniverseMembershipUnmodeled));
        let mut changed_configuration = configuration.clone();
        changed_configuration.feature_flags.push("strict".into());
        let config_changed = host
            .compile(
                &session,
                2,
                request(
                    changed_configuration,
                    vec![
                        ("src/App.ts", "export class App {}"),
                        ("src/Added.ts", "export class Added {}"),
                    ],
                    IncrementalReportSelector::Full,
                    true,
                ),
            )
            .unwrap();
        assert!(config_changed
            .incremental_report
            .unwrap()
            .fallback_reasons
            .contains(&IncrementalFallbackReasonV1::ConfigurationChanged));
        host.sessions
            .get_mut(&session)
            .unwrap()
            .incremental_baseline
            .as_mut()
            .unwrap()
            .publication_identity = "malformed".into();
        let malformed = host
            .compile(
                &session,
                3,
                request(
                    configuration,
                    baseline,
                    IncrementalReportSelector::Full,
                    true,
                ),
            )
            .unwrap();
        assert!(malformed
            .incremental_report
            .unwrap()
            .fallback_reasons
            .contains(&IncrementalFallbackReasonV1::MalformedBaselineGraph));
        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn l5_failure_isolation_no_persistence_and_twenty_run_determinism() {
        let configuration = WorkspaceConfiguration::default();
        let baseline = vec![
            ("src/Dependency.ts", "export class Dependency {}"),
            (
                "src/App.ts",
                "import { Dependency } from './Dependency'; export class App {}",
            ),
        ];
        let candidate = vec![
            (
                "src/Dependency.ts",
                "export class Dependency { value = 1; }",
            ),
            (
                "src/App.ts",
                "import { Dependency } from './Dependency'; export class App {}",
            ),
        ];
        let mut expected_report = None;
        for run in 0..20 {
            let root = root("determinism");
            let (mut host, session) = opened_host(&root, &configuration);
            host.compile(
                &session,
                0,
                request(
                    configuration.clone(),
                    baseline.clone(),
                    IncrementalReportSelector::None,
                    false,
                ),
            )
            .unwrap();
            let mut invalid = request(
                configuration.clone(),
                baseline.clone(),
                IncrementalReportSelector::None,
                false,
            );
            invalid.sources = vec![
                CompleteSource {
                    path: "src/App.ts".into(),
                    source: "export class App {}".into(),
                    language: None,
                },
                CompleteSource {
                    path: "src/App.ts".into(),
                    source: "export class Duplicate {}".into(),
                    language: None,
                },
            ];
            assert!(host.compile(&session, 1, invalid).is_err());
            let response = host
                .compile(
                    &session,
                    1,
                    request(
                        configuration.clone(),
                        candidate.clone(),
                        IncrementalReportSelector::Full,
                        true,
                    ),
                )
                .unwrap();
            let report = response.incremental_report.unwrap().to_canonical_json();
            if let Some(expected) = &expected_report {
                assert_eq!(&report, expected, "determinism run {run}");
            } else {
                expected_report = Some(report);
            }
            let persisted = read_tree(&root.join("service/sessions"));
            assert!(!String::from_utf8_lossy(&persisted).contains("value = 1"));
            std::fs::remove_dir_all(root).unwrap();
        }
    }
}
