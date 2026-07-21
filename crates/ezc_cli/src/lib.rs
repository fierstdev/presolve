//! Public Presolve CLI support surfaces.

pub mod build_check_commands;
pub mod cache_commands;
pub mod command_framework;
pub mod compilation_commands;
pub mod configuration_codec;

pub use build_check_commands::{
    parse_explicit_source_spec_v1, run_explicit_build_or_check_v1, CliBuildCheckErrorV1,
    CliExplicitSourceSpecV1,
};
pub use cache_commands::{
    run_project_cache_operation_v1, CliCacheOperationErrorV1, CliCacheOperationResultV1,
    CliCacheOperationV1,
};
pub use command_framework::{
    load_explicit_project_envelope_v1, parse_cli_command_v1, CliCommandV1, CliExitCodeV1,
    CliProjectEnvelopeErrorV1, CliProjectEnvelopeV1,
};
pub use compilation_commands::{
    compile_complete_candidate_v1, CliCompilationCandidateV1, CliCompilationErrorV1,
    CliCompilationResultV1, CliSourceInputV1,
};
pub use configuration_codec::{
    decode_cli_workspace_configuration_bytes_v1, decode_cli_workspace_configuration_v1,
    encode_cli_workspace_configuration_bytes_v1, encode_cli_workspace_configuration_v1,
    CliWorkspaceConfigurationDecodeError, CliWorkspaceConfigurationEncodeError,
};
