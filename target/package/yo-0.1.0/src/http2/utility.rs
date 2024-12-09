use std::collections::{HashSet, HashMap};
use regex::Regex;
use std::str::FromStr;

// Constants
const LARGEST_FLOW_CONTROL_WINDOW: u32 = 2_u32.pow(31) - 1;

static UPPER_RE: Lazy<Regex> = Lazy::new(|| Regex::new(r"[A-Z]").unwrap());

// Error types
#[derive(Debug)]
pub struct ProtocolError(pub String);

#[derive(Debug)]
pub struct FlowControlError(pub String);

#[derive(Debug)]
pub struct HeaderValidationFlags {
    is_client: bool,
    is_trailer: bool,
    is_response_header: bool,
    is_push_promise: bool,
}

// Secure Headers
fn secure_headers(headers: &[(String, String)]) -> Vec<(String, String)> {
    let mut result = Vec::new();
    for (key, value) in headers {
        if key == "authorization" || key == "proxy-authorization" {
            result.push((key.clone(), value.clone())); // Never-indexed headers
        } else if key == "cookie" && value.len() < 20 {
            result.push((key.clone(), value.clone())); // Short cookie made never-indexed
        } else {
            result.push((key.clone(), value.clone())); // Regular headers
        }
    }
    result
}

// Validating headers
fn validate_headers(headers: &[(String, String)], flags: &HeaderValidationFlags) -> Result<Vec<(String, String)>, ProtocolError> {
    let mut validated_headers = Vec::new();

    for (key, value) in headers {
        if key.is_empty() {
            return Err(ProtocolError("Received header name with zero length.".to_string()));
        }

        if UPPER_RE.is_match(key) {
            return Err(ProtocolError(format!("Received uppercase header name {}", key)));
        }

        // Reject surrounding whitespace
        if key.trim_start() != key || key.trim_end() != key {
            return Err(ProtocolError(format!("Header name surrounded by whitespace {}", key)));
        }

        if value.trim_start() != value || value.trim_end() != value {
            return Err(ProtocolError(format!("Header value surrounded by whitespace {}", value)));
        }

        // Reject connection headers
        let connection_headers = vec![
            "connection", "proxy-connection", "keep-alive", 
            "transfer-encoding", "upgrade"
        ];
        if connection_headers.contains(&key.as_str()) {
            return Err(ProtocolError(format!("Connection-specific header field present: {}", key)));
        }

        validated_headers.push((key.clone(), value.clone()));
    }

    Ok(validated_headers)
}

// Increment flow control window size, ensuring it doesn't exceed the largest size.
fn guard_increment_window(current: u32, increment: u32) -> Result<u32, FlowControlError> {
    let new_size = current + increment;

    if new_size > LARGEST_FLOW_CONTROL_WINDOW {
        return Err(FlowControlError(format!(
            "May not increment flow control window past {}",
            LARGEST_FLOW_CONTROL_WINDOW
        )));
    }

    Ok(new_size)
}

// Extract the authority header
fn authority_from_headers(headers: &[(String, String)]) -> Option<String> {
    for (key, value) in headers {
        if key == ":authority" {
            return Some(value.clone());
        }
    }
    None
}

fn main() {
    // Example Usage
    let headers = vec![
        (":method".to_string(), "GET".to_string()),
        (":authority".to_string(), "example.com".to_string())
    ];

    let flags = HeaderValidationFlags {
        is_client: true,
        is_trailer: false,
        is_response_header: true,
        is_push_promise: false,
    };

    match validate_headers(&headers, &flags) {
        Ok(valid_headers) => println!("Valid Headers: {:?}", valid_headers),
        Err(e) => println!("Validation Error: {:?}", e),
    }

    // Simulating flow control window increment
    match guard_increment_window(100, 500) {
        Ok(new_size) => println!("New Window Size: {}", new_size),
        Err(e) => println!("Flow Control Error: {:?}", e),
    }
}
use std::collections::{HashSet, HashMap};
use std::str::FromStr;
use regex::Regex;

#[derive(Debug)]
pub struct ProtocolError(pub String);

// Custom Error for Header Validation
#[derive(Debug)]
pub struct HeaderValidationError(pub String);

// Constants
static ALLOWED_PSEUDO_HEADER_FIELDS: &[&str] = &["method", "status", "scheme", "path"];
static REQUEST_ONLY_HEADERS: HashSet<&str> = ["method", "path", "scheme"].iter().copied().collect();
static RESPONSE_ONLY_HEADERS: HashSet<&str> = ["status"].iter().copied().collect();

