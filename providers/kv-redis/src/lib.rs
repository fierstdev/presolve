#![forbid(unsafe_code)]

//! Redis implementation of Presolve's baseline key/value provider interface.
//!
//! Redis connection identity is a node-side realization concern. The address
//! is intentionally absent from the Application Contract, Environment
//! Specification, deployment plan, and `.presolved` release.

use std::{
    io::{BufRead, BufReader, Read, Write},
    net::{TcpStream, ToSocketAddrs},
    time::Duration,
};

use presolve_provider_sdk::{KeyValueError, KeyValueProvider};
use thiserror::Error;

/// Node-side adapter identity used for Redis KV.
pub const REDIS_KV_ADAPTER_ID: &str = "redis:kv";

/// Provider-realization setting containing the Redis `host:port` address.
pub const REDIS_ADDRESS_SETTING: &str = "address";

const REQUEST_TIMEOUT: Duration = Duration::from_secs(5);

/// Redis provider configuration failure.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum RedisKvConfigError {
    /// Redis address is empty.
    #[error("Redis address must not be empty")]
    EmptyAddress,
}

/// Host-side Redis implementation of Presolve baseline KV semantics.
#[derive(Debug, Clone)]
pub struct RedisKvProvider {
    address: String,
}

impl RedisKvProvider {
    /// Creates a Redis provider for one `host:port` address.
    ///
    /// # Errors
    ///
    /// Returns [`RedisKvConfigError`] when the configured address is empty.
    pub fn new(address: impl Into<String>) -> Result<Self, RedisKvConfigError> {
        let address = address.into();

        if address.trim().is_empty() {
            return Err(RedisKvConfigError::EmptyAddress);
        }

        Ok(Self { address })
    }

    fn execute(&self, command: &[&str]) -> Result<RedisResponse, KeyValueError> {
        let mut stream = self.connect()?;
        write_command(&mut stream, command)?;
        stream
            .flush()
            .map_err(|error| transport_error("flush", &error))?;

        let mut reader = BufReader::new(stream);
        read_response(&mut reader)
    }

