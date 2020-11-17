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

#[derive(Debug)]
pub struct Snake {
    direction: Direction,
    body_tile: Tile,
    head_tile: Tile,
    segments: VecDeque<Coord2>,
}

impl Snake {
    pub fn new(body_tile: Tile, head_tile: Tile, bounds: Bounds) -> Self {
        let length = 3;
        let distance = 10;

        let mut rng = rand::thread_rng();
        let head = Coord2 {
            x: rng.gen_range(bounds.min.x, bounds.max.x + 1),
            y: rng.gen_range(
                bounds.min.y + distance,
                bounds.max.y - (distance + length - 1),
            ),
        };

        let mut segments = VecDeque::new();
        for i in 0 .. length {
            segments.push_back(Coord2 { y: head.y + i, ..head });
        }

        Self { direction: Direction::Up, segments, body_tile, head_tile }
    }

    pub fn mov(&mut self, bounds: Bounds, food: &Food) -> Option<bool> {
        let new_head = self.direction.move_coords(self.segments[0], bounds)?;

        if new_head != food.pos() {
            self.segments.pop_back();
        }
        self.segments.push_front(new_head);

        Some(new_head == food.pos())
    }

    pub fn change_direction(&mut self, direction: Direction) {
        self.direction = direction;
    }

    pub fn contains(&self, point: Coord2) -> bool {
        self.segments.iter().any(|segment| *segment == point)
    }

    pub fn head_intersects(&self) -> bool {
        let head = self.segments[0];

        for segment in self.segments.iter().skip(1) {
            if *segment == head {
                return true;
            }
        }

        false
    }

    pub fn in_bounds(&self, bounds: Bounds) -> bool {
        self.segments.iter().all(|point| {
            bounds.min.x <= point.x
                && point.x <= bounds.max.x
                && bounds.min.y <= point.y
                && point.y <= bounds.max.y
        })
    }

    pub fn render(&self, screen: &mut LockedScreen) {
        for segment in &self.segments {
            screen.set(*segment, self.body_tile.clone());
        }

        screen.set(self.segments[0], self.head_tile.clone());
    }

    pub fn length(&self) -> usize {
        self.segments.len()
    }
}
