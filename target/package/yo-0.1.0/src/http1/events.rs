use std::collections::HashMap;
use std::fmt::{Debug, Formatter, Result};
use std::str;

#[derive(Debug)]
pub enum Event {
    Request(Request),
    InformationalResponse(InformationalResponse),
    Response(Response),
    Data(Data),
    EndOfMessage(EndOfMessage),
    ConnectionClosed(ConnectionClosed),
}

#[derive(Debug, Clone)]
pub struct Headers(pub Vec<(Vec<u8>, Vec<u8>)>);

impl Headers {
    pub fn normalize_and_validate(headers: Vec<(Vec<u8>, Vec<u8>)>) -> Result<Self> {
        // Implement normalization and validation logic here
        Ok(Headers(headers))
    }
}

#[derive(Debug)]
pub struct LocalProtocolError(String);

impl std::fmt::Display for LocalProtocolError {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        write!(f, "{}", self.0)
    }
}

impl Event {
    pub fn validate_method(&self, method: &[u8]) -> Result<()> {
        // Implement method validation logic (e.g., regex)
        Ok(())
    }

    pub fn validate_target(&self, target: &[u8]) -> Result<()> {
        // Implement target validation logic (e.g., regex)
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
    ) -> Result<Self> {
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
    pub fn new(
        status_code: u16,
        headers: Vec<(Vec<u8>, Vec<u8>)>,
        reason: impl Into<Vec<u8>>,
        http_version: impl Into<Vec<u8>>,
    ) -> Result<Self> {
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
    pub fn new(
        status_code: u16,
        headers: Vec<(Vec<u8>, Vec<u8>)>,
        reason: impl Into<Vec<u8>>,
        http_version: impl Into<Vec<u8>>,
    ) -> Result<Self> {
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
    pub fn new() -> Self {
        ConnectionClosed
    }
}


