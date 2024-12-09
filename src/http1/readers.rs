use std::collections::HashMap;
use std::io::{self, Read};
use regex::Regex;
use std::str;

#[derive(Debug)]
pub struct ReceiveBuffer {
    data: Vec<u8>,
}

impl ReceiveBuffer {
    pub fn maybe_extract_lines(&mut self) -> Option<Vec<Vec<u8>>> {
        // Logic to extract lines from the buffer
        Some(vec![self.data.clone()])
    }

    pub fn maybe_extract_at_most(&mut self, length: usize) -> Option<Vec<u8>> {
        if self.data.len() >= length {
            Some(self.data.drain(0..length).collect())
        } else {
            None
        }
    }

    pub fn is_next_line_obviously_invalid_request_line(&self) -> bool {
        false
    }

    pub fn maybe_extract_next_line(&mut self) -> Option<Vec<u8>> {
        self.maybe_extract_at_most(self.data.len())
    }
}

#[derive(Debug)]
pub struct LocalProtocolError(pub String);

impl std::fmt::Display for LocalProtocolError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "LocalProtocolError: {}", self.0)
    }
}

#[derive(Debug)]
pub struct RemoteProtocolError(pub String);

impl std::fmt::Display for RemoteProtocolError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "RemoteProtocolError: {}", self.0)
    }
}

pub struct Request {
    headers: Vec<(Vec<u8>, Vec<u8>)>,
    parsed: bool,
}

pub struct Response {
    status_code: u16,
    reason: Vec<u8>,
    http_version: Vec<u8>,
    headers: Vec<(Vec<u8>, Vec<u8>)>,
    parsed: bool,
}

pub struct InformationalResponse {
    status_code: u16,
    reason: Vec<u8>,
    http_version: Vec<u8>,
    headers: Vec<(Vec<u8>, Vec<u8>)>,
    parsed: bool,
}

#[derive(Debug)]
pub struct Data {
    data: Vec<u8>,
    chunk_start: bool,
    chunk_end: bool,
}

#[derive(Debug)]
pub struct EndOfMessage {
    headers: Vec<(Vec<u8>, Vec<u8>)>,
}

pub trait Reader {
    fn read(&mut self, buf: &mut ReceiveBuffer) -> Option<Data>;
    fn read_eof(&mut self) -> Result<EndOfMessage, RemoteProtocolError>;
}

pub struct ContentLengthReader {
    length: usize,
    remaining: usize,
}

impl ContentLengthReader {
    pub fn new(length: usize) -> Self {
        ContentLengthReader {
            length,
            remaining: length,
        }
    }
}

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

pub struct ChunkedReader {
    bytes_in_chunk: usize,
    bytes_to_discard: usize,
    reading_trailer: bool,
}

impl ChunkedReader {
    pub fn new() -> Self {
        ChunkedReader {
            bytes_in_chunk: 0,
            bytes_to_discard: 0,
            reading_trailer: false,
        }
    }
}

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

pub fn expect_nothing(buf: &mut ReceiveBuffer) {
    if !buf.data.is_empty() {
        panic!("Got data when expecting EOF");
    }
}

pub type ReadersType = HashMap<(String, String), Box<dyn Reader>>;

pub fn build_readers() -> ReadersType {
    let mut readers: ReadersType = HashMap::new();

    readers.insert(("CLIENT".into(), "IDLE".into()), Box::new(ContentLengthReader::new(100)));
    readers.insert(("SERVER".into(), "IDLE".into()), Box::new(ChunkedReader::new()));
    readers.insert(("SERVER".into(), "SEND_RESPONSE".into()), Box::new(Http10Reader));

    readers
}

