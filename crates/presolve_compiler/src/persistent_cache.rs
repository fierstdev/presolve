//! L6 source-free, complete-result persistent cache.
//!
//! This module only encodes and validates canonical L3 products. It never
//! reads authored inputs, invokes parsing, or interprets compiler semantics.

#![allow(clippy::missing_errors_doc, clippy::too_many_lines)]

use std::fs::{self, File, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};

use sha2::{Digest as _, Sha256};

use crate::platform::{self, ContractVersion, WorkspaceGraph, WorkspaceSnapshot};

pub const PERSISTENT_ARTIFACT_CACHE_V1_SCHEMA: &str = "presolve.persistent-artifact-cache.v1";
pub const CACHE_MANIFEST_V1_SCHEMA: &str = "presolve.cache-manifest.v1";
pub const CACHE_ENTRY_ENVELOPE_V1_SCHEMA: &str = "presolve.cache-entry-envelope.v1";
pub const CACHE_INSPECTION_REPORT_V1_SCHEMA: &str = "presolve.cache-inspection-report.v1";
pub const CACHE_SCHEMA_VERSION: u32 = 1;
const PAYLOAD_CODEC: &str = "presolve.complete-result-payload.v1";
const PAYLOAD_MAGIC: &[u8] = b"PSL6\0";

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum CacheReasonCodeV1 {
    Disabled,
    RootUnavailable,
    ManifestMissing,
    ManifestIncompatible,
    LockUnavailable,
    EntryAbsent,
    EntryIncomplete,
    EnvelopeIncompatible,
    EnvelopeChecksumMismatch,
    KeyMismatch,
    CompatibilityMismatch,
    PayloadLengthMismatch,
    PayloadChecksumMismatch,
    PayloadDecodeFailure,
    CanonicalProductValidationFailure,
    ArtifactChecksumMismatch,
    RequestFingerprintMismatch,
    ResultFingerprintMismatch,
    PublicationIoFailure,
    EntryAlreadyValid,
    EntryReplaced,
    HitValidated,
    CleanupRefused,
    VerificationInvalid,
}
impl CacheReasonCodeV1 {
    #[must_use]
    pub const fn code(self) -> &'static str {
        match self {
            Self::Disabled => "L6C001",
            Self::RootUnavailable => "L6C002",
            Self::ManifestMissing => "L6C003",
            Self::ManifestIncompatible => "L6C004",
            Self::LockUnavailable => "L6C005",
            Self::EntryAbsent => "L6C006",
            Self::EntryIncomplete => "L6C007",
            Self::EnvelopeIncompatible => "L6C008",
            Self::EnvelopeChecksumMismatch => "L6C009",
            Self::KeyMismatch => "L6C010",
            Self::CompatibilityMismatch => "L6C011",
            Self::PayloadLengthMismatch => "L6C012",
            Self::PayloadChecksumMismatch => "L6C013",
            Self::PayloadDecodeFailure => "L6C014",
            Self::CanonicalProductValidationFailure => "L6C015",
            Self::ArtifactChecksumMismatch => "L6C016",
            Self::RequestFingerprintMismatch => "L6C017",
            Self::ResultFingerprintMismatch => "L6C018",
            Self::PublicationIoFailure => "L6C019",
            Self::EntryAlreadyValid => "L6C020",
            Self::EntryReplaced => "L6C021",
            Self::HitValidated => "L6C022",
            Self::CleanupRefused => "L6C023",
            Self::VerificationInvalid => "L6C024",
        }
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CacheOutcomeV1 {
    NotChecked,
    Hit,
    Miss,
    WriteFailed,
}
impl CacheOutcomeV1 {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::NotChecked => "not_checked",
            Self::Hit => "hit",
            Self::Miss => "miss",
            Self::WriteFailed => "write_failed",
        }
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CacheReportSelector {
    None,
    Summary,
    Full,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CacheTelemetryV1 {
    pub enabled: bool,
    pub outcome: CacheOutcomeV1,
    pub reasons: Vec<CacheReasonCodeV1>,
    pub cache_key: String,
    pub payload_length: Option<u64>,
    pub result_fingerprint: Option<String>,
    pub entry_published: bool,
    pub entry_replaced: bool,
}
#[derive(Debug, Clone)]
pub struct CacheKeyInputV1 {
    pub compiler_contract: ContractVersion,
    pub configuration_fingerprint: String,
    pub source_universe_fingerprint: String,
    pub compile_mode: &'static str,
}
impl CacheKeyInputV1 {
    /// # Panics
    ///
    /// Panics only if this fixed L6 field list exceeds the `u32` wire ordinal
    /// space, which is impossible for the contract-defined list.
    #[must_use]
    pub fn key(&self) -> String {
        let fields = [
            "presolve.compile-result-cache.v1",
            concat!(env!("CARGO_PKG_NAME"), ":", env!("CARGO_PKG_VERSION")),
            self.compiler_contract.as_str(),
            "compiler-service-protocol:1",
            "workspace-snapshot:1",
            "workspace-graph:1",
            "l5-plan:1",
            "l5-report:1",
            "features:default",
            "target:default",
            self.configuration_fingerprint.as_str(),
            self.source_universe_fingerprint.as_str(),
            self.compile_mode,
            "artifacts:phase-k-frozen",
            "diagnostics:phase-k-frozen",
            PAYLOAD_CODEC,
        ];
        let mut bytes = Vec::new();
        for (ordinal, field) in fields.iter().enumerate() {
            bytes.extend_from_slice(
                &u32::try_from(ordinal)
                    .expect("fixed cache-key fields")
                    .to_be_bytes(),
            );
            bytes.extend_from_slice(&(field.len() as u64).to_be_bytes());
            bytes.extend_from_slice(field.as_bytes());
        }
        format!("{:x}", Sha256::digest(bytes))
    }
    #[must_use]
    pub fn digest(&self) -> String {
        format!("sha256:{:x}", Sha256::digest(self.key().as_bytes()))
    }
}
#[derive(Debug, Clone)]
pub struct CachedCompileResultV1 {
    pub snapshot: WorkspaceSnapshot,
    pub graph: WorkspaceGraph,
    pub response_mode: String,
}
#[derive(Debug, Clone)]
pub struct CacheInspectionReportV1 {
    pub enabled: bool,
    pub manifest_valid: bool,
    pub valid_keys: Vec<String>,
    pub invalid: Vec<(String, CacheReasonCodeV1)>,
    pub total_payload_bytes: u64,
    pub total_artifact_bytes: u64,
    pub compatibility: Vec<String>,
    pub report_fingerprint: String,
}
impl CacheInspectionReportV1 {
    /// # Panics
    ///
    /// Panics only if serializing an owned Rust string fails.
    #[must_use]
    pub fn to_canonical_json(&self) -> Vec<u8> {
        let q = |s: &str| serde_json::to_string(s).expect("strings serialize");
        let valid = self
            .valid_keys
            .iter()
            .map(|key| q(key))
            .collect::<Vec<_>>()
            .join(",");
        let invalid = self
            .invalid
            .iter()
            .map(|(key, reason)| format!("{{\"key\":{},\"reason\":{}}}", q(key), q(reason.code())))
            .collect::<Vec<_>>()
            .join(",");
        let compatibility = self
            .compatibility
            .iter()
            .map(|item| q(item))
            .collect::<Vec<_>>()
            .join(",");
        format!("{{\"schema\":{},\"schema_version\":1,\"enabled\":{},\"manifest_valid\":{},\"valid_keys\":[{}],\"invalid_entries\":[{}],\"total_validated_payload_bytes\":{},\"total_validated_artifact_bytes\":{},\"compatibility\":[{}],\"report_fingerprint\":{}}}\n",q(CACHE_INSPECTION_REPORT_V1_SCHEMA),self.enabled,self.manifest_valid,valid,invalid,self.total_payload_bytes,self.total_artifact_bytes,compatibility,q(&self.report_fingerprint)).into_bytes()
    }
}

pub struct PersistentArtifactCacheV1 {
    state: CacheState,
}
enum CacheState {
    Disabled(CacheReasonCodeV1),
    Enabled { root: PathBuf, lock: PathBuf },
}
impl Drop for PersistentArtifactCacheV1 {
    fn drop(&mut self) {
        if let CacheState::Enabled { lock, .. } = &self.state {
            let _ = fs::remove_file(lock);
        }
    }
}
impl PersistentArtifactCacheV1 {
    #[must_use]
    pub fn disabled() -> Self {
        Self {
            state: CacheState::Disabled(CacheReasonCodeV1::Disabled),
        }
    }
    #[must_use]
    pub fn open(explicit_root: Option<&Path>, compiler: &ContractVersion) -> Self {
        let Some(root) = explicit_root else {
            return Self::disabled();
        };
        if fs::create_dir_all(root).is_err() {
            return Self {
                state: CacheState::Disabled(CacheReasonCodeV1::RootUnavailable),
            };
        }
        let Ok(root) = root.canonicalize() else {
            return Self {
                state: CacheState::Disabled(CacheReasonCodeV1::RootUnavailable),
            };
        };
        let manifest = root.join("manifest.json");
        if !manifest.exists() {
            let non_empty = fs::read_dir(&root)
                .ok()
                .is_some_and(|mut entries| entries.next().is_some());
            if non_empty {
                return Self {
                    state: CacheState::Disabled(CacheReasonCodeV1::ManifestMissing),
                };
            }
            if atomic_write(&manifest, manifest_json(compiler).as_bytes()).is_err() {
                return Self {
                    state: CacheState::Disabled(CacheReasonCodeV1::RootUnavailable),
                };
            }
        }
        if !valid_manifest(&manifest, compiler) {
            return Self {
                state: CacheState::Disabled(CacheReasonCodeV1::ManifestIncompatible),
            };
        }
        let lock = root.join(".l6.lock");
        if OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&lock)
            .is_err()
        {
            return Self {
                state: CacheState::Disabled(CacheReasonCodeV1::LockUnavailable),
            };
        }
        if fs::create_dir_all(root.join("entries")).is_err() {
            let _ = fs::remove_file(&lock);
            return Self {
                state: CacheState::Disabled(CacheReasonCodeV1::RootUnavailable),
            };
        }
        Self {
            state: CacheState::Enabled { root, lock },
        }
    }
    #[must_use]
    pub fn enabled(&self) -> bool {
        matches!(self.state, CacheState::Enabled { .. })
    }
    #[must_use]
    pub fn lookup(
        &self,
        input: &CacheKeyInputV1,
    ) -> (Option<CachedCompileResultV1>, CacheTelemetryV1) {
        let key = input.key();
        let disabled = |reason| {
            (
                None,
                telemetry(false, CacheOutcomeV1::Miss, key.clone(), vec![reason]),
            )
        };
        let CacheState::Enabled { root, .. } = &self.state else {
            return match self.state {
                CacheState::Disabled(reason) => disabled(reason),
                CacheState::Enabled { .. } => unreachable!(),
            };
        };
        let dir = entry_dir(root, &key);
        if !dir.exists() {
            return (
                None,
                telemetry(
                    true,
                    CacheOutcomeV1::Miss,
                    key,
                    vec![CacheReasonCodeV1::EntryAbsent],
                ),
            );
        }
        let (result, length, result_fingerprint) = match read_entry(&dir, &key, input) {
            Ok(value) => value,
            Err(reason) => {
                return (
                    None,
                    telemetry(true, CacheOutcomeV1::Miss, key, vec![reason]),
                )
            }
        };
        let mut value = telemetry(
            true,
            CacheOutcomeV1::Hit,
            key,
            vec![CacheReasonCodeV1::HitValidated],
        );
        value.payload_length = Some(length);
        value.result_fingerprint = Some(result_fingerprint);
        (Some(result), value)
    }
    /// # Panics
    ///
    /// Panics only if the cache module constructs an invalid internal entry path.
    #[must_use]
    pub fn publish(
        &self,
        input: &CacheKeyInputV1,
        result: &CachedCompileResultV1,
    ) -> CacheTelemetryV1 {
        let key = input.key();
        let CacheState::Enabled { root, .. } = &self.state else {
            return telemetry(
                false,
                CacheOutcomeV1::Miss,
                key,
                vec![match self.state {
                    CacheState::Disabled(reason) => reason,
                    CacheState::Enabled { .. } => unreachable!(),
                }],
            );
        };
        let target = entry_dir(root, &key);
        if target.exists() && read_entry(&target, &key, input).is_ok() {
            return telemetry(
                true,
                CacheOutcomeV1::Miss,
                key,
                vec![CacheReasonCodeV1::EntryAlreadyValid],
            );
        }
        let Ok(payload) = encode_payload(result) else {
            return telemetry(
                true,
                CacheOutcomeV1::WriteFailed,
                key,
                vec![CacheReasonCodeV1::PublicationIoFailure],
            );
        };
        let result_fingerprint = result_fingerprint(result).unwrap_or_default();
        let envelope = entry_json(&key, input, &payload, &result_fingerprint).into_bytes();
        let parent = target.parent().expect("entry parent");
        if fs::create_dir_all(parent).is_err()
            || publish_files(parent, &target, &payload, &envelope).is_err()
        {
            return telemetry(
                true,
                CacheOutcomeV1::WriteFailed,
                key,
                vec![CacheReasonCodeV1::PublicationIoFailure],
            );
        }
        let mut value = telemetry(true, CacheOutcomeV1::Miss, key, vec![]);
        value.entry_published = true;
        value.payload_length = Some(payload.len() as u64);
        value.result_fingerprint = Some(result_fingerprint);
        value
    }
    pub fn inspect(&self) -> Result<CacheInspectionReportV1, CacheReasonCodeV1> {
        inspect_state(&self.state)
    }
    pub fn verify(&self) -> Result<CacheInspectionReportV1, CacheReasonCodeV1> {
        self.inspect()
    }
    pub fn clean(&self) -> Result<Vec<String>, CacheReasonCodeV1> {
        let CacheState::Enabled { root, .. } = &self.state else {
            return Err(CacheReasonCodeV1::CleanupRefused);
        };
        if !valid_manifest(
            &root.join("manifest.json"),
            &ContractVersion::new(format!("presolve-compiler:{}", env!("CARGO_PKG_VERSION"))),
        ) {
            return Err(CacheReasonCodeV1::CleanupRefused);
        }
        let report = self.inspect()?;
        let entries = root.join("entries");
        fs::remove_dir_all(&entries).map_err(|_| CacheReasonCodeV1::RootUnavailable)?;
        fs::create_dir_all(&entries).map_err(|_| CacheReasonCodeV1::RootUnavailable)?;
        Ok(report.valid_keys)
    }
}

fn telemetry(
    enabled: bool,
    outcome: CacheOutcomeV1,
    cache_key: String,
    reasons: Vec<CacheReasonCodeV1>,
) -> CacheTelemetryV1 {
    CacheTelemetryV1 {
        enabled,
        outcome,
        reasons,
        cache_key,
        payload_length: None,
        result_fingerprint: None,
        entry_published: false,
        entry_replaced: false,
    }
}
fn entry_dir(root: &Path, key: &str) -> PathBuf {
    root.join("entries").join(&key[..2]).join(key)
}
fn manifest_json(compiler: &ContractVersion) -> String {
    let without = format!("{{\"schema\":\"{}\",\"schema_version\":1,\"product_identity\":\"presolve\",\"cache_format\":\"{}\",\"hash_algorithm\":\"sha256\",\"payload_codec\":\"{}\",\"entry_schema\":\"{}\",\"entry_schema_version\":1,\"creation_compatibility\":{}}}",CACHE_MANIFEST_V1_SCHEMA,PERSISTENT_ARTIFACT_CACHE_V1_SCHEMA,PAYLOAD_CODEC,CACHE_ENTRY_ENVELOPE_V1_SCHEMA,q(compiler.as_str()));
    format!(
        "{},\"manifest_checksum\":{}}}\n",
        without.strip_suffix('}').expect("object"),
        q(&checksum(without.as_bytes()))
    )
}
fn valid_manifest(path: &Path, compiler: &ContractVersion) -> bool {
    let Ok(bytes) = fs::read(path) else {
        return false;
    };
    let Ok(value) = serde_json::from_slice::<serde_json::Value>(&bytes) else {
        return false;
    };
    let Some(object) = value.as_object() else {
        return false;
    };
    object.get("schema").and_then(serde_json::Value::as_str) == Some(CACHE_MANIFEST_V1_SCHEMA)
        && object
            .get("schema_version")
            .and_then(serde_json::Value::as_u64)
            == Some(1)
        && object
            .get("creation_compatibility")
            .and_then(serde_json::Value::as_str)
            == Some(compiler.as_str())
        && object
            .get("manifest_checksum")
            .and_then(serde_json::Value::as_str)
            .is_some()
}
fn encode_payload(
    result: &CachedCompileResultV1,
) -> Result<Vec<u8>, platform::PlatformSerializationError> {
    let snapshot = result.snapshot.to_canonical_json()?;
    let graph = result.graph.to_canonical_json()?;
    let mut out = PAYLOAD_MAGIC.to_vec();
    for field in [
        result.response_mode.as_bytes(),
        snapshot.as_slice(),
        graph.as_slice(),
    ] {
        out.extend_from_slice(&(field.len() as u64).to_be_bytes());
        out.extend_from_slice(field);
    }
    Ok(out)
}
fn decode_payload(bytes: &[u8]) -> Result<CachedCompileResultV1, CacheReasonCodeV1> {
    if !bytes.starts_with(PAYLOAD_MAGIC) {
        return Err(CacheReasonCodeV1::PayloadDecodeFailure);
    }
    let mut cursor = PAYLOAD_MAGIC.len();
    let mut next = || -> Result<Vec<u8>, CacheReasonCodeV1> {
        if cursor + 8 > bytes.len() {
            return Err(CacheReasonCodeV1::PayloadDecodeFailure);
        }
        let length = usize::try_from(u64::from_be_bytes(
            bytes[cursor..cursor + 8]
                .try_into()
                .map_err(|_| CacheReasonCodeV1::PayloadDecodeFailure)?,
        ))
        .map_err(|_| CacheReasonCodeV1::PayloadDecodeFailure)?;
        cursor += 8;
        if cursor
            .checked_add(length)
            .is_none_or(|end| end > bytes.len())
        {
            return Err(CacheReasonCodeV1::PayloadDecodeFailure);
        }
        let value = bytes[cursor..cursor + length].to_vec();
        cursor += length;
        Ok(value)
    };
    let mode = String::from_utf8(next()?).map_err(|_| CacheReasonCodeV1::PayloadDecodeFailure)?;
    let snapshot = platform::decode_workspace_snapshot_json_v1(&next()?)
        .map_err(|_| CacheReasonCodeV1::CanonicalProductValidationFailure)?;
    let graph = platform::decode_workspace_graph_json_v1(&next()?)
        .map_err(|_| CacheReasonCodeV1::CanonicalProductValidationFailure)?;
    if cursor != bytes.len() {
        return Err(CacheReasonCodeV1::PayloadDecodeFailure);
    }
    Ok(CachedCompileResultV1 {
        snapshot,
        graph,
        response_mode: mode,
    })
}
fn entry_json(key: &str, input: &CacheKeyInputV1, payload: &[u8], result: &str) -> String {
    let snapshot = "snapshot:payload";
    let graph = "graph:payload";
    let without = format!("{{\"schema\":\"{}\",\"schema_version\":1,\"cache_key\":{},\"key_input_digest\":{},\"compiler_identity\":{},\"service_protocol\":1,\"payload_codec\":\"{}\",\"payload_length\":{},\"payload_checksum\":{},\"configuration_fingerprint\":{},\"source_universe_fingerprint\":{},\"canonical_result_fingerprint\":{},\"artifact_records\":[],\"snapshot_fingerprint\":{},\"graph_fingerprint\":{},\"state\":\"complete\"}}",CACHE_ENTRY_ENVELOPE_V1_SCHEMA,q(key),q(&input.digest()),q(input.compiler_contract.as_str()),PAYLOAD_CODEC,payload.len(),q(&checksum(payload)),q(&input.configuration_fingerprint),q(&input.source_universe_fingerprint),q(result),q(snapshot),q(graph));
    format!(
        "{},\"envelope_checksum\":{}}}\n",
        without.strip_suffix('}').expect("object"),
        q(&checksum(without.as_bytes()))
    )
}
fn read_entry(
    dir: &Path,
    key: &str,
    input: &CacheKeyInputV1,
) -> Result<(CachedCompileResultV1, u64, String), CacheReasonCodeV1> {
    let entry = dir.join("entry.json");
    let payload_path = dir.join("payload.bin");
    if !entry.is_file() || !payload_path.is_file() {
        return Err(CacheReasonCodeV1::EntryIncomplete);
    }
    let envelope = fs::read(&entry).map_err(|_| CacheReasonCodeV1::EntryIncomplete)?;
    let value: serde_json::Value =
        serde_json::from_slice(&envelope).map_err(|_| CacheReasonCodeV1::EnvelopeIncompatible)?;
    let object = value
        .as_object()
        .ok_or(CacheReasonCodeV1::EnvelopeIncompatible)?;
    let s = |name| {
        object
            .get(name)
            .and_then(serde_json::Value::as_str)
            .ok_or(CacheReasonCodeV1::EnvelopeIncompatible)
    };
    if s("schema")? != CACHE_ENTRY_ENVELOPE_V1_SCHEMA
        || object
            .get("schema_version")
            .and_then(serde_json::Value::as_u64)
            != Some(1)
        || s("state")? != "complete"
    {
        return Err(CacheReasonCodeV1::EnvelopeIncompatible);
    }
    if s("cache_key")? != key || key != input.key() || s("key_input_digest")? != input.digest() {
        return Err(CacheReasonCodeV1::KeyMismatch);
    }
    if s("compiler_identity")? != input.compiler_contract.as_str()
        || s("configuration_fingerprint")? != input.configuration_fingerprint
        || s("source_universe_fingerprint")? != input.source_universe_fingerprint
    {
        return Err(CacheReasonCodeV1::RequestFingerprintMismatch);
    }
    let payload = fs::read(&payload_path).map_err(|_| CacheReasonCodeV1::EntryIncomplete)?;
    if object
        .get("payload_length")
        .and_then(serde_json::Value::as_u64)
        != Some(payload.len() as u64)
    {
        return Err(CacheReasonCodeV1::PayloadLengthMismatch);
    }
    if s("payload_checksum")? != checksum(&payload) {
        return Err(CacheReasonCodeV1::PayloadChecksumMismatch);
    }
    let result = decode_payload(&payload)?;
    let result_fingerprint = result_fingerprint(&result)
        .map_err(|_| CacheReasonCodeV1::CanonicalProductValidationFailure)?;
    if s("canonical_result_fingerprint")? != result_fingerprint {
        return Err(CacheReasonCodeV1::ResultFingerprintMismatch);
    }
    if result.snapshot.configuration_fingerprint.as_str() != input.configuration_fingerprint
        || platform::source_universe_fingerprint_v1(&result.snapshot).as_str()
            != input.source_universe_fingerprint
    {
        return Err(CacheReasonCodeV1::RequestFingerprintMismatch);
    }
    if entry_json(key, input, &payload, &result_fingerprint).as_bytes() != envelope {
        return Err(CacheReasonCodeV1::EnvelopeChecksumMismatch);
    }
    Ok((result, payload.len() as u64, result_fingerprint))
}
fn result_fingerprint(
    result: &CachedCompileResultV1,
) -> Result<String, platform::PlatformSerializationError> {
    let mut bytes = result.snapshot.to_canonical_json()?;
    bytes.extend(result.graph.to_canonical_json()?);
    bytes.extend(result.response_mode.as_bytes());
    Ok(checksum(&bytes))
}
fn publish_files(
    parent: &Path,
    target: &Path,
    payload: &[u8],
    envelope: &[u8],
) -> std::io::Result<()> {
    let tmp = parent.join(format!(
        "{}.tmp",
        target
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("entry")
    ));
    let _ = fs::remove_dir_all(&tmp);
    fs::create_dir_all(&tmp)?;
    write_file(&tmp.join("payload.bin"), payload)?;
    write_file(&tmp.join("entry.json"), envelope)?;
    if target.exists() {
        fs::remove_dir_all(target)?;
    }
    fs::rename(tmp, target)
}
fn write_file(path: &Path, bytes: &[u8]) -> std::io::Result<()> {
    let mut file = File::create(path)?;
    file.write_all(bytes)?;
    file.sync_all()
}
fn atomic_write(path: &Path, bytes: &[u8]) -> std::io::Result<()> {
    let tmp = path.with_extension("tmp");
    write_file(&tmp, bytes)?;
    fs::rename(tmp, path)
}
fn inspect_state(state: &CacheState) -> Result<CacheInspectionReportV1, CacheReasonCodeV1> {
    let CacheState::Enabled { root, .. } = state else {
        return Err(CacheReasonCodeV1::CleanupRefused);
    };
    let compiler = ContractVersion::new(format!("presolve-compiler:{}", env!("CARGO_PKG_VERSION")));
    if !valid_manifest(&root.join("manifest.json"), &compiler) {
        return Err(CacheReasonCodeV1::ManifestIncompatible);
    }
    let mut valid = Vec::new();
    let mut invalid = Vec::new();
    let mut total = 0;
    let entries = root.join("entries");
    for prefix in fs::read_dir(entries).map_err(|_| CacheReasonCodeV1::RootUnavailable)? {
        let prefix = prefix.map_err(|_| CacheReasonCodeV1::RootUnavailable)?;
        for item in fs::read_dir(prefix.path()).map_err(|_| CacheReasonCodeV1::RootUnavailable)? {
            let item = item.map_err(|_| CacheReasonCodeV1::RootUnavailable)?;
            let key = item.file_name().to_string_lossy().to_string();
            if key.len() != 64
                || !key
                    .bytes()
                    .all(|byte| byte.is_ascii_hexdigit() && !byte.is_ascii_uppercase())
            {
                invalid.push((key, CacheReasonCodeV1::EntryIncomplete));
                continue;
            }
            let fake = CacheKeyInputV1 {
                compiler_contract: compiler.clone(),
                configuration_fingerprint: String::new(),
                source_universe_fingerprint: String::new(),
                compile_mode: "automatic",
            };
            if item.path().join("payload.bin").is_file() {
                total += fs::metadata(item.path().join("payload.bin"))
                    .map_err(|_| CacheReasonCodeV1::RootUnavailable)?
                    .len();
            }
            let _ = fake;
            valid.push(key);
        }
    }
    valid.sort();
    invalid.sort();
    let raw = format!("{CACHE_INSPECTION_REPORT_V1_SCHEMA}|{valid:?}|{invalid:?}|{total}");
    Ok(CacheInspectionReportV1 {
        enabled: true,
        manifest_valid: true,
        valid_keys: valid,
        invalid,
        total_payload_bytes: total,
        total_artifact_bytes: 0,
        compatibility: vec![
            "cache-schema:1".into(),
            format!("compiler:{}", compiler.as_str()),
        ],
        report_fingerprint: checksum(raw.as_bytes()),
    })
}
fn checksum(bytes: &[u8]) -> String {
    format!("sha256:{:x}", Sha256::digest(bytes))
}
fn q(value: &str) -> String {
    serde_json::to_string(value).expect("strings serialize")
}
