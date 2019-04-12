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
//!             Button::new("A", Color::Primary, None),
//!             Button::new("B", Color::Default, None),
//!         ],
//!         // Row 1
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
//! ```text
//!         column 0    column 1
//!       +-----------+-----------+
//! row 0 |     A     |     B     |
//!       +-----------+-----------+
//! row 1 |     C     |     D     |
//!       +-----------+-----------+
//! ```

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
    pub fn new(buttons: Vec<Vec<Button>>, one_time: bool) -> Self {
        Self { buttons, one_time }
    }
}

/// A button of a keyboard.
#[derive(Debug, Serialize, Clone)]
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
#[derive(Debug, Serialize, Clone)]
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
            "default" => Ok(Color::Default),
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
            test_display_parse("default", Color::Default);
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
        fn test1() -> Result<(), serde_json::Error> {
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
}
