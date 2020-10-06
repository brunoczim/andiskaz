//! Provides types related to the coordinate system.

/// A single scalar coordinate used in the terminal.
pub type Coord = u16;

/// A coordinate made of two components `x` and `y`. The `x` axis corresponds to
/// its expected meaning.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct Coord2 {
    /// The axis that varies up-down-wise. The smallest value of `x` is in the
    /// left.
    pub y: Coord,
    /// The axis that varies left-right-wise. The smallest value of `y` is in
    /// the top.
    pub x: Coord,
}
