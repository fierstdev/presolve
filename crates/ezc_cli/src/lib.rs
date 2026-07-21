//! Public Presolve CLI support surfaces.

pub mod configuration_codec;

pub use configuration_codec::{
    decode_cli_workspace_configuration_bytes_v1, decode_cli_workspace_configuration_v1,
    encode_cli_workspace_configuration_bytes_v1, encode_cli_workspace_configuration_v1,
    CliWorkspaceConfigurationDecodeError, CliWorkspaceConfigurationEncodeError,
};
