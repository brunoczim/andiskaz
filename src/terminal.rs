//! This crate exports a terminal terminal and its utilites.

use crate::{
    coord,
    coord::Coord2,
    error::{AlreadyRunning, Error, ErrorKind},
    event::{event_listener, Event, Events, ResizeEvent},
    screen::{renderer, Screen},
};
use std::{
    future::Future,
    sync::{
        atomic::{AtomicBool, Ordering::*},
        Arc,
    },
    time::Duration,
};
use tokio::{
    sync::{watch, Barrier},
    task,
};

/// State of the terminal guard. true means acquired, false means released.
static GUARD_STATE: AtomicBool = AtomicBool::new(false);

/// A guard of the terminal handle. Only one instance of terminal services is
/// allowed per time, this stucture ensures this.
#[derive(Debug)]
struct Guard;

impl Guard {
    /// Acquires the guard. Returns an error if the guard was already acquired.
    fn acquire() -> Result<Self, AlreadyRunning> {
        if GUARD_STATE.swap(true, Acquire) {
            Err(AlreadyRunning)
        } else {
            Ok(Self)
        }
    }
}

impl Drop for Guard {
    fn drop(&mut self) {
        GUARD_STATE.store(false, Release)
    }
}

/// A terminal configuration builder.
#[derive(Debug, Clone)]
pub struct Builder {
    /// Given minimum screen size.
    min_screen: Coord2,
    /// Given time that the screen is updated.
    frame_time: Duration,
    /// Interval between a failed poll and the next poll.
    event_interval: Duration,
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
            frame_time: Duration::from_millis(20),
            event_interval: Duration::from_millis(20),
        }
    }

    /// Builds the minimum screen size for the application.
    pub fn min_screen(self, min_screen: Coord2) -> Self {
        Self { min_screen, ..self }
    }

    /// Builds the rate that the screen is updated.
    pub fn frame_time(self, frame_time: Duration) -> Self {
        Self { frame_time, ..self }
    }

    /// Interval waited when a poll for an event fails.
    pub fn event_interval(self, event_interval: Duration) -> Self {
        Self { event_interval, ..self }
    }

    /// Starts the application and gives it a handle to the terminal. When the
    /// given start function finishes, the application's execution stops as
    /// well.
    ///
    /// After that `start`'s future returns, terminal services such as screen
    /// handle and events handle are not guaranteed to be available. One would
    /// prefer spawning tasks that use the terminal handle by joining them, and
    /// not detaching.
    ///
    /// Returns an [`AlreadyRunning`] error if there is already an instance of
    /// terminal services executing. In other words, one should not call
    /// this function again if another call did not finish yet, otherwise it
    /// will panic.
    ///
    /// Beware! If the given `start` future returns a `Result`, then `run` will
    /// return a double `Result`!!
    pub async fn run<F, A, T>(self, start: F) -> Result<T, Error>
    where
        F: FnOnce(Terminal) -> A + Send + 'static,
        A: Future<Output = T> + Send + 'static,
        T: Send + 'static,
    {
        // Ensures there are no other terminal sevices executing.
        let _guard = Guard::acquire()?;

        // Events channel.
        let dummy = Event::Resize(ResizeEvent { size: Coord2 { x: 0, y: 0 } });
        let (sender, receiver) = watch::channel(dummy);

        // Initializes terminal structures.
        let terminal = self.finish(receiver).await?;
        let screen = terminal.screen.clone();
        terminal.screen.setup().await?;

        // Synchronization.
        let barrier = Arc::new(Barrier::new(3));

        // Main task future.
        let main_fut = {
            let barrier = barrier.clone();
            tokio::spawn(main_task(barrier, terminal, start))
        };

        // Event listener task future.
        let events_fut = {
            let interval = self.event_interval;
            let screen = screen.clone();
            let barrier = barrier.clone();
            tokio::spawn(events_task(barrier, interval, screen, sender))
        };

        // Renderer task future.
        let renderer_fut = {
            let screen = screen.clone();
            tokio::spawn(renderer_task(barrier, screen))
        };

        let (main_ret, events_ret, renderer_ret) =
            tokio::join!(main_fut, events_fut, renderer_fut);

        // Cleans up screen configurations (such as raw mode).
        let _ = screen.cleanup().await;

        // Matches the error of events task result.
        if let Err(error) = events_ret? {
            match error.kind() {
                ErrorKind::RendererOff(_) => (),
                _ => Err(error)?,
            }
        }

        // Matches the error of renderer task result.
        if let Err(error) = renderer_ret? {
            match error.kind() {
                ErrorKind::RendererOff(_) => (),
                _ => Err(error)?,
            }
        }

        // Finally returns main task return value.
        Ok(main_ret?)
    }

    /// Finishes the builder and produces a terminal handle.
    async fn finish(
        &self,
        event_recv: watch::Receiver<Event>,
    ) -> Result<Terminal, Error> {
        let res = task::block_in_place(|| {
            crossterm::terminal::enable_raw_mode()?;
            crossterm::terminal::size()
        });
        let (width, height) = res.map_err(Error::from_crossterm)?;
        let screen_size = Coord2 {
            y: coord::from_crossterm(height),
            x: coord::from_crossterm(width),
        };
        let screen = Screen::new(screen_size, self.min_screen, self.frame_time);
        let events = Events::new(event_recv);
        Ok(Terminal { screen, events })
    }
}

