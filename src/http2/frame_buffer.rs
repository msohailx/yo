use std::collections::VecDeque;
use std::error::Error;
use std::fmt;

use hyperframe::{Frame, HeadersFrame, ContinuationFrame, PushPromiseFrame};
use crate::exceptions::{ProtocolError, FrameTooLargeError, FrameDataMissingError};

const CONTINUATION_BACKLOG: usize = 64;

pub struct FrameBuffer {
    data: Vec<u8>,
    max_frame_size: usize,
    preamble: Vec<u8>,
    preamble_len: usize,
    headers_buffer: VecDeque<Box<dyn Frame>>,
}

impl FrameBuffer {
    pub fn new(server: bool) -> FrameBuffer {
        let preamble = if server {
            b"PRI * HTTP/2.0\r\n\r\nSM\r\n\r\n".to_vec()
        } else {
            Vec::new()
        };

        FrameBuffer {
            data: Vec::new(),
            max_frame_size: 0,
            preamble,
            preamble_len: preamble.len(),
            headers_buffer: VecDeque::new(),
        }
    }

    pub fn add_data(&mut self, data: &[u8]) -> Result<(), ProtocolError> {
        let mut data = data.to_vec();

        if self.preamble_len > 0 {
            let data_len = data.len();
            let preamble_len = self.preamble_len.min(data_len);

            if &self.preamble[0..preamble_len] != &data[0..preamble_len] {
                return Err(ProtocolError("Invalid HTTP/2 preamble".to_string()));
            }

            data = data[preamble_len..].to_vec();
            self.preamble_len -= preamble_len;
            self.preamble = self.preamble[preamble_len..].to_vec();
        }

        self.data.extend_from_slice(&data);
        Ok(())
    }

    fn validate_frame_length(&self, length: usize) -> Result<(), FrameTooLargeError> {
        if length > self.max_frame_size {
            return Err(FrameTooLargeError(format!(
                "Received overlong frame: length {}, max {}",
                length, self.max_frame_size
            )));
        }
        Ok(())
    }

    fn update_header_buffer(
        &mut self,
        frame: Option<Box<dyn Frame>>,
    ) -> Result<Option<Box<dyn Frame>>, ProtocolError> {
        if !self.headers_buffer.is_empty() {
            let stream_id = self.headers_buffer[0].stream_id();
            let valid_frame = match &frame {
                Some(f) => f.stream_id() == stream_id,
                None => false,
            };

            if !valid_frame {
                return Err(ProtocolError("Invalid frame during header block.".to_string()));
            }

            self.headers_buffer.push_back(frame.unwrap());
            if self.headers_buffer.len() > CONTINUATION_BACKLOG {
                return Err(ProtocolError("Too many continuation frames received.".to_string()));
            }

            if let Some(f) = &self.headers_buffer.back() {
                if f.flags().contains("END_HEADERS") {
                    let first_frame = self.headers_buffer[0].clone();
                    first_frame.add_flags("END_HEADERS");
                    let combined_data = self
                        .headers_buffer
                        .iter()
                        .map(|x| x.data().clone())
                        .collect::<Vec<Vec<u8>>>();
                    first_frame.set_data(combined_data.concat());
                    self.headers_buffer.clear();
                    return Ok(Some(first_frame));
                }
            }
            return Ok(None);
        } else if let Some(f) = &frame {
            if (f.is_header_frame() || f.is_push_promise_frame()) && !f.flags().contains("END_HEADERS") {
                self.headers_buffer.push_back(f.clone());
                return Ok(None);
            }
        }
        Ok(frame)
    }
}

impl Iterator for FrameBuffer {
    type Item = Option<Box<dyn Frame>>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.data.len() < 9 {
            return None;
        }

        let (frame, length) = match Frame::parse_frame_header(&self.data[0..9]) {
            Ok(result) => result,
            Err(e) => {
                return Some(Err(ProtocolError(format!(
                    "Received frame with invalid header: {}",
                    e
                ))));
            }
        };

        if self.data.len() < length + 9 {
            return None;
        }

        if let Err(e) = self.validate_frame_length(length) {
            return Some(Err(e));
        }

        match frame.parse_body(&self.data[9..9 + length]) {
            Ok(_) => (),
            Err(_) => {
                return Some(Err(FrameDataMissingError("Frame data missing or invalid".to_string())));
            }
        }

        self.data.drain(0..9 + length);

        let frame = match self.update_header_buffer(Some(frame)) {
            Ok(f) => f,
            Err(e) => return Some(Err(e)),
        };

        match frame {
            Some(f) => Some(Ok(f)),
            None => self.next(),
        }
    }
}

