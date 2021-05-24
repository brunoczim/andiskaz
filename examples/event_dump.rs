//! This example dumps any event that may happen.

use andiskaz::{
    color::{BasicColor, Color2},
    emergency_restore,
    error::Error,
    event::{Event, Key, KeyEvent},
    style::Style,
    terminal::Terminal,
    tstring,
};
use std::{panic, process::exit};

/// Asynchronous main of a tokio project.
#[tokio::main]
async fn main() {
    // Sets panic hook so we can see the panic even if terminal was being used
    // in raw mode.
    panic::set_hook(Box::new(|info| {
        let _ = emergency_restore();
        eprintln!("{}", info);
    }));

    // Creates a terminal with default settings and runs it.
    let result = Terminal::run(term_main).await;
    // If error, prints it out and exits with bad code.
    if let Ok(Err(error)) | Err(error) = result {
        eprintln!("{}", error);
        exit(-1);
    }
}

/// The terminal main function.
async fn term_main(mut term: Terminal) -> Result<(), Error> {
    // Allocates memory for a string which is safe to be printed.
    let message = tstring!["Exits on ESC"];
    // Style for the message string.
    let msg_style = Style::with_colors(Color2::default());
    // Style for the event string.
    let evt_style = Style::with_colors(Color2::default()).top_margin(2);
    // Initial rendering.
    term.lock_now().await?.screen().styled_text(&message, msg_style);

    loop {
        // Listens for an event, and when it happens, returns a terminal guard,
        // a "session".
        let mut session = term.listen().await?;

        // There should be an event, then... print it.
        if let Some(event) = session.event() {
            match event {
                // If ESC is pressed, we exit.
                Event::Key(KeyEvent { main_key: Key::Esc, .. }) => break,
                // If resized, message needs to be reprinted.
                Event::Resize(_) => {
                    session.screen().clear(BasicColor::Black.into());
                    session.screen().styled_text(&message, msg_style);
                },
                _ => (),
            }

            // Finally, dump this event.
            session
                .screen()
                .styled_text(&tstring![format!("{:?}", event)], evt_style);
        }
    }

    Ok(())
}
