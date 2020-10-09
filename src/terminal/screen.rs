use crate::{
    color::{transform::PairTransformer, Color, Color2},
    coord,
    coord::{Coord, Coord2},
    string::{TermGrapheme, TermString},
    style::Style,
    terminal::{write_and_flush, TermError, Terminal},
};
use std::{collections::BTreeSet, fmt::Write, sync::atomic::Ordering::*};
use tokio::{io, sync::MutexGuard};

/// A [`Tile`] in the terminal, i.e. a single character with foreground and
/// background colors.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct Tile {
    /// Grapheme shown in this [`Tile`].
    pub grapheme: TermGrapheme,
    /// The foreground-background pair of colors.
    pub colors: Color2,
}

/// The (double) buffer of the buffer of a screen.
#[derive(Debug)]
pub struct ScreenBuffer {
    /// Width of the screen.
    width: usize,
    /// Old screen.
    old: Vec<Tile>,
    /// Currently editing screen.
    curr: Vec<Tile>,
    /// List of changed tiles.
    changed: BTreeSet<Coord2>,
}

impl ScreenBuffer {
    /// A blank screen.
    pub fn blank(size: Coord2) -> Self {
        let curr = vec![Tile::default(); coord::to_index(size.y * size.x)];
        let old = curr.clone();
        Self {
            width: coord::to_index(size.x),
            curr,
            old,
            changed: BTreeSet::new(),
        }
    }

    /// Resizes the screen using the given size.
    pub fn resize(&mut self, size: Coord2) {
        let old_size = self.curr.len();
        let new_size = coord::to_index(size.y * size.x);
        let needs_clear = old_size.min(new_size);
        let default_tile = Tile::default();

        self.curr.resize(new_size, default_tile.clone());
        self.old.resize(new_size, default_tile.clone());

        for tile in &mut self.old[.. needs_clear] {
            *tile = default_tile.clone();
        }
        for tile in &mut self.curr[.. needs_clear] {
            *tile = default_tile.clone();
        }

        self.width = coord::to_index(size.x);
        self.changed.clear();
    }

    /// Advances the buffer in one tick. I.e., `old` is discarded, `curr`
    /// becomes both `curr` and `old`.
    pub fn next_tick(&mut self) {
        self.changed.clear();
        let (old, curr) = (&mut self.old, &self.curr);
        old.clone_from(curr);
    }

    /// Size of the buffer in coordinates.
    ///
    /// Must be in sync with [`Terminal::screen_size`].
    pub fn size(&self) -> Coord2 {
        Coord2 {
            y: coord::from_index(
                self.curr.len().checked_div(self.width).unwrap_or(0),
            ),
            x: coord::from_index(self.width),
        }
    }

    /// Makes an index from a coordinate.
    pub fn make_index(&self, point: Coord2) -> Option<usize> {
        let x = coord::to_index(point.x);
        let y = coord::to_index(point.y);
        if x >= self.width || self.curr.len() / self.width <= y {
            None
        } else {
            Some(y * self.width + x % self.width)
        }
    }
}

/// A locked screen terminal with exclusive access to it.
#[derive(Debug)]
pub struct Screen<'terminal> {
    /// Reference to the underlying terminal.
    terminal: &'terminal Terminal,
    /// A locked screen buffer.
    buffer: MutexGuard<'terminal, ScreenBuffer>,
}

impl<'terminal> Screen<'terminal> {
    /// Builds a screen handle.
    pub(crate) fn new(
        terminal: &'terminal Terminal,
        buffer: MutexGuard<'terminal, ScreenBuffer>,
    ) -> Self {
        Self { terminal, buffer }
    }

