#![forbid(unsafe_code)]

//! Redis Streams implementation of `presolve:queue/work-queue@0.1.0`.
//!
//! Redis connection, stream, consumer-group, and lease configuration remain
//! node-side realization concerns. Provider-native stream IDs and consumer
//! ownership never enter the Application Contract, DeploymentPlan, or release
//! identity.

use std::{
    io::{BufRead, BufReader, Read, Write},
    net::{TcpStream, ToSocketAddrs},
    process,
    sync::{
        Arc,
        atomic::{AtomicU64, Ordering},
    },
    time::Duration,
};

use presolve_provider_sdk::{
    WorkQueueDelivery, WorkQueueError, WorkQueueProvider, WorkQueueReceiveOptions,
    WorkQueueRetryOptions, WorkQueueSendOptions,
};
use thiserror::Error;

/// Node-side adapter identity used for Redis Streams work queues.
pub const REDIS_QUEUE_ADAPTER_ID: &str = "redis:queue";
/// Redis `host:port` realization setting.
pub const REDIS_QUEUE_ADDRESS_SETTING: &str = "address";
/// Redis Stream key realization setting.
pub const REDIS_QUEUE_STREAM_SETTING: &str = "stream";
/// Redis consumer-group realization setting.
pub const REDIS_QUEUE_GROUP_SETTING: &str = "group";
/// Provider-defined baseline redelivery lease, in milliseconds.
pub const REDIS_QUEUE_LEASE_MS_SETTING: &str = "lease_ms";

const REQUEST_TIMEOUT: Duration = Duration::from_secs(5);
const DEFAULT_LEASE_MS: u64 = 30_000;
const BODY_FIELD: &[u8] = b"body";
static NEXT_CONSUMER_ID: AtomicU64 = AtomicU64::new(1);

/// Redis queue configuration failure.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum RedisQueueConfigError {
    /// A required string setting is empty.
    #[error("Redis queue {field} must not be empty")]
    EmptySetting { field: &'static str },
    /// Lease setting cannot be represented by Redis milliseconds.
    #[error("Redis queue lease must be greater than zero")]
    InvalidLease,
}

/// Redis Streams implementation of Presolve's baseline leased work queue.
#[derive(Debug, Clone)]
pub struct RedisWorkQueueProvider {
    inner: Arc<RedisQueueConfig>,
}

#[derive(Debug)]
struct RedisQueueConfig {
    address: String,
    stream: String,
    group: String,
    consumer: String,
    lease_ms: u64,
}

impl RedisWorkQueueProvider {
    /// Creates a provider using a process-unique Redis consumer name.
    ///
    /// # Errors
    ///
    /// Returns [`RedisQueueConfigError`] for empty settings or a zero lease.
    pub fn new(
        address: impl Into<String>,
        stream: impl Into<String>,
        group: impl Into<String>,
        lease_ms: u64,
    ) -> Result<Self, RedisQueueConfigError> {
        let address = require_setting(address.into(), "address")?;
        let stream = require_setting(stream.into(), "stream")?;
        let group = require_setting(group.into(), "group")?;
        if lease_ms == 0 {
            return Err(RedisQueueConfigError::InvalidLease);
        }
        let sequence = NEXT_CONSUMER_ID.fetch_add(1, Ordering::Relaxed);
        let consumer = format!("presolve-{}-{sequence}", process::id());

        Ok(Self {
            inner: Arc::new(RedisQueueConfig {
                address,
                stream,
                group,
                consumer,
                lease_ms,
            }),
        })
    }

    /// Creates a provider using the default 30-second baseline lease.
    ///
    /// # Errors
    ///
    /// Returns [`RedisQueueConfigError`] for empty settings.
    pub fn with_default_lease(
        address: impl Into<String>,
        stream: impl Into<String>,
        group: impl Into<String>,
    ) -> Result<Self, RedisQueueConfigError> {
        Self::new(address, stream, group, DEFAULT_LEASE_MS)
    }

