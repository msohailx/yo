use std::collections::{HashMap, HashSet};
use std::fmt::Debug;
use std::error::Error;
use lazy_static::lazy_static;

/// Enum representing various connection states and actions.
///
/// Used to track connection roles, request/response flow, protocol switching, and connection closure.
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

/// Type alias for an event callback function.
type Event = Box<dyn Fn()>;

/// Type alias for a nested map representing state transitions for events.
type EventTransitionType = HashMap<Sentinel, HashMap<Sentinel, HashMap<Sentinel, Sentinel>>>;

lazy_static! {
    /// A pre-defined map for event-triggered state transitions.
    /// 
    /// This maps the current state and event to the resulting state for `Client` and `Server` roles.
    static ref EVENT_TRIGGERED_TRANSITIONS: EventTransitionType = {
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
}

/// Type alias for a map representing state transitions based on two input states.
type StateTransitionType = HashMap<(Sentinel, Sentinel), HashMap<Sentinel, Sentinel>>;

lazy_static! {
    /// A pre-defined map for state-triggered transitions.
    /// 
    /// This defines how the system transitions from one state to another based on specific state pairs.
    static ref STATE_TRIGGERED_TRANSITIONS: StateTransitionType = {
        let mut map = HashMap::new();
        
        map.insert((Sentinel::Done, Sentinel::Closed), HashMap::from([(Sentinel::Server, Sentinel::MustClose)]));
        map.insert((Sentinel::Done, Sentinel::Error), HashMap::from([(Sentinel::Client, Sentinel::MustClose)]));
        
        map
    };
}

#[derive(Default)]
/// Represents the state of a connection, including keep-alive status,
/// pending switch proposals, and the states of the client and server.
pub struct ConnectionState {
    pub keep_alive: bool,
    pub pending_switch_proposals: HashSet<Sentinel>,
    pub states: HashMap<Sentinel, Sentinel>,
}

impl ConnectionState {
    /// Creates a new `ConnectionState` with initial values.
    ///
    /// Sets `keep_alive` to `true`, initializes `states` with `Idle` for both `Client` and `Server`,
    /// and clears any pending switch proposals.
    ///
    /// # Example
    /// ```
    /// let conn_state = ConnectionState::new();
    /// ```
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

    /// Processes an error by marking the given `role` as `Error` and triggering state transitions.
    ///
    /// # Example
    /// ```
    /// conn_state.process_error(Sentinel::Client);
    /// ```
    pub fn process_error(&mut self, role: Sentinel) {
        self.states.insert(role, Sentinel::Error); // **Fix #1**: No need to clone the role.
        self.fire_state_triggered_transitions();
    }

    /// Disables keep-alive and triggers state transitions.
    ///
    /// # Example
    /// ```
    /// conn_state.process_keep_alive_disabled();
    /// ```
    pub fn process_keep_alive_disabled(&mut self) {
        self.keep_alive = false;
        self.fire_state_triggered_transitions();
    }

    /// Processes a client switch proposal by adding it to the pending switch proposals.
    ///
    /// # Example
    /// ```
    /// conn_state.process_client_switch_proposal(Sentinel::SwitchUpgrade);
    /// ```
    pub fn process_client_switch_proposal(&mut self, switch_event: Sentinel) {
        self.pending_switch_proposals.insert(switch_event);
        self.fire_state_triggered_transitions();
    }

    /// Processes an event and triggers transitions based on the event type, including handling
    /// server switch events.
    ///
    /// # Example
    /// ```
    /// conn_state.process_event(Sentinel::Client, Sentinel::SwitchConnect, None);
    /// ```
    pub fn process_event(&mut self, role: Sentinel, event_type: Sentinel, server_switch_event: Option<Sentinel>) {
        let mut event_type = event_type; // **Fix #2**: No need to clone the event_type
        
        if let Some(server_switch_event) = server_switch_event {
            assert_eq!(role, Sentinel::Server);
            if !self.pending_switch_proposals.contains(&server_switch_event) {
                panic!("Received server _SWITCH_UPGRADE event without a pending proposal");
            }
            event_type = Sentinel::SwitchUpgrade; 
        }

        self.fire_event_triggered_transitions(role, event_type);
        /* 
        if event_type == Sentinel::SwitchUpgrade {
            assert_eq!(role, Sentinel::Client);
            self.fire_event_triggered_transitions(Sentinel::Server, event_type);
        }
        */
        self.fire_state_triggered_transitions();
    }

    /// Handles transitions triggered by events, updating the state based on role and event type.
    ///
    /// # Panics
    /// Panics if the event type cannot be handled for the current role and state.
    fn fire_event_triggered_transitions(&mut self, role: Sentinel, event_type: Sentinel) {
        // **Fix #4**: Avoid using `unwrap()` directly.
        let state = match self.states.get(&role) {
            Some(state) => state,
            None => panic!("Role {:?} not found in states.", role),
        };

        if let Some(new_state) = EVENT_TRIGGERED_TRANSITIONS
            .get(&role)
            .and_then(|role_map| role_map.get(state))
            .and_then(|event_map| event_map.get(&event_type))
        {
            self.states.insert(role, new_state.clone());
        } else {
            // Placeholder for error handling instead of panicking
            println!(
                "Warning: can't handle event type {:?} when role={:?} and state={:?}",
                event_type, role, state
            );
        }
    }

    /// Triggers state transitions that are based on the current states of the client and server,
    /// and updates the states accordingly.
    fn fire_state_triggered_transitions(&mut self) {
        // **Fix #5**: Simplify the logic, no need for a loop.
        let start_states = self.states.clone();
    
        // Handle pending switch proposals and client state transitions
        if !self.pending_switch_proposals.is_empty() {
            if self.states.get(&Sentinel::Client) == Some(&Sentinel::Done) {
                self.states.insert(Sentinel::Client, Sentinel::MightSwitchProtocol);
            }
        }
    
        if self.pending_switch_proposals.is_empty() {
            if self.states.get(&Sentinel::Client) == Some(&Sentinel::MightSwitchProtocol) {
                self.states.insert(Sentinel::Client, Sentinel::Done);
            }
        }
    
        // Handle connection closure when keep-alive is disabled
        if !self.keep_alive {
            for role in [Sentinel::Client, Sentinel::Server].iter() {
                if self.states.get(role) == Some(&Sentinel::Done) {
                    self.states.insert(role.clone(), Sentinel::MustClose);
                }
            }
        }
    
        // Check for state-triggered transitions
        let joint_state = (
            self.states.get(&Sentinel::Client).unwrap_or(&Sentinel::Idle),
            self.states.get(&Sentinel::Server).unwrap_or(&Sentinel::Idle),
        );
        /* 
        if let Some(changes) = STATE_TRIGGERED_TRANSITIONS.get(&joint_state) {
            for (role, new_state) in changes {
                self.states.insert(role.clone(), new_state.clone());
            }
        }
        */
        // Only return if no changes were made to the states
        if self.states == start_states {
            return;
        }
    }
    /// Starts a new cycle by resetting the connection states to `Idle` for both client and server.
    ///
    /// # Panics
    /// Panics if the current state is not `Done` for both client and server.
    ///
    /// # Example
    /// ```
    /// conn_state.start_next_cycle();
    /// ```
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
