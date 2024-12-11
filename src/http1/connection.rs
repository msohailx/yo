use std::collections::{HashMap, VecDeque};
use std::fmt::Debug;

/// The default maximum size for incomplete events, set to 16 KB.
pub const DEFAULT_MAX_INCOMPLETE_EVENT_SIZE: usize = 16 * 1024;

/// Sentinel values used to represent certain states in a process.
/// 
/// # Example
/// 
/// ```rust
/// let state = Sentinel::NeedData;
/// assert_eq!(state, Sentinel::NeedData);
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Sentinel {
    NeedData,
    Paused,
}

/// Roles for either a client or server.
/// 
/// # Example
/// 
/// ```rust
/// let role = Role::Client;
/// assert_eq!(role, Role::Client);
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Role {
    Client,
    Server,
}

/// Represents various states a connection can be in.
/// 
/// # Example
/// 
/// ```rust
/// let state = ConnectionState::Idle;
/// assert_eq!(state, ConnectionState::Idle);
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConnectionState {
    Done,
    Error,
    Idle,
    MightSwitchProtocol,
    SwitchedProtocol,
    SendBody,
}

/// Represents different types of events in the system.
/// 
/// # Example
/// 
/// ```rust
/// let event = Event::Data(vec![1, 2, 3]);
/// assert_eq!(event, Event::Data(vec![1, 2, 3]));
/// ```
#[derive(Debug, Clone)]
pub enum Event {
    ConnectionClosed,
    Data(Vec<u8>),
    EndOfMessage,
    Request(Request),
    Response(Response),
    InformationalResponse(InformationalResponse),
}

/// Represents an HTTP request with method, headers, and HTTP version.
/// 
/// # Fields:
/// - `method`: HTTP method (e.g., GET, POST).
/// - `headers`: Headers as key-value pairs.
/// - `http_version`: HTTP version (e.g., HTTP/1.1).
/// 
/// # Example
/// 
/// ```rust
/// let request = Request {
///     method: b"GET".to_vec(),
///     headers: HashMap::new(),
///     http_version: b"HTTP/1.1".to_vec(),
/// };
/// ```
#[derive(Debug, Clone)]
pub struct Request {
    pub method: Vec<u8>,
    pub headers: HashMap<Vec<u8>, Vec<u8>>,
    pub http_version: Vec<u8>,
}

/// Represents an HTTP response with status code, headers, reason, and HTTP version.
/// 
/// # Fields:
/// - `status_code`: HTTP status code (e.g., 200, 404).
/// - `headers`: Response headers as key-value pairs.
/// - `reason`: Reason phrase for the status code.
/// - `http_version`: HTTP version (e.g., HTTP/1.1).
/// 
/// # Example
/// 
/// ```rust
/// let response = Response {
///     status_code: 200,
///     headers: HashMap::new(),
///     reason: b"OK".to_vec(),
///     http_version: b"HTTP/1.1".to_vec(),
/// };
/// ```
#[derive(Debug, Clone)]
pub struct Response {
    pub status_code: u16,
    pub headers: HashMap<Vec<u8>, Vec<u8>>,
    pub reason: Vec<u8>,
    pub http_version: Vec<u8>,
}

/// Represents an informational HTTP response with status code, headers, and reason.
/// 
/// # Fields:
/// - `status_code`: HTTP status code (e.g., 100).
/// - `headers`: Response headers as key-value pairs.
/// - `reason`: Reason phrase for the status code.
/// 
/// # Example
/// 
/// ```rust
/// let info_response = InformationalResponse {
///     status_code: 100,
///     headers: HashMap::new(),
///     reason: b"Continue".to_vec(),
/// };
/// ```
#[derive(Debug, Clone)]
pub struct InformationalResponse {
    pub status_code: u16,
    pub headers: HashMap<Vec<u8>, Vec<u8>>,
    pub reason: Vec<u8>,
}

/// A buffer for receiving and storing data with efficient extraction operations.
/// 
/// # Fields:
/// - `buffer`: A `VecDeque` holding the buffered data.
/// 
/// # Example
/// 
/// ```rust
/// let mut buffer = ReceiveBuffer { buffer: VecDeque::new() };
/// buffer.buffer.push_back(b'H');
/// buffer.buffer.push_back(b'E');
/// assert_eq!(buffer.buffer.len(), 2);
/// ```
pub struct ReceiveBuffer {
    buffer: VecDeque<u8>,
}