    fn execute(&self, command: &[&[u8]]) -> Result<RedisResponse, WorkQueueError> {
        let mut stream = self.connect()?;
        write_command(&mut stream, command)?;
        stream
            .flush()
            .map_err(|error| transport_error("flush", &error))?;
        read_response(&mut BufReader::new(stream))
    }

    fn connect(&self) -> Result<TcpStream, WorkQueueError> {
        let addresses = self.inner.address.to_socket_addrs().map_err(|error| {
            WorkQueueError::unavailable(format!(
                "Redis address `{}` could not be resolved: {error}",
                self.inner.address
            ))
        })?;
        let mut last_error = None;

        for address in addresses {
            match TcpStream::connect_timeout(&address, REQUEST_TIMEOUT) {
                Ok(stream) => {
                    stream
                        .set_read_timeout(Some(REQUEST_TIMEOUT))
                        .map_err(|error| transport_error("configure read timeout", &error))?;
                    stream
                        .set_write_timeout(Some(REQUEST_TIMEOUT))
                        .map_err(|error| transport_error("configure write timeout", &error))?;
                    return Ok(stream);
                }
                Err(error) => last_error = Some(error),
            }
        }

        match last_error {
            Some(error) => Err(transport_error("connect", &error)),
            None => Err(WorkQueueError::unavailable(format!(
                "Redis address `{}` resolved to no socket addresses",
                self.inner.address
            ))),
        }
    }

    fn ensure_group(&self) -> Result<(), WorkQueueError> {
        let response = self.execute(&[
            b"XGROUP",
            b"CREATE",
            self.inner.stream.as_bytes(),
            self.inner.group.as_bytes(),
            b"0",
            b"MKSTREAM",
        ])?;
        match response {
            RedisResponse::Simple(message) if message == b"OK" => Ok(()),
            RedisResponse::Error(message) if message.starts_with(b"BUSYGROUP") => Ok(()),
            RedisResponse::Error(message) => Err(redis_server_error("XGROUP CREATE", &message)),
            response => Err(unexpected_response("XGROUP CREATE", &response)),
        }
    }

    fn recover_stale(&self) -> Result<Option<RedisStreamEntry>, WorkQueueError> {
        let lease = self.inner.lease_ms.to_string();
        let response = self.execute(&[
            b"XAUTOCLAIM",
            self.inner.stream.as_bytes(),
            self.inner.group.as_bytes(),
            self.inner.consumer.as_bytes(),
            lease.as_bytes(),
            b"0-0",
            b"COUNT",
            b"1",
        ])?;
        parse_xautoclaim(response)
    }

    fn receive_new(&self) -> Result<Option<RedisStreamEntry>, WorkQueueError> {
        let response = self.execute(&[
            b"XREADGROUP",
            b"GROUP",
            self.inner.group.as_bytes(),
            self.inner.consumer.as_bytes(),
            b"COUNT",
            b"1",
            b"STREAMS",
            self.inner.stream.as_bytes(),
            b">",
        ])?;
        parse_xreadgroup(response)
    }

    fn acknowledge(&self, id: &[u8]) -> Result<(), WorkQueueError> {
        match self.execute(&[
            b"XACK",
            self.inner.stream.as_bytes(),
            self.inner.group.as_bytes(),
            id,
        ])? {
            RedisResponse::Integer(1) => Ok(()),
            RedisResponse::Integer(0) => Err(WorkQueueError::invalid_delivery(
                "Redis delivery is no longer pending in the selected consumer group",
            )),
            RedisResponse::Error(message) => Err(redis_server_error("XACK", &message)),
            response => Err(unexpected_response("XACK", &response)),
        }
    }

