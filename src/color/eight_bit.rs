//! This module provides 8-bit color utilies.

use crate::color::{
    brightness::{Channel, ChannelVector},
    ApproxBrightness,
    BadCmyColor,
    BadGrayColor,
    BasicColor,
    Brightness,
};
use crossterm::style::Color as CrosstermColor;
use std::{convert::TryFrom, ops::Not};

/// A CMY (Cyan-Magenta-Yellow) color. The lower one of its component is, the
/// more it subtracts.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct CmyColor {
    /// `(0 .. 216)` Color code.
    code: u8,
}

impl CmyColor {
    /// Base of CMY colors (6).
    pub const BASE: u8 = 6;

    /// Creates a new `CmyColor` given its components. Returns an error if any
    /// of the components is `>= `[`Self::BASE`].
    pub fn try_new(
        cyan: u8,
        magenta: u8,
        yellow: u8,
    ) -> Result<Self, BadCmyColor> {
        if cyan >= Self::BASE || magenta >= Self::BASE || yellow >= Self::BASE {
            Err(BadCmyColor { cyan, magenta, yellow })
        } else {
            Ok(Self {
                code: cyan * Self::BASE.pow(2) + magenta * Self::BASE + yellow,
            })
        }
    }

    /// Creates a new `CmyColor` given its components.
    ///
    /// # Panics
    /// Panics if any of the components is `>= `[`Self::BASE`].
    pub fn new(cyan: u8, magenta: u8, yellow: u8) -> Self {
        Self::try_new(cyan, magenta, yellow).expect("Bad Cmy Color")
    }

    /// The level of cyan component.
    pub const fn cyan(self) -> u8 {
        self.code / Self::BASE / Self::BASE % Self::BASE
    }

    /// The level of magenta component.
    pub const fn magenta(self) -> u8 {
        self.code / Self::BASE % Self::BASE
    }

    /// The level of yellow component.
    pub const fn yellow(self) -> u8 {
        self.code % Self::BASE
    }

    /// The resulting code of the color.
    pub const fn code(self) -> u8 {
        self.code
    }

    /// Sets the cyan component.
    ///
    /// # Panics
    /// Panics if the component is `>= `[`Self::BASE`].
    pub fn set_cyan(self, cyan: u8) -> Self {
        Self::new(cyan, self.magenta(), self.yellow())
    }

    /// Sets the magenta component.
    ///
    /// # Panics
    /// Panics if the component is `>= `[`Self::BASE`].
    pub fn set_magenta(self, magenta: u8) -> Self {
        Self::new(self.cyan(), magenta, self.yellow())
    }

    /// Sets the yellow component.
    ///
    /// # Panics
    /// Panics if the component is `>= `[`Self::BASE`].
    pub fn set_yellow(self, yellow: u8) -> Self {
        Self::new(self.cyan(), self.magenta(), yellow)
    }

    /// Creates a CMY color from the given channels.
    fn from_channels(channels: [Channel; 3]) -> Self {
        Self::new(channels[0].value(), channels[1].value(), channels[2].value())
    }

    /// Returns a CMY color's channels.
    fn channels(self) -> [Channel; 3] {
        [
            Channel::new(self.cyan(), 30),
            Channel::new(self.magenta(), 59),
            Channel::new(self.yellow(), 11),
        ]
    }
}

impl Not for CmyColor {
    type Output = Self;

    fn not(self) -> Self::Output {
        Self::new(
            Self::BASE - self.cyan(),
            Self::BASE - self.magenta(),
            Self::BASE - self.yellow(),
        )
    }
}

impl ApproxBrightness for CmyColor {
    fn approx_brightness(&self) -> Brightness {
        let mut channels = self.channels();
        let vector = ChannelVector::new(&mut channels, Self::BASE - 1);
        vector.approx_brightness()
    }

    fn set_approx_brightness(&mut self, brightness: Brightness) {
        let mut channels = self.channels();
        let mut vector = ChannelVector::new(&mut channels, Self::BASE - 1);
        vector.set_approx_brightness(brightness);
        *self = Self::from_channels(channels);
    }
}

/// A gray-scale color. Goes from white, to gray, to black.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct GrayColor {
    /// Level of white.
    brightness: u8,
}

impl GrayColor {
    /// Minimum gray-scale brightness (0, black).
    pub const MIN: Self = Self { brightness: 0 };
    /// Half of maximum gray-scale brightness (gray).
    pub const HALF: Self = Self { brightness: 12 };
    /// Maximum gray-scale brightness (white).
    pub const MAX: Self = Self { brightness: 23 };

    /// Creates a new gray-scale color given its brightness. Returns an error if
    /// `brightness > MAX`.
    pub fn try_new(brightness: u8) -> Result<Self, BadGrayColor> {
        if brightness > Self::MAX.brightness() {
            Err(BadGrayColor { brightness })
        } else {
            Ok(Self { brightness })
        }
    }

    /// Creates a new gray-scale color given its brightness.
    ///
    /// # Panics
    /// Panics if `brightness > MAX`.
    pub fn new(brightness: u8) -> Self {
        Self::try_new(brightness).expect("Bad gray color")
    }

    /// Returns the brightness of this color.
    pub const fn brightness(self) -> u8 {
        self.brightness
    }
}

impl Not for GrayColor {
    type Output = Self;

