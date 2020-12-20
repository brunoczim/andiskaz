//! This module defines input events of a terminal.

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

type Epoch = u128;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Snapshot<E> {
    event: E,
    epoch: Epoch,
}

#[derive(Debug, Clone)]
struct Shared {
    key: Snapshot<KeyEvent>,
    resize: Snapshot<ResizeEvent>,
    wakers: VecDeque<Waker>,
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
    fn epoch(&self) -> Epoch {
        self.key.epoch.max(self.resize.epoch)
    }

    fn read(&self, epoch: Epoch) -> Event {
        if self.key.epoch > epoch && self.resize.epoch < epoch {
            self.key.event.into()
        } else {
            self.resize.event.into()
        }
    }

    fn write(&self, event: Event) {
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

    fn wake(&mut self) {
        while let Some(waker) = self.wakers.pop_front() {
            waker.wake();
        }
    }

    fn subscribe(&mut self, waker: Waker) {
        self.wakers.push_back(waker)
    }
}

pub(crate) fn channel() -> (Reactor, Listener) {
    let shared = Arc::new(Mutex::new(Shared::default()));
    let reactor = Reactor { shared: shared.clone() };
    let listener = Listener { shared, last: 0 };
    (reactor, listener)
}

#[derive(Debug)]
pub(crate) struct Reactor {
    shared: Arc<Mutex<Shared>>,
}

impl Reactor {
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

    async fn poll<'guard>(&self) -> Result<Option<CrosstermEvent>, Error> {
        let result = task::block_in_place(|| {
            match crossterm::event::poll(Duration::from_millis(0))? {
                true => crossterm::event::read().map(Some),
                false => Ok(None),
            }
        });

        result.map_err(Error::from_crossterm)
    }

    fn send(&self, event: Event) {
        let mut shared = self.shared.lock().unwrap();
        shared.write(event);
        shared.wake();
    }
}

impl Drop for Reactor {
    fn drop(&mut self) {
        let shared = self.shared.lock().unwrap();
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
    fn dummy() -> Self {
        Self { main_key: Key::Esc, ctrl: false, alt: false, shift: false }
    }
}

/// An event fired by a resize of the screen.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ResizeEvent {
    /// New dimensions of the screen.
    pub size: Option<Coord2>,
}

impl ResizeEvent {
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
    pub fn check(&self) -> Result<Option<Event>, EventsOff> {
        let shared = self.shared.lock().unwrap();
        let epoch = shared.epoch();
        if epoch > self.last {
            Ok(Some(shared.read(self.last)))
        } else if shared.connected {
            Ok(None)
        } else {
            Err(EventsOff)
        }
    }

    /// Listens for an event to happen. Waits until an event is available.
    pub async fn listen(&self) -> Result<Event, EventsOff> {
        loop {
            if let Some(event) = self.check()? {
                break Ok(event);
            }
            ListenerSubs { listener: &self }.await;
        }
    }
}

impl Drop for Listener {
    fn drop(&mut self) {
        if Arc::strong_count(&self.shared) <= 2 {
            let shared = self.shared.lock().unwrap();
            shared.connected = false;
            shared.wake();
        }
    }
}

#[derive(Debug)]
struct ListenerSubs<'list> {
    listener: &'list Listener,
}

impl<'list> Future for ListenerSubs<'list> {
    type Output = Result<Event, EventsOff>;

    fn poll(self: Pin<&mut Self>, ctx: &mut Context) -> Poll<Self::Output> {
        let mut shared = self.listener.shared.lock().unwrap();
        let epoch = shared.epoch();
        if epoch > self.listener.last {
            let event = shared.read(self.listener.last);
            Poll::Ready(Ok(event))
        } else if shared.connected {
            shared.subscribe(ctx.waker().clone());
            Poll::Pending
        } else {
            Poll::Ready(Err(EventsOff))
        }
    }
}

#[derive(Debug)]
struct ReactorSubs<'react> {
    reactor: &'react Reactor,
}

impl<'react> Future for ReactorSubs<'react> {
    type Output = ();

    fn poll(self: Pin<&mut Self>, ctx: &mut Context) -> Poll<Self::Output> {
        let mut shared = self.reactor.shared.lock().unwrap();
        if shared.connected {
            Poll::Pending
        } else {
            Poll::Ready(())
        }
    }
}
