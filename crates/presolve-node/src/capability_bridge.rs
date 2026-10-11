use std::{
    io::{BufRead, BufReader, Cursor, Read, Write},
    net::{TcpListener, TcpStream},
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
    thread::{self, JoinHandle},
    time::Duration,
};

use presolve_capability::{key_value_contract, object_store_contract};
use presolve_capability_bridge::{
    CAPABILITY_BRIDGE_PATH, CAPABILITY_BRIDGE_PROTOCOL, CapabilityBridgeRequest,
    CapabilityBridgeResponse,
};
use presolve_executor_sdk::{
    ContainerCapabilityBinding, ContainerCapabilityToken, ContainerCapabilityTransport,
};
use presolve_provider_sdk::{
    KEY_VALUE_INTERFACE, KeyValueError, KeyValueErrorCode, KeyValueProvider,
    OBJECT_STORE_INTERFACE, ObjectInfo, ObjectStoreError, ObjectStoreErrorCode,
    ObjectStoreProvider,
};
use presolve_resolver::DeploymentPlan;
use serde::Deserialize;
use serde_json::{Value, json};
use thiserror::Error;
use uuid::Uuid;

use crate::ProviderRegistry;

const MAX_HEADER_LINE_BYTES: usize = 16 * 1024;

/// Failure while creating the node-hosted container capability bridge.
#[derive(Debug, Error)]
pub enum ContainerCapabilityGatewayError {
    /// The selected capability has no bridge implementation.
    #[error("container capability bridge does not implement `{interface}`")]
    UnsupportedCapability {
        /// Unsupported semantic capability.
        interface: String,
    },

    /// The bridge implements a different exact semantic version.
    #[error(
        "container capability bridge implements `{interface}` at `{supported_version}`, \
         but the plan selected `{selected_version}`"
    )]
    UnsupportedVersion {
        /// Semantic capability interface.
        interface: String,

        /// Exact version selected by resolution.
        selected_version: String,

        /// Exact version implemented by the bridge.
        supported_version: String,
    },

    /// The bridge does not yet project negotiated optional capability features.
    #[error(
        "container capability bridge for `{interface}` does not implement negotiated features: {features}"
    )]
    UnsupportedFeatures {
        /// Semantic capability interface.
        interface: String,

        /// Negotiated feature names.
        features: String,
    },

    /// The selected provider implementation is absent from the node registry.
    #[error(
        "provider `{provider_id}` selected for capability `{interface}` \
         has no registered implementation"
    )]
    MissingProvider {
        /// Capability interface.
        interface: String,

        /// Selected provider identity.
        provider_id: presolve_core::ProviderId,
    },

    /// A built-in capability contract could not be constructed.
    #[error("could not load built-in capability contract `{interface}`: {message}")]
    InvalidBuiltinContract {
        /// Built-in capability interface.
        interface: &'static str,

        /// Contract construction failure.
        message: String,
    },

    /// The local bridge listener could not be created.
    #[error("could not start container capability bridge: {0}")]
    Io(#[from] std::io::Error),
}

/// Running node-hosted capability bridge.
///
/// Dropping this value stops the listener. Callers must keep the gateway alive
/// for as long as launched workloads need their capabilities.
pub struct ContainerCapabilityGateway {
    transport: ContainerCapabilityTransport,
    stop: Arc<AtomicBool>,
    thread: Option<JoinHandle<()>>,
}

impl ContainerCapabilityGateway {
    /// Returns the backend-neutral transport description injected into
    /// container launch requests.
    #[must_use]
    pub const fn transport(&self) -> &ContainerCapabilityTransport {
        &self.transport
    }
}

impl Drop for ContainerCapabilityGateway {
    fn drop(&mut self) {
        self.stop.store(true, Ordering::Release);

        if let Some(thread) = self.thread.take() {
            let _ = thread.join();
        }
    }
}

