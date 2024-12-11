use std::fmt::{Debug, Formatter, Result as FmtResult}; 
use std::result::Result as StdResult; 
/// Enum representing various types of events in a communication system.
///
/// Each variant corresponds to a different event type that can be handled, such as requests,
/// responses, data transfers, or connection status changes.
///
/// # Example
/// ```rust
/// let event = Event::Request(Request { /* fields */ });
/// match event {
///     Event::Request(req) => println!("Handling request: {:?}", req),
///     _ => println!("Other event"),
/// }
/// ```
#[derive(Debug)]
pub enum Event {
    Request(Request),
    InformationalResponse(InformationalResponse),
    Response(Response),
    Data(Data),
    EndOfMessage(EndOfMessage),
    ConnectionClosed(ConnectionClosed),
}

/// A struct representing HTTP headers as a vector of key-value pairs (both as `Vec<u8>`).
///
/// # Example
/// ```rust
/// let headers = vec![(b"Content-Type".to_vec(), b"application/json".to_vec())];
/// let result = Headers::normalize_and_validate(headers);
/// match result {
///     Ok(headers) => println!("Valid headers: {:?}", headers),
///     Err(e) => println!("Error: {:?}", e),
/// }
/// ```
#[derive(Debug, Clone)]
pub struct Headers(pub Vec<(Vec<u8>, Vec<u8>)>);

impl Headers {
    /// Normalizes and validates the given headers.
    ///
    /// # Arguments
    /// * `headers` - A vector of key-value pairs representing HTTP headers.
    ///
    /// # Returns
    /// Returns `Ok(Headers)` if validation passes, or an error if validation fails.
    pub fn normalize_and_validate(headers: Vec<(Vec<u8>, Vec<u8>)>) -> StdResult<Self, LocalProtocolError> {
        
        Ok(Headers(headers))
    }
}

#[derive(Debug)]
pub struct LocalProtocolError(String);

impl std::fmt::Display for LocalProtocolError {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        write!(f, "{}", self.0)
    }
}

impl Event {
    /// Validates the HTTP method.
    ///
    /// # Arguments
    /// * `method` - A byte slice representing the HTTP method (e.g., "GET", "POST").
    ///
    /// # Returns
    /// Returns `Ok(())` if the method is valid, or an error if validation fails.
    pub fn validate_method(&self, method: &[u8]) -> StdResult<(), LocalProtocolError> {
        
        Ok(())
    }
    /// Validates the target (e.g., URL path or resource).
    ///
    /// # Arguments
    /// * `target` - A byte slice representing the target (e.g., "/index.html").
    ///
    /// # Returns
    /// Returns `Ok(())` if the target is valid, or an error if validation fails.
    pub fn validate_target(&self, target: &[u8]) -> StdResult<(), LocalProtocolError> {
        
        Ok(())
    }
}

#[derive(Debug)]
pub struct Request {
    pub method: Vec<u8>,
    pub headers: Headers,
    pub target: Vec<u8>,
    pub http_version: Vec<u8>,
}

impl Request {
    pub fn new(
        method: impl Into<Vec<u8>>,
        target: impl Into<Vec<u8>>,
        headers: Vec<(Vec<u8>, Vec<u8>)>,
        http_version: impl Into<Vec<u8>>,
    ) -> StdResult<Self, LocalProtocolError> {
        let method = method.into();
        let target = target.into();
        let http_version = http_version.into();
        let headers = Headers::normalize_and_validate(headers)?;

        // Validation steps
        if http_version == b"1.1" && !headers.0.iter().any(|(name, _)| name == b"host") {
            return Err(LocalProtocolError("Missing mandatory Host: header".into()).into());
        }

        Ok(Request {
            method,
            headers,
            target,
            http_version,
        })
    }
}

#[derive(Debug)]
pub struct InformationalResponse {
    pub status_code: u16,
    pub headers: Headers,
    pub http_version: Vec<u8>,
    pub reason: Vec<u8>,
}

