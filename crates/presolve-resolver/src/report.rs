use crate::{DeploymentPlan, ResolutionProblem};

/// Result of evaluating an application against an environment.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResolutionReport {
    plan: Option<DeploymentPlan>,
    problems: Vec<ResolutionProblem>,
}

impl ResolutionReport {
    pub(crate) fn resolved(plan: DeploymentPlan) -> Self {
        Self {
            plan: Some(plan),
            problems: Vec::new(),
        }
    }

    pub(crate) fn rejected(problems: Vec<ResolutionProblem>) -> Self {
        Self {
            plan: None,
            problems,
        }
    }

    /// Returns true when the environment can satisfy the application.
    #[must_use]
    pub const fn is_resolved(&self) -> bool {
        self.plan.is_some()
    }

    /// Returns the deployment plan when resolution succeeded.
    #[must_use]
    pub const fn plan(&self) -> Option<&DeploymentPlan> {
        self.plan.as_ref()
    }

    /// Returns problems preventing deployment.
    #[must_use]
    pub fn problems(&self) -> &[ResolutionProblem] {
        &self.problems
    }
}
