use percent_encoding::percent_decode_str;
use presolve_provider_sdk::{ObjectInfo, ObjectStoreError};

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ListPage {
    pub(crate) objects: Vec<ObjectInfo>,
    pub(crate) next_continuation_token: Option<String>,
}

pub(crate) fn parse_list_page(xml: &str) -> Result<ListPage, ObjectStoreError> {
    let mut objects = Vec::new();
    let mut remaining = xml;

    while let Some(start) = remaining.find("<Contents>") {
        remaining = &remaining[start + "<Contents>".len()..];
        let Some(end) = remaining.find("</Contents>") else {
            return Err(ObjectStoreError::internal(
                "S3 ListObjectsV2 response contains an unterminated Contents element",
            ));
        };

        let contents = &remaining[..end];
        let encoded_key = required_text(contents, "Key")?;
        let size = required_text(contents, "Size")?
            .parse::<u64>()
            .map_err(|_| ObjectStoreError::internal("S3 object size is not a valid u64"))?;
        let key = percent_decode_str(encoded_key)
            .decode_utf8()
            .map_err(|_| ObjectStoreError::internal("S3 object key is not valid UTF-8"))?
            .into_owned();

        objects.push(ObjectInfo::new(key, size));
        remaining = &remaining[end + "</Contents>".len()..];
    }

    let truncated = optional_text(xml, "IsTruncated")
        .map(str::trim)
        .is_some_and(|value| value.eq_ignore_ascii_case("true"));

    let next_continuation_token = if truncated {
        let token = required_text(xml, "NextContinuationToken")?;
        Some(xml_unescape(token)?)
    } else {
        None
    };

    Ok(ListPage {
        objects,
        next_continuation_token,
    })
}

fn required_text<'a>(xml: &'a str, tag: &str) -> Result<&'a str, ObjectStoreError> {
    optional_text(xml, tag).ok_or_else(|| {
        ObjectStoreError::internal(format!("S3 response is missing required `{tag}` element"))
    })
}

fn optional_text<'a>(xml: &'a str, tag: &str) -> Option<&'a str> {
    let opening = format!("<{tag}>");
    let closing = format!("</{tag}>");
    let start = xml.find(&opening)? + opening.len();
    let tail = &xml[start..];
    let end = tail.find(&closing)?;
    Some(&tail[..end])
}

fn xml_unescape(value: &str) -> Result<String, ObjectStoreError> {
    let mut output = String::with_capacity(value.len());
    let mut remaining = value;

    while let Some(index) = remaining.find('&') {
        output.push_str(&remaining[..index]);
        remaining = &remaining[index..];

        let Some(end) = remaining.find(';') else {
            return Err(ObjectStoreError::internal(
                "S3 XML contains an unterminated entity",
            ));
        };

        let entity = &remaining[..=end];
        let decoded = match entity {
            "&amp;" => '&',
            "&lt;" => '<',
            "&gt;" => '>',
            "&quot;" => '"',
            "&apos;" => '\'',
            _ => {
                return Err(ObjectStoreError::internal(format!(
                    "S3 XML contains unsupported entity `{entity}`"
                )));
            }
        };

        output.push(decoded);
        remaining = &remaining[end + 1..];
    }

    output.push_str(remaining);
    Ok(output)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_encoded_keys_sizes_and_pagination() {
        let page = parse_list_page(
            r#"<?xml version="1.0" encoding="UTF-8"?>
<ListBucketResult xmlns="http://s3.amazonaws.com/doc/2006-03-01/">
  <EncodingType>url</EncodingType>
  <IsTruncated>true</IsTruncated>
  <Contents><Key>images%2Fa%20b.png</Key><Size>3</Size></Contents>
  <Contents><Key>images%2Fzero</Key><Size>0</Size></Contents>
  <NextContinuationToken>next&amp;token</NextContinuationToken>
</ListBucketResult>"#,
        )
        .expect("listing should parse");

        assert_eq!(
            page.objects,
            vec![
                ObjectInfo::new("images/a b.png", 3),
                ObjectInfo::new("images/zero", 0),
            ]
        );
        assert_eq!(page.next_continuation_token.as_deref(), Some("next&token"));
    }

    #[test]
    fn non_truncated_page_does_not_require_token() {
        let page = parse_list_page(
            "<ListBucketResult><IsTruncated>false</IsTruncated></ListBucketResult>",
        )
        .expect("empty listing should parse");

        assert_eq!(page.objects, Vec::<ObjectInfo>::new());
        assert!(page.next_continuation_token.is_none());
    }

    #[test]
    fn malformed_contents_is_rejected() {
        let error = parse_list_page(
            "<ListBucketResult><Contents><Key>x</Key></Contents></ListBucketResult>",
        )
        .expect_err("missing size should fail");

        assert_eq!(
            error.code(),
            presolve_provider_sdk::ObjectStoreErrorCode::Internal
        );
    }
}
