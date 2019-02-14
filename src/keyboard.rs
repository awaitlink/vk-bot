//! Keyboards.
//!
//! # Example
//! ```
//! # use vk_bot::keyboard::{Keyboard, Button, Color};
//! Keyboard::new(
//!     vec![
//!         vec![
//!             Button::new("A", Color::Primary, None),
//!             Button::new("B", Color::Default, None),
//!         ],
//!         vec![
//!             Button::new("C", Color::Positive, None),
//!             Button::new("D", Color::Negative, Some("{\"payload\": \"json\"}".into())),
//!         ],
//!     ],
//!     false,
//! );
//! ```
//!
//! will look like this:
//!
//! ```ignore
//! +-------+-------+
//! |   A   |   B   |
//! +-------+-------+
//! |   C   |   D   |
//! +-------+-------+
//! ```

use serde_derive::Serialize;
use std::fmt::{Display, Error, Formatter};

/// A keyboard consisting of [`Button`]s that may be shown to the user instead
/// of the regular keyboard.
#[derive(Debug, Serialize)]
pub struct Keyboard {
    buttons: Vec<Vec<Button>>,
    one_time: bool,
}

impl Default for Keyboard {
    fn default() -> Self {
        Self {
            buttons: Vec::new(),
            one_time: false,
        }
    }
}

impl Keyboard {
    /// Creates a new keyboard.
    ///
    /// `buttons` is a [`Vec`] of rows (which are [`Vec`]s themselves as well)
    /// of [`Button`]s.
    pub fn new(buttons: Vec<Vec<Button>>, one_time: bool) -> Self {
        Self { buttons, one_time }
    }
}

/// A button of a keyboard.
#[derive(Debug, Serialize)]
pub struct Button {
    color: Color,
    action: ButtonAction,
}

impl Default for Button {
    fn default() -> Self {
        Self {
            color: Default::default(),
            action: Default::default(),
        }
    }
}

impl Button {
    /// Creates a new button.
    pub fn new(label: &str, color: Color, payload: Option<String>) -> Self {
        Self {
            color,
            action: ButtonAction {
                label: label.into(),
                payload,
                ..Default::default()
            },
        }
    }
}

/// A [`Button`]'s "action".
#[derive(Debug, Serialize)]
pub struct ButtonAction {
    r#type: String,
    label: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    payload: Option<String>,
}

impl Default for ButtonAction {
    fn default() -> Self {
        Self {
            r#type: "text".into(),
            label: "Button".into(),
            payload: None,
        }
    }
}

/// The color of a button.
#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Color {
    /// `primary` color.
    Primary,
    /// `default` color.
    Default,
    /// `negative` color.
    Negative,
    /// `positive` color.
    Positive,
}

impl Default for Color {
    fn default() -> Self {
        Color::Default
    }
}

impl Display for Color {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        f.write_str(match self {
            Color::Primary => "primary",
            Color::Default => "default",
            Color::Negative => "negative",
            Color::Positive => "positive",
        })
    }
}

impl From<&str> for Color {
    /// Converts a `&`[`str`] into the associated color.
    ///
    /// # Panics
    /// - when given unknown color
    fn from(s: &str) -> Self {
        match s {
            "primary" => Color::Primary,
            "default" => Color::Default,
            "negative" => Color::Negative,
            "positive" => Color::Positive,

            _ => panic!("unknown color: `{}`", s),
        }
    }
}

impl From<String> for Color {
    /// Converts a [`String`] into the associated color.
    ///
    /// # Panics
    /// - when given unknown color
    fn from(s: String) -> Self {
        s.as_str().into()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn empty_keyboard() -> Result<(), serde_json::Error> {
        let kbd = Keyboard::new(vec![], false);

        assert_eq!(
            serde_json::to_value(&kbd)?,
            json!({
                  "one_time": false,
                  "buttons": [],
            })
        );

        Ok(())
    }

    #[test]
    fn keyboard() -> Result<(), serde_json::Error> {
        let payload = serde_json::to_string(&json!({"payload": "json"}))?;

        let kbd = Keyboard::new(
            vec![
                vec![
                    Button::new("1", Color::Default, None),
                    Button::new("2", Color::Primary, Some(payload.clone())),
                ],
                vec![
                    Button::new("3", Color::Negative, None),
                    Button::new("4", Color::Positive, None),
                ],
            ],
            true,
        );

        assert_eq!(
            serde_json::to_value(&kbd)?,
            json!({
                "buttons":[
                    [
                        {"color":"default","action":{"type":"text","label":"1"}},
                        {"color":"primary","action":{"type":"text","label":"2","payload":payload}}
                    ],
                    [
                        {"color":"negative","action":{"type":"text","label":"3"}},
                        {"color":"positive","action":{"type":"text","label":"4"}}
                    ]
                ],
                "one_time":true
            })
        );

        Ok(())
    }
}