/// The main task of a terminal application. Barrier must be shared between
/// tasks witht the same given screen or event channel.
async fn main_task<F, A, T>(
    barrier: Arc<Barrier>,
    terminal: Terminal,
    start: F,
) -> T
where
    F: FnOnce(Terminal) -> A + Send + 'static,
    A: Future<Output = T> + Send + 'static,
    T: Send + 'static,
{
    let screen = terminal.screen.clone();
    let _guard = screen.conn_guard();
    barrier.wait().await;
    start(terminal).await
}

/// The task that listens to events. Barrier must be shared between tasks
/// with the same given screen or event channel.
async fn events_task(
    barrier: Arc<Barrier>,
    interval: Duration,
    screen: Screen,
    sender: watch::Sender<Event>,
) -> Result<(), Error> {
    let mut stdout = None;
    {
        let mut locked = screen
            .lock()
            .await
            .expect("renderer can't have already disconnected");
        let size = locked.size();
        let min_size = locked.min_size();
        if size.x < min_size.x || size.y < min_size.y {
            locked.check_resize(min_size, &mut stdout).await?;
            locked.check_resize(size, &mut stdout).await?;
        }
    }
    barrier.wait().await;
    event_listener(interval, sender, &screen, &mut stdout).await
}

/// The task that renders the screen buffer, periodically. Barrier must be
/// shared between tasks with the same given screen.
async fn renderer_task(
    barrier: Arc<Barrier>,
    screen: Screen,
) -> Result<(), Error> {
    let _guard = screen.conn_guard();
    barrier.wait().await;
    renderer(&screen).await
}

/// A handle to the terminal.
#[derive(Debug, Clone)]
pub struct Terminal {
    /// Screen handle. Used to write into the screen.
    pub screen: Screen,
    /// Events handle. Used to check and await for events, such as resizing of
    /// the screen or keys pressed.
    pub events: Events,
}

impl Terminal {
    /// Starts the application and gives it a handle to the terminal. When the
    /// given start function finishes, the application's execution stops as
    /// well.
    ///
    /// After that `start`'s future returns, terminal services such as screen
    /// handle and events handle are not guaranteed to be available. One would
    /// prefer spawning tasks that use the terminal handle by joining them, and
    /// not detaching.
    ///
    /// This function uses the default configuration. See [`Builder`] for
    /// terminal settings.
    ///
    /// Returns an [`AlreadyRunning`] error if there already is an instance of
    /// terminal services executing. In other words, one should not call
    /// this function again if another call did not finish yet, otherwise it
    /// will panic.
    ///
    /// Beware! If the given `start` future returns a `Result`, then `run` will
    /// return a double `Result`!!
    pub async fn run<F, A, T>(start: F) -> Result<T, Error>
    where
        F: FnOnce(Terminal) -> A + Send + 'static,
        A: Future<Output = T> + Send + 'static,
        T: Send + 'static,
    {
        Builder::new().run(start).await
    }
}
