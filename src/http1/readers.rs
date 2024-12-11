use std::collections::HashMap;

/// ReceiveBuffer struct is used to recieve the data into a buffer, in byte form.
#[derive(Debug)]
pub struct ReceiveBuffer {
    data: Vec<u8>,
}

impl ReceiveBuffer {
    /// to extract data from the buffer
    /// ```
    /// let mut buffer = ReceiveBuffer { data: vec![72, 101, 108, 108, 111] };
    /// let lines = buffer.maybe_extract_lines();
    /// assert_eq!(lines, Some(vec![vec![72, 101, 108, 108, 111]]));
    /// ```
    pub fn maybe_extract_lines(&mut self) -> Option<Vec<Vec<u8>>> {
        
        Some(vec![self.data.clone()])
    }
     /// Tries to extract at most `length` bytes from the buffer.
    /// If the buffer has enough data, it drains the first `length` bytes and returns them.
    /// If not, it returns `None`.
    ///
    /// # Arguments
    /// * `length` - The maximum number of bytes to extract from the buffer.
    ///
    /// # Returns
    /// Returns an `Option<Vec<u8>>` containing the extracted bytes if available, or `None` if there aren't enough bytes.
    ///
    /// # Example
    /// ```
    /// let mut buffer = ReceiveBuffer { data: vec![1, 2, 3, 4, 5] };
    /// let extracted = buffer.maybe_extract_at_most(3);
    /// assert_eq!(extracted, Some(vec![1, 2, 3]));
    /// ```
    pub fn maybe_extract_at_most(&mut self, length: usize) -> Option<Vec<u8>> {
        if self.data.len() >= length {
            Some(self.data.drain(0..length).collect())
        } else {
            None
        }
    }
     /// Checks if the next line in the buffer is obviously an invalid request line.
    ///
    /// # Returns
    /// Always returns `false` in this placeholder implementation.
    ///
    /// # Example
    /// ```
    /// let buffer = ReceiveBuffer { data: vec![72, 101, 108, 108, 111] };
    /// assert_eq!(buffer.is_next_line_obviously_invalid_request_line(), false);
    /// ```
    pub fn is_next_line_obviously_invalid_request_line(&self) -> bool {
        false
    }
    /// Attempts to extract the next line from the buffer. 
    /// It calls `maybe_extract_at_most` to extract all available bytes, treating them as one line.
    ///
    /// # Returns
    /// Returns an `Option<Vec<u8>>` containing the extracted line or `None` if there are no bytes left.
    ///
    /// # Example
    /// ```
    /// let mut buffer = ReceiveBuffer { data: vec![72, 101, 108, 108, 111] };
    /// let line = buffer.maybe_extract_next_line();
    /// assert_eq!(line, Some(vec![72, 101, 108, 108, 111]));
    /// ```
    pub fn maybe_extract_next_line(&mut self) -> Option<Vec<u8>> {
        self.maybe_extract_at_most(self.data.len())
    }
}
/// Implements the `Display` trait for `LocalProtocolError`.
/// This allows for more human-readable error messages when using `println!` or `format!`.
/// The `Display` trait formats the error as: `LocalProtocolError: <error_message>`.
///
/// # Example
/// 
/// Here's an example of how to use the `Display` trait to print the error:
/// 
/// ```rust
/// // Creating a LocalProtocolError with a custom error message.
/// let error = LocalProtocolError("Failed to parse local protocol message.".to_string());
/// 
/// // Printing the error using Display trait.
/// println!("{}", error);  // Output: LocalProtocolError: Failed to parse local protocol message.
/// ```
/// 
/// The `Display` trait allows the error to be formatted in a more user-friendly way.
#[derive(Debug)]
pub struct LocalProtocolError(pub String);

impl std::fmt::Display for LocalProtocolError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "LocalProtocolError: {}", self.0)
    }
}
/// A struct representing an error specific to a remote protocol issue.
/// It holds a `String` that describes the error message.
///
/// # Example
/// 
/// Here's an example of how to use the `RemoteProtocolError` struct:
/// 
/// ```rust
/// // Creating a RemoteProtocolError with a custom error message.
/// let error = RemoteProtocolError("Connection lost while fetching remote data.".to_string());
/// 
/// // Printing the error using Debug trait.
/// println!("{:?}", error);  // Output: RemoteProtocolError("Connection lost while fetching remote data.")
/// ```
/// 
/// This struct is used to encapsulate error messages related to remote protocol issues.
/// The `Debug` trait is derived automatically, which allows the error to be printed using the `{:?}` format.
#[derive(Debug)]
pub struct RemoteProtocolError(pub String);