/// Starts an authenticated HTTP/JSON bridge for capability bindings selected in
/// `plan`.
///
/// The bridge uses the exact provider identities already chosen by resolution.
/// It never re-selects or substitutes providers.
///
/// # Errors
///
/// Returns an error when a selected capability/version/feature set is not
/// implemented by bridge protocol v0.1, a provider implementation is missing,
/// or the listener cannot be started.
pub fn start_container_capability_gateway(
    plan: &DeploymentPlan,
    registry: &ProviderRegistry,
) -> Result<ContainerCapabilityGateway, ContainerCapabilityGatewayError> {
    let routes = GatewayProviders::from_plan(plan, registry)?;
    let listener = TcpListener::bind(("0.0.0.0", 0))?;
    let port = listener.local_addr()?.port();
    listener.set_nonblocking(true)?;

    let token = format!("pcb_{}", Uuid::new_v4().simple());
    let bindings = plan
        .bindings()
        .iter()
        .map(|binding| {
            ContainerCapabilityBinding::new(
                binding.interface(),
                binding.provider_version().to_string(),
            )
        })
        .collect();

    let transport = ContainerCapabilityTransport::new(
        CAPABILITY_BRIDGE_PROTOCOL,
        port,
        ContainerCapabilityToken::new(token.clone()),
        bindings,
    );

    let stop = Arc::new(AtomicBool::new(false));
    let server_stop = Arc::clone(&stop);

    let thread = thread::spawn(move || {
        while !server_stop.load(Ordering::Acquire) {
            match listener.accept() {
                Ok((stream, _)) => {
                    let routes = routes.clone();
                    let token = token.clone();

                    thread::spawn(move || {
                        let _ = handle_connection(stream, &token, &routes);
                    });
                }

                Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => {
                    thread::sleep(Duration::from_millis(10));
                }

                Err(_) => break,
            }
        }
    });

    Ok(ContainerCapabilityGateway {
        transport,
        stop,
        thread: Some(thread),
    })
}

#[derive(Clone, Default)]
struct GatewayProviders {
    key_value: Option<Arc<dyn KeyValueProvider>>,
    object_store: Option<Arc<dyn ObjectStoreProvider>>,
}

impl GatewayProviders {
    fn from_plan(
        plan: &DeploymentPlan,
        registry: &ProviderRegistry,
    ) -> Result<Self, ContainerCapabilityGatewayError> {
        let mut providers = Self::default();

        for binding in plan.bindings() {
            match binding.interface() {
                KEY_VALUE_INTERFACE => {
                    validate_binding(
                        binding.interface(),
                        binding.provider_version().to_string(),
                        binding.negotiated_features(),
                        key_value_contract()
                            .map_err(|error| {
                                ContainerCapabilityGatewayError::InvalidBuiltinContract {
                                    interface: KEY_VALUE_INTERFACE,
                                    message: error.to_string(),
                                }
                            })?
                            .version()
                            .to_string(),
                    )?;

                    let provider_id = *binding.provider_id();
                    providers.key_value =
                        Some(registry.key_value(provider_id).ok_or_else(|| {
                            ContainerCapabilityGatewayError::MissingProvider {
                                interface: binding.interface().to_owned(),
                                provider_id,
                            }
                        })?);
                }

                OBJECT_STORE_INTERFACE => {
                    validate_binding(
                        binding.interface(),
                        binding.provider_version().to_string(),
                        binding.negotiated_features(),
                        object_store_contract()
                            .map_err(|error| {
                                ContainerCapabilityGatewayError::InvalidBuiltinContract {
                                    interface: OBJECT_STORE_INTERFACE,
                                    message: error.to_string(),
                                }
                            })?
                            .version()
                            .to_string(),
                    )?;

                    let provider_id = *binding.provider_id();
                    providers.object_store =
                        Some(registry.object_store(provider_id).ok_or_else(|| {
                            ContainerCapabilityGatewayError::MissingProvider {
                                interface: binding.interface().to_owned(),
                                provider_id,
                            }
                        })?);
                }

                interface => {
                    return Err(ContainerCapabilityGatewayError::UnsupportedCapability {
                        interface: interface.to_owned(),
                    });
                }
            }
        }

        Ok(providers)
    }
}

fn validate_binding(
    interface: &str,
    selected_version: String,
    negotiated_features: &[String],
    supported_version: String,
) -> Result<(), ContainerCapabilityGatewayError> {
    if selected_version != supported_version {
        return Err(ContainerCapabilityGatewayError::UnsupportedVersion {
            interface: interface.to_owned(),
            selected_version,
            supported_version,
        });
    }

    if !negotiated_features.is_empty() {
        return Err(ContainerCapabilityGatewayError::UnsupportedFeatures {
            interface: interface.to_owned(),
            features: negotiated_features.join(", "),
        });
    }

    Ok(())
}

