use regex::Regex;

/// Optional Whitespace (OWS) – RFC 7230 Section 3.2.3
pub const OWS: &str = r"[ \t]*";

/// Token – RFC 7230 Section 3.2.6
pub const TOKEN: &str = r"[-!#$%&'*+.^_`|~0-9a-zA-Z]+";

/// Field-name – RFC 7230 Section 3.2.6
pub const FIELD_NAME: &str = TOKEN;

/// Visible characters (vchar) – RFC 7230 Section 3.2.6
pub const VCHAR: &str = r"[\x21-\x7e]";
pub const FIELD_VCHAR: &str = r"[^\x00\s]";

/// Field content – RFC 7230 Section 3.2.6
pub fn field_content() -> String {
    format!(r"{}+(?:[ \t]+{})*", FIELD_VCHAR, FIELD_VCHAR)
}

/// Field value – RFC 7230 Section 3.2.6
pub fn field_value() -> String {
    format!(r"({})?", field_content())
}

/// Header field – RFC 7230 Section 3.2.6
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
pub const METHOD: &str = TOKEN;

/// Request target – RFC 7230 Section 5.3
pub const REQUEST_TARGET: &str = r"[\x21-\x7e]+"; // vchar

/// HTTP Version – RFC 7230 Section 2.6
pub const HTTP_VERSION: &str = r"HTTP/(?P<http_version>[0-9]\.[0-9])";

/// Request line – RFC 7230 Section 3.1
pub fn request_line() -> String {
    format!(
        r"(?P<method>{}) (?P<target>{}) {}",
        METHOD, REQUEST_TARGET, HTTP_VERSION
    )
}

/// Status code – RFC 7230 Section 3.2.6
pub const STATUS_CODE: &str = r"[0-9]{3}";

/// Reason phrase – RFC 7230 Section 3.2.6
pub const REASON_PHRASE: &str = r"([ \t]|[^\x00\s])*";

/// Status line – RFC 7230 Section 3.1
pub fn status_line() -> String {
    format!(
        r"{} (?P<status_code>{}) (?: (?P<reason>{}))?",
        HTTP_VERSION, STATUS_CODE, REASON_PHRASE
    )
}

/// Chunk size – RFC 7230 Section 3.1.2.2
pub const HEXDIG: &str = r"[0-9A-Fa-f]";
pub const CHUNK_SIZE: &str = r"({HEXDIG}){{1,20}}";

/// Chunk header – RFC 7230 Section 3.1.2.2
pub fn chunk_header() -> String {
    format!(
        r"(?P<chunk_size>{}) {}\\r\\n",
        CHUNK_SIZE, OWS
    )
}

/// Compile and return a regex for the given pattern.
pub fn compile_regex(pattern: &str) -> Regex {
    Regex::new(pattern).expect("Invalid regex pattern")
}

fn main() {
    // Example HTTP request line
    let request_line_regex = compile_regex(&request_line());
    let request = "GET /index.html HTTP/1.1";
    
    // Match the request line using the regex
    if let Some(captures) = request_line_regex.captures(request) {
        println!("Method: {}", &captures["method"]);
        println!("Request Target: {}", &captures["target"]);
        println!("HTTP Version: {}", &captures["http_version"]);
    }

    // Example HTTP status line
    let status_line_regex = compile_regex(&status_line());
    let response = "HTTP/1.1 200 OK";
    
    // Match the status line using the regex
    if let Some(captures) = status_line_regex.captures(response) {
        println!("HTTP Version: {}", &captures["http_version"]);
        println!("Status Code: {}", &captures["status_code"]);
        if let Some(reason) = captures.name("reason") {
            println!("Reason Phrase: {}", reason.as_str());
        }
    }

    // Example HTTP header field
    let header_field_regex = compile_regex(&header_field());
    let header = "Content-Type: text/html";
    
    // Match the header field using the regex
    if let Some(captures) = header_field_regex.captures(header) {
        println!("Field Name: {}", &captures["field_name"]);
        // Use .get() to safely access the capture group and handle None
        let field_value = captures.get(2).map_or("", |m| m.as_str());
        println!("Field Value: {}", field_value);
    }
    
    // Example chunk header
    let chunk_header_regex = compile_regex(&chunk_header());
    let chunk = "a1 ;extension\r\n";
    
    // Match the chunk header using the regex
    if let Some(captures) = chunk_header_regex.captures(chunk) {
        println!("Chunk Size: {}", &captures["chunk_size"]);
        if let Some(ext) = captures.name("chunk_ext") {
            println!("Chunk Extension: {}", ext.as_str());
        }
    }
}