impl std::fmt::Display for RemoteProtocolError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "RemoteProtocolError: {}", self.0)
    }
}
/// Represents an HTTP request, including headers and parsed status.
/// 
/// # Fields:
/// - `headers`: A vector of header key-value pairs (`Vec<u8>`, `Vec<u8>`).
/// - `parsed`: A boolean indicating if the request has been parsed.
///
/// # Example
/// 
/// ```rust
/// let request = Request {
///     headers: vec![(b"Content-Type".to_vec(), b"application/json".to_vec())],
///     parsed: true,
/// };
/// println!("{}", request.parsed);  // Output: true
/// ```
pub struct Request {
    headers: Vec<(Vec<u8>, Vec<u8>)>,
    parsed: bool,
}
/// Represents an HTTP response, including status code, reason, headers, and parsed status.
/// 
/// # Fields:
/// - `status_code`: The HTTP status code.
/// - `reason`: The reason phrase associated with the status code.
/// - `http_version`: The HTTP version.
/// - `headers`: A vector of header key-value pairs (`Vec<u8>`, `Vec<u8>`).
/// - `parsed`: A boolean indicating if the response has been parsed.
///
/// # Example
/// 
/// ```rust
/// let response = Response {
///     status_code: 200,
///     reason: b"OK".to_vec(),
///     http_version: b"HTTP/1.1".to_vec(),
///     headers: vec![(b"Content-Type".to_vec(), b"text/html".to_vec())],
///     parsed: true,
/// };
/// println!("{}", response.status_code);  // Output: 200
/// ```
pub struct Response {
    status_code: u16,
    reason: Vec<u8>,
    http_version: Vec<u8>,
    headers: Vec<(Vec<u8>, Vec<u8>)>,
    parsed: bool,
}
/// Represents an informational HTTP response (status codes 1xx), including status code, reason, headers, and parsed status.
/// 
/// # Fields:
/// - `status_code`: The HTTP status code (usually in the 1xx range).
/// - `reason`: The reason phrase associated with the status code.
/// - `http_version`: The HTTP version.
/// - `headers`: A vector of header key-value pairs (`Vec<u8>`, `Vec<u8>`).
/// - `parsed`: A boolean indicating if the response has been parsed.
///
/// # Example
/// 
/// ```rust
/// let info_response = InformationalResponse {
///     status_code: 100,
///     reason: b"Continue".to_vec(),
///     http_version: b"HTTP/1.1".to_vec(),
///     headers: vec![(b"Content-Length".to_vec(), b"0".to_vec())],
///     parsed: true,
/// };
/// println!("{}", info_response.status_code);  // Output: 100
/// ```
pub struct InformationalResponse {
    status_code: u16,
    reason: Vec<u8>,
    http_version: Vec<u8>,
    headers: Vec<(Vec<u8>, Vec<u8>)>,
    parsed: bool,
}
/// Represents chunked data in an HTTP message, including the data and chunk boundaries.
/// 
/// # Fields:
/// - `data`: A vector of bytes representing the data chunk.
/// - `chunk_start`: A boolean indicating if this chunk is the start of a chunked message.
/// - `chunk_end`: A boolean indicating if this chunk is the end of a chunked message.
///
/// # Example
/// 
/// ```rust
/// let data = Data {
///     data: b"Hello".to_vec(),
///     chunk_start: true,
///     chunk_end: false,
/// };
/// println!("{}", data.chunk_start);  // Output: true
/// ```
#[derive(Debug)]
pub struct Data {
    data: Vec<u8>,
    chunk_start: bool,
    chunk_end: bool,
}
/// Represents the end of an HTTP message, including headers.
/// 
/// # Fields:
/// - `headers`: A vector of header key-value pairs (`Vec<u8>`, `Vec<u8>`).
///
/// # Example
/// 
/// ```rust
/// let end_of_message = EndOfMessage {
///     headers: vec![(b"Content-Type".to_vec(), b"application/json".to_vec())],
/// };
/// println!("{:?}", end_of_message.headers);  // Output: [("Content-Type", "application/json")]
/// ```
#[derive(Debug)]
pub struct EndOfMessage {
    headers: Vec<(Vec<u8>, Vec<u8>)>,
}
/// Reader implementation from the reader.rs file
pub trait Reader {
    fn read(&mut self, buf: &mut ReceiveBuffer) -> Option<Data>;
    fn read_eof(&mut self) -> Result<EndOfMessage, RemoteProtocolError>;
}
/// A reader that handles content with a specified length.
/// 
/// # Fields:
/// - `length`: Total content length.
/// - `remaining`: Remaining content length to be read.
///
/// # Example
/// 
/// ```rust
/// let reader = ContentLengthReader::new(100);
/// println!("{}", reader.remaining);  // Output: 100
/// ```
pub struct ContentLengthReader {
    length: usize,
    remaining: usize,
}
/// Creates a new `ContentLengthReader` with a specified total length.
    ///
    /// # Example
    /// 
    /// ```rust
    /// let reader = ContentLengthReader::new(200);
    /// println!("{}", reader.length);  // Output: 200
    /// ```
