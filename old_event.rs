//! This module defines input events of a terminal.

#[cfg(test)]
mod test;

use crate::{
    coord::Coord2,
    error::{Error, EventsOff},
    screen::Screen,
    stdio::LockedStdout,
};
use crossterm::event::{Event as CrosstermEvent, KeyCode as CrosstermKey};
use std::{
    collections::VecDeque,
    future::Future,
    pin::Pin,
    sync::{Arc, Mutex},
    task::{Context, Poll, Waker},
    time::Duration,
};
use tokio::{task, time};

/// Epoch integer for our channel's versions. Hopefully, it won't overflow.
type Epoch = u128;

/// Snapshot of an event with a given epoch.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Snapshot<E> {
    /// The event of this snapshot.
    event: E,
    /// Epoch count of this snapshot.
    epoch: Epoch,
}

/// Shared data of the event channel.
#[derive(Debug, Clone)]
struct Shared {
    /// Last key event's snapshot.
    key: Snapshot<KeyEvent>,
    /// Last resize event's snapshot.
    resize: Snapshot<ResizeEvent>,
    /// List of wakers from subscribed tasks.
    wakers: VecDeque<Waker>,
    /// Whether there is a connection on the channel.
    connected: bool,
}

impl Default for Shared {
    fn default() -> Self {
        Self {
            key: Snapshot { event: KeyEvent::dummy(), epoch: 0 },
            resize: Snapshot { event: ResizeEvent::dummy(), epoch: 0 },
            wakers: VecDeque::new(),
            connected: true,
        }
    }
}

impl Shared {
    /// Selects maximum epoch.
    fn epoch(&self) -> Epoch {
        self.key.epoch.max(self.resize.epoch)
    }

    /// Reads the correct lastest event, given the epoch where the caller is.
    /// Uses the adequate event, i.e. resize event has precedence.
    fn read(&self, epoch: Epoch) -> Event {
        if self.key.epoch > epoch && self.resize.epoch <= epoch {
            self.key.event.into()
        } else {
            self.resize.event.into()
        }
    }

    /// Writes an event into the channel. Advances current epoch.
    fn write(&mut self, event: Event) {
        let epoch = self.epoch();

        match event {
            Event::Key(event) => {
                self.key = Snapshot { epoch: epoch + 1, event }
            },
            Event::Resize(event) => {
                self.resize = Snapshot { epoch: epoch + 1, event }
            },
        }
    }

    /// Awakes all the subscribed tasks.
    fn wake(&mut self) {
        while let Some(waker) = self.wakers.pop_front() {
            waker.wake();
        }
    }

    /// Subscribes a task's waker.
    fn subscribe(&mut self, waker: Waker) {
        self.wakers.push_back(waker)
    }
}

/// Creates an event channel.
pub(crate) fn channel() -> (Reactor, Listener) {
    let shared = Arc::new(Mutex::new(Shared::default()));
    let reactor = Reactor { shared: shared.clone() };
    let listener = Listener { shared, last: 0 };
    (reactor, listener)
}

/// An event reactor. The event reactor gets events from the OS, perform
/// reactions required to be done immediately, and makes events available to an
/// event listener.
#[derive(Debug)]
pub(crate) struct Reactor {
    /// Reference to concurrent shared channel data.
    shared: Arc<Mutex<Shared>>,
}

