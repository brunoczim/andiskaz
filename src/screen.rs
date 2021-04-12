//! This module defines screen related utilities.

mod buffer;
mod locked;

use crate::{
    color::{transform::PairTransformer, Color},
    coord::{Coord, Coord2},
    error::{Error, RendererOff},
    stdio,
    stdio::{restore_screen, save_screen, Stdout},
    string::TermString,
    style::Style,
};
use buffer::ScreenBuffer;
use std::{
    fmt::Write,
    sync::{
        atomic::{AtomicBool, Ordering::*},
        Arc,
    },
    time::Duration,
};
use tokio::{
    sync::{Mutex, Notify},
    task,
    time,
};

pub use self::{buffer::Tile, locked::LockedScreen};

/// Shared memory between terminal handle copies.
#[derive(Debug)]
pub(crate) struct Shared {
    /// Whether the terminal handle has been cleaned up (using
    /// terminal.cleanup).
    cleanedup: AtomicBool,
    /// A lock to the standard output.
    stdout: Stdout,
    /// Buffer responsible for rendering the screen.
    buffer: Mutex<ScreenBuffer>,
    renderer_conn: AtomicBool,
    renderer_notif: Notify,
    min_size: Coord2,
    frame_time: Duration,
}

impl Shared {
    pub async fn lock<'shared>(
        &'shared self,
    ) -> Result<LockedScreen<'shared>, RendererOff> {
        let locked = LockedScreen::new(self).await;

        if self.renderer_conn.load(Relaxed) {
            Ok(locked)
        } else {
            Err(RendererOff)
        }
    }
}

impl Drop for Shared {
    fn drop(&mut self) {
        if !self.cleanedup.load(Relaxed) {
            let _ = crossterm::terminal::disable_raw_mode();
            let mut buf = String::new();
            write!(buf, "{}", crossterm::cursor::Show)
                .ok()
                .and_then(|_| stdio::restore_screen(&mut buf).ok())
                .map(|_| println!("{}", buf));
        }
    }
}

/// A handle to the screen. Used to write or read from the screen cells, as well
/// to perform more advanced operations.
#[derive(Debug, Clone)]
pub struct Screen {
    shared: Arc<Shared>,
}

impl Screen {
    pub(crate) fn new(
        size: Coord2,
        min_size: Coord2,
        frame_time: Duration,
    ) -> Self {
        let shared = Arc::new(Shared {
            cleanedup: AtomicBool::new(false),
            stdout: Stdout::new(),
            buffer: Mutex::new(ScreenBuffer::blank(size)),
            renderer_conn: AtomicBool::new(true),
            renderer_notif: Notify::new(),
            min_size,
            frame_time,
        });

        Self { shared }
    }

    /// Returns the minimum size required for the screen.
    pub fn min_size(&self) -> Coord2 {
        self.shared.min_size
    }

