#![forbid(unsafe_code)]

//! Presolve command-line interface.

mod cli;
mod environment;
mod provider;

use std::{env, path::Path};

use anyhow::{Context, Result};
use clap::Parser;

use crate::{
    cli::{Cli, EnvironmentCommand, EnvironmentProviderCommand, RootCommand},
    environment::{create_environment, load_environment},
    provider::{add_provider, list_providers, remove_provider},
};

/// Runs the Presolve CLI using process arguments and the current directory.
///
/// # Errors
///
/// Returns an error when command execution fails.
pub fn run() -> Result<()> {
    let current_directory =
        env::current_dir().context("could not determine the current working directory")?;

    execute(Cli::parse(), &current_directory)
}

fn execute(cli: Cli, current_directory: &Path) -> Result<()> {
    match cli.command {
        RootCommand::Env { command } => match command {
            EnvironmentCommand::Init(args) => {
                let document = create_environment(
                    current_directory,
                    &args.name,
                    args.memory_mib,
                    args.cpu_millis,
                )?;

                println!("Initialized environment `{}`", args.name);
                println!("  id: {}", document.specification.environment().id());
                println!("  file: {}", document.path.display());
                println!(
                    "  capacity: {} MiB memory, {}m CPU",
                    document.specification.resources().memory_mib(),
                    document.specification.resources().cpu_millis()
                );
                println!(
                    "  note: capacity values are Presolve allocation defaults, not host detection"
                );
            }

            EnvironmentCommand::Show => {
                let document = load_environment(current_directory)?;
                print_environment(&document);
            }

            EnvironmentCommand::Provider { command } => match command {
                EnvironmentProviderCommand::Add(args) => {
                    let mutation = add_provider(
                        current_directory,
                        &args.name,
                        &args.capabilities,
                        &args.features,
                    )?;

                    println!("Added provider `{}`", mutation.provider.name());
                    println!("  id: {}", mutation.provider.id());
                    println!("  file: {}", mutation.path.display());

                    for capability in mutation.provider.capabilities() {
                        if capability.features().is_empty() {
                            println!(
                                "  provides: {}@{}",
                                capability.interface(),
                                capability.version()
                            );
                        } else {
                            println!(
                                "  provides: {}@{} [{}]",
                                capability.interface(),
                                capability.version(),
                                capability.features().join(", ")
                            );
                        }
                    }
                }

                EnvironmentProviderCommand::List => {
                    let document = list_providers(current_directory)?;
                    let providers = document.specification.providers();

                    println!(
                        "Providers for environment `{}`",
                        document.specification.environment().name()
                    );

                    if providers.is_empty() {
                        println!("  none");
                    } else {
                        for provider in providers {
                            println!("  {} ({})", provider.name(), provider.id());

                            for capability in provider.capabilities() {
                                if capability.features().is_empty() {
                                    println!(
                                        "    {}@{}",
                                        capability.interface(),
                                        capability.version()
                                    );
                                } else {
                                    println!(
                                        "    {}@{} [{}]",
                                        capability.interface(),
                                        capability.version(),
                                        capability.features().join(", ")
                                    );
                                }
                            }
                        }
                    }
                }

                EnvironmentProviderCommand::Remove(args) => {
                    let mutation = remove_provider(current_directory, &args.name)?;

                    println!("Removed provider `{}`", mutation.provider.name());
                    println!("  id: {}", mutation.provider.id());
                    println!("  file: {}", mutation.path.display());
                }
            },
        },
    }

    Ok(())
}

fn print_environment(document: &environment::EnvironmentDocument) {
    let specification = &document.specification;
    let resources = specification.resources();
    let workloads = specification
        .execution()
        .workloads()
        .iter()
        .map(ToString::to_string)
        .collect::<Vec<_>>()
        .join(", ");

    println!("Environment: {}", specification.environment().name());
    println!("ID: {}", specification.environment().id());
    println!("File: {}", document.path.display());
    println!(
        "Resources: {} MiB memory, {}m CPU",
        resources.memory_mib(),
        resources.cpu_millis()
    );
    println!("Workloads: {workloads}");
    println!("Providers: {}", specification.providers().len());

    for provider in specification.providers() {
        println!("  {} ({})", provider.name(), provider.id());

        for capability in provider.capabilities() {
            if capability.features().is_empty() {
                println!("    {}@{}", capability.interface(), capability.version());
            } else {
                println!(
                    "    {}@{} [{}]",
                    capability.interface(),
                    capability.version(),
                    capability.features().join(", ")
                );
            }
        }
    }
}