impl ReceiveBuffer {
    /// Creates a new, empty `ReceiveBuffer`.
    /// 
    /// # Example
    /// 
    /// ```rust
    /// let buffer = ReceiveBuffer::new();
    /// assert!(buffer.is_empty());
    /// ```
    pub fn new() -> Self {
        ReceiveBuffer {
            buffer: VecDeque::new(),
        }
    }
    /// Adds data to the buffer.
    /// 
    /// # Example
    /// 
    /// ```rust
    /// let mut buffer = ReceiveBuffer::new();
    /// buffer.add_data(&[1, 2, 3]);
    /// assert_eq!(buffer.len(), 3);
    /// ```
    pub fn add_data(&mut self, data: &[u8]) {
        self.buffer.extend(data);
    }
    /// Removes and returns the first byte from the buffer, or `None` if the buffer is empty.
    /// 
    /// # Example
    /// 
    /// ```rust
    /// let mut buffer = ReceiveBuffer::new();
    /// buffer.add_data(&[1, 2, 3]);
    /// assert_eq!(buffer.pop(), Some(1));
    /// assert_eq!(buffer.pop(), Some(2));
    /// assert_eq!(buffer.pop(), Some(3));
    /// assert_eq!(buffer.pop(), None);
    /// ```
    pub fn pop(&mut self) -> Option<u8> {
        self.buffer.pop_front()
    }
    /// Checks if the buffer is empty.
    /// 
    /// # Example
    /// 
    /// ```rust
    /// let mut buffer = ReceiveBuffer::new();
    /// assert!(buffer.is_empty());
    /// buffer.add_data(&[1, 2]);
    /// assert!(!buffer.is_empty());
    /// ```
    pub fn is_empty(&self) -> bool {
        self.buffer.is_empty()
    }
    /// Returns the number of bytes currently in the buffer.
    /// 
    /// # Example
    /// 
    /// ```rust
    /// let mut buffer = ReceiveBuffer::new();
    /// buffer.add_data(&[1, 2, 3]);
    /// assert_eq!(buffer.len(), 3);
    /// ```
    pub fn len(&self) -> usize {
        self.buffer.len()
    }
}

/// Retrieves a comma-separated list of values from the headers for the given key.
/// 
/// # Example
/// 
/// ```rust
/// let mut headers = HashMap::new();
/// headers.insert(b"key".to_vec(), b"value1,value2,value3".to_vec());
/// let values = get_comma_header(&headers, b"key");
/// assert_eq!(values, vec![b"value1".to_vec(), b"value2".to_vec(), b"value3".to_vec()]);
/// ```
pub fn get_comma_header(headers: &HashMap<Vec<u8>, Vec<u8>>, key: &[u8]) -> Vec<Vec<u8>> {
    headers
        .get(key)
        .map(|value| value.split(|&b| b == b',').map(|s| s.to_vec()).collect())
        .unwrap_or_default()
}
/// Sets a comma-separated list of values for the given key in the headers.
/// 
/// # Example
/// 
/// ```rust
/// let mut headers = HashMap::new();
/// set_comma_header(&mut headers, b"key", &[b"value1", b"value2", b"value3"]);
/// assert_eq!(headers.get(b"key"), Some(&b"value1,value2,value3"[..]));
/// ```
pub fn set_comma_header(
    headers: &mut HashMap<Vec<u8>, Vec<u8>>,
    key: &[u8],
    values: &[&[u8]],
) {
    headers.insert(key.to_vec(), values.join(&b", "[..]).to_vec());
}

/// Represents a connection with a client or server, managing roles, state, and communication.
/// 
/// # Example
/// 
/// ```rust
/// let connection = Connection::new(Role::Client, 16 * 1024);
/// assert_eq!(connection.our_role, Role::Client);
/// ```
pub struct Connection {
    our_role: Role,
    their_role: Role,
    max_incomplete_event_size: usize,
    cstate: ConnectionState,
    receive_buffer: ReceiveBuffer,
    receive_buffer_closed: bool,
    client_is_waiting_for_100_continue: bool,
    their_http_version: Option<Vec<u8>>,
    request_method: Option<Vec<u8>>,
}

