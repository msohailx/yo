use std::fmt;

// Top-level H2 Error
#[derive(Debug)]
pub enum H2Error {
    ProtocolError(ProtocolError),
    Rfc1122Error,
    DenialOfServiceError {
        details: String,
    },
}

// Specific Protocol Errors
#[derive(Debug)]
pub enum ProtocolError {
    FrameTooLarge,
    FrameDataMissing,
    TooManyStreams,
    FlowControl,
    StreamIdTooLow {
        stream_id: u32,
        max_stream_id: u32,
    },
    NoAvailableStreamId,
    NoSuchStream {
        stream_id: u32,
    },
    StreamClosed {
        stream_id: u32,
        error_code: u32,
        events: Vec<String>,
    },
    InvalidSettingsValue {
        message: String,
        error_code: u32,
    },
    InvalidBodyLength {
        expected: usize,
        actual: usize,
    },
    UnsupportedFrame,
}

// Implement Display for ProtocolError
impl fmt::Display for ProtocolError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ProtocolError::FrameTooLarge => write!(f, "Frame is too large."),
            ProtocolError::FrameDataMissing => write!(f, "Frame data is missing."),
            ProtocolError::TooManyStreams => write!(f, "Too many concurrent streams."),
            ProtocolError::FlowControl => write!(f, "Flow control error."),
            ProtocolError::StreamIdTooLow { stream_id, max_stream_id } => write!(
                f,
                "Stream ID {} is lower than the maximum seen ID {}.",
                stream_id, max_stream_id
            ),
            ProtocolError::NoAvailableStreamId => write!(f, "No available stream IDs."),
            ProtocolError::NoSuchStream { stream_id } => {
                write!(f, "No such stream exists with ID {}.", stream_id)
            }
            ProtocolError::StreamClosed {
                stream_id,
                error_code,
                ..
            } => write!(
                f,
                "Stream {} is closed. Error code: {}.",
                stream_id, error_code
            ),
            ProtocolError::InvalidSettingsValue { message, error_code } => write!(
                f,
                "Invalid settings value: {}. Error code: {}.",
                message, error_code
            ),
            ProtocolError::InvalidBodyLength { expected, actual } => write!(
                f,
                "Invalid body length: Expected {}, received {}.",
                expected, actual
            ),
            ProtocolError::UnsupportedFrame => write!(f, "Unsupported frame received."),
        }
    }
}

// Implement Display for H2Error
impl fmt::Display for H2Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            H2Error::ProtocolError(err) => write!(f, "Protocol error: {}", err),
            H2Error::Rfc1122Error => write!(f, "RFC1122 violation detected."),
            H2Error::DenialOfServiceError { details } => {
                write!(f, "Denial of Service detected: {}", details)
            }
        }
    }
}

// Example usage
fn perform_action(stream_id: u32, max_stream_id: u32) -> Result<(), H2Error> {
    if stream_id < max_stream_id {
        return Err(H2Error::ProtocolError(ProtocolError::StreamIdTooLow {
            stream_id,
            max_stream_id,
        }));
    }
    Ok(())
}

fn main() {
    match perform_action(1, 10) {
        Ok(_) => println!("Action succeeded."),
        Err(e) => println!("Error: {}", e),
    }
}

