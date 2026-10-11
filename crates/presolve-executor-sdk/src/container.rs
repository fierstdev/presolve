use thiserror::Error;

/// Immutable OCI image identity required by a container execution backend.
///
/// This is deliberately separate from a registry/repository/tag. Two different
/// sources may legitimately provide the same immutable descriptor.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ContainerImageDescriptor {
    media_type: String,
    digest: String,
    size: u64,
}

impl ContainerImageDescriptor {
    /// Creates an immutable container image descriptor.
    #[must_use]
    pub fn new(media_type: impl Into<String>, digest: impl Into<String>, size: u64) -> Self {
        Self {
            media_type: media_type.into(),
            digest: digest.into(),
            size,
        }
    }

    /// Returns the OCI descriptor media type.
    #[must_use]
    pub fn media_type(&self) -> &str {
        &self.media_type
    }

    /// Returns the immutable OCI content digest.
    #[must_use]
    pub fn digest(&self) -> &str {
        &self.digest
    }

    /// Returns the descriptor target size in bytes.
    #[must_use]
    pub const fn size(&self) -> u64 {
        self.size
    }
}

/// Environment-specific location from which OCI content can be obtained.
///
/// `reference` is opaque to Presolve core logic. A Docker adapter might receive
/// a registry reference, while another backend may use a platform-specific
/// artifact locator. Credentials are intentionally not stored here.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ContainerImageSource {
    reference: String,
}

impl ContainerImageSource {
    /// Creates an environment-specific image source.
    #[must_use]
    pub fn new(reference: impl Into<String>) -> Self {
        Self {
            reference: reference.into(),
        }
    }

    /// Returns the backend-specific image source reference.
    #[must_use]
    pub fn reference(&self) -> &str {
        &self.reference
    }
}

/// Resource limits requested for a container workload.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct ContainerResourceLimits {
    memory_mib: Option<u64>,
    cpu_millis: Option<u32>,
}

impl ContainerResourceLimits {
    /// Creates container resource limits.
    #[must_use]
    pub const fn new(memory_mib: Option<u64>, cpu_millis: Option<u32>) -> Self {
        Self {
            memory_mib,
            cpu_millis,
        }
    }

    /// Returns the requested memory limit in mebibytes.
    #[must_use]
    pub const fn memory_mib(self) -> Option<u64> {
        self.memory_mib
    }

    /// Returns the requested CPU limit in millicores.
    #[must_use]
    pub const fn cpu_millis(self) -> Option<u32> {
        self.cpu_millis
    }
}

/// Outbound network policy supplied to a container execution backend.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ContainerNetworkMode {
    /// No arbitrary outbound network access.
    Deny,

    /// Outbound access is restricted to explicit targets.
    AllowList,

    /// Arbitrary outbound access is allowed.
    AllowAll,
}

/// Network policy supplied when launching a container workload.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ContainerNetworkPolicy {
    outbound: ContainerNetworkMode,
    allow: Vec<String>,
}

impl ContainerNetworkPolicy {
    /// Creates a container network policy.
    #[must_use]
    pub fn new(outbound: ContainerNetworkMode, mut allow: Vec<String>) -> Self {
        allow.sort();
        allow.dedup();

        Self { outbound, allow }
    }

    /// Returns the outbound network mode.
    #[must_use]
    pub const fn outbound(&self) -> ContainerNetworkMode {
        self.outbound
    }

    /// Returns explicitly allowed outbound targets.
    #[must_use]
    pub fn allow(&self) -> &[String] {
        &self.allow
    }
}

/// One capability made available through a container transport.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ContainerCapabilityBinding {
    interface: String,
    version: String,
}

impl ContainerCapabilityBinding {
    /// Creates a transported capability binding.
    #[must_use]
    pub fn new(interface: impl Into<String>, version: impl Into<String>) -> Self {
        Self {
            interface: interface.into(),
            version: version.into(),
        }
    }

    /// Returns the semantic capability interface.
    #[must_use]
    pub fn interface(&self) -> &str {
        &self.interface
    }

    /// Returns the exact capability version selected by resolution.
    #[must_use]
    pub fn version(&self) -> &str {
        &self.version
    }
}

/// Per-deployment bearer token used to authorize capability bridge calls.
///
/// Debug formatting is intentionally redacted so launch-request diagnostics do
/// not expose the credential accidentally.
#[derive(Clone, PartialEq, Eq)]
pub struct ContainerCapabilityToken(String);

impl ContainerCapabilityToken {
    /// Creates a capability bridge bearer token.
    #[must_use]
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }

    /// Exposes the bearer token to an execution adapter that must inject it.
    #[must_use]
    pub fn expose_secret(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Debug for ContainerCapabilityToken {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str("ContainerCapabilityToken([redacted])")
    }
}

/// Backend-neutral description of a node-hosted capability bridge.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ContainerCapabilityTransport {
    protocol: String,
    node_port: u16,
    token: ContainerCapabilityToken,
    bindings: Vec<ContainerCapabilityBinding>,
}

