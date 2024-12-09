use std::collections::HashMap;
use std::str;
use std::vec::Vec;
use regex::Regex;

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Headers {
    full_items: Vec<(Vec<u8>, Vec<u8>, Vec<u8>)>, // (raw_name, lower_name, value)
}

#[derive(Debug)]
pub struct LocalProtocolError(String);

impl Headers {
    pub fn new(full_items: Vec<(Vec<u8>, Vec<u8>, Vec<u8>)>) -> Self {
        Headers { full_items }
    }

    pub fn raw_items(&self) -> Vec<(Vec<u8>, Vec<u8>)> {
        self.full_items.iter().map(|(raw_name, _, value)| (raw_name.clone(), value.clone())).collect()
    }

    pub fn normalize_and_validate(headers: &[(Vec<u8>, Vec<u8>)], parsed: bool) -> Result<Headers, LocalProtocolError> {
        let mut new_headers: Vec<(Vec<u8>, Vec<u8>, Vec<u8>)> = Vec::new();
        let mut seen_content_length: Option<Vec<u8>> = None;
        let mut saw_transfer_encoding = false;

        let content_length_re = Regex::new(r"^[0-9]+$").unwrap();
        let field_name_re = Regex::new(r"^[!#$%&'*+.^_`|~0-9A-Za-z-]+$").unwrap();
        let field_value_re = Regex::new(r"^[ -~]+$").unwrap();

        for (name, value) in headers {
            let (mut name, mut value) = if parsed {
                (name.clone(), value.clone())
            } else {
                let name = normalize_bytes(name)?;
                let value = normalize_bytes(value)?;
                validate(&field_name_re, &name, "Illegal header name")?;
                validate(&field_value_re, &value, "Illegal header value")?;
                (name, value)
            };

            let raw_name = name.clone();
            name = name.to_ascii_lowercase();

            if name == b"content-length" {
                let lengths: Vec<Vec<u8>> = value.split(|&b| b == b',')
                    .map(|s| s.trim().to_vec())
                    .collect();

                if lengths.len() != 1 {
                    return Err(LocalProtocolError("conflicting Content-Length headers".to_string()));
                }

                value = lengths[0].clone();
                validate(&content_length_re, &value, "bad Content-Length")?;

                if seen_content_length.is_none() {
                    seen_content_length = Some(value.clone());
                    new_headers.push((raw_name, name, value));
                } else if seen_content_length.as_ref() != Some(&value) {
                    return Err(LocalProtocolError("conflicting Content-Length headers".to_string()));
                }
            } else if name == b"transfer-encoding" {
                if saw_transfer_encoding {
                    return Err(LocalProtocolError("multiple Transfer-Encoding headers".to_string()));
                }

                value = value.to_ascii_lowercase();
                if value != b"chunked" {
                    return Err(LocalProtocolError("Only Transfer-Encoding: chunked is supported".to_string()));
                }

                saw_transfer_encoding = true;
                new_headers.push((raw_name, name, value));
            } else {
                new_headers.push((raw_name, name, value));
            }
        }

        Ok(Headers::new(new_headers))
    }

    pub fn get_comma_header(&self, name: &[u8]) -> Vec<Vec<u8>> {
        let mut out = Vec::new();

        for (_, found_name, found_raw_value) in &self.full_items {
            if found_name == name {
                let parts = found_raw_value.to_ascii_lowercase().split(|&b| b == b',')
                    .map(|s| s.trim().to_vec())
                    .collect::<Vec<Vec<u8>>>();
                out.extend(parts);
            }
        }

        out
    }

    pub fn set_comma_header(&self, name: &[u8], new_values: Vec<Vec<u8>>) -> Result<Headers, LocalProtocolError> {
        let mut new_headers: Vec<(Vec<u8>, Vec<u8>, Vec<u8>)> = Vec::new();

        // Remove existing headers with the same name
        for (raw_name, found_name, found_raw_value) in &self.full_items {
            if found_name != name {
                new_headers.push((raw_name.clone(), found_name.clone(), found_raw_value.clone()));
            }
        }

        // Add new comma-separated values
        for new_value in new_values {
            new_headers.push((name.to_ascii_uppercase(), name.to_ascii_lowercase(), new_value));
        }

        Self::normalize_and_validate(&new_headers.iter().map(|(n, v, _)| (n.clone(), v.clone())).collect::<Vec<_>>(), false)
    }

    pub fn has_expect_100_continue(&self) -> bool {
        let expect = self.get_comma_header(b"expect");
        expect.iter().any(|value| value == b"100-continue")
    }
}

fn validate(re: &Regex, value: &[u8], error_msg: &str) -> Result<(), LocalProtocolError> {
    if !re.is_match(value) {
        return Err(LocalProtocolError(format!("{}: {:?}", error_msg, value)));
    }
    Ok(())
}

fn normalize_bytes(input: &[u8]) -> Result<Vec<u8>, LocalProtocolError> {
    if input.is_empty() {
        return Err(LocalProtocolError("Empty input".to_string()));
    }

    Ok(input.to_vec())
}

