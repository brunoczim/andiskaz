use std::error::Error;

use async_trait::async_trait;

use crate::{color::Color, coord::Vec2, event::Event};

#[async_trait]
pub trait Screen {
    type Error: Error;

    async fn init(&mut self) -> Result<(), Self::Error>;

    async fn cleanup(&mut self) -> Result<(), Self::Error>;

    async fn set_cursor_pos(&mut self, pos: Vec2) -> Result<(), Self::Error>;

    async fn set_fg_color(&mut self, color: Color) -> Result<(), Self::Error>;

    async fn set_bg_color(&mut self, color: Color) -> Result<(), Self::Error>;

    async fn write(&mut self, string: &str) -> Result<(), Self::Error>;
}

#[async_trait]
pub trait EventStream {
    type Error: Error;

    async fn next(&mut self) -> Result<Option<Event>, Self::Error>;
}
