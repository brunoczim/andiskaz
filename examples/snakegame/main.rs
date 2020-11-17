mod plane;
mod snake;
mod food;
mod game;

use crate::game::{EndKind, Game};
use andiskaz::{
    color::{BasicColor, Color2},
    emergency_restore,
    error::Error as AndiskazError,
    style::Style,
    terminal::Terminal,
    tstring,
};
use backtrace::Backtrace;
use std::{panic, process::exit, time::Duration};
use tokio::time;

/// Asynchronous main of a tokio project.
#[tokio::main]
async fn main() {
    // Sets panic hook so we can see the panic even if terminal was being used
    // in raw mode.
    panic::set_hook(Box::new(|info| {
        let backtrace = Backtrace::new();
        let _ = emergency_restore();
        eprintln!("{}\n{:?}", info, backtrace);
    }));

    // Creates a terminal with default settings and run it.
    let result = Terminal::run(term_main).await;
    // If error, prints it out and exits with bad code.
    if let Ok(Err(error)) | Err(error) = result {
        eprintln!("{}", error);
        exit(-1);
    }
}

/// The terminal main function.
async fn term_main(mut terminal: Terminal) -> Result<(), AndiskazError> {
    let tick = Duration::from_millis(60);
    let game = Game::new(&mut terminal).await?;
    let end_kind = game.run(&mut terminal, tick).await?;

    match end_kind {
        EndKind::Cancel => (),

        EndKind::Win => {
            let colors = Color2 {
                foreground: BasicColor::Black.into(),
                background: BasicColor::LightGreen.into(),
            };
            let mut screen = terminal.screen.lock().await?;
            let style = Style::with_colors(colors)
                .align(1, 2)
                .top_margin(screen.size().y / 2);
            screen.styled_text(&tstring!["YOU WON!!"], style);
            drop(screen);

            wait_key_delay(&mut terminal).await?;
        },

        EndKind::Loss => {
            let colors = Color2 {
                foreground: BasicColor::Black.into(),
                background: BasicColor::LightRed.into(),
            };
            let mut screen = terminal.screen.lock().await?;
            let style = Style::with_colors(colors)
                .align(1, 2)
                .top_margin(screen.size().y / 2);
            screen.styled_text(&tstring!["YOU LOST!!"], style);
            drop(screen);

            wait_key_delay(&mut terminal).await?;
        },
    }

    Ok(())
}

async fn wait_key_delay(terminal: &mut Terminal) -> Result<(), AndiskazError> {
    time::sleep(Duration::from_millis(100)).await;
    terminal.events.check()?;

    time::sleep(Duration::from_millis(500)).await;
    terminal.events.listen().await?;
    Ok(())
}
