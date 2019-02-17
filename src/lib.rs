//! Crate for creating chat bots for VK (VKontakte) communities.
//!
//! # Example
//! The following example is from
//! [`examples/basic.rs`](https://github.com/u32i64/vk-bot/blob/master/examples/basic.rs).
//! It is not tested here as [`Bot::start`] never returns.
//!
//! ```ignore
#![doc(include = "../examples/basic.rs")]
//! ```

#![feature(external_doc)]
#![feature(proc_macro_hygiene, decl_macro)]
#![deny(missing_docs)]

#[macro_use]
extern crate rocket;

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
