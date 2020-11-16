//! This module defines input events of a terminal.

use crate::{
    coord::Coord2,
    error::{Error, EventsOff},
    screen::Screen,
    stdio::LockedStdout,
};
use crossterm::event::{Event as CrosstermEvent, KeyCode as CrosstermKey};
use futures::future::FutureExt;
use std::time::Duration;
use tokio::{sync::watch, task, time};

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

/// An event fired by a resize of the screen.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ResizeEvent {
    /// New dimensions of the screen.
    pub size: Coord2,
}

/// A generic event type.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Event {
    /// User resized screen.
    Resize(ResizeEvent),
    /// User pressed key.
    Key(KeyEvent),
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

pub(crate) async fn event_listener<'screen>(
    event_interval: Duration,
    sender: watch::Sender<Event>,
    screen: &'screen Screen,
    stdout_guard: &mut Option<LockedStdout<'screen>>,
) -> Result<(), Error> {
    let mut interval = time::interval(event_interval);

    while !sender.is_closed() {
        if !poll(screen, &sender, stdout_guard).await? {
            tokio::select! {
                _ = interval.tick() => (),
                _ = sender.closed() => (),
            }
        }
    }

    Ok(())
}

pub(crate) async fn poll<'guard>(
    screen: &'guard Screen,
    sender: &watch::Sender<Event>,
    stdout: &mut Option<LockedStdout<'guard>>,
) -> Result<bool, Error> {
    let evt = task::block_in_place(|| {
        match crossterm::event::poll(Duration::from_millis(0))? {
            true => crossterm::event::read().map(Some),
            false => Ok(None),
        }
    });

    match evt.map_err(Error::from_crossterm)? {
        Some(CrosstermEvent::Key(key)) => {
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

                let _ = sender.send(Event::Key(evt));
            }

            Ok(true)
        },

        Some(CrosstermEvent::Resize(width, height)) => {
            let size = Coord2 { x: width, y: height };
            screen.lock().await?.check_resize(size, stdout).await?;
            if stdout.is_none() {
                let evt = ResizeEvent { size };
                let _ = sender.send(Event::Resize(evt));
            }

            Ok(true)
        },

        _ => Ok(false),
    }
}

/// Handle to terminal events. It can listen for either key or resize events.
#[derive(Debug, Clone)]
pub struct Events {
    /// Receiver of the event channel.
    recv: watch::Receiver<Event>,
}

impl Events {
    /// Creates an events handle from the given receiver of the event channel.
    pub(crate) fn new(recv: watch::Receiver<Event>) -> Self {
        Self { recv }
    }

    /// Checks if an event happened, without blocking.
    pub fn check(&mut self) -> Result<Option<Event>, EventsOff> {
        self.listen().now_or_never().transpose()
    }

    /// Listens for an event to happen. Waits until an event is available.
    pub async fn listen(&mut self) -> Result<Event, EventsOff> {
        self.recv.changed().await.map_err(|_| EventsOff)?;
        Ok(self.recv.borrow().clone())
    }
}
