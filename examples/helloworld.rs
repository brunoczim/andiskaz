use andiskaz::{
    color::Color2,
    emergency_restore,
    error::Error,
    event::Event,
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
    let string = tstring!["Hello, World! Press any key..."];
    let style = Style::with_colors(Color2::default());
    term.enter().await?.screen().styled_text(&string, style);

    loop {
        // Locks the screen. The word "lock" is important here.
        let mut session = term.listen().await?;

        match session.event() {
            Some(Event::Key(_)) => break,
            Some(Event::Resize(_)) => {
                session.screen().styled_text(&string, style);
            },
            None => (),
        }
    }

    Ok(())
}
