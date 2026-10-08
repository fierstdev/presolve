use semver::VersionReq;

/// Reason an application cannot be placed into an environment.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ResolutionProblem {
    /// No environment provider satisfies a required capability.
    MissingCapability {
        /// Required capability interface.
        interface: String,

        /// Required compatible version range.
        version: VersionReq,
    },

    /// The environment has less memory than the application requires.
    InsufficientMemory {
        /// Application memory requirement.
        requested_mib: u64,

        /// Environment memory capacity.
        available_mib: u64,
    },

    /// The environment has less CPU capacity than the application requires.
    InsufficientCpu {
        /// Application CPU requirement.
        requested_millis: u32,

        /// Environment CPU capacity.
        available_millis: u32,
    },
}

impl ResolutionProblem {
    /// Returns a stable `Presolve` resolver diagnostic code.
    #[must_use]
    pub const fn code(&self) -> &'static str {
        match self {
            Self::MissingCapability { .. } => "PS2001",
            Self::InsufficientMemory { .. } => "PS2002",
            Self::InsufficientCpu { .. } => "PS2003",
        }
    }
}
