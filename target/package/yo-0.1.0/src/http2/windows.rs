const LARGEST_FLOW_CONTROL_WINDOW: u32 = 2u32.pow(31) - 1;

#[derive(Debug)]
pub struct FlowControlError(String);

pub struct WindowManager {
    max_window_size: u32,
    current_window_size: u32,
    bytes_processed: u32,
}

impl WindowManager {
    pub fn new(max_window_size: u32) -> Self {
        assert!(max_window_size <= LARGEST_FLOW_CONTROL_WINDOW);
        WindowManager {
            max_window_size,
            current_window_size: max_window_size,
            bytes_processed: 0,
        }
    }

    pub fn window_consumed(&mut self, size: u32) -> Result<(), FlowControlError> {
        self.current_window_size -= size;
        if self.current_window_size < 0 {
            return Err(FlowControlError("Flow control window shrunk below 0".to_string()));
        }
        Ok(())
    }

    pub fn window_opened(&mut self, size: u32) -> Result<(), FlowControlError> {
        self.current_window_size += size;
        if self.current_window_size > LARGEST_FLOW_CONTROL_WINDOW {
            return Err(FlowControlError(format!(
                "Flow control window mustn't exceed {}",
                LARGEST_FLOW_CONTROL_WINDOW
            )));
        }
        if self.current_window_size > self.max_window_size {
            self.max_window_size = self.current_window_size;
        }
        Ok(())
    }

    pub fn process_bytes(&mut self, size: u32) -> Option<u32> {
        self.bytes_processed += size;
        self.maybe_update_window()
    }

    fn maybe_update_window(&mut self) -> Option<u32> {
        if self.bytes_processed == 0 {
            return None;
        }

        let max_increment = self.max_window_size.saturating_sub(self.current_window_size);
        let mut increment = 0;

        if self.current_window_size == 0 && self.bytes_processed > self.max_window_size / 4 {
            increment = self.bytes_processed.min(max_increment);
            self.bytes_processed = 0;
        } else if self.bytes_processed >= self.max_window_size / 2 {
            increment = self.bytes_processed.min(max_increment);
            self.bytes_processed = 0;
        }

        self.current_window_size += increment;
        Some(increment)
    }
}

