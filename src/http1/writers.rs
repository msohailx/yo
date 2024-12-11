use std::collections::HashMap;
use std::io::Result;
use std::fmt::Debug;
use std::io;

// Define the Event enum, which can contain either Data or EndOfMessage (EOM) events
#[derive(Debug)]
pub enum Event {
    Data(Vec<u8>),
    EndOfMessage(Headers),
    // Add more variants as needed
}

// Define Headers struct, which holds a list of headers as tuples
#[derive(Debug)]
pub struct Headers {
    pub full_items: Vec<(Vec<u8>, Vec<u8>, Vec<u8>)>,
}

impl Headers {
    // Creates a new, empty `Headers` instance
    pub fn new() -> Self {
        Headers { full_items: vec![] }
    }
    
    // Adds a new header to the collection
    pub fn add(&mut self, raw_name: Vec<u8>, name: Vec<u8>, value: Vec<u8>) {
        self.full_items.push((raw_name, name, value));
    }
}

// Define the Request struct, which represents an HTTP request
#[derive(Debug)]
pub struct Request {
    pub method: Vec<u8>,
    pub target: Vec<u8>,
    pub http_version: Vec<u8>,
    pub headers: Headers,
}

// Define the Response struct, which represents an HTTP response
#[derive(Debug)]
pub struct Response {
    pub status_code: u16,
    pub reason: Vec<u8>,
    pub http_version: Vec<u8>,
    pub headers: Headers,
}

// Define the InformationalResponse struct, for HTTP 1xx responses
#[derive(Debug)]
pub struct InformationalResponse {
    pub status_code: u16,
    pub reason: Vec<u8>,
    pub http_version: Vec<u8>,
    pub headers: Headers,
}

// Define a Writer type alias that abstracts away the writing mechanism
pub type Writer = Box<dyn Fn(&[u8]) -> Result<()> + Send + Sync>;

// Define a custom error type for protocol errors
#[derive(Debug)]
pub struct LocalProtocolError(pub String);

// Implement the From trait to convert LocalProtocolError into io::Error
impl From<LocalProtocolError> for io::Error {
    fn from(err: LocalProtocolError) -> io::Error {
        io::Error::new(io::ErrorKind::Other, err.0)
    }
}

// Trait for writing bodies, e.g., in HTTP responses or requests
pub trait BodyWriter {
    fn send_data(&mut self, data: &[u8], write: &Writer) -> Result<()>;
    fn send_eom(&mut self, headers: Headers, write: &Writer) -> Result<()>;
}

// Writer for Content-Length encoding
pub struct ContentLengthWriter {
    length: usize,
}

impl ContentLengthWriter {
    pub fn new(length: usize) -> Self {
        ContentLengthWriter { length }
    }
}

impl BodyWriter for ContentLengthWriter {
    fn send_data(&mut self, data: &[u8], write: &Writer) -> Result<()> {
        if let Some(new_length) = self.length.checked_sub(data.len()) {
            self.length = new_length;
        } else {
            return Err(LocalProtocolError("Too much data for declared Content-Length".to_string()).into());
        }
        write(data)
    }

    fn send_eom(&mut self, headers: Headers, write: &Writer) -> Result<()> {
        if self.length != 0 {
            return Err(LocalProtocolError("Too little data for declared Content-Length".to_string()).into());
        }
        if !headers.full_items.is_empty() {
            return Err(LocalProtocolError("Content-Length and trailers don't mix".to_string()).into());
        }
        Ok(())
    }
}

// Chunked transfer encoding writer
pub struct ChunkedWriter;

impl BodyWriter for ChunkedWriter {
    fn send_data(&mut self, data: &[u8], write: &Writer) -> Result<()> {
        if data.is_empty() {
            return Ok(());
        }
        write(format!("{:x}\r\n", data.len()).as_bytes())?;
        write(data)?;
        write(b"\r\n")?;
        Ok(())
    }

    fn send_eom(&mut self, headers: Headers, write: &Writer) -> Result<()> {
        write(b"0\r\n")?;
        write(b"\r\n")?;
        Ok(())
    }
}

// HTTP/1.0 writer
pub struct Http10Writer;

impl BodyWriter for Http10Writer {
    fn send_data(&mut self, data: &[u8], write: &Writer) -> Result<()> {
        write(data)?;
        Ok(())
    }

