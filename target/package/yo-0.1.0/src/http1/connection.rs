use std::collections::{HashMap, VecDeque};
use std::fmt::Debug;

// Constants
const DEFAULT_MAX_INCOMPLETE_EVENT_SIZE: usize = 16 * 1024;

// Sentinel enums
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Sentinel {
    NeedData,
    Paused,
}

// Role enums
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Role {
    Client,
    Server,
}

// State enums
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConnectionState {
    Done,
    Error,
    Idle,
    MightSwitchProtocol,
    SwitchedProtocol,
    SendBody,
}

// Event enums
#[derive(Debug, Clone)]
pub enum Event {
    ConnectionClosed,
    Data(Vec<u8>),
    EndOfMessage,
    Request(Request),
    Response(Response),
    InformationalResponse(InformationalResponse),
}

// Request structure
#[derive(Debug, Clone)]
pub struct Request {
    pub method: Vec<u8>,
    pub headers: HashMap<Vec<u8>, Vec<u8>>,
    pub http_version: Vec<u8>,
}

// Response structure
#[derive(Debug, Clone)]
pub struct Response {
    pub status_code: u16,
    pub headers: HashMap<Vec<u8>, Vec<u8>>,
    pub reason: Vec<u8>,
    pub http_version: Vec<u8>,
}

// InformationalResponse structure
#[derive(Debug, Clone)]
pub struct InformationalResponse {
    pub status_code: u16,
    pub headers: HashMap<Vec<u8>, Vec<u8>>,
    pub reason: Vec<u8>,
}

// ReceiveBuffer for storing data
pub struct ReceiveBuffer {
    buffer: VecDeque<u8>,
}

impl ReceiveBuffer {
    pub fn new() -> Self {
        ReceiveBuffer {
            buffer: VecDeque::new(),
        }
    }

    pub fn add_data(&mut self, data: &[u8]) {
        self.buffer.extend(data);
    }

    pub fn pop(&mut self) -> Option<u8> {
        self.buffer.pop_front()
    }

    pub fn is_empty(&self) -> bool {
        self.buffer.is_empty()
    }

    pub fn len(&self) -> usize {
        self.buffer.len()
    }
}

// Utility functions
fn get_comma_header(headers: &HashMap<Vec<u8>, Vec<u8>>, key: &[u8]) -> Vec<Vec<u8>> {
    headers
        .get(key)
        .map(|value| value.split(|&b| b == b',').map(|s| s.to_vec()).collect())
        .unwrap_or_default()
}

fn set_comma_header(
    headers: &mut HashMap<Vec<u8>, Vec<u8>>,
    key: &[u8],
    values: &[&[u8]],
) {
    headers.insert(key.to_vec(), values.join(&b", "[..]).to_vec());
}

// Connection structure
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

    pub fn receive_data(&mut self, data: &[u8]) -> Result<(), String> {
        if data.is_empty() {
            self.receive_buffer_closed = true;
        } else {
            self.receive_buffer.add_data(data);
        }
        Ok(())
    }

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

