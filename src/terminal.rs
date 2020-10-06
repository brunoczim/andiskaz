//! This crate exports a terminal terminal and its utilites.

mod error;
mod raw_io;
mod screen;

pub use self::{
    error::{ListenerFailed, TermError},
    screen::{Screen, Tile},
};

use self::{
    raw_io::{restore_screen, save_screen, write_and_flush},
    screen::ScreenBuffer,
};
use crate::{
    coord,
    coord::{Coord, Coord2},
    input::{key_from_crossterm, Event, KeyEvent, ResizeEvent},
};
use crossterm::event::Event as CrosstermEvent;
use std::{
    fmt::Write,
    future::Future,
    sync::{
        atomic::{AtomicBool, AtomicU32, Ordering::*},
        Arc,
    },
    time::Duration,
};
use tokio::{
    io,
    sync::{watch, Mutex, MutexGuard},
    task,
    time,
};

/// A terminal configuration builder.
#[derive(Debug, Clone)]
pub struct Builder {
    /// Given minimum screen size.
    min_screen: Coord2,
    /// Given time that the screen is updated.
    frame_time: Duration,
}

impl Default for Builder {
    fn default() -> Self {
        Self::new()
    }
}

impl Builder {
    /// Initializes this configuration builder.
    pub fn new() -> Self {
        Self {
            min_screen: Coord2 { x: 80, y: 25 },
            frame_time: Duration::from_millis(200),
        }
    }

    /// Builderures the minimum screen size for the application.
    pub fn min_screen(self, min_screen: Coord2) -> Self {
        Self { min_screen, ..self }
    }

    /// Builderures the rate that the screen is updated.
    pub fn frame_time(self, frame_time: Duration) -> Self {
        Self { frame_time, ..self }
    }

    /// Starts the application and gives it a terminal to the terminal. When the
    /// given start function finishes, the application's execution stops as
    /// well.
    pub async fn run<F, A, T>(self, start: F) -> Result<T, TermError>
    where
        F: FnOnce(Terminal) -> A + Send + 'static,
        A: Future<Output = T> + Send + 'static,
        T: Send + 'static,
    {
        let dummy = Event::Resize(ResizeEvent { size: Coord2 { x: 0, y: 0 } });
        let (sender, mut receiver) = watch::channel(dummy);
        receiver.recv().await;

        let terminal = self.finish(receiver).await?;
        terminal.setup().await?;

        let main_fut = {
            let terminal = terminal.clone();
            tokio::spawn(async move { start(terminal).await })
        };

        let listener_fut = {
            let terminal = terminal.clone();
            tokio::spawn(async move { terminal.event_listener(sender).await })
        };

        let renderer_fut = {
            let terminal = terminal.clone();
            tokio::spawn(async move { terminal.renderer().await })
        };

        let result = tokio::select! {
            result = main_fut => result,
            result = listener_fut => result.map(|_| aux_must_fail()),
            result = renderer_fut => result.map(|_| aux_must_fail()),
        };

        let _ = terminal.cleanup().await;
        Ok(result?)
    }

    async fn finish(
        self,
        event_chan: watch::Receiver<Event>,
    ) -> Result<Terminal, TermError> {
        let res = task::block_in_place(|| {
            crossterm::terminal::enable_raw_mode()?;
            crossterm::terminal::size()
        });
        let (width, height) = res.map_err(TermError::from_crossterm)?;
        let screen_size = Coord2 {
            y: coord::from_crossterm(height),
            x: coord::from_crossterm(width),
        };
        let size_bits = AtomicU32::new(width as u32 | (height as u32) << 16);
        let shared = Arc::new(Shared {
            cleanedup: AtomicBool::new(false),
            min_screen: self.min_screen,
            event_chan: Mutex::new(event_chan),
            screen_size: size_bits,
            stdout: Mutex::new(io::stdout()),
            frame_time: self.frame_time,
            screen_buffer: Mutex::new(ScreenBuffer::blank(screen_size)),
        });
        Ok(Terminal { shared })
    }
}

#[inline(never)]
#[cold]
fn aux_must_fail() -> ! {
    panic!("Auxiliary task should not finish before main task unless it failed")
}

/// A terminal to the terminal. It uses atomic reference counting.
#[derive(Debug, Clone)]
pub struct Terminal {
    shared: Arc<Shared>,
}

impl Terminal {
    /// Returns current screen size.
    pub fn screen_size(&self) -> Coord2 {
        let bits = self.shared.screen_size.load(Acquire);
        Coord2 { x: bits as Coord, y: (bits >> 16) as Coord }
    }

    /// Returns the mininum screen size.
    pub fn min_screen(&self) -> Coord2 {
        self.shared.min_screen
    }

    /// Listens for an event to happen. Waits until an event is available.
    pub async fn listen_event(&self) -> Result<Event, ListenerFailed> {
        self.shared.event_chan.lock().await.recv().await.ok_or(ListenerFailed)
    }

