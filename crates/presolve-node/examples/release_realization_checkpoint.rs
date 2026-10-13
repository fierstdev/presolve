use std::{
    env, fs,
    io::{Cursor, Read},
    path::Path,
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
use tar::Archive;

const ACCOUNT_ID_ENV: &str = "PRESOLVE_CLOUDFLARE_ACCOUNT_ID";
const NAMESPACE_ID_ENV: &str = "PRESOLVE_CLOUDFLARE_KV_NAMESPACE_ID";
const API_TOKEN_ENV: &str = "PRESOLVE_CLOUDFLARE_API_TOKEN";
const CLOUD_RETRY_TIMEOUT: Duration = Duration::from_secs(75);
const CLOUD_RETRY_INTERVAL: Duration = Duration::from_secs(1);

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
    let specification = parse_environment(&environment_text).with_context(|| {
        format!("invalid Environment Specification `{environment_path}`")
    })?;
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

    let component = component_bytes(&release_bytes, &bundle)?;
    let runtime = assemble_runtime(plan, &registry)
        .context("could not assemble runtime for detached release")?;
    let output = execute_until_observed(mode, &runtime, &component, &input)?;

    println!("release digest: {release_digest}");
    println!(
        "environment: {} ({})",
        specification.environment().name(),
        specification.environment().id()
    );
    println!("provider: {} ({provider_id})", binding.provider_name());
    println!("mode: {}", match mode {
        RealizationMode::Local => "local",
        RealizationMode::Cloudflare => "cloudflare",
    });
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

fn component_bytes(release_bytes: &[u8], bundle: &PresolveBundle) -> Result<Vec<u8>> {
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

    extract_archive_entry(release_bytes, artifact.path()).with_context(|| {
        format!(
            "could not extract Component artifact `{}` from detached release",
            artifact.path()
        )
    })
}

fn extract_archive_entry(release_bytes: &[u8], expected_path: &str) -> Result<Vec<u8>> {
    let mut archive = Archive::new(Cursor::new(release_bytes));

    for entry in archive.entries().context("could not read release archive")? {
        let mut entry = entry.context("could not read release archive entry")?;
        let path = entry
            .path()
            .context("could not read release archive entry path")?;

        if path.as_ref() != Path::new(expected_path) {
            continue;
        }

        let mut bytes = Vec::new();
        entry
            .read_to_end(&mut bytes)
            .context("could not read embedded Component bytes")?;
        return Ok(bytes);
    }

    bail!("release archive does not contain `{expected_path}`")
}

fn execute_until_observed(
    mode: RealizationMode,
    runtime: &presolve_runtime::Runtime,
    component: &[u8],
    input: &str,
) -> Result<String> {
    let expected = format!("stored:{input}");
    let deadline = Instant::now() + CLOUD_RETRY_TIMEOUT;
    let mut last_output = None;

    loop {
        let output = runtime
            .run_bytes(component, input)
            .context("detached Component execution failed")?;

        if output == expected {
            return Ok(output);
        }

        if mode == RealizationMode::Local {
            bail!("local detached-release execution returned `{output}`, expected `{expected}`");
        }

        last_output = Some(output);

        if Instant::now() >= deadline {
            let observed = last_output.as_deref().unwrap_or("<none>");
            bail!(
                "timed out waiting for Cloudflare KV visibility through the detached release; last output `{observed}`"
            );
        }

        thread::sleep(CLOUD_RETRY_INTERVAL);
    }
}

fn required_env(name: &str) -> Result<String> {
    env::var(name).with_context(|| format!("required environment variable `{name}` is not set"))
}