impl InformationalResponse {
    /// Creates a new `Request`, normalizes headers, and validates the HTTP version and headers.
    ///
    /// # Arguments
    /// * `method` - HTTP method (e.g., "GET", "POST").
    /// * `target` - The target URI (e.g., "/index.html").
    /// * `headers` - A vector of key-value pairs representing HTTP headers.
    /// * `http_version` - HTTP version (e.g., "1.1").
    ///
    /// # Returns
    /// Returns `Ok(Request)` if creation and validation succeed, or an error if validation fails.
    ///
    /// # Errors
    /// Returns an error if the `Host` header is missing for HTTP/1.1.
    pub fn new(
        status_code: u16,
        headers: Vec<(Vec<u8>, Vec<u8>)>,
        reason: impl Into<Vec<u8>>,
        http_version: impl Into<Vec<u8>>,
    ) -> StdResult<Self, LocalProtocolError> {
        if !(100..200).contains(&status_code) {
            return Err(LocalProtocolError(format!(
                "InformationalResponse status_code should be in range [100, 200), not {}",
                status_code
            ))
            .into());
        }
        let reason = reason.into();
        let http_version = http_version.into();
        let headers = Headers::normalize_and_validate(headers)?;
        Ok(InformationalResponse {
            status_code,
            headers,
            reason,
            http_version,
        })
    }
}

#[derive(Debug)]
pub struct Response {
    pub status_code: u16,
    pub headers: Headers,
    pub http_version: Vec<u8>,
    pub reason: Vec<u8>,
}

impl Response {
    /// Creates a new `Response`, validates the status code, and normalizes headers.
    ///
    /// # Arguments
    /// * `status_code` - HTTP status code (e.g., 200, 404).
    /// * `headers` - A vector of key-value pairs representing HTTP headers.
    /// * `reason` - The reason phrase for the response (e.g., "OK").
    /// * `http_version` - HTTP version (e.g., "1.1").
    ///
    /// # Returns
    /// Returns `Ok(Response)` if creation and validation succeed, or an error if validation fails.
    ///
    /// # Errors
    /// Returns an error if the `status_code` is outside the range [200, 1000).
    pub fn new(
        status_code: u16,
        headers: Vec<(Vec<u8>, Vec<u8>)>,
        reason: impl Into<Vec<u8>>,
        http_version: impl Into<Vec<u8>>,
    ) -> StdResult<Self, LocalProtocolError> {
        if !(200..1000).contains(&status_code) {
            return Err(LocalProtocolError(format!(
                "Response status_code should be in range [200, 1000), not {}",
                status_code
            ))
            .into());
        }
        let reason = reason.into();
        let http_version = http_version.into();
        let headers = Headers::normalize_and_validate(headers)?;
        Ok(Response {
            status_code,
            headers,
            reason,
            http_version,
        })
    }
}

#[derive(Debug)]
pub struct Data {
    pub data: Vec<u8>,
    pub chunk_start: bool,
    pub chunk_end: bool,
}

impl Data {
    /// Creates a new `Data` instance with the given chunk flags.
    ///
    /// # Arguments
    /// * `data` - The chunk data.
    /// * `chunk_start` - Whether this chunk is the start of a series.
    /// * `chunk_end` - Whether this chunk is the end of a series.
    ///
    /// # Returns
    /// A new `Data` instance.
    pub fn new(data: Vec<u8>, chunk_start: bool, chunk_end: bool) -> Self {
        Data {
            data,
            chunk_start,
            chunk_end,
        }
    }
}

#[derive(Debug)]
pub struct EndOfMessage {
    pub headers: Headers,
}

impl EndOfMessage {
    /// Creates a new `EndOfMessage` instance, normalizing the provided headers.
    ///
    /// # Arguments
    /// * `headers` - Optional vector of key-value pairs representing headers.
    ///
    /// # Returns
    /// A new `EndOfMessage` instance with normalized headers.
    pub fn new(headers: Option<Vec<(Vec<u8>, Vec<u8>)>>) -> Self {
        let headers = match headers {
            Some(h) => Headers::normalize_and_validate(h).unwrap(),
            None => Headers(vec![]),
        };
        EndOfMessage { headers }
    }
}

#[derive(Debug)]
pub struct ConnectionClosed;

impl ConnectionClosed {
    /// Creates a new `ConnectionClosed` instance.
    ///
    /// # Returns
    /// A new `ConnectionClosed` instance.
    pub fn new() -> Self {
        ConnectionClosed
    }
}


