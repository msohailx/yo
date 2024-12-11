use std::collections::HashMap;
use std::fmt;
use std::sync::Arc;
use regex::Regex;

#[derive(Debug)]
pub enum ProtocolError {
    LocalProtocolError { message: String, error_status_hint: u16 },
    RemoteProtocolError { message: String, error_status_hint: u16 },
}

impl ProtocolError {
    /// Creates a new local protocol error.
    ///
    /// # Arguments
    /// * `message` - Error message.
    /// * `error_status_hint` - Error status code.
    ///
    /// # Returns
    /// A `ProtocolError::LocalProtocolError` variant.
    pub fn new_local(message: &str, error_status_hint: u16) -> Self {
        ProtocolError::LocalProtocolError {
            message: message.to_string(),
            error_status_hint,
        }
    }
    /// Creates a new remote protocol error.
    ///
    /// # Arguments
    /// * `message` - Error message.
    /// * `error_status_hint` - Error status code.
    ///
    /// # Returns
    /// A `ProtocolError::RemoteProtocolError` variant.
    pub fn new_remote(message: &str, error_status_hint: u16) -> Self {
        ProtocolError::RemoteProtocolError {
            message: message.to_string(),
            error_status_hint,
        }
    }
    /// Retrieves the error status hint.
    ///
    /// # Returns
    /// `u16` - The error status hint.
    pub fn error_status_hint(&self) -> u16 {
        match self {
            ProtocolError::LocalProtocolError { error_status_hint, .. } => *error_status_hint,
            ProtocolError::RemoteProtocolError { error_status_hint, .. } => *error_status_hint,
        }
    }
}

impl fmt::Display for ProtocolError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ProtocolError::LocalProtocolError { message, .. } => write!(f, "Local error: {}", message),
            ProtocolError::RemoteProtocolError { message, .. } => write!(f, "Remote error: {}", message),
        }
    }
}

impl std::error::Error for ProtocolError {}

/// Validates data against a regex pattern, returning captured groups as a hashmap.
///
/// # Arguments
/// * `regex` - The regex to validate against.
/// * `data` - The data to validate.
/// * `msg` - The error message to use if validation fails.
/// * `format_args` - Arguments to format into the message.
///
/// # Returns
/// `Ok(HashMap)` with captured group names and values on success, or a `ProtocolError` on failure.
pub fn validate(
    regex: &Regex,
    data: &[u8],
    msg: &str,
    format_args: &[String],
) -> Result<HashMap<String, String>, ProtocolError> {
    
    if let Ok(data_str) = std::str::from_utf8(data) {  
        if let Some(captures) = regex.captures(data_str) {  
            let mut group_dict = HashMap::new();
            
            
            for name in regex.capture_names() {  
                if let Some(name) = name {  
                    if let Some(value) = captures.name(name) {  
                        group_dict.insert(name.to_string(), value.as_str().to_string());
                    }
                }
            }
            Ok(group_dict)
        } else {
            let formatted_msg = if !format_args.is_empty() {
                format!("{} {}", msg, format_args.join(", "))
            } else {
                msg.to_string()
            };
            Err(ProtocolError::new_local(&formatted_msg, 400))
        }
    } else {
        
        Err(ProtocolError::new_local("Invalid UTF-8 input", 400))  
    }
}


/// Sentinel trait for types that act as sentinels.
///
/// This trait can be implemented by types that are used to signal specific conditions.
pub trait Sentinel: fmt::Debug + 'static {}

#[derive(Debug)]
pub struct SentinelType;

impl fmt::Display for SentinelType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "SentinelType")
    }
}

impl Sentinel for SentinelType {}

/// Converts the given input into a `Vec<u8>`, ensuring it is ASCII-compatible.
///
/// # Arguments
/// * `input` - The input data, which must be ASCII-compatible.
///
/// # Returns
/// `Vec<u8>` - The ASCII-compatible byte vector.
///
/// # Panics
/// If the input is not ASCII-compatible.
pub fn bytesify(input: impl Into<Arc<dyn AsRef<[u8]>>>) -> Vec<u8> {
    match input.into().as_ref().as_ref() {
        input if input.is_ascii() => input.to_vec(),
        _ => panic!("Invalid input: expected ASCII-compatible bytes"),
    }
}

