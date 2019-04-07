//! Structs for storing request information.

use rvk::objects::Integer;
use serde_derive::Deserialize;
use serde_json::Value;
use std::collections::HashMap;

/// A request received from Callback API.
#[derive(Debug, Deserialize)]
pub struct CallbackAPIRequest {
    secret: Option<String>,
    group_id: i32,
    #[serde(rename = "type")]
    r#type: String,
    #[serde(default)]
    object: Object,
}

impl CallbackAPIRequest {
    /// Creates a new [`CallbackAPIRequest`].
    pub fn new(secret: Option<String>, group_id: i32, r#type: &str, object: Object) -> Self {
        Self {
            secret,
            group_id,
            r#type: r#type.into(),
            object,
        }
    }

    /// Returns the secret sent in this request, if present.
    pub fn secret(&self) -> Option<String> {
        self.secret.clone()
    }

    /// Returns the group ID sent in this request.
    pub fn group_id(&self) -> i32 {
        self.group_id
    }

    /// Returns the type of this request.
    pub fn r#type(&self) -> &str {
        &self.r#type
    }

    /// Returns the [`Object`] sent in this request.
    pub fn object(&self) -> &Object {
        &self.object
    }
}

/// An object of a [`CallbackAPIRequest`].
#[derive(Debug, Deserialize, Clone)]
pub struct Object {
    from_id: Option<Integer>,
    peer_id: Option<Integer>,
    user_id: Option<Integer>,
    text: Option<String>,
    payload: Option<String>,
    action: Option<Value>,

    #[serde(flatten)]
    extra: HashMap<String, Value>,
}

impl Default for Object {
    fn default() -> Self {
        Self {
            from_id: None,
            peer_id: None,
            user_id: None,
            text: None,
            payload: None,
            action: None,
            extra: Default::default(),
        }
    }
}

impl Object {
    /// Creates a new [`Object`].
    pub fn new(
        from_id: Option<Integer>,
        peer_id: Option<Integer>,
        user_id: Option<Integer>,
        text: Option<String>,
        payload: Option<String>,
        action: Option<Value>,
        extra: HashMap<String, Value>,
    ) -> Self {
        Self {
            from_id,
            peer_id,
            user_id,
            text,
            payload,
            action,
            extra,
        }
    }

    /// Returns the "from" ID of this [`Object`].
    pub fn get_from_id(&self) -> &Option<Integer> {
        &self.from_id
    }

    /// Returns the peer ID of this [`Object`].
    pub fn peer_id(&self) -> &Option<Integer> {
        &self.peer_id
    }

    /// Returns the user ID of this [`Object`].
    pub fn user_id(&self) -> &Option<Integer> {
        &self.user_id
    }

    /// Returns the text of this [`Object`].
    pub fn text(&self) -> &Option<String> {
        &self.text
    }

    /// Returns the payload of this [`Object`].
    pub fn payload(&self) -> &Option<String> {
        &self.payload
    }

    /// Returns the action of this [`Object`].
    pub fn action(&self) -> &Option<Value> {
        &self.action
    }

    /// Returns extra fields of this [`Object`].
    pub fn extra(&self) -> &HashMap<String, Value> {
        &self.extra
    }
}
