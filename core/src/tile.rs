//! This module exports items related to a "tile" in the screen, i.e. a
//! character with foreground and background colors, corresponding to a graphic
//! unit.

use crate::{
    color::{ApproxBrightness, Color, Color2},
    string::TermGrapheme,
};

/// A [`Tile`] in the terminal, i.e. a single character with foreground and
/// background colors.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct Tile {
    /// Grapheme shown in this [`Tile`].
    pub grapheme: TermGrapheme,
    /// The foreground-background pair of colors.
    pub colors: Color2,
}

/// A function that updates a [`Tile`].
pub trait Updater {
    /// Receives a mutable reference to a tile and updates it.
    fn update(self, tile: &mut Tile);
}

impl Updater for Tile {
    fn update(self, tile: &mut Tile) {
        *tile = self;
    }
}

impl<F> Updater for F
where
    F: FnOnce(&mut Tile),
{
    fn update(self, tile: &mut Tile) {
        self(tile)
    }
}

/// Updates a [`Tile`] to set the foreground to the given character and color,
/// but the color will be adapted to contrast the background.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Foreground {
    /// The new displayed grapheme.
    pub grapheme: TermGrapheme,
    /// Foreground color to be contrasted with the background.
    pub color: Color,
}

impl Updater for Foreground {
    fn update(self, tile: &mut Tile) {
        tile.grapheme = self.grapheme;
        tile.colors.foreground = self
            .color
            .with_approx_brightness(!tile.colors.background.approx_brightness())
    }
}

/// Updates a [`Tile`] to set the background to the given color, but the
/// foreground will be adapted to contast the color.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Background {
    /// Background color to be contrasted with the foreground.
    pub color: Color,
}

impl Updater for Background {
    fn update(self, tile: &mut Tile) {
        tile.colors.background = self.color;
        tile.colors.foreground = tile
            .colors
            .foreground
            .with_approx_brightness(!tile.colors.background.approx_brightness())
    }
}
