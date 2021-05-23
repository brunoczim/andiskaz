use crate::{
    coord::Coord2,
    error::Error,
    event::{Event, Key, KeyEvent, ResizeEvent},
    stdio::LockedStdout,
    terminal::Shared,
};
use crossterm::event::{Event as CrosstermEvent, KeyCode as CrosstermKey};
use std::time::Duration;
use tokio::{task, time};

#[derive(Debug)]
pub(crate) struct Reactor<'shared> {
    shared: &'shared Shared,
}

impl<'shared> Reactor<'shared> {
    pub fn new(shared: &'shared Shared) -> Self {
        Self { shared }
    }

    pub async fn pre_loop<'stdout>(
        &mut self,
        stdout_guard: &mut Option<LockedStdout<'stdout>>,
    ) -> Result<(), Error>
    where
        'shared: 'stdout,
    {
        let mut locked = self.shared.screen().lock().await;
        let size = locked.size();
        let min_size = locked.min_size();
        if size.x < min_size.x || size.y < min_size.y {
            locked.check_resize(min_size, stdout_guard).await?;
            locked.check_resize(size, stdout_guard).await?;
            let evt = ResizeEvent { size: None };
            self.send(Event::Resize(evt));
        }
        Ok(())
    }

    /// Performs a "react loop", i.e. keeps polling events, reacting to the
    /// events, and sending them to the event listener.
    pub(crate) async fn react_loop<'stdout>(
        &'stdout mut self,
        event_interval: Duration,
        stdout_guard: &mut Option<LockedStdout<'stdout>>,
    ) -> Result<(), Error> {
        let mut interval = time::interval(event_interval);

        while self.shared.is_connected() {
            match self.poll().await? {
                Some(crossterm) => self.react(crossterm, stdout_guard).await?,
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
    async fn react<'stdout>(
        &'stdout self,
        crossterm: CrosstermEvent,
        stdout: &mut Option<LockedStdout<'stdout>>,
    ) -> Result<(), Error> {
        let _guard = self.shared.service_guard().await?;

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
                let mut locked_screen = self.shared.screen().lock().await;
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