    fn retry_now(&self, id: &[u8]) -> Result<(), WorkQueueError> {
        let idle_ms = self.inner.lease_ms.checked_add(1).ok_or_else(|| {
            WorkQueueError::internal("Redis queue lease cannot be advanced for retry")
        })?;
        let idle = idle_ms.to_string();
        let response = self.execute(&[
            b"XCLAIM",
            self.inner.stream.as_bytes(),
            self.inner.group.as_bytes(),
            self.inner.consumer.as_bytes(),
            b"0",
            id,
            b"IDLE",
            idle.as_bytes(),
            b"JUSTID",
        ])?;
        match response {
            RedisResponse::Array(Some(values)) if response_contains_id(&values, id) => Ok(()),
            RedisResponse::Array(Some(values)) if values.is_empty() => Err(
                WorkQueueError::invalid_delivery("Redis delivery is no longer pending"),
            ),
            RedisResponse::Error(message) => Err(redis_server_error("XCLAIM", &message)),
            response => Err(unexpected_response("XCLAIM", &response)),
        }
    }
}

impl WorkQueueProvider for RedisWorkQueueProvider {
    fn send(&self, body: &[u8], options: WorkQueueSendOptions) -> Result<(), WorkQueueError> {
        if options.delay().is_some() {
            return Err(WorkQueueError::unsupported(
                "Redis baseline queue realization does not negotiate delayed-send",
            ));
        }
        self.ensure_group()?;
        match self.execute(&[
            b"XADD",
            self.inner.stream.as_bytes(),
            b"*",
            BODY_FIELD,
            body,
        ])? {
            RedisResponse::Bulk(Some(_)) => Ok(()),
            RedisResponse::Error(message) => Err(redis_server_error("XADD", &message)),
            response => Err(unexpected_response("XADD", &response)),
        }
    }

    fn receive(
        &self,
        options: WorkQueueReceiveOptions,
    ) -> Result<Option<Box<dyn WorkQueueDelivery>>, WorkQueueError> {
        if options.lease_duration().is_some() {
            return Err(WorkQueueError::unsupported(
                "Redis baseline queue realization does not negotiate configurable-lease-duration",
            ));
        }
        self.ensure_group()?;
        let entry = match self.recover_stale()? {
            Some(entry) => Some(entry),
            None => self.receive_new()?,
        };

        Ok(entry.map(|entry| {
            Box::new(RedisDelivery {
                provider: self.clone(),
                id: entry.id,
                body: entry.body,
            }) as Box<dyn WorkQueueDelivery>
        }))
    }
}

#[derive(Debug)]
struct RedisDelivery {
    provider: RedisWorkQueueProvider,
    id: Vec<u8>,
    body: Vec<u8>,
}

impl WorkQueueDelivery for RedisDelivery {
    fn body(&self) -> &[u8] {
        &self.body
    }

    fn ack(self: Box<Self>) -> Result<(), WorkQueueError> {
        self.provider.acknowledge(&self.id)
    }

