//! The [`Core`] struct, supported [`Event`][crate::core::Event]s, and
//! handler/tester types.

use crate::{context::Context, request::CallbackAPIRequest};
use rvk::APIClient;
use serde_json::Value;
use std::{
    collections::{hash_map::Entry, HashMap},
    fmt::{Debug, Display, Error, Formatter},
    ops::Deref,
    sync::{Arc, Mutex},
};

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

    /// Generated instead of [`Event::MessageNew`] when start button was
    /// pressed.
    Start,
    /// Generated instead of [`Event::MessageNew`] when the message is a service
    /// action message.
    ServiceAction,

    /// Generated when no matching handler for an event/payload/command/regex is
    /// found.
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
pub type HandlerInner = Arc<dyn Fn(&mut Context) + Send + Sync + 'static>;

/// Handler's [`Fn`] should handle the message/event using the given `&mut`
/// [`Context`], and return it back when finished.
///
/// This is essentially a wrapper around `Arc<dyn Fn(&mut Context) + ...>`.
#[derive(Clone)]
pub struct Handler {
    inner: HandlerInner,
}

impl Handler {
    /// Creates a new wrapper.
    pub fn new<F>(handler: F) -> Self
    where
        F: Fn(&mut Context) + Send + Sync + 'static,
    {
        Self {
            inner: Arc::new(handler),
        }
    }
}

impl Deref for Handler {
    type Target = HandlerInner;
    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl Debug for Handler {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        f.write_str("Handler {...}")
    }
}

/// Inner type of [`Tester`].
pub type TesterInner = Arc<dyn (Fn(&String) -> bool) + Send + Sync + 'static>;

/// Tester's [`Fn`] should return whether a payload string (you to set the
/// payload in [`Keyboard`][`crate::keyboard::Keyboard`]
/// [`Button`][`crate::keyboard::Button`]s) is interesting for a handler to
/// handle.
///
/// This is essentially a wrapper around `Arc<dyn (Fn(&String) -> bool) + ...>`.
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
}

impl Deref for Tester {
    type Target = TesterInner;
    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl Debug for Tester {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        f.write_str("Tester {...}")
    }
}

