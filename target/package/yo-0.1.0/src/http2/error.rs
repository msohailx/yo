/// Global error code registry containing the established HTTP/2 error codes.
/// The current registry is available at: https://tools.ietf.org/html/rfc7540#section-11.4

#[derive(Debug)]
pub enum ErrorCodes {
    NoError = 0x0,
    ProtocolError = 0x1,
    InternalError = 0x2,
    FlowControlError = 0x3,
    SettingsTimeout = 0x4,
    StreamClosed = 0x5,
    FrameSizeError = 0x6,
    RefusedStream = 0x7,
    Cancel = 0x8,
    CompressionError = 0x9,
    ConnectError = 0xa,
    EnhanceYourCalm = 0xb,
    InadequateSecurity = 0xc,
    Http1_1Required = 0xd,
}

impl ErrorCodes {
    /// Converts an integer to its corresponding `ErrorCodes` variant.
    /// Returns the integer if the code is not part of the known set.
    pub fn from_int(code: u32) -> Self {
        match code {
            0x0 => ErrorCodes::NoError,
            0x1 => ErrorCodes::ProtocolError,
            0x2 => ErrorCodes::InternalError,
            0x3 => ErrorCodes::FlowControlError,
            0x4 => ErrorCodes::SettingsTimeout,
            0x5 => ErrorCodes::StreamClosed,
            0x6 => ErrorCodes::FrameSizeError,
            0x7 => ErrorCodes::RefusedStream,
            0x8 => ErrorCodes::Cancel,
            0x9 => ErrorCodes::CompressionError,
            0xa => ErrorCodes::ConnectError,
            0xb => ErrorCodes::EnhanceYourCalm,
            0xc => ErrorCodes::InadequateSecurity,
            0xd => ErrorCodes::Http1_1Required,
            _ => {
                // Return the raw value if it does not match any known error code
                panic!("Unknown error code: {:#x}", code);
            }
        }
    }
}

// Public export of the ErrorCodes enum
pub use ErrorCodes;

