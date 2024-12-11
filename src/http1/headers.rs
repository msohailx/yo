use std::str;
use std::vec::Vec;
use regex::Regex;

/// It is the Headers struct definition
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Headers {
    full_items: Vec<(Vec<u8>, Vec<u8>, Vec<u8>)>, // (raw_name, lower_name, value)
}

#[derive(Debug)]
pub struct LocalProtocolError(String);

impl Headers {
    /// Creates a new `Headers` instance with the provided header items.
    ///
    /// # Arguments
    /// * `full_items` - A vector of triplets containing raw name, normalized name, and value.
    ///
    /// # Returns
    /// A new `Headers` instance.
    pub fn new(full_items: Vec<(Vec<u8>, Vec<u8>, Vec<u8>)>) -> Self {
        Headers { full_items }
    }
    /// Returns a vector of raw key-value pairs from the headers.
    ///
    /// # Returns
    /// A vector of tuples containing raw header name and value.
    pub fn raw_items(&self) -> Vec<(Vec<u8>, Vec<u8>)> {
        self.full_items.iter().map(|(raw_name, _, value)| (raw_name.clone(), value.clone())).collect()
    }
    /// Normalizes and validates the provided headers.
    ///
    /// # Arguments
    /// * `headers` - A slice of key-value pairs representing headers.
    /// * `parsed` - Flag indicating if the headers are already parsed.
    ///
    /// # Returns
    /// A validated `Headers` instance, or an error if validation fails.
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
                    // Convert to `&str` before trimming
                    let s_str = str::from_utf8(s).unwrap_or("");
                    s_str.trim().as_bytes().to_vec()
                }) // Change: We need to convert bytes to str first and then trim
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
    /// Retrieves comma-separated values from a header.
    ///
    /// # Arguments
    /// * `name` - The header name to search for.
    ///
    /// # Returns
    /// A vector of values split by commas.
    pub fn get_comma_header(&self, name: &[u8]) -> Vec<Vec<u8>> {
        let mut out = Vec::new();

        for (_, found_name, found_raw_value) in &self.full_items {
            if found_name == name {
                let parts = found_raw_value.to_ascii_lowercase().split(|&b| b == b',')
                .map(|s| {
                    // Convert to `&str` before trimming
                    let s_str = str::from_utf8(s).unwrap_or("");
                    s_str.trim().as_bytes().to_vec()
                })
                    .collect::<Vec<Vec<u8>>>();
                out.extend(parts);
            }
        }

        out
    }
    /// Sets new comma-separated values for a header.
    ///
    /// # Arguments
    /// * `name` - The header name to update.
    /// * `new_values` - A vector of new values to be set.
    ///
    /// # Returns
    /// A new `Headers` instance with updated values, or an error if validation fails.
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
    /// Checks if the "Expect: 100-continue" header is present.
    ///
    /// # Returns
    /// `true` if the header is present, otherwise `false`.
    pub fn has_expect_100_continue(&self) -> bool {
        let expect = self.get_comma_header(b"expect");
        expect.iter().any(|value| value == b"100-continue")
    }
}
/// Validates the given value against a regular expression.
///
/// # Arguments
/// * `re` - A regex pattern to match the value against.
/// * `value` - The byte slice to validate.
/// * `error_msg` - The error message to return if validation fails.
///
/// # Returns
/// `Ok(())` if validation succeeds, or an error with a message if it fails.
fn validate(re: &Regex, value: &[u8], error_msg: &str) -> Result<(), LocalProtocolError> {
    // Convert value from &[u8] to &str for regex matching
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
///
/// # Arguments
/// * `input` - The byte slice to normalize.
///
/// # Returns
/// `Ok(Vec<u8>)` if successful, or an error if the input is empty.
fn normalize_bytes(input: &[u8]) -> Result<Vec<u8>, LocalProtocolError> {
    if input.is_empty() {
        return Err(LocalProtocolError("Empty input".to_string()));
    }

    Ok(input.to_vec())
}

