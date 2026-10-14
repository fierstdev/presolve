use std::{env, fs, sync::Arc};

use anyhow::{Context, Result, bail};
use presolve_bundle::{PresolveBundle, WorkloadKind, application_contract_for_resolution};
use presolve_environment::{inventory_from_specification, parse_environment};
use presolve_node::{assemble_runtime, realize_local_providers, realize_plan_providers};
use presolve_provider_objects_s3::{
    S3_ACCESS_KEY_ID_SECRET, S3_BUCKET_SETTING, S3_ENDPOINT_SETTING, S3_OBJECTS_ADAPTER_ID,
    S3_REGION_SETTING, S3_SECRET_ACCESS_KEY_SECRET, S3ObjectStoreAdapter,
};
use presolve_provider_realization::{
    EnvironmentSecretResolver, ProviderAdapterId, ProviderAdapterRegistry,
    ProviderRealizationCatalog, ProviderRealizationProfile, SecretReference,
};
use presolve_provider_sdk::OBJECT_STORE_INTERFACE;
use presolve_resolver::{DeploymentPlan, resolve};

const S3_ENDPOINT_ENV: &str = "PRESOLVE_S3_ENDPOINT";
const S3_BUCKET_ENV: &str = "PRESOLVE_S3_BUCKET";
const S3_REGION_ENV: &str = "PRESOLVE_S3_REGION";
const S3_ACCESS_KEY_ID_ENV: &str = "PRESOLVE_S3_ACCESS_KEY_ID";
const S3_SECRET_ACCESS_KEY_ENV: &str = "PRESOLVE_S3_SECRET_ACCESS_KEY";
const TAR_BLOCK_BYTES: usize = 512;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum RealizationMode {
    Local,
    S3,
}

impl RealizationMode {
    fn parse(value: &str) -> Result<Self> {
        match value {
            "local" => Ok(Self::Local),
            "s3" => Ok(Self::S3),
            _ => bail!("realization mode must be `local` or `s3`, got `{value}`"),
        }
    }

    const fn label(self) -> &'static str {
        match self {
            Self::Local => "local",
            Self::S3 => "s3",
        }
    }
}

fn main() -> Result<()> {
    let mut args = env::args().skip(1);
    let mode = RealizationMode::parse(
        &args
            .next()
            .context("missing realization mode (`local` or `s3`)")?,
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
            "usage: object_release_realization_checkpoint <local|s3> <release.presolved> <presolve.env.toml> <input>"
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
        .context("detached object release is not compatible with the selected environment")?;

    let binding = plan
        .bindings()
        .iter()
        .find(|binding| binding.interface() == OBJECT_STORE_INTERFACE)
        .context("resolved plan does not contain the object-store binding required by the demo")?;
    let provider_id = *binding.provider_id();

    let registry = match mode {
        RealizationMode::Local => realize_local_providers(plan)
            .context("could not realize selected object provider with local implementation")?,
        RealizationMode::S3 => realize_s3_provider(plan, provider_id)?,
    };

    let component = component_bytes_from_release(&release_bytes, &bundle)?;
    let runtime = assemble_runtime(plan, &registry)
        .context("could not assemble runtime for detached object release")?;
    let output = runtime
        .run_bytes(&component, &input)
        .context("detached Component object-storage execution failed")?;
    let expected = format!("objects:{input}");

    if output != expected {
        bail!("unexpected object demo output: expected `{expected}`, found `{output}`");
    }

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

fn realize_s3_provider(
    plan: &DeploymentPlan,
    provider_id: presolve_core::ProviderId,
) -> Result<presolve_node::ProviderRegistry> {
    let endpoint = required_env(S3_ENDPOINT_ENV)?;
    let bucket = required_env(S3_BUCKET_ENV)?;
    let region = required_env(S3_REGION_ENV)?;

    for name in [S3_ACCESS_KEY_ID_ENV, S3_SECRET_ACCESS_KEY_ENV] {
        if env::var_os(name).is_none() {
            bail!("required environment variable `{name}` is not set");
        }
    }

    let profile = ProviderRealizationProfile::new(
        provider_id,
        ProviderAdapterId::new(S3_OBJECTS_ADAPTER_ID)
            .context("S3 object adapter ID should be valid")?,
    )
    .with_setting(S3_ENDPOINT_SETTING, endpoint)
    .context("S3 endpoint realization setting should be valid")?
    .with_setting(S3_BUCKET_SETTING, bucket)
    .context("S3 bucket realization setting should be valid")?
    .with_setting(S3_REGION_SETTING, region)
    .context("S3 region realization setting should be valid")?
    .with_secret(
        S3_ACCESS_KEY_ID_SECRET,
        SecretReference::environment_variable(S3_ACCESS_KEY_ID_ENV)
            .context("S3 access key secret reference should be valid")?,
    )
    .context("S3 access key binding should be valid")?
    .with_secret(
        S3_SECRET_ACCESS_KEY_SECRET,
        SecretReference::environment_variable(S3_SECRET_ACCESS_KEY_ENV)
            .context("S3 secret key reference should be valid")?,
    )
    .context("S3 secret key binding should be valid")?;

    let mut profiles = ProviderRealizationCatalog::new();
    profiles
        .register(profile)
        .context("S3 object realization profile should register")?;

    let mut adapters = ProviderAdapterRegistry::new();
    adapters
        .register(
            ProviderAdapterId::new(S3_OBJECTS_ADAPTER_ID)
                .context("S3 object adapter ID should be valid")?,
            Arc::new(S3ObjectStoreAdapter),
        )
        .context("S3 object adapter should register")?;

    realize_plan_providers(plan, &profiles, &adapters, &EnvironmentSecretResolver)
        .context("could not realize plan-selected S3-compatible object provider")
}

fn component_bytes_from_release(release_bytes: &[u8], bundle: &PresolveBundle) -> Result<Vec<u8>> {
    let workload = match bundle.manifest().workloads() {
        [workload] => workload,
        workloads => bail!(
            "object release realization checkpoint requires exactly one workload, found {}",
            workloads.len()
        ),
    };

    if workload.kind() != WorkloadKind::Component {
        bail!(
            "object release realization checkpoint requires a Component workload, found `{}`",
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

fn required_env(name: &str) -> Result<String> {
    env::var(name).with_context(|| format!("required environment variable `{name}` is not set"))
}
