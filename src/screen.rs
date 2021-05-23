//! This module defines screen related utilities.

mod buffer;

pub use self::buffer::Tile;
use crate::{
    color::{transform::PairTransformer, Color, Color2},
    coord,
    coord::{Coord, Coord2},
    error::Error,
    screen::buffer::ScreenBuffer,
    stdio,
    stdio::{restore_screen, save_screen, LockedStdout, Stdout},
    string::{TermGrapheme, TermString},
    style::Style,
    terminal::Shared,
};
use std::{
    fmt::Write,
    sync::atomic::{AtomicBool, Ordering::*},
    time::Duration,
};
use tokio::{
    io,
    sync::{Mutex, MutexGuard, Notify},
    task,
    time,
};

/// Shared memory between terminal handle copies.
#[derive(Debug)]
pub(crate) struct ScreenData {
    min_size: Coord2,
    frame_time: Duration,
    /// Whether the terminal handle has been cleaned up (using
    /// terminal.cleanup).
    cleanedup: AtomicBool,
    /// A lock to the standard output.
    stdout: Stdout,
    /// Buffer responsible for rendering the screen.
    buffer: Mutex<ScreenBuffer>,
    /// Notification handle of the screen.
    notifier: Notify,
}

impl ScreenData {
    /// Creates screen data from the given settings. If given actual size is
    /// less than given minimum allowed size, the actual size is replaced by the
    /// minimum size.
    pub fn new(size: Coord2, min_size: Coord2, frame_time: Duration) -> Self {
        let corrected_size = if size.x >= min_size.x && size.y >= min_size.y {
            size
        } else {
            min_size
        };
        Self {
            min_size,
            frame_time,
            cleanedup: AtomicBool::new(false),
            stdout: Stdout::new(),
            buffer: Mutex::new(ScreenBuffer::blank(corrected_size)),
            notifier: Notify::new(),
        }
    }

    /// Notifies all parties subscribed to the screen updates.
    pub fn notify(&self) {
        self.notifier.notify_waiters()
    }

    /// Subscribes to changes in the screen data such as disconnection.
    async fn subscribe(&self) {
        self.notifier.notified().await
    }

    /// Locks the screen data into an actual screen handle.
    pub async fn lock<'this>(&'this self) -> Screen<'this> {
        Screen::new(self).await
    }

    /// Initialization of the terminal, such as cleaning the screen.
    pub async fn setup(&self) -> Result<(), Error> {
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
        self.stdout.write_and_flush(buf.as_bytes()).await?;
        Ok(())
    }

    /// Asynchronous cleanup. It is preferred to call this before dropping.
    pub async fn cleanup(&self) -> Result<(), Error> {
        task::block_in_place(|| crossterm::terminal::disable_raw_mode())
            .map_err(Error::from_crossterm)?;
        let mut buf = String::new();
        write!(buf, "{}", crossterm::cursor::Show)?;
        restore_screen(&mut buf)?;
        self.stdout.write_and_flush(buf.as_bytes()).await?;
        self.cleanedup.store(true, Release);
        Ok(())
    }
}

impl Drop for ScreenData {
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

/// Panics given that a point in the screen is out of bounds. This is here so
/// that the compiler can make other functions smaller.
#[cold]
#[inline(never)]
fn out_of_bounds(point: Coord2, size: Coord2) -> ! {
    panic!(
        "Point x: {}, y: {} out of screen size x: {}, y: {}",
        point.x, point.y, size.x, size.y
    )
}

/// A locked screen terminal with exclusive access to it. By default,
/// [`ScreenData`] locks and unlocks every operation. With this struct, a locked
/// screen handle, one can execute many operations without locking and
/// unlocking.
#[derive(Debug)]
pub struct Screen<'terminal> {
    /// Reference to the original screen.
    data: &'terminal ScreenData,
    /// Locked guard to the buffer.
    buffer: MutexGuard<'terminal, ScreenBuffer>,
}

