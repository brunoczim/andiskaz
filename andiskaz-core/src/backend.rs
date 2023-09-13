use std::{error::Error, io::Write};

use async_trait::async_trait;

use crate::{color::Color, coord::Vec2};

#[async_trait]
pub trait Backend: Sized + Write + Clone {
    type Error: Error;

    async fn init(&mut self) -> Result<(), Self::Error>;

    async fn cleanup(&mut self) -> Result<(), Self::Error>;

    async fn save_screen(&mut self) -> Result<(), Self::Error>;

    async fn restore_screen(&mut self) -> Result<(), Self::Error>;

    async fn set_backgroud_color(
        &mut self,
        color: Color,
    ) -> Result<(), Self::Error>;

    async fn set_foregroud_color(
        &mut self,
        color: Color,
    ) -> Result<(), Self::Error>;

    async fn move_cursor(&mut self, position: Vec2) -> Result<(), Self::Error>;
}
