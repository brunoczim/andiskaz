//! This example implements the "snake game". It does not have a menu or
//! anything like that. Arrows control the snake, ESC exits.

mod snake;
mod food;
mod game;

use crate::game::{EndKind, Game};
use andiskaz::{
    color::{BasicColor, Color2},
    coord::Vec2,
    emergency_restore,
    error::Error as AndiskazError,
    style::Style,
    terminal,
    terminal::Terminal,
    tstring,
};
use backtrace::Backtrace;
use std::{panic, process::exit, time::Duration};

/// Time interval between game ticks.
const TICK: Duration = Duration::from_millis(70);
/// Delay before waiting for the user.
const WAIT_USER_DELAY: Duration = Duration::from_millis(100);

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

    // Creates and runs a terminal with given settings.
    let result = terminal::Builder::default()
        // Interval between event polling.
        .event_interval(WAIT_USER_DELAY / 2)
        // Minimum screen size.
        .min_screen(Vec2 { x: 50, y: 20 })
        // Interval between rendering frames.
        .frame_time(TICK / 2)
        // Runs.
        .run(term_main)
        .await;

    // If error, prints it out and exits with bad code.
    if let Ok(Err(error)) | Err(error) = result {
        eprintln!("{}", error);
        exit(-1);
    }
}

/// The terminal main function.
async fn term_main(mut terminal: Terminal) -> Result<(), AndiskazError> {
    // Initializes the game getting info from the given terminal.
    let game = Game::new(terminal.lock_now().await?.screen()).await?;
    // Runs the game and gets info on how it ended.
    let end_kind = game.run(&mut terminal, TICK).await?;

    match end_kind {
        // User cancelled (ESC)? Do nothing.
        EndKind::Cancel => (),

        // User won. Print message.
        EndKind::Win => {
            // Black foreground, green background.
            let colors = Color2 {
                foreground: BasicColor::Black.into(),
                background: BasicColor::LightGreen.into(),
            };
            {
                let mut session = terminal.lock_now().await?;
                // Style for message. Centralized.
                let style = Style::with_colors(colors)
                    .align(1, 2)
                    .top_margin(session.screen().size().y / 2);
                // Puts message.
                session.screen().styled_text(&tstring!["YOU WON!!"], style);
            }

            // Waits a key with a delay before waiting for the key.
            terminal.wait_user(WAIT_USER_DELAY).await?;
        },

        // User lost. Print message.
        EndKind::Loss => {
            // Black foreground, red background.
            let colors = Color2 {
                foreground: BasicColor::Black.into(),
                background: BasicColor::LightRed.into(),
            };
            {
                let mut session = terminal.lock_now().await?;
                // Style for message. Centralized.
                let style = Style::with_colors(colors)
                    .align(1, 2)
                    .top_margin(session.screen().size().y / 2);
                // Puts message.
                session.screen().styled_text(&tstring!["YOU LOST!!"], style);
            }

            // Waits a key with a delay before waiting for the key.
            terminal.wait_user(WAIT_USER_DELAY).await?;
        },
    }

    Ok(())
}
