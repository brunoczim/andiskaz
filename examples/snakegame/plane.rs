use andiskaz::coord::Coord2;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Bounds {
    pub min: Coord2,
    pub max: Coord2,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Direction {
    Up,
    Down,
    Left,
    Right,
}

impl Direction {
    pub fn move_coords(self, coords: Coord2, bounds: Bounds) -> Option<Coord2> {
        match self {
            Direction::Up => {
                if coords.y > bounds.min.y {
                    Some(Coord2 { y: coords.y - 1, ..coords })
                } else {
                    None
                }
            },

            Direction::Down => {
                if coords.y < bounds.max.y {
                    Some(Coord2 { y: coords.y + 1, ..coords })
                } else {
                    None
                }
            },

            Direction::Left => {
                if coords.x > bounds.min.x {
                    Some(Coord2 { x: coords.x - 1, ..coords })
                } else {
                    None
                }
            },

            Direction::Right => {
                if coords.x < bounds.max.x {
                    Some(Coord2 { x: coords.x + 1, ..coords })
                } else {
                    None
                }
            },
        }
    }
}
