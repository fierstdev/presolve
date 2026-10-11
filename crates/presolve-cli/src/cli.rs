use clap::{Args, Parser, Subcommand};

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

    /// Show the nearest Environment Specification.
    Show,
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
}
