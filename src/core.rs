//! The [`Core`] struct, supported [`Event`]s, and
//! handler / tester types.

use crate::{context::Context, request::CallbackAPIRequest};
use regex::Regex;
use rvk::APIClient;
use serde_json::Value;
use std::{
    collections::{hash_map::Entry, HashMap},
    convert::TryFrom,
    fmt::{Debug, Display, Error, Formatter},
    ops::Deref,
    str::FromStr,
    sync::{Arc, Mutex},
};

/// Events that are supported for event handlers. See also [`Core::on`].
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

    /// Generated when no matching handler for an event / payload / command /
    /// regex is found.
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

/// Error type for `impl FromStr for Event`.
#[derive(Debug, Clone, PartialOrd, PartialEq, Eq, Ord)]
pub struct EventFromStrError(String);

impl Display for EventFromStrError {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        write!(f, "unknown event: `{}`", self.0)
    }
}

impl FromStr for Event {
    type Err = EventFromStrError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "message_new" => Ok(Event::MessageNew),
            "message_reply" => Ok(Event::MessageReply),
            "message_edit" => Ok(Event::MessageEdit),
            "message_typing_state" => Ok(Event::MessageTypingState),
            "message_allow" => Ok(Event::MessageAllow),
            "message_deny" => Ok(Event::MessageDeny),

            "start" => Ok(Event::Start),
            "service_action" => Ok(Event::ServiceAction),

            "no_match" => Ok(Event::NoMatch),

            _ => Err(EventFromStrError(s.into())),
        }
    }
}

impl TryFrom<&str> for Event {
    type Error = <Event as FromStr>::Err;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        value.parse()
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
/// payload in [`Keyboard`](crate::keyboard::Keyboard)
/// [`Button`](crate::keyboard::Button)s) is interesting for a handler to
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
    regex_handlers: Vec<(Regex, Handler)>,
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
        let entry = self.event_handlers.entry(event);

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
    pub fn regex(mut self, re: Regex, handler: Handler) -> Self {
        self.regex_handlers.push((re, handler));
        self
    }

    /// Handles a request by telling the appropriate [`Handler`] to do so.
    pub fn handle(&self, req: &CallbackAPIRequest, api: Arc<Mutex<APIClient>>) {
        trace!("handling {:#?}", req);

        let event: Event = req.r#type().parse().expect("error while handling request");
        let mut ctx = Context::new(event, req, api);
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

        if !self.try_handle_payload(ctx)
            && !self.try_handle_command(ctx)
            && !self.try_handle_regex(ctx)
        {
            trace!(
                "calling `no_match` (as `message_new` failed to match) handler for {:#?}",
                ctx
            );
            self.handle_event(Event::NoMatch, ctx);
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
                use regex::escape;

                let re = Regex::new(&format!(
                    r#"^( *\[club{}\|.*\])?( *{}{})+"#,
                    ctx.group_id(),
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
            for (re, handler) in self.regex_handlers.iter() {
                if re.is_match(&text) {
                    handler(ctx);
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

    mod event {
        use super::*;

        fn test_display_parse(expected_str: &str, expected_event: Event) {
            let event: Event = expected_str
                .parse()
                .expect(&format!("could not parse event: `{}`", expected_str));
            assert_eq!(event, expected_event);
            let str = format!("{}", event);
            assert_eq!(str, expected_str);
        }

        #[test]
        fn display_and_parse() {
            test_display_parse("message_new", Event::MessageNew);
            test_display_parse("message_reply", Event::MessageReply);
            test_display_parse("message_edit", Event::MessageEdit);
            test_display_parse("message_typing_state", Event::MessageTypingState);
            test_display_parse("message_allow", Event::MessageAllow);
            test_display_parse("message_deny", Event::MessageDeny);

            test_display_parse("start", Event::Start);
            test_display_parse("service_action", Event::ServiceAction);

            test_display_parse("no_match", Event::NoMatch);
        }

        #[test]
        #[should_panic(expected = "unknown event")]
        fn unknown() {
            panic!("{}", "foo_bar".parse::<Event>().unwrap_err());
        }
    }

    mod wiring {
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
                &CallbackAPIRequest::new(
                    Some("secret".into()),
                    1,
                    &Event::MessageNew.to_string(),
                    obj,
                ),
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
                .regex(
                    Regex::new(r#"\d"#).unwrap(),
                    wiring_sender(&tx, Wiring::Regex),
                )
                .on(Event::NoMatch, wiring_sender(&tx, Wiring::NoMatch))
                .handle_event(Event::MessageNew, &mut ctx);

            rx.recv().expect("failed to recv Wiring")
        }

        #[test]
        fn service_action() {
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
        fn start() {
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
        fn payload() {
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
        fn dyn_payload() {
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
        fn command() {
            assert_eq!(
                test_wiring(Object::new(
                    None,                                    // from_id
                    Some(1),                                 // peer_id
                    None,                                    // user_id
                    Some("[club1|Group Name] /test".into()), // text
                    None,                                    // payload
                    None,                                    // action
                    Default::default()                       // extra fields
                )),
                Wiring::Command
            );
        }

        #[test]
        fn regex() {
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
        fn no_match() {
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
}
