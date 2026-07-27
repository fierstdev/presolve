//! Opaque runtime occurrence identity codec shared by structural artifacts.

const PREFIX: &str = "presolve-structural-occurrence:v1:";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StructuralOccurrenceIdentity {
    pub parent_scope: String,
    pub region: String,
    pub template_instance: String,
    pub local_occurrence: String,
}

/// Encode the four compiler/runtime-issued identity inputs with the frozen v1
/// framing. Inputs must be non-empty; callers own their prior validation.
#[must_use]
pub fn encode_structural_occurrence_identity(
    parent_scope: &str,
    region: &str,
    template_instance: &str,
    local_occurrence: &str,
) -> String {
    format!(
        "{PREFIX}{}.{}.{}.{}",
        hex(parent_scope),
        hex(region),
        hex(template_instance),
        hex(local_occurrence)
    )
}

/// Decode a v1 opaque occurrence identity without interpreting its fields as
/// semantic IDs, list keys, or DOM addresses.
pub fn decode_structural_occurrence_identity(
    value: &str,
) -> Result<StructuralOccurrenceIdentity, String> {
    let fields = value
        .strip_prefix(PREFIX)
        .ok_or_else(|| "structural occurrence identity has an invalid prefix".to_string())?
        .split('.')
        .collect::<Vec<_>>();
    if fields.len() != 4 || fields.iter().any(|field| field.is_empty()) {
        return Err("structural occurrence identity has an invalid field count".to_string());
    }
    let decoded = fields
        .into_iter()
        .map(decode_hex)
        .collect::<Result<Vec<_>, _>>()?;
    Ok(StructuralOccurrenceIdentity {
        parent_scope: decoded[0].clone(),
        region: decoded[1].clone(),
        template_instance: decoded[2].clone(),
        local_occurrence: decoded[3].clone(),
    })
}

fn hex(value: &str) -> String {
    value
        .as_bytes()
        .iter()
        .map(|byte| format!("{byte:02X}"))
        .collect()
}

fn decode_hex(value: &str) -> Result<String, String> {
    if value.len() % 2 != 0 {
        return Err("structural occurrence identity has an odd hex field".to_string());
    }
    let bytes = (0..value.len())
        .step_by(2)
        .map(|index| u8::from_str_radix(&value[index..index + 2], 16))
        .collect::<Result<Vec<_>, _>>()
        .map_err(|_| "structural occurrence identity has invalid hex".to_string())?;
    let decoded = String::from_utf8(bytes)
        .map_err(|_| "structural occurrence identity has invalid UTF-8".to_string())?;
    if decoded.is_empty() {
        return Err("structural occurrence identity has an empty decoded field".to_string());
    }
    Ok(decoded)
}

#[cfg(test)]
mod tests {
    use super::{decode_structural_occurrence_identity, encode_structural_occurrence_identity};

    #[test]
    fn round_trips_utf8_and_uses_uppercase_byte_framing() {
        let encoded =
            encode_structural_occurrence_identity("parent:α", "region", "template", "key:7");
        assert_eq!(encoded, "presolve-structural-occurrence:v1:706172656E743ACEB1.726567696F6E.74656D706C617465.6B65793A37");
        assert_eq!(
            decode_structural_occurrence_identity(&encoded)
                .unwrap()
                .local_occurrence,
            "key:7"
        );
    }

    #[test]
    fn rejects_malformed_or_empty_fields() {
        for value in [
            "presolve-structural-occurrence:v2:61.62.63.64",
            "presolve-structural-occurrence:v1:61.62.63",
            "presolve-structural-occurrence:v1:61..63.64",
            "presolve-structural-occurrence:v1:6.62.63.64",
            "presolve-structural-occurrence:v1:GG.62.63.64",
            "presolve-structural-occurrence:v1:.62.63.64",
        ] {
            assert!(
                decode_structural_occurrence_identity(value).is_err(),
                "{value}"
            );
        }
    }
}
