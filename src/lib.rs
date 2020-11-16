//! This crate provides basic utilities for writing applications with Terminal
//! User Interface (TUI). It provides an event listener, and it provides a
//! handle to a double buffered screen renderer.

#![warn(missing_docs)]

#[macro_use]
mod macros;
mod stdio;

pub mod error;
pub mod string;
pub mod coord;
pub mod color;
pub mod style;
pub mod screen;
pub mod event;
pub mod terminal;

pub use self::stdio::emergency_restore;
