use std::{env, error::Error, io};

use presolve_executor_docker::DockerCliBackend;
use presolve_executor_sdk::{
    ContainerExecutor, ContainerImageDescriptor, ContainerImageResolver, ContainerLaunchRequest,
    ContainerNetworkMode, ContainerNetworkPolicy, ContainerResourceLimits,
};

fn main() -> Result<(), Box<dyn Error>> {
    let digest = env::args().nth(1).ok_or_else(|| {
        io::Error::new(
            io::ErrorKind::InvalidInput,
            "usage: smoke <sha256:image-digest>",
        )
    })?;

    let backend = DockerCliBackend::default();
    let image =
        ContainerImageDescriptor::new("application/vnd.oci.image.manifest.v1+json", digest, 0);
    let source = backend.resolve(&image)?;

    println!("resolved source: {}", source.reference());

    let request = ContainerLaunchRequest::new(
        "docker-smoke",
        image,
        source,
        ContainerResourceLimits::new(Some(128), Some(500)),
        ContainerNetworkPolicy::new(ContainerNetworkMode::Deny, Vec::new()),
    );

    let handle = backend.launch(&request)?;
    println!("launched container: {}", handle.id());

    backend.stop(&handle)?;
    println!("stopped container: {}", handle.id());

    Ok(())
}