    fn not(self) -> Self::Output {
        Self::new(Self::MAX.brightness() + 1 - self.brightness)
    }
}

impl ApproxBrightness for GrayColor {
    fn approx_brightness(&self) -> Brightness {
        let brightness = Brightness { level: u16::from(self.brightness) };
        brightness.spread(u16::from(Self::MAX.brightness))
    }

    fn set_approx_brightness(&mut self, brightness: Brightness) {
        let compressed = brightness.compress(u16::from(Self::MAX.brightness));
        let res = u8::try_from(compressed.level);
        self.brightness = res.expect("Color brightness bug");
    }
}

/// The kind of a color. `enum` representation of an 8-bit color.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Color8Kind {
    /// 16 Basic colors.
    Basic(BasicColor),
    /// 216 CMY colors.
    Cmy(CmyColor),
    /// 24 Gray-scale colors.
    Gray(GrayColor),
}

impl Not for Color8Kind {
    type Output = Self;

    fn not(self) -> Self::Output {
        match self {
            Color8Kind::Basic(color) => Color8Kind::Basic(!color),
            Color8Kind::Cmy(color) => Color8Kind::Cmy(!color),
            Color8Kind::Gray(color) => Color8Kind::Gray(!color),
        }
    }
}

impl From<BasicColor> for Color8Kind {
    fn from(color: BasicColor) -> Self {
        Color8Kind::Basic(color)
    }
}

impl From<CmyColor> for Color8Kind {
    fn from(color: CmyColor) -> Self {
        Color8Kind::Cmy(color)
    }
}

impl From<GrayColor> for Color8Kind {
    fn from(color: GrayColor) -> Self {
        Color8Kind::Gray(color)
    }
}

impl From<Color8> for Color8Kind {
    fn from(color: Color8) -> Self {
        color.kind()
    }
}

impl ApproxBrightness for Color8Kind {
    fn approx_brightness(&self) -> Brightness {
        match self {
            Color8Kind::Basic(color) => color.approx_brightness(),
            Color8Kind::Cmy(color) => color.approx_brightness(),
            Color8Kind::Gray(color) => color.approx_brightness(),
        }
    }

    fn set_approx_brightness(&mut self, brightness: Brightness) {
        match self {
            Color8Kind::Basic(color) => color.set_approx_brightness(brightness),
            Color8Kind::Cmy(color) => color.set_approx_brightness(brightness),
            Color8Kind::Gray(color) => color.set_approx_brightness(brightness),
        }
    }
}

/// An 8-bit encoded color for the terminal.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Color8 {
    code: u8,
}

impl Color8 {
    /// Size of basic colors.
    const BASIC_SIZE: u8 = 16;
    /// Size of basic colors + CMY colors.
    const BASIC_CMY_SIZE: u8 =
        Self::BASIC_SIZE + CmyColor::BASE * CmyColor::BASE * CmyColor::BASE;

    /// Creates a color that is basic.
    pub const fn basic(color: BasicColor) -> Self {
        Self { code: color as u8 }
    }

    /// Creates a color that is CMY.
    pub const fn cmy(color: CmyColor) -> Self {
        Self { code: color.code() + Self::BASIC_SIZE }
    }

    /// Creates a color that is gray-scale.
    pub const fn gray(color: GrayColor) -> Self {
        Self { code: color.brightness() + Self::BASIC_CMY_SIZE }
    }

    /// Returns the color code.
    pub const fn code(self) -> u8 {
        self.code
    }

    /// Converts to en `enum` representation.
    pub fn kind(self) -> Color8Kind {
        if self.code < 16 {
            Color8Kind::Basic(BasicColor::try_from(self.code).unwrap())
        } else if self.code < Self::BASIC_CMY_SIZE {
            Color8Kind::Cmy(CmyColor { code: self.code - Self::BASIC_SIZE })
        } else {
            Color8Kind::Gray(GrayColor {
                brightness: self.code - Self::BASIC_CMY_SIZE,
            })
        }
    }

    /// Translates this color to a crossterm color.
    pub(crate) fn to_crossterm(self) -> CrosstermColor {
        CrosstermColor::AnsiValue(self.code())
    }
}

impl Not for Color8 {
    type Output = Self;

    fn not(self) -> Self::Output {
        Self::from(!self.kind())
    }
}

impl From<BasicColor> for Color8 {
    fn from(color: BasicColor) -> Self {
        Self::basic(color)
    }
}

impl From<CmyColor> for Color8 {
    fn from(color: CmyColor) -> Self {
        Self::cmy(color)
    }
}

impl From<GrayColor> for Color8 {
    fn from(color: GrayColor) -> Self {
        Self::gray(color)
    }
}

impl From<Color8Kind> for Color8 {
    fn from(kind: Color8Kind) -> Self {
        match kind {
            Color8Kind::Basic(color) => Self::from(color),
            Color8Kind::Cmy(color) => Self::from(color),
            Color8Kind::Gray(color) => Self::from(color),
        }
    }
}

impl ApproxBrightness for Color8 {
    fn approx_brightness(&self) -> Brightness {
        self.kind().approx_brightness()
    }

    fn set_approx_brightness(&mut self, brightness: Brightness) {
        *self = Self::from(self.kind().with_approx_brightness(brightness));
    }
}
