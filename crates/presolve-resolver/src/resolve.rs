use std::collections::BTreeSet;

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

    resolve_workloads(contract, environment, &mut problems);
    resolve_resources(contract, environment, &mut problems);

    let mut bindings = Vec::new();
    let mut unbound_optional_capabilities = Vec::new();

    for requirement in contract.capabilities() {
        match select_provider(requirement, environment) {
            ProviderSelection::Selected {
                provider,
                capability,
                negotiated_features,
            } => {
                bindings.push(CapabilityBinding::new(
                    requirement.interface().to_owned(),
                    *provider.id(),
                    provider.name().to_owned(),
                    capability.version().clone(),
                    capability.semantic_contract().cloned(),
                    negotiated_features,
                ));
            }
            ProviderSelection::MissingCapability
            | ProviderSelection::MissingRequiredFeatures { .. }
                if requirement.optional() =>
            {
                unbound_optional_capabilities.push(requirement.interface().to_owned());
            }
            ProviderSelection::MissingCapability => {
                problems.push(ResolutionProblem::MissingCapability {
                    interface: requirement.interface().to_owned(),
                    version: requirement.version().clone(),
                });
            }
            ProviderSelection::MissingRequiredFeatures { available_features } => {
                problems.push(ResolutionProblem::MissingCapabilityFeatures {
                    interface: requirement.interface().to_owned(),
                    version: requirement.version().clone(),
                    required_features: requirement.required_features().to_vec(),
                    available_features,
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

fn resolve_workloads(
    contract: &ApplicationContract,
    environment: &EnvironmentInventory,
    problems: &mut Vec<ResolutionProblem>,
) {
    for workload in contract.workloads() {
        if environment.supported_workloads().contains(&workload.kind()) {
            continue;
        }

        problems.push(ResolutionProblem::UnsupportedWorkload {
            workload: workload.name().to_owned(),
            kind: workload.kind(),
            supported_kinds: environment.supported_workloads().to_vec(),
        });
    }
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

enum ProviderSelection<'a> {
    Selected {
        provider: &'a ProviderDescriptor,
        capability: &'a ProvidedCapability,
        negotiated_features: Vec<String>,
    },
    MissingCapability,
    MissingRequiredFeatures {
        available_features: Vec<String>,
    },
}

struct Candidate<'a> {
    provider: &'a ProviderDescriptor,
    capability: &'a ProvidedCapability,
    preferred_matches: usize,
}

fn select_provider<'a>(
    requirement: &CapabilityRequirement,
    environment: &'a EnvironmentInventory,
) -> ProviderSelection<'a> {
    let version_candidates = environment
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
        .collect::<Vec<_>>();

    if version_candidates.is_empty() {
        return ProviderSelection::MissingCapability;
    }

    let mut available_features = BTreeSet::new();
    let mut candidates = Vec::new();

    for (provider, capability) in version_candidates {
        let supported = supported_features(capability);

        available_features.extend(supported.iter().cloned());

        if !requirement
            .required_features()
            .iter()
            .all(|required| supported.iter().any(|feature| feature == required))
        {
            continue;
        }

        let preferred_matches = requirement
            .preferred_features()
            .iter()
            .filter(|preferred| supported.iter().any(|feature| feature == *preferred))
            .count();

        candidates.push(Candidate {
            provider,
            capability,
            preferred_matches,
        });
    }

    if candidates.is_empty() {
        return ProviderSelection::MissingRequiredFeatures {
            available_features: available_features.into_iter().collect(),
        };
    }

    candidates.sort_by(|candidate_a, candidate_b| {
        candidate_b
            .preferred_matches
            .cmp(&candidate_a.preferred_matches)
            .then_with(|| {
                candidate_b
                    .capability
                    .version()
                    .cmp(candidate_a.capability.version())
            })
            .then_with(|| {
                candidate_a
                    .provider
                    .id()
                    .to_string()
                    .cmp(&candidate_b.provider.id().to_string())
            })
    });

    let selected = candidates
        .into_iter()
        .next()
        .expect("non-empty candidate list should select a provider");

    let supported = supported_features(selected.capability);
    let mut negotiated_features = requirement.required_features().to_vec();

    negotiated_features.extend(
        requirement
            .preferred_features()
            .iter()
            .filter(|preferred| supported.iter().any(|feature| feature == *preferred))
            .cloned(),
    );

    negotiated_features.sort();
    negotiated_features.dedup();

    ProviderSelection::Selected {
        provider: selected.provider,
        capability: selected.capability,
        negotiated_features,
    }
}

fn supported_features(capability: &ProvidedCapability) -> &[String] {
    capability.supported_features()
}
