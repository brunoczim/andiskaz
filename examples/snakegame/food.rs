use crate::{plane::Bounds, snake::Snake};
use andiskaz::{
    coord::Coord2,
    screen::{LockedScreen, Tile},
};
use rand::Rng;

#[derive(Debug)]
pub struct Food {
    pos: Coord2,
    sprite: Tile,
}

impl Food {
    pub fn new(sprite: Tile, snake: &Snake, bounds: Bounds) -> Self {
        Self { pos: Self::gen_pos(snake, bounds), sprite }
    }

    pub fn regenerate(&mut self, snake: &Snake, bounds: Bounds) {
        self.pos = Self::gen_pos(snake, bounds);
    }

    pub fn in_bounds(&self, bounds: Bounds) -> bool {
        bounds.min.x <= self.pos.x
            && self.pos.x <= bounds.max.x
            && bounds.min.y <= self.pos.y
            && self.pos.y <= bounds.max.y
    }

    pub fn pos(&self) -> Coord2 {
        self.pos
    }

    fn gen_pos(snake: &Snake, bounds: Bounds) -> Coord2 {
        loop {
            let mut rng = rand::thread_rng();
            let point = Coord2 {
                x: rng.gen_range(bounds.min.x, bounds.max.x + 1),
                y: rng.gen_range(bounds.min.y, bounds.max.y + 1),
            };

            let valid = !snake.contains(point);

            if valid {
                break point;
            }
        }
    }

    pub fn render(&self, screen: &mut LockedScreen) {
        screen.set(self.pos, self.sprite.clone());
    }
}
