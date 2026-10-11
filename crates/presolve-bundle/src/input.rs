use presolve_core::WorkloadKind;

/// Component artifact supplied while constructing an application bundle.
///
/// The workload name refers to a workload declared by the Application
/// Contract. Artifact bytes are supplied by the build/package layer rather
/// than encoded as source filesystem paths in the contract.
#[derive(Debug, Clone, Copy)]
pub struct ComponentArtifactInput<'a> {
    workload: &'a str,
    bytes: &'a [u8],
}

impl<'a> ComponentArtifactInput<'a> {
    /// Creates a component artifact input for `workload`.
    #[must_use]
    pub const fn new(workload: &'a str, bytes: &'a [u8]) -> Self {
        Self { workload, bytes }
    }

    /// Returns the workload this artifact realizes.
    #[must_use]
    pub const fn workload(&self) -> &'a str {
        self.workload
    }

    /// Returns the component bytes.
    #[must_use]
    pub const fn bytes(&self) -> &'a [u8] {
        self.bytes
    }
}

/// Immutable OCI image descriptor supplied for a container workload.
///
/// Registry repositories, tags, credentials, and pull endpoints are
/// deliberately absent. They are realization/transport concerns rather than
/// application release identity.
#[derive(Debug, Clone, Copy)]
pub struct OciImageArtifactInput<'a> {
    workload: &'a str,
    media_type: &'a str,
    digest: &'a str,
    size: u64,
}

impl<'a> OciImageArtifactInput<'a> {
    /// Creates an OCI image descriptor input for `workload`.
    #[must_use]
    pub const fn new(workload: &'a str, media_type: &'a str, digest: &'a str, size: u64) -> Self {
        Self {
            workload,
            media_type,
            digest,
            size,
        }
    }

    /// Returns the workload this descriptor realizes.
    #[must_use]
    pub const fn workload(&self) -> &'a str {
        self.workload
    }

    /// Returns the OCI descriptor media type.
    #[must_use]
    pub const fn media_type(&self) -> &'a str {
        self.media_type
    }

    /// Returns the immutable OCI content digest.
    #[must_use]
    pub const fn digest(&self) -> &'a str {
        self.digest
    }

    /// Returns the descriptor target size in bytes.
    #[must_use]
    pub const fn size(&self) -> u64 {
        self.size
    }
}

/// Artifact identity supplied for one declared workload.
#[derive(Debug, Clone, Copy)]
pub enum WorkloadArtifactInput<'a> {
    /// Embedded WebAssembly Component bytes.
    Component(ComponentArtifactInput<'a>),

    /// External immutable OCI image content descriptor.
    OciImage(OciImageArtifactInput<'a>),
}

impl<'a> WorkloadArtifactInput<'a> {
    /// Returns the workload realized by this input.
    #[must_use]
    pub const fn workload(&self) -> &'a str {
        match self {
            Self::Component(component) => component.workload(),
            Self::OciImage(image) => image.workload(),
        }
    }

    /// Returns the workload execution class represented by this input.
    #[must_use]
    pub const fn kind(&self) -> WorkloadKind {
        match self {
            Self::Component(_) => WorkloadKind::Component,
            Self::OciImage(_) => WorkloadKind::Container,
        }
    }
}

impl<'a> From<ComponentArtifactInput<'a>> for WorkloadArtifactInput<'a> {
    fn from(value: ComponentArtifactInput<'a>) -> Self {
        Self::Component(value)
    }
}

impl<'a> From<OciImageArtifactInput<'a>> for WorkloadArtifactInput<'a> {
    fn from(value: OciImageArtifactInput<'a>) -> Self {
        Self::OciImage(value)
    }
}
