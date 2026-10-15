use std::{env, fs, sync::Arc};

use anyhow::{Context, Result, bail};
use presolve_bundle::{PresolveBundle, WorkloadKind, application_contract_for_resolution};
use presolve_environment::{inventory_from_specification, parse_environment};
use presolve_node::{assemble_runtime, realize_plan_providers};
use presolve_provider_queue_memory::{InMemoryWorkQueueAdapter, MEMORY_QUEUE_ADAPTER_ID};
use presolve_provider_realization::{
    EnvironmentSecretResolver, ProviderAdapterId, ProviderAdapterRegistry,
    ProviderRealizationCatalog, ProviderRealizationProfile,
};
use presolve_provider_sdk::WORK_QUEUE_INTERFACE;
use presolve_resolver::resolve;

const TAR_BLOCK_BYTES: usize = 512;

fn main() -> Result<()> {
    let mut args = env::args().skip(1);
    let release_path = args
        .next()
        .context("missing detached `.presolved` release path")?;
    let environment_path = args
        .next()
        .context("missing Environment Specification path")?;
    let input = args.next().context("missing application input")?;

    if args.next().is_some() {
        bail!(
            "usage: queue_release_realization_checkpoint <release.presolved> <presolve.env.toml> <input>"
        );
    }

    let release_bytes = fs::read(&release_path)
        .with_context(|| format!("could not read detached release `{release_path}`"))?;
    let bundle = PresolveBundle::decode(&release_bytes)
        .with_context(|| format!("invalid detached release `{release_path}`"))?;
    let release_digest = bundle
        .release_digest()
        .context("could not compute detached release identity")?;
    let contract = application_contract_for_resolution(bundle.manifest())
        .context("could not reconstruct resolver input from detached release")?;

    let environment_text = fs::read_to_string(&environment_path).with_context(|| {
        format!("could not read Environment Specification `{environment_path}`")
    })?;
    let specification = parse_environment(&environment_text)
        .with_context(|| format!("invalid Environment Specification `{environment_path}`"))?;
    let inventory = inventory_from_specification(&specification)
        .context("could not normalize Environment Specification for resolution")?;
    let resolution = resolve(&contract, &inventory);
    let plan = resolution
        .plan()
        .context("detached queue release is not compatible with the selected environment")?;

    let binding = plan
        .bindings()
        .iter()
        .find(|binding| binding.interface() == WORK_QUEUE_INTERFACE)
        .context("resolved plan does not contain the work-queue binding required by the demo")?;
    let provider_id = *binding.provider_id();

    let mut profiles = ProviderRealizationCatalog::new();
    profiles
        .register(ProviderRealizationProfile::new(
            provider_id,
            ProviderAdapterId::new(MEMORY_QUEUE_ADAPTER_ID)
                .context("memory queue adapter ID should be valid")?,
        ))
        .context("memory queue realization profile should register")?;

    let mut adapters = ProviderAdapterRegistry::new();
    adapters
        .register(
            ProviderAdapterId::new(MEMORY_QUEUE_ADAPTER_ID)
                .context("memory queue adapter ID should be valid")?,
            Arc::new(InMemoryWorkQueueAdapter),
        )
        .context("memory queue adapter should register")?;

    let registry = realize_plan_providers(plan, &profiles, &adapters, &EnvironmentSecretResolver)
        .context("could not realize plan-selected memory work queue")?;
    let component = component_bytes_from_release(&release_bytes, &bundle)?;
    let runtime = assemble_runtime(plan, &registry)
        .context("could not assemble runtime for detached queue release")?;
    let output = runtime
        .run_bytes(&component, &input)
        .context("detached Component work-queue execution failed")?;
    let expected = format!("queue:{input}");

    if output != expected {
        bail!("unexpected queue demo output: expected `{expected}`, found `{output}`");
    }

    println!("release digest: {release_digest}");
    println!(
        "environment: {} ({})",
        specification.environment().name(),
        specification.environment().id()
    );
    println!("provider: {} ({provider_id})", binding.provider_name());
    println!("output: {output}");

    Ok(())
}

fn component_bytes_from_release(release_bytes: &[u8], bundle: &PresolveBundle) -> Result<Vec<u8>> {
    let workload = match bundle.manifest().workloads() {
        [workload] => workload,
        workloads => bail!(
            "queue release realization checkpoint requires exactly one workload, found {}",
            workloads.len()
        ),
    };

    if workload.kind() != WorkloadKind::Component {
        bail!(
            "queue release realization checkpoint requires a Component workload, found `{}`",
            workload.kind()
        );
    }

    let artifact = match workload.artifacts() {
        [artifact] => artifact,
        artifacts => bail!(
            "Component workload `{}` must contain exactly one embedded artifact, found {}",
            workload.name(),
            artifacts.len()
        ),
    };

    read_verified_tar_entry(release_bytes, artifact.path())
}

fn read_verified_tar_entry(release_bytes: &[u8], target_path: &str) -> Result<Vec<u8>> {
    let mut offset = 0_usize;

    while offset
        .checked_add(TAR_BLOCK_BYTES)
        .is_some_and(|end| end <= release_bytes.len())
    {
        let header_end = offset + TAR_BLOCK_BYTES;
        let header = &release_bytes[offset..header_end];

        if header.iter().all(|byte| *byte == 0) {
            break;
        }

        let name = tar_text(&header[..100]).context("invalid verified tar entry name")?;
        let prefix = tar_text(&header[345..500]).context("invalid verified tar entry prefix")?;
        let path = if prefix.is_empty() {
            name.to_owned()
        } else {
            format!("{prefix}/{name}")
        };
        let size = tar_octal(&header[124..136]).context("invalid verified tar entry size")?;
        let data_start = header_end;
        let data_end = data_start
            .checked_add(size)
            .context("verified tar entry size overflow")?;

        if data_end > release_bytes.len() {
            bail!("verified tar entry `{path}` extends past release bytes");
        }

        if path == target_path {
            return Ok(release_bytes[data_start..data_end].to_vec());
        }

        let padded_size = size
            .checked_add(TAR_BLOCK_BYTES - 1)
            .context("verified tar padding overflow")?
            / TAR_BLOCK_BYTES
            * TAR_BLOCK_BYTES;
        offset = data_start
            .checked_add(padded_size)
            .context("verified tar offset overflow")?;
    }

    bail!("verified detached release does not expose embedded Component `{target_path}`")
}

fn tar_text(field: &[u8]) -> Result<&str> {
    let end = field
        .iter()
        .position(|byte| *byte == 0)
        .unwrap_or(field.len());
    std::str::from_utf8(&field[..end]).context("tar field is not UTF-8")
}

fn tar_octal(field: &[u8]) -> Result<usize> {
    let text = tar_text(field)?.trim();
    if text.is_empty() {
        return Ok(0);
    }

    usize::from_str_radix(text, 8).context("tar field is not an octal integer")
}
