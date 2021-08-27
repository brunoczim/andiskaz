//! This module defines the food type.

use crate::{plane::Bounds, snake::Snake};
use andiskaz::{coord::Vec2, screen::Screen, tile::Tile};
use rand::Rng;

/// A food in the game, or a fruit, or whatever our snake eats.
#[derive(Debug)]
pub struct Food {
    /// Position of the food/fruit.
    pos: Vec2,
    /// The tile of a food/fruit.
    tile: Tile,
}

impl Food {
    /// Initializes the food, given its tile, as well the generated snake (for
    /// random generation purposes) and the bounds of the screen.
    pub fn new(tile: Tile, snake: &Snake, bounds: Bounds) -> Self {
        // Initializes with random position.
        Self { pos: Self::gen_pos(snake, bounds), tile }
    }

    /// Generates a new food such that it is in bounds and not in the same place
    /// as the snake.
    pub fn regenerate(&mut self, snake: &Snake, bounds: Bounds) {
        self.pos = Self::gen_pos(snake, bounds);
    }

    /// Tests if the food is inside the bounds. Useful when the screen is
    /// resized.
    pub fn in_bounds(&self, bounds: Bounds) -> bool {
        bounds.min.x <= self.pos.x
            && self.pos.x <= bounds.max.x
            && bounds.min.y <= self.pos.y
            && self.pos.y <= bounds.max.y
    }

    /// Returns the position of the food.
    pub fn pos(&self) -> Vec2 {
        self.pos
    }

    /// Generates a random position for the food, such that it is inside of the
    /// bounds, and it is not at the same place as the snake.
    fn gen_pos(snake: &Snake, bounds: Bounds) -> Vec2 {
        loop {
            // Initializes the random number generator (RNG).
            let mut rng = rand::thread_rng();
            // Generates a random point.
            let point = Vec2 {
                x: rng.gen_range(bounds.min.x, bounds.max.x + 1),
                y: rng.gen_range(bounds.min.y, bounds.max.y + 1),
            };

            let valid = !snake.contains(point);

            if valid {
                // Only stops if the point is not contained by the snake.
                break point;
            }
        }
    }

    /// Renders the food on the screen.
    pub fn render(&self, screen: &mut Screen) {
        screen.set(self.pos, self.tile.clone());
    }
}
