#![forbid(unsafe_code)]

//! S3-compatible implementation of Presolve's baseline object-storage
//! capability.
//!
//! Endpoint, bucket, region, and credentials are node-side realization
//! concerns. They never enter the Application Contract, Environment
//! Specification, or immutable `.presolved` release.
//!
//! The initial adapter uses path-style S3 requests, which makes the same
//! implementation suitable for local `MinIO` and compatible managed endpoints
//! such as Cloudflare R2. Optional object semantics remain outside the baseline
//! capability until explicitly negotiated.

mod list;
mod signing;

use std::{fmt, io::Read, sync::Arc, time::Duration};

use presolve_provider_realization::SecretValue;
use presolve_provider_sdk::{ObjectInfo, ObjectRead, ObjectStoreError, ObjectStoreProvider};
use reqwest::{
    Method, StatusCode,
    blocking::{Client, Response},
    header::{AUTHORIZATION, CONTENT_LENGTH},
};
use thiserror::Error;

use list::parse_list_page;
use signing::{SigV4Input, authorization, aws_percent_encode, canonical_query, now, sha256_hex};

/// Node-side adapter identity for S3-compatible object storage.
pub const S3_OBJECTS_ADAPTER_ID: &str = "s3:objects";

/// Provider-realization setting containing the S3-compatible endpoint.
pub const S3_ENDPOINT_SETTING: &str = "endpoint";

/// Provider-realization setting containing the bound bucket name.
pub const S3_BUCKET_SETTING: &str = "bucket";

/// Provider-realization setting containing the `SigV4` region.
pub const S3_REGION_SETTING: &str = "region";

/// Provider-realization secret binding containing the access-key identifier.
pub const S3_ACCESS_KEY_ID_SECRET: &str = "access_key_id";

/// Provider-realization secret binding containing the secret access key.
pub const S3_SECRET_ACCESS_KEY_SECRET: &str = "secret_access_key";

const S3_SERVICE: &str = "s3";
const MAX_KEY_BYTES: usize = 1_024;
const REQUEST_TIMEOUT: Duration = Duration::from_secs(30);

/// S3-compatible provider configuration failure.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum S3ObjectStoreConfigError {
    /// Endpoint is not a supported root HTTP(S) URL.
    #[error(
        "S3 endpoint must be a root http:// or https:// URL without credentials, query, or fragment"
    )]
    InvalidEndpoint,

    /// Bucket name cannot be represented by this path-style adapter.
    #[error("S3 bucket must be non-empty and must not contain `/` or ASCII control characters")]
    InvalidBucket,

    /// Region is empty or cannot safely enter a `SigV4` credential scope.
    #[error("S3 region must be non-empty lowercase ASCII letters, digits, or hyphens")]
    InvalidRegion,

    /// Access-key identifier is empty.
    #[error("S3 access-key identifier must not be empty")]
    EmptyAccessKeyId,

    /// Secret access key is empty.
    #[error("S3 secret access key must not be empty")]
    EmptySecretAccessKey,

    /// HTTP client construction failed.
    #[error("could not construct S3 HTTP client: {message}")]
    HttpClient {
        /// Human-readable client construction failure.
        message: String,
    },
}

/// Host-side S3-compatible object-storage provider.
pub struct S3ObjectStoreProvider {
    client: Client,
    endpoint: String,
    canonical_host: String,
    bucket: String,
    region: String,
    access_key_id: SecretValue,
    secret_access_key: SecretValue,
}

