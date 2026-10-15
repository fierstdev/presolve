use std::{env, fs, sync::Arc};

use anyhow::{Context, Result, bail};
use presolve_bundle::{PresolveBundle, WorkloadKind, application_contract_for_resolution};
use presolve_environment::{inventory_from_specification, parse_environment};
use presolve_node::{assemble_runtime, realize_plan_providers};
use presolve_provider_realization::{
    EnvironmentSecretResolver, ProviderAdapterId, ProviderAdapterRegistry,
    ProviderRealizationCatalog, ProviderRealizationProfile, SecretReference,
};
use presolve_provider_sdk::SQL_DATABASE_INTERFACE;
use presolve_provider_sql_postgres::{
    POSTGRES_DATABASE_SETTING, POSTGRES_HOST_SETTING, POSTGRES_PASSWORD_SECRET,
    POSTGRES_PORT_SETTING, POSTGRES_SQL_ADAPTER_ID, POSTGRES_USER_SETTING, PostgresSqlAdapter,
};
use presolve_provider_sql_sqlite::{SQLITE_PATH_SETTING, SQLITE_SQL_ADAPTER_ID, SqliteSqlAdapter};
use presolve_resolver::{DeploymentPlan, resolve};

const SQLITE_PATH_ENV: &str = "PRESOLVE_SQLITE_PATH";
const POSTGRES_HOST_ENV: &str = "PRESOLVE_POSTGRES_HOST";
const POSTGRES_PORT_ENV: &str = "PRESOLVE_POSTGRES_PORT";
const POSTGRES_DATABASE_ENV: &str = "PRESOLVE_POSTGRES_DATABASE";
const POSTGRES_USER_ENV: &str = "PRESOLVE_POSTGRES_USER";
const POSTGRES_PASSWORD_ENV: &str = "PRESOLVE_POSTGRES_PASSWORD";
const TAR_BLOCK_BYTES: usize = 512;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum RealizationMode {
    Sqlite,
    Postgres,
}

impl RealizationMode {
    fn parse(value: &str) -> Result<Self> {
        match value {
            "sqlite" => Ok(Self::Sqlite),
            "postgres" => Ok(Self::Postgres),
            _ => bail!("realization mode must be `sqlite` or `postgres`, got `{value}`"),
        }
    }

    const fn label(self) -> &'static str {
        match self {
            Self::Sqlite => "sqlite",
            Self::Postgres => "postgres",
        }
    }
}

