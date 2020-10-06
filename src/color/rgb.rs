//! This module provides an RGB color utilites.

use crate::color::{ApproxBrightness, Brightness};
use crossterm::style::Color as CrosstermColor;
use std::{convert::TryFrom, ops::Not};

/// Weight of red channel in brightness.
const RED_WEIGHT: u32 = 30;
/// Weight of green channel in brightness.
const GREEN_WEIGHT: u32 = 59;
/// Weight of blue channel in brightness.
const BLUE_WEIGHT: u32 = 11;
/// Total sum of all weights.
const WEIGHT_TOTAL: u32 = RED_WEIGHT + GREEN_WEIGHT + BLUE_WEIGHT;

/// An RGB color ((Red-Green-Blue)). This is an additive color model, where the
/// value of a color channel is how much the channel is added to the color. All
/// channels zeroed are black.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct RgbColor {
    /// The red channel of this RGB color. Higher values means more red to the
    /// color.
    pub red: u8,
    /// The green channel of this RGB color. Higher values means more green to
    /// the color.
    pub green: u8,
    /// The blue channel of this RGB color. Higher values means more blue to
    /// the color.
    pub blue: u8,
}

impl RgbColor {
    /// Translates this color to a crossterm color.
    pub(crate) fn to_crossterm(self) -> CrosstermColor {
        CrosstermColor::Rgb { r: self.red, g: self.green, b: self.blue }
    }
}

impl ApproxBrightness for RgbColor {
    fn approx_brightness(&self) -> Brightness {
        let red = self.red as u32 * RED_WEIGHT;
        let green = self.green as u32 * GREEN_WEIGHT;
        let blue = self.blue as u32 * BLUE_WEIGHT;
        let total = (red + green + blue) / WEIGHT_TOTAL;
        let level = u8::try_from(total).expect("Color brightness bug");

        Brightness { level }
    }

    fn set_approx_brightness(&mut self, brightness: Brightness) {
        let red = self.red as u32 * RED_WEIGHT;
        let green = self.green as u32 * GREEN_WEIGHT;
        let blue = self.blue as u32 * BLUE_WEIGHT;
        let total = red + green + blue;
        let new_total = brightness.level as u32 * WEIGHT_TOTAL;

        let new_red = (red * total / new_total) / RED_WEIGHT;
        let new_blue = (blue * total / new_total) / BLUE_WEIGHT;
        let new_green = (green * total / new_total) / GREEN_WEIGHT;

        *self = Self {
            red: u8::try_from(new_red).expect("Color brightness bug"),
            green: u8::try_from(new_blue).expect("Color brightness bug"),
            blue: u8::try_from(new_green).expect("Color brightness bug"),
        };
    }
}

impl Not for RgbColor {
    type Output = Self;

    fn not(self) -> Self::Output {
        let max = u8::max_value();
        Self {
            red: max - self.red,
            green: max - self.green,
            blue: max - self.blue,
        }
    }
}
