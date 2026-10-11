use std::{
    env,
    process::Command,
    thread,
    time::{Duration, Instant},
};

use anyhow::{Context, Result, bail};
use presolve_bundle::{
    OCI_IMAGE_INDEX_MEDIA_TYPE, OCI_IMAGE_MANIFEST_MEDIA_TYPE, OciImageArtifactInput,
    PresolveBundle,
};
use presolve_capability::key_value_contract;
use presolve_contract::parse_contract;
use presolve_core::{EnvironmentId, ProviderId, WorkloadKind};
use presolve_executor_docker::DockerCliBackend;
use presolve_executor_sdk::ContainerExecutor;
use presolve_node::{
    materialize_container_workloads_with_capability_transport, realize_local_providers,
    start_container_capability_gateway,
};
use presolve_provider_sdk::{KEY_VALUE_INTERFACE, KeyValueProvider};
use presolve_resolver::{
    EnvironmentInventory, EnvironmentResources, ProvidedCapability, ProviderDescriptor, resolve,
};

const PROOF_KEY: &str = "presolve.chunk16.checkpoint";
const PROOF_VALUE: &str = "container-capability-ok";

const CONTRACT: &str = r#"
contract_version = "0.1"

[application]
name = "oci-container-checkpoint"
version = "1.0.0"

[[workloads]]
name = "api"
kind = "container"

[[capabilities]]
interface = "presolve:kv/store"
version = "^0.1"

[resources]
memory_mib = 128
cpu_millis = 500

[network]
outbound = "allow_all"
"#;

fn main() -> Result<()> {
    let arguments = env::args().skip(1).collect::<Vec<_>>();

    if arguments.len() != 3 {
        bail!("usage: oci_checkpoint <oci-digest> <oci-media-type> <descriptor-size>");
    }

    let digest = &arguments[0];
    let media_type = &arguments[1];
    let size = arguments[2]
        .parse::<u64>()
        .context("descriptor size must be an unsigned integer")?;

    if media_type != OCI_IMAGE_MANIFEST_MEDIA_TYPE && media_type != OCI_IMAGE_INDEX_MEDIA_TYPE {
        bail!("unsupported OCI descriptor media type `{media_type}`");
    }

    let contract = parse_contract(CONTRACT).context("checkpoint contract should parse")?;
    let provider_id = ProviderId::new();
    let environment = EnvironmentInventory::new(
        EnvironmentId::new(),
        EnvironmentResources::new(1_024, 1_000),
        vec![ProviderDescriptor::new(
            provider_id,
            "checkpoint-memory-kv",
            vec![ProvidedCapability::from_contract(
                key_value_contract().context("built-in KV contract should be valid")?,
            )],
        )],
    )
    .with_supported_workloads(vec![WorkloadKind::Container]);

    let outcome = resolve(&contract, &environment);
    let plan = outcome
        .plan()
        .context("container checkpoint application should resolve")?
        .clone();

    let bundle = PresolveBundle::from_oci_image(
        &contract,
        OciImageArtifactInput::new("api", media_type, digest, size),
    )
    .context("checkpoint .presolved release should build")?;

    let release_digest = bundle
        .release_digest()
        .context("checkpoint release should have an identity")?;
    let encoded = bundle
        .encode()
        .context("checkpoint release should encode")?;
    let decoded = PresolveBundle::decode(&encoded).context("checkpoint release should decode")?;

    if decoded
        .release_digest()
        .context("decoded release should have an identity")?
        != release_digest
    {
        bail!("checkpoint release identity changed across encode/decode");
    }

    let registry = realize_local_providers(&plan).context("checkpoint providers should realize")?;
    let selected_kv = plan
        .bindings()
        .iter()
        .find(|binding| binding.interface() == KEY_VALUE_INTERFACE)
        .context("checkpoint plan should contain a KV binding")?;
    let provider = registry
        .key_value(*selected_kv.provider_id())
        .context("selected KV provider implementation should be registered")?;

    let gateway = start_container_capability_gateway(&plan, &registry)
        .context("container capability gateway should start")?;
    let backend = DockerCliBackend::default();
    let requests = materialize_container_workloads_with_capability_transport(
        &plan,
        &decoded,
        &backend,
        gateway.transport(),
    )
    .context("container launch request should materialize")?;

    if requests.len() != 1 {
        bail!(
            "checkpoint expected exactly one container launch request, found {}",
            requests.len()
        );
    }

    println!("release digest: {release_digest}");
    println!("OCI artifact: {digest}");
    println!(
        "selected provider: {} ({})",
        selected_kv.provider_name(),
        selected_kv.provider_id()
    );

    let handle = backend
        .launch(&requests[0])
        .context("Docker checkpoint container should launch")?;

    println!("launched container: {}", handle.id());

    let proof_result = wait_for_proof(provider.as_ref());

    if let Err(error) = &proof_result {
        eprintln!("checkpoint container did not complete capability proof: {error}");
        eprintln!("container logs:\n{}", docker_logs(handle.id()));
    }

    let stop_result = backend.stop(&handle);

    proof_result?;
    stop_result.context("checkpoint container should be removed")?;

    println!("observed provider write: {PROOF_KEY}={PROOF_VALUE}");
    println!("stopped container: {}", handle.id());
    println!("Chunk 16 OCI/container checkpoint passed");

    Ok(())
}

fn wait_for_proof(provider: &dyn KeyValueProvider) -> Result<()> {
    let deadline = Instant::now() + Duration::from_secs(15);

    loop {
        match provider
            .get(PROOF_KEY)
            .context("reading checkpoint KV provider failed")?
        {
            Some(value) if value == PROOF_VALUE => return Ok(()),

            Some(value) => {
                bail!(
                    "checkpoint key contained unexpected value `{value}` instead of `{PROOF_VALUE}`"
                );
            }

            None if Instant::now() >= deadline => {
                bail!("timed out waiting for the container to write the checkpoint KV value");
            }

            None => thread::sleep(Duration::from_millis(100)),
        }
    }
}

fn docker_logs(container_id: &str) -> String {
    match Command::new("docker").args(["logs", container_id]).output() {
        Ok(output) => {
            let stdout = String::from_utf8_lossy(&output.stdout);
            let stderr = String::from_utf8_lossy(&output.stderr);

            if stderr.trim().is_empty() {
                stdout.into_owned()
            } else if stdout.trim().is_empty() {
                stderr.into_owned()
            } else {
                format!("{stdout}\n{stderr}")
            }
        }

        Err(error) => format!("could not read Docker logs: {error}"),
    }
}
