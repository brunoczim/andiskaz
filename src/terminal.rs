//! This crate exports a terminal terminal and its utilites.

use crate::{
    coord,
    coord::Coord2,
    error::{AlreadyRunning, Error, ErrorKind, ServicesOff},
    event,
    event::{Event, Reactor},
    screen::{renderer, Screen, ScreenData},
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
    sync::{Barrier, RwLock, RwLockReadGuard, RwLockWriteGuard},
    task,
};

/// State of the terminal guard. true means acquired, false means released.
static RUN_GUARD_STATE: AtomicBool = AtomicBool::new(false);

/// A guard of the terminal handle. Only one instance of terminal services is
/// allowed per time, this stucture ensures this.
#[derive(Debug)]
struct RunGuard;

impl RunGuard {
    /// Acquires the guard. Returns an error if the guard was already acquired.
    fn acquire() -> Result<Self, AlreadyRunning> {
        if RUN_GUARD_STATE.swap(true, Acquire) {
            Err(AlreadyRunning)
        } else {
            Ok(Self)
        }
    }
}

impl Drop for RunGuard {
    fn drop(&mut self) {
        RUN_GUARD_STATE.store(false, Release)
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
        let _guard = RunGuard::acquire()?;

        // Initializes terminal structures.
        let initial_size = self.initial_size()?;
        let terminal = self.finish(initial_size).await;
        let shared = terminal.shared.clone();
        shared.screen().setup().await?;

        // Synchronization between parties.
        let barrier = Arc::new(Barrier::new(3));

        // Event listener task future.
        let events_fut = {
            let interval = self.event_interval;
            let barrier = barrier.clone();
            let shared = shared.clone();
            tokio::spawn(events_task(barrier, interval, shared, initial_size))
        };

        // Renderer task future.
        let renderer_fut = {
            let barrier = barrier.clone();
            let shared = shared.clone();
            tokio::spawn(renderer_task(barrier, shared))
        };

        // Main task future.
        let main_fut = {
            let barrier = barrier.clone();
            tokio::spawn(main_task(barrier, terminal, start))
        };

        // Joins every task.
        let (main_ret, events_ret, renderer_ret) =
            tokio::join!(main_fut, events_fut, renderer_fut);

        // Cleans up screen configurations (such as raw mode).
        let _ = shared.screen().cleanup().await;

        // Matches the error of events task result.
        if let Err(error) = events_ret? {
            match error.kind() {
                ErrorKind::ServicesOff(_) => (),
                _ => Err(error)?,
            }
        }

        // Matches the error of renderer task result.
        if let Err(error) = renderer_ret? {
            match error.kind() {
                ErrorKind::ServicesOff(_) => (),
                _ => Err(error)?,
            }
        }

        // Finally returns main task return value.
        Ok(main_ret?)
    }

    /// Finds the initial size of the terminal.
    fn initial_size(&self) -> Result<Coord2, Error> {
        let size_res = task::block_in_place(|| {
            crossterm::terminal::enable_raw_mode()?;
            crossterm::terminal::size()
        });
        let (width, height) = size_res.map_err(Error::from_crossterm)?;
        Ok(Coord2 {
            y: coord::from_crossterm(height),
            x: coord::from_crossterm(width),
        })
    }

    /// Finishes the builder and produces a terminal handle.
    async fn finish(&self, screen_size: Coord2) -> Terminal {
        let shared = Arc::new(Shared::new(
            screen_size,
            self.min_screen,
            self.frame_time,
        ));
        Terminal { shared, curr_epoch: 0 }
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
    let cloned = terminal.clone();
    let _guard = cloned.shared.conn_guard();
    barrier.wait().await;
    start(terminal).await
}

/// The task that listens to events. Barrier must be shared between tasks
/// with the same given screen or event channel.
async fn events_task(
    barrier: Arc<Barrier>,
    interval: Duration,
    shared: Arc<Shared>,
    initial_size: Coord2,
) -> Result<(), Error> {
    let mut reactor = Reactor::new(&shared);
    reactor.pre_loop(initial_size).await?;
    barrier.wait().await;
    reactor.react_loop(interval).await
}

/// The task that renders the screen buffer, periodically. Barrier must be
/// shared between tasks with the same given screen.
async fn renderer_task(
    barrier: Arc<Barrier>,
    shared: Arc<Shared>,
) -> Result<(), Error> {
    let _guard = shared.conn_guard();
    barrier.wait().await;
    renderer(&shared).await
}

/// An application's handle to the terminal.
#[derive(Debug, Clone)]
pub struct Terminal {
    /// Shared data between application's terminal handles and services.
    shared: Arc<Shared>,
    /// Current epoch on event channel for this channel.
    curr_epoch: event::Epoch,
}

impl Terminal {
    /// Runs the terminal application with the default settings, i.e. minumum
    /// screen is 80x25, 20ms for frame rendering interval, and 20ms for event
    /// polling from the OS.
    ///
    /// Gives the application a handle to the terminal. When the given start
    /// function finishes, the application's execution stops as well.
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
    pub async fn run<F, A, T>(start: F) -> Result<T, Error>
    where
        F: FnOnce(Terminal) -> A + Send + 'static,
        A: Future<Output = T> + Send + 'static,
        T: Send + 'static,
    {
        Builder::default().run(start).await
    }

