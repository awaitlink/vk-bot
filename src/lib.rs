//! Crate for creating chat bots for VK (VKontakte) communities.
//!
//! You can see [`Core`] documentation for information on how to
//! define bot behavior. In particular, make sure to take a look
//! at [`Core::on`] first.
//!
//! # Example
//! The following example is from
//! [`examples/basic.rs`](https://github.com/u32i64/vk-bot/blob/master/examples/basic.rs).
//! It is not tested as a doc-test because [`Bot::start`] never returns.
//!
//! ```ignore
#![doc(include = "../examples/basic.rs")]
//! ```

#![feature(external_doc)]
#![feature(proc_macro_hygiene, decl_macro)]
#![deny(missing_docs)]

#[macro_use]
extern crate rocket;

#[macro_use]
extern crate log;

pub use crate::{
    bot::Bot,
    context::Context,
    core::{Core, Event, Handler, Tester},
};

pub mod bot;
pub mod context;
pub mod core;
pub mod keyboard;
pub mod request;
pub mod response;
