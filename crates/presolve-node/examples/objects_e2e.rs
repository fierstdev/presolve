use std::{
    env, fs,
    path::{Path, PathBuf},
    sync::Arc,
};

use anyhow::{Context, Result, bail};
use presolve_capability::object_store_contract;
use presolve_contract::parse_contract;
use presolve_core::{EnvironmentId, ProviderId};
use presolve_node::{ProviderRegistry, assemble_runtime};
use presolve_provider_objects_memory::InMemoryObjectStoreProvider;
use presolve_resolver::{
    EnvironmentInventory, EnvironmentResources, ProvidedCapability, ProviderDescriptor, resolve,
};

const DEFAULT_INPUT: &str = "presolve-e2e";

fn main() -> Result<()> {
    let mut args = env::args_os().skip(1);

    let component_path = args
        .next()
        .map_or_else(default_component_path, PathBuf::from);

    let input = args
        .next()
        .map(|value| {
            value
                .into_string()
                .map_err(|_| anyhow::anyhow!("input must be valid UTF-8"))
        })
        .transpose()?
        .unwrap_or_else(|| DEFAULT_INPUT.to_owned());

    if args.next().is_some() {
        bail!("usage: objects_e2e [component.wasm] [input]");
    }

    let contract_source = fs::read_to_string("examples/objects-demo/presolve.toml")
        .context("failed to read objects-demo application contract")?;
    let contract =
        parse_contract(&contract_source).context("objects-demo contract should parse")?;

    let provider_id = ProviderId::new();
    let semantic_contract =
        object_store_contract().context("canonical object-store contract should be valid")?;

    let environment = EnvironmentInventory::new(
        EnvironmentId::new(),
        EnvironmentResources::new(512, 1_000),
        vec![ProviderDescriptor::new(
            provider_id,
            "memory-objects",
            vec![ProvidedCapability::from_contract(semantic_contract)],
        )],
    );

    let report = resolve(&contract, &environment);
    let plan = report
        .plan()
        .context("objects-demo should resolve against the memory object provider")?;

    let provider = Arc::new(InMemoryObjectStoreProvider::new());
    let mut registry = ProviderRegistry::new();

    registry
        .register_object_store(provider_id, provider)
        .context("object provider registration should succeed")?;

    let runtime =
        assemble_runtime(plan, &registry).context("deployment plan should materialize")?;

    let component = fs::read(&component_path)
        .with_context(|| format!("failed to read component `{}`", component_path.display()))?;

    let output = runtime
        .run_bytes(&component, &input)
        .context("object-storage component execution failed")?;

    let expected = format!("objects:{input}");

    if output != expected {
        bail!("unexpected component output: expected `{expected}`, found `{output}`");
    }

    println!("{output}");
    Ok(())
}

fn default_component_path() -> PathBuf {
    Path::new("examples")
        .join("objects-demo")
        .join("target")
        .join("wasm32-wasip2")
        .join("debug")
        .join("presolve_example_objects_demo.wasm")
}
