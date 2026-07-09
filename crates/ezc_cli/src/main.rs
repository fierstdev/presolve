use std::env;
use std::fs;
use std::process::ExitCode;

use ezc_core::{explain_json, explain_text, summarize_source};

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(message) => {
            eprintln!("error: {message}");
            ExitCode::FAILURE
        }
    }
}

fn run() -> Result<(), String> {
    let mut args = env::args().skip(1);
    let command = args.next().ok_or_else(print_usage)?;

    match command.as_str() {
        "explain" => explain_command(args.collect()),
        "help" | "--help" | "-h" => {
            println!("{}", usage());
            Ok(())
        }
        other => Err(format!("unknown command `{other}`\n\n{}", usage())),
    }
}

fn explain_command(args: Vec<String>) -> Result<(), String> {
    let mut path = None;
    let mut format = OutputFormat::Text;
    let mut index = 0;

    while index < args.len() {
        match args[index].as_str() {
            "--format" => {
                let value = args
                    .get(index + 1)
                    .ok_or_else(|| "--format requires `text` or `json`".to_string())?;
                format = OutputFormat::parse(value)?;
                index += 2;
            }
            value if value.starts_with("--format=") => {
                let value = value.trim_start_matches("--format=");
                format = OutputFormat::parse(value)?;
                index += 1;
            }
            value if value.starts_with('-') => {
                return Err(format!("unknown option `{value}`"));
            }
            value => {
                if path.replace(value.to_string()).is_some() {
                    return Err("explain accepts exactly one source path".to_string());
                }
                index += 1;
            }
        }
    }

    let path = path.ok_or_else(|| "explain requires a source path".to_string())?;
    let source = fs::read_to_string(&path).map_err(|err| format!("failed to read `{path}`: {err}"))?;
    let summary = summarize_source(&path, &source);

    match format {
        OutputFormat::Text => print!("{}", explain_text(&summary)),
        OutputFormat::Json => print!("{}", explain_json(&summary)),
    }

    Ok(())
}

fn print_usage() -> String {
    usage()
}

fn usage() -> String {
    "Usage:\n  ezc explain <path> [--format text|json]\n  ezc help".to_string()
}

enum OutputFormat {
    Text,
    Json,
}

impl OutputFormat {
    fn parse(value: &str) -> Result<Self, String> {
        match value {
            "text" => Ok(Self::Text),
            "json" => Ok(Self::Json),
            other => Err(format!("unsupported format `{other}`; expected `text` or `json`")),
        }
    }
}
