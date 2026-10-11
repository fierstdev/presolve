use presolve_core::WorkloadKind;
use semver::VersionReq;

/// Reason an application cannot be placed into an environment.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ResolutionProblem {
    /// The environment cannot execute one declared application workload.
    UnsupportedWorkload {
        /// Stable workload name from the Application Contract.
        workload: String,

        /// Execution class required by the workload.
        kind: WorkloadKind,

        /// Execution classes supplied by the environment.
        supported_kinds: Vec<WorkloadKind>,
    },

    /// No environment provider satisfies a required capability.
    MissingCapability {
        /// Required capability interface.
        interface: String,

        /// Required compatible version range.
        version: VersionReq,
    },

    /// Providers match the interface and version but cannot prove all
    /// required semantic features.
    MissingCapabilityFeatures {
        /// Required capability interface.
        interface: String,

        /// Required compatible version range.
        version: VersionReq,

        /// Semantic features the application requires.
        required_features: Vec<String>,

        /// Semantic features advertised by otherwise-compatible providers.
        available_features: Vec<String>,
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
            Self::UnsupportedWorkload { .. } => "PS2005",
            Self::MissingCapability { .. } => "PS2001",
            Self::MissingCapabilityFeatures { .. } => "PS2004",
            Self::InsufficientMemory { .. } => "PS2002",
            Self::InsufficientCpu { .. } => "PS2003",
        }
    }
}
