#![forbid(unsafe_code)]

//! Cloudflare Workers KV implementation of Presolve's baseline key/value
//! provider interface.
//!
//! This adapter talks to the Cloudflare REST API. Provider credentials and
//! namespace/account identity are node-side realization concerns and never
//! enter the Application Contract, Environment Specification, or `.presolved`
//! release.

use std::{fmt, time::Duration};

use percent_encoding::{NON_ALPHANUMERIC, utf8_percent_encode};
use presolve_provider_realization::SecretValue;
use presolve_provider_sdk::{KeyValueError, KeyValueProvider};
use reqwest::{
    StatusCode,
    blocking::{Client, Response},
};
use thiserror::Error;

/// Node-side adapter identity used for Cloudflare Workers KV.
pub const CLOUDFLARE_KV_ADAPTER_ID: &str = "cloudflare:kv";

/// Provider-realization setting containing the Cloudflare account ID.
pub const CLOUDFLARE_ACCOUNT_ID_SETTING: &str = "account_id";

/// Provider-realization setting containing the Workers KV namespace ID.
pub const CLOUDFLARE_NAMESPACE_ID_SETTING: &str = "namespace_id";

/// Provider-realization secret binding containing an API token.
pub const CLOUDFLARE_API_TOKEN_SECRET: &str = "api_token";

const CLOUDFLARE_API_BASE: &str = "https://api.cloudflare.com/client/v4";
const MAX_KEY_BYTES: usize = 512;
const REQUEST_TIMEOUT: Duration = Duration::from_secs(30);

/// Cloudflare Workers KV provider configuration failure.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum CloudflareKvConfigError {
    /// Account ID is not a canonical Cloudflare account identifier.
    #[error("Cloudflare account ID must contain exactly 32 ASCII hexadecimal characters")]
    InvalidAccountId,

    /// KV namespace ID is not a canonical Cloudflare namespace identifier.
    #[error("Cloudflare KV namespace ID must contain exactly 32 ASCII hexadecimal characters")]
    InvalidNamespaceId,

    /// API token is empty.
    #[error("Cloudflare API token must not be empty")]
    EmptyApiToken,

    /// HTTP client construction failed.
    #[error("could not construct Cloudflare HTTP client: {message}")]
    HttpClient {
        /// Human-readable client construction failure.
        message: String,
    },
}

/// Host-side Cloudflare Workers KV implementation.
pub struct CloudflareKvProvider {
    client: Client,
    account_id: String,
    namespace_id: String,
    api_token: SecretValue,
    api_base: String,
}

impl CloudflareKvProvider {
    /// Creates a provider using Cloudflare's public API endpoint.
    ///
    /// # Errors
    ///
    /// Returns [`CloudflareKvConfigError`] when account/namespace identity,
    /// secret material, or HTTP client construction is invalid.
    pub fn new(
        account_id: impl Into<String>,
        namespace_id: impl Into<String>,
        api_token: SecretValue,
    ) -> Result<Self, CloudflareKvConfigError> {
        Self::with_api_base(
            account_id.into(),
            namespace_id.into(),
            api_token,
            CLOUDFLARE_API_BASE.to_owned(),
        )
    }

    fn with_api_base(
        account_id: String,
        namespace_id: String,
        api_token: SecretValue,
        api_base: String,
    ) -> Result<Self, CloudflareKvConfigError> {
        if !valid_cloudflare_id(&account_id) {
            return Err(CloudflareKvConfigError::InvalidAccountId);
        }

        if !valid_cloudflare_id(&namespace_id) {
            return Err(CloudflareKvConfigError::InvalidNamespaceId);
        }

        if api_token.expose_secret().is_empty() {
            return Err(CloudflareKvConfigError::EmptyApiToken);
        }

        let client = Client::builder()
            .timeout(REQUEST_TIMEOUT)
            .build()
            .map_err(|error| CloudflareKvConfigError::HttpClient {
                message: error.to_string(),
            })?;

        Ok(Self {
            client,
            account_id,
            namespace_id,
            api_token,
            api_base,
        })
    }

