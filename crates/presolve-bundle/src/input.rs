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