impl Connection {
    /// Creates a new `Connection` with the given role and max incomplete event size.
    /// 
    /// # Example
    /// 
    /// ```rust
    /// let connection = Connection::new(Role::Client, 16 * 1024);
    /// assert_eq!(connection.our_role, Role::Client);
    /// ```
    pub fn new(our_role: Role, max_incomplete_event_size: usize) -> Self {
        let their_role = match our_role {
            Role::Client => Role::Server,
            Role::Server => Role::Client,
        };

        Connection {
            our_role,
            their_role,
            max_incomplete_event_size,
            cstate: ConnectionState::Idle,
            receive_buffer: ReceiveBuffer::new(),
            receive_buffer_closed: false,
            client_is_waiting_for_100_continue: false,
            their_http_version: None,
            request_method: None,
        }
    }
    /// Receives and stores incoming data in the connection's buffer.
    /// 
    /// # Example
    /// 
    /// ```rust
    /// let mut connection = Connection::new(Role::Client, 16 * 1024);
    /// connection.receive_data(b"Some data");
    /// assert!(!connection.receive_buffer.is_empty());
    /// ```
    pub fn receive_data(&mut self, data: &[u8]) -> Result<(), String> {
        if data.is_empty() {
            self.receive_buffer_closed = true;
        } else {
            self.receive_buffer.add_data(data);
        }
        Ok(())
    }
    /// Retrieves the next event for the connection based on its current state and data.
    /// 
    /// # Example
    /// 
    /// ```rust
    /// let mut connection = Connection::new(Role::Client, 16 * 1024);
    /// connection.receive_data(b"Some data");
    /// let event = connection.next_event();
    /// assert!(event.is_ok());
    /// ```
    pub fn next_event(&mut self) -> Result<Event, String> {
        if self.cstate == ConnectionState::Error {
            return Err("Cannot process events in the ERROR state.".to_string());
        }

        if self.receive_buffer.is_empty() && self.receive_buffer_closed {
            return Ok(Event::ConnectionClosed);
        }

        // Implement event extraction logic here
        Err("next_event not fully implemented".to_string())
    }
    /// Sends the given event by serializing it to bytes based on the connection's state.
    /// 
    /// # Example
    /// 
    /// ```rust
    /// let mut connection = Connection::new(Role::Client, 16 * 1024);
    /// let event = Event::Request(Request { method: b"GET".to_vec(), headers: HashMap::new(), http_version: b"HTTP/1.1".to_vec() });
    /// let result = connection.send(event);
    /// assert!(result.is_ok());
    /// ```
    pub fn send(&mut self, event: Event) -> Result<Vec<u8>, String> {
        if self.cstate == ConnectionState::Error {
            return Err("Cannot send data in the ERROR state.".to_string());
        }

        match event {
            Event::Request(req) => Ok(serialize_request(req)),
            Event::Response(res) => Ok(serialize_response(res)),
            _ => Err("Unsupported event type".to_string()),
        }
    }
}
/// Serializes an HTTP request into a byte vector.
///
/// # Example
/// 
/// ```rust
/// let request = Request {
///     method: b"GET".to_vec(),
///     headers: HashMap::new(),
///     http_version: b"1.1".to_vec(),
/// };
/// let serialized = serialize_request(request);
/// assert!(serialized.starts_with(b"GET"));
/// ```
fn serialize_request(request: Request) -> Vec<u8> {
    let mut output = Vec::new();
    output.extend(request.method);
    output.extend(b" / HTTP/");
    output.extend(request.http_version);
    output.extend(b"\r\n");
    for (key, value) in request.headers {
        output.extend(key);
        output.extend(b": ");
        output.extend(value);
        output.extend(b"\r\n");
    }
    output.extend(b"\r\n");
    output
}
/// Serializes an HTTP response into a byte vector.
///
/// # Example
/// 
/// ```rust
/// let response = Response {
///     status_code: 200,
///     reason: b"OK".to_vec(),
///     headers: HashMap::new(),
///     http_version: b"1.1".to_vec(),
/// };
/// let serialized = serialize_response(response);
/// assert!(serialized.starts_with(b"HTTP/"));
/// ```
fn serialize_response(response: Response) -> Vec<u8> {
    let mut output = Vec::new();
    output.extend(b"HTTP/");
    output.extend(response.http_version);
    output.extend(b" ");
    output.extend(response.status_code.to_string().as_bytes());
    output.extend(b" ");
    output.extend(response.reason);
    output.extend(b"\r\n");
    for (key, value) in response.headers {
        output.extend(key);
        output.extend(b": ");
        output.extend(value);
        output.extend(b"\r\n");
    }
    output.extend(b"\r\n");
    output
}

