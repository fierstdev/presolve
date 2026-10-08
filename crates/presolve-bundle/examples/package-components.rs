use std::{env, fs};

use anyhow::{bail, Context as _, Result};
use presolve_bundle::{ComponentArtifactInput, PresolveBundle};
use presolve_contract::parse_contract;

fn main() -> Result<()> {
    let mut args = env::args().skip(1);

    let contract_path = args.next().context(
        "usage: package-components <presolve.toml> <output.presolve> \
         <workload=component.wasm>...",
    )?;

    let output_path = args.next().context(
        "usage: package-components <presolve.toml> <output.presolve> \
         <workload=component.wasm>...",
    )?;

    let artifact_args = args.collect::<Vec<_>>();

    if artifact_args.is_empty() {
        bail!(
            "usage: package-components <presolve.toml> <output.presolve> \
             <workload=component.wasm>..."
        );
    }

    let contract_source = fs::read_to_string(&contract_path)
        .with_context(|| format!("failed to read contract `{contract_path}`"))?;

    let contract = parse_contract(&contract_source).map_err(|diagnostics| {
        anyhow::anyhow!("failed to parse application contract: {diagnostics:?}")
    })?;

    let mut artifacts = Vec::<(String, Vec<u8>)>::new();

    for argument in artifact_args {
        let (workload, component_path) = argument.split_once('=').with_context(|| {
            format!(
                "invalid component argument `{argument}`; \
                 expected <workload=component.wasm>"
            )
        })?;

        if workload.is_empty() || component_path.is_empty() {
            bail!(
                "invalid component argument `{argument}`; \
                 expected <workload=component.wasm>"
            );
        }

        let component = fs::read(component_path)
            .with_context(|| format!("failed to read component `{component_path}`"))?;

        artifacts.push((workload.to_owned(), component));
    }

    let inputs = artifacts
        .iter()
        .map(|(workload, component)| ComponentArtifactInput::new(workload, component))
        .collect::<Vec<_>>();

    let bundle = PresolveBundle::from_components(&contract, &inputs)
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

    for workload in bundle.manifest().workloads() {
        println!("✓ workload {} ({})", workload.name(), workload.kind());
    }

    println!("✓ release digest: {release_digest}");
    println!("✓ bundle round-trip verified");

    Ok(())
}
