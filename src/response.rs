//! Structs for storing response information.

use crate::keyboard::Keyboard;
use std::fmt::{Display, Error, Formatter};

/// Manages the bot's current response to a message/event.
#[derive(Debug)]
pub struct Response {
    message: String,
    attachments: Vec<AttachmentInformation>,
    keyboard: Option<Keyboard>,
}

impl Default for Response {
    fn default() -> Self {
        Self {
            message: String::new(),
            attachments: Vec::new(),
            keyboard: None,
        }
    }
}

impl Response {
    /// Creates a new [`Response`].
    pub fn new() -> Self {
        Default::default()
    }

    // TODO
}

/// Essentially an attachment's unique ID, possibly with an access key.
#[derive(Debug)]
pub struct AttachmentInformation {
    r#type: String,
    owner_id: i64,
    resource_id: i64,
    access_key: Option<String>,
}

impl Display for AttachmentInformation {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        f.write_str(&format!(
            "{}{}_{}{}",
            self.r#type,
            self.owner_id,
            self.resource_id,
            match &self.access_key {
                Some(key) => format!("_{}", key),
                None => "".into(),
            }
        ))
    }
}

impl From<(String, i64, i64)> for AttachmentInformation {
    fn from((r#type, owner_id, resource_id): (String, i64, i64)) -> Self {
        Self {
            r#type,
            owner_id,
            resource_id,
            access_key: None,
        }
    }
}

impl From<(String, i64, i64, String)> for AttachmentInformation {
    fn from((r#type, owner_id, resource_id, access_key): (String, i64, i64, String)) -> Self {
        Self {
            r#type,
            owner_id,
            resource_id,
            access_key: Some(access_key),
        }
    }
}

impl AttachmentInformation {
    /// Creates a new [`AttachmentInformation`].
    pub fn new(
        r#type: String,
        owner_id: i64,
        resource_id: i64,
        access_key: Option<String>,
    ) -> Self {
        Self {
            r#type,
            owner_id,
            resource_id,
            access_key,
        }
    }
}
