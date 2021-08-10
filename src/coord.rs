//! Provides types related to the coordinate system.

use std::convert::TryFrom;

#[cfg(feature = "plane")]
use gardiz::coord as gardiz;

/// A single scalar coordinate used in the terminal.
pub type Coord = u16;

/// Convert a crossterm coordinate to a Andiskaz coordinate.
pub(crate) fn from_crossterm(coord: u16) -> Coord {
    Coord::try_from(coord).unwrap_or(Coord::max_value())
}

/// Converts an Andiskaz coordinate to a crossterm coordinate.
pub(crate) fn to_crossterm(coord: Coord) -> u16 {
    u16::try_from(coord).unwrap_or(u16::max_value())
}

/// Convert a crossterm coordinate to a Andiskaz coordinate.
pub(crate) fn from_index(index: usize) -> Coord {
    Coord::try_from(index).unwrap_or(Coord::max_value())
}

/// Converts an Andiskaz coordinate to a crossterm coordinate.
pub(crate) fn to_index(coord: Coord) -> usize {
    usize::try_from(coord).unwrap_or(usize::max_value())
}

#[cfg(feature = "plane")]
/// A coordinate made of two components `x` and `y`. The `x` axis corresponds to
/// its expected meaning. When the feature `plane` is enabled, this is simply an
/// alias to [`gardiz::coord::Vec2`], otherwise it is a dedicated struct.
pub type Vec2 = gardiz::Vec2<Coord>;

/// A coordinate made of two components `x` and `y`. The `x` axis corresponds to
/// its expected meaning. When the feature `plane` is enabled, this is simply an
/// alias to [`gardiz::coord::Vec2`], otherwise it is a dedicated struct.
#[cfg(not(feature = "plane"))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct Vec2 {
    /// The axis that varies up-down-wise. The smallest value of `x` is in the
    /// left.
    pub y: Coord,
    /// The axis that varies left-right-wise. The smallest value of `y` is in
    /// the top.
    pub x: Coord,
}