impl<'terminal> Screen<'terminal> {
    /// Creates a locked screen from a reference to the unlocked screen handle,
    /// and a locked guard to the buffer.
    pub(crate) async fn new<'param>(
        data: &'param ScreenData,
    ) -> Screen<'terminal>
    where
        'param: 'terminal,
    {
        Self { data, buffer: data.buffer.lock().await }
    }

    /// Returns the current size of the screen.
    pub fn size(&self) -> Coord2 {
        self.buffer.size()
    }

    /// Returns the minimum size required for the screen.
    pub fn min_size(&self) -> Coord2 {
        self.data.min_size
    }

    /// Sets every attribute of a given [`Tile`]. This operation is buffered.
    pub fn set(&mut self, point: Coord2, tile: Tile) {
        self.update(point, |stored| *stored = tile);
    }

    /// Sets the colors of a given [`Tile`]. This operation is buffered.
    pub fn transform_colors<P>(&mut self, point: Coord2, transformer: P)
    where
        P: PairTransformer,
    {
        self.update(point, |stored| {
            stored.colors = transformer.transform_pair(stored.colors)
        })
    }

    /// Applies an update function to a [`Tile`]. An update function gets access
    /// to a mutable reference of a [`Tile`], updates it, and then the screen
    /// handles any changes made to it. This operation is buffered.
    pub fn update<F, T>(&mut self, point: Coord2, updater: F) -> T
    where
        F: FnOnce(&mut Tile) -> T,
    {
        let index = self
            .buffer
            .make_index(point)
            .unwrap_or_else(|| out_of_bounds(point, self.buffer.size()));
        let ret = updater(&mut self.buffer.curr[index]);
        if self.buffer.old[index] != self.buffer.curr[index] {
            self.buffer.changed.insert(point);
        } else {
            self.buffer.changed.remove(&point);
        }
        ret
    }

    /// Gets the attributes of a given [`Tile`], regardless of being flushed to
    /// the screen yet or not.
    pub fn get(&self, point: Coord2) -> &Tile {
        let index = self
            .buffer
            .make_index(point)
            .unwrap_or_else(|| out_of_bounds(point, self.buffer.size()));
        &self.buffer.curr[index]
    }

    /// Sets every [`Tile`] into a whitespace grapheme with the given color.
    pub fn clear(&mut self, background: Color) {
        let size = self.buffer.size();
        let tile = Tile {
            colors: Color2 { background, ..Color2::default() },
            grapheme: TermGrapheme::space(),
        };

        for y in 0 .. size.y {
            for x in 0 .. size.x {
                self.set(Coord2 { x, y }, tile.clone());
            }
        }
    }

    /// Prints a grapheme-encoded text (a [`TermString`]) using some style
    /// options like ratio to the screen, color, margin and others. See
    /// [`Style`].
    pub fn styled_text<P>(
        &mut self,
        tstring: &TermString,
        style: Style<P>,
    ) -> Coord
    where
        P: PairTransformer,
    {
        let mut len = tstring.count_graphemes();
        let mut slice = tstring.index(..);
        let screen_size = self.buffer.size();
        let size = style.make_size(screen_size);

        let mut cursor = Coord2 { x: 0, y: style.top_margin };
        let mut is_inside = cursor.y - style.top_margin < size.y;

        while len > 0 && is_inside {
            is_inside = cursor.y - style.top_margin + 1 < size.y;
            let width = coord::to_index(size.x);
            let pos = self.find_break_pos(width, len, size, &slice, is_inside);

            cursor.x = size.x - pos as Coord;
            cursor.x = cursor.x + style.left_margin - style.right_margin;
            cursor.x = cursor.x / style.align_denom * style.align_numer;

            slice = slice.index(.. pos);

            self.write_styled_slice(&slice, &style, &mut cursor);

            if pos != len && !is_inside {
                self.update(cursor, |tile| {
                    let grapheme = TermGrapheme::new_lossy("…");
                    let colors = style.colors.transform_pair(tile.colors);
                    *tile = Tile { grapheme, colors };
                });
            }

            cursor.y += 1;
            len -= pos;
        }
        cursor.y
    }

    /// Finds the position where a line should break in a styled text.
    fn find_break_pos(
        &self,
        width: usize,
        total_graphemes: usize,
        term_size: Coord2,
        slice: &TermString,
        is_inside: bool,
    ) -> usize {
        if width <= slice.len() {
            let mut pos = slice
                .index(.. term_size.x as usize)
                .iter()
                .rev()
                .position(|grapheme| grapheme == TermGrapheme::space())
                .map_or(width, |rev| total_graphemes - rev);
            if !is_inside {
                pos -= 1;
            }
            pos
        } else {
            total_graphemes
        }
    }

    /// Writes a slice using the given style. It should fit in one line.
    fn write_styled_slice<P>(
        &mut self,
        slice: &TermString,
        style: &Style<P>,
        cursor: &mut Coord2,
    ) where
        P: PairTransformer,
    {
        for grapheme in slice {
            self.update(*cursor, |tile| {
                let colors = style.colors.transform_pair(tile.colors);
                *tile = Tile { grapheme: grapheme.clone(), colors };
            });
            cursor.x += 1;
        }
    }

    /// Checks if the new size is valid. If valid, then it resizes the screen,
    /// and sets the `guard` to `None`. Otherwise, stdout is locked, and the
    /// locked stdout is put into `guard`.
    pub(crate) async fn check_resize(
        &mut self,
        new_size: Coord2,
        guard: &mut Option<LockedStdout<'terminal>>,
    ) -> io::Result<()> {
        let min_size = self.data.min_size;
        if new_size.x < min_size.x || new_size.y < min_size.y {
            if guard.is_none() {
                let mut stdout = self.data.stdout.lock().await;
                self.ask_resize(&mut stdout, min_size).await?;
                *guard = Some(stdout);
            }
        } else {
            let mut stdout = match guard.take() {
                Some(stdout) => stdout,
                None => self.data.stdout.lock().await,
            };

            self.resize(new_size, &mut stdout).await?;
        }

        Ok(())
    }

    /// Asks the user to resize the screen (manually).
    async fn ask_resize(
        &mut self,
        stdout: &mut LockedStdout<'terminal>,
        min_size: Coord2,
    ) -> io::Result<()> {
        let buf = format!(
            "{}{}RESIZE {}x{}",
            crossterm::terminal::Clear(crossterm::terminal::ClearType::All),
            crossterm::cursor::MoveTo(0, 0),
            min_size.x,
            min_size.y,
        );

        stdout.write_and_flush(buf.as_bytes()).await?;

        Ok(())
    }

    /// Triggers the resize of the screen.
    async fn resize(
        &mut self,
        new_size: Coord2,
        stdout: &mut LockedStdout<'terminal>,
    ) -> io::Result<()> {
        let buf = format!(
            "{}{}{}",
            crossterm::style::SetForegroundColor(
                crossterm::style::Color::White
            ),
            crossterm::style::SetBackgroundColor(
                crossterm::style::Color::Black
            ),
            crossterm::terminal::Clear(crossterm::terminal::ClearType::All)
        );
        stdout.write_and_flush(buf.as_bytes()).await?;
        self.buffer.resize(new_size);

        Ok(())
    }

    /// Renders the buffer into the screen using the referred terminal.
    pub(crate) async fn render(
        &mut self,
        buf: &mut String,
    ) -> Result<(), Error> {
        let screen_size = self.buffer.size();
        buf.clear();

        let mut colors = Color2::default();
        let mut cursor = Coord2 { x: 0, y: 0 };
        self.render_init_term(buf, colors, cursor)?;

        for &coord in self.buffer.changed.iter() {
            self.render_tile(
                buf,
                &mut colors,
                &mut cursor,
                screen_size,
                coord,
            )?;
        }

        if let Some(mut stdout) = self.data.stdout.try_lock() {
            stdout.write_and_flush(buf.as_bytes()).await?;
        }

        self.buffer.next_tick();

        Ok(())
    }

    /// Initializes terminal state into the buffer before rendering.
    fn render_init_term(
        &self,
        buf: &mut String,
        colors: Color2,
        cursor: Coord2,
    ) -> Result<(), Error> {
        write!(
            buf,
            "{}{}{}",
            crossterm::style::SetForegroundColor(
                colors.foreground.to_crossterm()
            ),
            crossterm::style::SetBackgroundColor(
                colors.background.to_crossterm()
            ),
            crossterm::cursor::MoveTo(
                coord::to_crossterm(cursor.x),
                coord::to_crossterm(cursor.y)
            ),
        )?;

        Ok(())
    }

    /// Renders a single tile in the given coordinate.
    fn render_tile(
        &self,
        buf: &mut String,
        colors: &mut Color2,
        cursor: &mut Coord2,
        screen_size: Coord2,
        coord: Coord2,
    ) -> Result<(), Error> {
        if *cursor != coord {
            write!(
                buf,
                "{}",
                crossterm::cursor::MoveTo(
                    coord::to_crossterm(coord.x),
                    coord::to_crossterm(coord.y)
                )
            )?;
        }
        *cursor = coord;

        let tile = self.get(*cursor);
        if colors.background != tile.colors.background {
            let color = tile.colors.background.to_crossterm();
            write!(buf, "{}", crossterm::style::SetBackgroundColor(color))?;
        }
        if colors.foreground != tile.colors.foreground {
            let color = tile.colors.foreground.to_crossterm();
            write!(buf, "{}", crossterm::style::SetForegroundColor(color))?;
        }
        *colors = tile.colors;

        write!(buf, "{}", tile.grapheme)?;

        if cursor.x <= screen_size.x {
            cursor.x += 1;
        }

        Ok(())
    }
}

/// The renderer loop. Should be called only when setting up a terminal handler.
/// Exits on error or when notified that it should exit.
pub(crate) async fn renderer(shared: &Shared) -> Result<(), Error> {
    let mut interval = time::interval(shared.screen().frame_time);
    let mut buf = String::new();

    loop {
        {
            let _guard = shared.service_guard().await?;
            let mut screen = shared.screen().lock().await;
            screen.render(&mut buf).await?;
        }

        tokio::select! {
            _ = interval.tick() => (),
            _ = shared.screen().subscribe() => break,
        };
    }

    Ok(())
}
