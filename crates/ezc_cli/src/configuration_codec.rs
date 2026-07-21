//! Strict L9 public `presolve.json` configuration codec.
//!
//! This adapter is deliberately separate from L3's frozen durable serializer.
#![allow(clippy::missing_errors_doc, clippy::missing_panics_doc)]

use std::collections::BTreeSet;
use std::fmt;

use ezc_core::platform::{
    validate_workspace_configuration_v1, WorkspaceConfiguration, WorkspacePath,
};
use serde::de::{self, Deserialize, Deserializer, MapAccess, SeqAccess, Visitor};
use serde_json::Value;

const FIELDS: [&str; 4] = [
    "source_roots",
    "feature_flags",
    "target_profile",
    "platform_options",
];

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CliWorkspaceConfigurationDecodeError {
    pub code: &'static str,
    pub pointer: String,
    pub message: String,
}
impl fmt::Display for CliWorkspaceConfigurationDecodeError {
    fn fmt(&self, output: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            output,
            "{} at {}: {}",
            self.code, self.pointer, self.message
        )
    }
}
impl std::error::Error for CliWorkspaceConfigurationDecodeError {}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CliWorkspaceConfigurationEncodeError {
    pub code: &'static str,
    pub message: String,
}
impl fmt::Display for CliWorkspaceConfigurationEncodeError {
    fn fmt(&self, output: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(output, "{}: {}", self.code, self.message)
    }
}
impl std::error::Error for CliWorkspaceConfigurationEncodeError {}

fn decode_error(
    code: &'static str,
    pointer: impl Into<String>,
    message: impl Into<String>,
) -> CliWorkspaceConfigurationDecodeError {
    CliWorkspaceConfigurationDecodeError {
        code,
        pointer: pointer.into(),
        message: message.into(),
    }
}

fn valid_scalar(value: &str) -> bool {
    !value.trim().is_empty() && !value.bytes().any(|byte| byte.is_ascii_control())
}

fn strings(
    value: &Value,
    pointer: &str,
) -> Result<Vec<String>, CliWorkspaceConfigurationDecodeError> {
    let values = value.as_array().ok_or_else(|| {
        decode_error(
            "L9C005_INVALID_FIELD_TYPE",
            pointer,
            "expected an array of strings",
        )
    })?;
    values
        .iter()
        .enumerate()
        .map(|(index, entry)| {
            entry.as_str().map(str::to_owned).ok_or_else(|| {
                decode_error(
                    "L9C005_INVALID_FIELD_TYPE",
                    format!("{pointer}/{index}"),
                    "expected a string",
                )
            })
        })
        .collect()
}

