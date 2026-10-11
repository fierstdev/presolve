use std::{
    env,
    io::{Read, Write},
    net::TcpStream,
    time::Duration,
};

use serde_json::Value;
use thiserror::Error;

use crate::{
    CAPABILITY_BRIDGE_PATH, CAPABILITY_BRIDGE_PROTOCOL, CAPABILITY_ENDPOINT_ENV,
    CAPABILITY_PROTOCOL_ENV, CAPABILITY_TOKEN_ENV, CapabilityBridgeRequest,
    CapabilityBridgeResponse,
};

const HTTP_HEADER_END: &[u8] = b"\r\n\r\n";

/// Failure while invoking the Presolve capability bridge.
#[derive(Debug, Error)]
pub enum CapabilityBridgeClientError {
    /// One required discovery environment variable is absent.
    #[error("required capability environment variable `{name}` is not set")]
    MissingEnvironment {
        /// Missing variable name.
        name: &'static str,
    },

    /// The container was configured for a protocol this client does not support.
    #[error("unsupported capability protocol `{found}`; expected `{expected}`")]
    ProtocolMismatch {
        /// Protocol advertised to the container.
        found: String,

        /// Protocol implemented by this client.
        expected: &'static str,
    },

    /// The endpoint URL does not match the initial bridge URL form.
    #[error("invalid capability endpoint `{endpoint}`")]
    InvalidEndpoint {
        /// Invalid endpoint.
        endpoint: String,
    },