impl Reactor {
    /// Performs a "react loop", i.e. keeps polling events, reacting to the
    /// events, and sending them to the event listener.
    pub(crate) async fn react_loop<'screen>(
        &mut self,
        event_interval: Duration,
        screen: &'screen Screen,
        stdout_guard: &mut Option<LockedStdout<'screen>>,
    ) -> Result<(), Error> {
        let mut interval = time::interval(event_interval);

        while self.shared.lock().unwrap().connected {
            match self.poll().await? {
                Some(crossterm) => {
                    self.react(crossterm, screen, stdout_guard).await?
                },
                None => tokio::select! {
                    _ = interval.tick() => (),
                    _ = ReactorSubs { reactor: &self } => (),
                },
            }
        }

        Ok(())
    }

    /// Reacts to a single event. Input uses the crossterm encoding for the
    /// event.
    async fn react<'guard>(
        &self,
        crossterm: CrosstermEvent,
        screen: &'guard Screen,
        stdout: &mut Option<LockedStdout<'guard>>,
    ) -> Result<(), Error> {
        match crossterm {
            CrosstermEvent::Key(key) => {
                let maybe_key =
                    key_from_crossterm(key.code).filter(|_| stdout.is_none());
                if let Some(main_key) = maybe_key {
                    use crossterm::event::KeyModifiers as Mod;

                    let evt = KeyEvent {
                        main_key,
                        ctrl: key.modifiers.intersects(Mod::CONTROL),
                        alt: key.modifiers.intersects(Mod::ALT),
                        shift: key.modifiers.intersects(Mod::SHIFT),
                    };

                    self.send(Event::Key(evt));
                }
            },

            CrosstermEvent::Resize(width, height) => {
                let size = Coord2 { x: width, y: height };
                let mut locked_screen = screen.lock().await?;
                let prev_size_good = stdout.is_none();
                locked_screen.check_resize(size, stdout).await?;

                if stdout.is_none() {
                    let evt = ResizeEvent { size: Some(size) };
                    self.send(Event::Resize(evt));
                } else if prev_size_good {
                    let evt = ResizeEvent { size: None };
                    self.send(Event::Resize(evt));
                }
                drop(locked_screen);
            },

            _ => (),
        }

        Ok(())
    }

    /// Polls for a single event from the OS.
    async fn poll<'guard>(&self) -> Result<Option<CrosstermEvent>, Error> {
        let result = task::block_in_place(|| {
            match crossterm::event::poll(Duration::from_millis(0))? {
                true => crossterm::event::read().map(Some),
                false => Ok(None),
            }
        });

        result.map_err(Error::from_crossterm)
    }

    /// Sends an event through the channel, so that the listener receives it.
    fn send(&self, event: Event) {
        let mut shared = self.shared.lock().unwrap();
        shared.write(event);
        shared.wake();
    }
}

impl Drop for Reactor {
    fn drop(&mut self) {
        let mut shared = self.shared.lock().unwrap();
        shared.connected = false;
        shared.wake();
    }
}

/// A supported pressed key.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Key {
    /// A regular, unicode character. E.g. `Key::Char('a')` or
    /// `Key::Char('ç')`.
    Char(char),
    /// The up arrow key.
    Up,
    /// The down arrow key.
    Down,
    /// The left arrow key.
    Left,
    /// The right arrow key.
    Right,
    /// The escape key.
    Esc,
    /// The enter key. Preferred over `Char('\n')`.
    Enter,
    /// The backspace key
    Backspace,
}

/// An event fired by a key pressed by the user.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct KeyEvent {
    /// Key pressed by the user.
    pub main_key: Key,
    /// Whether control is modifiying the key (pressed).
    pub ctrl: bool,
    /// Whether alt is modifiying the key (pressed).
    pub alt: bool,
    /// Whether shift is modifiying the key (pressed).
    pub shift: bool,
}

impl KeyEvent {
    /// A dummy key event, with dummy data.
    fn dummy() -> Self {
        Self { main_key: Key::Esc, ctrl: false, alt: false, shift: false }
    }
}

/// An event fired by a resize of the screen.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ResizeEvent {
    /// New dimensions of the screen. If set to `None`, then the screen was
    /// resized to an invalid size, and andiskaz's event reactor is taking care
    /// of this (or andiskis event reactor if you will).
    pub size: Option<Coord2>,
}

impl ResizeEvent {
    /// A dummy resize event, with dummy data.
    fn dummy() -> Self {
        Self { size: None }
    }
}

