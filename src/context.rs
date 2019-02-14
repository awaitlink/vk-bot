//! The [`Context`] struct.

use crate::{core::Event, request::Object, response::Response};
use rvk::APIClient;
use std::sync::{Arc, Mutex};

/// Stores information necessary for handlers, allows to send the resulting
/// message.
#[derive(Debug)]
pub struct Context {
    event: Event,
    object: Object,
    api: Arc<Mutex<APIClient>>,
    response: Response,
}

impl Context {
    /// Creates a new [`Context`].
    pub fn new(event: Event, object: Object, api: Arc<Mutex<APIClient>>) -> Self {
        Self {
            event,
            object,
            api,
            response: Response::new(),
        }
    }

    /// Returns the event type that caused this handler to run.
    pub fn event(&self) -> Event {
        self.event
    }

    /// Returns the object associated with the event (given by Callback API).
    pub fn object(&self) -> &Object {
        &self.object
    }

    /// Returns an [`rvk::APIClient`], wrapped into
    /// [`Arc`][`std::sync::Arc`]`<`[`Mutex`][`std::sync::Mutex`]`<...>>`.
    pub fn api(&self) -> Arc<Mutex<APIClient>> {
        Arc::clone(&self.api)
    }

    /// Returns the current pending response object.
    pub fn response(&mut self) -> &mut Response {
        &mut self.response
    }
}