impl ContentLengthReader {
    pub fn new(length: usize) -> Self {
        ContentLengthReader {
            length,
            remaining: length,
        }
    }
}
/// Implements the `Reader` trait for `ContentLengthReader`.
/// 
/// # Methods:
/// - `read`: Reads data up to the remaining content length from the buffer.
/// - `read_eof`: Checks if the entire content has been read and handles EOF errors.
///
/// # Example
/// 
/// ```rust
/// let mut reader = ContentLengthReader::new(100);
/// let mut buffer = ReceiveBuffer { data: vec![1, 2, 3] };
/// 
/// // Reading data from the buffer.
/// if let Some(data) = reader.read(&mut buffer) {
///     println!("{:?}", data);
/// }
/// 
/// // Checking if the content is fully read.
/// match reader.read_eof() {
///     Ok(end_of_message) => println!("End of message: {:?}", end_of_message),
///     Err(e) => println!("Error: {}", e),
/// }
/// ```
impl Reader for ContentLengthReader {
    fn read(&mut self, buf: &mut ReceiveBuffer) -> Option<Data> {
        if self.remaining == 0 {
            return Some(Data {
                data: vec![],
                chunk_start: false,
                chunk_end: false,
            });
        }
        let data = buf.maybe_extract_at_most(self.remaining);
        match data {
            Some(data) => {
                self.remaining -= data.len();
                Some(Data {
                    data,
                    chunk_start: false,
                    chunk_end: self.remaining == 0,
                })
            }
            None => None,
        }
    }

    fn read_eof(&mut self) -> Result<EndOfMessage, RemoteProtocolError> {
        if self.remaining > 0 {
            Err(RemoteProtocolError(format!(
                "peer closed connection without sending complete message body (received {} bytes, expected {})",
                self.length - self.remaining,
                self.length
            )))
        } else {
            Ok(EndOfMessage {
                headers: vec![], // Simplified for illustration
            })
        }
    }
}
/// A reader for chunked transfer encoding in HTTP.
/// 
/// # Fields:
/// - `bytes_in_chunk`: The number of bytes in the current chunk.
/// - `bytes_to_discard`: Bytes to discard (e.g., for chunk trailers).
/// - `reading_trailer`: Flag indicating if the trailer is being read.
///
/// # Example
/// 
/// ```rust
/// let reader = ChunkedReader::new();
/// println!("{}", reader.bytes_in_chunk);  // Output: 0
/// ```
pub struct ChunkedReader {
    bytes_in_chunk: usize,
    bytes_to_discard: usize,
    reading_trailer: bool,
}
/// Creates a new `ChunkedReader` with initial values.
///
/// # Example
/// 
/// ```rust
/// let reader = ChunkedReader::new();
/// println!("{}", reader.reading_trailer);  // Output: false
/// ```
impl ChunkedReader {
    pub fn new() -> Self {
        ChunkedReader {
            bytes_in_chunk: 0,
            bytes_to_discard: 0,
            reading_trailer: false,
        }
    }
}
/// Implements the `Reader` trait for `ChunkedReader`.
/// 
/// # Methods:
/// - `read`: Reads chunked data from the buffer, processing the chunk and trailer phases.
/// - `read_eof`: Handles EOF by returning an error if the body is incomplete.
///
/// # Example
/// 
/// ```rust
/// let mut reader = ChunkedReader::new();
/// let mut buffer = ReceiveBuffer { data: vec![1, 2, 3] };
/// 
/// // Reading chunked data from the buffer.
/// if let Some(data) = reader.read(&mut buffer) {
///     println!("{:?}", data);
/// }
/// 
/// // Handling EOF error.
/// match reader.read_eof() {
///     Ok(end_of_message) => println!("End of message: {:?}", end_of_message),
///     Err(e) => println!("Error: {}", e),
/// }
/// ```
impl Reader for ChunkedReader {
    fn read(&mut self, buf: &mut ReceiveBuffer) -> Option<Data> {
        if self.reading_trailer {
            let lines = buf.maybe_extract_lines();
            if let Some(lines) = lines {
                return Some(Data {
                    data: vec![], // Placeholder for actual header parsing
                    chunk_start: false,
                    chunk_end: false,
                });
            }
        }
        if self.bytes_to_discard > 0 {
            let data = buf.maybe_extract_at_most(self.bytes_to_discard);
            if data.is_none() {
                return None;
            }
            self.bytes_to_discard -= data.unwrap().len();
            if self.bytes_to_discard > 0 {
                return None;
            }
        }

        if self.bytes_in_chunk == 0 {
            // Simulating chunk header processing
            self.bytes_in_chunk = 100; // Simulated chunk size for demonstration
        }

        if self.bytes_in_chunk > 0 {
            let data = buf.maybe_extract_at_most(self.bytes_in_chunk);
            match data {
                Some(data) => {
                    self.bytes_in_chunk -= data.len();
                    Some(Data {
                        data,
                        chunk_start: true,
                        chunk_end: self.bytes_in_chunk == 0,
                    })
                }
                None => None,
            }
        } else {
            None
        }
    }