    /// Sets tile contents at a given position.
    pub async fn lock_screen<'terminal>(&'terminal self) -> Screen<'terminal> {
        Screen::new(self, self.shared.screen_buffer.lock().await)
    }

    async fn event_listener(
        &self,
        sender: watch::Sender<Event>,
    ) -> Result<(), TermError> {
        let mut stdout = None;
        self.check_screen_size(self.screen_size(), &mut stdout).await?;

        while !self.shared.cleanedup.load(Acquire) {
            let evt = task::block_in_place(|| {
                match crossterm::event::poll(Duration::from_millis(10))? {
                    true => crossterm::event::read().map(Some),
                    false => Ok(None),
                }
            });

            match evt.map_err(TermError::from_crossterm)? {
                Some(CrosstermEvent::Key(key)) => {
                    let maybe_key = key_from_crossterm(key.code)
                        .filter(|_| stdout.is_none());
                    if let Some(main_key) = maybe_key {
                        use crossterm::event::KeyModifiers as Mod;

                        let evt = KeyEvent {
                            main_key,
                            ctrl: key.modifiers.intersects(Mod::CONTROL),
                            alt: key.modifiers.intersects(Mod::ALT),
                            shift: key.modifiers.intersects(Mod::SHIFT),
                        };

                        let _ = sender.broadcast(Event::Key(evt));
                    }
                },

                Some(CrosstermEvent::Resize(width, height)) => {
                    let size = Coord2 { x: width, y: height };
                    self.check_screen_size(size, &mut stdout).await?;
                    if stdout.is_none() {
                        let mut screen = self.lock_screen().await;
                        screen.resize(size).await?;

                        let evt = ResizeEvent { size };
                        let _ = sender.broadcast(Event::Resize(evt));
                    }
                },

                _ => (),
            }
        }

        Ok(())
    }

    async fn renderer(&self) -> Result<(), TermError> {
        let mut interval = time::interval(self.shared.frame_time);
        let mut buf = String::new();
        while !self.shared.cleanedup.load(Acquire) {
            interval.tick().await;
            self.lock_screen().await.render(&mut buf).await?;
        }

        Ok(())
    }

    async fn check_screen_size<'guard>(
        &'guard self,
        size: Coord2,
        guard: &mut Option<MutexGuard<'guard, io::Stdout>>,
    ) -> io::Result<()> {
        if size.x < self.shared.min_screen.x
            || size.y < self.shared.min_screen.y
        {
            if guard.is_none() {
                let mut stdout = self.shared.stdout.lock().await;
                let buf = format!(
                    "{}{}RESIZE {}x{}",
                    crossterm::terminal::Clear(
                        crossterm::terminal::ClearType::All
                    ),
                    crossterm::cursor::MoveTo(0, 0),
                    self.shared.min_screen.x,
                    self.shared.min_screen.y,
                );
                write_and_flush(buf.as_bytes(), &mut stdout).await?;
                *guard = Some(stdout);
            }
        } else {
            *guard = None;
        }
        Ok(())
    }

    async fn setup(&self) -> Result<(), TermError> {
        let mut buf = String::new();
        save_screen(&mut buf)?;
        write!(
            buf,
            "{}{}{}{}",
            crossterm::style::SetBackgroundColor(
                crossterm::style::Color::Black
            ),
            crossterm::style::SetForegroundColor(
                crossterm::style::Color::White
            ),
            crossterm::cursor::Hide,
            crossterm::terminal::Clear(crossterm::terminal::ClearType::All),
        )?;
        let mut guard = self.shared.stdout.lock().await;
        write_and_flush(buf.as_bytes(), &mut guard).await?;
        Ok(())
    }

    async fn cleanup(&self) -> Result<(), TermError> {
        task::block_in_place(|| crossterm::terminal::disable_raw_mode())
            .map_err(TermError::from_crossterm)?;
        let mut buf = String::new();
        write!(buf, "{}", crossterm::cursor::Show)?;
        restore_screen(&mut buf)?;
        let mut guard = self.shared.stdout.lock().await;
        write_and_flush(buf.as_bytes(), &mut guard).await?;
        self.shared.cleanedup.store(true, Release);
        Ok(())
    }
}

#[derive(Debug)]
struct Shared {
    cleanedup: AtomicBool,
    min_screen: Coord2,
    event_chan: Mutex<watch::Receiver<Event>>,
    stdout: Mutex<io::Stdout>,
    screen_size: AtomicU32,
    screen_buffer: Mutex<ScreenBuffer>,
    frame_time: Duration,
}

impl Drop for Shared {
    fn drop(&mut self) {
        if !self.cleanedup.load(Relaxed) {
            let _ = crossterm::terminal::disable_raw_mode();
            let mut buf = String::new();
            write!(buf, "{}", crossterm::cursor::Show)
                .ok()
                .and_then(|_| restore_screen(&mut buf).ok())
                .map(|_| println!("{}", buf));
        }
    }
}
