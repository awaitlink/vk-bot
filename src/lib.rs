//! Crate for creating chat bots for VK (VKontakte) communities.

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
pub mod request;