impl S3ObjectStoreProvider {
    /// Creates a path-style S3-compatible provider.
    ///
    /// # Errors
    ///
    /// Returns [`S3ObjectStoreConfigError`] when endpoint, bucket, region,
    /// credentials, or HTTP client construction is invalid.
    pub fn new(
        endpoint: impl Into<String>,
        bucket: impl Into<String>,
        region: impl Into<String>,
        access_key_id: SecretValue,
        secret_access_key: SecretValue,
    ) -> Result<Self, S3ObjectStoreConfigError> {
        let endpoint = endpoint.into();
        let bucket = bucket.into();
        let region = region.into();
        let parsed = reqwest::Url::parse(&endpoint)
            .map_err(|_| S3ObjectStoreConfigError::InvalidEndpoint)?;

        if !matches!(parsed.scheme(), "http" | "https")
            || parsed.username() != ""
            || parsed.password().is_some()
            || parsed.query().is_some()
            || parsed.fragment().is_some()
            || !matches!(parsed.path(), "" | "/")
        {
            return Err(S3ObjectStoreConfigError::InvalidEndpoint);
        }

        let host = parsed
            .host_str()
            .filter(|host| !host.is_empty())
            .ok_or(S3ObjectStoreConfigError::InvalidEndpoint)?;
        let host = if host.contains(':') && !host.starts_with('[') {
            format!("[{host}]")
        } else {
            host.to_owned()
        };
        let canonical_host = match parsed.port() {
            Some(80) if parsed.scheme() == "http" => host,
            Some(443) if parsed.scheme() == "https" => host,
            Some(port) => format!("{host}:{port}"),
            None => host,
        };
        let endpoint = format!("{}://{canonical_host}", parsed.scheme());

        if bucket.is_empty()
            || bucket
                .bytes()
                .any(|byte| byte == b'/' || byte.is_ascii_control())
        {
            return Err(S3ObjectStoreConfigError::InvalidBucket);
        }

        if region.is_empty()
            || !region
                .bytes()
                .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'-')
        {
            return Err(S3ObjectStoreConfigError::InvalidRegion);
        }

        if access_key_id.expose_secret().is_empty() {
            return Err(S3ObjectStoreConfigError::EmptyAccessKeyId);
        }

        if secret_access_key.expose_secret().is_empty() {
            return Err(S3ObjectStoreConfigError::EmptySecretAccessKey);
        }

        let client = Client::builder()
            .timeout(REQUEST_TIMEOUT)
            .build()
            .map_err(|error| S3ObjectStoreConfigError::HttpClient {
                message: error.to_string(),
            })?;

        Ok(Self {
            client,
            endpoint,
            canonical_host,
            bucket,
            region,
            access_key_id,
            secret_access_key,
        })
    }

    fn validate_key(key: &str) -> Result<(), ObjectStoreError> {
        if key.is_empty() {
            return Err(ObjectStoreError::invalid_key(
                "S3 object keys must not be empty",
            ));
        }

        if key.len() > MAX_KEY_BYTES {
            return Err(ObjectStoreError::invalid_key(format!(
                "S3 object keys must not exceed {MAX_KEY_BYTES} UTF-8 bytes"
            )));
        }

        Ok(())
    }

    fn bucket_uri(&self) -> String {
        format!("/{}", aws_percent_encode(&self.bucket, true))
    }

    fn object_uri(&self, key: &str) -> String {
        format!("{}/{}", self.bucket_uri(), aws_percent_encode(key, false))
    }

    fn request_url(&self, canonical_uri: &str, canonical_query: &str) -> String {
        if canonical_query.is_empty() {
            format!("{}{}", self.endpoint, canonical_uri)
        } else {
            format!("{}{}?{}", self.endpoint, canonical_uri, canonical_query)
        }
    }

    fn send_signed(
        &self,
        method: Method,
        canonical_uri: &str,
        canonical_query: &str,
        body: Option<Vec<u8>>,
    ) -> Result<Response, ObjectStoreError> {
        let payload_hash = body.as_deref().map_or_else(|| sha256_hex(b""), sha256_hex);
        let timestamp = now().map_err(ObjectStoreError::internal)?;
        let authorization = authorization(&SigV4Input {
            access_key_id: self.access_key_id.expose_secret(),
            secret_access_key: self.secret_access_key.expose_secret(),
            region: &self.region,
            service: S3_SERVICE,
            method: method.as_str(),
            canonical_uri,
            canonical_query,
            canonical_host: &self.canonical_host,
            payload_hash: &payload_hash,
            timestamp: &timestamp,
        });

        let mut request = self
            .client
            .request(method, self.request_url(canonical_uri, canonical_query))
            .header("x-amz-content-sha256", &payload_hash)
            .header("x-amz-date", &timestamp.full)
            .header(AUTHORIZATION, authorization);

        if let Some(body) = body {
            request = request
                .header("content-type", "application/octet-stream")
                .body(body);
        }

        request.send().map_err(|error| map_transport_error(&error))
    }

    fn head_existing(&self, key: &str) -> Result<Option<ObjectInfo>, ObjectStoreError> {
        let uri = self.object_uri(key);
        let response = self.send_signed(Method::HEAD, &uri, "", None)?;

        if response.status() == StatusCode::NOT_FOUND {
            return Ok(None);
        }

        let response = require_success(response, "head")?;
        let size = response
            .headers()
            .get(CONTENT_LENGTH)
            .and_then(|value| value.to_str().ok())
            .and_then(|value| value.parse::<u64>().ok())
            .ok_or_else(|| {
                ObjectStoreError::internal(
                    "S3 HEAD response did not include a valid Content-Length header",
                )
            })?;

        Ok(Some(ObjectInfo::new(key, size)))
    }
}

