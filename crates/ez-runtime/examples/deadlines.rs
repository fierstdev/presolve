use std::{
    env, fs,
    sync::{Arc, Barrier},
    thread,
    time::Duration,
};

use anyhow::{Context as _, Result, bail};
use edgezero_runtime::{Runtime, RuntimeError, RuntimeLimits};

fn main() -> Result<()> {
    let mut arguments = env::args().skip(1);

    let hello_path = arguments
        .next()
        .context("usage: deadlines <hello.wasm> <adversarial.wasm>")?;

    let adversarial_path = arguments
        .next()
        .context("usage: deadlines <hello.wasm> <adversarial.wasm>")?;

    let hello_bytes =
        fs::read(&hello_path).with_context(|| format!("failed to read `{hello_path}`"))?;

    let adversarial_bytes = fs::read(&adversarial_path)
        .with_context(|| format!("failed to read `{adversarial_path}`"))?;

    let limits = RuntimeLimits::new()
        .with_fuel(u64::MAX)
        .with_execution_timeout(Duration::from_millis(250));

    let runtime =
        Arc::new(Runtime::with_limits(limits).context("failed to create EdgeZero runtime")?);

    let hello = Arc::new(
        runtime
            .compile(&hello_bytes)
            .context("failed to compile hello component")?,
    );

    let adversarial = Arc::new(
        runtime
            .compile(&adversarial_bytes)
            .context("failed to compile adversarial component")?,
    );

    let barrier = Arc::new(Barrier::new(2));

    let spin_runtime = Arc::clone(&runtime);
    let spin_application = Arc::clone(&adversarial);
    let spin_barrier = Arc::clone(&barrier);

    let spin_thread = thread::spawn(move || {
        spin_barrier.wait();
        spin_runtime.run(&spin_application, "spin")
    });

    barrier.wait();

    for _ in 0..20 {
        let output = runtime
            .run(&hello, "Concurrent")
            .context("bounded concurrent application failed")?;

        if output != "Hello from EdgeZero, Concurrent." {
            bail!("unexpected concurrent application output: {output}");
        }
    }

    let spin_result = spin_thread
        .join()
        .map_err(|_| anyhow::anyhow!("spin thread panicked"))?;

    match spin_result {
        Err(RuntimeError::DeadlineExceeded(_)) => {}
        Err(error) => {
            bail!("runaway component failed for unexpected reason: {error}");
        }
        Ok(output) => {
            bail!("runaway component unexpectedly completed: {output}");
        }
    }

    println!("✓ runaway application stopped by wall-clock deadline");
    println!("✓ concurrent bounded applications remained healthy");
    println!("EdgeZero deadline checks passed");

    Ok(())
}
