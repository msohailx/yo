use std::collections::VecDeque;
use std::str;
/// Regular expression pattern for matching blank lines (two consecutive newlines).
/// 
/// # Example
/// 
/// ```rust
/// let regex = Regex::new(BLANK_LINE_REGEX).unwrap();
/// assert!(regex.is_match("\r\n\r\n"));
/// ```
const BLANK_LINE_REGEX: &str = r"\r?\n\r?\n";  // Use regex for matching blank lines
/// A buffer for receiving data, with efficient operations for appending, extracting, and searching.
///
/// # Fields:
/// - `data`: A deque to hold the buffered data.
/// - `next_line_search`: Index for searching the next line.
/// - `multiple_lines_search`: Index for searching multiple lines.
///
/// # Example
/// 
/// ```rust
/// let mut buffer = ReceiveBuffer::new();
/// buffer.append(b"Hello\r\nWorld\r\n\r\n");
/// let lines = buffer.maybe_extract_lines();
/// assert_eq!(lines, Some(vec![b"Hello".to_vec(), b"World".to_vec()]));
/// ```
pub struct ReceiveBuffer {
    data: VecDeque<u8>,  // Using VecDeque for efficient removal from the front
    next_line_search: usize,
    multiple_lines_search: usize,
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
        Self {
            data: VecDeque::new(),
            next_line_search: 0,
            multiple_lines_search: 0,
        }
    }

    /// Appends bytes to the buffer.
    /// 
    /// # Example
    /// 
    /// ```rust
    /// let mut buffer = ReceiveBuffer::new();
    /// buffer.append(b"Hello");
    /// assert_eq!(buffer.len(), 5);
    /// ```
    pub fn append(&mut self, bytes: &[u8]) {
        self.data.extend(bytes); 
    }

    /// Checks if the buffer is empty.
    /// 
    /// # Example
    /// 
    /// ```rust
    /// let buffer = ReceiveBuffer::new();
    /// assert!(buffer.is_empty());
    /// ```
    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }

    /// Returns the length of the buffer.
    /// 
    /// # Example
    /// 
    /// ```rust
    /// let mut buffer = ReceiveBuffer::new();
    /// buffer.append(b"Hello");
    /// assert_eq!(buffer.len(), 5);
    /// ```
    pub fn len(&self) -> usize {
        self.data.len()
    }

    /// Returns the buffer data as a byte slice.
    /// 
    /// # Example
    /// 
    /// ```rust
    /// let mut buffer = ReceiveBuffer::new();
    /// buffer.append(b"Hello");
    /// assert_eq!(buffer.as_bytes(), b"Hello");
    /// ```
    pub fn as_bytes(&self) -> &[u8] {
        &self.data.as_slices().0
    }

    /// Attempts to extract at most `count` bytes from the buffer.
    /// 
    /// # Example
    /// 
    /// ```rust
    /// let mut buffer = ReceiveBuffer::new();
    /// buffer.append(b"Hello");
    /// let data = buffer.maybe_extract_at_most(3);
    /// assert_eq!(data, Some(b"Hel".to_vec()));
    /// ```
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

    /// Try to extract at most 'count' bytes from the buffer
    pub fn maybe_extract_at_most(&mut self, count: usize) -> Option<Vec<u8>> {
        if self.data.len() < count {
            return None;
        }

        Some(self.extract(count))
    }

    /// Attempts to extract the next complete line (ends with `\r\n` or `\n`).
    /// 
    /// # Example
    /// 
    /// ```rust
    /// let mut buffer = ReceiveBuffer::new();
    /// buffer.append(b"Hello\r\nWorld");
    /// let line = buffer.maybe_extract_next_line();
    /// assert_eq!(line, Some(b"Hello".to_vec()));
    /// ```
    pub fn maybe_extract_next_line(&mut self) -> Option<Vec<u8>> {
        let search_start_index = self.next_line_search.saturating_sub(1);
        if let Some(pos) = self.find_line(search_start_index) {
            return Some(self.extract(pos + 2));  // skip \r\n
        }
        None
    }

    /// Attempts to extract lines up to the first blank line.
    /// 
    /// # Example
    /// 
    /// ```rust
    /// let mut buffer = ReceiveBuffer::new();
    /// buffer.append(b"Hello\r\nWorld\r\n\r\n");
    /// let lines = buffer.maybe_extract_lines();
    /// assert_eq!(lines, Some(vec![b"Hello".to_vec(), b"World".to_vec()]));
    /// ```
    pub fn maybe_extract_lines(&mut self) -> Option<Vec<Vec<u8>>> {
        // Handle immediate empty lines
        if self.data.len() > 0 && self.data[0] == b'\n' {
            self.extract(1);
            return Some(vec![]);
        }

        if self.data.len() > 1 && self.data[0] == b'\r' && self.data[1] == b'\n' {
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

    /// Helper function to search for the next line in the buffer
    fn find_line(&self, start: usize) -> Option<usize> {
        // CHANGE: Use `as_slices` to convert `VecDeque` to a slice, and apply `windows` to look for "\r\n"
        self.data.as_slices().0.windows(2)  // Accessing slice using `as_slices()`
            .position(|window| window == b"\r\n")
            .map(|pos| start + pos)
    }

    /// Helper function to find the first blank line
    fn find_blank_line(&self) -> Option<usize> {
        let data_str = str::from_utf8(&self.data.as_slices().0).ok()?;  
        data_str.find("\r\n\r\n").map(|pos| pos + 4)    
    }

    /// Split the data into lines by \n
    fn split_into_lines(&self, data: Vec<u8>) -> Vec<Vec<u8>> {
        let data_str = str::from_utf8(&data).expect("Invalid UTF-8 data");
        data_str
            .split('\n')
            .map(|line| line.as_bytes().to_vec())
            .collect()
    }

    /// Checks if the next line is an obviously invalid request line (non-printable characters).
    /// 
    /// # Example
    /// 
    /// ```rust
    /// let mut buffer = ReceiveBuffer::new();
    /// buffer.append(&[0x10]); // Non-printable character
    /// assert!(buffer.is_next_line_obviously_invalid_request_line());
    /// ```
    pub fn is_next_line_obviously_invalid_request_line(&self) -> bool {
        if let Some(&first_byte) = self.data.get(0) {
            first_byte < 0x21  // Check for non-printable characters
        } else {
            false
        }
    }
}

