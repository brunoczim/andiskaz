//! This module defines utilities central to the game state.

use crate::{
    food::Food,
    plane::{Bounds, Direction},
    snake::Snake,
};
use andiskaz::{
    color::{BasicColor, Color2},
    coord::Coord2,
    error::Error,
    event::{Event, Key, KeyEvent, ResizeEvent},
    screen::{LockedScreen, Tile},
    string::{TermGrapheme, TermString},
    style::Style,
    terminal::Terminal,
    tstring,
};
use std::time::Duration;
use tokio::time;

/// In what manner the game ended.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum EndKind {
    /// User won!
    Win,
    /// User lost...
    Loss,
    /// User cancelled via ESC.
    Cancel,
}

/// The central game state.
#[derive(Debug)]
pub struct Game {
    /// Bounds of the game's plane.
    bounds: Bounds,
    /// Food/fruit's data.
    food: Food,
    /// Snake's data.
    snake: Snake,
    /// Tile for the vertical component of the border.
    vertical_tile: Tile,
    /// Tile for the horizontal component of the border.
    horizontal_tile: Tile,
    /// Tile for the corner component of the border.
    corner_tile: Tile,
    /// Message displayed above the border.
    message: TermString,
}

impl Game {
    /// Initializes the game state, given terminal info.
    pub async fn new(terminal: &Terminal) -> Result<Self, Error> {
        // Locks the screen.
        let screen = terminal.screen.lock().await?;

        // Tile for the snake's body.
        let grapheme = TermGrapheme::new_lossy("O");
        let colors = Color2 {
            foreground: BasicColor::Black.into(),
            background: BasicColor::White.into(),
        };
        let body_tile = Tile { grapheme, colors };

        // Tile for the snake's head.
        let grapheme = TermGrapheme::new_lossy("@");
        let colors = Color2 {
            foreground: BasicColor::Black.into(),
            background: BasicColor::White.into(),
        };
        let head_tile = Tile { grapheme, colors };

        // Tile for the food/fruit.
        let grapheme = TermGrapheme::new_lossy("ɔ́");
        let colors = Color2 {
            foreground: BasicColor::Black.into(),
            background: BasicColor::LightRed.into(),
        };
        let food_tile = Tile { grapheme, colors };

        // Tile for the vertical component of the border.
        let grapheme = TermGrapheme::new_lossy("|");
        let colors = Color2 {
            foreground: BasicColor::Black.into(),
            background: BasicColor::LightGreen.into(),
        };
        let vertical_tile = Tile { grapheme, colors };

        // Tile for the horizontal component of the border.
        let grapheme = TermGrapheme::new_lossy("—");
        let colors = Color2 {
            foreground: BasicColor::Black.into(),
            background: BasicColor::LightGreen.into(),
        };
        let horizontal_tile = Tile { grapheme, colors };

        // Tile for the corner component of the border.
        let grapheme = TermGrapheme::new_lossy("+");
        let colors = Color2 {
            foreground: BasicColor::Black.into(),
            background: BasicColor::LightGreen.into(),
        };
        let corner_tile = Tile { grapheme, colors };

        // Message shown above the border.
        let message = tstring!["ESC to exit, arrows to move"];

        // Bounds of the plane (where the snake is allowed to go to).
        let bounds = Self::make_bounds(screen.size());

        // Initialization of snake and food.
        let snake = Snake::new(body_tile, head_tile, bounds);
        let food = Food::new(food_tile, &snake, bounds);

        Ok(Self {
            bounds,
            snake,
            food,
            corner_tile,
            vertical_tile,
            horizontal_tile,
            message,
        })
    }

    /// Runs the game, given the initial state, terminal handle, and interval
    /// between ticks.
    pub async fn run(
        mut self,
        terminal: &mut Terminal,
        tick: Duration,
    ) -> Result<EndKind, Error> {
        let mut interval = time::interval(tick);
        loop {
            let session = terminal.session().await?;
            let maybe_end = self.tick(session.event);
            self.render(&mut session.screen);
            if let Some(end) = maybe_end {
                break Ok(end);
            }
            interval.tick().await;
        }
    }

    /// Computes the plane bounds from the screen size.
    fn make_bounds(screen_size: Coord2) -> Bounds {
        Bounds {
            min: Coord2 { x: 1, y: 2 },
            max: Coord2 { x: screen_size.x - 2, y: screen_size.y - 2 },
        }
    }

    /// Performs the game's logical operations inside a game tick. Returns
    /// `Some` if the end of the game happens.
    fn tick(&mut self, event: Option<Event>) -> Option<EndKind> {
        // Handles the event.
        if let Some(curr_event) = event {
            if let Some(end) = self.handle_event(curr_event) {
                return Some(end);
            }
        }

        // Moves the snake.
        self.move_snake()
    }

