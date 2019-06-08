//! Keyboards.
//!
//! # Example
//! ```
//! # use vk_bot::keyboard::{Keyboard, Button, Color};
//! Keyboard::new(
//!     // Vec of rows
//!     vec![
//!         // Row 0
//!         vec![
//!             Button::text("A", Color::Primary, None),
//!             Button::text("B", Color::Secondary, None),
//!         ],
//!         // Row 1
//!         vec![
//!             Button::text("C", Color::Positive, None),
//!             Button::text("D", Color::Negative, Some(r#"{"payload": "json"}"#.into())),
//!         ],
//!     ],
//!     false,
//! );
//! ```
//!
//! will look like this:
//!
//! ```text
//!         column 0    column 1
//!       +-----------+-----------+
//! row 0 |     A     |     B     |
//!       +-----------+-----------+
//! row 1 |     C     |     D     |
//!       +-----------+-----------+
//! ```

use rvk::objects::Integer;
use serde_derive::Serialize;
use std::{
    convert::TryFrom,
    fmt::{Display, Error, Formatter},
    str::FromStr,
};

/// A keyboard consisting of [`Button`]s that may be shown to the user instead
/// of the regular keyboard.
#[derive(Debug, Serialize, Clone)]
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
    ///
    /// `one_time` sets if the keyboard should be shown only until a button of it is pressed.
    ///
    /// Note that `one_time` is ignored by VK API if keyboard contains a button with
    /// [`Action::VKPay`] or [`Action::OpenApp`].
    pub fn new(buttons: Vec<Vec<Button>>, one_time: bool) -> Self {
        Self { buttons, one_time }
    }

    /// Returns the buttons of this keyboard.
    pub fn buttons(&self) -> &Vec<Vec<Button>> {
        &self.buttons
    }

    /// Indicates whether this keyboard is one-time or not.
    pub fn one_time(&self) -> bool {
        self.one_time
    }
}

/// A button of a keyboard.
#[derive(Debug, Serialize, Clone)]
pub struct Button {
    #[serde(skip_serializing_if = "Option::is_none")]
    color: Option<Color>,
    action: Action,
}

impl Button {
    /// Creates a new text button.
    #[deprecated(since = "2.0.0", note = "please use `text` instead")]
    pub fn new(label: &str, color: Color, payload: Option<String>) -> Self {
        Button::text(label, color, payload)
    }

    /// Creates a new text button (see [`Action::Text`]).
    pub fn text(label: &str, color: Color, payload: Option<String>) -> Self {
        Self {
            color: Some(color),
            action: Action::Text {
                label: label.into(),
                payload,
            },
        }
    }

    /// Creates a new location-sending button (see [`Action::Location`]).
    pub fn location(payload: Option<String>) -> Self {
        Self {
            color: None,
            action: Action::Location { payload },
        }
    }

    /// Creates a new VK Pay button (see [`Action::VKPay`]).
    ///
    /// This button always uses full keyboard width.
    pub fn vk_pay(hash: impl Into<String>) -> Self {
        Self {
            color: None,
            action: Action::VKPay { hash: hash.into() },
        }
    }

    /// Creates a new VK App-opening button (see [`Action::OpenApp`]).
    pub fn open_app(
        app_id: Integer,
        owner_id: Option<Integer>,
        label: impl Into<String>,
        hash: impl Into<String>,
    ) -> Self {
        Self {
            color: None,
            action: Action::OpenApp {
                app_id,
                owner_id,
                label: label.into(),
                hash: hash.into(),
            },
        }
    }

    /// Returns the color of this button.
    pub fn color(&self) -> Option<Color> {
        self.color
    }

    /// Returns the action of this button.
    pub fn action(&self) -> &Action {
        &self.action
    }
}

/// A [`Button`]'s action.
#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "lowercase")]
#[serde(tag = "type")]
pub enum Action {
    /// Simple text button, type `text`.
    Text {
        /// Text shown on the button, will be sent as a message.
        label: String,
        /// Payload that will be sent with the message.
        #[serde(skip_serializing_if = "Option::is_none")]
        payload: Option<String>,
    },

    /// Location-sending button, type `location`.
    ///
    /// Always uses full keyboard width.
    Location {
        /// Payload that will be sent with the message.
        #[serde(skip_serializing_if = "Option::is_none")]
        payload: Option<String>,
    },