/// Decodes the strict, public L9 configuration representation into the
/// existing L3 Rust semantic product. It neither reads paths nor calls an L3
/// JSON decoder (none is introduced by L9).
#[allow(clippy::too_many_lines)]
pub fn decode_cli_workspace_configuration_v1(
    value: &Value,
) -> Result<WorkspaceConfiguration, CliWorkspaceConfigurationDecodeError> {
    let object = value
        .as_object()
        .ok_or_else(|| decode_error("L9C002_EXPECTED_OBJECT", "", "expected a JSON object"))?;
    for key in object.keys() {
        if !FIELDS.contains(&key.as_str()) {
            return Err(decode_error(
                "L9C003_UNKNOWN_FIELD",
                format!("/{key}"),
                "unknown public CLI configuration field",
            ));
        }
    }
    for field in FIELDS {
        if !object.contains_key(field) {
            return Err(decode_error(
                "L9C004_MISSING_FIELD",
                format!("/{field}"),
                "required public CLI configuration field is missing",
            ));
        }
    }

    let roots = strings(&object["source_roots"], "/source_roots")?;
    if roots.is_empty() {
        return Err(decode_error(
            "L9C006_INVALID_SOURCE_ROOT",
            "/source_roots",
            "at least one source root is required",
        ));
    }
    let mut source_roots = Vec::with_capacity(roots.len());
    let mut root_set = BTreeSet::new();
    for (index, root) in roots.into_iter().enumerate() {
        let root = WorkspacePath::new(&root).map_err(|_| {
            decode_error(
                "L9C006_INVALID_SOURCE_ROOT",
                format!("/source_roots/{index}"),
                "source root must be a normalized relative workspace path",
            )
        })?;
        if !root_set.insert(root.clone()) {
            return Err(decode_error(
                "L9C007_DUPLICATE_SOURCE_ROOT",
                format!("/source_roots/{index}"),
                "source roots must be unique",
            ));
        }
        source_roots.push(root);
    }

    let feature_flags = strings(&object["feature_flags"], "/feature_flags")?;
    if feature_flags.iter().any(|flag| !valid_scalar(flag)) {
        return Err(decode_error(
            "L9C008_INVALID_FEATURE_FLAG",
            "/feature_flags",
            "feature flags must be non-empty printable strings",
        ));
    }
    if feature_flags.windows(2).any(|pair| pair[0] >= pair[1]) {
        return Err(decode_error(
            "L9C009_DUPLICATE_OR_NONCANONICAL_FEATURE_FLAG",
            "/feature_flags",
            "feature flags must be unique and lexicographically ordered",
        ));
    }

    let target_profile = object["target_profile"].as_str().ok_or_else(|| {
        decode_error(
            "L9C005_INVALID_FIELD_TYPE",
            "/target_profile",
            "expected a string",
        )
    })?;
    if !matches!(target_profile, "default" | "development" | "production") {
        return Err(decode_error(
            "L9C010_INVALID_TARGET_PROFILE",
            "/target_profile",
            "target profile must be default, development, or production",
        ));
    }

    let options = object["platform_options"].as_array().ok_or_else(|| {
        decode_error(
            "L9C005_INVALID_FIELD_TYPE",
            "/platform_options",
            "expected an array of [key, value] tuples",
        )
    })?;
    let mut platform_options = Vec::with_capacity(options.len());
    let mut option_keys = BTreeSet::new();
    for (index, option) in options.iter().enumerate() {
        let tuple = option.as_array().ok_or_else(|| {
            decode_error(
                "L9C011_INVALID_OPTION_TUPLE",
                format!("/platform_options/{index}"),
                "platform options must be [key, value] tuples",
            )
        })?;
        if tuple.len() != 2 {
            return Err(decode_error(
                "L9C011_INVALID_OPTION_TUPLE",
                format!("/platform_options/{index}"),
                "platform options must contain exactly two strings",
            ));
        }
        let key = tuple[0]
            .as_str()
            .filter(|key| valid_scalar(key))
            .ok_or_else(|| {
                decode_error(
                    "L9C012_INVALID_OPTION_KEY",
                    format!("/platform_options/{index}/0"),
                    "platform option key must be a non-empty printable string",
                )
            })?;
        let option_value = tuple[1]
            .as_str()
            .filter(|option_value| valid_scalar(option_value))
            .ok_or_else(|| {
                decode_error(
                    "L9C013_INVALID_OPTION_VALUE",
                    format!("/platform_options/{index}/1"),
                    "platform option value must be a non-empty printable string",
                )
            })?;
        if !option_keys.insert(key.to_owned()) {
            return Err(decode_error(
                "L9C014_DUPLICATE_OPTION_KEY",
                format!("/platform_options/{index}/0"),
                "platform option keys must be unique",
            ));
        }
        platform_options.push((key.to_owned(), option_value.to_owned()));
    }
    if platform_options.windows(2).any(|pair| pair[0] >= pair[1]) {
        return Err(decode_error(
            "L9C015_NONCANONICAL_OPTION_ORDER",
            "/platform_options",
            "platform options must be lexicographically ordered",
        ));
    }

    let configuration = WorkspaceConfiguration {
        source_roots,
        feature_flags,
        target_profile: target_profile.to_owned(),
        platform_options,
    };
    validate_workspace_configuration_v1(&configuration)
        .map_err(|error| decode_error("L9C016_L3_VALIDATION_FAILED", "", error.message))?;
    Ok(configuration)
}

/// Strict byte decoding additionally rejects duplicate object keys before a
/// `serde_json::Value` can erase their evidence.
pub fn decode_cli_workspace_configuration_bytes_v1(
    bytes: &[u8],
) -> Result<WorkspaceConfiguration, CliWorkspaceConfigurationDecodeError> {
    let StrictValue(value) = serde_json::from_slice(bytes).map_err(|error| {
        let code = if error.to_string().contains("duplicate object key") {
            "L9C017_DUPLICATE_OBJECT_KEY"
        } else {
            "L9C001_INVALID_JSON"
        };
        decode_error(code, "", error.to_string())
    })?;
    decode_cli_workspace_configuration_v1(&value)
}

/// Encodes a normalized public CLI configuration. It calls existing L3
/// validation but does not use the L3 serializer as an authoring format.
pub fn encode_cli_workspace_configuration_v1(
    configuration: &WorkspaceConfiguration,
) -> Result<Value, CliWorkspaceConfigurationEncodeError> {
    let configuration = normalize_for_cli(configuration)?;
    Ok(serde_json::json!({
        "source_roots": configuration.source_roots.iter().map(ToString::to_string).collect::<Vec<_>>(),
        "feature_flags": configuration.feature_flags,
        "target_profile": configuration.target_profile,
        "platform_options": configuration.platform_options.into_iter().map(|(key, value)| vec![key, value]).collect::<Vec<_>>(),
    }))
}

