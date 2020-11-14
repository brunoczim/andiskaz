//! This crate exports a terminal terminal and its utilites.

mod error;
mod raw_io;

use crate::{
    coord,
    coord::Coord2,
    error::Error,
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

        let mut barrier = Arc::new(Barrier::new(3));

        let main_fut = {
            let terminal = terminal.clone();
            let barrier = barrier.clone();
            tokio::spawn(async move {
                let screen = terminal.screen.clone();
                let _guard = screen.conn_guard();
                barrier.wait().await;
                start(terminal).await
            })
        };

        let listener_fut = {
            let interval = self.event_interval;
            let screen = terminal.screen.clone();
            tokio::spawn(async move {
                barrier.wait().await;
                event_listener(interval, sender, &screen, &mut None).await
            })
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

    /// Finishes the builder and produces a terminal handle.
    async fn finish(
        self,
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
        let events = EventListener { recv: event_recv };
        Ok(Terminal { screen, events })
    }
}

/// Panic used when an auxiliary task did not fail but finished before main.
#[inline(never)]
#[cold]
fn aux_must_fail() -> ! {
    panic!("Auxiliary task should not finish before main task unless it failed")
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
        F: FnOnce(&mut Terminal) -> A + Send + 'static,
        A: Future<Output = T> + Send + 'static,
        T: Send + 'static,
    {
        Builder::new().run(start).await
    }
}
