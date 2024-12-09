use std::collections::VecDeque;
use std::str;

const BLANK_LINE_REGEX: &str = r"\r?\n\r?\n";  // Use regex for matching blank lines

pub struct ReceiveBuffer {
    data: VecDeque<u8>,  // Using VecDeque for efficient removal from the front
    next_line_search: usize,
    multiple_lines_search: usize,
}

impl ReceiveBuffer {
    pub fn new() -> Self {
        Self {
            data: VecDeque::new(),
            next_line_search: 0,
            multiple_lines_search: 0,
        }
    }

    // Append data to the buffer
    pub fn append(&mut self, bytes: &[u8]) {
        self.data.extend_from_slice(bytes);
    }

    // Check if the buffer is empty
    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }

    // Get the length of the buffer
    pub fn len(&self) -> usize {
        self.data.len()
    }

    // Return the data as a byte slice
    pub fn as_bytes(&self) -> &[u8] {
        &self.data
    }

    // Extract and return the first 'count' bytes from the buffer
    fn extract(&mut self, count: usize) -> Vec<u8> {
        let mut out = Vec::with_capacity(count);
        for _ in 0..count {
            if let Some(byte) = self.data.pop_front() {
                out.push(byte);
            }
        }

        self.next_line_search = 0;
        self.multiple_lines_search = 0;

        out
    }

    // Try to extract at most 'count' bytes from the buffer
    pub fn maybe_extract_at_most(&mut self, count: usize) -> Option<Vec<u8>> {
        if self.data.len() < count {
            return None;
        }

        Some(self.extract(count))
    }

    // Try to extract the first complete line (ends with \r\n or \n)
    pub fn maybe_extract_next_line(&mut self) -> Option<Vec<u8>> {
        let search_start_index = self.next_line_search.saturating_sub(1);
        if let Some(pos) = self.find_line(search_start_index) {
            return Some(self.extract(pos + 2));  // skip \r\n
        }
        None
    }

    // Try to extract all lines up to the first blank line
    pub fn maybe_extract_lines(&mut self) -> Option<Vec<Vec<u8>>> {
        // Handle immediate empty lines
        if self.data.starts_with(&[b'\n']) {
            self.extract(1);
            return Some(vec![]);
        }

        if self.data.starts_with(&[b'\r', b'\n']) {
            self.extract(2);
            return Some(vec![]);
        }

        // Look for a blank line
        if let Some(pos) = self.find_blank_line() {
            let extracted_data = self.extract(pos);
            let lines = self.split_into_lines(extracted_data);
            return Some(lines);
        }

        None
    }

    // Helper function to search for the next line in the buffer
    fn find_line(&self, start: usize) -> Option<usize> {
        self.data[start..]
            .windows(2)
            .position(|window| window == b"\r\n")
            .map(|pos| start + pos)
    }

    // Helper function to find the first blank line
    fn find_blank_line(&self) -> Option<usize> {
        let data_str = str::from_utf8(&self.data).ok()?;
        data_str.find("\r\n\r\n").map(|pos| pos + 4)  // `\r\n\r\n` is the blank line separator
    }

    // Split the data into lines by \n
    fn split_into_lines(&self, data: Vec<u8>) -> Vec<Vec<u8>> {
        let data_str = str::from_utf8(&data).expect("Invalid UTF-8 data");
        data_str
            .split('\n')
            .map(|line| line.as_bytes().to_vec())
            .collect()
    }

    // A sanity check for the request line
    pub fn is_next_line_obviously_invalid_request_line(&self) -> bool {
        if let Some(&first_byte) = self.data.get(0) {
            first_byte < 0x21  // Check for non-printable characters
        } else {
            false
        }
    }
}

