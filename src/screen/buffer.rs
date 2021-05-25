//! This module defines the screen (double) buffer and related items.

use crate::{color::Color2, coord, coord::Coord2, string::TermGrapheme};
use std::collections::BTreeSet;

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
    /// Whether this has a valid screen size.
    pub valid: bool,
    /// Old screen.
    pub old: Vec<Tile>,
    /// Currently editing screen.
    pub curr: Vec<Tile>,
    /// List of changed tiles.
    pub changed: BTreeSet<Coord2>,
}

impl ScreenBuffer {
    /// A blank screen.
    pub fn blank(size: Coord2) -> Self {
        let curr = vec![Tile::default(); coord::to_index(size.y * size.x)];
        let old = curr.clone();
        Self {
            width: coord::to_index(size.x),
            valid: true,
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
