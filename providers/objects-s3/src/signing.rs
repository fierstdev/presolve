use std::time::{SystemTime, UNIX_EPOCH};

use sha2::{Digest, Sha256};

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct AmzTimestamp {
    pub(crate) date: String,
    pub(crate) full: String,
}

pub(crate) fn now() -> Result<AmzTimestamp, String> {
    format_timestamp(SystemTime::now())
}

pub(crate) fn format_timestamp(time: SystemTime) -> Result<AmzTimestamp, String> {
    let seconds = time
        .duration_since(UNIX_EPOCH)
        .map_err(|_| "system clock is before the Unix epoch".to_owned())?
        .as_secs();

    let days = seconds / 86_400;
    let seconds_of_day = seconds % 86_400;
    let days = i64::try_from(days).map_err(|_| "system clock is out of supported range")?;
    let (year, month, day) = civil_from_days(days);
    let hour = seconds_of_day / 3_600;
    let minute = (seconds_of_day % 3_600) / 60;
    let second = seconds_of_day % 60;

    if !(0..=9999).contains(&year) {
        return Err("system clock year is out of supported range".to_owned());
    }

    let date = format!("{year:04}{month:02}{day:02}");
    let full = format!("{date}T{hour:02}{minute:02}{second:02}Z");

    Ok(AmzTimestamp { date, full })
}

pub(crate) fn sha256_hex(bytes: &[u8]) -> String {
    hex::encode(Sha256::digest(bytes))
}

pub(crate) struct SigV4Input<'a> {
    pub(crate) access_key_id: &'a str,
    pub(crate) secret_access_key: &'a str,
    pub(crate) region: &'a str,
    pub(crate) service: &'a str,
    pub(crate) method: &'a str,
    pub(crate) canonical_uri: &'a str,
    pub(crate) canonical_query: &'a str,
    pub(crate) canonical_host: &'a str,
    pub(crate) payload_hash: &'a str,
    pub(crate) timestamp: &'a AmzTimestamp,
}

pub(crate) fn authorization(input: &SigV4Input<'_>) -> String {
    let canonical_headers = format!(
        "host:{}\nx-amz-content-sha256:{}\nx-amz-date:{}\n",
        input.canonical_host, input.payload_hash, input.timestamp.full
    );
    let signed_headers = "host;x-amz-content-sha256;x-amz-date";
    let canonical_request = format!(
        "{}\n{}\n{}\n{}\n{}\n{}",
        input.method,
        input.canonical_uri,
        input.canonical_query,
        canonical_headers,
        signed_headers,
        input.payload_hash
    );
    let scope = format!(
        "{}/{}/{}/aws4_request",
        input.timestamp.date, input.region, input.service
    );
    let string_to_sign = format!(
        "AWS4-HMAC-SHA256\n{}\n{}\n{}",
        input.timestamp.full,
        scope,
        sha256_hex(canonical_request.as_bytes())
    );

    let date_key = hmac_sha256(
        format!("AWS4{}", input.secret_access_key).as_bytes(),
        input.timestamp.date.as_bytes(),
    );
    let region_key = hmac_sha256(&date_key, input.region.as_bytes());
    let service_key = hmac_sha256(&region_key, input.service.as_bytes());
    let signing_key = hmac_sha256(&service_key, b"aws4_request");
    let signature = hex::encode(hmac_sha256(&signing_key, string_to_sign.as_bytes()));

    format!(
        "AWS4-HMAC-SHA256 Credential={}/{}, SignedHeaders={}, Signature={}",
        input.access_key_id, scope, signed_headers, signature
    )
}

pub(crate) fn aws_percent_encode(input: &str, encode_slash: bool) -> String {
    let mut encoded = String::with_capacity(input.len());

    for byte in input.as_bytes() {
        let unreserved = byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'.' | b'_' | b'~');
        let slash = *byte == b'/' && !encode_slash;

        if unreserved || slash {
            encoded.push(char::from(*byte));
        } else {
            use std::fmt::Write as _;
            write!(&mut encoded, "%{byte:02X}").expect("writing to String cannot fail");
        }
    }

    encoded
}

pub(crate) fn canonical_query<'a, I>(pairs: I) -> String
where
    I: IntoIterator<Item = (&'a str, &'a str)>,
{
    let mut pairs = pairs
        .into_iter()
        .map(|(key, value)| {
            (
                aws_percent_encode(key, true),
                aws_percent_encode(value, true),
            )
        })
        .collect::<Vec<_>>();

    pairs.sort();
    pairs
        .into_iter()
        .map(|(key, value)| format!("{key}={value}"))
        .collect::<Vec<_>>()
        .join("&")
}

