use andiskaz::{
    color::Color2,
    input::Event,
    style::Style,
    terminal,
    terminal::Terminal,
    tstring,
};
use std::{panic, process::exit};

/// Asynchronous main of a tokio project.
#[tokio::main]
async fn main() {
    // Set panic hook so we can see the panic even if terminal was being used in
    // raw mode.
    panic::set_hook(Box::new(|info| {
        let _ = terminal::emergency_restore();
        eprintln!("{}", info);
    }));

    // Create a terminal with default settings and run it.
    let result = Terminal::run(term_main).await;
    // If error, print it out and exit with bad code.
    if let Err(error) = result {
        eprintln!("{}", error);
        exit(-1);
    }
}

/// The terminal main function.
async fn term_main(term: Terminal) {
    // Allocate space for a string safe for being printed.
    let string = tstring!["Hello, World! Press any key..."];
    loop {
        // Lock the screen. The word "lock" is important here.
        let mut screen = term.lock_screen().await;
        // Put our message.
        screen.styled_text(&string, Style::with_colors(Color2::default()));
        // Drop the screen handle. IMPORTANT. If we don't drop, it will block
        // things like the screen renderer.
        drop(screen);

        // Check if a key was pressed. We'll wait until an event happens. If a
        // key was pressed or an error happened, we exit. The only event that
        // makes this `if` fail is a resize event, so, we have to re-print our
        // message in the next iteration.
        if let Ok(Event::Key(_)) | Err(_) = term.listen_event().await {
            break;
        }
    }
}