    /// TCP I/O failed.
    #[error(transparent)]
    Io(#[from] std::io::Error),

    /// Request serialization failed.
    #[error("could not encode capability request: {0}")]
    Encode(#[source] serde_json::Error),

    /// The bridge returned malformed HTTP.
    #[error("invalid HTTP response from capability bridge")]
    InvalidHttp,

    /// The bridge returned malformed JSON.
    #[error("could not decode capability response: {0}")]
    Decode(#[source] serde_json::Error),

    /// The bridge or selected provider rejected the operation.
    #[error("capability bridge error `{code}`: {message}")]
    Remote {
        /// Stable remote error category.
        code: String,

        /// Human-readable remote message.
        message: String,
    },
}

/// Blocking client for the initial Presolve HTTP/JSON capability bridge.
#[derive(Debug, Clone)]
pub struct CapabilityBridgeClient {
    endpoint: Endpoint,
    token: String,
}

impl CapabilityBridgeClient {
    /// Creates a client from an explicit endpoint and bearer token.
    ///
    /// # Errors
    ///
    /// Returns [`CapabilityBridgeClientError::InvalidEndpoint`] when `endpoint`
    /// does not use the supported `http://host:port/v1/call` form.
    pub fn new(
        endpoint: impl Into<String>,
        token: impl Into<String>,
    ) -> Result<Self, CapabilityBridgeClientError> {
        let endpoint = endpoint.into();

        Ok(Self {
            endpoint: Endpoint::parse(&endpoint)?,
            token: token.into(),
        })
    }

    /// Discovers bridge configuration from Presolve's container environment.
    ///
    /// # Errors
    ///
    /// Returns an error when discovery variables are absent, the protocol does
    /// not match this client, or the endpoint is invalid.
    pub fn from_env() -> Result<Self, CapabilityBridgeClientError> {
        let protocol = env::var(CAPABILITY_PROTOCOL_ENV).map_err(|_| {
            CapabilityBridgeClientError::MissingEnvironment {
                name: CAPABILITY_PROTOCOL_ENV,
            }
        })?;

        if protocol != CAPABILITY_BRIDGE_PROTOCOL {
            return Err(CapabilityBridgeClientError::ProtocolMismatch {
                found: protocol,
                expected: CAPABILITY_BRIDGE_PROTOCOL,
            });
        }

        let endpoint = env::var(CAPABILITY_ENDPOINT_ENV).map_err(|_| {
            CapabilityBridgeClientError::MissingEnvironment {
                name: CAPABILITY_ENDPOINT_ENV,
            }
        })?;
        let token = env::var(CAPABILITY_TOKEN_ENV).map_err(|_| {
            CapabilityBridgeClientError::MissingEnvironment {
                name: CAPABILITY_TOKEN_ENV,
            }
        })?;

        Self::new(endpoint, token)
    }

    /// Invokes one semantic capability operation.
    ///
    /// # Errors
    ///
    /// Returns transport errors or a stable remote/provider error.
    pub fn call(
        &self,
        interface: &str,
        operation: &str,
        input: Value,
    ) -> Result<Value, CapabilityBridgeClientError> {
        let response = self.send(&CapabilityBridgeRequest::new(interface, operation, input))?;

        match response {
            CapabilityBridgeResponse::Ok { output } => Ok(output),
            CapabilityBridgeResponse::Error { code, message } => {
                Err(CapabilityBridgeClientError::Remote { code, message })
            }
        }
    }

    fn send(
        &self,
        request: &CapabilityBridgeRequest,
    ) -> Result<CapabilityBridgeResponse, CapabilityBridgeClientError> {
        let body = serde_json::to_vec(request).map_err(CapabilityBridgeClientError::Encode)?;
        let mut stream = TcpStream::connect((self.endpoint.host.as_str(), self.endpoint.port))?;
        let timeout = Some(Duration::from_secs(30));

        stream.set_read_timeout(timeout)?;
        stream.set_write_timeout(timeout)?;

        let header = format!(
            "POST {} HTTP/1.1\r\nHost: {}:{}\r\nAuthorization: Bearer {}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
            self.endpoint.path,
            self.endpoint.host,
            self.endpoint.port,
            self.token,
            body.len()
        );

        stream.write_all(header.as_bytes())?;
        stream.write_all(&body)?;
        stream.flush()?;

        let mut response = Vec::new();
        stream.read_to_end(&mut response)?;

        let header_end = response
            .windows(HTTP_HEADER_END.len())
            .position(|window| window == HTTP_HEADER_END)
            .ok_or(CapabilityBridgeClientError::InvalidHttp)?;

        let headers = std::str::from_utf8(&response[..header_end])
            .map_err(|_| CapabilityBridgeClientError::InvalidHttp)?;
        let status = headers
            .lines()
            .next()
            .and_then(|line| line.split_whitespace().nth(1))
            .and_then(|value| value.parse::<u16>().ok())
            .ok_or(CapabilityBridgeClientError::InvalidHttp)?;

        let body_start = header_end + HTTP_HEADER_END.len();
        let bridge_response: CapabilityBridgeResponse =
            serde_json::from_slice(&response[body_start..])
                .map_err(CapabilityBridgeClientError::Decode)?;

        if !(200..300).contains(&status)
            && matches!(&bridge_response, CapabilityBridgeResponse::Ok { .. })
        {
            return Err(CapabilityBridgeClientError::InvalidHttp);
        }

        Ok(bridge_response)
    }
}

#[derive(Debug, Clone)]
struct Endpoint {
    host: String,
    port: u16,
    path: String,
}

impl Endpoint {
    fn parse(value: &str) -> Result<Self, CapabilityBridgeClientError> {
        let Some(rest) = value.strip_prefix("http://") else {
            return Err(CapabilityBridgeClientError::InvalidEndpoint {
                endpoint: value.to_owned(),
            });
        };

        let Some((authority, path)) = rest.split_once('/') else {
            return Err(CapabilityBridgeClientError::InvalidEndpoint {
                endpoint: value.to_owned(),
            });
        };
        let path = format!("/{path}");

        if path != CAPABILITY_BRIDGE_PATH {
            return Err(CapabilityBridgeClientError::InvalidEndpoint {
                endpoint: value.to_owned(),
            });
        }

        let Some((host, port)) = authority.rsplit_once(':') else {
            return Err(CapabilityBridgeClientError::InvalidEndpoint {
                endpoint: value.to_owned(),
            });
        };
        let port =
            port.parse::<u16>()
                .map_err(|_| CapabilityBridgeClientError::InvalidEndpoint {
                    endpoint: value.to_owned(),
                })?;

        if host.is_empty() {
            return Err(CapabilityBridgeClientError::InvalidEndpoint {
                endpoint: value.to_owned(),
            });
        }

        Ok(Self {
            host: host.to_owned(),
            port,
            path,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn endpoint_requires_versioned_http_call_path() {
        let client = CapabilityBridgeClient::new("http://localhost:1234/v1/call", "secret")
            .expect("valid endpoint should parse");

        assert_eq!(client.endpoint.host, "localhost");
        assert_eq!(client.endpoint.port, 1234);
        assert_eq!(client.endpoint.path, "/v1/call");
    }

    #[test]
    fn endpoint_rejects_wrong_path() {
        let Err(error) = CapabilityBridgeClient::new("http://localhost:1234/other", "secret")
        else {
            panic!("wrong path should fail");
        };

        assert!(matches!(
            error,
            CapabilityBridgeClientError::InvalidEndpoint { .. }
        ));
    }
}
