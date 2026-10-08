use std::{env, fs};

use anyhow::{Context as _, Result, bail};
use presolve_bundle::PresolveBundle;
use presolve_contract::parse_contract;

fn main() -> Result<()> {
    let mut args = env::args().skip(1);

    let contract_path = args
        .next()
        .context("usage: package <presolve.toml> <component.wasm> <output.presolve>")?;

    let component_path = args
        .next()
        .context("usage: package <presolve.toml> <component.wasm> <output.presolve>")?;

    let output_path = args
        .next()
        .context("usage: package <presolve.toml> <component.wasm> <output.presolve>")?;

    if args.next().is_some() {
        bail!("usage: package <presolve.toml> <component.wasm> <output.presolve>");
    }

    let contract_source = fs::read_to_string(&contract_path)
        .with_context(|| format!("failed to read contract `{contract_path}`"))?;

    let contract = parse_contract(&contract_source).map_err(|diagnostics| {
        anyhow::anyhow!("failed to parse application contract: {diagnostics:?}")
    })?;

    let component = fs::read(&component_path)
        .with_context(|| format!("failed to read component `{component_path}`"))?;

    let bundle = PresolveBundle::from_component(&contract, &component)
        .context("failed to construct application bundle")?;

    let release_digest = bundle
        .release_digest()
        .context("failed to compute application release digest")?;

    let encoded = bundle
        .encode()
        .context("failed to encode application bundle")?;

    let decoded = PresolveBundle::decode(&encoded).context("encoded bundle did not verify")?;

    let decoded_digest = decoded
        .release_digest()
        .context("failed to compute decoded release digest")?;

    if decoded_digest != release_digest {
        bail!(
            "release identity changed after bundle round trip: \
             {release_digest} != {decoded_digest}"
        );
    }

    fs::write(&output_path, &encoded)
        .with_context(|| format!("failed to write bundle `{output_path}`"))?;

    println!(
        "✓ packaged {}@{}",
        bundle.manifest().application().name(),
        bundle.manifest().application().version(),
    );
    println!("✓ release digest: {release_digest}");
    println!("✓ bundle round-trip verified");

    Ok(())
}
