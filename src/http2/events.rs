use std::fmt;

// Base trait for all events.
pub trait Event {}

#[derive(Debug)]
pub struct RequestReceived {
    pub stream_id: Option<u32>,
    pub headers: Option<Vec<u8>>,
    pub stream_ended: Option<StreamEnded>,
    pub priority_updated: Option<PriorityUpdated>,
}

impl fmt::Display for RequestReceived {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "<RequestReceived stream_id:{:?}, headers:{:?}>",
            self.stream_id, self.headers
        )
    }
}

impl Event for RequestReceived {}

#[derive(Debug)]
pub struct ResponseReceived {
    pub stream_id: Option<u32>,
    pub headers: Option<Vec<u8>>,
    pub stream_ended: Option<StreamEnded>,
    pub priority_updated: Option<PriorityUpdated>,
}

impl fmt::Display for ResponseReceived {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "<ResponseReceived stream_id:{:?}, headers:{:?}>",
            self.stream_id, self.headers
        )
    }
}

impl Event for ResponseReceived {}

#[derive(Debug)]
pub struct TrailersReceived {
    pub stream_id: Option<u32>,
    pub headers: Option<Vec<u8>>,
    pub stream_ended: Option<StreamEnded>,
    pub priority_updated: Option<PriorityUpdated>,
}

impl fmt::Display for TrailersReceived {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "<TrailersReceived stream_id:{:?}, headers:{:?}>",
            self.stream_id, self.headers
        )
    }
}

impl Event for TrailersReceived {}

#[derive(Debug)]
pub struct _HeadersSent;

impl Event for _HeadersSent {}

#[derive(Debug)]
pub struct _ResponseSent;

impl Event for _ResponseSent {}

#[derive(Debug)]
pub struct _RequestSent;

impl Event for _RequestSent {}

#[derive(Debug)]
pub struct _TrailersSent;

impl Event for _TrailersSent {}

#[derive(Debug)]
pub struct _PushedRequestSent;

impl Event for _PushedRequestSent {}

#[derive(Debug)]
pub struct InformationalResponseReceived {
    pub stream_id: Option<u32>,
    pub headers: Option<Vec<u8>>,
    pub priority_updated: Option<PriorityUpdated>,
}

impl fmt::Display for InformationalResponseReceived {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "<InformationalResponseReceived stream_id:{:?}, headers:{:?}>",
            self.stream_id, self.headers
        )
    }
}

impl Event for InformationalResponseReceived {}

#[derive(Debug)]
pub struct DataReceived {
    pub stream_id: Option<u32>,
    pub data: Option<Vec<u8>>,
    pub flow_controlled_length: Option<usize>,
    pub stream_ended: Option<StreamEnded>,
}

impl fmt::Display for DataReceived {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "<DataReceived stream_id:{:?}, flow_controlled_length:{:?}, data:{:?}>",
            self.stream_id,
            self.flow_controlled_length,
            self.data.as_ref().map(|d| &d[..20])
        )
    }
}

impl Event for DataReceived {}

#[derive(Debug)]
pub struct WindowUpdated {
    pub stream_id: Option<u32>,
    pub delta: Option<u32>,
}

impl fmt::Display for WindowUpdated {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "<WindowUpdated stream_id:{:?}, delta:{:?}>", self.stream_id, self.delta)
    }
}

impl Event for WindowUpdated {}

#[derive(Debug)]
pub struct StreamEnded;
#[derive(Debug)]
pub struct PriorityUpdated;

// Function to help display byte representations for DataReceived (if needed).
fn _bytes_representation(data: &[u8]) -> String {
    data.iter().map(|byte| format!("{:02x}", byte)).collect::<Vec<String>>().join(" ")
}
// Event trait for all event types.
pub trait Event {}

#[derive(Debug)]
pub struct RemoteSettingsChanged {
    pub changed_settings: HashMap<u32, ChangedSetting>,
}

impl RemoteSettingsChanged {
    pub fn from_settings(old_settings: HashMap<u32, u32>, new_settings: HashMap<u32, u32>) -> Self {
        let mut e = RemoteSettingsChanged {
            changed_settings: HashMap::new(),
        };
        for (setting, new_value) in new_settings {
            let original_value = old_settings.get(&setting).cloned();
            let change = ChangedSetting { setting, original_value, new_value };
            e.changed_settings.insert(setting, change);
        }
        e
    }
}