    /// Locks the screen handle. Most operations of the screen handle will lock
    /// the handle, perform the operation, and then unlock the handle. When one
    /// locks the screen manually, one can use the methods directly on the
    /// locked screen, without having to lock and unlock everytime.
    pub async fn lock<'screen>(
        &'screen self,
    ) -> Result<LockedScreen<'screen>, RendererOff> {
        let locked = LockedScreen::new(&self.shared).await;
        if self.shared.renderer_conn.load(Relaxed) {
            Ok(locked)
        } else {
            Err(RendererOff)
        }
    }

    /// Returns the current size of the screen.
    pub async fn size(&self) -> Result<Coord2, RendererOff> {
        self.lock().await.map(|locked| locked.size())
    }

    /// Sets every attribute of a given [`Tile`]. This operation is buffered.
    pub async fn set(
        &mut self,
        point: Coord2,
        tile: Tile,
    ) -> Result<(), RendererOff> {
        self.lock().await.map(|mut locked| locked.set(point, tile))
    }

    /// Sets the colors of a given [`Tile`]. This operation is buffered.
    pub async fn transform_colors<P>(
        &mut self,
        point: Coord2,
        transformer: P,
    ) -> Result<(), RendererOff>
    where
        P: PairTransformer,
    {
        self.lock()
            .await
            .map(|mut locked| locked.transform_colors(point, transformer))
    }

    /// Applies an update function to a [`Tile`]. An update function gets access
    /// to a mutable reference of a [`Tile`], updates it, and then the screen
    /// handles any changes made to it. This operation is buffered.
    pub async fn update<F, T>(
        &mut self,
        point: Coord2,
        updater: F,
    ) -> Result<T, RendererOff>
    where
        F: FnOnce(&mut Tile) -> T,
    {
        self.lock().await.map(|mut locked| locked.update(point, updater))
    }

    /// Gets the attributes of a given [`Tile`], regardless of being flushed to
    /// the screen yet or not.
    pub async fn get(&self, point: Coord2) -> Result<Tile, RendererOff> {
        self.lock().await.map(|locked| locked.get(point).clone())
    }

    /// Sets every [`Tile`] into a whitespace grapheme with the given color.
    pub async fn clear(
        &mut self,
        background: Color,
    ) -> Result<(), RendererOff> {
        self.lock().await.map(|mut locked| locked.clear(background))
    }

    /// Prints a grapheme-encoded text (a [`TermString`]) using some style
    /// options like ratio to the screen, color, margin and others. See
    /// [`Style`].
    pub async fn styled_text<P>(
        &mut self,
        tstring: &TermString,
        style: Style<P>,
    ) -> Result<Coord, RendererOff>
    where
        P: PairTransformer,
    {
        self.lock().await.map(|mut locked| locked.styled_text(tstring, style))
    }

    /// Initialization of the terminal, such as cleaning the screen.
    pub(crate) async fn setup(&self) -> Result<(), Error> {
        let mut buf = String::new();
        save_screen(&mut buf)?;
        write!(
            buf,
            "{}{}{}{}",
            crossterm::style::SetBackgroundColor(
                crossterm::style::Color::Black
            ),
            crossterm::style::SetForegroundColor(
                crossterm::style::Color::White
            ),
            crossterm::cursor::Hide,
            crossterm::terminal::Clear(crossterm::terminal::ClearType::All),
        )?;
        self.shared.stdout.write_and_flush(buf.as_bytes()).await?;
        Ok(())
    }

    /// Asynchronous cleanup. It is preferred to call this before dropping.
    pub(crate) async fn cleanup(&self) -> Result<(), Error> {
        task::block_in_place(|| crossterm::terminal::disable_raw_mode())
            .map_err(Error::from_crossterm)?;
        let mut buf = String::new();
        write!(buf, "{}", crossterm::cursor::Show)?;
        restore_screen(&mut buf)?;
        self.shared.stdout.write_and_flush(buf.as_bytes()).await?;
        self.shared.cleanedup.store(true, Release);
        Ok(())
    }

    /// Creates a connection guard. On drop, it closes the connection with the
    /// renderer.
    pub(crate) fn conn_guard(&self) -> ScreenGuard {
        ScreenGuard { screen: self }
    }
}

/// A connection guard. It closes the connection between screen handle and
/// renderer on drop.
#[derive(Debug)]
pub(crate) struct ScreenGuard<'screen> {
    /// Reference to the original screen handle, on which connection will be
    /// terminated.
    screen: &'screen Screen,
}

impl<'screen> Drop for ScreenGuard<'screen> {
    fn drop(&mut self) {
        self.screen.shared.renderer_conn.store(false, Release);
        self.screen.shared.renderer_notif.notify_one();
    }
}

/// The renderer loop. Should be called only when setting up a terminal handler.
/// Exits on error or when notified that it should exit.
pub(crate) async fn renderer(screen: &Screen) -> Result<(), Error> {
    let mut interval = time::interval(screen.shared.frame_time);
    let mut buf = String::new();

    loop {
        {
            let mut locked = screen.lock().await?;
            locked.render(&mut buf).await?;
        }

        tokio::select! {
            _ = interval.tick() => (),
            _ = screen.shared.renderer_notif.notified() => break,
        };
    }

    Ok(())
}
