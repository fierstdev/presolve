//! Explicit environment-input classification for application publication.
//!
//! This product never reads process state or files. Callers provide a named,
//! already-authorized value map, and only `PRESOLVE_PUBLIC_*` values can enter
//! the browser projection.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

pub const ENVIRONMENT_INPUT_SCHEMA_VERSION: u32 = 1;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct EnvironmentInputManifestV1 {
    pub schema_version: u32,
    pub source_label: String,
    pub browser_values: BTreeMap<String, String>,
    pub server_value_names: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EnvironmentInputErrorV1 {
    pub code: &'static str,
    pub message: String,
}

/// Classifies one explicit environment input map.
///
/// Values whose names begin exactly with `PRESOLVE_PUBLIC_` are browser
/// eligible. Other valid environment names remain server-owned and are
/// represented only by name, never by value, in this publication product.
pub fn build_environment_input_manifest_v1(
    source_label: &str,
    values: &BTreeMap<String, String>,
) -> Result<EnvironmentInputManifestV1, EnvironmentInputErrorV1> {
    if source_label.is_empty() || source_label.contains('\0') || source_label.contains('\n') {
        return Err(EnvironmentInputErrorV1 {
            code: "PSENV1001_SOURCE_LABEL_INVALID",
            message: source_label.into(),
        });
    }
    let mut browser_values = BTreeMap::new();
    let mut server_value_names = Vec::new();
    for (name, value) in values {
        if !is_environment_name(name) {
            return Err(EnvironmentInputErrorV1 {
                code: "PSENV1002_NAME_INVALID",
                message: name.clone(),
            });
        }
        if value.contains('\0') {
            return Err(EnvironmentInputErrorV1 {
                code: "PSENV1003_VALUE_INVALID",
                message: name.clone(),
            });
        }
        if name.starts_with("PRESOLVE_PUBLIC_") && name.len() > "PRESOLVE_PUBLIC_".len() {
            browser_values.insert(name.clone(), value.clone());
        } else {
            server_value_names.push(name.clone());
        }
    }
    Ok(EnvironmentInputManifestV1 {
        schema_version: ENVIRONMENT_INPUT_SCHEMA_VERSION,
        source_label: source_label.into(),
        browser_values,
        server_value_names,
    })
}

#[must_use]
pub fn environment_input_manifest_json_v1(value: &EnvironmentInputManifestV1) -> String {
    serde_json::to_string_pretty(value).expect("environment input manifest serializes") + "\n"
}

/// Decodes a caller-selected manifest without consulting a dotenv file or
/// process state. The decoded value receives the same classification checks as
/// a manifest constructed from explicit values.
pub fn environment_input_manifest_from_json_v1(
    source: &str,
) -> Result<EnvironmentInputManifestV1, EnvironmentInputErrorV1> {
    let manifest = serde_json::from_str::<EnvironmentInputManifestV1>(source).map_err(|error| {
        EnvironmentInputErrorV1 {
            code: "PSENV1007_MANIFEST_JSON_INVALID",
            message: error.to_string(),
        }
    })?;
    validate_environment_input_manifest_v1(&manifest)?;
    Ok(manifest)
}

/// Validates the full immutable manifest shape independently of its transport.
pub fn validate_environment_input_manifest_v1(
    manifest: &EnvironmentInputManifestV1,
) -> Result<(), EnvironmentInputErrorV1> {
    if manifest.schema_version != ENVIRONMENT_INPUT_SCHEMA_VERSION {
        return Err(EnvironmentInputErrorV1 {
            code: "PSENV1008_MANIFEST_SCHEMA_UNSUPPORTED",
            message: manifest.schema_version.to_string(),
        });
    }
    if manifest.source_label.is_empty()
        || manifest.source_label.contains('\0')
        || manifest.source_label.contains('\n')
    {
        return Err(EnvironmentInputErrorV1 {
            code: "PSENV1001_SOURCE_LABEL_INVALID",
            message: manifest.source_label.clone(),
        });
    }
    for (name, value) in &manifest.browser_values {
        if !is_environment_name(name)
            || !name.starts_with("PRESOLVE_PUBLIC_")
            || name.len() == "PRESOLVE_PUBLIC_".len()
            || value.contains('\0')
        {
            return Err(EnvironmentInputErrorV1 {
                code: "PSENV1009_MANIFEST_BROWSER_VALUES_INVALID",
                message: name.clone(),
            });
        }
    }
    let mut previous = None;
    for name in &manifest.server_value_names {
        if !is_environment_name(name)
            || name.starts_with("PRESOLVE_PUBLIC_")
            || manifest.browser_values.contains_key(name)
            || previous.is_some_and(|previous: &String| previous >= name)
        {
            return Err(EnvironmentInputErrorV1 {
                code: "PSENV1010_MANIFEST_SERVER_NAMES_INVALID",
                message: name.clone(),
            });
        }
        previous = Some(name);
    }
    Ok(())
}

fn is_environment_name(value: &str) -> bool {
    value
        .chars()
        .next()
        .is_some_and(|character| character.is_ascii_uppercase() || character == '_')
        && value.chars().all(|character| {
            character.is_ascii_uppercase() || character.is_ascii_digit() || character == '_'
        })
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use super::{
        build_environment_input_manifest_v1, environment_input_manifest_from_json_v1,
        environment_input_manifest_json_v1,
    };

    #[test]
    fn publishes_only_prefixed_values_and_never_server_values() {
        let values = BTreeMap::from([
            ("PRESOLVE_PUBLIC_NAME".into(), "Presolve".into()),
            ("DATABASE_URL".into(), "postgres://secret".into()),
        ]);
        let manifest = build_environment_input_manifest_v1(".env", &values).unwrap();
        assert_eq!(manifest.browser_values["PRESOLVE_PUBLIC_NAME"], "Presolve");
        assert_eq!(manifest.server_value_names, ["DATABASE_URL"]);
        assert!(!environment_input_manifest_json_v1(&manifest).contains("postgres://secret"));
    }

    #[test]
    fn rejects_ambient_or_malformed_name_forms() {
        let values = BTreeMap::from([("process.env.SECRET".into(), "value".into())]);
        assert_eq!(
            build_environment_input_manifest_v1(".env", &values)
                .unwrap_err()
                .code,
            "PSENV1002_NAME_INVALID"
        );
    }

    #[test]
    fn decodes_only_canonical_explicit_manifest_json() {
        let manifest = build_environment_input_manifest_v1(
            ".env.production",
            &BTreeMap::from([
                ("PRESOLVE_PUBLIC_NAME".into(), "Presolve".into()),
                ("DATABASE_URL".into(), "postgres://secret".into()),
            ]),
        )
        .unwrap();
        let decoded =
            environment_input_manifest_from_json_v1(&environment_input_manifest_json_v1(&manifest))
                .unwrap();
        assert_eq!(decoded, manifest);
        assert_eq!(
            environment_input_manifest_from_json_v1(
                r#"{"schemaVersion":1,"sourceLabel":".env","browserValues":{"DATABASE_URL":"secret"},"serverValueNames":[]}"#
            )
            .unwrap_err()
            .code,
            "PSENV1009_MANIFEST_BROWSER_VALUES_INVALID"
        );
    }
}
