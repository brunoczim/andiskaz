//! This crate exports a terminal terminal and its utilites.

use crate::{
    coord,
    coord::Coord2,
    error::{Error, ErrorKind},
    event::{event_listener, Event, EventListener, ResizeEvent},
    screen::{renderer, Screen},
};
use std::{future::Future, sync::Arc, time::Duration};
use tokio::{
    sync::{watch, Barrier},
    task,
};

/// A terminal configuration builder.
#[derive(Debug, Clone)]
pub struct Builder {
    /// Given minimum screen size.
    min_screen: Coord2,
    /// Given time that the screen is updated.
    frame_time: Duration,
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

    /// Builds the rate that the screen is updated.
    pub fn event_interval(self, event_interval: Duration) -> Self {
        Self { event_interval, ..self }
    }

    /// Starts the application and gives it a handle to the terminal. When the
    /// given start function finishes, the application's execution stops as
    /// well.
    pub async fn run<F, A, T>(self, start: F) -> Result<T, Error>
    where
        F: FnOnce(Terminal) -> A + Send + 'static,
        A: Future<Output = T> + Send + 'static,
        T: Send + 'static,
    {
        let dummy = Event::Resize(ResizeEvent { size: Coord2 { x: 0, y: 0 } });
        let (sender, receiver) = watch::channel(dummy);

        let terminal = self.finish(receiver).await?;
        terminal.screen.setup().await?;

        let screen = terminal.screen.clone();
        let barrier = Arc::new(Barrier::new(3));

        let main_fut = {
            let barrier = barrier.clone();
            tokio::spawn(async move {
                let screen = terminal.screen.clone();
                let _guard = screen.conn_guard();
                barrier.wait().await;
                start(terminal).await
            })
        };

        let events_fut = {
            let interval = self.event_interval;
            let screen = screen.clone();
            let barrier = barrier.clone();
            tokio::spawn(Self::events_task(barrier, interval, screen, sender))
        };

        let renderer_fut = {
            let screen = screen.clone();
            tokio::spawn(async move {
                let _guard = screen.conn_guard();
                barrier.wait().await;
                renderer(&screen).await
            })
        };

        let (main_ret, events_ret, renderer_ret) =
            tokio::join!(main_fut, events_fut, renderer_fut);

        let _ = screen.cleanup().await;

        if let Err(error) = events_ret? {
            match error.kind() {
                ErrorKind::RendererOff(_) => (),
                _ => Err(error)?,
            }
        }

        if let Err(error) = renderer_ret? {
            match error.kind() {
                ErrorKind::RendererOff(_) => (),
                _ => Err(error)?,
            }
        }

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
        let events = EventListener::new(event_recv);
        Ok(Terminal { screen, events })
    }

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
}

#[derive(Debug, Clone)]
pub struct Terminal {
    pub screen: Screen,
    pub events: EventListener,
}

impl Terminal {
    /// Starts the application and gives it a handle to the terminal. When the
    /// given start function finishes, the application's execution stops as
    /// well.
    ///
    /// This function uses the default configuration. See [`Builder`] for
    /// terminal settings.
    pub async fn run<F, A, T>(start: F) -> Result<T, Error>
    where
        F: FnOnce(Terminal) -> A + Send + 'static,
        A: Future<Output = T> + Send + 'static,
        T: Send + 'static,
    {
        Builder::new().run(start).await
    }
}