    /// VK Pay button, type `vkpay`.
    ///
    /// Always uses full keyboard width.
    VKPay {
        /// VK Pay payment parameters and app identifier in `aid` parameter,
        /// delimited using `&`, e.g. `action=transfer-to-group&group_id=1&aid=10`.
        hash: String,
    },

    /// Button to open a VK App, type `open_app`.
    ///
    /// Always uses full keyboard width.
    #[serde(rename = "open_app")]
    OpenApp {
        /// App identifier.
        app_id: Integer,
        /// Group identifier, if the app needs to be opened in the context of a group.
        owner_id: Option<Integer>,
        /// App name, shown on the button.
        label: String,
        /// Hash for navigation inside an app.
        hash: String,
    },
}

/// The color of a button.
#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Color {
    /// `primary` color, `#5181B8`.
    Primary,
    /// `secondary` color, `#FFFFFF`.
    Secondary,
    /// `negative` color, `#E64646`.
    Negative,
    /// `positive` color, `#4BB34B`.
    Positive,
}

impl Default for Color {
    fn default() -> Self {
        Color::Secondary
    }
}

impl Display for Color {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        f.write_str(match self {
            Color::Primary => "primary",
            Color::Secondary => "secondary",
            Color::Negative => "negative",
            Color::Positive => "positive",
        })
    }
}

/// Error type for `impl FromStr for Color`.
#[derive(Debug, Clone, PartialOrd, PartialEq, Eq, Ord)]
pub struct ColorFromStrError(String);

impl Display for ColorFromStrError {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        write!(f, "unknown color: `{}`", self.0)
    }
}

impl FromStr for Color {
    type Err = ColorFromStrError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "primary" => Ok(Color::Primary),
            "secondary" => Ok(Color::Secondary),
            "negative" => Ok(Color::Negative),
            "positive" => Ok(Color::Positive),

            _ => Err(ColorFromStrError(s.into())),
        }
    }
}

impl TryFrom<&str> for Color {
    type Error = <Color as FromStr>::Err;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        value.parse()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    mod color {
        use super::*;

        fn test_display_parse(expected_str: &str, expected_color: Color) {
            let color: Color = expected_str
                .parse()
                .expect(&format!("could not parse color: `{}`", expected_str));
            assert_eq!(color, expected_color);
            let str = format!("{}", color);
            assert_eq!(str, expected_str);
        }

        #[test]
        fn display_and_parse() {
            test_display_parse("primary", Color::Primary);
            test_display_parse("secondary", Color::Secondary);
            test_display_parse("negative", Color::Negative);
            test_display_parse("positive", Color::Positive);
        }

        #[test]
        #[should_panic(expected = "unknown color")]
        fn unknown() {
            panic!("{}", "foo".parse::<Color>().unwrap_err());
        }
    }

    mod keyboard {
        use super::*;
        use serde_json::json;

        #[test]
        fn empty() -> Result<(), serde_json::Error> {
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
        fn full() -> Result<(), serde_json::Error> {
            let payload = serde_json::to_string(&json!({"payload": "json"}))?;

            let kbd = Keyboard::new(
                vec![
                    vec![
                        Button::text("1", Color::Secondary, None),
                        Button::text("2", Color::Primary, Some(payload.clone())),
                        Button::text("3", Color::Negative, None),
                        Button::text("4", Color::Positive, None),
                    ],
                    vec![Button::location(None)],
                    vec![Button::vk_pay("action=transfer-to-group&group_id=1&aid=10")],
                    vec![Button::open_app(1, Some(1), "My App", "test")],
                ],
                true,
            );

            assert_eq!(
                serde_json::to_value(&kbd)?,
                json!({
                    "buttons":[
                        [
                            {"color":"secondary","action":{"type":"text","label":"1"}},
                            {"color":"primary","action":{"type":"text","label":"2","payload":payload}},
                            {"color":"negative","action":{"type":"text","label":"3"}},
                            {"color":"positive","action":{"type":"text","label":"4"}}
                        ],
                        [{"action":{"type":"location"}}],
                        [{"action":{"type":"vkpay", "hash": "action=transfer-to-group&group_id=1&aid=10"}}],
                        [{"action":{"type":"open_app", "app_id": 1, "owner_id": 1, "label": "My App", "hash": "test"}}]
                    ],
                    "one_time":true
                })
            );

            Ok(())
        }
    }
}