impl fmt::Display for RemoteSettingsChanged {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "<RemoteSettingsChanged changed_settings:{:?}>",
            self.changed_settings.values()
        )
    }
}

impl Event for RemoteSettingsChanged {}

#[derive(Debug)]
pub struct PingReceived {
    pub ping_data: Option<Vec<u8>>,
}

impl fmt::Display for PingReceived {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "<PingReceived ping_data:{:?}>", _bytes_representation(self.ping_data.as_ref()))
    }
}

impl Event for PingReceived {}

#[derive(Debug)]
pub struct PingAckReceived {
    pub ping_data: Option<Vec<u8>>,
}

impl fmt::Display for PingAckReceived {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "<PingAckReceived ping_data:{:?}>", _bytes_representation(self.ping_data.as_ref()))
    }
}

impl Event for PingAckReceived {}

#[derive(Debug)]
pub struct StreamEnded {
    pub stream_id: Option<u32>,
}

impl fmt::Display for StreamEnded {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "<StreamEnded stream_id:{:?}>", self.stream_id)
    }
}

impl Event for StreamEnded {}

#[derive(Debug)]
pub struct StreamReset {
    pub stream_id: Option<u32>,
    pub error_code: Option<u32>,
    pub remote_reset: bool,
}

impl fmt::Display for StreamReset {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "<StreamReset stream_id:{:?}, error_code:{:?}, remote_reset:{:?}>",
            self.stream_id, self.error_code, self.remote_reset
        )
    }
}

impl Event for StreamReset {}

#[derive(Debug)]
pub struct PushedStreamReceived {
    pub pushed_stream_id: Option<u32>,
    pub parent_stream_id: Option<u32>,
    pub headers: Option<Vec<u8>>,
}

impl fmt::Display for PushedStreamReceived {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "<PushedStreamReceived pushed_stream_id:{:?}, parent_stream_id:{:?}, headers:{:?}>",
            self.pushed_stream_id, self.parent_stream_id, self.headers
        )
    }
}

impl Event for PushedStreamReceived {}

#[derive(Debug)]
pub struct SettingsAcknowledged {
    pub changed_settings: HashMap<u32, ChangedSetting>,
}

impl fmt::Display for SettingsAcknowledged {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "<SettingsAcknowledged changed_settings:{:?}>",
            self.changed_settings.values()
        )
    }
}

impl Event for SettingsAcknowledged {}

#[derive(Debug)]
pub struct PriorityUpdated {
    pub stream_id: Option<u32>,
    pub weight: Option<u32>,
    pub depends_on: Option<u32>,
    pub exclusive: Option<bool>,
}

impl fmt::Display for PriorityUpdated {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "<PriorityUpdated stream_id:{:?}, weight:{:?}, depends_on:{:?}, exclusive:{:?}>",
            self.stream_id, self.weight, self.depends_on, self.exclusive
        )
    }
}

impl Event for PriorityUpdated {}

#[derive(Debug)]
pub struct ConnectionTerminated {
    pub error_code: Option<u32>,
    pub last_stream_id: Option<u32>,
    pub additional_data: Option<Vec<u8>>,
}

impl fmt::Display for ConnectionTerminated {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "<ConnectionTerminated error_code:{:?}, last_stream_id:{:?}, additional_data:{:?}>",
            self.error_code,
            self.last_stream_id,
            _bytes_representation(self.additional_data.as_ref())
        )
    }
}

impl Event for ConnectionTerminated {}

#[derive(Debug)]
pub struct AlternativeServiceAvailable {
    pub origin: Option<String>,
    pub field_value: Option<String>,
}

impl fmt::Display for AlternativeServiceAvailable {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "<AlternativeServiceAvailable origin:{:?}, field_value:{:?}>",
            self.origin, self.field_value
        )
    }
}

impl Event for AlternativeServiceAvailable {}

#[derive(Debug)]
pub struct UnknownFrameReceived {
    pub frame: Option<Vec<u8>>,
}

impl fmt::Display for UnknownFrameReceived {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "<UnknownFrameReceived>")
    }
}

impl Event for UnknownFrameReceived {}

#[derive(Debug)]
pub struct ChangedSetting {
    pub setting: u32,
    pub original_value: Option<u32>,
    pub new_value: u32,
}

// Helper function for byte representation
fn _bytes_representation(data: Option<&Vec<u8>>) -> String {
    match data {
        Some(d) => d.iter().map(|byte| format!("{:02x}", byte)).collect::<Vec<String>>().join(" "),
        None => String::from("None"),
    }
}

