use std::{env, fs};

use anyhow::{Context as _, Result};
use edgezero_runtime::Runtime;

fn main() -> Result<()> {
    let mut arguments = env::args().skip(1);

    let component_path = arguments
        .next()
        .context("usage: invoke <component.wasm> <input>")?;

    let input = arguments
        .next()
        .context("usage: invoke <component.wasm> <input>")?;

    let bytes = fs::read(&component_path)
        .with_context(|| format!("failed to read component `{component_path}`"))?;

    let runtime = Runtime::new().context("failed to create EdgeZero runtime")?;

    let application = runtime
        .compile(&bytes)
        .context("failed to compile EdgeZero application")?;

    let output = runtime
        .run(&application, &input)
        .context("failed to execute EdgeZero application")?;

    println!("{output}");

    Ok(())
}