/// [`Core`] accepts user-defined handlers, and invokes them when needed.
/// Note that only one handler (the first found, according to the
/// [`Core::on`] docs) is called for a given message.
///
/// Works like a builder.
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

    /// Modifies this [`Core`]'s command prefix.
    pub fn cmd_prefix(mut self, cmd_prefix: &str) -> Self {
        self.cmd_prefix = Some(cmd_prefix.into());
        self
    }

    /// Adds a new event handler to this [`Core`].
    ///
    /// See [`Event`] for possible events.
    ///
    /// Be very careful in implementation of [`Event::MessageReply`]. It will be triggered
    /// for **every message your community admins or the bot sends**, including ones sent
    /// through an [`Event::MessageReply`] or [`Event::NoMatch`] handler!
    ///
    /// Handler for the [`Event::MessageNew`] is built-in, not changeable, and works like this:
    ///
    /// \# | cause | action
    /// ---|---|---
    /// 1 | `action` field on object | [`Event::ServiceAction`]
    /// 2 | special `{"command": "start"}` payload | [`Event::Start`]
    /// 3 | exact payload match set up via [`Core::payload`] | respective handler
    /// 4 | 'dynamic' payload match set up via [`Core::dyn_payload`] | respective handler
    /// 5 | command handlers ([`Core::cmd_prefix`] and [`Core::cmd`]) | respective handler
    /// 6 | regex handlers ([`Core::regex`]) | respective handler
    /// 7 | anything except [`Event::MessageReply`] and [`Event::NoMatch`] | [`Event::NoMatch`]
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

    /// Adds a new payload handler to this [`Core`].
    ///
    /// See also [`Core::dyn_payload`].
    pub fn payload(mut self, payload: &str, handler: Handler) -> Self {
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

    /// Adds a new dynamic (provided a [`Tester`]) payload handler to this
    /// [`Core`].
    ///
    /// See also [`Core::payload`].
    pub fn dyn_payload(mut self, tester: Tester, handler: Handler) -> Self {
        self.dyn_payload_handlers.push((tester, handler));
        self
    }

    /// Adds a new command (exact string after command prefix) handler to this
    /// [`Core`].
    pub fn cmd(mut self, cmd: &str, handler: Handler) -> Self {
        let entry = self.command_handlers.entry(cmd.into());
        match entry {
            Entry::Occupied(_) => {
                panic!("attempt to set up duplicate handler for command `{}`", cmd);
            }
            Entry::Vacant(entry) => entry.insert(handler),
        };

        self
    }

    /// Adds a new regex handler to this [`Core`].
    pub fn regex(mut self, regex: &str, handler: Handler) -> Self {
        let entry = self.regex_handlers.entry(regex.into());
        match entry {
            Entry::Occupied(_) => {
                panic!("attempt to set up duplicate handler for regex `{}`", regex);
            }
            Entry::Vacant(entry) => entry.insert(handler),
        };

        self
    }

    /// Handles a request by telling the appropriate [`Handler`] to do so.
    pub fn handle(&self, req: &CallbackAPIRequest, api: Arc<Mutex<APIClient>>) {
        trace!("handling {:#?}", req);

        let event: Event = req.r#type().into();
        let mut ctx = Context::new(event, req.object().clone(), api);
        self.handle_event(event, &mut ctx);
    }

    /// Handles an event.
    fn handle_event(&self, event: Event, ctx: &mut Context) {
        debug!("handling event `{}`", event);
        match event {
            Event::MessageNew => self.handle_message_new(ctx),
            Event::NoMatch => {
                if let Some(handler) = self.event_handlers.get(&Event::NoMatch) {
                    handler(ctx)
                }
            }
            e => match self.event_handlers.get(&e) {
                Some(handler) => {
                    trace!("calling `{}` handler for {:#?}", e, ctx);
                    handler(ctx)
                }
                None => match e {
                    // Prevent infinite loop when Event::MessageReply handler is not present, while
                    // Event::NoMatch sends a message.
                    Event::MessageReply => {}
                    _ => self.handle_event(Event::NoMatch, ctx),
                },
            },
        };
    }

    /// Handles the [`Event::MessageNew`], trying to detect
    /// [`Event::ServiceAction`] first, and then: [`Core::try_handle_payload`]
    /// -> [`Core::try_handle_command`] -> [`Core::try_handle_regex`] ->
    /// [`Event::NoMatch`].
    fn handle_message_new(&self, ctx: &mut Context) {
        if ctx.object().action().is_some() {
            trace!("calling `service_action` handler for {:#?}", ctx);
            self.handle_event(Event::ServiceAction, ctx);
            return;
        }

        if !self.try_handle_payload(ctx) {
            if !self.try_handle_command(ctx) {
                if !self.try_handle_regex(ctx) {
                    trace!(
                        "calling `no_match` (as `message_new` failed to match) handler for {:#?}",
                        ctx
                    );
                    self.handle_event(Event::NoMatch, ctx);
                }
            }
        }
    }

    /// Tries to handle this message using a payload handler. Returns `true` if
    /// that was successful, `false` otherwise.
    fn try_handle_payload(&self, ctx: &mut Context) -> bool {
        let payload = match ctx.object().payload() {
            Some(payload) => payload,
            None => return false,
        };

        // Handle special payload `{"command": "start"}`
        if let Ok(payload) = serde_json::from_str::<Value>(payload) {
            if let Some(object) = payload.as_object() {
                if let Some(command) = object.get("command") {
                    if command == "start" {
                        self.handle_event(Event::Start, ctx);
                        return true;
                    }
                }
            }
        }

        // Static payload handlers
        if let Some(handler) = self.static_payload_handlers.get(payload) {
            handler(ctx);
            return true;
        }

        // So-called "dynamic" payload handlers
        for (tester, handler) in &self.dyn_payload_handlers {
            if tester(payload) {
                handler(ctx);
                return true;
            }
        }

        false
    }

    /// Tries to handle this message using a command handler. Returns `true` if
    /// that was successful, `false` otherwise.
    fn try_handle_command(&self, ctx: &mut Context) -> bool {
        if let Some(text) = ctx.object().text() {
            for command in self.command_handlers.keys() {
                use regex::{escape, Regex};

                // TODO: Should match only the bot's group ID instead of all (`\d+`)
                let re = Regex::new(&format!(
                    "^( *\\[club\\d+\\|.*\\])?( *{}{})+",
                    match &self.cmd_prefix {
                        Some(prefix) => escape(prefix.as_str()),
                        None => "".into(),
                    },
                    escape(command)
                ))
                .expect("invalid regex");

                if re.is_match(&text) {
                    self.command_handlers[command](ctx);
                    return true;
                }
            }
        }

        false
    }

    /// Tries to handle this message using a regex handler. Returns `true` if
    /// that was successful, `false` otherwise.
    fn try_handle_regex(&self, ctx: &mut Context) -> bool {
        if let Some(text) = ctx.object().text() {
            for regex_str in self.regex_handlers.keys() {
                use regex::Regex;
                let re = Regex::new(&regex_str).expect("invalid regex");

                if re.is_match(&text) {
                    self.regex_handlers[regex_str](ctx);
                    return true;
                }
            }
        }

        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::request::Object;
    use std::sync::mpsc;

    #[derive(Clone, Copy, PartialEq, Debug)]
    enum Wiring {
        ServiceAction,
        Start,
        Payload,
        DynPayload,
        Command,
        Regex,
        NoMatch,
    }

    fn wiring_sender(tx: &Arc<Mutex<mpsc::SyncSender<Wiring>>>, wiring: Wiring) -> Handler {
        let tx = Arc::clone(tx);

        Handler::new(move |_| {
            tx.lock()
                .expect("failed to lock Mutex")
                .send(wiring)
                .expect("failed to send Wiring");
        })
    }

    fn test_wiring(obj: Object) -> Wiring {
        let (tx, rx) = mpsc::sync_channel(1);
        let tx = Arc::new(Mutex::new(tx));

        let mut ctx = Context::new(
            Event::MessageNew,
            obj,
            Arc::new(Mutex::new(APIClient::new("vk_token".into()))),
        );

        Core::new()
            .cmd_prefix("/")
            .on(
                Event::ServiceAction,
                wiring_sender(&tx, Wiring::ServiceAction),
            )
            .on(Event::Start, wiring_sender(&tx, Wiring::Start))
            .payload(r#"{"a": "b"}"#, wiring_sender(&tx, Wiring::Payload))
            .dyn_payload(
                Tester::new(|_| true),
                wiring_sender(&tx, Wiring::DynPayload),
            )
            .cmd("test", wiring_sender(&tx, Wiring::Command))
            .regex(r#"\d"#, wiring_sender(&tx, Wiring::Regex))
            .on(Event::NoMatch, wiring_sender(&tx, Wiring::NoMatch))
            .handle_event(Event::MessageNew, &mut ctx);

        rx.recv().expect("failed to recv Wiring")
    }

    #[test]
    fn wiring_service_action() {
        assert_eq!(
            test_wiring(Object::new(
                None,               // from_id
                Some(1),            // peer_id
                None,               // user_id
                None,               // text
                None,               // payload
                Some(Value::Null),  // action
                Default::default()  // extra fields
            )),
            Wiring::ServiceAction
        );
    }

    #[test]
    fn wiring_start() {
        assert_eq!(
            test_wiring(Object::new(
                None,                                   // from_id
                Some(1),                                // peer_id
                None,                                   // user_id
                None,                                   // text
                Some(r#"{"command": "start"}"#.into()), // payload
                None,                                   // action
                Default::default()                      // extra fields
            )),
            Wiring::Start
        );
    }

    #[test]
    fn wiring_payload() {
        assert_eq!(
            test_wiring(Object::new(
                None,                         // from_id
                Some(1),                      // peer_id
                None,                         // user_id
                None,                         // text
                Some(r#"{"a": "b"}"#.into()), // payload
                None,                         // action
                Default::default()            // extra fields
            )),
            Wiring::Payload
        );
    }

    #[test]
    fn wiring_dyn_payload() {
        assert_eq!(
            test_wiring(Object::new(
                None,                                   // from_id
                Some(1),                                // peer_id
                None,                                   // user_id
                None,                                   // text
                Some(r#"{"other": "payload"}"#.into()), // payload
                None,                                   // action
                Default::default()                      // extra fields
            )),
            Wiring::DynPayload
        );
    }

    #[test]
    fn wiring_command() {
        assert_eq!(
            test_wiring(Object::new(
                None,                 // from_id
                Some(1),              // peer_id
                None,                 // user_id
                Some("/test".into()), // text
                None,                 // payload
                None,                 // action
                Default::default()    // extra fields
            )),
            Wiring::Command
        );
    }

    #[test]
    fn wiring_regex() {
        assert_eq!(
            test_wiring(Object::new(
                None,                // from_id
                Some(1),             // peer_id
                None,                // user_id
                Some("1337".into()), // text
                None,                // payload
                None,                // action
                Default::default()   // extra fields
            )),
            Wiring::Regex
        );
    }

    #[test]
    fn wiring_no_match() {
        assert_eq!(
            test_wiring(Object::new(
                None,                 // from_id
                Some(1),              // peer_id
                None,                 // user_id
                Some("words".into()), // text
                None,                 // payload
                None,                 // action
                Default::default()    // extra fields
            )),
            Wiring::NoMatch
        );
    }
}