// Function to assert header presence
fn assert_header_in_set(string_header: &str, bytes_header: &[u8], header_set: &HashSet<&str>) -> Result<(), ProtocolError> {
    if !header_set.contains(string_header) && !header_set.contains(std::str::from_utf8(bytes_header).unwrap()) {
        return Err(ProtocolError(format!("Header block missing mandatory {} header", string_header)));
    }
    Ok(())
}

// Function to reject pseudo-header fields
fn reject_pseudo_header_fields(headers: &[(String, String)], hdr_validation_flags: &HeaderValidationFlags) -> Result<Vec<(String, String)>, ProtocolError> {
    let mut seen_pseudo_header_fields: HashSet<String> = HashSet::new();
    let mut seen_regular_header = false;
    let mut method = None;

    for header in headers {
        let (key, value) = header;

        if key.starts_with(":") {
            if seen_pseudo_header_fields.contains(key) {
                return Err(ProtocolError(format!("Received duplicate pseudo-header field {}", key)));
            }

            seen_pseudo_header_fields.insert(key.clone());

            if seen_regular_header {
                return Err(ProtocolError(format!("Received pseudo-header field out of sequence: {}", key)));
            }

            if !ALLOWED_PSEUDO_HEADER_FIELDS.contains(&key.as_str()) {
                return Err(ProtocolError(format!("Received custom pseudo-header field {}", key)));
            }

            if key == ":method" {
                method = Some(value.clone());
            }
        } else {
            seen_regular_header = true;
        }
    }

    // Check pseudo-header acceptability
    check_pseudo_header_field_acceptability(&seen_pseudo_header_fields, method, hdr_validation_flags)?;

    Ok(headers.to_vec())
}

// Function to check pseudo-header field acceptability
fn check_pseudo_header_field_acceptability(pseudo_headers: &HashSet<String>, method: Option<String>, hdr_validation_flags: &HeaderValidationFlags) -> Result<(), ProtocolError> {
    if hdr_validation_flags.is_trailer && !pseudo_headers.is_empty() {
        return Err(ProtocolError(format!("Received pseudo-header in trailer {:?}", pseudo_headers)));
    }

    if hdr_validation_flags.is_response_header {
        assert_header_in_set(":status", b":status", pseudo_headers)?;
        for invalid_header in pseudo_headers.intersection(&REQUEST_ONLY_HEADERS) {
            return Err(ProtocolError(format!("Encountered request-only headers {:?}", invalid_header)));
        }
    } else if !hdr_validation_flags.is_response_header {
        assert_header_in_set(":path", b":path", pseudo_headers)?;
        assert_header_in_set(":method", b":method", pseudo_headers)?;
        assert_header_in_set(":scheme", b":scheme", pseudo_headers)?;

        for invalid_header in pseudo_headers.intersection(&RESPONSE_ONLY_HEADERS) {
            return Err(ProtocolError(format!("Encountered response-only headers {:?}", invalid_header)));
        }

        if let Some(method) = method {
            if method != "CONNECT" {
                for invalid_header in pseudo_headers.intersection(&REQUEST_ONLY_HEADERS) {
                    return Err(ProtocolError(format!("Encountered connect-request-only headers {:?}", invalid_header)));
                }
            }
        }
    }

    Ok(())
}

// Header validation flags struct
pub struct HeaderValidationFlags {
    pub is_client: bool,
    pub is_trailer: bool,
    pub is_response_header: bool,
    pub is_push_promise: bool,
}

// Normalize headers (example)
fn normalize_outbound_headers(headers: &[(String, String)], hdr_validation_flags: &HeaderValidationFlags) -> Vec<(String, String)> {
    headers.iter()
        .map(|(key, value)| {
            let mut normalized_key = key.to_lowercase();
            let mut normalized_value = value.trim().to_string();

            if normalized_key == "cookie" {
                // Cookie handling logic can be added here
            }

            (normalized_key, normalized_value)
        })
        .collect()
}

// Example of using the functions
fn main() {
    let headers = vec![
        (":method".to_string(), "GET".to_string()),
        (":path".to_string(), "/index".to_string())
    ];

    let hdr_validation_flags = HeaderValidationFlags {
        is_client: true,
        is_trailer: false,
        is_response_header: false,
        is_push_promise: false,
    };

    match reject_pseudo_header_fields(&headers, &hdr_validation_flags) {
        Ok(validated_headers) => println!("Validated Headers: {:?}", validated_headers),
        Err(e) => println!("Protocol Error: {:?}", e),
    }
}