impl fmt::Debug for S3ObjectStoreProvider {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("S3ObjectStoreProvider")
            .field("endpoint", &self.endpoint)
            .field("bucket", &self.bucket)
            .field("region", &self.region)
            .field("access_key_id", &"[redacted]")
            .field("secret_access_key", &"[redacted]")
            .finish_non_exhaustive()
    }
}

impl ObjectStoreProvider for S3ObjectStoreProvider {
    fn get(&self, key: &str) -> Result<Option<ObjectRead>, ObjectStoreError> {
        Self::validate_key(key)?;
        let uri = self.object_uri(key);
        let response = self.send_signed(Method::GET, &uri, "", None)?;

        if response.status() == StatusCode::NOT_FOUND {
            return Ok(None);
        }

        let response = require_success(response, "get")?;
        let size = response.content_length().ok_or_else(|| {
            ObjectStoreError::internal("S3 GET response did not include Content-Length")
        })?;

        Ok(Some(ObjectRead::new(
            ObjectInfo::new(key, size),
            Box::new(response),
        )))
    }

    fn put(
        &self,
        key: &str,
        size: u64,
        body: &mut dyn Read,
    ) -> Result<ObjectInfo, ObjectStoreError> {
        Self::validate_key(key)?;
        let bytes = read_exact_body(size, body)?;
        let uri = self.object_uri(key);
        let response = self.send_signed(Method::PUT, &uri, "", Some(bytes))?;

        require_success(response, "put")?;
        Ok(ObjectInfo::new(key, size))
    }

    fn head(&self, key: &str) -> Result<Option<ObjectInfo>, ObjectStoreError> {
        Self::validate_key(key)?;
        self.head_existing(key)
    }

    fn delete(&self, key: &str) -> Result<bool, ObjectStoreError> {
        Self::validate_key(key)?;

        // S3 DELETE is idempotent and does not report whether an object existed.
        // Presolve's baseline contract does, so preserve the sequential result by
        // checking existence first. As with Workers KV, this cannot be atomic
        // against concurrent external mutation and must not be represented as
        // stronger semantics than the provider actually supplies.
        if self.head_existing(key)?.is_none() {
            return Ok(false);
        }

        let uri = self.object_uri(key);
        let response = self.send_signed(Method::DELETE, &uri, "", None)?;
        require_success(response, "delete")?;
        Ok(true)
    }

    fn list(&self, prefix: &str) -> Result<Vec<ObjectInfo>, ObjectStoreError> {
        let mut objects = Vec::new();
        let mut continuation_token: Option<String> = None;

        loop {
            let mut pairs = vec![
                ("encoding-type", "url"),
                ("list-type", "2"),
                ("prefix", prefix),
            ];

            if let Some(token) = continuation_token.as_deref() {
                pairs.push(("continuation-token", token));
            }

            let query = canonical_query(pairs);
            let response = self.send_signed(Method::GET, &self.bucket_uri(), &query, None)?;
            let response = require_success(response, "list")?;
            let xml = response.text().map_err(|error| {
                ObjectStoreError::internal(format!("could not read S3 list response: {error}"))
            })?;
            let page = parse_list_page(&xml)?;
            objects.extend(page.objects);

            let Some(next_token) = page.next_continuation_token else {
                break;
            };

            if continuation_token.as_deref() == Some(next_token.as_str()) {
                return Err(ObjectStoreError::internal(
                    "S3 listing returned the same continuation token twice",
                ));
            }

            continuation_token = Some(next_token);
        }

        Ok(objects)
    }
}

fn read_exact_body(size: u64, body: &mut dyn Read) -> Result<Vec<u8>, ObjectStoreError> {
    let mut bytes = Vec::new();

    {
        let mut limited = (&mut *body).take(size);
        limited.read_to_end(&mut bytes).map_err(|error| {
            ObjectStoreError::invalid_body(format!("failed to read object body: {error}"))
        })?;
    }

    let actual = u64::try_from(bytes.len())
        .map_err(|_| ObjectStoreError::invalid_body("object body is too large"))?;

    if actual != size {
        return Err(ObjectStoreError::invalid_body(format!(
            "object body length mismatch: expected {size} bytes, found {actual}"
        )));
    }

    let mut extra = [0_u8; 1];
    let trailing = body.read(&mut extra).map_err(|error| {
        ObjectStoreError::invalid_body(format!("failed to read object body: {error}"))
    })?;

    if trailing != 0 {
        return Err(ObjectStoreError::invalid_body(format!(
            "object body length exceeds declared size of {size} bytes"
        )));
    }

    Ok(bytes)
}

