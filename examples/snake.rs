use andiskaz::{
    color::{BasicColor, Color2},
    coord::Coord2,
    emergency_restore,
    error::Error as AndiskazError,
    event::{Event, Key, KeyEvent, ResizeEvent},
    screen::{LockedScreen, Tile},
    string::{TermGrapheme, TermString},
    style::Style,
    terminal::Terminal,
    tstring,
};
use backtrace::Backtrace;
use rand::Rng;
use std::{collections::VecDeque, panic, process::exit, time::Duration};
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
    let result = match Game::new(&mut terminal).await {
        Ok(game) => game.render(&mut terminal).await.map(|_| game),
        Err(exit) => Err(exit),
    };

    let mut interval = time::interval(Duration::from_millis(50));

    let error = match result {
        Ok(mut game) => loop {
            if let Err(error) = game.tick(&mut terminal).await {
                break error;
            }
            interval.tick().await;
        },
        Err(error) => error,
    };

    match error.exit_is_ok()? {
        Exit::Esc => (),

        Exit::Won => {
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

        Exit::Lost => {
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

#[derive(Debug)]
enum Exit {
    Esc,
    Lost,
    Won,
}

#[derive(Debug)]
enum SnakeError {
    Exit(Exit),
    Andiskaz(AndiskazError),
}

impl SnakeError {
    fn exit_is_ok(self) -> Result<Exit, AndiskazError> {
        match self {
            SnakeError::Exit(exit) => Ok(exit),
            SnakeError::Andiskaz(error) => Err(error),
        }
    }
}

impl From<Exit> for SnakeError {
    fn from(exit: Exit) -> Self {
        SnakeError::Exit(exit)
    }
}

impl<E> From<E> for SnakeError
where
    E: Into<AndiskazError>,
{
    fn from(error: E) -> Self {
        SnakeError::Andiskaz(error.into())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
struct Bounds {
    min: Coord2,
    max: Coord2,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
enum Direction {
    Up,
    Down,
    Left,
    Right,
}

impl Direction {
    fn move_coords(
        self,
        coords: Coord2,
        bounds: Bounds,
    ) -> Result<Coord2, Exit> {
        match self {
            Direction::Up => {
                if coords.y > bounds.min.y {
                    Ok(Coord2 { y: coords.y - 1, ..coords })
                } else {
                    Err(Exit::Lost)
                }
            },

            Direction::Down => {
                if coords.y < bounds.max.y {
                    Ok(Coord2 { y: coords.y + 1, ..coords })
                } else {
                    Err(Exit::Lost)
                }
            },

            Direction::Left => {
                if coords.x > bounds.min.x {
                    Ok(Coord2 { x: coords.x - 1, ..coords })
                } else {
                    Err(Exit::Lost)
                }
            },

            Direction::Right => {
                if coords.x < bounds.max.x {
                    Ok(Coord2 { x: coords.x + 1, ..coords })
                } else {
                    Err(Exit::Lost)
                }
            },
        }
    }
}

#[derive(Debug)]
struct Snake {
    direction: Direction,
    body_sprite: TermGrapheme,
    head_sprite: TermGrapheme,
    segments: VecDeque<Coord2>,
}

impl Snake {
    fn new(
        body_sprite: TermGrapheme,
        head_sprite: TermGrapheme,
        bounds: Bounds,
    ) -> Self {
        let length = 3;
        let distance = 10;

        let mut rng = rand::thread_rng();
        let head = Coord2 {
            x: rng.gen_range(bounds.min.x, bounds.max.x + 1),
            y: rng.gen_range(
                bounds.min.y + distance,
                bounds.max.y - (distance + length - 1),
            ),
        };

        let mut segments = VecDeque::new();
        for i in 0 .. length {
            segments.push_back(Coord2 { y: head.y + i, ..head });
        }

        Self { direction: Direction::Up, segments, body_sprite, head_sprite }
    }

    fn mov(&mut self, bounds: Bounds, food: &Food) -> Result<bool, Exit> {
        let new_head = self.direction.move_coords(self.segments[0], bounds)?;

        if new_head != food.pos {
            self.segments.pop_back();
        }
        self.segments.push_front(new_head);

        Ok(new_head == food.pos)
    }

    fn change_direction(&mut self, direction: Direction) {
        self.direction = direction;
    }

    fn head_intersects(&self) -> Result<(), Exit> {
        let head = self.segments[0];

        for segment in self.segments.iter().skip(1) {
            if *segment == head {
                Err(Exit::Lost)?;
            }
        }

        Ok(())
    }

    fn in_bounds(&self, bounds: Bounds) -> Result<(), Exit> {
        let in_bounds = self.segments.iter().all(|point| {
            bounds.min.x <= point.x
                && point.x <= bounds.max.x
                && bounds.min.y <= point.y
                && point.y <= bounds.max.y
        });

        if in_bounds {
            Ok(())
        } else {
            Err(Exit::Lost)
        }
    }

    fn render(&self, screen: &mut LockedScreen) {
        let colors = Color2 {
            foreground: BasicColor::Black.into(),
            background: BasicColor::White.into(),
        };

        let body_tile = Tile { grapheme: self.body_sprite.clone(), colors };

        for segment in &self.segments {
            screen.set(*segment, body_tile.clone());
        }

        let head_tile = Tile { grapheme: self.head_sprite.clone(), colors };

        screen.set(self.segments[0], head_tile.clone());
    }
}

#[derive(Debug)]
struct Food {
    pos: Coord2,
    sprite: TermGrapheme,
}

impl Food {
    fn new(sprite: TermGrapheme, snake: &Snake, bounds: Bounds) -> Self {
        Self { pos: Self::gen_pos(snake, bounds), sprite }
    }

    fn regenerate(&mut self, snake: &Snake, bounds: Bounds) {
        self.pos = Self::gen_pos(snake, bounds);
    }

    fn in_bounds(&self, bounds: Bounds) -> bool {
        bounds.min.x <= self.pos.x
            && self.pos.x <= bounds.max.x
            && bounds.min.y <= self.pos.y
            && self.pos.y <= bounds.max.y
    }

    fn gen_pos(snake: &Snake, bounds: Bounds) -> Coord2 {
        loop {
            let mut rng = rand::thread_rng();
            let point = Coord2 {
                x: rng.gen_range(bounds.min.x, bounds.max.x + 1),
                y: rng.gen_range(bounds.min.y, bounds.max.y + 1),
            };

            let valid = snake.segments.iter().all(|segment| *segment != point);

            if valid {
                break point;
            }
        }
    }

    fn render(&self, screen: &mut LockedScreen) {
        let colors = Color2 {
            foreground: BasicColor::Black.into(),
            background: BasicColor::LightRed.into(),
        };

        let tile = Tile { grapheme: self.sprite.clone(), colors };

        screen.set(self.pos, tile);
    }
}

#[derive(Debug)]
struct Game {
    bounds: Bounds,
    food: Food,
    snake: Snake,
    vertical_sprite: TermGrapheme,
    horizontal_sprite: TermGrapheme,
    corner_sprite: TermGrapheme,
    message: TermString,
}

impl Game {
    async fn new(terminal: &Terminal) -> Result<Self, SnakeError> {
        let screen = terminal.screen.lock().await?;

        let body_sprite = TermGrapheme::new_lossy("O");
        let head_sprite = TermGrapheme::new_lossy("@");
        let food_sprite = TermGrapheme::new_lossy("ɔ́");
        let vertical_sprite = TermGrapheme::new_lossy("|");
        let horizontal_sprite = TermGrapheme::new_lossy("—");
        let corner_sprite = TermGrapheme::new_lossy("+");
        let message = tstring!["ESC to exit, arrows to move"];

        let bounds = Self::make_bounds(screen.size());

        let snake = Snake::new(body_sprite, head_sprite, bounds);
        let food = Food::new(food_sprite, &snake, bounds);

        Ok(Self {
            bounds,
            snake,
            food,
            corner_sprite,
            vertical_sprite,
            horizontal_sprite,
            message,
        })
    }

    fn make_bounds(screen_size: Coord2) -> Bounds {
        Bounds {
            min: Coord2 { x: 1, y: 2 },
            max: Coord2 { x: screen_size.x - 2, y: screen_size.y - 2 },
        }
    }

    async fn tick(
        &mut self,
        terminal: &mut Terminal,
    ) -> Result<(), SnakeError> {
        match terminal.events.check()? {
            Some(Event::Key(KeyEvent { main_key: Key::Esc, .. })) => {
                Err(Exit::Esc)?
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

            Some(Event::Resize(resize)) => self.resize(resize)?,

            _ => (),
        }

        if self.snake.mov(self.bounds, &self.food)? {
            if self.snake.segments.len() >= self.won_size() {
                Err(Exit::Won)?;
            }
            self.food.regenerate(&self.snake, self.bounds);
        }

        self.snake.head_intersects()?;
        self.render(terminal).await?;

        Ok(())
    }

    async fn render(&self, terminal: &Terminal) -> Result<(), SnakeError> {
        let mut screen = terminal.screen.lock().await?;
        screen.clear(BasicColor::Black.into());

        self.snake.render(&mut screen);
        self.food.render(&mut screen);
        self.render_borders(&mut screen);
        self.render_message(&mut screen);

        Ok(())
    }

    fn render_borders<'screen>(&self, screen: &mut LockedScreen<'screen>) {
        let colors = Color2 {
            foreground: BasicColor::Black.into(),
            background: BasicColor::LightGreen.into(),
        };
        let horizontal_tile =
            Tile { grapheme: self.horizontal_sprite.clone(), colors };
        let vertical_tile =
            Tile { grapheme: self.vertical_sprite.clone(), colors };
        let corner_tile = Tile { grapheme: self.corner_sprite.clone(), colors };

        for x in self.bounds.min.x .. self.bounds.max.x + 1 {
            screen.set(
                Coord2 { x, y: self.bounds.min.y - 1 },
                horizontal_tile.clone(),
            );
        }
        for x in self.bounds.min.x .. self.bounds.max.x + 1 {
            screen.set(
                Coord2 { x, y: self.bounds.max.y + 1 },
                horizontal_tile.clone(),
            );
        }

        for y in self.bounds.min.y .. self.bounds.max.y + 1 {
            screen.set(
                Coord2 { y, x: self.bounds.min.x - 1 },
                vertical_tile.clone(),
            );
        }
        for y in self.bounds.min.y .. self.bounds.max.y + 1 {
            screen.set(
                Coord2 { y, x: self.bounds.max.x + 1 },
                vertical_tile.clone(),
            );
        }

        screen.set(
            Coord2 { x: self.bounds.min.x - 1, y: self.bounds.min.y - 1 },
            corner_tile.clone(),
        );
        screen.set(
            Coord2 { x: self.bounds.min.x - 1, y: self.bounds.max.y + 1 },
            corner_tile.clone(),
        );
        screen.set(
            Coord2 { x: self.bounds.max.x + 1, y: self.bounds.min.y - 1 },
            corner_tile.clone(),
        );
        screen.set(
            Coord2 { x: self.bounds.max.x + 1, y: self.bounds.max.y + 1 },
            corner_tile.clone(),
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

    fn resize(&mut self, event: ResizeEvent) -> Result<(), Exit> {
        self.bounds = Self::make_bounds(event.size);
        self.snake.in_bounds(self.bounds)?;
        if !self.food.in_bounds(self.bounds) {
            self.food.regenerate(&self.snake, self.bounds)
        }
        Ok(())
    }

    fn won_size(&self) -> usize {
        let x = usize::from(self.bounds.max.x + 1 - self.bounds.min.x);
        let y = usize::from(self.bounds.max.y + 1 - self.bounds.min.y);

        (x + y) / 2
    }
}
