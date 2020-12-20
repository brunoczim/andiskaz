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
    // Allocates space for a string safe, to print it.
    let message = tstring!["Exits on ESC"];
    // Last event dumped.
    let mut curr_event = None;

    loop {
        // Locks the screen. The word "lock" is important here.
        let mut screen = term.screen.lock().await?;

        // Clears the screen from our previous events. Remember, the cost of
        // this operation is amortized by the double buffering.
        screen.clear(BasicColor::Black.into());

        // Puts our instructions.
        screen.styled_text(&message, Style::with_colors(Color2::default()));

        if let Some(event_dump) = &curr_event {
            // If there was a previous event dump, output it.
            let style = Style::with_colors(Color2::default()).top_margin(2);
            screen.styled_text(&event_dump, style);
        }

        // Drops the screen handle. IMPORTANT. If we don't drop, it will block
        // things like the screen renderer.
        drop(screen);

        let event_string = match term.events.listen().await? {
            // Exits if ESC.
            Event::Key(KeyEvent { main_key: Key::Esc, .. }) => break,
            // Otherwise dump it.
            event => format!("{:?}", event),
        };

        // Save the event dump.
        curr_event = Some(tstring![event_string]);
    }

    Ok(())
}