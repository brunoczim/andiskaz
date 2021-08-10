//! This module defines input events of a terminal.

mod reactor;

use crate::coord::Vec2;
use std::sync::Mutex;
use tokio::sync::Notify;

pub(crate) use reactor::Reactor;

/// Epoch integer for our channel's versions. Hopefully, it won't overflow.
pub type Epoch = u128;

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
    pub size: Option<Vec2>,
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

/// Snapshot of an event with a given epoch.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Snapshot<E> {
    /// The event of this snapshot.
    event: E,
    /// Epoch count of this snapshot.
    epoch: Epoch,
}

/// Channel's data that needs locking.
#[derive(Debug)]
struct ChannelData {
    /// Last key event's snapshot.
    key: Snapshot<KeyEvent>,
    /// Last resize event's snapshot.
    resize: Snapshot<ResizeEvent>,
}

impl Default for ChannelData {
    fn default() -> Self {
        Self {
            key: Snapshot { event: KeyEvent::dummy(), epoch: 0 },
            resize: Snapshot { event: ResizeEvent::dummy(), epoch: 0 },
        }
    }
}

impl ChannelData {
    /// Selects maximum last epoch.
    fn epoch(&self) -> Epoch {
        self.key.epoch.max(self.resize.epoch)
    }

    /// Reads the correct lastest event, given the epoch where the caller is.
    /// Uses the adequate event, i.e. resize event has precedence.
    fn read(&self, epoch: Epoch) -> Option<(Epoch, Event)> {
        if self.resize.epoch > epoch {
            Some((self.resize.epoch, self.resize.event.into()))
        } else if self.key.epoch > epoch {
            Some((self.key.epoch, self.key.event.into()))
        } else {
            None
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
}

/// Shared channel handle (for both senders and receivers).
#[derive(Debug)]
pub(crate) struct Channel {
    /// Data that needs to be lock (the effective data of the channel).
    data: Mutex<ChannelData>,
    /// Notification handle of the channel.
    notifier: Notify,
}

impl Default for Channel {
    fn default() -> Self {
        Channel {
            data: Mutex::new(ChannelData::default()),
            notifier: Notify::new(),
        }
    }
}

impl Channel {
    /// Notifies all parties subscribed to the channel.
    pub fn notify(&self) {
        self.notifier.notify_waiters()
    }

    /// Subscribes to changes in this channel.
    pub async fn subscribe(&self) {
        self.notifier.notified().await
    }

    /// Selects maximum last epoch.
    pub fn epoch(&self) -> Epoch {
        self.data.lock().unwrap().epoch()
    }

    /// Reads the correct lastest event, given the epoch where the caller is.
    /// Uses the adequate event, i.e. resize event has precedence.
    pub fn read(&self, epoch: Epoch) -> Option<(Epoch, Event)> {
        self.data.lock().unwrap().read(epoch)
    }

    /// Writes an event into the channel. Advances current epoch.
    pub fn write(&self, event: Event) {
        self.data.lock().unwrap().write(event)
    }
}
