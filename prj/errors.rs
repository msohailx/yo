use std::collections::HashMap;
use std::fmt;
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
pub fn bytesify<T: AsRef<[u8]>>(input: T) -> Vec<u8> {
    input.as_ref().to_vec()
}

fn main() {
    // 1. Define a regex for email validation with capturing groups for local and domain parts
    let email_regex = Regex::new(r"(?i)^(?P<local>[\w\.-]+)@(?P<domain>[\w\.-]+\.\w+)$").unwrap();
    
    // 2. Test data: A valid email and an invalid email
    let valid_email = b"test@example.com";
    let invalid_email = b"invalid-email.com";
    
    // 3. Validate the valid email
    match validate(&email_regex, valid_email, "Invalid email format", &[]) {
        Ok(groups) => {
            println!("Email validated successfully! Captured groups: {:?}", groups);
        }
        Err(e) => {
            println!("Validation failed for valid email: {}", e);
        }
    }
    
    // 4. Validate the invalid email
    match validate(&email_regex, invalid_email, "Invalid email format", &[]) {
        Ok(groups) => {
            println!("Email validated successfully! Captured groups: {:?}", groups);
        }
        Err(e) => {
            println!("Validation failed for invalid email: {}", e);
        }
    }
    
    // 5. Demonstrate `bytesify` with ASCII-compatible input
    let input = "Hello, world!";
    let byte_vector = bytesify(input);
    println!("Bytes from input: {:?}", byte_vector);

    // 6. Demonstrate `SentinelType`
    let sentinel = SentinelType;
    println!("Sentinel: {}", sentinel);
    
    // 7. Create and display a ProtocolError
    let error = ProtocolError::new_local("Local error occurred", 500);
    println!("Error: {}", error);
}
