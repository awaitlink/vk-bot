//! The [`Core`] struct, supported [`Event`][crate::core::Event]s, and handler/tester types.

use crate::{context::Context, request::CallbackAPIRequest};
use log::{debug, error, info, trace, warn};
use rvk::APIClient;
use std::collections::{hash_map::Entry, HashMap};
use std::fmt::{Debug, Display, Error, Formatter};
use std::sync::{Arc, Mutex};

/// Events that are supported for event handlers.
#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
pub enum Event {
    /// Callback API: `message_new`.
    MessageNew,
    /// Callback API: `message_reply`.
    MessageReply,
    /// Callback API: `message_edit`.
    MessageEdit,
    /// Callback API: `message_typing_state`.
    MessageTypingState,
    /// Callback API: `message_allow`.
    MessageAllow,
    /// Callback API: `message_deny`.
    MessageDeny,

    /// Generated instead of [`Event::MessageNew`] when
    /// start button was pressed.
    Start,
    /// Generated instead of [`Event::MessageNew`] when
    /// the message is a service action message.
    ServiceAction,

    /// Generated when no matching handler for an event
    /// is found.
    NoMatch,
    // TODO: HandlerError event?
}

impl Display for Event {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        f.write_str(match self {
            Event::MessageNew => "message_new",
            Event::MessageReply => "message_reply",
            Event::MessageEdit => "message_edit",
            Event::MessageTypingState => "message_typing_state",
            Event::MessageAllow => "message_allow",
            Event::MessageDeny => "message_deny",

            Event::Start => "start",
            Event::ServiceAction => "service_action",

            Event::NoMatch => "no_match",
        })
    }
}

impl From<&str> for Event {
    /// Converts a `&`[`str`] into the associated event.
    ///
    /// # Panics
    /// - when given unknown event
    fn from(s: &str) -> Self {
        match s {
            "message_new" => Event::MessageNew,
            "message_reply" => Event::MessageReply,
            "message_edit" => Event::MessageEdit,
            "message_typing_state" => Event::MessageTypingState,
            "message_allow" => Event::MessageAllow,
            "message_deny" => Event::MessageDeny,

            "start" => Event::Start,
            "service_action" => Event::ServiceAction,

            "no_match" => Event::NoMatch,

            _ => panic!("unknown event: `{}`", s),
        }
    }
}

impl From<String> for Event {
    /// Converts a [`String`] into the associated event.
    ///
    /// # Panics
    /// - when given unknown event
    fn from(s: String) -> Self {
        s.as_str().into()
    }
}

/// Inner type of [`Handler`].
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

/// Inner type of [`Tester`].
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
    event_handlers: HashMap<Event, Handler>,
    static_payload_handlers: HashMap<String, Handler>,
    dyn_payload_handlers: Vec<(Tester, Handler)>,
    command_handlers: HashMap<String, Handler>,
    regex_handlers: HashMap<String, Handler>,
}

impl Default for Core {
    fn default() -> Self {
        Self {
            cmd_prefix: None,
            event_handlers: Default::default(),
            static_payload_handlers: Default::default(),
            dyn_payload_handlers: Default::default(),
            command_handlers: Default::default(),
            regex_handlers: Default::default(),
        }
    }
}

impl Core {
    /// Creates a new [`Core`].
    pub fn new() -> Self {
        Default::default()
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
    /// Handler for the `message_new` event is built-in,
    /// and is not changeable.
    ///
    /// See also [`Event`].
    pub fn on(mut self, event: Event, handler: Handler) -> Self {
        let entry = self.event_handlers.entry(event.into());

        match event {
            Event::MessageNew => panic!(
                "attempt to set up handler for `{}`, \
                 which is defined internally and should not be replaced",
                Event::MessageNew
            ),
            _ => match entry {
                Entry::Occupied(_) => {
                    panic!("attempt to set up duplicate handler for event `{}`", event)
                }
                Entry::Vacant(entry) => {
                    entry.insert(handler);
                }
            },
        }

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
            Entry::Vacant(entry) => entry.insert(handler),
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
            Entry::Vacant(entry) => entry.insert(handler),
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
            Entry::Vacant(entry) => entry.insert(handler),
        };

        self
    }

    /// Handles a request by telling the appropriate
    /// [`Handler`] to do so.
    pub fn handle(&self, req: &CallbackAPIRequest, api: Arc<Mutex<APIClient>>) {
        debug!("handling {:#?}", req);
        self.handle_event(req.r#type().into(), api, req);
    }

    fn handle_event(&self, event: Event, api: Arc<Mutex<APIClient>>, req: &CallbackAPIRequest) {
        let mut ctx = Context::new(event, req.object().clone(), api);

        match event {
            Event::MessageNew => self.handle_message_new(&mut ctx),
            _ => unimplemented!(),
        };
    }

    fn handle_message_new<'a>(&self, ctx: &'a mut Context) -> &'a mut Context {
        unimplemented!();
        ctx
    }
}
