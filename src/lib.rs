#![allow(unused_variables)]
#![allow(dead_code)]
pub mod http1 {
    pub mod connection;
    pub mod http_regex;
    pub mod recieve_buffer;
    pub mod version;
    pub mod events;
    pub mod state;
    pub mod writers;
    pub mod headers;
    pub mod readers;
    pub mod util;
}