    fn connect(&self) -> Result<TcpStream, KeyValueError> {
        let addresses = self.address.to_socket_addrs().map_err(|error| {
            KeyValueError::unavailable(format!(
                "Redis address `{}` could not be resolved: {error}",
                self.address
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
            None => Err(KeyValueError::unavailable(format!(
                "Redis address `{}` resolved to no socket addresses",
                self.address
            ))),
        }
    }
}

impl KeyValueProvider for RedisKvProvider {
    fn get(&self, key: &str) -> Result<Option<String>, KeyValueError> {
        validate_key(key)?;

        match self.execute(&["GET", key])? {
            RedisResponse::Bulk(None) => Ok(None),
            RedisResponse::Bulk(Some(bytes)) => String::from_utf8(bytes).map(Some).map_err(|_| {
                KeyValueError::internal(
                    "Redis value is not valid UTF-8 for Presolve string KV semantics",
                )
            }),
            RedisResponse::Error(message) => Err(redis_server_error(&message)),
            response => Err(unexpected_response("GET", &response)),
        }
    }

    fn set(&self, key: &str, value: &str) -> Result<(), KeyValueError> {
        validate_key(key)?;

        match self.execute(&["SET", key, value])? {
            RedisResponse::Simple(message) if message == "OK" => Ok(()),
            RedisResponse::Error(message) => Err(redis_server_error(&message)),
            response => Err(unexpected_response("SET", &response)),
        }
    }

    fn delete(&self, key: &str) -> Result<bool, KeyValueError> {
        validate_key(key)?;

        match self.execute(&["DEL", key])? {
            RedisResponse::Integer(0) => Ok(false),
            RedisResponse::Integer(1) => Ok(true),
            RedisResponse::Integer(count) => Err(KeyValueError::internal(format!(
                "Redis DEL for one key returned unexpected deletion count {count}"
            ))),
            RedisResponse::Error(message) => Err(redis_server_error(&message)),
            response => Err(unexpected_response("DEL", &response)),
        }
    }
}

fn validate_key(key: &str) -> Result<(), KeyValueError> {
    if key.is_empty() {
        return Err(KeyValueError::invalid_key("key must not be empty"));
    }

    Ok(())
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum RedisResponse {
    Simple(String),
    Error(String),
    Integer(i64),
    Bulk(Option<Vec<u8>>),
}

fn write_command(stream: &mut TcpStream, command: &[&str]) -> Result<(), KeyValueError> {
    write!(stream, "*{}\r\n", command.len())
        .map_err(|error| transport_error("write command header", &error))?;

    for argument in command {
        write!(stream, "${}\r\n", argument.len())
            .map_err(|error| transport_error("write argument header", &error))?;
        stream
            .write_all(argument.as_bytes())
            .map_err(|error| transport_error("write argument", &error))?;
        stream
            .write_all(b"\r\n")
            .map_err(|error| transport_error("write argument terminator", &error))?;
    }

    Ok(())
}

fn read_response(reader: &mut BufReader<TcpStream>) -> Result<RedisResponse, KeyValueError> {
    let mut prefix = [0_u8; 1];
    reader
        .read_exact(&mut prefix)
        .map_err(|error| transport_error("read response prefix", &error))?;

    match prefix[0] {
        b'+' => read_line(reader).map(RedisResponse::Simple),
        b'-' => read_line(reader).map(RedisResponse::Error),
        b':' => {
            let value = read_line(reader)?;
            let integer = value.parse::<i64>().map_err(|error| {
                KeyValueError::internal(format!("Redis integer response is malformed: {error}"))
            })?;
            Ok(RedisResponse::Integer(integer))
        }
        b'$' => read_bulk(reader),
        marker => Err(KeyValueError::internal(format!(
            "Redis returned unsupported RESP marker 0x{marker:02x}"
        ))),
    }
}

fn read_line(reader: &mut BufReader<TcpStream>) -> Result<String, KeyValueError> {
    let mut bytes = Vec::new();
    reader
        .read_until(b'\n', &mut bytes)
        .map_err(|error| transport_error("read response line", &error))?;

    if !bytes.ends_with(b"\r\n") {
        return Err(KeyValueError::internal(
            "Redis response line is missing CRLF terminator",
        ));
    }

    bytes.truncate(bytes.len() - 2);
    String::from_utf8(bytes)
        .map_err(|_| KeyValueError::internal("Redis response line is not valid UTF-8"))
}

fn read_bulk(reader: &mut BufReader<TcpStream>) -> Result<RedisResponse, KeyValueError> {
    let length_text = read_line(reader)?;
    let length = length_text.parse::<i64>().map_err(|error| {
        KeyValueError::internal(format!("Redis bulk length is malformed: {error}"))
    })?;

    if length == -1 {
        return Ok(RedisResponse::Bulk(None));
    }

    let length = usize::try_from(length).map_err(|_| {
        KeyValueError::internal(format!("Redis bulk length `{length_text}` is invalid"))
    })?;
    let mut bytes = vec![0_u8; length];
    reader
        .read_exact(&mut bytes)
        .map_err(|error| transport_error("read bulk response", &error))?;

    let mut terminator = [0_u8; 2];
    reader
        .read_exact(&mut terminator)
        .map_err(|error| transport_error("read bulk terminator", &error))?;

    if terminator != *b"\r\n" {
        return Err(KeyValueError::internal(
            "Redis bulk response is missing CRLF terminator",
        ));
    }

    Ok(RedisResponse::Bulk(Some(bytes)))
}

fn transport_error(operation: &str, error: &std::io::Error) -> KeyValueError {
    KeyValueError::unavailable(format!("Redis {operation} failed: {error}"))
}

fn redis_server_error(message: &str) -> KeyValueError {
    KeyValueError::unavailable(format!("Redis returned an error: {message}"))
}

fn unexpected_response(operation: &str, response: &RedisResponse) -> KeyValueError {
    KeyValueError::internal(format!(
        "Redis {operation} returned unexpected response {response:?}"
    ))
}

/// Provider-realization adapter for Redis KV.
#[derive(Debug, Default, Clone, Copy)]
pub struct RedisKvAdapter;

impl presolve_provider_realization::ProviderAdapter for RedisKvAdapter {
    fn realize(
        &self,
        profile: &presolve_provider_realization::ProviderRealizationProfile,
        requested: &[presolve_provider_realization::ProviderCapabilityRequest],
        _secrets: &dyn presolve_provider_realization::SecretResolver,
    ) -> Result<
        presolve_provider_realization::RealizedProvider,
        presolve_provider_realization::ProviderAdapterError,
    > {
        use std::sync::Arc;

        use presolve_provider_realization::ProviderAdapterError;
        use presolve_provider_sdk::KEY_VALUE_INTERFACE;
        use semver::Version;

        let supported_version = Version::new(0, 1, 0);

        for capability in requested {
            if capability.interface() != KEY_VALUE_INTERFACE {
                return Err(ProviderAdapterError::UnsupportedCapability {
                    interface: capability.interface().to_owned(),
                });
            }

            if capability.version() != &supported_version {
                return Err(ProviderAdapterError::UnsupportedVersion {
                    interface: KEY_VALUE_INTERFACE.to_owned(),
                    selected_version: capability.version().clone(),
                    supported_version: supported_version.clone(),
                });
            }

            if !capability.negotiated_features().is_empty() {
                return Err(ProviderAdapterError::UnsupportedFeatures {
                    interface: KEY_VALUE_INTERFACE.to_owned(),
                    features: capability.negotiated_features().join(", "),
                });
            }
        }

        let address = profile.setting(REDIS_ADDRESS_SETTING).ok_or_else(|| {
            ProviderAdapterError::MissingSetting {
                key: REDIS_ADDRESS_SETTING.to_owned(),
            }
        })?;

        let provider =
            RedisKvProvider::new(address).map_err(|error| ProviderAdapterError::Configuration {
                message: error.to_string(),
            })?;

        Ok(
            presolve_provider_realization::RealizedProvider::new(*profile.provider_id())
                .with_key_value(Arc::new(provider)),
        )
    }
}

#[cfg(test)]
mod tests {
    use std::{
        io::{BufRead, BufReader, Read, Write},
        net::{TcpListener, TcpStream},
        sync::mpsc,
        thread,
    };

    use presolve_provider_sdk::KeyValueErrorCode;

    use super::*;

    #[derive(Debug)]
    struct RecordedCommand(Vec<Vec<u8>>);

    fn serve(
        responses: Vec<&'static [u8]>,
    ) -> (
        String,
        mpsc::Receiver<Vec<RecordedCommand>>,
        thread::JoinHandle<()>,
    ) {
        let listener = TcpListener::bind("127.0.0.1:0").expect("test listener should bind");
        let address = listener
            .local_addr()
            .expect("test listener should have an address");
        let (sender, receiver) = mpsc::channel();

        let handle = thread::spawn(move || {
            let mut commands = Vec::new();

            for response in responses {
                let (mut stream, _) = listener.accept().expect("test request should arrive");
                commands.push(read_command(&mut stream));
                stream.write_all(response).expect("response should write");
            }

            sender.send(commands).expect("commands should be recorded");
        });

        (address.to_string(), receiver, handle)
    }

    fn read_command(stream: &mut TcpStream) -> RecordedCommand {
        let clone = stream.try_clone().expect("stream should clone");
        let mut reader = BufReader::new(clone);
        let mut line = String::new();
        reader
            .read_line(&mut line)
            .expect("array header should read");
        let count = line
            .trim_end()
            .strip_prefix('*')
            .expect("command should be RESP array")
            .parse::<usize>()
            .expect("array count should parse");
        let mut parts = Vec::new();

        for _ in 0..count {
            line.clear();
            reader
                .read_line(&mut line)
                .expect("bulk header should read");
            let length = line
                .trim_end()
                .strip_prefix('$')
                .expect("argument should be RESP bulk string")
                .parse::<usize>()
                .expect("bulk length should parse");
            let mut bytes = vec![0_u8; length];
            reader
                .read_exact(&mut bytes)
                .expect("bulk value should read");
            let mut terminator = [0_u8; 2];
            reader
                .read_exact(&mut terminator)
                .expect("bulk terminator should read");
            assert_eq!(terminator, *b"\r\n");
            parts.push(bytes);
        }

        RecordedCommand(parts)
    }

    fn finish(
        receiver: &mpsc::Receiver<Vec<RecordedCommand>>,
        handle: thread::JoinHandle<()>,
    ) -> Vec<RecordedCommand> {
        let commands = receiver.recv().expect("commands should be returned");
        handle.join().expect("test server should finish");
        commands
    }

    #[test]
    fn rejects_empty_address() {
        assert!(matches!(
            RedisKvProvider::new(""),
            Err(RedisKvConfigError::EmptyAddress)
        ));
    }

    #[test]
    fn get_reads_bulk_value_and_encodes_key() {
        let (address, receiver, handle) = serve(vec![b"$5\r\nvalue\r\n"]);
        let provider = RedisKvProvider::new(address).expect("provider should configure");

        assert_eq!(
            provider.get("Case Key").expect("get should work"),
            Some("value".to_owned())
        );

        let commands = finish(&receiver, handle);
        assert_eq!(commands[0].0, [b"GET".to_vec(), b"Case Key".to_vec()]);
    }

    #[test]
    fn get_maps_nil_bulk_to_missing() {
        let (address, receiver, handle) = serve(vec![b"$-1\r\n"]);
        let provider = RedisKvProvider::new(address).expect("provider should configure");

        assert_eq!(provider.get("missing").expect("get should work"), None);
        finish(&receiver, handle);
    }

    #[test]
    fn set_requires_ok_response() {
        let (address, receiver, handle) = serve(vec![b"+OK\r\n"]);
        let provider = RedisKvProvider::new(address).expect("provider should configure");

        provider
            .set("example", "complete-value")
            .expect("set should work");

        let commands = finish(&receiver, handle);
        assert_eq!(
            commands[0].0,
            [
                b"SET".to_vec(),
                b"example".to_vec(),
                b"complete-value".to_vec()
            ]
        );
    }

    #[test]
    fn delete_preserves_exact_existence_semantics() {
        let (address, receiver, handle) = serve(vec![b":1\r\n", b":0\r\n"]);
        let provider = RedisKvProvider::new(address).expect("provider should configure");

        assert!(provider.delete("present").expect("delete should work"));
        assert!(!provider.delete("missing").expect("delete should work"));

        finish(&receiver, handle);
    }

    #[test]
    fn server_errors_are_unavailable() {
        let (address, receiver, handle) = serve(vec![b"-LOADING loading dataset\r\n"]);
        let provider = RedisKvProvider::new(address).expect("provider should configure");

        let error = provider.get("example").expect_err("server should fail");
        assert_eq!(error.code(), KeyValueErrorCode::Unavailable);
        assert!(error.message().contains("LOADING"));

        finish(&receiver, handle);
    }

    #[test]
    fn empty_keys_are_rejected_before_network_access() {
        let provider = RedisKvProvider::new("127.0.0.1:1").expect("provider should configure");
        let error = provider.get("").expect_err("key should be invalid");
        assert_eq!(error.code(), KeyValueErrorCode::InvalidKey);
    }
}

#[cfg(test)]
mod realization_tests {
    use presolve_core::ProviderId;
    use presolve_provider_realization::{
        EnvironmentSecretResolver, ProviderAdapter, ProviderAdapterError, ProviderAdapterId,
        ProviderCapabilityRequest, ProviderRealizationProfile,
    };
    use presolve_provider_sdk::KEY_VALUE_INTERFACE;
    use semver::Version;

    use super::{REDIS_ADDRESS_SETTING, REDIS_KV_ADAPTER_ID, RedisKvAdapter};

    fn profile() -> ProviderRealizationProfile {
        ProviderRealizationProfile::new(
            ProviderId::new(),
            ProviderAdapterId::new(REDIS_KV_ADAPTER_ID).expect("adapter ID should be valid"),
        )
        .with_setting(REDIS_ADDRESS_SETTING, "127.0.0.1:6379")
        .expect("address setting should work")
    }

    #[test]
    fn adapter_realizes_selected_baseline_kv() {
        let profile = profile();
        let request = ProviderCapabilityRequest::new(
            KEY_VALUE_INTERFACE,
            Version::new(0, 1, 0),
            std::iter::empty::<&str>(),
        );

        let realized = RedisKvAdapter
            .realize(&profile, &[request], &EnvironmentSecretResolver)
            .expect("Redis adapter should realize");

        assert_eq!(realized.provider_id(), profile.provider_id());
        assert!(realized.key_value().is_some());
        assert!(realized.object_store().is_none());
    }

    #[test]
    fn adapter_rejects_unsupported_version() {
        let profile = profile();
        let request = ProviderCapabilityRequest::new(
            KEY_VALUE_INTERFACE,
            Version::new(9, 0, 0),
            std::iter::empty::<&str>(),
        );

        let error = RedisKvAdapter
            .realize(&profile, &[request], &EnvironmentSecretResolver)
            .expect_err("future version should fail");

        assert!(matches!(
            error,
            ProviderAdapterError::UnsupportedVersion {
                selected_version,
                ..
            } if selected_version == Version::new(9, 0, 0)
        ));
    }
}