    /// Handles an event, be it resizing or key pressing. Returns `Some` if the
    /// end of the game happens.
    fn handle_event(&mut self, event: Event) -> Option<EndKind> {
        match event {
            // ESC was pressed. Cancels the game.
            Event::Key(KeyEvent { main_key: Key::Esc, .. }) => {
                return Some(EndKind::Cancel);
            },

            // Arrow up was pressed, with no modifiers. Changes direction to up.
            Event::Key(KeyEvent {
                main_key: Key::Up,
                shift: false,
                ctrl: false,
                alt: false,
            }) => {
                self.snake.change_direction(Direction::Up);
            },

            // Arrow down was pressed, with no modifiers. Changes direction to
            // down.
            Event::Key(KeyEvent {
                main_key: Key::Down,
                shift: false,
                ctrl: false,
                alt: false,
            }) => {
                self.snake.change_direction(Direction::Down);
            },

            // Arrow left was pressed, with no modifiers. Changes direction to
            // left.
            Event::Key(KeyEvent {
                main_key: Key::Left,
                shift: false,
                ctrl: false,
                alt: false,
            }) => {
                self.snake.change_direction(Direction::Left);
            },

            // Arrow right was pressed, with no modifiers. Changes direction to
            // right.
            Event::Key(KeyEvent {
                main_key: Key::Right,
                shift: false,
                ctrl: false,
                alt: false,
            }) => {
                self.snake.change_direction(Direction::Right);
            },

            // Screen was resized. Propagates the new size.
            Event::Resize(resize) => {
                if let Some(end) = self.resize(resize) {
                    return Some(end);
                }
            },

            // Other keys, ignores.
            Event::Key(_) => (),
        }

        None
    }

    /// Moves the snake. Returns `Some` if the end of the game happens.
    fn move_snake(&mut self) -> Option<EndKind> {
        match self.snake.mov(self.bounds, &self.food) {
            // Handles the case where the food was eaten.
            Some(true) => {
                if self.snake.length() >= self.win_size() {
                    return Some(EndKind::Win);
                }

                self.food.regenerate(&self.snake, self.bounds);
            },

            // Handles the case where the food was not eaten, but this is not
            // the end of the game.
            Some(false) => (),

            // Out of bounds. End of the game.
            None => return Some(EndKind::Loss),
        }

        if self.snake.head_intersects() {
            // If the head intersects the body, game over.
            Some(EndKind::Loss)
        } else {
            None
        }
    }

    /// Renders all game data into the screen.
    fn render(&self, screen: &mut LockedScreen) {
        // Clears screen with black as background color.
        screen.clear(BasicColor::Black.into());

        // Renders the snake.
        self.snake.render(screen);
        // Renders the food.
        self.food.render(screen);
        // Renders the borders.
        self.render_borders(screen);
        // Renders the message above the borders.
        self.render_message(screen);
    }

    /// Renders the borders via the given locked screen.
    fn render_borders<'screen>(&self, screen: &mut LockedScreen<'screen>) {
        // Top border.
        for x in self.bounds.min.x .. self.bounds.max.x + 1 {
            screen.set(
                Coord2 { x, y: self.bounds.min.y - 1 },
                self.horizontal_tile.clone(),
            );
        }
        // Down border.
        for x in self.bounds.min.x .. self.bounds.max.x + 1 {
            screen.set(
                Coord2 { x, y: self.bounds.max.y + 1 },
                self.horizontal_tile.clone(),
            );
        }

        // Left border.
        for y in self.bounds.min.y .. self.bounds.max.y + 1 {
            screen.set(
                Coord2 { y, x: self.bounds.min.x - 1 },
                self.vertical_tile.clone(),
            );
        }
        // Right border.
        for y in self.bounds.min.y .. self.bounds.max.y + 1 {
            screen.set(
                Coord2 { y, x: self.bounds.max.x + 1 },
                self.vertical_tile.clone(),
            );
        }

        // Corners.
        screen.set(
            Coord2 { x: self.bounds.min.x - 1, y: self.bounds.min.y - 1 },
            self.corner_tile.clone(),
        );
        screen.set(
            Coord2 { x: self.bounds.min.x - 1, y: self.bounds.max.y + 1 },
            self.corner_tile.clone(),
        );
        screen.set(
            Coord2 { x: self.bounds.max.x + 1, y: self.bounds.min.y - 1 },
            self.corner_tile.clone(),
        );
        screen.set(
            Coord2 { x: self.bounds.max.x + 1, y: self.bounds.max.y + 1 },
            self.corner_tile.clone(),
        );
    }

    /// Renders the message above the borders.
    fn render_message(&self, screen: &mut LockedScreen) {
        // White foreground, black background.
        let colors = Color2 {
            foreground: BasicColor::White.into(),
            background: BasicColor::Black.into(),
        };
        // Centralized, at the top.
        let style = Style::with_colors(colors).align(1, 2);
        // Writes the text into the buffer.
        screen.styled_text(&self.message, style);
    }

    /// Resizes the game state about screen size.
    fn resize(&mut self, event: ResizeEvent) -> Option<EndKind> {
        // Keeps track if the end is reached.
        let mut end = None;
        // New bounds.
        self.bounds = Self::make_bounds(event.size);

        if self.snake.saturate_at_bounds(self.bounds) {
            // We will consider that the game is lost if snake gets outside of
            // the plane when resizing.
            end = Some(EndKind::Loss);
        }
        if !self.food.in_bounds(self.bounds) {
            // If the food is outside of the plane, regenerates.
            self.food.regenerate(&self.snake, self.bounds);
        }

        end
    }

    /// Computes the size of the snake when the player wins.
    fn win_size(&self) -> usize {
        let x = usize::from(self.bounds.max.x + 1 - self.bounds.min.x);
        let y = usize::from(self.bounds.max.y + 1 - self.bounds.min.y);

        // Mean(x,y)/2
        (x + y) / 4
    }
}