fn map_transport_error(error: &reqwest::Error) -> ObjectStoreError {
    if error.is_timeout() || error.is_connect() {
        ObjectStoreError::unavailable(format!(
            "S3-compatible object storage is unavailable: {error}"
        ))
    } else {
        ObjectStoreError::internal(format!("S3-compatible object request failed: {error}"))
    }
}

fn require_success(response: Response, operation: &str) -> Result<Response, ObjectStoreError> {
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
        return Err(ObjectStoreError::unavailable(format!(
            "S3-compatible {operation} returned HTTP {status}"
        )));
    }

    Err(ObjectStoreError::internal(format!(
        "S3-compatible {operation} returned unexpected HTTP {status}"
    )))
}

/// Provider-realization adapter for S3-compatible object storage.
#[derive(Debug, Default, Clone, Copy)]
pub struct S3ObjectStoreAdapter;

impl presolve_provider_realization::ProviderAdapter for S3ObjectStoreAdapter {
    fn realize(
        &self,
        profile: &presolve_provider_realization::ProviderRealizationProfile,
        requested: &[presolve_provider_realization::ProviderCapabilityRequest],
        secrets: &dyn presolve_provider_realization::SecretResolver,
    ) -> Result<
        presolve_provider_realization::RealizedProvider,
        presolve_provider_realization::ProviderAdapterError,
    > {
        use presolve_provider_realization::ProviderAdapterError;
        use presolve_provider_sdk::OBJECT_STORE_INTERFACE;
        use semver::Version;

        let supported_version = Version::new(0, 1, 0);

        for capability in requested {
            if capability.interface() != OBJECT_STORE_INTERFACE {
                return Err(ProviderAdapterError::UnsupportedCapability {
                    interface: capability.interface().to_owned(),
                });
            }

            if capability.version() != &supported_version {
                return Err(ProviderAdapterError::UnsupportedVersion {
                    interface: OBJECT_STORE_INTERFACE.to_owned(),
                    selected_version: capability.version().clone(),
                    supported_version: supported_version.clone(),
                });
            }

            if !capability.negotiated_features().is_empty() {
                return Err(ProviderAdapterError::UnsupportedFeatures {
                    interface: OBJECT_STORE_INTERFACE.to_owned(),
                    features: capability.negotiated_features().join(", "),
                });
            }
        }

        let endpoint = required_setting(profile, S3_ENDPOINT_SETTING)?;
        let bucket = required_setting(profile, S3_BUCKET_SETTING)?;
        let region = required_setting(profile, S3_REGION_SETTING)?;
        let access_key_id = resolve_secret(profile, S3_ACCESS_KEY_ID_SECRET, secrets)?;
        let secret_access_key = resolve_secret(profile, S3_SECRET_ACCESS_KEY_SECRET, secrets)?;

        let provider =
            S3ObjectStoreProvider::new(endpoint, bucket, region, access_key_id, secret_access_key)
                .map_err(|error| ProviderAdapterError::Configuration {
                    message: error.to_string(),
                })?;

        Ok(
            presolve_provider_realization::RealizedProvider::new(*profile.provider_id())
                .with_object_store(Arc::new(provider)),
        )
    }
}

fn required_setting<'a>(
    profile: &'a presolve_provider_realization::ProviderRealizationProfile,
    key: &str,
) -> Result<&'a str, presolve_provider_realization::ProviderAdapterError> {
    profile.setting(key).ok_or_else(|| {
        presolve_provider_realization::ProviderAdapterError::MissingSetting {
            key: key.to_owned(),
        }
    })
}

fn resolve_secret(
    profile: &presolve_provider_realization::ProviderRealizationProfile,
    name: &str,
    secrets: &dyn presolve_provider_realization::SecretResolver,
) -> Result<SecretValue, presolve_provider_realization::ProviderAdapterError> {
    let reference = profile.secret(name).ok_or_else(|| {
        presolve_provider_realization::ProviderAdapterError::MissingSecret {
            name: name.to_owned(),
        }
    })?;

    secrets.resolve(reference).map_err(|error| {
        presolve_provider_realization::ProviderAdapterError::SecretResolution {
            name: name.to_owned(),
            message: error.to_string(),
        }
    })
}

