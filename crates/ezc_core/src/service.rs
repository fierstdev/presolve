//! Local, durable compiler-service host (Phase L4).
//!
//! The host accepts complete request-owned workspace inputs and delegates every
//! compilation to `platform::CompilerSessionState`; it never reads workspace
//! files or persists source text.

#![allow(
    clippy::missing_errors_doc,
    clippy::needless_pass_by_value,
    clippy::too_many_lines
)]

use std::collections::BTreeMap;
use std::fs::{self, File, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

use sha2::{Digest as _, Sha256};

use crate::platform::{
    self, CacheLimits, CancellationToken, CompilationOutcome, CompileWorkspaceRequest,
    CompilerSessionState, ContractVersion, RequestedCompilationMode, WorkspaceConfiguration,
    WorkspaceGraph, WorkspaceId, WorkspaceInput, WorkspaceSnapshot, WorkspaceSource,
};

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
}
#[derive(Debug, Clone)]
pub struct CompileResponse {
    pub commit_sequence: u64,
    pub snapshot: Arc<WorkspaceSnapshot>,
    pub graph: Arc<WorkspaceGraph>,
    pub mode: String,
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
}
struct DurableSession {
    workspace_id: WorkspaceId,
    configuration: WorkspaceConfiguration,
    compiler_contract: ContractVersion,
    commit_sequence: u64,
    closed: bool,
    l3: CompilerSessionState,
}
impl CompilerServiceHost {
    pub fn start(
        root: impl AsRef<Path>,
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
        };
        self.write_session(&id, &session)?;
        append_journal(&self.root, &id, 1, "session_created", 0, None)?;
        self.sessions.insert(id.clone(), session);
        Ok(id)
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
        let config = platform::canonical_workspace_configuration_json_v1(&request.configuration)
            .map_err(|error| ServiceError::new("invalid_request", error.message))?;
        let retained = platform::canonical_workspace_configuration_json_v1(&session.configuration)
            .map_err(|error| ServiceError::new("internal_invariant_failed", error.message))?;
        if config != retained {
            return Err(ServiceError::new(
                "workspace_mismatch",
                "request configuration differs from durable session",
            ));
        }
        let derived = WorkspaceSnapshot::from_input(&WorkspaceInput {
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
        })
        .map_err(platform_failure)?;
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
        let outcome = session.l3.compile_workspace(CompileWorkspaceRequest {
            workspace: WorkspaceInput {
                configuration: request.configuration,
                sources: request
                    .sources
                    .into_iter()
                    .map(|source| WorkspaceSource {
                        path: source.path,
                        source: source.source,
                        language: source.language,
                    })
                    .collect(),
                compiler_contract: session.compiler_contract.clone(),
            },
            mode: request.mode,
            cancellation: CancellationToken::new(),
        });
        let CompilationOutcome::Committed(committed) = outcome else {
            return Err(ServiceError::new(
                "compiler_platform_failed",
                "compiler did not commit",
            ));
        };
        session.commit_sequence += 1;
        publish_commit(
            &root,
            session_id,
            session,
            session.commit_sequence,
            &committed.snapshot,
            &committed.graph,
        )?;
        Ok(CompileResponse {
            commit_sequence: session.commit_sequence,
            snapshot: committed.snapshot,
            graph: committed.graph,
            mode: match committed.plan.mode {
                platform::IncrementalMode::NoOp => "no_op",
                platform::IncrementalMode::Incremental => "incremental",
                platform::IncrementalMode::Full => "full",
            }
            .into(),
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

pub mod protocol {
    pub use super::{
        decode_frame, encode_frame, CompilerServiceDescriptor, ServiceError, ServiceOperation,
        ServiceResponseStatus,
    };
}
pub mod host {
    pub use super::{CompileRequest, CompileResponse, CompilerServiceHost, CompleteSource};
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
        std::fs::remove_dir_all(root).unwrap();
    }
}
