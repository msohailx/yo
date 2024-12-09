use regex::Regex;

/// Optional Whitespace (OWS) – RFC 7230 Section 3.2.3
// like how much white space is permittible in the http header 
pub const OWS: &str = r"[ \t]*";

/// Token - RFC 7230 Section 3.2.6
/// defines the characters allowed in http fields
pub const TOKEN: &str = r"[-!#$%&'*+.^_`|~0-9a-zA-Z]+";

/// Field-name - RFC 7230 Section 3.2.6
/// we are using field_name because it can be reused and there are many
/// fields in http headers
pub const FIELD_NAME: &str = TOKEN;

/// Visible characters (vchar) - RFC 7230 Section 3.2.6
/// vchar : allows the characters from 0x21 to 0x7e in hexa, all the can be used characters 
/// char_or_obs_text : removes the \x00 (null character) and \s (whitespace character)
/// field_vchar : defines all the allowed characters 
pub const VCHAR: &str = r"[\x21-\x7e]";
pub const VCHAR_OR_OBS_TEXT: &str = r"[^\x00\s]";
pub const FIELD_VCHAR: &str = VCHAR_OR_OBS_TEXT;

/// Field content – RFC 7230 Section 3.2.6
/// field_content() : this helps us ensure that header values can contain multiple tokens with
/// spaces in between but still follow the rfc's guidelines
pub fn field_content() -> String {
    format!(r"{}+(?:[ \t]+{})*", FIELD_VCHAR)
}

/// Field value – RFC 7230 Section 3.2.6
/// field_value() : sometimes the content may get distributed to many lines because of the spaces
/// so we use this to help store here temporarily so that the folding can be handled somewhere else 
pub fn field_value() -> String {
    format!(r"({})?", field_content())
}

/// Header field – RFC 7230 Section 3.2.6
/// header_field() : it constructs an http field, so we use field_name, ows, :, field_value and
/// then again ows to generate a header field 
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
/// as defined above we use this to get the method from the http req , eg : get, post etc 
pub const METHOD: &str = TOKEN;

/// Request target – RFC 7230 Section 5.3
/// we use request_target to get the route of the req, like /home, http://example.com etc 
pub const REQUEST_TARGET: &str = r"[\x21-\x7e]+"; // vchar

/// HTTP Version – RFC 7230 Section 2.6
/// we use http_version to get the http version from the http req  
pub const HTTP_VERSION: &str = r"HTTP/(?P<http_version>[0-9]\.[0-9])";

/// Request line – RFC 7230 Section 3.1
/// request_line() : this function is used here to generate the http request line 
pub fn request_line() -> String {
    format!(
        r"(?P<method>{}) (?P<target>{}) {}",
        METHOD, REQUEST_TARGET, HTTP_VERSION
    )
}

/// Status code – RFC 7230 Section 3.2.6
/// status_code : checks where our code is 3 digits
pub const STATUS_CODE: &str = r"[0-9]{3}";

/// Reason phrase – RFC 7230 Section 3.2.6
/// reason_phrase : human readable explanation of the status code 
pub const REASON_PHRASE: &str = r"([ \t]|[^\x00\s])*";

/// Status line – RFC 7230 Section 3.1
/// status_line() : formats the string value of status_line 
pub fn status_line() -> String {
    format!(
        r"{} (?P<status_code>{}) (?: (?P<reason>{}))?",
        HTTP_VERSION, STATUS_CODE, REASON_PHRASE
    )
}

/// Chunk size – RFC 7230 Section 3.1.2.2
/// chunk-size is defined by the RFC as a hexadecimal number, but we limit the length of the chunk size to avoid excessive sizes. The regex matches a sequence of hexadecimal digits (between 1 and 20 characters).
pub const HEXDIG: &str = r"[0-9A-Fa-f]";
pub const CHUNK_SIZE: &str = r"({HEXDIG}){{1,20}}";

/// Chunk extension – RFC 7230 Section 3.1.2.2
/// so in http defined in rfc 7230 we use send data in chunks and also to send it we need to match
/// the chunk extensions, like param=value 
pub const CHUNK_EXT: &str = r";.*";

/// Chunk header – RFC 7230 Section 3.1.2.2
/// chunck_header() : defines the chunk header, meaning formats the chunk header into a string to
/// be able to send it 
pub fn chunk_header() -> String {
    format!(
        r"(?P<chunk_size>{}) (?P<chunk_ext>{})?{}\\r\\n",
        CHUNK_SIZE, CHUNK_EXT, OWS
    )
}
/// makes Reges::new(pattern) into a normal readable code so that we don't have to do it many times 
pub fn compile_regex(pattern: &str) -> Regex {
    Regex::new(pattern).expect("Invalid regex pattern")
}

#[cfg(test)]
mod tests {
    use super::*; // used to bring all the imported stuff into this mod's scope

    #[test]
    fn test_header_field() {
        let regex = compile_regex(&header_field());
        assert!(regex.is_match("Content-Type: text/html"));
    }

    #[test]
    fn test_request_line() {
        let regex = compile_regex(&request_line());
        assert!(regex.is_match("GET /index.html HTTP/1.1"));
    }

    #[test]
    fn test_status_line() {
        let regex = compile_regex(&status_line());
        assert!(regex.is_match("HTTP/1.1 200 OK"));
    }

    #[test]
    fn test_chunk_header() {
        let regex = compile_regex(&chunk_header());
        assert!(regex.is_match("1\r\nchunk data\r\n"));
    }
}