#[cfg(test)]
mod tests {
    use std::{
        io::{BufRead, BufReader, Read, Write},
        net::{TcpListener, TcpStream},
        sync::mpsc,
        thread,
    };

    use presolve_provider_sdk::ObjectStoreErrorCode;

    use super::*;

    type TestResponse = (
        &'static str,
        Vec<(&'static str, &'static str)>,
        &'static [u8],
    );

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

    fn provider(endpoint: String) -> S3ObjectStoreProvider {
        S3ObjectStoreProvider::new(
            endpoint,
            "presolve-test",
            "us-east-1",
            SecretValue::new("test-access-key"),
            SecretValue::new("test-secret-key"),
        )
        .expect("test provider should configure")
    }

    fn serve(
        responses: Vec<TestResponse>,
    ) -> (
        String,
        mpsc::Receiver<Vec<RecordedRequest>>,
        thread::JoinHandle<()>,
    ) {
        let listener = TcpListener::bind("127.0.0.1:0").expect("test listener should bind");
        let address = listener
            .local_addr()
            .expect("test listener should have address");
        let (sender, receiver) = mpsc::channel();

        let handle = thread::spawn(move || {
            let mut requests = Vec::new();

            for (status, headers, body) in responses {
                let (mut stream, _) = listener.accept().expect("test request should arrive");
                requests.push(read_request(&mut stream));

                let mut response = format!(
                    "HTTP/1.1 {status}\r\nContent-Length: {}\r\nConnection: close\r\n",
                    body.len()
                );

                for (name, value) in headers {
                    use std::fmt::Write as _;
                    write!(&mut response, "{name}: {value}\r\n")
                        .expect("writing response header to String should succeed");
                }

                response.push_str("\r\n");
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
    fn rejects_invalid_configuration() {
        assert!(matches!(
            S3ObjectStoreProvider::new(
                "ftp://example.com",
                "bucket",
                "us-east-1",
                SecretValue::new("key"),
                SecretValue::new("secret")
            ),
            Err(S3ObjectStoreConfigError::InvalidEndpoint)
        ));

        assert!(matches!(
            S3ObjectStoreProvider::new(
                "https://example.com",
                "bad/bucket",
                "us-east-1",
                SecretValue::new("key"),
                SecretValue::new("secret")
            ),
            Err(S3ObjectStoreConfigError::InvalidBucket)
        ));
    }

    #[test]
    fn debug_output_redacts_credentials() {
        let provider = S3ObjectStoreProvider::new(
            "https://objects.example.com",
            "bucket",
            "auto",
            SecretValue::new("visible-access"),
            SecretValue::new("visible-secret"),
        )
        .expect("provider should configure");
        let debug = format!("{provider:?}");

        assert!(debug.contains("[redacted]"));
        assert!(!debug.contains("visible-access"));
        assert!(!debug.contains("visible-secret"));
    }

    #[test]
    fn get_signs_encoded_path_and_streams_body() {
        let (endpoint, receiver, handle) = serve(vec![("200 OK", vec![], b"data")]);
        let provider = provider(endpoint);
        let mut object = provider
            .get("folder/Case key.bin")
            .expect("get should succeed")
            .expect("object should exist");
        let mut body = Vec::new();
        object
            .body()
            .read_to_end(&mut body)
            .expect("body should read");

        assert_eq!(body, b"data");
        assert_eq!(object.info().size(), 4);

        let requests = finish(&receiver, handle);
        assert_eq!(
            requests[0].request_line,
            "GET /presolve-test/folder/Case%20key.bin HTTP/1.1"
        );
        assert!(
            requests[0]
                .header("authorization")
                .is_some_and(|value| value.starts_with("AWS4-HMAC-SHA256 Credential="))
        );
        assert!(requests[0].header("x-amz-date").is_some());
        assert!(requests[0].header("x-amz-content-sha256").is_some());
        assert_eq!(requests[0].body, Vec::<u8>::new());
    }

    #[test]
    fn put_validates_declared_body_before_network_mutation() {
        let provider = S3ObjectStoreProvider::new(
            "http://127.0.0.1:1",
            "bucket",
            "us-east-1",
            SecretValue::new("key"),
            SecretValue::new("secret"),
        )
        .expect("provider should configure");
        let mut body = std::io::Cursor::new(b"short".to_vec());

        let error = provider
            .put("example", 6, &mut body)
            .expect_err("short body should fail before network access");
        assert_eq!(error.code(), ObjectStoreErrorCode::InvalidBody);
    }

    #[test]
    fn delete_missing_object_does_not_send_delete() {
        let (endpoint, receiver, handle) = serve(vec![("404 Not Found", vec![], b"")]);
        let provider = provider(endpoint);

        assert!(!provider.delete("missing").expect("delete should succeed"));

        let requests = finish(&receiver, handle);
        assert_eq!(requests.len(), 1);
        assert!(requests[0].request_line.starts_with("HEAD "));
    }

    #[test]
    fn list_uses_v2_prefix_query() {
        let xml = br"<ListBucketResult><EncodingType>url</EncodingType><IsTruncated>false</IsTruncated><Contents><Key>demo%2Fa.bin</Key><Size>2</Size></Contents></ListBucketResult>";
        let (endpoint, receiver, handle) = serve(vec![("200 OK", vec![], xml)]);
        let provider = provider(endpoint);

        assert_eq!(
            provider.list("demo/").expect("list should succeed"),
            vec![ObjectInfo::new("demo/a.bin", 2)]
        );

        let requests = finish(&receiver, handle);
        assert_eq!(
            requests[0].request_line,
            "GET /presolve-test?encoding-type=url&list-type=2&prefix=demo%2F HTTP/1.1"
        );
    }
}

#[cfg(test)]
mod realization_tests {
    use presolve_core::ProviderId;
    use presolve_provider_realization::{
        ProviderAdapter, ProviderAdapterError, ProviderAdapterId, ProviderCapabilityRequest,
        ProviderRealizationProfile, SecretReference, SecretResolutionError, SecretResolver,
    };
    use presolve_provider_sdk::OBJECT_STORE_INTERFACE;
    use semver::Version;

    use super::*;

    struct StaticSecrets;

    impl SecretResolver for StaticSecrets {
        fn resolve(
            &self,
            reference: &SecretReference,
        ) -> Result<SecretValue, SecretResolutionError> {
            let value = match reference.environment_variable_name() {
                "PRESOLVE_TEST_S3_ACCESS_KEY_ID" => "test-access-key",
                "PRESOLVE_TEST_S3_SECRET_ACCESS_KEY" => "test-secret-key",
                other => panic!("unexpected test secret reference: {other}"),
            };

            Ok(SecretValue::new(value))
        }
    }

    fn profile() -> ProviderRealizationProfile {
        ProviderRealizationProfile::new(
            ProviderId::new(),
            ProviderAdapterId::new(S3_OBJECTS_ADAPTER_ID).expect("adapter ID should be valid"),
        )
        .with_setting(S3_ENDPOINT_SETTING, "http://127.0.0.1:9000")
        .expect("endpoint setting should work")
        .with_setting(S3_BUCKET_SETTING, "presolve-test")
        .expect("bucket setting should work")
        .with_setting(S3_REGION_SETTING, "us-east-1")
        .expect("region setting should work")
        .with_secret(
            S3_ACCESS_KEY_ID_SECRET,
            SecretReference::environment_variable("PRESOLVE_TEST_S3_ACCESS_KEY_ID")
                .expect("secret reference should be valid"),
        )
        .expect("access key binding should work")
        .with_secret(
            S3_SECRET_ACCESS_KEY_SECRET,
            SecretReference::environment_variable("PRESOLVE_TEST_S3_SECRET_ACCESS_KEY")
                .expect("secret reference should be valid"),
        )
        .expect("secret key binding should work")
    }

    #[test]
    fn adapter_realizes_selected_baseline_objects() {
        let profile = profile();
        let request = ProviderCapabilityRequest::new(
            OBJECT_STORE_INTERFACE,
            Version::new(0, 1, 0),
            std::iter::empty::<&str>(),
        );

        let realized = S3ObjectStoreAdapter
            .realize(&profile, &[request], &StaticSecrets)
            .expect("S3 adapter should realize");

        assert_eq!(realized.provider_id(), profile.provider_id());
        assert!(realized.object_store().is_some());
        assert!(realized.key_value().is_none());
    }

    #[test]
    fn adapter_rejects_unsupported_features() {
        let profile = profile();
        let request = ProviderCapabilityRequest::new(
            OBJECT_STORE_INTERFACE,
            Version::new(0, 1, 0),
            ["conditional-put"],
        );

        let error = S3ObjectStoreAdapter
            .realize(&profile, &[request], &StaticSecrets)
            .expect_err("unsupported feature should fail");

        assert!(matches!(
            error,
            ProviderAdapterError::UnsupportedFeatures { .. }
        ));
    }
}
