#![forbid(unsafe_code)]

//! Presolve command-line interface.

mod application;
mod build;
mod check;
mod cli;
mod dev;
mod environment;
mod inspect;
mod json_output;
mod plan;
mod provider;
mod release_inspect;
mod typescript;

use std::{env, path::Path};

use anyhow::{Context, Result, bail};
use clap::Parser;
use presolve_environment::inventory_from_specification;
use presolve_resolver::ResolutionProblem;

use crate::{
    build::{BuildArtifactOutput, build_release},
    check::{CompatibilityCheck, check_compatibility, check_release_compatibility},
    cli::{
        Cli, EnvironmentCommand, EnvironmentProviderCommand, InspectArgs, RootCommand,
        StructuredOutputArgs,
    },
    dev::run_local_dev,
    environment::{create_environment, load_environment},
    inspect::inspect_application,
    json_output::{
        render_application_json, render_check_json, render_plan_json, render_release_json,
    },
    plan::render_deployment_plan,
    provider::{add_provider, expand_provider_supply, list_providers, remove_provider},
    release_inspect::{load_release, render_release},
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
        RootCommand::Build(args) => execute_build(&args, current_directory),
        RootCommand::Inspect(args) => execute_inspect(&args, current_directory),
        RootCommand::Check(args) => execute_check(&args, current_directory),
        RootCommand::Plan(args) => execute_plan(&args, current_directory),
        RootCommand::Dev(args) => execute_dev(&args, current_directory),
        RootCommand::Env { command } => execute_environment(command, current_directory),
    }
}

fn execute_build(args: &cli::BuildArgs, current_directory: &Path) -> Result<()> {
    let output = build_release(
        current_directory,
        args.component.as_deref(),
        args.typescript_entry.as_deref(),
        &args.component_for,
        &args.oci_image_for,
        args.output.as_deref(),
    )?;

    println!("Presolve build");
    println!("  application: {} {}", output.application, output.version);
    println!("  artifacts:");

    for artifact in &output.artifacts {
        match artifact {
            BuildArtifactOutput::Component { workload, path } => {
                println!("    {workload} -> component {}", path.display());
            }
            BuildArtifactOutput::OciImage {
                workload,
                media_type,
                digest,
                size,
            } => {
                println!("    {workload} -> OCI {media_type} {digest} ({size} bytes)");
            }
        }
    }

    println!("  release: {}", output.release.display());
    println!("  digest: {}", output.release_digest);
    println!("  size: {} bytes", output.encoded_size);

    Ok(())
}

fn execute_inspect(args: &InspectArgs, current_directory: &Path) -> Result<()> {
    if let Some(release) = &args.release {
        let document = load_release(current_directory, release)?;

        if args.json {
            let rendered = render_release_json(&document)?;
            println!("{rendered}");
        } else {
            let rendered = render_release(&document)?;
            print!("{rendered}");
        }

        return Ok(());
    }

    if args.json {
        let rendered = render_application_json(current_directory)?;
        println!("{rendered}");
    } else {
        let rendered = inspect_application(current_directory)?;
        print!("{rendered}");
    }

    Ok(())
}

fn execute_check(args: &StructuredOutputArgs, current_directory: &Path) -> Result<()> {
    let check = compatibility_check_for_args(args, current_directory)?;

    if args.json {
        let rendered = render_check_json(&check)?;
        println!("{rendered}");
    } else {
        print_compatibility_check(&check);
    }

    if !check.report.is_resolved() {
        bail!(
            "application `{}` is not compatible with environment `{}`",
            check.application.contract.application().name(),
            check.environment.specification.environment().name()
        );
    }

    Ok(())
}

fn execute_plan(args: &StructuredOutputArgs, current_directory: &Path) -> Result<()> {
    let check = compatibility_check_for_args(args, current_directory)?;

    if args.json {
        let rendered = render_plan_json(&check)?;
        println!("{rendered}");
    } else if check.report.is_resolved() {
        let rendered = render_deployment_plan(&check)?;
        print!("{rendered}");
    } else {
        print_compatibility_check(&check);
    }

    if !check.report.is_resolved() {
        bail!(
            "application `{}` has no DeploymentPlan in environment `{}`",
            check.application.contract.application().name(),
            check.environment.specification.environment().name()
        );
    }

    Ok(())
}

fn compatibility_check_for_args(
    args: &StructuredOutputArgs,
    current_directory: &Path,
) -> Result<CompatibilityCheck> {
    match args.release.as_deref() {
        Some(release) => check_release_compatibility(current_directory, release),
        None => check_compatibility(current_directory),
    }
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

        EnvironmentCommand::Check => {
            let document = load_environment(current_directory)?;
            inventory_from_specification(&document.specification).with_context(|| {
                format!(
                    "Environment Specification `{}` cannot be normalized for resolution",
                    document.path.display()
                )
            })?;
            let environment = document.specification.environment();

            println!("Environment Specification is valid");
            println!(
                "  environment: {} ({})",
                environment.name(),
                environment.id()
            );
            println!("  file: {}", document.path.display());
            println!("  providers: {}", document.specification.providers().len());

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
    println!(
        "  {}: {}",
        check.application.source_label(),
        check.application.path.display()
    );
    if let Some(digest) = check.application.release_digest() {
        println!("  release digest: {digest}");
    }
    println!("Environment: {}", environment.name());
    println!("  specification: {}", check.environment.path.display());

    if let Some(plan) = check.report.plan() {
        println!("Status: compatible");
        println!("  workloads:");

        for workload in plan.workloads() {
            println!("    {} -> {}", workload.name(), workload.kind());
        }

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
