/// All the stuff in the directory http1 is sitting here, which then can be imported to lib.rs in
/// one go
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