    fn value_url(&self, key: &str) -> String {
        let encoded_key = utf8_percent_encode(key, NON_ALPHANUMERIC);

        format!(
            "{}/accounts/{}/storage/kv/namespaces/{}/values/{}",
            self.api_base, self.account_id, self.namespace_id, encoded_key
        )
    }

    fn validate_key(key: &str) -> Result<(), KeyValueError> {
        if key.is_empty() {
            return Err(KeyValueError::invalid_key(
                "Cloudflare Workers KV keys must not be empty",
            ));
        }

        if matches!(key, "." | "..") {
            return Err(KeyValueError::invalid_key(
                "Cloudflare Workers KV keys must not be `.` or `..`",
            ));
        }

        if key.len() > MAX_KEY_BYTES {
            return Err(KeyValueError::invalid_key(format!(
                "Cloudflare Workers KV keys must not exceed {MAX_KEY_BYTES} UTF-8 bytes"
            )));
        }

        Ok(())
    }

    fn authenticated(
        &self,
        builder: reqwest::blocking::RequestBuilder,
    ) -> reqwest::blocking::RequestBuilder {
        builder.bearer_auth(self.api_token.expose_secret())
    }

    fn send_get(&self, key: &str) -> Result<Response, KeyValueError> {
        self.authenticated(self.client.get(self.value_url(key)))
            .send()
            .map_err(|error| map_transport_error(&error))
    }

    fn get_existing(&self, key: &str) -> Result<Option<String>, KeyValueError> {
        let response = self.send_get(key)?;

        if response.status() == StatusCode::NOT_FOUND {
            return Ok(None);
        }

        let response = require_success(response, "read")?;
        let bytes = response
            .bytes()
            .map_err(|error| map_transport_error(&error))?;

        String::from_utf8(bytes.to_vec()).map(Some).map_err(|_| {
            KeyValueError::internal(
                "Cloudflare Workers KV value is not valid UTF-8 for Presolve string KV semantics",
            )
        })
    }
}

impl fmt::Debug for CloudflareKvProvider {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("CloudflareKvProvider")
            .field("account_id", &self.account_id)
            .field("namespace_id", &self.namespace_id)
            .field("api_token", &"[redacted]")
            .field("api_base", &self.api_base)
            .finish_non_exhaustive()
    }
}

impl KeyValueProvider for CloudflareKvProvider {
    fn get(&self, key: &str) -> Result<Option<String>, KeyValueError> {
        Self::validate_key(key)?;
        self.get_existing(key)
    }

    fn set(&self, key: &str, value: &str) -> Result<(), KeyValueError> {
        Self::validate_key(key)?;

        let response = self
            .authenticated(
                self.client
                    .put(self.value_url(key))
                    .header("content-type", "application/octet-stream")
                    .body(value.to_owned()),
            )
            .send()
            .map_err(|error| map_transport_error(&error))?;

        require_success(response, "write")?;
        Ok(())
    }

    fn delete(&self, key: &str) -> Result<bool, KeyValueError> {
        Self::validate_key(key)?;

        // Cloudflare's delete endpoint is idempotent and does not report prior
        // existence. Presolve's baseline KV contract does. Read first so the
        // sequential API preserves that semantic result.
        //
        // This is not atomic with respect to concurrent external writers. Chunk
        // 18 documents that limitation rather than claiming stronger semantics.
        if self.get_existing(key)?.is_none() {
            return Ok(false);
        }

        let response = self
            .authenticated(self.client.delete(self.value_url(key)))
            .send()
            .map_err(|error| map_transport_error(&error))?;

        require_success(response, "delete")?;
        Ok(true)
    }
}

fn valid_cloudflare_id(value: &str) -> bool {
    value.len() == 32 && value.bytes().all(|byte| byte.is_ascii_hexdigit())
}

fn map_transport_error(error: &reqwest::Error) -> KeyValueError {
    if error.is_timeout() || error.is_connect() {
        KeyValueError::unavailable(format!("Cloudflare Workers KV is unavailable: {error}"))
    } else {
        KeyValueError::internal(format!("Cloudflare Workers KV request failed: {error}"))
    }
}

