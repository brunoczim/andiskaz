//! This module provides color brightness utilies.

use std::{convert::TryFrom, ops::Not};

/// The brightness of a color.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Brightness {
    /// Level of brightness. The lower the brightness, the darker the color is.
    pub level: u8,
}

impl Brightness {
    /// Spreads this brightness so it becomes distributed along the range of
    /// brightness, assuming the maximum value it could be is the one given
    /// by `max`.
    ///
    /// # Panics
    /// Panics if `self.level > max` (or if conversion is incorrect).
    pub(crate) fn spread(self, max: u8) -> Self {
        assert!(self.level <= max);
        let level = self.level as u16;
        let max = max as u16;
        let max_byte = u8::max_value() as u16;
        let correction = if max % max_byte > level { 1 } else { 0 };
        let converted = level * max_byte / max + correction;

        Self { level: u8::try_from(converted).expect("Color brigthness bug") }
    }

    /// Compress this brightness so it becomes a value between zero and the
    /// maximum value, given by `max`.
    ///
    /// # Panics
    /// Panics if conversion is incorrect.
    pub(crate) fn compress(self, max: u8) -> Self {
        let level = self.level as u16;
        let max = max as u16;
        let max_byte = u8::max_value() as u16;
        let correction = if max % max_byte > level { 1 } else { 0 };
        let converted = (level - correction) * max / max_byte;

        Self { level: u8::try_from(converted).expect("Color brigthness bug") }
    }
}

impl Not for Brightness {
    type Output = Self;

    fn not(self) -> Self::Output {
        Self { level: u8::max_value() - self.level }
    }
}

/// A trait for types that can approximate their brightness.
pub trait ApproxBrightness {
    /// Approximate the brightness of the color.
    fn approx_brightness(&self) -> Brightness;
    /// Set the approximate brightness of the color.
    fn set_approx_brightness(&mut self, brightness: Brightness);

    /// Like [`Self::set_approx_brightness`] but takes and returns `self`
    /// instead of mutating it.
    fn with_approx_brightness(mut self, brightness: Brightness) -> Self
    where
        Self: Copy,
    {
        self.set_approx_brightness(brightness);
        self
    }
}
