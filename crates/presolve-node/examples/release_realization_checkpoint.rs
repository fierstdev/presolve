use std::{
    env, fs,
    sync::Arc,
    thread,
    time::{Duration, Instant},
};

use anyhow::{Context, Result, bail};
use presolve_bundle::{PresolveBundle, WorkloadKind, application_contract_for_resolution};
use presolve_environment::{inventory_from_specification, parse_environment};
use presolve_node::{assemble_runtime, realize_local_providers, realize_plan_providers};
use presolve_provider_kv_cloudflare::{
    CLOUDFLARE_ACCOUNT_ID_SETTING, CLOUDFLARE_API_TOKEN_SECRET, CLOUDFLARE_KV_ADAPTER_ID,
    CLOUDFLARE_NAMESPACE_ID_SETTING, CloudflareKvAdapter,
};
use presolve_provider_realization::{
    EnvironmentSecretResolver, ProviderAdapterId, ProviderAdapterRegistry,
    ProviderRealizationCatalog, ProviderRealizationProfile, SecretReference,
};
use presolve_provider_sdk::KEY_VALUE_INTERFACE;
use presolve_resolver::{DeploymentPlan, resolve};

const ACCOUNT_ID_ENV: &str = "PRESOLVE_CLOUDFLARE_ACCOUNT_ID";
const NAMESPACE_ID_ENV: &str = "PRESOLVE_CLOUDFLARE_KV_NAMESPACE_ID";
const API_TOKEN_ENV: &str = "PRESOLVE_CLOUDFLARE_API_TOKEN";
const CLOUD_VISIBILITY_TIMEOUT: Duration = Duration::from_secs(75);
const CLOUD_POLL_INTERVAL: Duration = Duration::from_secs(1);
const TAR_BLOCK_BYTES: usize = 512;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum RealizationMode {
    Local,
    Cloudflare,
}

impl RealizationMode {
    fn parse(value: &str) -> Result<Self> {
        match value {
            "local" => Ok(Self::Local),
            "cloudflare" => Ok(Self::Cloudflare),
            _ => bail!("realization mode must be `local` or `cloudflare`, got `{value}`"),
        }
    }

    const fn label(self) -> &'static str {
        match self {
            Self::Local => "local",
            Self::Cloudflare => "cloudflare",
        }
    }
}

