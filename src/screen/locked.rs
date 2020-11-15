use crate::{
    color::{transform::PairTransformer, Color, Color2},
    coord,
    coord::{Coord, Coord2},
    error::Error,
    screen::{
        buffer::{ScreenBuffer, Tile},
        Screen,
    },
    stdio::LockedStdout,
    string::{TermGrapheme, TermString},
    style::Style,
};
use std::fmt::Write;
use tokio::{io, sync::MutexGuard};

/// A locked screen terminal with exclusive access to it.
#[derive(Debug)]
pub struct LockedScreen<'screen> {
    screen: &'screen Screen,
    buffer: MutexGuard<'screen, ScreenBuffer>,
}

impl<'screen> LockedScreen<'screen> {
    /// Size of the screen. In sync with [`Terminal::screen_size`].
    pub fn size(&self) -> Coord2 {
        self.buffer.size()
    }

    pub fn min_size(&self) -> Coord2 {
        self.screen.min_size()
    }

    /// Sets the buffer of a given [`Tile`]. This operation is buffered.
    pub fn set(&mut self, point: Coord2, tile: Tile) {
        self.update(point, |stored| *stored = tile);
    }

    /// Sets the buffer of a given [`Tile`]. This operation is buffered.
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
        let index =
            self.buffer.make_index(point).expect("Screen out of bounds");
        let ret = updater(&mut self.buffer.curr[index]);
        if self.buffer.old[index] != self.buffer.curr[index] {
            self.buffer.changed.insert(point);
        } else {
            self.buffer.changed.remove(&point);
        }
        ret
    }

    /// Gets the buffer of a given [`Tile`] consistently with the buffer.
    pub fn get(&self, point: Coord2) -> &Tile {
        let index =
            self.buffer.make_index(point).expect("Screen out of bounds");
        &self.buffer.curr[index]
    }

    /// Sets every [`Tile`] into a whitespace grapheme with the given colors.
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

    /// Prints a grapheme identifier-encoded text using some style options like
    /// ratio to the screen.
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

    pub(crate) async fn check_resize(
        &mut self,
        new_size: Coord2,
        guard: &mut Option<LockedStdout<'screen>>,
    ) -> io::Result<()> {
        let min_size = self.screen.shared.min_size;
        if new_size.x < min_size.x || new_size.y < min_size.y {
            if guard.is_none() {
                let mut stdout = self.screen.shared.stdout.lock().await;
                self.ask_resize(&mut stdout, min_size).await?;
                *guard = Some(stdout);
            }
        } else {
            let mut stdout = match guard.take() {
                Some(stdout) => stdout,
                None => self.screen.shared.stdout.lock().await,
            };

            self.resize(new_size, &mut stdout).await?;
        }

        Ok(())
    }

    async fn ask_resize(
        &mut self,
        stdout: &mut LockedStdout<'screen>,
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
        stdout: &mut LockedStdout<'screen>,
    ) -> io::Result<()> {
        let buf = format!(
            "{}",
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

        self.screen.shared.stdout.write_and_flush(buf.as_bytes()).await?;
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
