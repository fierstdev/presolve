use std::path::PathBuf;

use clap::{ArgGroup, Args, Parser, Subcommand, ValueEnum};

pub(crate) const DEFAULT_LOCAL_MEMORY_MIB: u64 = 1_024;
pub(crate) const DEFAULT_LOCAL_CPU_MILLIS: u32 = 1_000;

#[derive(Debug, Parser)]
#[command(
    name = "presolve",
    version,
    about = "Build software once. Decide where it runs later."
)]
pub(crate) struct Cli {
    #[command(subcommand)]
    pub(crate) command: RootCommand,
}

#[derive(Debug, Subcommand)]
pub(crate) enum RootCommand {
    /// Build the nearest application into an immutable `.presolved` release.
    Build(BuildArgs),

    /// Check whether the nearest application can run in the nearest environment.
    Check,

    /// Build and run the nearest application in the local environment.
    Dev(DevArgs),

    /// Create, inspect, and manage Presolve environments.
    #[command(visible_alias = "environment")]
    Env {
        #[command(subcommand)]
        command: EnvironmentCommand,
    },
}

#[derive(Debug, Subcommand)]
pub(crate) enum EnvironmentCommand {
    /// Create a new local Environment Specification.
    Init(EnvironmentInitArgs),

    /// Validate the nearest Environment Specification.
    Check,

    /// Show the nearest Environment Specification.
    Show,

    /// Manage providers declared by the nearest Environment Specification.
    Provider {
        #[command(subcommand)]
        command: EnvironmentProviderCommand,
    },
}

#[derive(Debug, Subcommand)]
pub(crate) enum EnvironmentProviderCommand {
    /// Add a provider declaration with a generated stable `ProviderId`.
    Add(ProviderAddArgs),

    /// List provider declarations.
    List,

    /// Remove a provider declaration by name.
    Remove(ProviderRemoveArgs),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub(crate) enum BuiltinCapabilityAlias {
    /// Presolve key-value storage.
    Kv,

    /// Presolve object storage.
    Objects,
}

impl BuiltinCapabilityAlias {
    pub(crate) fn parse(value: &str) -> Option<Self> {
        match value {
            "kv" => Some(Self::Kv),
            "objects" => Some(Self::Objects),
            _ => None,
        }
    }
}

#[derive(Debug, Args)]
pub(crate) struct BuildArgs {
    /// Prebuilt Component for a single-workload application.
    #[arg(
        long,
        value_name = "PATH",
        conflicts_with_all = ["component_for", "oci_image_for"]
    )]
    pub(crate) component: Option<PathBuf>,

    /// Map a declared Component workload to a prebuilt artifact. Repeat per workload.
    #[arg(
        long = "component-for",
        value_name = "WORKLOAD=PATH",
        conflicts_with = "component"
    )]
    pub(crate) component_for: Vec<String>,

    /// Map a container workload to immutable OCI descriptor identity. Repeat per workload.
    #[arg(
        long = "oci-image-for",
        value_name = "WORKLOAD=TYPE,DIGEST,SIZE",
        conflicts_with = "component"
    )]
    pub(crate) oci_image_for: Vec<String>,

    /// Output `.presolved` path. Defaults to `dist/<application>.presolved`.
    #[arg(long, value_name = "PATH")]
    pub(crate) output: Option<PathBuf>,
}

#[derive(Debug, Args)]
pub(crate) struct DevArgs {
    /// Prebuilt WebAssembly Component to execute instead of building a Rust application.
    #[arg(long, value_name = "PATH")]
    pub(crate) component: Option<PathBuf>,

    /// String passed to the application's current `run` entry point.
    #[arg(long, value_name = "TEXT", default_value = "")]
    pub(crate) input: String,
}

#[derive(Debug, Args)]
pub(crate) struct EnvironmentInitArgs {
    /// Human-readable environment name.
    #[arg(value_name = "NAME", default_value = "local")]
    pub(crate) name: String,

    /// Allocatable memory budget exposed to the resolver.
    #[arg(long, default_value_t = DEFAULT_LOCAL_MEMORY_MIB)]
    pub(crate) memory_mib: u64,

    /// Allocatable CPU budget in millicores exposed to the resolver.
    #[arg(long, default_value_t = DEFAULT_LOCAL_CPU_MILLIS)]
    pub(crate) cpu_millis: u32,
}

#[derive(Debug, Args)]
#[command(group(
    ArgGroup::new("provider-supply")
        .required(true)
        .args(["capabilities", "provides"])
))]
pub(crate) struct ProviderAddArgs {
    /// Human-readable provider name, unique within the environment.
    #[arg(value_name = "NAME")]
    pub(crate) name: String,

    /// Built-in capability alias (`kv` or `objects`). Repeat or comma-separate.
    #[arg(long = "provides", value_name = "CAPABILITY", value_delimiter = ',')]
    pub(crate) provides: Vec<BuiltinCapabilityAlias>,