fn handle_connection(
    mut stream: TcpStream,
    token: &str,
    routes: &GatewayProviders,
) -> Result<(), std::io::Error> {
    let mut reader = BufReader::new(stream.try_clone()?);
    let mut request_line = String::new();

    if reader.read_line(&mut request_line)? == 0 {
        return Ok(());
    }

    if request_line.len() > MAX_HEADER_LINE_BYTES {
        return write_bridge_response(
            &mut stream,
            400,
            &CapabilityBridgeResponse::error("invalid-request", "request line is too large"),
        );
    }

    let mut parts = request_line.split_whitespace();
    let method = parts.next().unwrap_or_default();
    let path = parts.next().unwrap_or_default();

    let mut authorization = None;
    let mut content_length = None;

    loop {
        let mut line = String::new();
        let bytes = reader.read_line(&mut line)?;

        if bytes == 0 || line == "\r\n" || line == "\n" {
            break;
        }

        if line.len() > MAX_HEADER_LINE_BYTES {
            return write_bridge_response(
                &mut stream,
                400,
                &CapabilityBridgeResponse::error("invalid-request", "header line is too large"),
            );
        }

        if let Some((name, value)) = line.split_once(':') {
            let value = value.trim();

            if name.eq_ignore_ascii_case("authorization") {
                authorization = Some(value.to_owned());
            } else if name.eq_ignore_ascii_case("content-length") {
                content_length = value.parse::<usize>().ok();
            }
        }
    }

    if method != "POST" || path != CAPABILITY_BRIDGE_PATH {
        return write_bridge_response(
            &mut stream,
            404,
            &CapabilityBridgeResponse::error("not-found", "unknown capability bridge endpoint"),
        );
    }

    let expected_authorization = format!("Bearer {token}");

    if authorization.as_deref() != Some(expected_authorization.as_str()) {
        return write_bridge_response(
            &mut stream,
            401,
            &CapabilityBridgeResponse::error(
                "unauthorized",
                "capability bridge bearer token is missing or invalid",
            ),
        );
    }

    let Some(content_length) = content_length else {
        return write_bridge_response(
            &mut stream,
            411,
            &CapabilityBridgeResponse::error("invalid-request", "Content-Length is required"),
        );
    };

    let mut body = vec![0_u8; content_length];
    reader.read_exact(&mut body)?;

    let request: CapabilityBridgeRequest = match serde_json::from_slice(&body) {
        Ok(request) => request,
        Err(error) => {
            return write_bridge_response(
                &mut stream,
                400,
                &CapabilityBridgeResponse::error(
                    "invalid-request",
                    format!("invalid JSON request: {error}"),
                ),
            );
        }
    };

    let response = dispatch(&request, routes);
    write_bridge_response(&mut stream, 200, &response)
}

fn write_bridge_response(
    stream: &mut TcpStream,
    status: u16,
    response: &CapabilityBridgeResponse,
) -> Result<(), std::io::Error> {
    let body = serde_json::to_vec(&response).map_err(std::io::Error::other)?;
    let reason = match status {
        200 => "OK",
        400 => "Bad Request",
        401 => "Unauthorized",
        404 => "Not Found",
        411 => "Length Required",
        _ => "Error",
    };
    let header = format!(
        "HTTP/1.1 {status} {reason}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
        body.len()
    );

    stream.write_all(header.as_bytes())?;
    stream.write_all(&body)?;
    stream.flush()
}

