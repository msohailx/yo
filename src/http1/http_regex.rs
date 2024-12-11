use regex::Regex;

/// Optional Whitespace (OWS) – RFC 7230 Section 3.2.3
/// Represents zero or more spaces or tabs.
pub const OWS: &str = r"[ \t]*";

/// Token – RFC 7230 Section 3.2.6
/// Defines the allowed characters for tokens (e.g., HTTP method, header names).
pub const TOKEN: &str = r"[-!#$%&'*+.^_`|~0-9a-zA-Z]+";

/// Field-name – RFC 7230 Section 3.2.6
/// Defines a field name, which is a token.
pub const FIELD_NAME: &str = TOKEN;

/// Visible characters (vchar) – RFC 7230 Section 3.2.6
/// Represents visible ASCII characters (excluding control characters).
pub const VCHAR: &str = r"[\x21-\x7e]";
pub const VCHAR_OR_OBS_TEXT: &str = r"[^\x00\s]";
pub const FIELD_VCHAR: &str = VCHAR_OR_OBS_TEXT;

/// Field content – RFC 7230 Section 3.2.6
/// Represents the content of a field (may include multiple vchar-separated segments).
pub fn field_content() -> String {
    format!(r"{}+(?:[ \t]+{})*", FIELD_VCHAR, FIELD_VCHAR)
}

/// Field value – RFC 7230 Section 3.2.6
/// Represents an optional field value, which may be empty or contain field content.
pub fn field_value() -> String {
    format!(r"({})?", field_content())
}

/// Header field – RFC 7230 Section 3.2.6
/// Represents a complete header field (name + optional value with OWS).
pub fn header_field() -> String {
    format!(
        r"(?P<field_name>{}){}{}(?P<field_value>{}){}",
        FIELD_NAME, 
        OWS, 
        ":", 
        field_value(), 
        OWS
    )
}

/// HTTP Method – RFC 7230 Section 3.2.6
/// Defines valid HTTP methods (e.g., GET, POST).
pub const METHOD: &str = TOKEN;

/// Request target – RFC 7230 Section 5.3
/// Represents the target part of an HTTP request (e.g., `/index.html`).
pub const REQUEST_TARGET: &str = r"[\x21-\x7e]+"; // vchar

/// HTTP Version – RFC 7230 Section 2.6
/// Represents the HTTP version (e.g., HTTP/1.1).
pub const HTTP_VERSION: &str = r"HTTP/(?P<http_version>[0-9]\.[0-9])";

/// Request line – RFC 7230 Section 3.1
/// Represents the complete request line (method, target, version).
pub fn request_line() -> String {
    format!(
        r"(?P<method>{}) (?P<target>{}) {}",
        METHOD, REQUEST_TARGET, HTTP_VERSION
    )
}

/// Status code – RFC 7230 Section 3.2.6
/// Defines the format for HTTP status codes (3 digits).
pub const STATUS_CODE: &str = r"[0-9]{3}";

/// Reason phrase – RFC 7230 Section 3.2.6
/// Defines a valid reason phrase, which can include visible characters and spaces.
pub const REASON_PHRASE: &str = r"([ \t]|[^\x00\s])*";

/// Status line – RFC 7230 Section 3.1
/// Represents the status line of an HTTP response (version, status code, reason).
pub fn status_line() -> String {
    format!(
        r"{} (?P<status_code>{}) (?: (?P<reason>{}))?",
        HTTP_VERSION, STATUS_CODE, REASON_PHRASE
    )
}

/// Chunk size – RFC 7230 Section 3.1.2.2
/// Represents the chunk size in hexadecimal format (1 to 20 hex digits).
pub const HEXDIG: &str = r"[0-9A-Fa-f]";
pub const CHUNK_SIZE: &str = r"({HEXDIG}){{1,20}}";

/// Chunk extension – RFC 7230 Section 3.1.2.2
/// Represents optional extensions for chunks.
pub const CHUNK_EXT: &str = r";.*";

/// Chunk header – RFC 7230 Section 3.1.2.2
/// Represents the full chunk header (size + optional extension).
pub fn chunk_header() -> String {
    format!(
        r"(?P<chunk_size>{}) (?P<chunk_ext>{})?{}\\r\\n",
        CHUNK_SIZE, CHUNK_EXT, OWS
    )
}
/// Compiles a regex pattern and returns a `Regex` instance.
///
/// # Arguments
/// * `pattern` - The regex pattern to compile.
///
/// # Returns
/// A compiled `Regex` instance.
///
/// # Panics
/// If the pattern is invalid.
pub fn compile_regex(pattern: &str) -> Regex {
    Regex::new(pattern).expect("Invalid regex pattern")
}


