//! This file defines an event reactor.

use crate::{
    coord::Vec2,
    error::Error,
    event::{Event, Key, KeyEvent, ResizeEvent},
    stdio::LockedStdout,
    terminal::Shared,
};
use crossterm::event::{Event as CrosstermEvent, KeyCode as CrosstermKey};
use std::time::Duration;
use tokio::{task, time};

/// Event reactor: gets events from the low-level (OS + Crossterm), and sends
/// them to the events channel.
#[derive(Debug)]
pub(crate) struct Reactor<'shared> {
    /// Shared data between every party of the application.
    shared: &'shared Shared,
    /// A guard to the standard output, to prevent renderer from rendering if
    /// invalid size.
    stdout_guard: Option<LockedStdout<'shared>>,
}

impl<'shared> Reactor<'shared> {
    /// Constructs the reactor from a reference to shared data on which it will
    /// place the events.
    pub fn new(shared: &'shared Shared) -> Self {
        Self { shared, stdout_guard: None }
    }

    /// Returns whether the current screen size is valid.
    fn is_size_valid(&self) -> bool {
        self.stdout_guard.is_none()
    }

    /// Executes the pre-"reactor loop" functions, handling the initial screen
    /// size and correctly dealing with the fact that it is invalid, if it is.
    pub async fn pre_loop(
        &mut self,
        initial_size: Vec2,
    ) -> Result<(), Error> {
        let mut screen = self.shared.screen().lock().await;
        let min_size = screen.min_size();
        if initial_size.x < min_size.x || initial_size.y < min_size.y {
            screen.check_resize(initial_size, &mut self.stdout_guard).await?;
            let evt = ResizeEvent { size: None };
            self.send(Event::Resize(evt));
        }
        Ok(())
    }

    /// Performs a "react loop", i.e. keeps polling events, reacting to the
    /// events, and sending them to the event listener.
    pub async fn react_loop(
        &mut self,
        event_interval: Duration,
    ) -> Result<(), Error> {
        let mut interval = time::interval(event_interval);

        while self.shared.is_connected() {
            match self.poll().await? {
                Some(crossterm) => self.react(crossterm).await?,
                None => tokio::select! {
                    _ = interval.tick() => (),
                    _ = self.shared.events().subscribe() => (),
                },
            }
        }

        Ok(())
    }

    /// Reacts to a single event. Input uses the crossterm encoding for the
    /// event.
    async fn react(&mut self, crossterm: CrosstermEvent) -> Result<(), Error> {
        let _guard = self.shared.service_guard().await?;

        match crossterm {
            CrosstermEvent::Key(key) => {
                let maybe_key = key_from_crossterm(key.code);
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
                let size = Vec2 { x: width, y: height };
                let mut locked_screen = self.shared.screen().lock().await;
                let prev_size_valid = self.is_size_valid();
                locked_screen
                    .check_resize(size, &mut self.stdout_guard)
                    .await?;

                if self.is_size_valid() {
                    let evt = ResizeEvent { size: Some(size) };
                    self.send(Event::Resize(evt));
                } else if prev_size_valid {
                    let evt = ResizeEvent { size: None };
                    self.send(Event::Resize(evt));
                }
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
        self.shared.events().write(event);
        self.shared.events().notify();
    }
}

impl<'shared> Drop for Reactor<'shared> {
    fn drop(&mut self) {
        self.shared.disconnect();
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