impl ContainerCapabilityTransport {
    /// Creates a capability transport description.
    #[must_use]
    pub fn new(
        protocol: impl Into<String>,
        node_port: u16,
        token: ContainerCapabilityToken,
        mut bindings: Vec<ContainerCapabilityBinding>,
    ) -> Self {
        bindings.sort_by(|left, right| {
            left.interface()
                .cmp(right.interface())
                .then_with(|| left.version().cmp(right.version()))
        });
        bindings.dedup();

        Self {
            protocol: protocol.into(),
            node_port,
            token,
            bindings,
        }
    }

    /// Returns the transport protocol identifier.
    #[must_use]
    pub fn protocol(&self) -> &str {
        &self.protocol
    }

    /// Returns the node-side TCP port hosting the bridge.
    #[must_use]
    pub const fn node_port(&self) -> u16 {
        self.node_port
    }

    /// Returns the per-deployment bearer token.
    #[must_use]
    pub const fn token(&self) -> &ContainerCapabilityToken {
        &self.token
    }

    /// Returns capabilities exposed through this bridge.
    #[must_use]
    pub fn bindings(&self) -> &[ContainerCapabilityBinding] {
        &self.bindings
    }
}

/// Fully materialized request supplied to a container execution backend.
///
/// The immutable descriptor and environment-specific source are both present so
/// an executor can verify that the located content still matches application
/// release identity before launch.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ContainerLaunchRequest {
    workload: String,
    image: ContainerImageDescriptor,
    source: ContainerImageSource,
    resources: ContainerResourceLimits,
    network: ContainerNetworkPolicy,
    capability_transport: Option<ContainerCapabilityTransport>,
}

impl ContainerLaunchRequest {
    /// Creates a container launch request.
    #[must_use]
    pub fn new(
        workload: impl Into<String>,
        image: ContainerImageDescriptor,
        source: ContainerImageSource,
        resources: ContainerResourceLimits,
        network: ContainerNetworkPolicy,
    ) -> Self {
        Self {
            workload: workload.into(),
            image,
            source,
            resources,
            network,
            capability_transport: None,
        }
    }

    /// Returns the workload name.
    #[must_use]
    pub fn workload(&self) -> &str {
        &self.workload
    }

    /// Returns the immutable image descriptor.
    #[must_use]
    pub const fn image(&self) -> &ContainerImageDescriptor {
        &self.image
    }

    /// Returns the environment-specific image source.
    #[must_use]
    pub const fn source(&self) -> &ContainerImageSource {
        &self.source
    }

    /// Returns requested resource limits.
    #[must_use]
    pub const fn resources(&self) -> ContainerResourceLimits {
        self.resources
    }

    /// Returns requested network policy.
    #[must_use]
    pub const fn network(&self) -> &ContainerNetworkPolicy {
        &self.network
    }

    /// Attaches a node-hosted capability transport to this launch request.
    #[must_use]
    pub fn with_capability_transport(mut self, transport: ContainerCapabilityTransport) -> Self {
        self.capability_transport = Some(transport);
        self
    }

    /// Returns the capability transport, when this workload has bound capabilities.
    #[must_use]
    pub const fn capability_transport(&self) -> Option<&ContainerCapabilityTransport> {
        self.capability_transport.as_ref()
    }
}

/// Opaque backend instance identity returned after a successful launch.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ContainerHandle {
    id: String,
}

impl ContainerHandle {
    /// Creates a backend-scoped container handle.
    #[must_use]
    pub fn new(id: impl Into<String>) -> Self {
        Self { id: id.into() }
    }

    /// Returns the backend-scoped instance identifier.
    #[must_use]
    pub fn id(&self) -> &str {
        &self.id
    }
}

/// Stable categories for image-source resolution failures.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ContainerImageResolveErrorCode {
    /// The image-source backend is not currently available.
    Unavailable,

    /// No source for the immutable image descriptor is available.
    NotFound,

    /// The configured source cannot be accessed with current authorization.
    Unauthorized,

    /// The resolved source is invalid for the active execution environment.
    InvalidSource,

    /// Source resolution failed internally.
    Internal,
}

/// Failure while resolving immutable OCI identity to environment-local source.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
#[error("{message}")]
pub struct ContainerImageResolveError {
    code: ContainerImageResolveErrorCode,
    message: String,
}

impl ContainerImageResolveError {
    /// Creates an unavailable-image-source-backend error.
    #[must_use]
    pub fn unavailable(message: impl Into<String>) -> Self {
        Self {
            code: ContainerImageResolveErrorCode::Unavailable,
            message: message.into(),
        }
    }

    /// Creates a missing-image-source error.
    #[must_use]
    pub fn not_found(message: impl Into<String>) -> Self {
        Self {
            code: ContainerImageResolveErrorCode::NotFound,
            message: message.into(),
        }
    }

