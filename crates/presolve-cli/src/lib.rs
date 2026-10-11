#![forbid(unsafe_code)]

//! Presolve command-line interface.

mod application;
mod check;
mod cli;
mod dev;
mod environment;
mod provider;

use std::{env, path::Path};

use anyhow::{Context, Result, bail};
use clap::Parser;
use presolve_resolver::ResolutionProblem;

use crate::{
    check::{CompatibilityCheck, check_compatibility},
    cli::{Cli, EnvironmentCommand, EnvironmentProviderCommand, RootCommand},
    dev::run_local_dev,
    environment::{create_environment, load_environment},
    provider::{add_provider, expand_provider_supply, list_providers, remove_provider},
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
        RootCommand::Check => execute_check(current_directory),
        RootCommand::Dev(args) => execute_dev(&args, current_directory),
        RootCommand::Env { command } => execute_environment(command, current_directory),
    }
}

fn execute_check(current_directory: &Path) -> Result<()> {
    let check = check_compatibility(current_directory)?;
    print_compatibility_check(&check);

    if !check.report.is_resolved() {
        bail!(
            "application `{}` is not compatible with environment `{}`",
            check.application.contract.application().name(),
            check.environment.specification.environment().name()
        );
    }

    Ok(())
}

fn execute_dev(args: &cli::DevArgs, current_directory: &Path) -> Result<()> {
    let check = check_compatibility(current_directory)?;

    if !check.report.is_resolved() {
        print_compatibility_check(&check);

        bail!(
            "application `{}` is not compatible with environment `{}`",
            check.application.contract.application().name(),
            check.environment.specification.environment().name()
        );
    }

    let application = check.application.contract.application();
    let environment = check.environment.specification.environment();

    println!("Presolve dev");
    println!(
        "  application: {} {}",
        application.name(),
        application.version()
    );
    println!("  environment: {}", environment.name());

    let execution = run_local_dev(
        &check,
        current_directory,
        args.component.as_deref(),
        &args.input,
    )?;

    println!("  workload: {}", execution.workload);
    println!("  component: {}", execution.component.display());
    println!();
    println!("{}", execution.output);

    Ok(())
}

fn execute_environment(command: EnvironmentCommand, current_directory: &Path) -> Result<()> {
    match command {
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

            Ok(())
        }

        EnvironmentCommand::Show => {
            let document = load_environment(current_directory)?;
            print_environment(&document);

            Ok(())
        }

        EnvironmentCommand::Provider { command } => {
            execute_environment_provider(command, current_directory)
        }
    }
}

fn execute_environment_provider(
    command: EnvironmentProviderCommand,
    current_directory: &Path,
) -> Result<()> {
    match command {
        EnvironmentProviderCommand::Add(args) => {
            let (capabilities, features) = expand_provider_supply(
                &args.capabilities,
                &args.features,
                &args.provides,
                &args.builtin_features,
            )?;

            let mutation = add_provider(current_directory, &args.name, &capabilities, &features)?;

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

            Ok(())
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

            Ok(())
        }

        EnvironmentProviderCommand::Remove(args) => {
            let mutation = remove_provider(current_directory, &args.name)?;

            println!("Removed provider `{}`", mutation.provider.name());
            println!("  id: {}", mutation.provider.id());
            println!("  file: {}", mutation.path.display());

            Ok(())
        }
    }
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

fn print_compatibility_check(check: &CompatibilityCheck) {
    let application = check.application.contract.application();
    let environment = check.environment.specification.environment();

    println!(
        "Application: {} {}",
        application.name(),
        application.version()
    );
    println!("  contract: {}", check.application.path.display());
    println!("Environment: {}", environment.name());
    println!("  specification: {}", check.environment.path.display());

    if let Some(plan) = check.report.plan() {
        println!("Status: compatible");

        if plan.bindings().is_empty() {
            println!("  capability bindings: none");
        } else {
            println!("  capability bindings:");

            for binding in plan.bindings() {
                if binding.negotiated_features().is_empty() {
                    println!(
                        "    {} -> {} ({}) @ {}",
                        binding.interface(),
                        binding.provider_name(),
                        binding.provider_id(),
                        binding.provider_version()
                    );
                } else {
                    println!(
                        "    {} -> {} ({}) @ {} [{}]",
                        binding.interface(),
                        binding.provider_name(),
                        binding.provider_id(),
                        binding.provider_version(),
                        binding.negotiated_features().join(", ")
                    );
                }
            }
        }

        if !plan.unbound_optional_capabilities().is_empty() {
            println!(
                "  optional capabilities not bound: {}",
                plan.unbound_optional_capabilities().join(", ")
            );
        }

        return;
    }

    println!("Status: incompatible");

    for problem in check.report.problems() {
        print_resolution_problem(problem);
    }
}

fn print_resolution_problem(problem: &ResolutionProblem) {
    match problem {
        ResolutionProblem::UnsupportedWorkload {
            workload,
            kind,
            supported_kinds,
        } => {
            let supported = if supported_kinds.is_empty() {
                "none".to_owned()
            } else {
                supported_kinds
                    .iter()
                    .map(ToString::to_string)
                    .collect::<Vec<_>>()
                    .join(", ")
            };

            println!(
                "{}: workload `{workload}` requires `{kind}` execution; environment supports {supported}",
                problem.code()
            );
            println!("  help: use an environment that supports this workload execution class");
        }

        ResolutionProblem::MissingCapability { interface, version } => {
            println!(
                "{}: missing required capability `{interface}` compatible with `{version}`",
                problem.code()
            );
            println!(
                "  help: add or update an environment provider that supplies a compatible exact version"
            );
        }

        ResolutionProblem::MissingCapabilityFeatures {
            interface,
            version,
            required_features,
            available_features,
        } => {
            let available = if available_features.is_empty() {
                "none".to_owned()
            } else {
                available_features.join(", ")
            };

            println!(
                "{}: capability `{interface}` compatible with `{version}` is missing required features [{}]",
                problem.code(),
                required_features.join(", ")
            );
            println!("  available features: {available}");
            println!("  help: use a provider that advertises every required semantic feature");
        }

        ResolutionProblem::InsufficientMemory {
            requested_mib,
            available_mib,
        } => {
            println!(
                "{}: application requires {requested_mib} MiB memory; environment provides {available_mib} MiB",
                problem.code()
            );
            println!(
                "  help: increase the environment memory capacity or reduce the application requirement"
            );
        }

        ResolutionProblem::InsufficientCpu {
            requested_millis,
            available_millis,
        } => {
            println!(
                "{}: application requires {requested_millis}m CPU; environment provides {available_millis}m",
                problem.code()
            );
            println!(
                "  help: increase the environment CPU capacity or reduce the application requirement"
            );
        }
    }
}
