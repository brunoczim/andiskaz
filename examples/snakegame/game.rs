//! This module defines utilities central to the game state.

use crate::{food::Food, snake::Snake};
use andiskaz::{
    color::{BasicColor, Color2},
    coord::{Coord, Vec2},
    error::Error,
    event::{Event, Key, KeyEvent, ResizeEvent},
    screen::Screen,
    string::{TermGrapheme, TermString},
    style::Style,
    terminal::Terminal,
    tile::Tile,
    tstring,
};
use gardiz::{direc::Direction, rect::Rect};
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

/// Tag of the internal state of the game.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
enum State {
    /// Game running normally.
    Running,
    /// Screen has been put to a bad size.
    BadScreen,
    /// The game ended.
    Ended(EndKind),
}

/// The central game state.
#[derive(Debug)]
pub struct Game {
    /// Bounds of the game's plane.
    bounds: Rect<Coord>,
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
    /// Game state, initially "running".
    state: State,
}

impl Game {
    /// Initializes the game state, given terminal info.
    pub async fn new<'terminal>(
        screen: &mut Screen<'terminal>,
    ) -> Result<Self, Error> {
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
            state: State::Running,
        })
    }

    /// Runs the game, given the initial state, terminal handle, and interval
    /// between ticks.
    pub async fn run(
        mut self,
        terminal: &mut Terminal,
        tick: Duration,
    ) -> Result<EndKind, Error> {
        // Interval between ticks.
        let mut interval = time::interval(tick);
        loop {
            match self.state {
                State::Running => {
                    // Immediately acquires a terminal locked session.
                    let mut session = terminal.lock_now().await?;
                    // Let's interact with the event, change state, and see if
                    // it ended.
                    self.tick(session.event());
                    // But before, we will render what we have.
                    self.render(&mut session.screen());
                },

                State::BadScreen => {
                    let mut session = terminal.lock_now().await?;
                    // If there is an event, it potentially resized the screen
                    // to a valid size.
                    if let Some(event) = session.event() {
                        self.restricted_event(event);
                    }
                },

                // Game ended so we can stop the loop.
                State::Ended(end) => break Ok(end),
            }

            interval.tick().await;
        }
    }

    /// Computes the plane bounds from the screen size.
    fn make_bounds(screen_size: Vec2) -> Rect<Coord> {
        Rect::from_range_incl(
            Vec2 { x: 1, y: 2 },
            Vec2 { x: screen_size.x - 2, y: screen_size.y - 2 },
        )
    }

    /// Performs the game's logical operations inside a game tick.
    fn tick(&mut self, event: Option<Event>) {
        // Handles the event.
        if let Some(curr_event) = event {
            self.full_event(curr_event);
        }

        // Moves the snake.
        self.move_snake();
    }

    /// Handles an event, reacting to the full list of used events, be it
    /// resizing or key pressing.
    fn full_event(&mut self, event: Event) {
        match event {
            // ESC was pressed. Cancels the game.
            Event::Key(KeyEvent { main_key: Key::Esc, .. }) => {
                self.state = State::Ended(EndKind::Cancel);
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
                self.resize(resize);
            },

            // Other keys, ignores.
            Event::Key(_) => (),
        }
    }

    /// Handles an event, but only to a restricted set of used events: resizing
    /// or ESC.
    fn restricted_event(&mut self, event: Event) {
        match event {
            // ESC was pressed. Cancels the game.
            Event::Key(KeyEvent { main_key: Key::Esc, .. }) => {
                self.state = State::Ended(EndKind::Cancel);
            },

            // Screen was resized. Propagates the new size.
            Event::Resize(event) => {
                if event.size.is_some() {
                    // If valid screen size, then we can keep the game running.
                    self.state = State::Running;
                }
            },

            // Other keys, ignores.
            Event::Key(_) => (),
        }
    }

    /// Moves the snake in the current direction.
    fn move_snake(&mut self) {
        match self.snake.mov(self.bounds, &self.food) {
            // Handles the case where the food was eaten.
            Some(eaten) => {
                if self.snake.length() >= self.win_size() {
                    self.state = State::Ended(EndKind::Win);
                } else if eaten {
                    // Handles the case where the food was eaten: it needs to be
                    // regenerated.
                    self.food.regenerate(&self.snake, self.bounds);
                }

                if self.snake.head_intersects() {
                    // If the head intersects the body, game over.
                    self.state = State::Ended(EndKind::Loss)
                }
            },

            // Out of bounds. End of the game.
            None => self.state = State::Ended(EndKind::Loss),
        }
    }

    /// Renders all game data into the screen.
    fn render(&self, screen: &mut Screen) {
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
    fn render_borders(&self, screen: &mut Screen) {
        // Top border.
        for x in self.bounds.start.x .. self.bounds.end().x {
            screen.set(
                Vec2 { x, y: self.bounds.start.y - 1 },
                self.horizontal_tile.clone(),
            );
        }
        // Down border.
        for x in self.bounds.start.x .. self.bounds.end().x {
            screen.set(
                Vec2 { x, y: self.bounds.end().y },
                self.horizontal_tile.clone(),
            );
        }

        // Left border.
        for y in self.bounds.start.y .. self.bounds.end().y + 1 {
            screen.set(
                Vec2 { y, x: self.bounds.start.x - 1 },
                self.vertical_tile.clone(),
            );
        }
        // Right border.
        for y in self.bounds.start.y .. self.bounds.end().y + 1 {
            screen.set(
                Vec2 { y, x: self.bounds.end().x },
                self.vertical_tile.clone(),
            );
        }

        // Corners.
        screen.set(
            Vec2 { x: self.bounds.start.x - 1, y: self.bounds.start.y - 1 },
            self.corner_tile.clone(),
        );
        screen.set(
            Vec2 { x: self.bounds.start.x - 1, y: self.bounds.end().y },
            self.corner_tile.clone(),
        );
        screen.set(
            Vec2 { x: self.bounds.end().x, y: self.bounds.start.y - 1 },
            self.corner_tile.clone(),
        );
        screen.set(
            Vec2 { x: self.bounds.end().x, y: self.bounds.end().y },
            self.corner_tile.clone(),
        );
    }

    /// Renders the message above the borders.
    fn render_message(&self, screen: &mut Screen) {
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
    fn resize(&mut self, event: ResizeEvent) {
        if let Some(size) = event.size {
            // New bounds.
            self.bounds = Self::make_bounds(size);

            if self.snake.saturate_at_bounds(self.bounds) {
                // We will consider that the game is lost if snake gets outside
                // of the plane when resizing.
                self.state = State::Ended(EndKind::Loss);
            }
            if !self.food.in_bounds(self.bounds) {
                // If the food is outside of the plane, regenerates.
                self.food.regenerate(&self.snake, self.bounds);
            }
        } else {
            // Screen size is not valid? Save this as a bad screen state.
            self.state = State::BadScreen;
        }
    }

    /// Computes the size of the snake when the player wins.
    fn win_size(&self) -> usize {
        let x = usize::from(self.bounds.size.x);
        let y = usize::from(self.bounds.size.y);

        // Mean(x,y)/2
        (x + y) / 4
    }
}
