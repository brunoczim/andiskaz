//! This module defines the snake type.

use crate::{
    food::Food,
    plane::{Bounds, Direction},
};
use andiskaz::{
    coord::Coord2,
    screen::{LockedScreen, Tile},
};
use rand::Rng;
use std::collections::VecDeque;

/// The snake of the game.
#[derive(Debug)]
pub struct Snake {
    /// Direction to which the snake is going.
    direction: Direction,
    /// Tile of the snake's body.
    body_tile: Tile,
    /// Tile of the snake's head.
    head_tile: Tile,
    /// Coordinates occupied by the snake.
    segments: VecDeque<Coord2>,
}

impl Snake {
    /// Initializes the snake, given the tiles for its head and body, as well
    /// the bounds of the plane.
    pub fn new(body_tile: Tile, head_tile: Tile, bounds: Bounds) -> Self {
        // Initial length.
        let length = 3;
        // Random head distance from the y-borders.
        let distance = 10;

        // Initializes the random number generator (RNG).
        let mut rng = rand::thread_rng();
        // A random head for the snake.
        let head = Coord2 {
            x: rng.gen_range(bounds.min.x, bounds.max.x + 1),
            y: rng.gen_range(
                bounds.min.y + distance,
                bounds.max.y - (distance + length - 1),
            ),
        };

        // Initializes the segments, in vertical position.
        let mut segments = VecDeque::new();
        for i in 0 .. length {
            segments.push_back(Coord2 { y: head.y + i, ..head });
        }

        Self { direction: Direction::Up, segments, body_tile, head_tile }
    }

    /// Moves the snake in the current direction by 1 step, checks if food will
    /// be eaten.
    ///
    /// Return value:
    /// - `None` means the snake got out of bounds.
    /// - `Some(ate)` means everything is Ok, and `ate` will tell if the fruit
    ///   has been eaten.
    pub fn mov(&mut self, bounds: Bounds, food: &Food) -> Option<bool> {
        let new_head = self.direction.move_coords(self.segments[0], bounds)?;

        if new_head != food.pos() {
            // Only pops last segment if no food was eaten.
            self.segments.pop_back();
        }
        // Pushing new head gives the sensation of movement.
        self.segments.push_front(new_head);

        Some(new_head == food.pos())
    }

    /// Changes the direction to which the snake is going.
    pub fn change_direction(&mut self, direction: Direction) {
        self.direction = direction;
    }

    /// Tests whether the given point is part of the snake's segments (head and
    /// body).
    pub fn contains(&self, point: Coord2) -> bool {
        self.segments.iter().any(|segment| *segment == point)
    }

    /// Tests if the head intersects the rest of the body.
    pub fn head_intersects(&self) -> bool {
        let head = self.segments[0];

        // For all segments other than the first, it must be different from the
        // first.
        self.segments.iter().skip(1).any(|segment| *segment == head)
    }

    /// Tests if all the segments are inside of a bound. Useful when resizing.
    pub fn in_bounds(&self, bounds: Bounds) -> bool {
        // For all segments, they must be contained in the bounds.
        self.segments.iter().all(|point| {
            bounds.min.x <= point.x
                && point.x <= bounds.max.x
                && bounds.min.y <= point.y
                && point.y <= bounds.max.y
        })
    }

    /// Renders the snake.
    pub fn render(&self, screen: &mut LockedScreen) {
        // All the body segments. Although we will initially render the head as
        // a body segment, it's ok. There is no extra IO involved, since the
        // screen is buffered.
        for segment in &self.segments {
            screen.set(*segment, self.body_tile.clone());
        }
        // Finally we render the head's segment.
        screen.set(self.segments[0], self.head_tile.clone());
    }

    /// Returns the length of the snake.
    pub fn length(&self) -> usize {
        self.segments.len()
    }
}
