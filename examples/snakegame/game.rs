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

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum EndKind {
    Win,
    Loss,
    Cancel,
}

#[derive(Debug)]
pub struct Game {
    bounds: Bounds,
    food: Food,
    snake: Snake,
    vertical_tile: Tile,
    horizontal_tile: Tile,
    corner_tile: Tile,
    message: TermString,
}

impl Game {
    pub async fn new(terminal: &Terminal) -> Result<Self, Error> {
        let screen = terminal.screen.lock().await?;

        let grapheme = TermGrapheme::new_lossy("O");
        let colors = Color2 {
            foreground: BasicColor::Black.into(),
            background: BasicColor::White.into(),
        };
        let body_tile = Tile { grapheme, colors };

        let grapheme = TermGrapheme::new_lossy("@");
        let colors = Color2 {
            foreground: BasicColor::Black.into(),
            background: BasicColor::White.into(),
        };
        let head_tile = Tile { grapheme, colors };

        let grapheme = TermGrapheme::new_lossy("ɔ́");
        let colors = Color2 {
            foreground: BasicColor::Black.into(),
            background: BasicColor::LightRed.into(),
        };
        let food_tile = Tile { grapheme, colors };

        let grapheme = TermGrapheme::new_lossy("|");
        let colors = Color2 {
            foreground: BasicColor::Black.into(),
            background: BasicColor::LightGreen.into(),
        };
        let vertical_tile = Tile { grapheme, colors };

        let grapheme = TermGrapheme::new_lossy("—");
        let colors = Color2 {
            foreground: BasicColor::Black.into(),
            background: BasicColor::LightGreen.into(),
        };
        let horizontal_tile = Tile { grapheme, colors };

        let grapheme = TermGrapheme::new_lossy("+");
        let colors = Color2 {
            foreground: BasicColor::Black.into(),
            background: BasicColor::LightGreen.into(),
        };
        let corner_tile = Tile { grapheme, colors };

        let message = tstring!["ESC to exit, arrows to move"];

        let bounds = Self::make_bounds(screen.size());

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

    pub async fn run(
        mut self,
        terminal: &mut Terminal,
        tick: Duration,
    ) -> Result<EndKind, Error> {
        let mut interval = time::interval(tick);
        self.render(terminal).await?;

        loop {
            let event = terminal.events.check()?;
            let maybe_end = self.tick(event);
            self.render(terminal).await?;

            if let Some(end) = maybe_end {
                break Ok(end);
            }

            interval.tick().await;
        }
    }

    fn make_bounds(screen_size: Coord2) -> Bounds {
        Bounds {
            min: Coord2 { x: 1, y: 2 },
            max: Coord2 { x: screen_size.x - 2, y: screen_size.y - 2 },
        }
    }

    fn tick(&mut self, event: Option<Event>) -> Option<EndKind> {
        match event {
            Some(Event::Key(KeyEvent { main_key: Key::Esc, .. })) => {
                return Some(EndKind::Cancel);
            },

            Some(Event::Key(KeyEvent {
                main_key: Key::Up,
                shift: false,
                ctrl: false,
                alt: false,
            })) => {
                self.snake.change_direction(Direction::Up);
            },

            Some(Event::Key(KeyEvent {
                main_key: Key::Down,
                shift: false,
                ctrl: false,
                alt: false,
            })) => {
                self.snake.change_direction(Direction::Down);
            },

            Some(Event::Key(KeyEvent {
                main_key: Key::Left,
                shift: false,
                ctrl: false,
                alt: false,
            })) => {
                self.snake.change_direction(Direction::Left);
            },

            Some(Event::Key(KeyEvent {
                main_key: Key::Right,
                shift: false,
                ctrl: false,
                alt: false,
            })) => {
                self.snake.change_direction(Direction::Right);
            },

            Some(Event::Resize(resize)) => {
                if let Some(end) = self.resize(resize) {
                    return Some(end);
                }
            },

            _ => (),
        }

        match self.snake.mov(self.bounds, &self.food) {
            Some(true) => {
                if self.snake.length() >= self.win_size() {
                    return Some(EndKind::Win);
                }

                self.food.regenerate(&self.snake, self.bounds);
            },

            Some(false) => (),

            None => return Some(EndKind::Loss),
        }

        if self.snake.head_intersects() {
            return Some(EndKind::Loss);
        }

        None
    }

    pub async fn render(&self, terminal: &Terminal) -> Result<(), Error> {
        let mut screen = terminal.screen.lock().await?;
        screen.clear(BasicColor::Black.into());

        self.snake.render(&mut screen);
        self.food.render(&mut screen);
        self.render_borders(&mut screen);
        self.render_message(&mut screen);

        Ok(())
    }

    fn render_borders<'screen>(&self, screen: &mut LockedScreen<'screen>) {
        for x in self.bounds.min.x .. self.bounds.max.x + 1 {
            screen.set(
                Coord2 { x, y: self.bounds.min.y - 1 },
                self.horizontal_tile.clone(),
            );
        }
        for x in self.bounds.min.x .. self.bounds.max.x + 1 {
            screen.set(
                Coord2 { x, y: self.bounds.max.y + 1 },
                self.horizontal_tile.clone(),
            );
        }

        for y in self.bounds.min.y .. self.bounds.max.y + 1 {
            screen.set(
                Coord2 { y, x: self.bounds.min.x - 1 },
                self.vertical_tile.clone(),
            );
        }
        for y in self.bounds.min.y .. self.bounds.max.y + 1 {
            screen.set(
                Coord2 { y, x: self.bounds.max.x + 1 },
                self.vertical_tile.clone(),
            );
        }

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

    fn render_message<'screen>(&self, screen: &mut LockedScreen<'screen>) {
        let colors = Color2 {
            foreground: BasicColor::White.into(),
            background: BasicColor::Black.into(),
        };
        let style = Style::with_colors(colors).align(1, 2);
        screen.styled_text(&self.message, style);
    }

    fn resize(&mut self, event: ResizeEvent) -> Option<EndKind> {
        self.bounds = Self::make_bounds(event.size);
        if !self.snake.in_bounds(self.bounds) {
            return Some(EndKind::Loss);
        }
        if !self.food.in_bounds(self.bounds) {
            self.food.regenerate(&self.snake, self.bounds);
        }
        None
    }

    fn win_size(&self) -> usize {
        let x = usize::from(self.bounds.max.x + 1 - self.bounds.min.x);
        let y = usize::from(self.bounds.max.y + 1 - self.bounds.min.y);

        (x + y) / 2
    }
}