fn hmac_sha256(key: &[u8], message: &[u8]) -> [u8; 32] {
    const BLOCK_BYTES: usize = 64;

    let mut normalized_key = [0_u8; BLOCK_BYTES];

    if key.len() > BLOCK_BYTES {
        let digest = Sha256::digest(key);
        normalized_key[..digest.len()].copy_from_slice(&digest);
    } else {
        normalized_key[..key.len()].copy_from_slice(key);
    }

    let mut inner_pad = [0_u8; BLOCK_BYTES];
    let mut outer_pad = [0_u8; BLOCK_BYTES];

    for (index, byte) in normalized_key.iter().enumerate() {
        inner_pad[index] = byte ^ 0x36;
        outer_pad[index] = byte ^ 0x5c;
    }

    let mut inner = Sha256::new();
    inner.update(inner_pad);
    inner.update(message);
    let inner_digest = inner.finalize();

    let mut outer = Sha256::new();
    outer.update(outer_pad);
    outer.update(inner_digest);
    outer.finalize().into()
}

fn civil_from_days(days_since_epoch: i64) -> (i64, u64, u64) {
    let shifted = days_since_epoch + 719_468;
    let era = if shifted >= 0 {
        shifted / 146_097
    } else {
        (shifted - 146_096) / 146_097
    };
    let day_of_era = shifted - era * 146_097;
    let year_of_era =
        (day_of_era - day_of_era / 1_460 + day_of_era / 36_524 - day_of_era / 146_096)
            / 365;
    let mut year = year_of_era + era * 400;
    let day_of_year = day_of_era - (365 * year_of_era + year_of_era / 4 - year_of_era / 100);
    let month_prime = (5 * day_of_year + 2) / 153;
    let day = day_of_year - (153 * month_prime + 2) / 5 + 1;
    let month = month_prime + if month_prime < 10 { 3 } else { -9 };

    if month <= 2 {
        year += 1;
    }

    (
        year,
        u64::try_from(month).expect("civil month is positive"),
        u64::try_from(day).expect("civil day is positive"),
    )
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use super::*;

    #[test]
    fn unix_epoch_formats_as_amz_timestamp() {
        assert_eq!(
            format_timestamp(UNIX_EPOCH).expect("epoch should format"),
            AmzTimestamp {
                date: "19700101".to_owned(),
                full: "19700101T000000Z".to_owned(),
            }
        );
    }

    #[test]
    fn timestamp_crosses_leap_day_correctly() {
        // 2024-02-29T12:34:56Z
        let instant = UNIX_EPOCH + Duration::from_secs(1_709_210_096);
        assert_eq!(
            format_timestamp(instant).expect("timestamp should format").full,
            "20240229T123456Z"
        );
    }

    #[test]
    fn aws_encoding_preserves_only_unreserved_and_optional_slashes() {
        assert_eq!(
            aws_percent_encode("folder/Case key+%", false),
            "folder/Case%20key%2B%25"
        );
        assert_eq!(
            aws_percent_encode("folder/Case key+%", true),
            "folder%2FCase%20key%2B%25"
        );
    }

    #[test]
    fn query_is_encoded_and_sorted() {
        assert_eq!(
            canonical_query([("prefix", "b/"), ("list-type", "2"), ("encoding-type", "url")]),
            "encoding-type=url&list-type=2&prefix=b%2F"
        );
    }

    #[test]
    fn signature_matches_aws_documented_iam_vector_shape() {
        let timestamp = AmzTimestamp {
            date: "20150830".to_owned(),
            full: "20150830T123600Z".to_owned(),
        };
        let payload_hash = sha256_hex(b"");
        let authorization = authorization(&SigV4Input {
            access_key_id: "AKIDEXAMPLE",
            secret_access_key: "wJalrXUtnFEMI/K7MDENG+bPxRfiCYEXAMPLEKEY",
            region: "us-east-1",
            service: "service",
            method: "GET",
            canonical_uri: "/",
            canonical_query: "",
            canonical_host: "example.amazonaws.com",
            payload_hash: &payload_hash,
            timestamp: &timestamp,
        });

        assert!(authorization.starts_with(
            "AWS4-HMAC-SHA256 Credential=AKIDEXAMPLE/20150830/us-east-1/service/aws4_request, "
        ));
        assert!(authorization.contains("SignedHeaders=host;x-amz-content-sha256;x-amz-date"));
        assert!(authorization.contains("Signature="));
    }
}
