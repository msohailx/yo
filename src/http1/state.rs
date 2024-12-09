use std::collections::{HashMap, HashSet};
use std::fmt::Debug;
use std::error::Error;

#[derive(Debug, Eq, PartialEq, Hash, Clone)]
pub enum Sentinel {
    Client,
    Server,
    Idle,
    SendResponse,
    SendBody,
    Done,
    MustClose,
    Closed,
    Error,
    MightSwitchProtocol,
    SwitchedProtocol,
    SwitchUpgrade,
    SwitchConnect,
}

#[derive(Debug)]
pub struct LocalProtocolError(String);

impl std::fmt::Display for LocalProtocolError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Error for LocalProtocolError {}

type Event = Box<dyn Fn()>;

type EventTransitionType = HashMap<Sentinel, HashMap<Sentinel, HashMap<Sentinel, Sentinel>>>;

const EVENT_TRIGGERED_TRANSITIONS: EventTransitionType = {
    let mut map = HashMap::new();
    
    let mut client_map = HashMap::new();
    client_map.insert(Sentinel::Idle, HashMap::from([
        (Sentinel::SwitchConnect, Sentinel::Closed),
        (Sentinel::SwitchUpgrade, Sentinel::Closed),
    ]));
    map.insert(Sentinel::Client, client_map);

    let mut server_map = HashMap::new();
    server_map.insert(Sentinel::Idle, HashMap::from([
        (Sentinel::SwitchConnect, Sentinel::Closed),
        (Sentinel::SwitchUpgrade, Sentinel::Closed),
    ]));
    map.insert(Sentinel::Server, server_map);
    
    map
};

type StateTransitionType = HashMap<(Sentinel, Sentinel), HashMap<Sentinel, Sentinel>>;

const STATE_TRIGGERED_TRANSITIONS: StateTransitionType = {
    let mut map = HashMap::new();
    
    map.insert((Sentinel::Done, Sentinel::Closed), HashMap::from([(Sentinel::Server, Sentinel::MustClose)]));
    map.insert((Sentinel::Done, Sentinel::Error), HashMap::from([(Sentinel::Client, Sentinel::MustClose)]));
    
    map
};

#[derive(Default)]
pub struct ConnectionState {
    pub keep_alive: bool,
    pub pending_switch_proposals: HashSet<Sentinel>,
    pub states: HashMap<Sentinel, Sentinel>,
}

impl ConnectionState {
    pub fn new() -> Self {
        ConnectionState {
            keep_alive: true,
            pending_switch_proposals: HashSet::new(),
            states: HashMap::from([
                (Sentinel::Client, Sentinel::Idle),
                (Sentinel::Server, Sentinel::Idle),
            ]),
        }
    }

    pub fn process_error(&mut self, role: Sentinel) {
        self.states.insert(role.clone(), Sentinel::Error);
        self.fire_state_triggered_transitions();
    }

    pub fn process_keep_alive_disabled(&mut self) {
        self.keep_alive = false;
        self.fire_state_triggered_transitions();
    }

    pub fn process_client_switch_proposal(&mut self, switch_event: Sentinel) {
        self.pending_switch_proposals.insert(switch_event);
        self.fire_state_triggered_transitions();
    }

    pub fn process_event(&mut self, role: Sentinel, event_type: Sentinel, server_switch_event: Option<Sentinel>) {
        let mut event_type = event_type.clone();
        
        if let Some(server_switch_event) = server_switch_event {
            assert_eq!(role, Sentinel::Server);
            if !self.pending_switch_proposals.contains(&server_switch_event) {
                panic!("Received server _SWITCH_UPGRADE event without a pending proposal");
            }
            event_type = Sentinel::SwitchUpgrade;
        }

        self.fire_event_triggered_transitions(role, event_type);
        if event_type == Sentinel::SwitchUpgrade {
            assert_eq!(role, Sentinel::Client);
            self.fire_event_triggered_transitions(Sentinel::Server, event_type);
        }
        self.fire_state_triggered_transitions();
    }

    fn fire_event_triggered_transitions(&mut self, role: Sentinel, event_type: Sentinel) {
        let state = self.states.get(&role).unwrap();
        if let Some(new_state) = EVENT_TRIGGERED_TRANSITIONS.get(&role).and_then(|role_map| role_map.get(state).and_then(|event_map| event_map.get(&event_type))) {
            self.states.insert(role, new_state.clone());
        } else {
            panic!("can't handle event type {:?} when role={:?} and state={:?}", event_type, role, state);
        }
    }

    fn fire_state_triggered_transitions(&mut self) {
        loop {
            let start_states = self.states.clone();

            if !self.pending_switch_proposals.is_empty() {
                if self.states[&Sentinel::Client] == Sentinel::Done {
                    self.states.insert(Sentinel::Client, Sentinel::MightSwitchProtocol);
                }
            }

            if self.pending_switch_proposals.is_empty() {
                if self.states[&Sentinel::Client] == Sentinel::MightSwitchProtocol {
                    self.states.insert(Sentinel::Client, Sentinel::Done);
                }
            }

            if !self.keep_alive {
                for role in [Sentinel::Client, Sentinel::Server].iter() {
                    if self.states[role] == Sentinel::Done {
                        self.states.insert(role.clone(), Sentinel::MustClose);
                    }
                }
            }

            let joint_state = (self.states[&Sentinel::Client].clone(), self.states[&Sentinel::Server].clone());
            if let Some(changes) = STATE_TRIGGERED_TRANSITIONS.get(&joint_state) {
                for (role, new_state) in changes {
                    self.states.insert(role.clone(), new_state.clone());
                }
            }

            if self.states == start_states {
                return;
            }
        }
    }

    pub fn start_next_cycle(&mut self) {
        if self.states != HashMap::from([
            (Sentinel::Client, Sentinel::Done),
            (Sentinel::Server, Sentinel::Done),
        ]) {
            panic!("not in a reusable state. self.states={:?}", self.states);
        }

        assert!(self.keep_alive);
        assert!(self.pending_switch_proposals.is_empty());
        self.states = HashMap::from([
            (Sentinel::Client, Sentinel::Idle),
            (Sentinel::Server, Sentinel::Idle),
        ]);
    }
}