fn main() -> Result<()> {
    let mut args = env::args().skip(1);
    let mode = RealizationMode::parse(
        &args
            .next()
            .context("missing realization mode (`sqlite` or `postgres`)")?,
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
            "usage: sql_release_realization_checkpoint <sqlite|postgres> <release.presolved> <presolve.env.toml> <input>"
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
        .context("detached SQL release is not compatible with the selected environment")?;

    let binding = plan
        .bindings()
        .iter()
        .find(|binding| binding.interface() == SQL_DATABASE_INTERFACE)
        .context("resolved plan does not contain the SQL binding required by the demo")?;
    let provider_id = *binding.provider_id();

    let registry = match mode {
        RealizationMode::Sqlite => realize_sqlite_provider(plan, provider_id)?,
        RealizationMode::Postgres => realize_postgres_provider(plan, provider_id)?,
    };

    provision_demo_schema(
        mode,
        registry
            .sql_database(provider_id)
            .context("realized registry does not contain the plan-selected SQL provider")?
            .as_ref(),
    )?;

    let component = component_bytes_from_release(&release_bytes, &bundle)?;
    let runtime = assemble_runtime(plan, &registry)
        .context("could not assemble runtime for detached SQL release")?;
    let output = runtime
        .run_bytes(&component, &input)
        .context("detached Component SQL execution failed")?;
    let expected = format!("sql:{input}");

    if output != expected {
        bail!("unexpected SQL demo output: expected `{expected}`, found `{output}`");
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

fn realize_sqlite_provider(
    plan: &DeploymentPlan,
    provider_id: presolve_core::ProviderId,
) -> Result<presolve_node::ProviderRegistry> {
    let path = required_env(SQLITE_PATH_ENV)?;
    let profile = ProviderRealizationProfile::new(
        provider_id,
        ProviderAdapterId::new(SQLITE_SQL_ADAPTER_ID)
            .context("SQLite SQL adapter ID should be valid")?,
    )
    .with_setting(SQLITE_PATH_SETTING, path)
    .context("SQLite path realization setting should be valid")?;

    let mut profiles = ProviderRealizationCatalog::new();
    profiles
        .register(profile)
        .context("SQLite SQL realization profile should register")?;

    let mut adapters = ProviderAdapterRegistry::new();
    adapters
        .register(
            ProviderAdapterId::new(SQLITE_SQL_ADAPTER_ID)
                .context("SQLite SQL adapter ID should be valid")?,
            Arc::new(SqliteSqlAdapter),
        )
        .context("SQLite SQL adapter should register")?;

    realize_plan_providers(plan, &profiles, &adapters, &EnvironmentSecretResolver)
        .context("could not realize plan-selected SQLite provider")
}

fn realize_postgres_provider(
    plan: &DeploymentPlan,
    provider_id: presolve_core::ProviderId,
) -> Result<presolve_node::ProviderRegistry> {
    let host = required_env(POSTGRES_HOST_ENV)?;
    let port = env::var(POSTGRES_PORT_ENV).unwrap_or_else(|_| "5432".to_owned());
    let database = required_env(POSTGRES_DATABASE_ENV)?;
    let user = required_env(POSTGRES_USER_ENV)?;

    if env::var_os(POSTGRES_PASSWORD_ENV).is_none() {
        bail!("required environment variable `{POSTGRES_PASSWORD_ENV}` is not set");
    }

    let profile = ProviderRealizationProfile::new(
        provider_id,
        ProviderAdapterId::new(POSTGRES_SQL_ADAPTER_ID)
            .context("PostgreSQL SQL adapter ID should be valid")?,
    )
    .with_setting(POSTGRES_HOST_SETTING, host)
    .context("PostgreSQL host realization setting should be valid")?
    .with_setting(POSTGRES_PORT_SETTING, port)
    .context("PostgreSQL port realization setting should be valid")?
    .with_setting(POSTGRES_DATABASE_SETTING, database)
    .context("PostgreSQL database realization setting should be valid")?
    .with_setting(POSTGRES_USER_SETTING, user)
    .context("PostgreSQL user realization setting should be valid")?
    .with_secret(
        POSTGRES_PASSWORD_SECRET,
        SecretReference::environment_variable(POSTGRES_PASSWORD_ENV)
            .context("PostgreSQL password secret reference should be valid")?,
    )
    .context("PostgreSQL password binding should be valid")?;

    let mut profiles = ProviderRealizationCatalog::new();
    profiles
        .register(profile)
        .context("PostgreSQL SQL realization profile should register")?;

    let mut adapters = ProviderAdapterRegistry::new();
    adapters
        .register(
            ProviderAdapterId::new(POSTGRES_SQL_ADAPTER_ID)
                .context("PostgreSQL SQL adapter ID should be valid")?,
            Arc::new(PostgresSqlAdapter),
        )
        .context("PostgreSQL SQL adapter should register")?;

    realize_plan_providers(plan, &profiles, &adapters, &EnvironmentSecretResolver)
        .context("could not realize plan-selected PostgreSQL provider")
}

fn provision_demo_schema(
    mode: RealizationMode,
    provider: &dyn presolve_provider_sdk::SqlDatabaseProvider,
) -> Result<()> {
    let statement = match mode {
        RealizationMode::Sqlite => {
            "CREATE TABLE IF NOT EXISTS presolve_demo (id INTEGER PRIMARY KEY, label TEXT NOT NULL, optional_value BLOB)"
        }
        RealizationMode::Postgres => {
            "CREATE TABLE IF NOT EXISTS presolve_demo (id BIGINT PRIMARY KEY, label TEXT NOT NULL, optional_value BYTEA)"
        }
    };

    provider
        .execute(statement, &[])
        .map(|_| ())
        .with_context(|| format!("could not provision {} demo schema", mode.label()))
}

fn component_bytes_from_release(release_bytes: &[u8], bundle: &PresolveBundle) -> Result<Vec<u8>> {
    let workload = match bundle.manifest().workloads() {
        [workload] => workload,
        workloads => bail!(
            "SQL release realization checkpoint requires exactly one workload, found {}",
            workloads.len()
        ),
    };

    if workload.kind() != WorkloadKind::Component {
        bail!(
            "SQL release realization checkpoint requires a Component workload, found `{}`",
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