/// Returns canonical public CLI JSON bytes in the L9 field order.
pub fn encode_cli_workspace_configuration_bytes_v1(
    configuration: &WorkspaceConfiguration,
) -> Result<Vec<u8>, CliWorkspaceConfigurationEncodeError> {
    let configuration = normalize_for_cli(configuration)?;
    let quote = |value: &str| serde_json::to_string(value).expect("strings serialize");
    let roots = configuration
        .source_roots
        .iter()
        .map(ToString::to_string)
        .map(|root| quote(&root))
        .collect::<Vec<_>>()
        .join(",");
    let flags = configuration
        .feature_flags
        .iter()
        .map(|flag| quote(flag))
        .collect::<Vec<_>>()
        .join(",");
    let options = configuration
        .platform_options
        .iter()
        .map(|(key, value)| format!("[{},{}]", quote(key), quote(value)))
        .collect::<Vec<_>>()
        .join(",");
    Ok(format!("{{\"source_roots\":[{roots}],\"feature_flags\":[{flags}],\"target_profile\":{},\"platform_options\":[{options}]}}\n", quote(&configuration.target_profile)).into_bytes())
}

fn normalize_for_cli(
    configuration: &WorkspaceConfiguration,
) -> Result<WorkspaceConfiguration, CliWorkspaceConfigurationEncodeError> {
    let mut configuration = configuration.clone();
    configuration.feature_flags.sort();
    configuration.feature_flags.dedup();
    configuration.platform_options.sort();
    if configuration
        .platform_options
        .windows(2)
        .any(|pair| pair[0].0 == pair[1].0)
    {
        return Err(CliWorkspaceConfigurationEncodeError {
            code: "L9C114_DUPLICATE_OPTION_KEY",
            message: "platform option keys must be unique".into(),
        });
    }
    if !matches!(
        configuration.target_profile.as_str(),
        "default" | "development" | "production"
    ) {
        return Err(CliWorkspaceConfigurationEncodeError {
            code: "L9C110_INVALID_TARGET_PROFILE",
            message: "target profile must be default, development, or production".into(),
        });
    }
    if configuration
        .feature_flags
        .iter()
        .any(|flag| !valid_scalar(flag))
        || configuration
            .platform_options
            .iter()
            .any(|(key, value)| !valid_scalar(key) || !valid_scalar(value))
    {
        return Err(CliWorkspaceConfigurationEncodeError {
            code: "L9C108_INVALID_PUBLIC_CONFIGURATION",
            message:
                "public feature flags and platform options must be printable non-empty strings"
                    .into(),
        });
    }
    validate_workspace_configuration_v1(&configuration).map_err(|error| {
        CliWorkspaceConfigurationEncodeError {
            code: "L9C116_L3_VALIDATION_FAILED",
            message: error.message,
        }
    })?;
    Ok(configuration)
}

