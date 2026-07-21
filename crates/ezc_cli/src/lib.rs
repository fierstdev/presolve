//! Public Presolve CLI support surfaces.

pub mod command_framework;
pub mod configuration_codec;

pub use command_framework::{
    load_explicit_project_envelope_v1, parse_cli_command_v1, CliCommandV1, CliExitCodeV1,
    CliProjectEnvelopeErrorV1, CliProjectEnvelopeV1,
};
pub use configuration_codec::{
    decode_cli_workspace_configuration_bytes_v1, decode_cli_workspace_configuration_v1,
    encode_cli_workspace_configuration_bytes_v1, encode_cli_workspace_configuration_v1,
    CliWorkspaceConfigurationDecodeError, CliWorkspaceConfigurationEncodeError,
};