    /// Creates an authorization failure.
    #[must_use]
    pub fn unauthorized(message: impl Into<String>) -> Self {
        Self {
            code: ContainerImageResolveErrorCode::Unauthorized,
            message: message.into(),
        }
    }

    /// Creates an invalid-source failure.
    #[must_use]
    pub fn invalid_source(message: impl Into<String>) -> Self {
        Self {
            code: ContainerImageResolveErrorCode::InvalidSource,
            message: message.into(),
        }
    }

    /// Creates an internal source-resolution failure.
    #[must_use]
    pub fn internal(message: impl Into<String>) -> Self {
        Self {
            code: ContainerImageResolveErrorCode::Internal,
            message: message.into(),
        }
    }

    /// Returns the stable error category.
    #[must_use]
    pub const fn code(&self) -> ContainerImageResolveErrorCode {
        self.code
    }

    /// Returns the human-readable error message.
    #[must_use]
    pub fn message(&self) -> &str {
        &self.message
    }
}

/// Resolves immutable OCI identity to a source usable in one environment.
///
/// Implementations may use local image stores, registry mirrors, artifact
/// caches, cloud registries, or other environment-specific mechanisms.
pub trait ContainerImageResolver: Send + Sync {
    /// Locates content matching `image`.
    ///
    /// # Errors
    ///
    /// Returns [`ContainerImageResolveError`] when no acceptable source can be
    /// produced.
    fn resolve(
        &self,
        image: &ContainerImageDescriptor,
    ) -> Result<ContainerImageSource, ContainerImageResolveError>;
}

/// Stable categories for container execution failures.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ContainerExecutorErrorCode {
    /// The execution backend is not currently available.
    Unavailable,

    /// Located image content does not match the immutable descriptor.
    ImageMismatch,

    /// The launch request cannot be represented by this backend.
    InvalidRequest,

    /// The execution backend failed internally.
    Internal,
}

/// Failure returned by a container execution backend.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
#[error("{message}")]
pub struct ContainerExecutorError {
    code: ContainerExecutorErrorCode,
    message: String,
}

impl ContainerExecutorError {
    /// Creates an unavailable-backend error.
    #[must_use]
    pub fn unavailable(message: impl Into<String>) -> Self {
        Self {
            code: ContainerExecutorErrorCode::Unavailable,
            message: message.into(),
        }
    }

    /// Creates an immutable-image mismatch error.
    #[must_use]
    pub fn image_mismatch(message: impl Into<String>) -> Self {
        Self {
            code: ContainerExecutorErrorCode::ImageMismatch,
            message: message.into(),
        }
    }

    /// Creates an invalid-request error.
    #[must_use]
    pub fn invalid_request(message: impl Into<String>) -> Self {
        Self {
            code: ContainerExecutorErrorCode::InvalidRequest,
            message: message.into(),
        }
    }

    /// Creates an internal execution error.
    #[must_use]
    pub fn internal(message: impl Into<String>) -> Self {
        Self {
            code: ContainerExecutorErrorCode::Internal,
            message: message.into(),
        }
    }

    /// Returns the stable error category.
    #[must_use]
    pub const fn code(&self) -> ContainerExecutorErrorCode {
        self.code
    }

    /// Returns the human-readable error message.
    #[must_use]
    pub fn message(&self) -> &str {
        &self.message
    }
}

/// Backend capable of launching and stopping container workloads.
///
/// Implementations must ensure that content launched from
/// [`ContainerLaunchRequest::source`] matches the immutable descriptor in
/// [`ContainerLaunchRequest::image`] before executing it.
pub trait ContainerExecutor: Send + Sync {
    /// Launches one container workload.
    ///
    /// # Errors
    ///
    /// Returns [`ContainerExecutorError`] when the backend cannot safely realize
    /// the request.
    fn launch(
        &self,
        request: &ContainerLaunchRequest,
    ) -> Result<ContainerHandle, ContainerExecutorError>;

    /// Stops one previously launched container workload.
    ///
    /// # Errors
    ///
    /// Returns [`ContainerExecutorError`] when the backend cannot stop the
    /// referenced instance.
    fn stop(&self, handle: &ContainerHandle) -> Result<(), ContainerExecutorError>;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn network_allow_list_is_canonicalized() {
        let policy = ContainerNetworkPolicy::new(
            ContainerNetworkMode::AllowList,
            vec![
                "b.example:443".to_owned(),
                "a.example:443".to_owned(),
                "a.example:443".to_owned(),
            ],
        );

        assert_eq!(
            policy.allow(),
            ["a.example:443".to_owned(), "b.example:443".to_owned()]
        );
    }

    #[test]
    fn errors_preserve_stable_categories() {
        let resolve = ContainerImageResolveError::not_found("missing");
        let execute = ContainerExecutorError::image_mismatch("wrong digest");

        assert_eq!(resolve.code(), ContainerImageResolveErrorCode::NotFound);
        assert_eq!(execute.code(), ContainerExecutorErrorCode::ImageMismatch);
    }
}
