use std::{env, fs, sync::Arc};

use anyhow::{Context as _, Result, bail};
use presolve_provider_kv_memory::InMemoryKeyValueProvider;
use presolve_runtime::Runtime;

fn main() -> Result<()> {
    let component_path = env::args()
        .nth(1)
        .context("usage: capability <kv-demo.wasm>")?;

    let bytes = fs::read(&component_path)
        .with_context(|| format!("failed to read component `{component_path}`"))?;

    let unavailable_runtime = Runtime::new().context("failed to create default runtime")?;

    let unavailable_application = unavailable_runtime
        .compile(&bytes)
        .context("failed to compile capability application")?;

    let unavailable_output = unavailable_runtime
        .run(&unavailable_application, "Presolve")
        .context("failed to invoke capability application")?;

    if unavailable_output != "error:unavailable" {
        bail!(
            "expected unavailable provider, got: \
             {unavailable_output}"
        );
    }

    println!("✓ missing provider is explicit rather than ambient");

    let provider = Arc::new(InMemoryKeyValueProvider::new());

    let configured_runtime = Runtime::builder()
        .key_value_provider(provider)
        .build()
        .context("failed to create configured runtime")?;

    let configured_application = configured_runtime
        .compile(&bytes)
        .context("failed to compile capability application")?;

    let configured_output = configured_runtime
        .run(&configured_application, "Presolve")
        .context("failed to execute capability application")?;

    if configured_output != "stored:Presolve" {
        bail!(
            "unexpected configured output: \
             {configured_output}"
        );
    }

    println!("✓ same component executed against in-memory provider");
    println!("Presolve capability check passed");

    Ok(())
}