    fn send_eom(&mut self, headers: Headers, write: &Writer) -> Result<()> {
        if !headers.full_items.is_empty() {
            return Err(LocalProtocolError("can't send trailers to HTTP/1.0 client".to_string()).into());
        }
        Ok(())
    }
}

// Define Sentinel enum, which represents different states in the protocol flow
#[derive(Debug, Clone, Copy, Eq, Hash, PartialEq)]
pub enum Sentinel {
    Client,
    Server,
    Idle,
    SendBody,
    SendResponse,
}

// Writers type maps tuples of Sentinels to their corresponding writer functions
type Writers = HashMap<(Sentinel, Sentinel), Box<dyn Fn(Event, Writer) -> Result<()> + Send + Sync>>;

// Function to write HTTP headers
pub fn write_headers(headers: &Headers, write: &Writer) -> Result<()> {
    for (raw_name, name, value) in &headers.full_items {
        if name == b"host" {
            write(format!("{}: {}\r\n", String::from_utf8_lossy(raw_name), String::from_utf8_lossy(value)).as_bytes())?;
        }
    }
    for (raw_name, name, value) in &headers.full_items {
        if name != b"host" {
            write(format!("{}: {}\r\n", String::from_utf8_lossy(raw_name), String::from_utf8_lossy(value)).as_bytes())?;
        }
    }
    write(b"\r\n")?;
    Ok(())
}

// Function to write HTTP request to the writer
pub fn write_request(request: &Request, write: &Writer) -> Result<()> {
    if request.http_version != b"1.1" {
        return Err(LocalProtocolError("I only send HTTP/1.1".to_string()).into());
    }
    write(format!("{} {} HTTP/1.1\r\n", String::from_utf8_lossy(&request.method), String::from_utf8_lossy(&request.target)).as_bytes())?;
    write_headers(&request.headers, write)
}

// Function to write HTTP response to the writer
pub fn write_any_response(response: &Response, write: &Writer) -> Result<()> {
    if response.http_version != b"1.1" {
        return Err(LocalProtocolError("I only send HTTP/1.1".to_string()).into());
    }
    write(format!("HTTP/1.1 {} {}\r\n", response.status_code, String::from_utf8_lossy(&response.reason)).as_bytes())?;
    write_headers(&response.headers, write)
}

// Function to create writers for different protocol states
pub fn create_writers() -> Writers {
    let mut writers: Writers = HashMap::new();

    // The closure signature has been modified to ensure it matches the expected `Box<dyn Fn(Event, Writer) -> Result<()> + Send + Sync>`
    writers.insert(
        (Sentinel::Client, Sentinel::Idle),
        Box::new(|event: Event, write: Writer| {
            match event {
                Event::Data(data) => {
                    if let Some(request) = parse_request(&data) {
                        if request.http_version != b"1.1" {
                            return Err(LocalProtocolError("I only send HTTP/1.1".to_string()).into());
                        }
                        write(format!("{} {} HTTP/1.1\r\n", String::from_utf8_lossy(&request.method), String::from_utf8_lossy(&request.target)).as_bytes())?;
                        write_headers(&request.headers, &write)?;
                    } else {
                        return Err(LocalProtocolError("Invalid request data".to_string()).into());
                    }
                }
                Event::EndOfMessage(headers) => {
                    write_headers(&headers, &write)?;
                }
            }
            Ok(())
        }) as Box<dyn Fn(Event, Writer) -> Result<()> + Send + Sync>,
    );

    writers.insert(
        (Sentinel::Server, Sentinel::Idle),
        Box::new(|event: Event, write: Writer| {
            match event {
                Event::Data(data) => {
                    // Handle server-side data
                }
                Event::EndOfMessage(headers) => {
                    write_headers(&headers, &write)?;
                }
            }
            Ok(())
        }) as Box<dyn Fn(Event, Writer) -> Result<()> + Send + Sync>,
    );

    // Add more writers for other states as needed

    writers
}

// Dummy function to simulate parsing request data (you can replace this with actual parsing logic)
fn parse_request(data: &[u8]) -> Option<Request> {
    Some(Request {
        method: b"GET".to_vec(),
        target: b"/index.html".to_vec(),
        http_version: b"1.1".to_vec(),
        headers: Headers::new(),
    })
}

