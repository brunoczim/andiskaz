use andiskaz::{
    color::{BasicColor, Color, Color2},
    coord::{Coord, Coord2},
    emergency_restore,
    error::Error,
    event::{Event, Key, KeyEvent, ResizeEvent},
    screen::{LockedScreen, Tile},
    string::{TermGrapheme, TermString},
    style::Style,
    terminal,
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

    // Initializes game state.
    let game = Game::new();
    // Let's say minimum width is the width of the message.
    let width = game.message.count_graphemes() as Coord;

    // Minimum screen size.
    let min_screen = Coord2 { x: width, y: 2 };

    // Initalizes the terminal handle builder.
    let result = terminal::Builder::new()
        // Sets the minimum screen size.
        .min_screen(min_screen)
        // Finishes the builder and runs our terminal main.
        .run(|term| term_main(game, term))
        // Awaits for the terminal main future.
        .await;

    // If error, prints it out and exit with bad code.
    if let Ok(Err(error)) | Err(error) = result {
        eprintln!("{}", error);
        exit(-1);
    }
}

/// The terminal main function.
async fn term_main(mut game: Game, mut term: Terminal) -> Result<(), Error> {
    // Renders for the first time.
    game.render(&term).await?;

    loop {
        // Awaits for an event.
        match term.events.listen().await {
            // This is a key event.
            Ok(Event::Key(evt)) => {
                // Let the game state handle the key, and they will tell us if
                // we should keep executing (i.e. ESC was not pressed).
                if !game.handle_key(evt, &term).await? {
                    break;
                }
            },

            // This is a resize event. Let the game state handle it.
            Ok(Event::Resize(evt)) => game.handle_resize(evt, &term).await?,

            // Only possible error is if the event listener failed. In this
            // case, Terminal::run or Builder::run will already tell us that
            // this fail happened and we will handle it in main.
            Err(_) => break,
        }
    }

    Ok(())
}

/// Game state.
#[derive(Debug)]
struct Game {
    /// The message we will show at the top of the terminal.
    message: TermString,
    /// The cursor position in the screen (y is always below the message).
    cursor: Coord2,
}

impl Game {
    /// Initializes this game state.
    fn new() -> Self {
        Self {
            message: tstring!["Use arrows to control, press ESC to exit!"],
            // Y never goes above 1, because 0 is the position of the message.
            cursor: Coord2 { x: 0, y: 1 },
        }
    }

    /// Renders the game state. Should be called only on resize or first render.
    async fn render(&self, term: &Terminal) -> Result<(), Error> {
        // Colors of the message. Black foreground, green background.
        let colors = Color2 {
            foreground: BasicColor::Black.into(),
            background: BasicColor::LightGreen.into(),
        };

        // Message style. With the colors above and centralized.
        let style = Style::with_colors(colors).align(1, 2);

        // Locking the screen.
        let mut screen = term.screen.lock().await?;

        // Puts our message.
        screen.styled_text(&self.message, style);

        // Renders the cursor with black foreground, white foreground.
        self.render_cursor(&mut screen, BasicColor::White.into()).await;

        Ok(())
    }

    /// Renders the cursor with the given color.
    async fn render_cursor<'term>(
        &self,
        screen: &mut LockedScreen<'term>,
        color: Color,
    ) {
        // A space with the given colors.
        let tile = Tile {
            grapheme: TermGrapheme::space(),
            colors: Color2 {
                // Dummy foreground color here. Ignored since the grapheme is a
                // blank space.
                foreground: BasicColor::White.into(),
                // Actually used color.
                background: color,
            },
        };
        // Sets the cursor's character and colors.
        screen.set(self.cursor, tile);
    }

    /// Handles a key event.
    async fn handle_key(
        &mut self,
        key: KeyEvent,
        term: &Terminal,
    ) -> Result<bool, Error> {
        // Locks the screen.
        let mut screen = term.screen.lock().await?;
        // "Erases" the cursor using the same color as the background color
        // of all the terminal.
        self.render_cursor(&mut screen, BasicColor::Black.into()).await;

        // Whether the "game" should keep executing.
        let mut executing = true;

        // We don't want any modifiers.
        if !key.ctrl && !key.alt && !key.shift {
            // Matches the main key (i.e. the key that is not a modifier).
            match key.main_key {
                // ESC. Stop executing.
                Key::Esc => executing = false,

                // Arrow up. Moves the cursor up.
                Key::Up => {
                    if self.cursor.y > 1 {
                        self.cursor.y -= 1;
                    }
                },

                // Arrow down. Moves the cursor down.
                Key::Down => {
                    if self.cursor.y < screen.size().y - 1 {
                        self.cursor.y += 1;
                    }
                },

                // Arrow left. Moves the cursor left.
                Key::Left => {
                    if self.cursor.x > 0 {
                        self.cursor.x -= 1;
                    }
                },

                // Arrow right. Moves the cursor right.
                Key::Right => {
                    if self.cursor.x < screen.size().x - 1 {
                        self.cursor.x += 1;
                    }
                },

                // Otherwise, do nothing.
                _ => (),
            }
        };

        // Renders the new cursor.
        self.render_cursor(&mut screen, BasicColor::White.into()).await;

        Ok(executing)
    }

    /// Handles a resize event.
    async fn handle_resize(
        &mut self,
        evt: ResizeEvent,
        term: &Terminal,
    ) -> Result<(), Error> {
        // Adjust the cursor position, if it would be outside of the terminal.
        if self.cursor.y >= evt.size.y {
            self.cursor.y = evt.size.y - 1;
        }
        if self.cursor.x >= evt.size.x {
            self.cursor.x = evt.size.x - 1;
        }

        // Renders everything, since resizing scrambles the screen.
        self.render(term).await?;

        Ok(())
    }
}
