use presolve_bundle::{BundleError, PresolveBundle};
use presolve_contract::OutboundNetworkMode;
use presolve_core::WorkloadKind;
use presolve_executor_sdk::{
    ContainerImageDescriptor, ContainerImageResolveError, ContainerImageResolver,
    ContainerLaunchRequest, ContainerNetworkMode, ContainerNetworkPolicy, ContainerResourceLimits,
};
use presolve_resolver::DeploymentPlan;
use thiserror::Error;

/// Failure while converting a resolved container workload into an execution
/// backend request.
#[derive(Debug, Error)]
pub enum ContainerMaterializationError {
    /// The application bundle failed its own integrity checks.
    #[error(transparent)]
    Bundle(#[from] BundleError),

    /// The `DeploymentPlan` and release bundle disagree about workload topology.
    #[error("resolved workload `{workload}` is absent from the application release")]
    MissingReleaseWorkload {
        /// Workload selected by the resolver.
        workload: String,
    },

    /// The release contains a workload not present in the `DeploymentPlan`.
    #[error("application release workload `{workload}` is absent from the DeploymentPlan")]
    UnexpectedReleaseWorkload {
        /// Workload present only in the release.
        workload: String,
    },

    /// The plan and release disagree about one workload's execution class.
    #[error(
        "workload `{workload}` is `{plan_kind}` in the DeploymentPlan \
         but `{release_kind}` in the application release"
    )]
    WorkloadKindMismatch {
        /// Mismatched workload.
        workload: String,

        /// Workload kind selected by resolution.
        plan_kind: WorkloadKind,

        /// Workload kind stored in the release.
        release_kind: WorkloadKind,
    },

    /// A container workload does not carry immutable OCI identity.
    #[error("container workload `{workload}` has no OCI image descriptor")]
    MissingOciImage {
        /// Container workload missing its descriptor.
        workload: String,
    },

    /// Container capability bindings cannot yet be transported into the
    /// execution backend.
    #[error(
        "container materialization cannot yet transport {bindings} capability binding(s); \
         container capability transport is introduced in Chunk 16.5"
    )]
    CapabilityTransportUnavailable {
        /// Number of resolved capability bindings that cannot yet be transported.
        bindings: usize,
    },

    /// Immutable OCI identity could not be mapped to an environment-local source.
    #[error("could not resolve OCI source for workload `{workload}`: {source}")]
    ImageResolution {
        /// Container workload being materialized.
        workload: String,

        /// Environment-specific source-resolution failure.
        #[source]
        source: ContainerImageResolveError,
    },
}

/// Materializes all resolved container workloads into backend-neutral launch
/// requests.
///
/// This function joins three different layers without collapsing them:
///
/// - [`DeploymentPlan`] determines which workload execution classes were
///   accepted by the environment.
/// - [`PresolveBundle`] supplies immutable OCI artifact identity.
/// - [`ContainerImageResolver`] supplies an environment-specific content source.
///
/// No container is launched here. The resulting [`ContainerLaunchRequest`]s are
/// handed to a concrete [`presolve_executor_sdk::ContainerExecutor`] later.
///
/// # Errors
///
/// Returns an error if the bundle is invalid, the plan and release disagree,
/// container capability transport would be required, or an immutable OCI image
/// cannot be located in the active environment.
pub fn materialize_container_workloads(
    plan: &DeploymentPlan,
    bundle: &PresolveBundle,
    image_resolver: &dyn ContainerImageResolver,
) -> Result<Vec<ContainerLaunchRequest>, ContainerMaterializationError> {
    bundle.verify()?;
    validate_workload_topology(plan, bundle)?;

    let container_count = plan
        .workloads()
        .iter()
        .filter(|workload| workload.kind() == WorkloadKind::Container)
        .count();

    if container_count == 0 {
        return Ok(Vec::new());
    }

    if !plan.bindings().is_empty() {
        return Err(
            ContainerMaterializationError::CapabilityTransportUnavailable {
                bindings: plan.bindings().len(),
            },
        );
    }

    let requirements = bundle.manifest().requirements();
    let resource_manifest = requirements.resources();
    let resources = ContainerResourceLimits::new(
        resource_manifest.memory_mib(),
        resource_manifest.cpu_millis(),
    );
    let network = ContainerNetworkPolicy::new(
        map_network_mode(requirements.network().outbound()),
        requirements.network().allow().to_vec(),
    );

    let mut requests = Vec::with_capacity(container_count);

    for workload in plan
        .workloads()
        .iter()
        .filter(|workload| workload.kind() == WorkloadKind::Container)
    {
        let release_workload = bundle
            .manifest()
            .workloads()
            .iter()
            .find(|candidate| candidate.name() == workload.name())
            .ok_or_else(|| ContainerMaterializationError::MissingReleaseWorkload {
                workload: workload.name().to_owned(),
            })?;

        let image = release_workload.oci_image().ok_or_else(|| {
            ContainerMaterializationError::MissingOciImage {
                workload: workload.name().to_owned(),
            }
        })?;

        let descriptor =
            ContainerImageDescriptor::new(image.media_type(), image.digest(), image.size());

        let source = image_resolver.resolve(&descriptor).map_err(|source| {
            ContainerMaterializationError::ImageResolution {
                workload: workload.name().to_owned(),
                source,
            }
        })?;

        requests.push(ContainerLaunchRequest::new(
            workload.name(),
            descriptor,
            source,
            resources,
            network.clone(),
        ));
    }

    Ok(requests)
}

