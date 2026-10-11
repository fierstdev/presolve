use std::{
    env,
    sync::Arc,
    thread,
    time::{Duration, Instant},
};

use anyhow::{Context, Result, bail};
use presolve_contract::parse_contract;
use presolve_core::{EnvironmentId, ProviderId};
use presolve_environment::{
    EnvironmentMetadata, EnvironmentSpecification, ExecutionSupport,
    ProvidedCapabilitySpecification, ProviderSpecification, ResourceCapacity,
    inventory_from_specification,
};
use presolve_node::realize_plan_providers;
use presolve_provider_kv_cloudflare::{
    CLOUDFLARE_ACCOUNT_ID_SETTING, CLOUDFLARE_API_TOKEN_SECRET, CLOUDFLARE_KV_ADAPTER_ID,
    CLOUDFLARE_NAMESPACE_ID_SETTING, CloudflareKvAdapter,
};
use presolve_provider_realization::{
    EnvironmentSecretResolver, ProviderAdapterId, ProviderAdapterRegistry,
    ProviderRealizationCatalog, ProviderRealizationProfile, SecretReference,
};
use presolve_provider_sdk::KEY_VALUE_INTERFACE;
use presolve_resolver::resolve;
use semver::Version;
use uuid::Uuid;

const ACCOUNT_ID_ENV: &str = "PRESOLVE_CLOUDFLARE_ACCOUNT_ID";
const NAMESPACE_ID_ENV: &str = "PRESOLVE_CLOUDFLARE_KV_NAMESPACE_ID";
const API_TOKEN_ENV: &str = "PRESOLVE_CLOUDFLARE_API_TOKEN";

const VISIBILITY_TIMEOUT: Duration = Duration::from_secs(75);
const POLL_INTERVAL: Duration = Duration::from_secs(1);

fn main() -> Result<()> {
    let account_id = required_env(ACCOUNT_ID_ENV)?;
    let namespace_id = required_env(NAMESPACE_ID_ENV)?;

    if env::var_os(API_TOKEN_ENV).is_none() {
        bail!("required environment variable `{API_TOKEN_ENV}` is not set");
    }

    let (provider_id, provider) = materialize_live_provider(account_id, namespace_id)?;

    let run_id = Uuid::new_v4();
    let key = format!("presolve/chunk18/live/{run_id}");
    let value = format!("presolve-cloudflare-ok:{run_id}");

    println!("resolved provider: {provider_id}");
    println!("live proof key: {key}");

    provider
        .set(&key, &value)
        .context("live Cloudflare KV write failed")?;

    let visibility_result = wait_for_value(provider.as_ref(), &key, &value);

    if let Err(error) = visibility_result {
        let _ = provider.delete(&key);
        return Err(error);
    }

    println!("observed live value through selected provider");

    let deleted = provider
        .delete(&key)
        .context("live Cloudflare KV cleanup delete failed")?;

    if !deleted {
        bail!("Cloudflare KV cleanup did not report the live proof key as existing");
    }

    println!("cleaned up live proof key");
    println!("Chunk 18.4 live Cloudflare KV portability proof passed");

    Ok(())
}

fn materialize_live_provider(
    account_id: String,
    namespace_id: String,
) -> Result<(ProviderId, Arc<dyn presolve_provider_sdk::KeyValueProvider>)> {
    let provider_id = ProviderId::new();
    let environment_id = EnvironmentId::new();

    let contract = parse_contract(
        r#"
contract_version = "0.1"

[application]
name = "cloudflare-kv-live-checkpoint"
version = "1.0.0"

[[capabilities]]
interface = "presolve:kv/store"
version = "^0.1"
"#,
    )
    .context("live proof Application Contract should parse")?;

    let specification = EnvironmentSpecification::current(
        EnvironmentMetadata::new(environment_id, "cloudflare-live-proof"),
        ResourceCapacity::new(1_024, 1_000),
        ExecutionSupport::components(),
        vec![ProviderSpecification::new(
            provider_id,
            "cloudflare-kv",
            vec![ProvidedCapabilitySpecification::new(
                KEY_VALUE_INTERFACE,
                Version::new(0, 1, 0),
            )],
        )],
    )
    .context("live proof Environment Specification should be valid")?;

    let inventory = inventory_from_specification(&specification)
        .context("live proof Environment Specification should enrich")?;

    let resolution = resolve(&contract, &inventory);
    let plan = resolution
        .plan()
        .context("live proof application should resolve against Cloudflare KV supply")?;

    let selected_binding = plan
        .bindings()
        .iter()
        .find(|binding| binding.interface() == KEY_VALUE_INTERFACE)
        .context("resolved plan should contain the KV capability binding")?;

    if *selected_binding.provider_id() != provider_id {
        bail!(
            "resolver selected provider `{}` instead of expected `{provider_id}`",
            selected_binding.provider_id()
        );
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

    let registry = realize_plan_providers(plan, &profiles, &adapters, &EnvironmentSecretResolver)
        .context("resolved Cloudflare provider should materialize")?;

    let provider = registry
        .key_value(provider_id)
        .context("materialized registry should contain selected Cloudflare KV provider")?;

    Ok((provider_id, provider))
}

fn required_env(name: &str) -> Result<String> {
    env::var(name).with_context(|| format!("required environment variable `{name}` is not set"))
}

fn wait_for_value(
    provider: &dyn presolve_provider_sdk::KeyValueProvider,
    key: &str,
    expected: &str,
) -> Result<()> {
    let deadline = Instant::now() + VISIBILITY_TIMEOUT;
    let mut last_observed: Option<String> = None;

    loop {
        match provider.get(key) {
            Ok(Some(value)) if value == expected => return Ok(()),
            Ok(Some(value)) => {
                last_observed = Some(value);
            }
            Ok(None) => {}
            Err(error) => {
                if Instant::now() >= deadline {
                    return Err(error)
                        .context("Cloudflare KV read failed before visibility timeout");
                }
            }
        }

        if Instant::now() >= deadline {
            match last_observed {
                Some(value) => bail!(
                    "timed out waiting for Cloudflare KV visibility; last observed unexpected value `{value}`"
                ),
                None => {
                    bail!("timed out waiting for Cloudflare KV visibility; key remained missing")
                }
            }
        }

        thread::sleep(POLL_INTERVAL);
    }
}