fn dispatch(
    request: &CapabilityBridgeRequest,
    routes: &GatewayProviders,
) -> CapabilityBridgeResponse {
    match request.interface() {
        KEY_VALUE_INTERFACE => {
            let Some(provider) = routes.key_value.as_ref() else {
                return CapabilityBridgeResponse::error(
                    "unavailable",
                    "key/value capability is not bound",
                );
            };

            dispatch_key_value(provider.as_ref(), request.operation(), request.input())
        }

        OBJECT_STORE_INTERFACE => {
            let Some(provider) = routes.object_store.as_ref() else {
                return CapabilityBridgeResponse::error(
                    "unavailable",
                    "object-storage capability is not bound",
                );
            };

            dispatch_objects(provider.as_ref(), request.operation(), request.input())
        }

        interface => CapabilityBridgeResponse::error(
            "unsupported-interface",
            format!("capability interface `{interface}` is not exposed by this deployment"),
        ),
    }
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct KeyInput {
    key: String,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct KeyValueInput {
    key: String,
    value: String,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ObjectPutInput {
    key: String,
    body: Vec<u8>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ObjectListInput {
    prefix: String,
}

fn dispatch_key_value(
    provider: &dyn KeyValueProvider,
    operation: &str,
    input: &Value,
) -> CapabilityBridgeResponse {
    match operation {
        "get" => {
            let input: KeyInput = match decode_input(input) {
                Ok(input) => input,
                Err(response) => return response,
            };

            match provider.get(&input.key) {
                Ok(value) => CapabilityBridgeResponse::ok(json!({ "value": value })),
                Err(error) => kv_error(&error),
            }
        }

        "set" => {
            let input: KeyValueInput = match decode_input(input) {
                Ok(input) => input,
                Err(response) => return response,
            };

            match provider.set(&input.key, &input.value) {
                Ok(()) => CapabilityBridgeResponse::ok(json!({})),
                Err(error) => kv_error(&error),
            }
        }

        "delete" => {
            let input: KeyInput = match decode_input(input) {
                Ok(input) => input,
                Err(response) => return response,
            };

            match provider.delete(&input.key) {
                Ok(deleted) => CapabilityBridgeResponse::ok(json!({ "deleted": deleted })),
                Err(error) => kv_error(&error),
            }
        }

        operation => CapabilityBridgeResponse::error(
            "unsupported-operation",
            format!("key/value operation `{operation}` is not implemented"),
        ),
    }
}

fn dispatch_objects(
    provider: &dyn ObjectStoreProvider,
    operation: &str,
    input: &Value,
) -> CapabilityBridgeResponse {
    match operation {
        "get" => {
            let input: KeyInput = match decode_input(input) {
                Ok(input) => input,
                Err(response) => return response,
            };

            match provider.get(&input.key) {
                Ok(None) => CapabilityBridgeResponse::ok(json!({ "object": null })),
                Ok(Some(object)) => {
                    let (info, mut body) = object.into_parts();
                    let mut bytes = Vec::new();

                    if let Err(error) = body.read_to_end(&mut bytes) {
                        return CapabilityBridgeResponse::error(
                            "internal",
                            format!("could not read object body: {error}"),
                        );
                    }

                    CapabilityBridgeResponse::ok(json!({
                        "object": {
                            "key": info.key(),
                            "size": info.size(),
                            "body": bytes,
                        }
                    }))
                }
                Err(error) => object_error(&error),
            }
        }

        "put" => {
            let input: ObjectPutInput = match decode_input(input) {
                Ok(input) => input,
                Err(response) => return response,
            };
            let Ok(size) = u64::try_from(input.body.len()) else {
                return CapabilityBridgeResponse::error(
                    "invalid-body",
                    "object body is too large to represent",
                );
            };
            let mut body = Cursor::new(input.body);

            match provider.put(&input.key, size, &mut body) {
                Ok(info) => object_info_json(&info),
                Err(error) => object_error(&error),
            }
        }

        "head" => {
            let input: KeyInput = match decode_input(input) {
                Ok(input) => input,
                Err(response) => return response,
            };

            match provider.head(&input.key) {
                Ok(info) => CapabilityBridgeResponse::ok(json!({
                    "info": info.as_ref().map(object_info_value),
                })),
                Err(error) => object_error(&error),
            }
        }

        "delete" => {
            let input: KeyInput = match decode_input(input) {
                Ok(input) => input,
                Err(response) => return response,
            };

            match provider.delete(&input.key) {
                Ok(deleted) => CapabilityBridgeResponse::ok(json!({ "deleted": deleted })),
                Err(error) => object_error(&error),
            }
        }

        "list" => {
            let input: ObjectListInput = match decode_input(input) {
                Ok(input) => input,
                Err(response) => return response,
            };

            match provider.list(&input.prefix) {
                Ok(objects) => CapabilityBridgeResponse::ok(json!({
                    "objects": objects.iter().map(object_info_value).collect::<Vec<_>>(),
                })),
                Err(error) => object_error(&error),
            }
        }

        operation => CapabilityBridgeResponse::error(
            "unsupported-operation",
            format!("object-storage operation `{operation}` is not implemented"),
        ),
    }
}

fn decode_input<T>(input: &Value) -> Result<T, CapabilityBridgeResponse>
where
    T: for<'de> Deserialize<'de>,
{
    serde_json::from_value(input.clone()).map_err(|error| {
        CapabilityBridgeResponse::error(
            "invalid-request",
            format!("invalid operation input: {error}"),
        )
    })
}

fn object_info_json(info: &ObjectInfo) -> CapabilityBridgeResponse {
    CapabilityBridgeResponse::ok(json!({
        "info": object_info_value(info),
    }))
}

fn object_info_value(info: &ObjectInfo) -> Value {
    json!({
        "key": info.key(),
        "size": info.size(),
    })
}

fn kv_error(error: &KeyValueError) -> CapabilityBridgeResponse {
    let code = match error.code() {
        KeyValueErrorCode::Unavailable => "unavailable",
        KeyValueErrorCode::InvalidKey => "invalid-key",
        KeyValueErrorCode::Internal => "internal",
    };

    CapabilityBridgeResponse::error(code, error.message())
}

fn object_error(error: &ObjectStoreError) -> CapabilityBridgeResponse {
    let code = match error.code() {
        ObjectStoreErrorCode::Unavailable => "unavailable",
        ObjectStoreErrorCode::InvalidKey => "invalid-key",
        ObjectStoreErrorCode::InvalidBody => "invalid-body",
        ObjectStoreErrorCode::Internal => "internal",
    };

    CapabilityBridgeResponse::error(code, error.message())
}

#[cfg(test)]
mod tests {
    use presolve_capability::{key_value_contract, object_store_contract};
    use presolve_capability_bridge::CapabilityBridgeClient;
    use presolve_contract::parse_contract;
    use presolve_core::{EnvironmentId, ProviderId, WorkloadKind};
    use presolve_resolver::{
        EnvironmentInventory, EnvironmentResources, ProvidedCapability, ProviderDescriptor, resolve,
    };
    use serde_json::json;

    use crate::realize_local_providers;

    use super::*;

    fn capability_plan() -> DeploymentPlan {
        let contract = parse_contract(
            r#"
contract_version = "0.1"

[application]
name = "container-capability-bridge"
version = "1.0.0"

[[workloads]]
name = "api"
kind = "container"

[[capabilities]]
interface = "presolve:kv/store"
version = "^0.1"

[[capabilities]]
interface = "presolve:objects/store"
version = "^0.1"

[network]
outbound = "allow_all"
"#,
        )
        .expect("contract should parse");

        let provider_id = ProviderId::new();
        let environment = EnvironmentInventory::new(
            EnvironmentId::new(),
            EnvironmentResources::new(1_024, 1_000),
            vec![ProviderDescriptor::new(
                provider_id,
                "local-memory",
                vec![
                    ProvidedCapability::from_contract(
                        key_value_contract().expect("KV contract should be valid"),
                    ),
                    ProvidedCapability::from_contract(
                        object_store_contract().expect("object contract should be valid"),
                    ),
                ],
            )],
        )
        .with_supported_workloads(vec![WorkloadKind::Container]);

        resolve(&contract, &environment)
            .plan()
            .expect("capability-bearing container should resolve")
            .clone()
    }

    fn bridge_client(gateway: &ContainerCapabilityGateway) -> CapabilityBridgeClient {
        CapabilityBridgeClient::new(
            format!(
                "http://127.0.0.1:{}{}",
                gateway.transport().node_port(),
                CAPABILITY_BRIDGE_PATH
            ),
            gateway.transport().token().expose_secret(),
        )
        .expect("bridge client should configure")
    }

    #[test]
    fn bridge_routes_key_value_to_selected_provider() {
        let plan = capability_plan();
        let registry = realize_local_providers(&plan).expect("providers should realize");
        let gateway =
            start_container_capability_gateway(&plan, &registry).expect("gateway should start");
        let client = bridge_client(&gateway);

        client
            .call(
                KEY_VALUE_INTERFACE,
                "set",
                json!({ "key": "example", "value": "stored" }),
            )
            .expect("set should cross bridge");

        let output = client
            .call(KEY_VALUE_INTERFACE, "get", json!({ "key": "example" }))
            .expect("get should cross bridge");

        assert_eq!(output["value"], "stored");
    }

    #[test]
    fn bridge_routes_object_storage_to_selected_provider() {
        let plan = capability_plan();
        let registry = realize_local_providers(&plan).expect("providers should realize");
        let gateway =
            start_container_capability_gateway(&plan, &registry).expect("gateway should start");
        let client = bridge_client(&gateway);

        client
            .call(
                OBJECT_STORE_INTERFACE,
                "put",
                json!({ "key": "images/logo.bin", "body": [1, 2, 3, 4] }),
            )
            .expect("put should cross bridge");

        let output = client
            .call(
                OBJECT_STORE_INTERFACE,
                "get",
                json!({ "key": "images/logo.bin" }),
            )
            .expect("get should cross bridge");

        assert_eq!(output["object"]["key"], "images/logo.bin");
        assert_eq!(output["object"]["size"], 4);
        assert_eq!(output["object"]["body"], json!([1, 2, 3, 4]));
    }

    #[test]
    fn gateway_transport_does_not_expose_token_in_debug_output() {
        let plan = capability_plan();
        let registry = realize_local_providers(&plan).expect("providers should realize");
        let gateway =
            start_container_capability_gateway(&plan, &registry).expect("gateway should start");

        let debug = format!("{:?}", gateway.transport());

        assert!(debug.contains("[redacted]"));
        assert!(!debug.contains(gateway.transport().token().expose_secret()));
    }
}
