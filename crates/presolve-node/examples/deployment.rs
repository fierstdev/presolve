use std::{env, fs, sync::Arc};

use anyhow::{Context as _, Result, bail};
use presolve_contract::parse_contract;
use presolve_core::{EnvironmentId, ProviderId};
use presolve_node::{AssemblyError, ProviderRegistry, assemble_runtime};
use presolve_provider_kv_memory::InMemoryKeyValueProvider;
use presolve_provider_sdk::KEY_VALUE_INTERFACE;
use presolve_resolver::{
    EnvironmentInventory, EnvironmentResources, ProvidedCapability, ProviderDescriptor, resolve,
};
use semver::Version;

fn main() -> Result<()> {
    let mut args = env::args().skip(1);

    let contract_path = args
        .next()
        .context("usage: deployment <presolve.toml> <kv-demo.wasm>")?;

    let component_path = args
        .next()
        .context("usage: deployment <presolve.toml> <kv-demo.wasm>")?;

    if args.next().is_some() {
        bail!("usage: deployment <presolve.toml> <kv-demo.wasm>");
    }

    let contract_source = fs::read_to_string(&contract_path)
        .with_context(|| format!("failed to read contract `{contract_path}`"))?;

    let contract = parse_contract(&contract_source).map_err(|diagnostics| {
        anyhow::anyhow!("failed to parse application contract: {diagnostics:?}")
    })?;

    let component_bytes = fs::read(&component_path)
        .with_context(|| format!("failed to read component `{component_path}`"))?;

    let provider_id = ProviderId::new();

    let environment = EnvironmentInventory::new(
        EnvironmentId::new(),
        EnvironmentResources::new(512, 1_000),
        vec![ProviderDescriptor::new(
            provider_id,
            "memory-kv",
            vec![ProvidedCapability::new(
                KEY_VALUE_INTERFACE,
                Version::new(0, 1, 0),
            )],
        )],
    );

    let report = resolve(&contract, &environment);

    let plan = report.plan().with_context(|| {
        format!(
            "environment should resolve application, problems: {:?}",
            report.problems()
        )
    })?;

    let binding = plan
        .bindings()
        .iter()
        .find(|binding| binding.interface() == KEY_VALUE_INTERFACE)
        .context("resolved plan did not contain key/value binding")?;

    println!(
        "✓ resolver selected {}@{} from {}",
        binding.interface(),
        binding.provider_version(),
        binding.provider_name(),
    );

    let selected_provider_id = *binding.provider_id();

    match assemble_runtime(plan, &ProviderRegistry::new()) {
        Err(AssemblyError::MissingProvider { .. }) => {
            println!("✓ unresolved provider implementation is rejected");
        }

        Err(error) => {
            return Err(error).context("empty provider registry failed for an unexpected reason");
        }

        Ok(_) => {
            bail!(
                "runtime assembly unexpectedly succeeded without the selected \
                 provider implementation"
            );
        }
    }

    let mut registry = ProviderRegistry::new();

    registry
        .register_key_value(
            selected_provider_id,
            Arc::new(InMemoryKeyValueProvider::new()),
        )
        .context("failed to register selected provider implementation")?;

    let runtime =
        assemble_runtime(plan, &registry).context("failed to materialize deployment plan")?;

    println!("✓ deployment plan materialized into runtime bindings");

    let application = runtime
        .compile(&component_bytes)
        .context("failed to compile resolved component")?;

    let output = runtime
        .run(&application, "Presolve")
        .context("failed to execute resolved component")?;

    if output != "stored:Presolve" {
        bail!("unexpected component output: {output}");
    }

    println!("✓ component executed through resolved deployment plan");

    Ok(())
}
