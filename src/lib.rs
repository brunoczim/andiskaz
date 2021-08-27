//! This crate provides basic utilities for writing applications with Terminal
//! User Interface (TUI). It provides an event listener, and it provides a
//! handle to a double buffered screen renderer.
//!
//! # Examples
//!
//! This is an example for a simple Hello, World! application. For more
//! examples, see examples folder.
//!
//! ```no_run
//! use andiskaz::{
//!     color::Color2,
//!     emergency_restore,
//!     error::Error,
//!     event::Event,
//!     style::Style,
//!     terminal::Terminal,
//!     tstring,
//! };
//! use std::{panic, process::exit};
//!
//! /// Asynchronous main of a tokio project.
//! #[tokio::main]
//! async fn main() {
//!     // Sets panic hook so we can see the panic even if terminal was being used
//!     // in raw mode.
//!     panic::set_hook(Box::new(|info| {
//!         let _ = emergency_restore();
//!         eprintln!("{}", info);
//!     }));
//!
//! // Creates a terminal with default settings and runs it.
//!     let result = Terminal::run(term_main).await;
//!     // If error, prints it out and exits with bad code.
//!     if let Ok(Err(error)) | Err(error) = result {
//!         eprintln!("{}", error);
//!         exit(-1);
//!     }
//! }
//!
//! /// The terminal main function.
//! async fn term_main(mut term: Terminal) -> Result<(), Error> {
//!     // Allocates memory for a string which is safe to be printed.
//!     let string = tstring!["Hello, World! Press any key..."];
//!     // Style for the string.
//!     let style = Style::with_colors(Color2::default());
//!     // Initial rendering.
//!     term.lock_now().await?.screen().styled_text(&string, style);
//!
//!     loop {
//!         // Awaits for an event (key pressed, screen resized, etc).
//!         let mut session = term.listen().await?;
//!
//!         // Gets the event for the current terminal session.
//!         match session.event() {
//!             // We expect a key to exit the example.
//!             Some(Event::Key(_)) => break,
//!             // User resized screen? Then the whole screen was thrown out,
//!             // re-rendering is required.
//!             Some(Event::Resize(_)) => {
//!                 session.screen().styled_text(&string, style);
//!             },
//!             // Won't really happen since we waited for an event.
//!             None => (),
//!         }
//!     }
//!
//!     Ok(())
//! }
//! ```

#![warn(missing_docs)]
#![deny(unused_must_use)]

#[macro_use]
mod macros;
mod stdio;

pub mod error;
pub mod string;
pub mod coord;
pub mod color;
pub mod tile;
pub mod style;
pub mod screen;
pub mod event;
pub mod terminal;
pub mod ui;

pub use self::stdio::emergency_restore;
