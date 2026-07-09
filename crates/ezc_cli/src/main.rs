use std::env;
use std::fs;
use std::path::PathBuf;
use std::process;

use ezc_core::{explain_json, explain_text, summarize_source};
use ezc_parser::{parse_file, ParseSeverity, ParsedFile};

fn main() {
    let mut args = env::args().skip(1).collect::<Vec<_>>();

    if args.is_empty() {
        print_usage_and_exit();
    }

    let command = args.remove(0);

    match command.as_str() {
        "explain" => run_explain(args),
        "parse" => run_parse(args),
        _ => {
            eprintln!("unknown command: {command}");
            print_usage_and_exit();
        }
    }
}

fn run_explain(mut args: Vec<String>) {
    if args.is_empty() {
        eprintln!("missing file path");
        print_usage_and_exit();
    }

    let path = PathBuf::from(args.remove(0));

    let format = parse_format(&args);

    let source = fs::read_to_string(&path).unwrap_or_else(|error| {
        eprintln!("failed to read {}: {error}", path.display());
        process::exit(1);
    });

    let summary = summarize_source(&path, &source);

    match format.as_str() {
        "text" => print!("{}", explain_text(&summary)),
        "json" => print!("{}", explain_json(&summary)),
        _ => {
            eprintln!("unsupported format: {format}");
            process::exit(1);
        }
    }
}

fn run_parse(mut args: Vec<String>) {
    if args.is_empty() {
        eprintln!("missing file path");
        print_usage_and_exit();
    }

    let path = PathBuf::from(args.remove(0));

    let source = fs::read_to_string(&path).unwrap_or_else(|error| {
        eprintln!("failed to read {}: {error}", path.display());
        process::exit(1);
    });

    let parsed = parse_file(&path, &source);

    print_parsed_file(&parsed);
}

fn parse_format(args: &[String]) -> String {
    let mut format = "text".to_string();

    let mut index = 0;
    while index < args.len() {
        match args[index].as_str() {
            "--format" => {
                let Some(value) = args.get(index + 1) else {
                    eprintln!("missing value for --format");
                    process::exit(1);
                };

                format = value.clone();
                index += 2;
            }
            unknown => {
                eprintln!("unknown option: {unknown}");
                process::exit(1);
            }
        }
    }

    format
}

fn print_parsed_file(parsed: &ParsedFile) {
    println!("File: {}", parsed.path.display());

    println!("Diagnostics:");
    if parsed.diagnostics.is_empty() {
        println!("  none");
    } else {
        for diagnostic in &parsed.diagnostics {
            println!(
                "  {}: {}",
                diagnostic_severity_label(&diagnostic.severity),
                diagnostic.message
            );

            for label in &diagnostic.labels {
                println!(
                    "    at {}:{} span={}..{}",
                    label.span.line, label.span.column, label.span.start, label.span.end
                );
            }
        }
    }

    println!();
    println!("Classes:");
    if parsed.classes.is_empty() {
        println!("  none");
        return;
    }

    for class in &parsed.classes {
        println!(
            "  class {} at {}:{}",
            class.name, class.span.line, class.span.column
        );

        println!("    decorators:");
        if class.decorators.is_empty() {
            println!("      none");
        } else {
            for decorator in &class.decorators {
                match &decorator.argument {
                    Some(argument) => {
                        println!(
                            "      @{}({argument:?}) at {}:{}",
                            decorator.name, decorator.span.line, decorator.span.column
                        );
                    }
                    None => {
                        println!(
                            "      @{} at {}:{}",
                            decorator.name, decorator.span.line, decorator.span.column
                        );
                    }
                }
            }
        }

        println!("    properties:");
        if class.properties.is_empty() {
            println!("      none");
        } else {
            for property in &class.properties {
                match &property.initializer {
                    Some(initializer) => {
                        println!(
                            "      {} = {} at {}:{}",
                            property.name, initializer, property.span.line, property.span.column
                        );
                    }
                    None => {
                        println!(
                            "      {} at {}:{}",
                            property.name, property.span.line, property.span.column
                        );
                    }
                }
            }
        }

        println!("    methods:");
        if class.methods.is_empty() {
            println!("      none");
        } else {
            for method in &class.methods {
                println!(
                    "      {} at {}:{}",
                    method.name, method.span.line, method.span.column
                );

                if method.jsx_roots.is_empty() {
                    println!("        jsx roots: none");
                } else {
                    for jsx in &method.jsx_roots {
                        println!(
                            "        jsx root <{}> at {}:{}",
                            jsx.name, jsx.span.line, jsx.span.column
                        );

                        if jsx.attributes.is_empty() {
                            println!("          attributes: none");
                        } else {
                            println!("          attributes: {}", jsx.attributes.join(", "));
                        }
                    }
                }

                if method.bindings.is_empty() {
                    println!("        bindings: none");
                } else {
                    println!("        bindings: {}", method.bindings.join(", "));
                }
            }
        }
    }
}

fn diagnostic_severity_label(severity: &ParseSeverity) -> &'static str {
    match severity {
        ParseSeverity::Info => "Info",
        ParseSeverity::Warning => "Warning",
        ParseSeverity::Error => "Error",
    }
}

fn print_usage_and_exit() -> ! {
    eprintln!("usage:");
    eprintln!("  ezc_cli explain <file> [--format text|json]");
    eprintln!("  ezc_cli parse <file>");
    process::exit(1);
}