fn require_success(response: Response, operation: &str) -> Result<Response, KeyValueError> {
    let status = response.status();

    if status.is_success() {
        return Ok(response);
    }

    if matches!(
        status,
        StatusCode::UNAUTHORIZED
            | StatusCode::FORBIDDEN
            | StatusCode::TOO_MANY_REQUESTS
            | StatusCode::REQUEST_TIMEOUT
    ) || status.is_server_error()
    {
        return Err(KeyValueError::unavailable(format!(
            "Cloudflare Workers KV {operation} returned HTTP {status}"
        )));
    }

    Err(KeyValueError::internal(format!(
        "Cloudflare Workers KV {operation} returned unexpected HTTP {status}"
    )))
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

    const ACCOUNT_ID: &str = "0123456789abcdef0123456789abcdef";
    const NAMESPACE_ID: &str = "abcdef0123456789abcdef0123456789";

    struct RecordedRequest {
        request_line: String,
        headers: Vec<(String, String)>,
        body: Vec<u8>,
    }

    impl RecordedRequest {
        fn header(&self, name: &str) -> Option<&str> {
            self.headers
                .iter()
                .find(|(candidate, _)| candidate.eq_ignore_ascii_case(name))
                .map(|(_, value)| value.as_str())
        }
    }

    fn provider(api_base: String) -> CloudflareKvProvider {
        CloudflareKvProvider::with_api_base(
            ACCOUNT_ID.to_owned(),
            NAMESPACE_ID.to_owned(),
            SecretValue::new("test-token"),
            api_base,
        )
        .expect("test provider should configure")
    }

    fn serve(
        responses: Vec<(&'static str, &'static [u8])>,
    ) -> (
        String,
        mpsc::Receiver<Vec<RecordedRequest>>,
        thread::JoinHandle<()>,
    ) {
        let listener = TcpListener::bind("127.0.0.1:0").expect("test listener should bind");
        let address = listener
            .local_addr()
            .expect("test listener should have an address");
        let (sender, receiver) = mpsc::channel();

        let handle = thread::spawn(move || {
            let mut requests = Vec::new();

            for (status, body) in responses {
                let (mut stream, _) = listener.accept().expect("test request should arrive");
                requests.push(read_request(&mut stream));

                let response = format!(
                    "HTTP/1.1 {status}\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
                    body.len()
                );
                stream
                    .write_all(response.as_bytes())
                    .expect("response headers should write");
                stream.write_all(body).expect("response body should write");
            }

            sender.send(requests).expect("requests should be recorded");
        });

        (format!("http://{address}"), receiver, handle)
    }

    fn read_request(stream: &mut TcpStream) -> RecordedRequest {
        let clone = stream.try_clone().expect("stream should clone");
        let mut reader = BufReader::new(clone);
        let mut request_line = String::new();

        reader
            .read_line(&mut request_line)
            .expect("request line should read");

        let mut headers = Vec::new();
        let mut content_length = 0_usize;

        loop {
            let mut line = String::new();
            reader.read_line(&mut line).expect("header should read");

            if line == "\r\n" {
                break;
            }

            let (name, value) = line
                .trim_end()
                .split_once(':')
                .expect("header should contain colon");

            if name.eq_ignore_ascii_case("content-length") {
                content_length = value
                    .trim()
                    .parse::<usize>()
                    .expect("content length should parse");
            }

            headers.push((name.to_owned(), value.trim().to_owned()));
        }

        let mut body = vec![0_u8; content_length];
        reader
            .read_exact(&mut body)
            .expect("request body should read");

        RecordedRequest {
            request_line: request_line.trim_end().to_owned(),
            headers,
            body,
        }
    }

    fn finish(
        receiver: &mpsc::Receiver<Vec<RecordedRequest>>,
        handle: thread::JoinHandle<()>,
    ) -> Vec<RecordedRequest> {
        let requests = receiver.recv().expect("requests should be returned");
        handle.join().expect("test server should finish");
        requests
    }

    #[test]
    fn rejects_invalid_cloudflare_configuration() {
        assert!(matches!(
            CloudflareKvProvider::new("short", NAMESPACE_ID, SecretValue::new("token"),),
            Err(CloudflareKvConfigError::InvalidAccountId)
        ));

        assert!(matches!(
            CloudflareKvProvider::new(ACCOUNT_ID, NAMESPACE_ID, SecretValue::new(""),),
            Err(CloudflareKvConfigError::EmptyApiToken)
        ));
    }

    #[test]
    fn debug_output_redacts_api_token() {
        let provider =
            CloudflareKvProvider::new(ACCOUNT_ID, NAMESPACE_ID, SecretValue::new("super-secret"))
                .expect("provider should configure");
        let debug = format!("{provider:?}");

        assert!(debug.contains("[redacted]"));
        assert!(!debug.contains("super-secret"));
    }

    #[test]
    fn get_percent_encodes_exact_key_and_uses_bearer_auth() {
        let (base, receiver, handle) = serve(vec![("200 OK", b"value")]);
        let provider = provider(base);

        assert_eq!(
            provider.get("folder/Case Key").expect("get should work"),
            Some("value".to_owned())
        );

        let requests = finish(&receiver, handle);
        assert_eq!(requests.len(), 1);
        assert_eq!(
            requests[0].request_line,
            format!(
                "GET /accounts/{ACCOUNT_ID}/storage/kv/namespaces/{NAMESPACE_ID}/values/folder%2FCase%20Key HTTP/1.1"
            )
        );
        assert_eq!(
            requests[0].header("authorization"),
            Some("Bearer test-token")
        );
    }

    #[test]
    fn get_maps_cloudflare_404_to_missing() {
        let (base, receiver, handle) = serve(vec![("404 Not Found", b"")]);
        let provider = provider(base);

        assert_eq!(provider.get("missing").expect("get should work"), None);

        finish(&receiver, handle);
    }

    #[test]
    fn set_sends_complete_value_as_octet_stream() {
        let (base, receiver, handle) = serve(vec![("200 OK", b"")]);
        let provider = provider(base);

        provider
            .set("example", "complete-value")
            .expect("set should work");

        let requests = finish(&receiver, handle);
        assert_eq!(requests[0].body, b"complete-value");
        assert_eq!(
            requests[0].header("content-type"),
            Some("application/octet-stream")
        );
    }

    #[test]
    fn delete_reports_existence_using_read_then_delete() {
        let (base, receiver, handle) = serve(vec![("200 OK", b"value"), ("200 OK", b"")]);
        let provider = provider(base);

        assert!(provider.delete("present").expect("delete should work"));

        let requests = finish(&receiver, handle);
        assert_eq!(requests.len(), 2);
        assert!(requests[0].request_line.starts_with("GET "));
        assert!(requests[1].request_line.starts_with("DELETE "));
    }

    #[test]
    fn delete_missing_key_returns_false_without_delete_request() {
        let (base, receiver, handle) = serve(vec![("404 Not Found", b"")]);
        let provider = provider(base);

        assert!(!provider.delete("missing").expect("delete should work"));

        let requests = finish(&receiver, handle);
        assert_eq!(requests.len(), 1);
        assert!(requests[0].request_line.starts_with("GET "));
    }

    #[test]
    fn invalid_keys_fail_before_network_access() {
        let provider =
            CloudflareKvProvider::new(ACCOUNT_ID, NAMESPACE_ID, SecretValue::new("token"))
                .expect("provider should configure");

        for key in ["", ".", ".."] {
            let error = provider.get(key).expect_err("key should be invalid");
            assert_eq!(error.code(), KeyValueErrorCode::InvalidKey);
        }

        let oversized = "x".repeat(MAX_KEY_BYTES + 1);
        let error = provider
            .set(&oversized, "value")
            .expect_err("oversized key should fail");
        assert_eq!(error.code(), KeyValueErrorCode::InvalidKey);
    }

    #[test]
    fn authorization_failure_is_unavailable() {
        let (base, receiver, handle) = serve(vec![("401 Unauthorized", b"")]);
        let provider = provider(base);

        let error = provider
            .get("example")
            .expect_err("authorization should fail");
        assert_eq!(error.code(), KeyValueErrorCode::Unavailable);

        finish(&receiver, handle);
    }
}

