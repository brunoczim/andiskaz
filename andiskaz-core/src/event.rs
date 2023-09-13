use crate::coord::Vec2;

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
