use andiskaz::{
    emergency_restore,
    error::Error,
    terminal::Terminal,
    tstring,
    ui::{info::InfoDialog, input::InputDialog},
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
    let mut dialog = InputDialog::new(
        tstring!["Just type your username"],
        tstring![],
        16,
        |ch| ch.is_ascii_alphabetic() || ch.is_ascii_digit() || ch == '_',
    );
    let username = dialog.select(&mut term).await?;
    InfoDialog::new(tstring!["So, this is your name"], username)
        .run(&mut term)
        .await?;

    let mut dialog = InputDialog::new(
        tstring!["Type your username again (You can cancel)"],
        tstring![],
        16,
        |ch| ch.is_ascii(),
    );
    let maybe_uname = dialog.select_with_cancel(&mut term).await?;
    let message = maybe_uname.unwrap_or_else(|| tstring!["Why you cancelled?"]);
    InfoDialog::new(tstring!["So, this is your name?"], message)
        .run(&mut term)
        .await?;

    Ok(())
}