/// Provider-realization adapter for Cloudflare Workers KV.
#[derive(Debug, Default, Clone, Copy)]
pub struct CloudflareKvAdapter;

impl presolve_provider_realization::ProviderAdapter for CloudflareKvAdapter {
    fn realize(
        &self,
        profile: &presolve_provider_realization::ProviderRealizationProfile,
        requested: &[presolve_provider_realization::ProviderCapabilityRequest],
        secrets: &dyn presolve_provider_realization::SecretResolver,
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

        let account_id = profile
            .setting(CLOUDFLARE_ACCOUNT_ID_SETTING)
            .ok_or_else(|| ProviderAdapterError::MissingSetting {
                key: CLOUDFLARE_ACCOUNT_ID_SETTING.to_owned(),
            })?;

        let namespace_id = profile
            .setting(CLOUDFLARE_NAMESPACE_ID_SETTING)
            .ok_or_else(|| ProviderAdapterError::MissingSetting {
                key: CLOUDFLARE_NAMESPACE_ID_SETTING.to_owned(),
            })?;

        let token_reference = profile.secret(CLOUDFLARE_API_TOKEN_SECRET).ok_or_else(|| {
            ProviderAdapterError::MissingSecret {
                name: CLOUDFLARE_API_TOKEN_SECRET.to_owned(),
            }
        })?;

        let api_token = secrets.resolve(token_reference).map_err(|error| {
            ProviderAdapterError::SecretResolution {
                name: CLOUDFLARE_API_TOKEN_SECRET.to_owned(),
                message: error.to_string(),
            }
        })?;

        let provider =
            CloudflareKvProvider::new(account_id, namespace_id, api_token).map_err(|error| {
                ProviderAdapterError::Configuration {
                    message: error.to_string(),
                }
            })?;

        Ok(
            presolve_provider_realization::RealizedProvider::new(*profile.provider_id())
                .with_key_value(Arc::new(provider)),
        )
    }
}

