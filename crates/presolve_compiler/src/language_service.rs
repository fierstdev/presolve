use crate::{
    decode_tooling_query_snapshot_v1, ToolingQueryDiagnosticV1, ToolingQueryReferenceV1,
    ToolingQuerySemanticRecordV1, ToolingQuerySnapshotV1,
};
use serde::{Deserialize, Serialize};

const REQUEST_SCHEMA_V1: &str = "presolve.language-service-wasm-request";
const RESPONSE_SCHEMA_V1: &str = "presolve.language-service-wasm-response";

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct PositionRequestV1 {
    schema: String,
    version: u32,
    operation: String,
    source_unit_id: String,
    offset: u64,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct SemanticIdRequestV1 {
    schema: String,
    version: u32,
    operation: String,
    query_semantic_id: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct SourceUnitRequestV1 {
    schema: String,
    version: u32,
    operation: String,
    source_unit_id: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct UnsupportedRequestV1 {
    schema: String,
    version: u32,
    operation: String,
}

enum QueryRequestV1 {
    Position(PositionRequestV1),
    Hover(SemanticIdRequestV1),
    Definition(SemanticIdRequestV1),
    References(SemanticIdRequestV1),
    DocumentSymbols(SourceUnitRequestV1),
    Diagnostics(SourceUnitRequestV1),
    Unsupported(UnsupportedRequestV1),
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct ErrorResponseV1<'a> {
    schema: &'static str,
    version: u32,
    operation: &'a str,
    status: &'static str,
    code: &'a str,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct UnsupportedResponseV1<'a> {
    schema: &'static str,
    version: u32,
    operation: &'a str,
    status: &'static str,
    capability: &'a str,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct RecordsResultV1 {
    records: Vec<ToolingQuerySemanticRecordV1>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct RecordResultV1 {
    record: ToolingQuerySemanticRecordV1,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct ReferencesResultV1 {
    references: Vec<ToolingQueryReferenceV1>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct DiagnosticsResultV1 {
    diagnostics: Vec<ToolingQueryDiagnosticV1>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct OkResponseV1<T> {
    schema: &'static str,
    version: u32,
    operation: String,
    status: &'static str,
    result: T,
}

/// Projects one caller-supplied query snapshot through the L12-C WASM protocol.
///
/// This stays crate-private so no native API becomes a competing public delivery
/// authority. The WASM adapter is the only planned external caller.
#[expect(
    clippy::too_many_lines,
    reason = "the five fixed L12 projections are reviewed as one protocol dispatch"
)]
pub(crate) fn query_snapshot_v1(product_bytes: &[u8], request_bytes: &[u8]) -> Vec<u8> {
    let Ok(product) = decode_tooling_query_snapshot_v1(product_bytes) else {
        return error_response("", "invalid_product");
    };
    let request = match decode_request_v1(request_bytes) {
        Ok(request) => request,
        Err(operation) => return error_response(&operation, "invalid_request"),
    };

    match request {
        QueryRequestV1::Position(request) => {
            let Some(unit) = product
                .source_units
                .iter()
                .find(|unit| unit.source_unit_id == request.source_unit_id)
            else {
                return error_response(&request.operation, "unknown_source_unit");
            };
            if request.offset > unit.source_length {
                return error_response(&request.operation, "offset_out_of_range");
            }
            ok_response(
                request.operation,
                RecordsResultV1 {
                    records: product
                        .semantic_records
                        .into_iter()
                        .filter(|record| {
                            record.range.source_unit_id == unit.source_unit_id
                                && record.range.start <= request.offset
                                && request.offset < record.range.end
                        })
                        .collect(),
                },
            )
        }
        QueryRequestV1::Hover(request) => {
            let Some(record) = product
                .semantic_records
                .into_iter()
                .find(|record| record.query_semantic_id == request.query_semantic_id)
            else {
                return error_response(&request.operation, "unknown_query_semantic_id");
            };
            ok_response(request.operation, RecordResultV1 { record })
        }
        QueryRequestV1::Definition(request) => {
            let Some(record) = product
                .semantic_records
                .into_iter()
                .find(|record| record.query_semantic_id == request.query_semantic_id)
            else {
                return error_response(&request.operation, "unknown_query_semantic_id");
            };
            ok_response(request.operation, RecordResultV1 { record })
        }
        QueryRequestV1::References(request) => {
            if !product
                .semantic_records
                .iter()
                .any(|record| record.query_semantic_id == request.query_semantic_id)
            {
                return error_response(&request.operation, "unknown_query_semantic_id");
            }
            ok_response(
                request.operation,
                ReferencesResultV1 {
                    references: product
                        .references
                        .into_iter()
                        .filter(|reference| {
                            reference.target_query_semantic_id == request.query_semantic_id
                        })
                        .collect(),
                },
            )
        }
        QueryRequestV1::DocumentSymbols(request) => {
            if !has_source_unit(&product, &request.source_unit_id) {
                return error_response(&request.operation, "unknown_source_unit");
            }
            ok_response(
                request.operation,
                RecordsResultV1 {
                    records: product
                        .semantic_records
                        .into_iter()
                        .filter(|record| record.range.source_unit_id == request.source_unit_id)
                        .collect(),
                },
            )
        }
        QueryRequestV1::Diagnostics(request) => {
            if !has_source_unit(&product, &request.source_unit_id) {
                return error_response(&request.operation, "unknown_source_unit");
            }
            ok_response(
                request.operation,
                DiagnosticsResultV1 {
                    diagnostics: product
                        .diagnostics
                        .into_iter()
                        .filter(|diagnostic| {
                            diagnostic
                                .primary_range
                                .as_ref()
                                .is_some_and(|range| range.source_unit_id == request.source_unit_id)
                        })
                        .collect(),
                },
            )
        }
        QueryRequestV1::Unsupported(request) => unsupported_response(&request.operation),
    }
}

fn has_source_unit(product: &ToolingQuerySnapshotV1, source_unit_id: &str) -> bool {
    product
        .source_units
        .iter()
        .any(|unit| unit.source_unit_id == source_unit_id)
}

fn decode_request_v1(bytes: &[u8]) -> Result<QueryRequestV1, String> {
    let value: serde_json::Value = serde_json::from_slice(bytes).map_err(|_| String::new())?;
    let operation = value
        .get("operation")
        .and_then(serde_json::Value::as_str)
        .map_or_else(String::new, ToOwned::to_owned);
    let request = match operation.as_str() {
        "position" => serde_json::from_value::<PositionRequestV1>(value.clone())
            .ok()
            .filter(valid_position_request)
            .map(QueryRequestV1::Position),
        "hover" => serde_json::from_value::<SemanticIdRequestV1>(value.clone())
            .ok()
            .filter(|request| valid_semantic_id_request(request, "hover"))
            .map(QueryRequestV1::Hover),
        "definition" => serde_json::from_value::<SemanticIdRequestV1>(value.clone())
            .ok()
            .filter(|request| valid_semantic_id_request(request, "definition"))
            .map(QueryRequestV1::Definition),
        "references" => serde_json::from_value::<SemanticIdRequestV1>(value.clone())
            .ok()
            .filter(|request| valid_semantic_id_request(request, "references"))
            .map(QueryRequestV1::References),
        "documentSymbols" => serde_json::from_value::<SourceUnitRequestV1>(value.clone())
            .ok()
            .filter(|request| valid_source_unit_request(request, "documentSymbols"))
            .map(QueryRequestV1::DocumentSymbols),
        "diagnostics" => serde_json::from_value::<SourceUnitRequestV1>(value.clone())
            .ok()
            .filter(|request| valid_source_unit_request(request, "diagnostics"))
            .map(QueryRequestV1::Diagnostics),
        "rename" | "completion" | "signatureHelp" | "semanticTokens" | "sourceMapping"
        | "edits" | "codeActions" => serde_json::from_value::<UnsupportedRequestV1>(value.clone())
            .ok()
            .filter(valid_unsupported_request)
            .map(QueryRequestV1::Unsupported),
        _ => None,
    }
    .ok_or_else(|| operation.clone())?;

    (request_json(&request).as_bytes() == bytes)
        .then_some(request)
        .ok_or(operation)
}

fn valid_position_request(request: &PositionRequestV1) -> bool {
    request.schema == REQUEST_SCHEMA_V1 && request.version == 1 && request.operation == "position"
}

fn valid_semantic_id_request(request: &SemanticIdRequestV1, operation: &str) -> bool {
    request.schema == REQUEST_SCHEMA_V1
        && request.version == 1
        && request.operation == operation
        && !request.query_semantic_id.is_empty()
}

fn valid_source_unit_request(request: &SourceUnitRequestV1, operation: &str) -> bool {
    request.schema == REQUEST_SCHEMA_V1
        && request.version == 1
        && request.operation == operation
        && !request.source_unit_id.is_empty()
}

fn valid_unsupported_request(request: &UnsupportedRequestV1) -> bool {
    request.schema == REQUEST_SCHEMA_V1
        && request.version == 1
        && matches!(
            request.operation.as_str(),
            "hover"
                | "rename"
                | "completion"
                | "signatureHelp"
                | "semanticTokens"
                | "sourceMapping"
                | "edits"
                | "codeActions"
        )
}

fn request_json(request: &QueryRequestV1) -> String {
    match request {
        QueryRequestV1::Position(request) => json(request),
        QueryRequestV1::Hover(request)
        | QueryRequestV1::Definition(request)
        | QueryRequestV1::References(request) => json(request),
        QueryRequestV1::DocumentSymbols(request) | QueryRequestV1::Diagnostics(request) => {
            json(request)
        }
        QueryRequestV1::Unsupported(request) => json(request),
    }
}

fn ok_response<T: Serialize>(operation: String, result: T) -> Vec<u8> {
    json(&OkResponseV1 {
        schema: RESPONSE_SCHEMA_V1,
        version: 1,
        operation,
        status: "ok",
        result,
    })
    .into_bytes()
}

fn error_response(operation: &str, code: &str) -> Vec<u8> {
    json(&ErrorResponseV1 {
        schema: RESPONSE_SCHEMA_V1,
        version: 1,
        operation,
        status: "error",
        code,
    })
    .into_bytes()
}

fn unsupported_response(operation: &str) -> Vec<u8> {
    json(&UnsupportedResponseV1 {
        schema: RESPONSE_SCHEMA_V1,
        version: 1,
        operation,
        status: "unsupported",
        capability: operation,
    })
    .into_bytes()
}

fn json<T: Serialize>(value: &T) -> String {
    serde_json::to_string(value).expect("language-service response serializes") + "\n"
}

#[cfg(test)]
mod tests {
    use super::*;

    const PRODUCT: &[u8] = include_bytes!("../fixtures/tooling/query-snapshot-v1.json");

    fn snapshot() -> ToolingQuerySnapshotV1 {
        decode_tooling_query_snapshot_v1(PRODUCT).expect("frozen query product decodes")
    }

    fn request(operation: &str, fields: &str) -> Vec<u8> {
        format!(
            "{{\"schema\":\"{REQUEST_SCHEMA_V1}\",\"version\":1,\"operation\":\"{operation}\"{fields}}}\n"
        )
        .into_bytes()
    }

    fn response(product: &[u8], request: &[u8]) -> serde_json::Value {
        let bytes = query_snapshot_v1(product, request);
        assert!(bytes.ends_with(b"\n"));
        serde_json::from_slice(&bytes).expect("response is json")
    }

    #[test]
    fn l12c2_projects_only_records_present_in_a_strict_product() {
        let snapshot = snapshot();
        let source_unit_id = &snapshot.source_units[0].source_unit_id;
        let result = response(
            PRODUCT,
            &request(
                "position",
                &format!(",\"sourceUnitId\":\"{source_unit_id}\",\"offset\":114"),
            ),
        );
        let records = result["result"]["records"]
            .as_array()
            .expect("position returns records");
        let expected = snapshot
            .semantic_records
            .iter()
            .filter(|record| record.range.start <= 114 && 114 < record.range.end)
            .map(|record| serde_json::to_value(record).expect("record serializes"))
            .collect::<Vec<_>>();
        assert_eq!(records, &expected);
        assert_eq!(result["status"], "ok");
        assert_eq!(result["operation"], "position");
    }

    #[test]
    fn l12c2_projects_definition_references_symbols_and_diagnostics_without_inference() {
        let snapshot = snapshot();
        let source_unit_id = &snapshot.source_units[0].source_unit_id;
        let target = snapshot.references[0].target_query_semantic_id.clone();
        let definition = response(
            PRODUCT,
            &request("definition", &format!(",\"querySemanticId\":\"{target}\"")),
        );
        assert_eq!(definition["result"]["record"]["querySemanticId"], target);

        let hover = response(
            PRODUCT,
            &request("hover", &format!(",\"querySemanticId\":\"{target}\"")),
        );
        assert_eq!(hover["result"]["record"]["querySemanticId"], target);

        let references = response(
            PRODUCT,
            &request("references", &format!(",\"querySemanticId\":\"{target}\"")),
        );
        assert_eq!(
            references["result"]["references"].as_array().map(Vec::len),
            Some(1)
        );

        let symbols = response(
            PRODUCT,
            &request(
                "documentSymbols",
                &format!(",\"sourceUnitId\":\"{source_unit_id}\""),
            ),
        );
        assert_eq!(
            symbols["result"]["records"].as_array().map(Vec::len),
            Some(7)
        );

        let diagnostics = response(
            PRODUCT,
            &request(
                "diagnostics",
                &format!(",\"sourceUnitId\":\"{source_unit_id}\""),
            ),
        );
        assert_eq!(
            diagnostics["result"]["diagnostics"]
                .as_array()
                .map(Vec::len),
            Some(0)
        );
    }

    #[test]
    fn l12c2_rejects_noncanonical_or_invalid_products_before_requests() {
        let noncanonical_request = b"{\"schema\":\"wrong\"}\n";
        let invalid_product_response = response(b"{}\n", noncanonical_request);
        assert_eq!(invalid_product_response["operation"], "");
        assert_eq!(invalid_product_response["code"], "invalid_product");

        let snapshot = snapshot();
        let source_unit_id = &snapshot.source_units[0].source_unit_id;
        let invalid_identity = String::from_utf8(PRODUCT.to_vec())
            .expect("fixture is utf8")
            .replacen(
                "5cc4030a46cbc247f02be9fc8e9cc34661b8d9017e4ae808e48954c7ea31d802",
                "10b83768e45763bda26d18867a69751b58b05f0d81125e3a4d6b4bd3c9ed0ffc",
                1,
            );
        let invalid_identity_response = response(
            invalid_identity.as_bytes(),
            &request(
                "position",
                &format!(",\"sourceUnitId\":\"{source_unit_id}\",\"offset\":0"),
            ),
        );
        assert_eq!(invalid_identity_response["code"], "invalid_product");
    }

    #[test]
    fn l12c2_errors_and_unsupported_capabilities_are_stable() {
        let snapshot = snapshot();
        let source_unit_id = &snapshot.source_units[0].source_unit_id;
        let unknown_unit = response(
            PRODUCT,
            &request(
                "position",
                ",\"sourceUnitId\":\"source:missing\",\"offset\":0",
            ),
        );
        assert_eq!(unknown_unit["code"], "unknown_source_unit");

        let out_of_range = response(
            PRODUCT,
            &request(
                "position",
                &format!(",\"sourceUnitId\":\"{source_unit_id}\",\"offset\":140"),
            ),
        );
        assert_eq!(out_of_range["code"], "offset_out_of_range");

        let unknown_id = response(
            PRODUCT,
            &request(
                "definition",
                ",\"querySemanticId\":\"query-semantic:missing\"",
            ),
        );
        assert_eq!(unknown_id["code"], "unknown_query_semantic_id");

        for operation in [
            "rename",
            "completion",
            "signatureHelp",
            "semanticTokens",
            "sourceMapping",
            "edits",
            "codeActions",
        ] {
            let unsupported = response(PRODUCT, &request(operation, ""));
            assert_eq!(unsupported["status"], "unsupported");
            assert_eq!(unsupported["capability"], operation);
        }
    }

    #[test]
    fn l12c2_rejects_noncanonical_and_unknown_requests() {
        let snapshot = snapshot();
        let source_unit_id = &snapshot.source_units[0].source_unit_id;
        let mut noncanonical = request(
            "position",
            &format!(",\"sourceUnitId\":\"{source_unit_id}\",\"offset\":0"),
        );
        noncanonical.insert(0, b' ');
        let noncanonical_response = response(PRODUCT, &noncanonical);
        assert_eq!(noncanonical_response["code"], "invalid_request");
        assert_eq!(noncanonical_response["operation"], "position");

        let unknown = request("notAQuery", "");
        let unknown_response = response(PRODUCT, &unknown);
        assert_eq!(unknown_response["code"], "invalid_request");
        assert_eq!(unknown_response["operation"], "notAQuery");
    }
}