    /// Built-in features as CAPABILITY=FEATURE[,FEATURE...]. Repeat as needed.
    #[arg(long = "features", value_name = "CAPABILITY=FEATURE[,FEATURE...]")]
    pub(crate) builtin_features: Vec<String>,

    /// Exact supplied capability as INTERFACE@VERSION. Repeat for custom/advanced use.
    #[arg(long = "capability", value_name = "INTERFACE@VERSION")]
    pub(crate) capabilities: Vec<String>,

    /// Exact semantic feature as INTERFACE@VERSION=FEATURE. Repeat for advanced use.
    #[arg(long = "feature", value_name = "INTERFACE@VERSION=FEATURE")]
    pub(crate) features: Vec<String>,
}

#[derive(Debug, Args)]
pub(crate) struct ProviderRemoveArgs {
    /// Human-readable provider name.
    #[arg(value_name = "NAME")]
    pub(crate) name: String,
}

#[cfg(test)]
mod tests {
    use clap::Parser;

    use super::*;

    #[test]
    fn env_init_has_conservative_local_defaults() {
        let cli = Cli::try_parse_from(["presolve", "env", "init"])
            .expect("default environment init command should parse");

        let RootCommand::Env {
            command: EnvironmentCommand::Init(args),
        } = cli.command
        else {
            panic!("expected env init command");
        };

        assert_eq!(args.name, "local");
        assert_eq!(args.memory_mib, DEFAULT_LOCAL_MEMORY_MIB);
        assert_eq!(args.cpu_millis, DEFAULT_LOCAL_CPU_MILLIS);
    }

    #[test]
    fn build_command_defaults_to_source_build_and_default_output() {
        let cli = Cli::try_parse_from(["presolve", "build"]).expect("build command should parse");

        let RootCommand::Build(args) = cli.command else {
            panic!("expected build command");
        };

        assert!(args.component.is_none());
        assert_eq!(args.component_for, Vec::<String>::new());
        assert_eq!(args.oci_image_for, Vec::<String>::new());
        assert!(args.output.is_none());
    }

    #[test]
    fn build_command_accepts_component_and_output() {
        let cli = Cli::try_parse_from([
            "presolve",
            "build",
            "--component",
            "build/app.wasm",
            "--output",
            "release/app.presolved",
        ])
        .expect("build command options should parse");

        let RootCommand::Build(args) = cli.command else {
            panic!("expected build command");
        };

        assert_eq!(args.component, Some(PathBuf::from("build/app.wasm")));
        assert_eq!(args.output, Some(PathBuf::from("release/app.presolved")));
    }

    #[test]
    fn build_command_accepts_explicit_workload_component_mappings() {
        let cli = Cli::try_parse_from([
            "presolve",
            "build",
            "--component-for",
            "api=build/api.wasm",
            "--component-for",
            "worker=build/worker.wasm",
        ])
        .expect("multi-workload build command should parse");

        let RootCommand::Build(args) = cli.command else {
            panic!("expected build command");
        };

        assert!(args.component.is_none());
        assert_eq!(
            args.component_for,
            ["api=build/api.wasm", "worker=build/worker.wasm"]
        );
    }

    #[test]
    fn build_command_accepts_oci_image_mapping() {
        let cli = Cli::try_parse_from([
            "presolve",
            "build",
            "--oci-image-for",
            "worker=manifest,sha256:0123456789abcdef,1024",
        ])
        .expect("OCI workload mapping should parse");

        let RootCommand::Build(args) = cli.command else {
            panic!("expected build command");
        };

        assert_eq!(
            args.oci_image_for,
            ["worker=manifest,sha256:0123456789abcdef,1024"]
        );
    }

    #[test]
    fn build_command_accepts_mixed_artifact_mappings() {
        let cli = Cli::try_parse_from([
            "presolve",
            "build",
            "--component-for",
            "api=build/api.wasm",
            "--oci-image-for",
            "worker=index,sha256:0123456789abcdef,2048",
        ])
        .expect("mixed workload mappings should parse");

        let RootCommand::Build(args) = cli.command else {
            panic!("expected build command");
        };

        assert_eq!(args.component_for, ["api=build/api.wasm"]);
        assert_eq!(
            args.oci_image_for,
            ["worker=index,sha256:0123456789abcdef,2048"]
        );
    }

    #[test]
    fn build_command_rejects_component_and_component_for_together() {
        let error = Cli::try_parse_from([
            "presolve",
            "build",
            "--component",
            "build/app.wasm",
            "--component-for",
            "api=build/api.wasm",
        ])
        .expect_err("component input forms should conflict");

        assert_eq!(error.kind(), clap::error::ErrorKind::ArgumentConflict);

        let error = Cli::try_parse_from([
            "presolve",
            "build",
            "--component",
            "build/app.wasm",
            "--oci-image-for",
            "worker=manifest,sha256:0123456789abcdef,1024",
        ])
        .expect_err("single Component shorthand should conflict with OCI mappings");

        assert_eq!(error.kind(), clap::error::ErrorKind::ArgumentConflict);
    }

