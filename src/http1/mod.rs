// mod.rs (equivalent to __init__.py in Python)

pub use crate::connection::Connection;
pub use crate::connection::NEED_DATA;
pub use crate::connection::PAUSED;

pub use crate::events::{
    ConnectionClosed,
    Data,
    EndOfMessage,
    Event,
    InformationalResponse,
    Request,
    Response,
};

pub use crate::state::{
    CLIENT,
    CLOSED,
    DONE,
    ERROR,
    IDLE,
    MIGHT_SWITCH_PROTOCOL,
    MUST_CLOSE,
    SEND_BODY,
    SEND_RESPONSE,
    SERVER,
    SWITCHED_PROTOCOL,
};

pub use crate::errors::{
    ProtocolError,
    LocalProtocolError,
    RemoteProtocolError,
};

pub use crate::version::{PRODUCT_ID, VERSION};

