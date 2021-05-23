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
//!     // Sets panic hook so we can see the panic even if terminal was being
//!     // used in raw mode.
//!     panic::set_hook(Box::new(|info| {
//!         let _ = emergency_restore();
//!         eprintln!("{}", info);
//!     }));
//!
//!     // Create a terminal with default settings and run it.
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
//!     // Allocates space for a string safe, to print it.
//!     let string = tstring!["Hello, World! Press any key..."];
//!     loop {
//!         // Locks the screen. The word "lock" is important here.
//!         let mut screen = term.screen.lock().await?;
//!         // Puts our message.
//!         screen.styled_text(&string, Style::with_colors(Color2::default()));
//!         // Drops the screen handle. IMPORTANT. If we don't drop, it will
//!         // block things like the screen renderer.
//!         drop(screen);
//!
//!         // Checks if a key was pressed. We'll wait until an event happens.
//!         // If a key was pressed or an error happened, we exit. The only
//!         // event that makes this `if` fail is a resize event, so, we have to
//!         // re-print our message in the next iteration.
//!         if let Ok(Event::Key(_)) | Err(_) = term.events.listen().await {
//!             break;
//!         }
//!     }
//!
//!     Ok(())
//! }
//! ```

//#![warn(missing_docs)]
#![deny(unused_must_use)]

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
