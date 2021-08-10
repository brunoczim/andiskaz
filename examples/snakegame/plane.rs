//! This module defines utilites related to the 2D plane, the place where our
//! game happens.

use andiskaz::coord::Vec2;

/// Bounds of the plane.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Bounds {
    /// Minimum coordinate for x and y (included).
    pub min: Vec2,
    /// Maximum coordinate for x and y (included).
    pub max: Vec2,
}

/// Direction our snake can take on the plane.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Direction {
    /// up (-Y).
    Up,
    /// down (+Y).
    Down,
    /// left (-Y).
    Left,
    /// right (+Y).
    Right,
}

impl Direction {
    /// Moves a point through this direction. Returns `None` if out of bounds.
    pub fn move_coords(self, coords: Vec2, bounds: Bounds) -> Option<Vec2> {
        match self {
            // (-Y)
            Direction::Up => {
                if coords.y > bounds.min.y {
                    Some(Vec2 { y: coords.y - 1, ..coords })
                } else {
                    None
                }
            },

            // (+Y)
            Direction::Down => {
                if coords.y < bounds.max.y {
                    Some(Vec2 { y: coords.y + 1, ..coords })
                } else {
                    None
                }
            },

            // (-X)
            Direction::Left => {
                if coords.x > bounds.min.x {
                    Some(Vec2 { x: coords.x - 1, ..coords })
                } else {
                    None
                }
            },

            // (+X)
            Direction::Right => {
                if coords.x < bounds.max.x {
                    Some(Vec2 { x: coords.x + 1, ..coords })
                } else {
                    None
                }
            },
        }
    }
}