    /// Returns a terminal to the underlying terminal.
    pub fn terminal(&self) -> &'terminal Terminal {
        self.terminal
    }

    /// Size of the screen. In sync with [`Terminal::screen_size`].
    pub fn size(&self) -> Coord2 {
        self.buffer.size()
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
        let size = self.terminal.screen_size();
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
    // TODO: break in smaller functions.
    pub fn styled_text<P>(
        &mut self,
        tstring: &TermString,
        style: Style<P>,
    ) -> Coord
    where
        P: PairTransformer,
    {
        tdebug!("{}\n", tstring);
        let mut len = tstring.count_graphemes();
        let mut slice = tstring.index(..);
        let screen_size = self.terminal.screen_size();
        let size = style.make_size(screen_size);

        let mut cursor = Coord2 { x: 0, y: style.top_margin };
        let mut is_inside = cursor.y - style.top_margin < size.y;

        while len > 0 && is_inside {
            is_inside = cursor.y - style.top_margin + 1 < size.y;
            let width = coord::to_index(size.x);
            let pos = if width <= slice.len() {
                let mut pos = slice
                    .index(.. size.x as usize)
                    .iter()
                    .rev()
                    .position(|grapheme| grapheme == TermGrapheme::space())
                    .map_or(width, |rev| len - rev);
                if !is_inside {
                    pos -= 1;
                }
                pos
            } else {
                len
            };
            cursor.x = size.x - pos as Coord;
            cursor.x = cursor.x + style.left_margin - style.right_margin;
            cursor.x = cursor.x / style.align_denom * style.align_numer;
            for grapheme in &slice.index(.. pos) {
                self.update(cursor, |tile| {
                    let colors = style.colors.transform_pair(tile.colors);
                    *tile = Tile { grapheme: grapheme.clone(), colors };
                });
                cursor.x += 1;
            }

            if pos != len && !is_inside {
                self.update(cursor, |tile| {
                    let grapheme = TermGrapheme::new_lossy("…");
                    let colors = style.colors.transform_pair(tile.colors);
                    *tile = Tile { grapheme, colors };
                });
            }

            slice = slice.index(pos ..);
            cursor.y += 1;
            len -= pos;
        }
        cursor.y
    }

    /// Triggers the resize of the screen.
    pub(crate) async fn resize(&mut self, size: Coord2) -> io::Result<()> {
        let mut stdout = self.terminal.shared.stdout.lock().await;
        let buf = format!(
            "{}",
            crossterm::terminal::Clear(crossterm::terminal::ClearType::All)
        );
        write_and_flush(buf.as_bytes(), &mut stdout).await?;
        self.buffer.resize(size);

        self.terminal
            .shared
            .screen_size
            .store(size.x as u32 | (size.y as u32) << 16, Release);
        Ok(())
    }

    /// Renders the buffer into the screen using the referred terminal.
    // TODO: break in smaller functions.
    pub(crate) async fn render(
        &mut self,
        buf: &mut String,
    ) -> Result<(), TermError> {
        let screen_size = self.terminal.screen_size();
        buf.clear();

        let mut cursor = Coord2 { x: 0, y: 0 };
        let mut colors = Color2::default();
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

        for &coord in self.buffer.changed.iter() {
            if cursor != coord {
                write!(
                    buf,
                    "{}",
                    crossterm::cursor::MoveTo(
                        coord::to_crossterm(coord.x),
                        coord::to_crossterm(coord.y)
                    )
                )?;
            }
            cursor = coord;

            let tile = self.get(cursor);
            if colors.background != tile.colors.background {
                let color = tile.colors.background.to_crossterm();
                write!(buf, "{}", crossterm::style::SetBackgroundColor(color))?;
            }
            if colors.foreground != tile.colors.foreground {
                let color = tile.colors.foreground.to_crossterm();
                write!(buf, "{}", crossterm::style::SetForegroundColor(color))?;
            }
            colors = tile.colors;

            write!(buf, "{}", tile.grapheme)?;

            if cursor.x <= screen_size.x {
                cursor.x += 1;
            }
        }

        let stdout = &mut self.terminal.shared.stdout.lock().await;
        write_and_flush(buf.as_bytes(), stdout).await?;
        self.buffer.next_tick();

        Ok(())
    }
}
