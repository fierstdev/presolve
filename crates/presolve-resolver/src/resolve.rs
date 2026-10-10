use presolve_contract::{ApplicationContract, CapabilityRequirement};

use crate::{
    CapabilityBinding, DeploymentPlan, EnvironmentInventory, ProvidedCapability,
    ProviderDescriptor, ResolutionProblem, ResolutionReport,
};

/// Resolves an application contract against an environment inventory.
#[must_use]
pub fn resolve(
    contract: &ApplicationContract,
    environment: &EnvironmentInventory,
) -> ResolutionReport {
    let mut problems = Vec::new();

    resolve_resources(contract, environment, &mut problems);

    let mut bindings = Vec::new();
    let mut unbound_optional_capabilities = Vec::new();

    for requirement in contract.capabilities() {
        match select_provider(requirement, environment) {
            Some((provider, capability)) => {
                bindings.push(CapabilityBinding::new(
                    requirement.interface().to_owned(),
                    *provider.id(),
                    provider.name().to_owned(),
                    capability.version().clone(),
                    capability.semantic_contract().cloned(),
                ));
            }
            None if requirement.optional() => {
                unbound_optional_capabilities.push(requirement.interface().to_owned());
            }
            None => {
                problems.push(ResolutionProblem::MissingCapability {
                    interface: requirement.interface().to_owned(),
                    version: requirement.version().clone(),
                });
            }
        }
    }

    if !problems.is_empty() {
        return ResolutionReport::rejected(problems);
    }

    ResolutionReport::resolved(DeploymentPlan::new(
        *environment.id(),
        bindings,
        unbound_optional_capabilities,
    ))
}

fn resolve_resources(
    contract: &ApplicationContract,
    environment: &EnvironmentInventory,
    problems: &mut Vec<ResolutionProblem>,
) {
    let requested = contract.resources();
    let available = environment.resources();

    if let Some(requested_mib) = requested.memory_mib()
        && requested_mib > available.memory_mib()
    {
        problems.push(ResolutionProblem::InsufficientMemory {
            requested_mib,
            available_mib: available.memory_mib(),
        });
    }

    if let Some(requested_millis) = requested.cpu_millis()
        && requested_millis > available.cpu_millis()
    {
        problems.push(ResolutionProblem::InsufficientCpu {
            requested_millis,
            available_millis: available.cpu_millis(),
        });
    }
}

fn select_provider<'a>(
    requirement: &CapabilityRequirement,
    environment: &'a EnvironmentInventory,
) -> Option<(&'a ProviderDescriptor, &'a ProvidedCapability)> {
    let mut candidates: Vec<_> = environment
        .providers()
        .iter()
        .flat_map(|provider| {
            provider
                .capabilities()
                .iter()
                .map(move |capability| (provider, capability))
        })
        .filter(|(_, capability)| {
            capability.interface() == requirement.interface()
                && requirement.version().matches(capability.version())
        })
        .collect();

    candidates.sort_by(|(provider_a, capability_a), (provider_b, capability_b)| {
        capability_b
            .version()
            .cmp(capability_a.version())
            .then_with(|| {
                provider_a
                    .id()
                    .to_string()
                    .cmp(&provider_b.id().to_string())
            })
    });

    candidates.into_iter().next()
}