    fn read_eof(&mut self) -> Result<EndOfMessage, RemoteProtocolError> {
        Err(RemoteProtocolError(
            "peer closed connection without sending complete message body".into(),
        ))
    }
}
/// Implements the `Reader` trait for `Http10Reader`.
/// 
/// # Methods:
/// - `read`: Reads data from the buffer with a large fixed length (999999999 bytes).
/// - `read_eof`: Returns an `EndOfMessage` when EOF is reached.
///
/// # Example
/// 
/// ```rust
/// let mut reader = Http10Reader;
/// let mut buffer = ReceiveBuffer { data: vec![1, 2, 3] };
/// 
/// // Reading data from the buffer.
/// if let Some(data) = reader.read(&mut buffer) {
///     println!("{:?}", data);
/// }
/// 
/// // Handling EOF.
/// match reader.read_eof() {
///     Ok(end_of_message) => println!("End of message: {:?}", end_of_message),
///     Err(e) => println!("Error: {}", e),
/// }
/// ```
pub struct Http10Reader;

impl Reader for Http10Reader {
    fn read(&mut self, buf: &mut ReceiveBuffer) -> Option<Data> {
        let data = buf.maybe_extract_at_most(999999999);
        data.map(|data| Data {
            data,
            chunk_start: false,
            chunk_end: false,
        })
    }

    fn read_eof(&mut self) -> Result<EndOfMessage, RemoteProtocolError> {
        Ok(EndOfMessage {
            headers: vec![], // Simplified for demonstration
        })
    }
}
/// Type alias for a hashmap storing reader objects, keyed by a tuple of strings.
/// 
/// # Example
/// 
/// ```rust
/// let mut readers: ReadersType = HashMap::new();
/// readers.insert(("CLIENT".into(), "IDLE".into()), Box::new(ContentLengthReader::new(100)));
/// ```
pub fn expect_nothing(buf: &mut ReceiveBuffer) {
    if !buf.data.is_empty() {
        panic!("Got data when expecting EOF");
    }
}
/// Creates and returns a `ReadersType` hashmap with predefined reader instances.
/// 
/// # Example
/// 
/// ```rust
/// let readers = build_readers();
/// // Access a reader for a specific state.
/// let reader = readers.get(&("CLIENT".into(), "IDLE".into()));
/// ```
pub type ReadersType = HashMap<(String, String), Box<dyn Reader>>;

pub fn build_readers() -> ReadersType {
    let mut readers: ReadersType = HashMap::new();

    readers.insert(("CLIENT".into(), "IDLE".into()), Box::new(ContentLengthReader::new(100)));
    readers.insert(("SERVER".into(), "IDLE".into()), Box::new(ChunkedReader::new()));
    readers.insert(("SERVER".into(), "SEND_RESPONSE".into()), Box::new(Http10Reader));

    readers
}

