//! This module provides color brightness utilies.

use std::{convert::TryFrom, ops::Not};

/// The brightness of a color.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Brightness {
    /// Level of brightness. The lower the brightness, the darker the color is.
    pub level: u16,
}

impl Brightness {
    /// Minimum brightness (i.e. dark).
    pub fn min() -> Self {
        Self { level: u16::min_value() }
    }

    /// Maximum brightness (i.e. white).
    pub fn max() -> Self {
        Self { level: u16::max_value() }
    }

    /// Spreads this brightness so it becomes distributed along the range of
    /// brightness, assuming the maximum value it could be is the one given
    /// by `soft_max`.
    ///
    /// # Panics
    /// Panics if `self.level > soft_max` (or if conversion is incorrect).
    pub(crate) fn spread(self, soft_max: u16) -> Self {
        assert!(self.level <= soft_max);
        let level = u32::from(self.level);
        let soft_max = u32::from(soft_max);
        let max = u32::from(u16::max_value());
        let converted = (level * max + soft_max / 2 + 1) / soft_max;

        Self { level: u16::try_from(converted).expect("Color brigthness bug") }
    }

    /// Compress this brightness so it becomes a value between zero and the
    /// maximum value, given by `max`.
    ///
    /// # Panics
    /// Panics if conversion is incorrect.
    pub(crate) fn compress(self, soft_max: u16) -> Self {
        let level = u32::from(self.level);
        let soft_max = u32::from(soft_max);
        let max = u32::from(u16::max_value());
        let converted = (level * soft_max + max / 2 + 1) / max;

        Self { level: u16::try_from(converted).expect("Color brigthness bug") }
    }
}

impl Default for Brightness {
    fn default() -> Self {
        Self::max()
    }
}

impl Not for Brightness {
    type Output = Self;

    fn not(self) -> Self::Output {
        Self { level: u16::max_value() - self.level }
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
        Self: Sized,
    {
        self.set_approx_brightness(brightness);
        self
    }
}

/// A channel's data.
#[derive(Debug, Clone, Copy)]
pub struct Channel {
    /// Channel actual value.
    value: u8,
    /// Channel's weight.
    weight: u8,
    /// Computed weighted value.
    weighted: u16,
}

impl Channel {
    /// Creates channel data from value and weight.
    pub fn new(value: u8, weight: u8) -> Self {
        let mut this = Self { value, weight, weighted: 0 };
        this.update_cache();
        this
    }

    /// Channel's value.
    pub fn value(self) -> u8 {
        self.value
    }

    /// Channel's weight.
    #[allow(dead_code)]
    pub fn weight(self) -> u8 {
        self.weight
    }

    /// Computed channel's value applied to the weight.
    fn weighted(self) -> u16 {
        self.weighted
    }

    /// Sets the channel's value and updates computed values.
    fn set_value(&mut self, value: u8) {
        self.value = value;
        self.update_cache();
    }

    /// Updates computed values.
    fn update_cache(&mut self) {
        self.weighted = u16::from(self.value) * u16::from(self.weight);
    }
}

/// A vector of color channels.
#[derive(Debug)]
pub struct ChannelVector<'channels> {
    /// The maximum value of the channels.
    soft_max: u8,
    /// The channels' data.
    channels: &'channels mut [Channel],
    /// Computed total sum of channels' values.
    total_value: u64,
    /// Computed total sum of channels' weights.
    total_weights: u64,
}

impl<'channels> ChannelVector<'channels> {
    /// Creates a new vector given the channels' data and a maximum value for
    /// the channels.
    ///
    /// `soft_max * sum of weights` must fit into an `u16`.
    pub fn new(channels: &'channels mut [Channel], soft_max: u8) -> Self {
        let mut this =
            Self { total_value: 0, channels, soft_max, total_weights: 0 };
        this.update_cache();
        this.total_weights =
            this.channels.iter().map(|chan| chan.weight).map(u64::from).sum();
        assert!(this.total_weights > 0);
        this
    }

    /// Returns the raw brightness of this vector.
    fn raw_brightness(&self) -> u64 {
        (self.total_value + self.total_weights / 2 + 1) / self.total_weights
    }

    /// Updates the vector given an updater and recomputes the total value.
    fn update<F, T>(&mut self, updater: F) -> T
    where
        F: FnOnce(&mut [Channel]) -> T,
    {
        let ret = updater(self.channels);
        self.update_cache();
        ret
    }

    /// Updates cached computed values.
    fn update_cache(&mut self) {
        self.total_value = self
            .channels
            .iter()
            .copied()
            .map(Channel::weighted)
            .map(u64::from)
            .sum();
    }

    /// Sets the brightness when total channels value is `0`.
    fn set_brightness_total_zero(&mut self, level: u64) {
        let res = u8::try_from(level / self.total_weights);
        let value = res.unwrap_or(self.soft_max);
        self.update(|channels| {
            for entry in channels {
                entry.set_value(value);
            }
        });
    }

    /// Sets the brightness when total channels value is not `0`.
    fn set_brightness_total_nonzero(&mut self, level: u64) {
        let new_total = level;
        let total_value = self.total_value;
        let soft_max = self.soft_max;
        self.update(|channels| {
            for entry in channels {
                let lifted = u64::from(entry.value) * new_total;
                let divided = (lifted + total_value / 2 + 1) / total_value;
                let res = u8::try_from(divided);
                entry.set_value(res.unwrap_or(soft_max).min(soft_max));
            }
        });
    }
}

impl<'channels> ApproxBrightness for ChannelVector<'channels> {
    fn approx_brightness(&self) -> Brightness {
        let max = u16::from(self.soft_max);
        let level =
            u16::try_from(self.raw_brightness()).expect("Color brightness bug");
        Brightness { level }.spread(max)
    }

    fn set_approx_brightness(&mut self, brightness: Brightness) {
        let max = u64::from(self.soft_max) * self.total_weights;
        let max = u16::try_from(max).expect("Color brightness bug");
        let level = u64::from(brightness.compress(max).level);
        if self.total_value == 0 {
            self.set_brightness_total_zero(level);
        } else {
            self.set_brightness_total_nonzero(level);
        }
    }
}