    fn retry(self: Box<Self>, options: WorkQueueRetryOptions) -> Result<(), WorkQueueError> {
        if options.delay().is_some() {
            return Err(WorkQueueError::unsupported(
                "Redis baseline queue realization does not negotiate delayed-retry",
            ));
        }
        self.provider.retry_now(&self.id)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct RedisStreamEntry {
    id: Vec<u8>,
    body: Vec<u8>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum RedisResponse {
    Simple(Vec<u8>),
    Error(Vec<u8>),
    Integer(i64),
    Bulk(Option<Vec<u8>>),
    Array(Option<Vec<Self>>),
}

fn require_setting(
    value: String,
    field: &'static str,
) -> Result<String, RedisQueueConfigError> {
    if value.trim().is_empty() {
        return Err(RedisQueueConfigError::EmptySetting { field });
    }
    Ok(value)
}

fn write_command(stream: &mut TcpStream, command: &[&[u8]]) -> Result<(), WorkQueueError> {
    write!(stream, "*{}\r\n", command.len())
        .map_err(|error| transport_error("write command header", &error))?;
    for argument in command {
        write!(stream, "${}\r\n", argument.len())
            .map_err(|error| transport_error("write argument header", &error))?;
        stream
            .write_all(argument)
            .map_err(|error| transport_error("write argument", &error))?;
        stream
            .write_all(b"\r\n")
            .map_err(|error| transport_error("write argument terminator", &error))?;
    }
    Ok(())
}

fn read_response(reader: &mut BufReader<TcpStream>) -> Result<RedisResponse, WorkQueueError> {
    let mut prefix = [0_u8; 1];
    reader
        .read_exact(&mut prefix)
        .map_err(|error| transport_error("read response prefix", &error))?;
    match prefix[0] {
        b'+' => read_line(reader).map(RedisResponse::Simple),
        b'-' => read_line(reader).map(RedisResponse::Error),
        b':' => {
            let value = read_line(reader)?;
            let value = std::str::from_utf8(&value)
                .map_err(|_| WorkQueueError::internal("Redis integer is not UTF-8"))?;
            let integer = value.parse::<i64>().map_err(|error| {
                WorkQueueError::internal(format!("Redis integer response is malformed: {error}"))
            })?;
            Ok(RedisResponse::Integer(integer))
        }
        b'$' => read_bulk(reader),
        b'*' => read_array(reader),
        marker => Err(WorkQueueError::internal(format!(
            "Redis returned unsupported RESP marker 0x{marker:02x}"
        ))),
    }
}

fn read_line(reader: &mut BufReader<TcpStream>) -> Result<Vec<u8>, WorkQueueError> {
    let mut bytes = Vec::new();
    reader
        .read_until(b'\n', &mut bytes)
        .map_err(|error| transport_error("read response line", &error))?;
    if !bytes.ends_with(b"\r\n") {
        return Err(WorkQueueError::internal(
            "Redis response line is missing CRLF terminator",
        ));
    }
    bytes.truncate(bytes.len() - 2);
    Ok(bytes)
}

fn read_bulk(reader: &mut BufReader<TcpStream>) -> Result<RedisResponse, WorkQueueError> {
    let length = parse_length(&read_line(reader)?, "bulk")?;
    if length == -1 {
        return Ok(RedisResponse::Bulk(None));
    }
    let length = usize::try_from(length)
        .map_err(|_| WorkQueueError::internal("Redis bulk length is invalid"))?;
    let mut bytes = vec![0_u8; length];
    reader
        .read_exact(&mut bytes)
        .map_err(|error| transport_error("read bulk response", &error))?;
    read_crlf(reader, "bulk")?;
    Ok(RedisResponse::Bulk(Some(bytes)))
}

fn read_array(reader: &mut BufReader<TcpStream>) -> Result<RedisResponse, WorkQueueError> {
    let length = parse_length(&read_line(reader)?, "array")?;
    if length == -1 {
        return Ok(RedisResponse::Array(None));
    }
    let length = usize::try_from(length)
        .map_err(|_| WorkQueueError::internal("Redis array length is invalid"))?;
    let mut values = Vec::with_capacity(length);
    for _ in 0..length {
        values.push(read_response(reader)?);
    }
    Ok(RedisResponse::Array(Some(values)))
}

fn parse_length(bytes: &[u8], kind: &str) -> Result<i64, WorkQueueError> {
    let text = std::str::from_utf8(bytes)
        .map_err(|_| WorkQueueError::internal(format!("Redis {kind} length is not UTF-8")))?;
    text.parse::<i64>().map_err(|error| {
        WorkQueueError::internal(format!("Redis {kind} length is malformed: {error}"))
    })
}

fn read_crlf(reader: &mut BufReader<TcpStream>, kind: &str) -> Result<(), WorkQueueError> {
    let mut terminator = [0_u8; 2];
    reader
        .read_exact(&mut terminator)
        .map_err(|error| transport_error("read response terminator", &error))?;
    if terminator != *b"\r\n" {
        return Err(WorkQueueError::internal(format!(
            "Redis {kind} response is missing CRLF terminator"
        )));
    }
    Ok(())
}

fn parse_xautoclaim(response: RedisResponse) -> Result<Option<RedisStreamEntry>, WorkQueueError> {
    let RedisResponse::Array(Some(parts)) = response else {
        if matches!(response, RedisResponse::Error(_)) {
            return response_error("XAUTOCLAIM", response);
        }
        return Err(unexpected_response("XAUTOCLAIM", &response));
    };
    let entries = parts
        .get(1)
        .ok_or_else(|| WorkQueueError::internal("Redis XAUTOCLAIM omitted entries"))?;
    first_stream_entry(entries)
}

fn parse_xreadgroup(response: RedisResponse) -> Result<Option<RedisStreamEntry>, WorkQueueError> {
    match response {
        RedisResponse::Array(None) => Ok(None),
        RedisResponse::Error(message) => Err(redis_server_error("XREADGROUP", &message)),
        RedisResponse::Array(Some(streams)) => {
            let Some(RedisResponse::Array(Some(stream))) = streams.first() else {
                return Ok(None);
            };
            let entries = stream
                .get(1)
                .ok_or_else(|| WorkQueueError::internal("Redis XREADGROUP omitted entries"))?;
            first_stream_entry(entries)
        }
        response => Err(unexpected_response("XREADGROUP", &response)),
    }
}

fn first_stream_entry(response: &RedisResponse) -> Result<Option<RedisStreamEntry>, WorkQueueError> {
    let RedisResponse::Array(Some(entries)) = response else {
        return Err(unexpected_response("stream entries", response));
    };
    let Some(RedisResponse::Array(Some(entry))) = entries.first() else {
        return Ok(None);
    };
    let id = bulk_bytes(
        entry
            .first()
            .ok_or_else(|| WorkQueueError::internal("Redis stream entry omitted ID"))?,
        "stream entry ID",
    )?;
    let fields = entry
        .get(1)
        .ok_or_else(|| WorkQueueError::internal("Redis stream entry omitted fields"))?;
    let body = body_from_fields(fields)?;
    Ok(Some(RedisStreamEntry { id, body }))
}

fn body_from_fields(response: &RedisResponse) -> Result<Vec<u8>, WorkQueueError> {
    let RedisResponse::Array(Some(fields)) = response else {
        return Err(unexpected_response("stream fields", response));
    };
    for pair in fields.chunks_exact(2) {
        if bulk_bytes(&pair[0], "stream field name")? == BODY_FIELD {
            return bulk_bytes(&pair[1], "stream body");
        }
    }
    Err(WorkQueueError::internal(
        "Redis stream entry does not contain Presolve `body` field",
    ))
}

fn bulk_bytes(response: &RedisResponse, context: &str) -> Result<Vec<u8>, WorkQueueError> {
    match response {
        RedisResponse::Bulk(Some(bytes)) => Ok(bytes.clone()),
        _ => Err(WorkQueueError::internal(format!(
            "Redis {context} is not a bulk string"
        ))),
    }
}

fn response_contains_id(values: &[RedisResponse], id: &[u8]) -> bool {
    values
        .iter()
        .any(|value| matches!(value, RedisResponse::Bulk(Some(bytes)) if bytes == id))
}

fn response_error<T>(operation: &str, response: RedisResponse) -> Result<T, WorkQueueError> {
    match response {
        RedisResponse::Error(message) => Err(redis_server_error(operation, &message)),
        response => Err(unexpected_response(operation, &response)),
    }
}

fn transport_error(operation: &str, error: &std::io::Error) -> WorkQueueError {
    WorkQueueError::unavailable(format!("Redis {operation} failed: {error}"))
}

fn redis_server_error(operation: &str, message: &[u8]) -> WorkQueueError {
    WorkQueueError::unavailable(format!(
        "Redis {operation} returned an error: {}",
        String::from_utf8_lossy(message)
    ))
}

fn unexpected_response(operation: &str, response: &RedisResponse) -> WorkQueueError {
    WorkQueueError::internal(format!(
        "Redis {operation} returned unexpected response {response:?}"
    ))
}

/// Provider-realization adapter for Redis Streams.
#[derive(Debug, Default, Clone, Copy)]
pub struct RedisWorkQueueAdapter;

impl presolve_provider_realization::ProviderAdapter for RedisWorkQueueAdapter {
    fn realize(
        &self,
        profile: &presolve_provider_realization::ProviderRealizationProfile,
        requested: &[presolve_provider_realization::ProviderCapabilityRequest],
        _secrets: &dyn presolve_provider_realization::SecretResolver,
    ) -> Result<
        presolve_provider_realization::RealizedProvider,
        presolve_provider_realization::ProviderAdapterError,
    > {
        use presolve_provider_realization::ProviderAdapterError;
        use presolve_provider_sdk::WORK_QUEUE_INTERFACE;
        use semver::Version;

        let supported_version = Version::new(0, 1, 0);
        for capability in requested {
            if capability.interface() != WORK_QUEUE_INTERFACE {
                return Err(ProviderAdapterError::UnsupportedCapability {
                    interface: capability.interface().to_owned(),
                });
            }
            if capability.version() != &supported_version {
                return Err(ProviderAdapterError::UnsupportedVersion {
                    interface: WORK_QUEUE_INTERFACE.to_owned(),
                    selected_version: capability.version().clone(),
                    supported_version: supported_version.clone(),
                });
            }
            if !capability.negotiated_features().is_empty() {
                return Err(ProviderAdapterError::UnsupportedFeatures {
                    interface: WORK_QUEUE_INTERFACE.to_owned(),
                    features: capability.negotiated_features().join(", "),
                });
            }
        }

        let address = required_profile_setting(profile, REDIS_QUEUE_ADDRESS_SETTING)?;
        let stream = required_profile_setting(profile, REDIS_QUEUE_STREAM_SETTING)?;
        let group = required_profile_setting(profile, REDIS_QUEUE_GROUP_SETTING)?;
        let lease_ms = match profile.setting(REDIS_QUEUE_LEASE_MS_SETTING) {
            Some(value) => value.parse::<u64>().map_err(|error| {
                ProviderAdapterError::Configuration {
                    message: format!("Redis queue lease_ms is invalid: {error}"),
                }
            })?,
            None => DEFAULT_LEASE_MS,
        };
        let provider = RedisWorkQueueProvider::new(address, stream, group, lease_ms).map_err(
            |error| ProviderAdapterError::Configuration {
                message: error.to_string(),
            },
        )?;

        Ok(
            presolve_provider_realization::RealizedProvider::new(*profile.provider_id())
                .with_work_queue(Arc::new(provider)),
        )
    }
}

fn required_profile_setting<'a>(
    profile: &'a presolve_provider_realization::ProviderRealizationProfile,
    key: &str,
) -> Result<&'a str, presolve_provider_realization::ProviderAdapterError> {
    profile.setting(key).ok_or_else(|| {
        presolve_provider_realization::ProviderAdapterError::MissingSetting {
            key: key.to_owned(),
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_empty_configuration() {
        assert!(matches!(
            RedisWorkQueueProvider::new("", "stream", "group", 100),
            Err(RedisQueueConfigError::EmptySetting { field: "address" })
        ));
        assert!(matches!(
            RedisWorkQueueProvider::new("127.0.0.1:6379", "stream", "group", 0),
            Err(RedisQueueConfigError::InvalidLease)
        ));
    }

    #[test]
    fn nested_stream_entry_parser_preserves_binary_body() {
        let entries = RedisResponse::Array(Some(vec![RedisResponse::Array(Some(vec![
            RedisResponse::Bulk(Some(b"1-0".to_vec())),
            RedisResponse::Array(Some(vec![
                RedisResponse::Bulk(Some(BODY_FIELD.to_vec())),
                RedisResponse::Bulk(Some(vec![0, 1, 255])),
            ])),
        ]))]));

        assert_eq!(
            first_stream_entry(&entries).expect("entry should parse"),
            Some(RedisStreamEntry {
                id: b"1-0".to_vec(),
                body: vec![0, 1, 255],
            })
        );
    }
}
