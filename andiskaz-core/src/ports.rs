use std::{error::Error, io::Write};

use async_trait::async_trait;

use crate::{color::Color, coord::Vec2};

pub mod task;
pub mod interior_mut;
pub mod stdio;

#[async_trait]
pub trait Backend {
    type Error: Error;
}
