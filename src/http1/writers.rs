use std::collections::HashMap;
use std::io::{Write, Result};
use std::sync::Arc;
use std::fmt::Debug;

#[derive(Debug)]
pub enum Event {
    Data(Vec<u8>),
    EndOfMessage(Headers),
    // Add more variants as needed
}

pub struct Headers {
    pub full_items: Vec<(Vec<u8>, Vec<u8>, Vec<u8>)>,
}

impl Headers {
    pub fn new() -> Self {
        Headers { full_items: vec![] }
    }

    pub fn add(&mut self, raw_name: Vec<u8>, name: Vec<u8>, value: Vec<u8>) {
        self.full_items.push((raw_name, name, value));
    }
}

#[derive(Debug)]
pub struct Request {
    pub method: Vec<u8>,
    pub target: Vec<u8>,
    pub http_version: Vec<u8>,
    pub headers: Headers,
}

#[derive(Debug)]
pub struct Response {
    pub status_code: u16,
    pub reason: Vec<u8>,
    pub http_version: Vec<u8>,
    pub headers: Headers,
}

#[derive(Debug)]
pub struct InformationalResponse {
    pub status_code: u16,
    pub reason: Vec<u8>,
    pub http_version: Vec<u8>,
    pub headers: Headers,
}

pub type Writer = Box<dyn Fn(&[u8]) -> Result<()> + Send + Sync>;

#[derive(Debug)]
pub struct LocalProtocolError(pub String);

pub trait BodyWriter {
    fn send_data(&mut self, data: &[u8], write: &Writer) -> Result<()>;
    fn send_eom(&mut self, headers: Headers, write: &Writer) -> Result<()>;
}

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
        self.length -= data.len();
        if self.length < 0 {
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
        // Send headers here (like Content-Length)
        write(b"\r\n")?;
        Ok(())
    }
}

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

#[derive(Debug, Clone, Copy)]
pub enum Sentinel {
    Client,
    Server,
    Idle,
    SendBody,
    SendResponse,
}

type Writers = HashMap<(Sentinel, Sentinel), Box<dyn Fn(Event, Writer) -> Result<()> + Send + Sync>>;

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

pub fn write_request(request: &Request, write: &Writer) -> Result<()> {
    if request.http_version != b"1.1" {
        return Err(LocalProtocolError("I only send HTTP/1.1".to_string()).into());
    }
    write(format!("{} {} HTTP/1.1\r\n", String::from_utf8_lossy(&request.method), String::from_utf8_lossy(&request.target)).as_bytes())?;
    write_headers(&request.headers, write)
}

pub fn write_any_response(response: &Response, write: &Writer) -> Result<()> {
    if response.http_version != b"1.1" {
        return Err(LocalProtocolError("I only send HTTP/1.1".to_string()).into());
    }
    write(format!("HTTP/1.1 {} {}\r\n", response.status_code, String::from_utf8_lossy(&response.reason)).as_bytes())?;
    write_headers(&response.headers, write)
}

pub fn create_writers() -> Writers {
    let mut writers: Writers = HashMap::new();

    writers.insert(
        (Sentinel::Client, Sentinel::Idle),
        Box::new(write_request) as Box<dyn Fn(Event, Writer) -> Result<()>>,
    );

    writers.insert(
        (Sentinel::Server, Sentinel::Idle),
        Box::new(write_any_response) as Box<dyn Fn(Event, Writer) -> Result<()>>,
    );

    writers.insert(
        (Sentinel::Server, Sentinel::SendResponse),
        Box::new(write_any_response) as Box<dyn Fn(Event, Writer) -> Result<()>>,
    );

    // Add body writers as examples
    let mut send_body_writers: HashMap<String, Box<dyn BodyWriter + Send>> = HashMap::new();
    send_body_writers.insert("chunked".to_string(), Box::new(ChunkedWriter));
    send_body_writers.insert("content-length".to_string(), Box::new(ContentLengthWriter::new(0)));
    send_body_writers.insert("http/1.0".to_string(), Box::new(Http10Writer));

    writers.insert(
        (Sentinel::SendBody, Sentinel::Idle),
        Box::new(move |event, write| {
            // Implement body writers logic here.
            Ok(())
        }) as Box<dyn Fn(Event, Writer) -> Result<()>>,
    );

    writers
}


