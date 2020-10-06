//! This module provides colors that are usable with the terminal handle
//! implemented by this library.

mod error;
mod brightness;
mod basic;
mod eight_bit;
mod rgb;

pub mod transform;

pub use self::{
    basic::BasicColor,
    brightness::{ApproxBrightness, Brightness},
    eight_bit::{CmyColor, Color8, Color8Kind, GrayColor},
    error::{BadBasicColor, BadCmyColor, BadGrayColor},
    rgb::RgbColor,
};

use crossterm::style::Color as CrosstermColor;
use std::ops::Not;

/// A pair of colors (foreground and background).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Color2 {
    /// The foreground of this pair.
    pub foreground: Color,
    /// The background of this pair.
    pub background: Color,
}

impl Color2 {
    /// Just a convenience method for creating color pairs with conversion.
    pub fn new<F, B>(foreground: F, background: B) -> Self
    where
        F: Into<Color>,
        B: Into<Color>,
    {
        Self { foreground: foreground.into(), background: background.into() }
    }
}

impl Default for Color2 {
    fn default() -> Self {
        Self::new(BasicColor::White, BasicColor::Black)
    }
}

/// A color usable in the terminal.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Color {
    /// A basic color. Totals 16 colors. By far, the most portable color set.
    Basic(BasicColor),
    /// ANSI 8-bit color. Totals 256 colors: 16 basic colors (likely the same
    /// as `Color::Basic`), 216 CMY Colors and 24 gray-scale colors. Not as
    /// portable as `Color::Basic`, but still portable (it's ANSI).
    EightBit(Color8),
    /// RGB color (Red-Green-Blue). Not very portable, but some terminals do
    /// implement it.
    Rgb(RgbColor),
}

impl Color {
    /// Translates this color to a crossterm color.
    pub(crate) fn to_crossterm(self) -> CrosstermColor {
        match self {
            Color::Basic(color) => color.to_crossterm(),
            Color::EightBit(color) => color.to_crossterm(),
            Color::Rgb(color) => color.to_crossterm(),
        }
    }
}

impl ApproxBrightness for Color {
    fn approx_brightness(&self) -> Brightness {
        match self {
            Color::Basic(color) => color.approx_brightness(),
            Color::EightBit(color) => color.approx_brightness(),
            Color::Rgb(color) => color.approx_brightness(),
        }
    }

    fn set_approx_brightness(&mut self, brightness: Brightness) {
        match self {
            Color::Basic(color) => color.set_approx_brightness(brightness),
            Color::EightBit(color) => color.set_approx_brightness(brightness),
            Color::Rgb(color) => color.set_approx_brightness(brightness),
        }
    }
}

impl Not for Color {
    type Output = Self;

    fn not(self) -> Self::Output {
        match self {
            Color::Basic(color) => Color::Basic(!color),
            Color::EightBit(color) => Color::EightBit(!color),
            Color::Rgb(color) => Color::Rgb(!color),
        }
    }
}

impl From<BasicColor> for Color {
    fn from(color: BasicColor) -> Self {
        Color::Basic(color)
    }
}

impl From<Color8> for Color {
    fn from(color: Color8) -> Self {
        Color::EightBit(color)
    }
}

impl From<Color8Kind> for Color {
    fn from(kind: Color8Kind) -> Self {
        Color::from(Color8::from(kind))
    }
}

impl From<CmyColor> for Color {
    fn from(color: CmyColor) -> Self {
        Color::from(Color8::from(color))
    }
}

impl From<GrayColor> for Color {
    fn from(color: GrayColor) -> Self {
        Color::from(Color8::from(color))
    }
}

impl From<RgbColor> for Color {
    fn from(color: RgbColor) -> Self {
        Color::Rgb(color)
    }
}
