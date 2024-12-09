use std::fmt;

pub struct BooleanConfigOption {
    name: String,
}

impl BooleanConfigOption {
    pub fn new(name: &str) -> Self {
        BooleanConfigOption {
            name: name.to_string(),
        }
    }

    pub fn get(&self, instance: &H2Configuration) -> bool {
        instance.get_boolean_config_option(&self.name)
    }

    pub fn set(&self, instance: &mut H2Configuration, value: bool) {
        instance.set_boolean_config_option(&self.name, value);
    }
}

pub struct DummyLogger;

impl DummyLogger {
    pub fn new() -> Self {
        DummyLogger
    }

    pub fn debug(&self, _fmtstr: &str, _args: &[&dyn fmt::Debug]) {}
    pub fn trace(&self, _fmtstr: &str, _args: &[&dyn fmt::Debug]) {}
}

pub struct OutputLogger {
    file: Box<dyn std::io::Write>,
    trace_level: bool,
}

impl OutputLogger {
    pub fn new(file: Option<Box<dyn std::io::Write>>, trace_level: bool) -> Self {
        OutputLogger {
            file: file.unwrap_or_else(|| Box::new(std::io::stderr())),
            trace_level,
        }
    }

    pub fn debug(&self, fmtstr: &str, args: &[&dyn fmt::Debug]) {
        writeln!(self.file, "h2 (debug): {}", fmt::format(format_args!(fmtstr, args))).unwrap();
    }

    pub fn trace(&self, fmtstr: &str, args: &[&dyn fmt::Debug]) {
        if self.trace_level {
            writeln!(self.file, "h2 (trace): {}", fmt::format(format_args!(fmtstr, args))).unwrap();
        }
    }
}

pub struct H2Configuration {
    client_side: bool,
    header_encoding: Option<String>,
    validate_outbound_headers: bool,
    normalize_outbound_headers: bool,
    split_outbound_cookies: bool,
    validate_inbound_headers: bool,
    normalize_inbound_headers: bool,
    logger: Box<dyn Logger>,
}

impl H2Configuration {
    pub fn new(
        client_side: bool,
        header_encoding: Option<String>,
        validate_outbound_headers: bool,
        normalize_outbound_headers: bool,
        split_outbound_cookies: bool,
        validate_inbound_headers: bool,
        normalize_inbound_headers: bool,
        logger: Option<Box<dyn Logger>>,
    ) -> Self {
        H2Configuration {
            client_side,
            header_encoding,
            validate_outbound_headers,
            normalize_outbound_headers,
            split_outbound_cookies,
            validate_inbound_headers,
            normalize_inbound_headers,
            logger: logger.unwrap_or_else(|| Box::new(DummyLogger::new())),
        }
    }

    pub fn get_boolean_config_option(&self, name: &str) -> bool {
        match name {
            "client_side" => self.client_side,
            "validate_outbound_headers" => self.validate_outbound_headers,
            "normalize_outbound_headers" => self.normalize_outbound_headers,
            "split_outbound_cookies" => self.split_outbound_cookies,
            "validate_inbound_headers" => self.validate_inbound_headers,
            "normalize_inbound_headers" => self.normalize_inbound_headers,
            _ => panic!("Unknown config option: {}", name),
        }
    }

    pub fn set_boolean_config_option(&mut self, name: &str, value: bool) {
        match name {
            "client_side" => self.client_side = value,
            "validate_outbound_headers" => self.validate_outbound_headers = value,
            "normalize_outbound_headers" => self.normalize_outbound_headers = value,
            "split_outbound_cookies" => self.split_outbound_cookies = value,
            "validate_inbound_headers" => self.validate_inbound_headers = value,
            "normalize_inbound_headers" => self.normalize_inbound_headers = value,
            _ => panic!("Unknown config option: {}", name),
        }
    }

    pub fn header_encoding(&self) -> &Option<String> {
        &self.header_encoding
    }

    pub fn set_header_encoding(&mut self, value: Option<String>) {
        if let Some(ref v) = value {
            if v == "true" {
                panic!("header_encoding cannot be 'true'");
            }
        }
        self.header_encoding = value;
    }
}

pub trait Logger {
    fn debug(&self, fmtstr: &str, args: &[&dyn fmt::Debug]);
    fn trace(&self, fmtstr: &str, args: &[&dyn fmt::Debug]);
}

impl Logger for DummyLogger {
    fn debug(&self, _fmtstr: &str, _args: &[&dyn fmt::Debug]) {}
    fn trace(&self, _fmtstr: &str, _args: &[&dyn fmt::Debug]) {}
}

impl Logger for OutputLogger {
    fn debug(&self, fmtstr: &str, args: &[&dyn fmt::Debug]) {
        self.debug(fmtstr, args);
    }

    fn trace(&self, fmtstr: &str, args: &[&dyn fmt::Debug]) {
        self.trace(fmtstr, args);
    }
}

