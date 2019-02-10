//! The [`Core`] struct and handler/tester types.

use crate::context::Context;
use crate::request::CallbackAPIRequest;
use log::{debug, error, info, trace, warn};
use std::collections::{hash_map::Entry, HashMap};
use std::fmt::{Debug, Error, Formatter};
use std::sync::Arc;

/// Events that are supported by event handlers.
pub const SUPPORTED_EVENTS: [&'static str; 10] = [
    // Callback API
    "message_new",
    "message_reply",
    "message_edit",
    "message_typing_state",
    "message_allow",
    "message_deny",
    // Detected when parsing 'message_new' event
    "start",
    "service_action",
    // Internal events
    "no_match",
    "handler_error",
];

/// Inner type of [`Handler`]
pub type HandlerInner = Arc<dyn (Fn(&mut Context) -> &mut Context) + Send + Sync + 'static>;

/// Handler's [`Fn`] should handle the message/event using the
/// given `&mut` [`Context`], and return it back when finished.
///
/// This is a wrapper around `Arc<dyn (Fn(&mut Context) -> &mut Context) + ...>`
/// to provide a [`Debug`] impl needed by [`HashMap`].
#[derive(Clone)]
pub struct Handler {
    inner: HandlerInner,
}

impl Handler {
    /// Creates a new wrapper.
    pub fn new<F>(handler: F) -> Self
    where
        F: (Fn(&mut Context) -> &mut Context) + Send + Sync + 'static,
    {
        Self {
            inner: Arc::new(handler),
        }
    }

    /// Returns the wrapped value by cloning the [`Arc`].
    pub fn inner(&self) -> HandlerInner {
        Arc::clone(&self.inner)
    }
}

impl Debug for Handler {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        f.write_str("Handler {...}")
    }
}

/// Inner type of [`Tester`]
pub type TesterInner = Arc<dyn (Fn(&String) -> bool) + Send + Sync + 'static>;

/// Tester's [`Fn`] should return whether a stringified JSON is
/// interesting for a handler to handle.
///
/// This is a wrapper around `Arc<dyn (Fn(&String) -> bool) + ...>`.
#[derive(Clone)]
pub struct Tester {
    inner: TesterInner,
}

impl Tester {
    /// Creates a new wrapper.
    pub fn new<F>(tester: F) -> Self
    where
        F: (Fn(&String) -> bool) + Send + Sync + 'static,
    {
        Self {
            inner: Arc::new(tester),
        }
    }

    /// Returns the wrapped value by cloning the [`Arc`].
    pub fn inner(&self) -> TesterInner {
        Arc::clone(&self.inner)
    }
}

impl Debug for Tester {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        f.write_str("Tester {...}")
    }
}

/// [`Core`] accepts user-defined handlers, and invokes them when needed.
#[derive(Debug, Clone)]
pub struct Core {
    cmd_prefix: Option<String>,
    event_handlers: HashMap<String, Handler>,
    static_payload_handlers: HashMap<String, Handler>,
    dyn_payload_handlers: Vec<(Tester, Handler)>,
    command_handlers: HashMap<String, Handler>,
    regex_handlers: HashMap<String, Handler>,
}

impl Core {
    /// Creates a new effectively empty [`Core`].
    pub fn new() -> Self {
        Self {
            cmd_prefix: None,
            event_handlers: Default::default(),
            static_payload_handlers: Default::default(),
            dyn_payload_handlers: Default::default(),
            command_handlers: Default::default(),
            regex_handlers: Default::default(),
        }
    }

    /// Modifies this [`Core`]'s command prefix, consuming
    /// `mut self` and returning it after the modification.
    pub fn with_cmd_prefix(mut self, cmd_prefix: &str) -> Self {
        self.cmd_prefix = Some(cmd_prefix.into());
        self
    }

    /// Adds a new event handler to this [`Core`], consuming
    /// `mut self` and returning it after the modification.
    ///
    /// See also [`SUPPORTED_EVENTS`].
    pub fn on(mut self, event: &str, handler: Handler) -> Self {
        let entry = self.event_handlers.entry(event.into());
        match entry {
            Entry::Occupied(_) => {
                panic!("attempt to set up duplicate handler for event `{}`", event);
            }
            Entry::Vacant(e) => {
                if SUPPORTED_EVENTS.contains(&event) {
                    e.insert(handler);
                } else {
                    panic!(
                        "attempt to set up handler for unsupported event `{}`",
                        event
                    );
                };
            }
        };

        self
    }

    /// Adds a new payload handler to this [`Core`], consuming
    /// `mut self` and returning it after the modification.
    ///
    /// See also [`Core::dyn_payload`].
    pub fn payload(mut self, payload: &'static str, handler: Handler) -> Self {
        let entry = self.static_payload_handlers.entry(payload.into());
        match entry {
            Entry::Occupied(_) => panic!(
                "attempt to set up duplicate handler for payload {:#?}",
                payload
            ),
            Entry::Vacant(e) => e.insert(handler),
        };

        self
    }

    /// Adds a new dynamic (provided a [`Tester`]) payload handler
    /// to this [`Core`], consuming `mut self` and returning it after
    /// the modification.
    ///
    /// See also [`Core::payload`].
    pub fn dyn_payload(mut self, tester: Tester, handler: Handler) -> Self {
        self.dyn_payload_handlers.push((tester, handler));
        self
    }

    /// Adds a new command (exact string after command prefix)
    /// handler to this [`Core`], consuming `mut self` and
    /// returning it after the modification.
    pub fn cmd(mut self, cmd: &'static str, handler: Handler) -> Self {
        let entry = self.command_handlers.entry(cmd.into());
        match entry {
            Entry::Occupied(_) => {
                panic!("attempt to set up duplicate handler for command `{}`", cmd);
            }
            Entry::Vacant(e) => e.insert(handler),
        };

        self
    }

    /// Adds a new regex handler to this [`Core`], consuming
    /// `mut self` and returning it after the modification.
    pub fn regex(mut self, regex: String, handler: Handler) -> Self {
        let entry = self.regex_handlers.entry(regex.clone());
        match entry {
            Entry::Occupied(_) => {
                panic!("attempt to set up duplicate handler for regex {:#?}", regex);
            }
            Entry::Vacant(e) => e.insert(handler),
        };

        self
    }

    /// Handles a request by telling the appropriate
    /// [`Handler`] to do so.
    pub fn handle(&self, req: &CallbackAPIRequest, vk_token: &str) {
        debug!("handling {:#?}", req);
        unimplemented!()
    }
}