#[cfg(test)]
mod realization_tests {
    use presolve_core::ProviderId;
    use presolve_provider_realization::{
        ProviderAdapter, ProviderAdapterError, ProviderAdapterId, ProviderCapabilityRequest,
        ProviderRealizationProfile, SecretReference, SecretResolutionError, SecretResolver,
        SecretValue,
    };
    use presolve_provider_sdk::KEY_VALUE_INTERFACE;
    use semver::Version;

    use super::{
        CLOUDFLARE_ACCOUNT_ID_SETTING, CLOUDFLARE_API_TOKEN_SECRET, CLOUDFLARE_KV_ADAPTER_ID,
        CLOUDFLARE_NAMESPACE_ID_SETTING, CloudflareKvAdapter,
    };

    struct StaticSecrets;

    impl SecretResolver for StaticSecrets {
        fn resolve(
            &self,
            _reference: &SecretReference,
        ) -> Result<SecretValue, SecretResolutionError> {
            Ok(SecretValue::new("test-token"))
        }
    }

    fn profile() -> ProviderRealizationProfile {
        ProviderRealizationProfile::new(
            ProviderId::new(),
            ProviderAdapterId::new(CLOUDFLARE_KV_ADAPTER_ID).expect("adapter ID should be valid"),
        )
        .with_setting(
            CLOUDFLARE_ACCOUNT_ID_SETTING,
            "0123456789abcdef0123456789abcdef",
        )
        .expect("account setting should work")
        .with_setting(
            CLOUDFLARE_NAMESPACE_ID_SETTING,
            "abcdef0123456789abcdef0123456789",
        )
        .expect("namespace setting should work")
        .with_secret(
            CLOUDFLARE_API_TOKEN_SECRET,
            SecretReference::environment_variable("PRESOLVE_TEST_CLOUDFLARE_TOKEN")
                .expect("secret reference should be valid"),
        )
        .expect("secret binding should work")
    }

    #[test]
    fn adapter_realizes_selected_baseline_kv() {
        let profile = profile();
        let request = ProviderCapabilityRequest::new(
            KEY_VALUE_INTERFACE,
            Version::new(0, 1, 0),
            std::iter::empty::<&str>(),
        );

        let realized = CloudflareKvAdapter
            .realize(&profile, &[request], &StaticSecrets)
            .expect("Cloudflare adapter should realize");

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

        let error = CloudflareKvAdapter
            .realize(&profile, &[request], &StaticSecrets)
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
