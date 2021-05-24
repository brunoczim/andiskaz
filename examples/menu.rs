use andiskaz::{
    emergency_restore,
    error::Error,
    terminal::Terminal,
    tstring,
    ui::{info::InfoDialog, menu::Menu},
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
    let mut menu = Menu::new(
        tstring!["Choose One"],
        vec![
            tstring!["This is the right choice."],
            tstring!["This is the wrong choice."],
            tstring!["I cannot judge this choice."],
        ],
    );
    let option = menu.select(&mut term).await?;
    InfoDialog::new(
        tstring!["Judging what you chose"],
        menu.options[option].clone(),
    )
    .run(&mut term)
    .await?;

    menu.title = tstring!["Choose One But You Can Cancel"];
    let maybe_option = menu.select_with_cancel(&mut term).await?;

    let message = match maybe_option {
        Some(option) => menu.options[option].clone(),
        None => tstring!["You chose nothing? Really?"],
    };
    InfoDialog::new(tstring!["Judging what you chose"], message)
        .run(&mut term)
        .await?;

    Ok(())
}
