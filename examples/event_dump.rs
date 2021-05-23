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
    term.enter()
        .await?
        .screen()
        .styled_text(&message, Style::with_colors(Color2::default()));

    loop {
        let mut session = term.listen().await?;
        session.screen().clear(BasicColor::Black.into());
        session
            .screen()
            .styled_text(&message, Style::with_colors(Color2::default()));

        if let Some(event) = session.event() {
            if let Event::Key(KeyEvent { main_key: Key::Esc, .. }) = event {
                break;
            }
            let style = Style::with_colors(Color2::default()).top_margin(2);
            session
                .screen()
                .styled_text(&tstring![format!("{:?}", event)], style);
        }
    }

    Ok(())
}
