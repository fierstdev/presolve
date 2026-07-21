//! L10 transport-neutral tooling schema registry and negotiation.
//!
//! This module is intentionally independent from L3-L8 execution and durable
//! products. It only names already-published schemas and rejects unavailable
//! future tooling products.

#![allow(clippy::missing_errors_doc, clippy::missing_panics_doc)]

use std::fmt;

pub const TOOLING_SCHEMA_NEGOTIATION_V1_SCHEMA: &str = "presolve.tooling-schema-negotiation";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ToolingSchemaAvailabilityV1 {
    Available,
    Reserved,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ToolingSchemaEntryV1 {
    pub schema: &'static str,
    pub version: u32,
    pub availability: ToolingSchemaAvailabilityV1,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ToolingSchemaNegotiationRequestV1 {
    pub schema: String,
    pub versions: Vec<u32>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ToolingSchemaNegotiationResponseV1 {
    pub schema: String,
    pub accepted_version: u32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ToolingSchemaNegotiationErrorV1 {
    pub code: &'static str,
    pub message: String,
}

impl fmt::Display for ToolingSchemaNegotiationErrorV1 {
    fn fmt(&self, output: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(output, "{}: {}", self.code, self.message)
    }
}

impl std::error::Error for ToolingSchemaNegotiationErrorV1 {}

#[must_use]
pub const fn tooling_schema_registry_v1() -> &'static [ToolingSchemaEntryV1] {
    &[
        ToolingSchemaEntryV1 {
            schema: "presolve.workspace-configuration",
            version: 1,
            availability: ToolingSchemaAvailabilityV1::Available,
        },
        ToolingSchemaEntryV1 {
            schema: "presolve.workspace-snapshot",
            version: 1,
            availability: ToolingSchemaAvailabilityV1::Available,
        },
        ToolingSchemaEntryV1 {
            schema: "presolve.workspace-graph",
            version: 1,
            availability: ToolingSchemaAvailabilityV1::Available,
        },
        ToolingSchemaEntryV1 {
            schema: "presolve.compiler-service-protocol",
            version: 1,
            availability: ToolingSchemaAvailabilityV1::Available,
        },
        ToolingSchemaEntryV1 {
            schema: "presolve.persistent-artifact-cache",
            version: 1,
            availability: ToolingSchemaAvailabilityV1::Available,
        },
        ToolingSchemaEntryV1 {
            schema: "presolve.cache-inspection-report.v1",
            version: 1,
            availability: ToolingSchemaAvailabilityV1::Available,
        },
        ToolingSchemaEntryV1 {
            schema: "presolve.workspace-manifest",
            version: 1,
            availability: ToolingSchemaAvailabilityV1::Available,
        },
        ToolingSchemaEntryV1 {
            schema: "presolve.watch-session-configuration",
            version: 1,
            availability: ToolingSchemaAvailabilityV1::Available,
        },
        ToolingSchemaEntryV1 {
            schema: "presolve.watch-change-batch",
            version: 1,
            availability: ToolingSchemaAvailabilityV1::Available,
        },
        ToolingSchemaEntryV1 {
            schema: "presolve.build-trace",
            version: 1,
            availability: ToolingSchemaAvailabilityV1::Reserved,
        },
        ToolingSchemaEntryV1 {
            schema: "presolve.compile-cost-report",
            version: 1,
            availability: ToolingSchemaAvailabilityV1::Reserved,
        },
        ToolingSchemaEntryV1 {
            schema: "presolve.artifact-graph",
            version: 1,
            availability: ToolingSchemaAvailabilityV1::Reserved,
        },
    ]
}

pub fn decode_tooling_schema_negotiation_request_v1(
    bytes: &[u8],
) -> Result<ToolingSchemaNegotiationRequestV1, ToolingSchemaNegotiationErrorV1> {
    let value: serde_json::Value =
        serde_json::from_slice(bytes).map_err(|error| ToolingSchemaNegotiationErrorV1 {
            code: "L10S001_INVALID_NEGOTIATION_JSON",
            message: error.to_string(),
        })?;
    let object = value.as_object().ok_or(ToolingSchemaNegotiationErrorV1 {
        code: "L10S002_INVALID_NEGOTIATION_SHAPE",
        message: "request must be an object".into(),
    })?;
    if object.len() != 2 || !object.contains_key("schema") || !object.contains_key("versions") {
        return Err(ToolingSchemaNegotiationErrorV1 {
            code: "L10S002_INVALID_NEGOTIATION_SHAPE",
            message: "request contains unknown or missing fields".into(),
        });
    }
    let schema = object
        .get("schema")
        .and_then(serde_json::Value::as_str)
        .filter(|value| !value.is_empty())
        .ok_or(ToolingSchemaNegotiationErrorV1 {
            code: "L10S002_INVALID_NEGOTIATION_SHAPE",
            message: "schema must be a non-empty string".into(),
        })?
        .to_owned();
    let versions = object
        .get("versions")
        .and_then(serde_json::Value::as_array)
        .ok_or(ToolingSchemaNegotiationErrorV1 {
            code: "L10S002_INVALID_NEGOTIATION_SHAPE",
            message: "versions must be an array".into(),
        })?
        .iter()
        .map(|value| {
            value
                .as_u64()
                .and_then(|value| u32::try_from(value).ok())
                .filter(|value| *value > 0)
                .ok_or(ToolingSchemaNegotiationErrorV1 {
                    code: "L10S003_INVALID_VERSION_LIST",
                    message: "versions must contain positive u32 values".into(),
                })
        })
        .collect::<Result<Vec<_>, _>>()?;
    if versions.is_empty() || versions.windows(2).any(|pair| pair[0] <= pair[1]) {
        return Err(ToolingSchemaNegotiationErrorV1 {
            code: "L10S003_INVALID_VERSION_LIST",
            message: "versions must be unique descending values".into(),
        });
    }
    Ok(ToolingSchemaNegotiationRequestV1 { schema, versions })
}

pub fn negotiate_tooling_schema_v1(
    request: &ToolingSchemaNegotiationRequestV1,
) -> Result<ToolingSchemaNegotiationResponseV1, ToolingSchemaNegotiationErrorV1> {
    let entry = tooling_schema_registry_v1()
        .iter()
        .find(|entry| entry.schema == request.schema)
        .ok_or(ToolingSchemaNegotiationErrorV1 {
            code: "L10S004_UNKNOWN_SCHEMA",
            message: "schema is not registered".into(),
        })?;
    if entry.availability == ToolingSchemaAvailabilityV1::Reserved {
        return Err(ToolingSchemaNegotiationErrorV1 {
            code: "L10S005_RESERVED_SCHEMA",
            message: "schema has no canonical producer".into(),
        });
    }
    if !request.versions.contains(&entry.version) {
        return Err(ToolingSchemaNegotiationErrorV1 {
            code: "L10S006_NO_SHARED_VERSION",
            message: "no supported version was requested".into(),
        });
    }
    Ok(ToolingSchemaNegotiationResponseV1 {
        schema: request.schema.clone(),
        accepted_version: entry.version,
    })
}

#[must_use]
pub fn encode_tooling_schema_negotiation_response_v1(
    response: &ToolingSchemaNegotiationResponseV1,
) -> Vec<u8> {
    format!(
        "{{\"schema\":\"{}\",\"version\":{}}}\n",
        response.schema, response.accepted_version
    )
    .into_bytes()
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn l10_registry_is_deterministic_and_reserved_entries_reject() {
        assert_eq!(tooling_schema_registry_v1(), tooling_schema_registry_v1());
        let request = decode_tooling_schema_negotiation_request_v1(
            br#"{"schema":"presolve.build-trace","versions":[1]}"#,
        )
        .unwrap();
        assert_eq!(
            negotiate_tooling_schema_v1(&request).unwrap_err().code,
            "L10S005_RESERVED_SCHEMA"
        );
    }
    #[test]
    fn l10_negotiation_validates_shape_versions_and_compatibility() {
        let request = decode_tooling_schema_negotiation_request_v1(
            br#"{"schema":"presolve.workspace-graph","versions":[2,1]}"#,
        )
        .unwrap();
        let response = negotiate_tooling_schema_v1(&request).unwrap();
        assert_eq!(
            encode_tooling_schema_negotiation_response_v1(&response),
            b"{\"schema\":\"presolve.workspace-graph\",\"version\":1}\n"
        );
        for bytes in [
            br"{}".as_slice(),
            br#"{"schema":"x","versions":[]}"#.as_slice(),
            br#"{"schema":"x","versions":[1,1]}"#.as_slice(),
        ] {
            assert!(decode_tooling_schema_negotiation_request_v1(bytes).is_err());
        }
        let unknown =
            decode_tooling_schema_negotiation_request_v1(br#"{"schema":"x","versions":[1]}"#)
                .unwrap();
        assert_eq!(
            negotiate_tooling_schema_v1(&unknown).unwrap_err().code,
            "L10S004_UNKNOWN_SCHEMA"
        );
    }
}
