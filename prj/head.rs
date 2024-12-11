use std::str;
use std::vec::Vec;
use regex::Regex;

/// Struct to represent HTTP headers.
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Headers {
    full_items: Vec<(Vec<u8>, Vec<u8>, Vec<u8>)>, // (raw_name, lower_name, value)
}

/// A custom error type to handle protocol validation errors.
#[derive(Debug)]
pub struct LocalProtocolError(String);

impl Headers {
    /// Creates a new `Headers` instance with the provided header items.
    pub fn new(full_items: Vec<(Vec<u8>, Vec<u8>, Vec<u8>)>) -> Self {
        Headers { full_items }
    }

    /// Returns a vector of raw key-value pairs from the headers.
    pub fn raw_items(&self) -> Vec<(Vec<u8>, Vec<u8>)> {
        self.full_items.iter().map(|(raw_name, _, value)| (raw_name.clone(), value.clone())).collect()
    }

    /// Normalizes and validates the provided headers.
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
                    .map(|s| {
                        let s_str = str::from_utf8(s).unwrap_or("");
                        s_str.trim().as_bytes().to_vec()
                    })
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
                    .map(|s| {
                        let s_str = str::from_utf8(s).unwrap_or("");
                        s_str.trim().as_bytes().to_vec()
                    })
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

/// Validates the given value against a regular expression.
fn validate(re: &Regex, value: &[u8], error_msg: &str) -> Result<(), LocalProtocolError> {
    if let Ok(value_str) = str::from_utf8(value) {
        if !re.is_match(value_str) {
            return Err(LocalProtocolError(format!("{}: {:?}", error_msg, value)));
        }
    } else {
        return Err(LocalProtocolError(format!("Invalid UTF-8 value: {:?}", value)));
    }

    Ok(())
}

/// Normalizes the input bytes to a `Vec<u8>`.
fn normalize_bytes(input: &[u8]) -> Result<Vec<u8>, LocalProtocolError> {
    if input.is_empty() {
        return Err(LocalProtocolError("Empty input".to_string()));
    }

    Ok(input.to_vec())
}

fn main() {
    // Step 1: Simulate receiving raw HTTP headers
    let raw_headers = vec![
        (b"Content-Type".to_vec(), b"application/json".to_vec()),
        (b"Content-Length".to_vec(), b"123".to_vec()),
        (b"Transfer-Encoding".to_vec(), b"chunked".to_vec()),
        (b"Expect".to_vec(), b"100-continue".to_vec()),
        (b"Custom-Header".to_vec(), b"value1,value2,value3".to_vec()),
    ];

    // Step 2: Normalize and validate the headers
    match Headers::normalize_and_validate(&raw_headers, false) {
        Ok(valid_headers) => {
            println!("Headers after normalization and validation:");
            for (raw_name, _, value) in &valid_headers.full_items {
                println!("{:?}: {:?}", str::from_utf8(raw_name).unwrap(), str::from_utf8(value).unwrap());
            }

            // Step 3: Get and print comma-separated values of a header
            let custom_header_values = valid_headers.get_comma_header(b"Custom-Header");
            println!("\nComma-separated values for 'Custom-Header':");
            for value in custom_header_values {
                println!("{:?}", str::from_utf8(&value).unwrap());
            }

            // Step 4: Modify a comma-separated header (e.g., add a new value to "Custom-Header")
            let new_values = vec![
                b"value4".to_vec(),
                b"value5".to_vec(),
            ];
            match valid_headers.set_comma_header(b"Custom-Header", new_values) {
                Ok(updated_headers) => {
                    println!("\nUpdated headers after modifying 'Custom-Header':");
                    for (raw_name, _, value) in &updated_headers.full_items {
                        println!("{:?}: {:?}", str::from_utf8(raw_name).unwrap(), str::from_utf8(value).unwrap());
                    }
                }
                Err(e) => println!("Error updating header: {:?}", e),
            }

            // Step 5: Check for 'Expect: 100-continue' header
            let expect_100_continue = valid_headers.has_expect_100_continue();
            println!("\nIs 'Expect: 100-continue' header present? {}", expect_100_continue);
        }
        Err(e) => {
            println!("Error: {:?}", e);
        }
    }
}

