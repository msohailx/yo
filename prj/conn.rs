use std::collections::HashMap;
use std::fmt::Debug;
//use std::thread;
//use std::time::Duration;

use std::collections::VecDeque;

// Importing the necessary components from the provided file.
mod connection {
    use super::*;

    pub const DEFAULT_MAX_INCOMPLETE_EVENT_SIZE: usize = 16 * 1024;

    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub enum Sentinel {
        NeedData,
        Paused,
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub enum Role {
        Client,
        Server,
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub enum ConnectionState {
        Done,
        Error,
        Idle,
        MightSwitchProtocol,
        SwitchedProtocol,
        SendBody,
    }

    #[derive(Debug, Clone)]
    pub enum Event {
        ConnectionClosed,
        Data(Vec<u8>),
        EndOfMessage,
        Request(Request),
        Response(Response),
        InformationalResponse(InformationalResponse),
    }

    #[derive(Debug, Clone)]
    pub struct Request {
        pub method: Vec<u8>,
        pub headers: HashMap<Vec<u8>, Vec<u8>>,
        pub http_version: Vec<u8>,
    }

    #[derive(Debug, Clone)]
    pub struct Response {
        pub status_code: u16,
        pub headers: HashMap<Vec<u8>, Vec<u8>>,
        pub reason: Vec<u8>,
        pub http_version: Vec<u8>,
    }

    #[derive(Debug, Clone)]
    pub struct InformationalResponse {
        pub status_code: u16,
        pub headers: HashMap<Vec<u8>, Vec<u8>>,
        pub reason: Vec<u8>,
    }

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

            // Handle basic event processing for demonstration.
            Ok(Event::Data(self.receive_buffer.pop().map(|b| vec![b]).unwrap_or_default()))
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
}

use connection::*;

// Example of using the connection system with basic interactions.
fn main() {
    let mut client = Connection::new(Role::Client, DEFAULT_MAX_INCOMPLETE_EVENT_SIZE);
    let mut server = Connection::new(Role::Server, DEFAULT_MAX_INCOMPLETE_EVENT_SIZE);

    // Simulate sending a request from the client to the server.
    let request = Request {
        method: b"GET".to_vec(),
        headers: HashMap::new(),
        http_version: b"1.1".to_vec(),
    };

    let event = Event::Request(request);
    let serialized_request = client.send(event).unwrap();
    println!("Serialized request sent by client: {:?}", String::from_utf8_lossy(&serialized_request));

    // Simulate the server receiving the request.
    server.receive_data(&serialized_request).unwrap();
    let event_received = server.next_event().unwrap();
    println!("Server received event: {:?}", event_received);

    // Simulate the server sending a response back to the client.
    let response = Response {
        status_code: 200,
        reason: b"OK".to_vec(),
        headers: HashMap::new(),
        http_version: b"1.1".to_vec(),
    };
    let event = Event::Response(response);
    let serialized_response = server.send(event).unwrap();
    println!("Serialized response sent by server: {:?}", String::from_utf8_lossy(&serialized_response));

    // Simulate the client receiving the response.
    client.receive_data(&serialized_response).unwrap();
    let event_received = client.next_event().unwrap();
    println!("Client received event: {:?}", event_received);

    // Simulate connection closure after data exchange.
    client.receive_data(&[]).unwrap();
    let closure_event = client.next_event().unwrap();
    println!("Client receives connection closure event: {:?}", closure_event);
}