    /// Locks the terminal immediately (except it has to wait for the lock to be
    /// available). A locked terminal handle is returned, on which an
    /// application can write to the screen or get the most recent event.
    ///
    /// Screen is locked, event channel is locked.
    pub async fn lock_now<'terminal>(
        &'terminal mut self,
    ) -> Result<TerminalGuard<'terminal>, ServicesOff> {
        let guard = self.shared.app_guard().await?;
        let event = self.shared.events().read(self.curr_epoch);
        let screen = self.shared.screen().lock().await;
        Ok(TerminalGuard {
            screen,
            guard,
            event,
            curr_epoch: &mut self.curr_epoch,
        })
    }

    /// Listens for an event and only finishes when an event arrives. A locked
    /// terminal handle is returned, on which an application can write to the
    /// screen or get the most recent event, which will be present.
    ///
    /// Screen is locked, event channel is locked.
    pub async fn listen<'terminal>(
        &'terminal mut self,
    ) -> Result<TerminalGuard<'terminal>, ServicesOff> {
        self.shared.events.subscribe().await;
        self.lock_now().await
    }

    /// Clears the event channel. After this call, the current event is marked
    /// as read and it will no longer be available.
    pub fn clear_event(&mut self) {
        self.curr_epoch = self.shared.events().epoch();
    }
}

/// A guard on a locked terminal handle.
///
/// Screen is locked, event channel is locked.
#[derive(Debug)]
pub struct TerminalGuard<'terminal> {
    /// Synchronization lock for application (exclusive).
    guard: AppSyncGuard<'terminal>,
    /// The result of reading the event channel (event epoch, event itself).
    event: Option<(event::Epoch, Event)>,
    /// Reference to the current epoch so we can update it when the event is
    /// read.
    curr_epoch: &'terminal mut event::Epoch,
    /// Locked handle to the screen acquired by the application.
    screen: Screen<'terminal>,
}

impl<'terminal> TerminalGuard<'terminal> {
    /// Reads the last event. When this method is called, the event is marked as
    /// read, and in subsequent locks to the terminal handle, it won't be
    /// available. If not called and no other event arrives, this will not
    /// be marked as read and it will be available in the next locking.
    pub fn event(&mut self) -> Option<Event> {
        self.event.map(|(new_epoch, event)| {
            *self.curr_epoch = new_epoch;
            event
        })
    }

    /// Returns a locked handle to the screen.
    pub fn screen(&mut self) -> &mut Screen<'terminal> {
        &mut self.screen
    }
}

/// Shared data between parties of the terminal application (the application
/// itself, the reactor service, the renderer service).
#[derive(Debug)]
pub(crate) struct Shared {
    /// Synchronization between application handles (each one is exclusive) and
    /// services handles (all shared, such as event reactor and screen
    /// renderer).
    sync: RwLock<()>,
    /// Flags whether every party is connected (i.e. application, reactor and
    /// renderer).
    connected: AtomicBool,
    /// Screen managing's data.
    screen: ScreenData,
    /// Events channel data.
    events: event::Channel,
}

impl Shared {
    /// Creates shared data from: current screen size, minimum screen size,
    /// frame interval time .
    pub fn new(
        screen_size: Coord2,
        min_screen: Coord2,
        frame_time: Duration,
    ) -> Self {
        Self {
            sync: RwLock::new(()),
            connected: AtomicBool::new(true),
            screen: ScreenData::new(screen_size, min_screen, frame_time),
            events: event::Channel::default(),
        }
    }

    /// Whether all parties are connected.
    pub fn is_connected(&self) -> bool {
        self.connected.load(Acquire)
    }

    /// Disconnects one party, and therefore, marks the whole shared data as
    /// disconnected (idempotent).
    pub fn disconnect(&self) {
        self.connected.store(false, Release);
        self.events.notify();
        self.screen.notify();
    }

    /// Locks the synchronization between application and services for the
    /// services (shared).
    pub async fn service_guard<'this>(
        &'this self,
    ) -> Result<ServiceSyncGuard<'this>, ServicesOff> {
        let guard = ServiceSyncGuard { inner: self.sync.read().await };
        if self.is_connected() {
            Ok(guard)
        } else {
            Err(ServicesOff)
        }
    }

    /// Locks the synchronization between application and services for the
    /// application (exclusive).
    pub async fn app_guard<'this>(
        &'this self,
    ) -> Result<AppSyncGuard<'this>, ServicesOff> {
        let guard = AppSyncGuard { inner: self.sync.write().await };
        if self.is_connected() {
            Ok(guard)
        } else {
            Err(ServicesOff)
        }
    }

    /// Returns reference to the events' channel used by this application.
    pub fn events(&self) -> &event::Channel {
        &self.events
    }

    /// Returns reference to the screen manager's data used by this application.
    pub fn screen(&self) -> &ScreenData {
        &self.screen
    }

    /// Creates a connection guard. A connection guard disconnects the shared
    /// data when the guard is dropped.
    pub fn conn_guard(&self) -> ConnGuard {
        ConnGuard { shared: self }
    }
}

/// Synchronization guard acquired by a service (shared).
#[derive(Debug)]
pub(crate) struct ServiceSyncGuard<'shared> {
    /// Inner lock guard.
    inner: RwLockReadGuard<'shared, ()>,
}

/// Synchronization guard acquired by an application handle (exclusive).
#[derive(Debug)]
pub(crate) struct AppSyncGuard<'shared> {
    /// Inner lock guard.
    inner: RwLockWriteGuard<'shared, ()>,
}

/// Connection guard for shared data. Disconnects when dropped.
#[derive(Debug)]
pub(crate) struct ConnGuard<'shared> {
    /// Reference to the shared data on which drop will happen.
    shared: &'shared Shared,
}

impl<'shared> Drop for ConnGuard<'shared> {
    fn drop(&mut self) {
        self.shared.disconnect()
    }
}