struct StrictValue(Value);
impl<'de> Deserialize<'de> for StrictValue {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        struct StrictVisitor;
        impl<'de> Visitor<'de> for StrictVisitor {
            type Value = StrictValue;
            fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
                formatter.write_str("valid JSON without duplicate object keys")
            }
            fn visit_unit<E: de::Error>(self) -> Result<Self::Value, E> {
                Ok(StrictValue(Value::Null))
            }
            fn visit_none<E: de::Error>(self) -> Result<Self::Value, E> {
                Ok(StrictValue(Value::Null))
            }
            fn visit_bool<E: de::Error>(self, value: bool) -> Result<Self::Value, E> {
                Ok(StrictValue(Value::Bool(value)))
            }
            fn visit_i64<E: de::Error>(self, value: i64) -> Result<Self::Value, E> {
                Ok(StrictValue(Value::Number(value.into())))
            }
            fn visit_u64<E: de::Error>(self, value: u64) -> Result<Self::Value, E> {
                Ok(StrictValue(Value::Number(value.into())))
            }
            fn visit_f64<E: de::Error>(self, value: f64) -> Result<Self::Value, E> {
                serde_json::Number::from_f64(value)
                    .map(|number| StrictValue(Value::Number(number)))
                    .ok_or_else(|| E::custom("invalid JSON number"))
            }
            fn visit_str<E: de::Error>(self, value: &str) -> Result<Self::Value, E> {
                Ok(StrictValue(Value::String(value.into())))
            }
            fn visit_string<E: de::Error>(self, value: String) -> Result<Self::Value, E> {
                Ok(StrictValue(Value::String(value)))
            }
            fn visit_seq<A: SeqAccess<'de>>(
                self,
                mut sequence: A,
            ) -> Result<Self::Value, A::Error> {
                let mut values = Vec::new();
                while let Some(StrictValue(value)) = sequence.next_element()? {
                    values.push(value);
                }
                Ok(StrictValue(Value::Array(values)))
            }
            fn visit_map<A: MapAccess<'de>>(self, mut map: A) -> Result<Self::Value, A::Error> {
                let mut values = serde_json::Map::new();
                while let Some(key) = map.next_key::<String>()? {
                    if values.contains_key(&key) {
                        return Err(de::Error::custom("duplicate object key"));
                    }
                    let StrictValue(value) = map.next_value()?;
                    values.insert(key, value);
                }
                Ok(StrictValue(Value::Object(values)))
            }
        }
        deserializer.deserialize_any(StrictVisitor)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ezc_core::platform::{
        canonical_workspace_configuration_json_v1, workspace_configuration_fingerprint_v1,
    };

    fn minimum() -> WorkspaceConfiguration {
        WorkspaceConfiguration::default()
    }
    fn nonempty() -> WorkspaceConfiguration {
        WorkspaceConfiguration {
            source_roots: vec![
                WorkspacePath::new("app").unwrap(),
                WorkspacePath::new("shared").unwrap(),
            ],
            feature_flags: vec!["alpha".into(), "strict".into()],
            target_profile: "development".into(),
            platform_options: vec![
                ("emit".into(), "full".into()),
                ("optimize".into(), "true".into()),
            ],
        }
    }
    fn round_trip(configuration: &WorkspaceConfiguration) {
        validate_workspace_configuration_v1(configuration).unwrap();
        let encoded = encode_cli_workspace_configuration_v1(configuration).unwrap();
        let decoded = decode_cli_workspace_configuration_v1(&encoded).unwrap();
        assert_eq!(&decoded, configuration);
        assert_eq!(
            workspace_configuration_fingerprint_v1(&decoded).unwrap(),
            workspace_configuration_fingerprint_v1(configuration).unwrap()
        );
    }
    #[test]
    fn l9a_constructed_configurations_round_trip_and_preserve_l3_identity() {
        round_trip(&minimum());
        round_trip(&nonempty());
    }
    #[test]
    fn l9a_frozen_l3_and_distinct_cli_fixtures_match() {
        for (configuration, l3, cli) in [
            (
                minimum(),
                include_bytes!("../fixtures/configuration/minimum-l3-v1.json").as_slice(),
                include_bytes!("../fixtures/configuration/minimum-cli-v1.json").as_slice(),
            ),
            (
                nonempty(),
                include_bytes!("../fixtures/configuration/nonempty-l3-v1.json").as_slice(),
                include_bytes!("../fixtures/configuration/nonempty-cli-v1.json").as_slice(),
            ),
        ] {
            assert_eq!(
                canonical_workspace_configuration_json_v1(&configuration).unwrap(),
                l3
            );
            assert_eq!(
                encode_cli_workspace_configuration_bytes_v1(&configuration).unwrap(),
                cli
            );
            assert_ne!(l3, cli);
        }
    }
    #[test]
    fn l9a_strict_shape_and_duplicate_bytes_are_rejected() {
        for input in [
            r#"{"schema_version":1,"source_roots":["src"],"feature_flags":[],"target_profile":"default","platform_options":[]}"#,
            r#"{"source_roots":["src"],"compiler_flags":[],"target_profile":"default","platform_options":[]}"#,
            r#"{"source_roots":["src"],"feature_flags":[],"target_profile":"default","platform_options":[{"key":"a","value":"b"}]}"#,
            r#"{"source_roots":["src"],"source_roots":["app"],"feature_flags":[],"target_profile":"default","platform_options":[]}"#,
        ] {
            assert!(decode_cli_workspace_configuration_bytes_v1(input.as_bytes()).is_err());
        }
    }
    #[test]
    fn l9a_twenty_shuffled_object_orders_are_equal() {
        let canonical = encode_cli_workspace_configuration_v1(&nonempty()).unwrap();
        for shift in 0..20 {
            let fields = [
                ("source_roots", canonical["source_roots"].to_string()),
                ("feature_flags", canonical["feature_flags"].to_string()),
                ("target_profile", canonical["target_profile"].to_string()),
                (
                    "platform_options",
                    canonical["platform_options"].to_string(),
                ),
            ];
            let bytes = (0..4)
                .map(|index| {
                    let (key, value) = &fields[(index + shift) % 4];
                    format!("\"{key}\":{value}")
                })
                .collect::<Vec<_>>()
                .join(",");
            assert_eq!(
                decode_cli_workspace_configuration_bytes_v1(format!("{{{bytes}}}").as_bytes())
                    .unwrap(),
                nonempty()
            );
        }
    }
}