/// A generic event type.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Event {
    /// User resized screen.
    Resize(ResizeEvent),
    /// User pressed key.
    Key(KeyEvent),
}

impl From<ResizeEvent> for Event {
    fn from(event: ResizeEvent) -> Self {
        Event::Resize(event)
    }
}

impl From<KeyEvent> for Event {
    fn from(event: KeyEvent) -> Self {
        Event::Key(event)
    }
}

/// Translates a crossterm key to an Andiskaz key.
fn key_from_crossterm(crossterm: CrosstermKey) -> Option<Key> {
    match crossterm {
        CrosstermKey::Esc => Some(Key::Esc),
        CrosstermKey::Backspace => Some(Key::Backspace),
        CrosstermKey::Enter => Some(Key::Enter),
        CrosstermKey::Up => Some(Key::Up),
        CrosstermKey::Down => Some(Key::Down),
        CrosstermKey::Left => Some(Key::Left),
        CrosstermKey::Right => Some(Key::Right),
        CrosstermKey::Char(ch) => Some(Key::Char(ch)),
        _ => None,
    }
}

/// Handle to terminal events. It can listen for either key or resize events.
#[derive(Debug, Clone)]
pub struct Listener {
    last: Epoch,
    shared: Arc<Mutex<Shared>>,
}

impl Listener {
    /// Checks if an event happened, without blocking.
    pub fn check(&mut self) -> Result<Option<Event>, EventsOff> {
        let shared = self.shared.lock().unwrap();
        let epoch = shared.epoch();
        if epoch > self.last {
            let event = shared.read(self.last);
            self.last = epoch;
            Ok(Some(event))
        } else if shared.connected {
            Ok(None)
        } else {
            Err(EventsOff)
        }
    }

    /// Listens for an event to happen. Waits until an event is available.
    pub async fn listen(&mut self) -> Result<Event, EventsOff> {
        ListenerSubs { listener: self }.await
    }
}

impl Drop for Listener {
    fn drop(&mut self) {
        if Arc::strong_count(&self.shared) <= 2 {
            // Only disconnect if we are the last listener (or if there is no
            // reactor).
            let mut shared = self.shared.lock().unwrap();
            shared.connected = false;
            shared.wake();
        }
    }
}

/// Subscribes a listener to the channel's waker list. Subscription wakes the
/// task up whenever there is a message or when the reactor disconnected.
#[derive(Debug)]
struct ListenerSubs<'list> {
    /// The listener being subscribed.
    listener: &'list mut Listener,
}

impl<'list> Future for ListenerSubs<'list> {
    type Output = Result<Event, EventsOff>;

    fn poll(mut self: Pin<&mut Self>, ctx: &mut Context) -> Poll<Self::Output> {
        // Reborrowing cause otherwise borrow checker will treat all mutable
        // borrows to Self's fields as a single borrow.
        let this = &mut *self;

        let mut shared = this.listener.shared.lock().unwrap();
        let epoch = shared.epoch();
        if epoch > this.listener.last {
            let event = shared.read(this.listener.last);
            this.listener.last = epoch;
            Poll::Ready(Ok(event))
        } else if shared.connected {
            shared.subscribe(ctx.waker().clone());
            Poll::Pending
        } else {
            Poll::Ready(Err(EventsOff))
        }
    }
}

/// Subscribes a reactor to the channel's waker list. Subscription wakes the
/// task up when the listeners disconnected.
#[derive(Debug)]
struct ReactorSubs<'react> {
    /// The reactor being subscribed.
    reactor: &'react Reactor,
}

impl<'react> Future for ReactorSubs<'react> {
    type Output = ();

    fn poll(self: Pin<&mut Self>, ctx: &mut Context) -> Poll<Self::Output> {
        let mut shared = self.reactor.shared.lock().unwrap();
        shared.subscribe(ctx.waker().clone());
        if shared.connected {
            Poll::Pending
        } else {
            Poll::Ready(())
        }
    }
}
