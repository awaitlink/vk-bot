//! The [`Context`] struct.

use crate::{core::Event, request::Object};
use rvk::APIClient;
use std::sync::{Arc, Mutex};

/// Stores information necessary for handlers, manages
/// the state of the bot's current response to the
/// message/event this [`Context`] is associated with,
/// and provides convenient ways for modifying that
/// state as well as sending the resulting message.
#[derive(Debug)]
pub struct Context {
    event: Event,
    object: Object,
    api: Arc<Mutex<APIClient>>,
    // TODO
}

impl Context {
    /// Creates a new [`Context`].
    pub fn new(event: Event, object: Object, api: Arc<Mutex<APIClient>>) -> Self {
        Self { event, object, api }
    }
}