    #[test]
    fn check_command_parses() {
        let cli = Cli::try_parse_from(["presolve", "check"]).expect("check command should parse");

        assert!(matches!(cli.command, RootCommand::Check));
    }

    #[test]
    fn dev_command_has_empty_input_by_default() {
        let cli = Cli::try_parse_from(["presolve", "dev"]).expect("dev command should parse");

        let RootCommand::Dev(args) = cli.command else {
            panic!("expected dev command");
        };

        assert!(args.component.is_none());
        assert_eq!(args.input, "");
    }

    #[test]
    fn dev_command_accepts_component_and_input() {
        let cli = Cli::try_parse_from([
            "presolve",
            "dev",
            "--component",
            "build/app.wasm",
            "--input",
            "Austin",
        ])
        .expect("dev command options should parse");

        let RootCommand::Dev(args) = cli.command else {
            panic!("expected dev command");
        };

        assert_eq!(args.component, Some(PathBuf::from("build/app.wasm")));
        assert_eq!(args.input, "Austin");
    }

    #[test]
    fn env_check_command_parses() {
        let cli = Cli::try_parse_from(["presolve", "env", "check"])
            .expect("env check command should parse");

        assert!(matches!(
            cli.command,
            RootCommand::Env {
                command: EnvironmentCommand::Check
            }
        ));
    }

    #[test]
    fn environment_is_a_visible_alias_for_env() {
        let cli = Cli::try_parse_from(["presolve", "environment", "show"])
            .expect("environment alias should parse");

        assert!(matches!(
            cli.command,
            RootCommand::Env {
                command: EnvironmentCommand::Show
            }
        ));
    }

    #[test]
    fn env_init_accepts_explicit_capacity() {
        let cli = Cli::try_parse_from([
            "presolve",
            "env",
            "init",
            "staging",
            "--memory-mib",
            "8192",
            "--cpu-millis",
            "4000",
        ])
        .expect("explicit environment init command should parse");

        let RootCommand::Env {
            command: EnvironmentCommand::Init(args),
        } = cli.command
        else {
            panic!("expected env init command");
        };

        assert_eq!(args.name, "staging");
        assert_eq!(args.memory_mib, 8_192);
        assert_eq!(args.cpu_millis, 4_000);
    }

    #[test]
    fn provider_add_collects_explicit_capabilities_and_features() {
        let cli = Cli::try_parse_from([
            "presolve",
            "env",
            "provider",
            "add",
            "memory-services",
            "--capability",
            "presolve:kv/store@0.1.0",
            "--capability",
            "presolve:objects/store@0.1.0",
            "--feature",
            "presolve:objects/store@0.1.0=range-read",
        ])
        .expect("explicit provider add command should parse");

        let RootCommand::Env {
            command:
                EnvironmentCommand::Provider {
                    command: EnvironmentProviderCommand::Add(args),
                },
        } = cli.command
        else {
            panic!("expected env provider add command");
        };

        assert_eq!(args.name, "memory-services");
        assert_eq!(args.provides, []);
        assert_eq!(
            args.capabilities,
            ["presolve:kv/store@0.1.0", "presolve:objects/store@0.1.0"]
        );
        assert_eq!(args.features, ["presolve:objects/store@0.1.0=range-read"]);
    }

    #[test]
    fn provider_add_collects_builtin_shorthand() {
        let cli = Cli::try_parse_from([
            "presolve",
            "env",
            "provider",
            "add",
            "memory-services",
            "--provides",
            "kv,objects",
            "--features",
            "objects=range-read,user-metadata",
        ])
        .expect("shorthand provider add command should parse");

        let RootCommand::Env {
            command:
                EnvironmentCommand::Provider {
                    command: EnvironmentProviderCommand::Add(args),
                },
        } = cli.command
        else {
            panic!("expected env provider add command");
        };

        assert_eq!(
            args.provides,
            [BuiltinCapabilityAlias::Kv, BuiltinCapabilityAlias::Objects]
        );
        assert_eq!(args.builtin_features, ["objects=range-read,user-metadata"]);
        assert_eq!(args.capabilities, Vec::<String>::new());
        assert_eq!(args.features, Vec::<String>::new());
    }

    #[test]
    fn provider_add_requires_at_least_one_supply_declaration() {
        let result = Cli::try_parse_from(["presolve", "env", "provider", "add", "empty-provider"]);

        assert!(result.is_err());
    }

    #[test]
    fn provider_remove_parses_name() {
        let cli = Cli::try_parse_from(["presolve", "env", "provider", "remove", "memory-services"])
            .expect("provider remove command should parse");

        let RootCommand::Env {
            command:
                EnvironmentCommand::Provider {
                    command: EnvironmentProviderCommand::Remove(args),
                },
        } = cli.command
        else {
            panic!("expected env provider remove command");
        };

        assert_eq!(args.name, "memory-services");
    }
}
