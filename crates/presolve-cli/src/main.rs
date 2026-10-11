#![forbid(unsafe_code)]

use std::process::ExitCode;

fn main() -> ExitCode {
    match presolve_cli::run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("error: {error:#}");
            ExitCode::FAILURE
        }
    }
}