fn validate_workload_topology(
    plan: &DeploymentPlan,
    bundle: &PresolveBundle,
) -> Result<(), ContainerMaterializationError> {
    for workload in plan.workloads() {
        let Some(release_workload) = bundle
            .manifest()
            .workloads()
            .iter()
            .find(|candidate| candidate.name() == workload.name())
        else {
            return Err(ContainerMaterializationError::MissingReleaseWorkload {
                workload: workload.name().to_owned(),
            });
        };

        if release_workload.kind() != workload.kind() {
            return Err(ContainerMaterializationError::WorkloadKindMismatch {
                workload: workload.name().to_owned(),
                plan_kind: workload.kind(),
                release_kind: release_workload.kind(),
            });
        }
    }

    for workload in bundle.manifest().workloads() {
        if !plan
            .workloads()
            .iter()
            .any(|candidate| candidate.name() == workload.name())
        {
            return Err(ContainerMaterializationError::UnexpectedReleaseWorkload {
                workload: workload.name().to_owned(),
            });
        }
    }

    Ok(())
}

const fn map_network_mode(mode: OutboundNetworkMode) -> ContainerNetworkMode {
    match mode {
        OutboundNetworkMode::Deny => ContainerNetworkMode::Deny,
        OutboundNetworkMode::AllowList => ContainerNetworkMode::AllowList,
        OutboundNetworkMode::AllowAll => ContainerNetworkMode::AllowAll,
    }
}

#[cfg(test)]
mod tests {
    use presolve_bundle::{OCI_IMAGE_MANIFEST_MEDIA_TYPE, OciImageArtifactInput, PresolveBundle};
    use presolve_contract::parse_contract;
    use presolve_core::EnvironmentId;
    use presolve_executor_sdk::{ContainerImageResolveError, ContainerImageSource};
    use presolve_resolver::{EnvironmentInventory, EnvironmentResources, resolve};

    use super::*;

    const IMAGE_DIGEST: &str =
        "sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";

    struct StaticImageResolver;

    impl ContainerImageResolver for StaticImageResolver {
        fn resolve(
            &self,
            image: &ContainerImageDescriptor,
        ) -> Result<ContainerImageSource, ContainerImageResolveError> {
            Ok(ContainerImageSource::new(format!(
                "registry.example/presolve-test@{}",
                image.digest()
            )))
        }
    }

    fn container_contract() -> presolve_contract::ApplicationContract {
        parse_contract(
            r#"
contract_version = "0.1"

[application]
name = "container-materialization"
version = "1.0.0"

[[workloads]]
name = "api"
kind = "container"

[resources]
memory_mib = 256
cpu_millis = 500

[network]
outbound = "allow_list"
allow = ["api.example:443"]
"#,
        )
        .expect("container contract should parse")
    }

    fn container_plan(
        contract: &presolve_contract::ApplicationContract,
    ) -> presolve_resolver::DeploymentPlan {
        let environment = EnvironmentInventory::new(
            EnvironmentId::new(),
            EnvironmentResources::new(1_024, 1_000),
            Vec::new(),
        )
        .with_supported_workloads(vec![WorkloadKind::Container]);

        resolve(contract, &environment)
            .plan()
            .expect("container workload should resolve")
            .clone()
    }

    #[test]
    fn materializes_container_descriptor_into_backend_neutral_request() {
        let contract = container_contract();
        let plan = container_plan(&contract);
        let bundle = PresolveBundle::from_oci_image(
            &contract,
            OciImageArtifactInput::new("api", OCI_IMAGE_MANIFEST_MEDIA_TYPE, IMAGE_DIGEST, 4_096),
        )
        .expect("container bundle should build");

        let requests = materialize_container_workloads(&plan, &bundle, &StaticImageResolver)
            .expect("container workload should materialize");

        assert_eq!(requests.len(), 1);

        let request = &requests[0];
        assert_eq!(request.workload(), "api");
        assert_eq!(request.image().digest(), IMAGE_DIGEST);
        assert_eq!(request.image().size(), 4_096);
        assert_eq!(
            request.source().reference(),
            format!("registry.example/presolve-test@{IMAGE_DIGEST}")
        );
        assert_eq!(request.resources().memory_mib(), Some(256));
        assert_eq!(request.resources().cpu_millis(), Some(500));
        assert_eq!(
            request.network().outbound(),
            ContainerNetworkMode::AllowList
        );
        assert_eq!(request.network().allow(), ["api.example:443"]);
    }

    #[test]
    fn component_only_plan_produces_no_container_requests() {
        let contract = parse_contract(
            r#"
contract_version = "0.1"

[application]
name = "component-only"
version = "1.0.0"
"#,
        )
        .expect("component contract should parse");

        let environment = EnvironmentInventory::new(
            EnvironmentId::new(),
            EnvironmentResources::new(1_024, 1_000),
            Vec::new(),
        );
        let plan = resolve(&contract, &environment)
            .plan()
            .expect("component workload should resolve")
            .clone();
        let bundle = PresolveBundle::from_component(&contract, b"component")
            .expect("component bundle should build");

        let requests = materialize_container_workloads(&plan, &bundle, &StaticImageResolver)
            .expect("component-only materialization should be a no-op");

        assert_eq!(requests, []);
    }
}