fn main() -> Result<()> {
    let mut args = env::args().skip(1);
    let mode = RealizationMode::parse(
        &args
            .next()
            .context("missing realization mode (`local` or `cloudflare`)")?,
    )?;
    let release_path = args
        .next()
        .context("missing detached `.presolved` release path")?;
    let environment_path = args
        .next()
        .context("missing Environment Specification path")?;
    let input = args.next().context("missing application input")?;

    if args.next().is_some() {
        bail!(
            "usage: release_realization_checkpoint <local|cloudflare> <release.presolved> <presolve.env.toml> <input>"
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
        .context("detached release is not compatible with the selected environment")?;

    let binding = plan
        .bindings()
        .iter()
        .find(|binding| binding.interface() == KEY_VALUE_INTERFACE)
        .context("resolved plan does not contain the KV binding required by the demo")?;
    let provider_id = *binding.provider_id();

    let registry = match mode {
        RealizationMode::Local => realize_local_providers(plan)
            .context("could not realize selected provider with local implementation")?,
        RealizationMode::Cloudflare => realize_cloudflare_provider(plan, provider_id)?,
    };

    let component = component_bytes_from_release(&release_bytes, &bundle)?;
    let runtime = assemble_runtime(plan, &registry)
        .context("could not assemble runtime for detached release")?;
    let output = execute_kv_proof(mode, &runtime, &component, &input)?;

    println!("release digest: {release_digest}");
    println!(
        "environment: {} ({})",
        specification.environment().name(),
        specification.environment().id()
    );
    println!("provider: {} ({provider_id})", binding.provider_name());
    println!("mode: {}", mode.label());
    println!("output: {output}");

    Ok(())
}

fn realize_cloudflare_provider(
    plan: &DeploymentPlan,
    provider_id: presolve_core::ProviderId,
) -> Result<presolve_node::ProviderRegistry> {
    let account_id = required_env(ACCOUNT_ID_ENV)?;
    let namespace_id = required_env(NAMESPACE_ID_ENV)?;

    if env::var_os(API_TOKEN_ENV).is_none() {
        bail!("required environment variable `{API_TOKEN_ENV}` is not set");
    }

    let profile = ProviderRealizationProfile::new(
        provider_id,
        ProviderAdapterId::new(CLOUDFLARE_KV_ADAPTER_ID)
            .context("Cloudflare adapter ID should be valid")?,
    )
    .with_setting(CLOUDFLARE_ACCOUNT_ID_SETTING, account_id)
    .context("Cloudflare account realization setting should be valid")?
    .with_setting(CLOUDFLARE_NAMESPACE_ID_SETTING, namespace_id)
    .context("Cloudflare namespace realization setting should be valid")?
    .with_secret(
        CLOUDFLARE_API_TOKEN_SECRET,
        SecretReference::environment_variable(API_TOKEN_ENV)
            .context("Cloudflare token secret reference should be valid")?,
    )
    .context("Cloudflare token secret binding should be valid")?;

    let mut profiles = ProviderRealizationCatalog::new();
    profiles
        .register(profile)
        .context("Cloudflare realization profile should register")?;

    let mut adapters = ProviderAdapterRegistry::new();
    adapters
        .register(
            ProviderAdapterId::new(CLOUDFLARE_KV_ADAPTER_ID)
                .context("Cloudflare adapter ID should be valid")?,
            Arc::new(CloudflareKvAdapter),
        )
        .context("Cloudflare adapter should register")?;

    realize_plan_providers(plan, &profiles, &adapters, &EnvironmentSecretResolver)
        .context("could not realize plan-selected Cloudflare KV provider")
}

fn component_bytes_from_release(release_bytes: &[u8], bundle: &PresolveBundle) -> Result<Vec<u8>> {
    let workload = match bundle.manifest().workloads() {
        [workload] => workload,
        workloads => bail!(
            "release realization checkpoint requires exactly one workload, found {}",
            workloads.len()
        ),
    };

    if workload.kind() != WorkloadKind::Component {
        bail!(
            "release realization checkpoint requires a Component workload, found `{}`",
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

    // `PresolveBundle::decode` above has already validated the archive topology,
    // artifact size, and artifact digest. This small reader only selects the
    // manifest-named bytes from that same verified in-memory archive so the proof
    // does not need an extracted `.wasm` sidecar or another crate dependency.
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
    let end = field.iter().position(|byte| *byte == 0).unwrap_or(field.len());
    std::str::from_utf8(&field[..end]).context("tar field is not UTF-8")
}

fn tar_octal(field: &[u8]) -> Result<usize> {
    let text = tar_text(field)?.trim();
    if text.is_empty() {
        return Ok(0);
    }

    usize::from_str_radix(text, 8).context("tar field is not an octal integer")
}

fn execute_kv_proof(
    mode: RealizationMode,
    runtime: &presolve_runtime::Runtime,
    component: &[u8],
    input: &str,
) -> Result<String> {
    let key = format!("presolve/chunk21/{input}");
    let set_command = format!("set:{key}:{input}");
    let set_output = runtime
        .run_bytes(component, &set_command)
        .context("detached Component KV write failed")?;

    if set_output != "stored" {
        bail!("detached Component KV write returned `{set_output}` instead of `stored`");
    }

    let expected = format!("value:{input}");
    let get_command = format!("get:{key}");
    let deadline = Instant::now() + CLOUD_VISIBILITY_TIMEOUT;

    let observed = loop {
        let output = runtime
            .run_bytes(component, &get_command)
            .context("detached Component KV read failed")?;

        if output == expected {
            break output;
        }

        if mode == RealizationMode::Local {
            bail!("local detached-release read returned `{output}`, expected `{expected}`");
        }

        if Instant::now() >= deadline {
            bail!(
                "timed out waiting for Cloudflare KV visibility through the detached release; last output `{output}`"
            );
        }

        thread::sleep(CLOUD_POLL_INTERVAL);
    };

    let delete_command = format!("delete:{key}");
    let delete_output = runtime
        .run_bytes(component, &delete_command)
        .context("detached Component KV cleanup failed")?;

    if delete_output != "deleted" {
        bail!("detached Component KV cleanup returned `{delete_output}` instead of `deleted`");
    }

    Ok(observed)
}

fn required_env(name: &str) -> Result<String> {
    env::var(name).with_context(|| format!("required environment variable `{name}` is not set"))
}
