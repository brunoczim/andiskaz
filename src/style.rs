//! This module provides styles for terminal text.

use crate::{
    color::{self, Color2},
    coord::{Coord, Vec2},
};

/// Alignment, margin and other settings for texts.
#[derive(Debug, Clone, Copy, PartialEq, Eq,Hash)]
pub struct Style<C = Color2>
where
    C: color::Updater,
{
    /// Left margin.
    pub left_margin: Coord,
    /// Right margin.
    pub right_margin: Coord,
    /// Top margin.
    pub top_margin: Coord,
    /// Bottom margin.
    pub bottom_margin: Coord,
    /// Minimum width.
    pub min_width: Coord,
    /// Maximum width.
    pub max_width: Coord,
    /// Minimum height.
    pub min_height: Coord,
    /// Maximum height.
    pub max_height: Coord,
    /// Alignment align_numererator.
    pub align_numer: Coord,
    /// Alignment align_denomominator.
    pub align_denom: Coord,
    /// Foreground-background color pair.
    pub colors: C,
}

impl Default for Style {
    fn default() -> Self {
        Self::with_colors(Color2::default())
    }
}

impl<C> Style<C>
where
    C: color::Updater,
{
    /// Creates a style with the given colors.
    pub fn with_colors(colors: C) -> Self {
        Self {
            left_margin: 0,
            right_margin: 0,
            top_margin: 0,
            bottom_margin: 0,
            min_width: 0,
            max_width: Coord::max_value(),
            min_height: 0,
            max_height: Coord::max_value(),
            align_numer: 0,
            align_denom: 1,
            colors,
        }
    }

    /// Updates the style to the given color updater.
    pub fn colors<D>(self, colors: D) -> Style<D>
    where
        D: color::Updater,
    {
        Style {
            left_margin: self.left_margin,
            right_margin: self.right_margin,
            top_margin: self.top_margin,
            bottom_margin: self.bottom_margin,
            min_width: self.min_width,
            max_width: self.max_width,
            min_height: self.min_height,
            max_height: self.max_height,
            align_numer: self.align_numer,
            align_denom: self.align_denom,
            colors,
        }
    }

    /// Sets left margin.
    pub fn left_margin(self, left_margin: Coord) -> Self {
        Self { left_margin, ..self }
    }

    /// Sets right margin.
    pub fn right_margin(self, right_margin: Coord) -> Self {
        Self { right_margin, ..self }
    }

    /// Sets top margin.
    pub fn top_margin(self, top_margin: Coord) -> Self {
        Self { top_margin, ..self }
    }

    /// Sets bottom margin.
    pub fn bottom_margin(self, bottom_margin: Coord) -> Self {
        Self { bottom_margin, ..self }
    }

    /// Sets minimum width.
    pub fn min_width(self, min_width: Coord) -> Self {
        Self { min_width, ..self }
    }

    /// Sets maximum width.
    pub fn max_width(self, max_width: Coord) -> Self {
        Self { max_width, ..self }
    }

    /// Sets minimum height.
    pub fn min_height(self, min_height: Coord) -> Self {
        Self { min_height, ..self }
    }

    /// Sets maximum height.
    pub fn max_height(self, max_height: Coord) -> Self {
        Self { max_height, ..self }
    }

    /// Sets alignment. Numerator and align_denomominator are used such that
    /// `line\[index\] * align_numer / align_denom == screen\[index\]`
    pub fn align(self, align_numer: Coord, align_denom: Coord) -> Self {
        Self { align_numer, align_denom, ..self }
    }

    /// Makes a coordinate pair that contains the margin dimensions that are
    /// "less".
    pub fn make_margin_below(&self) -> Vec2 {
        Vec2 { x: self.left_margin, y: self.top_margin }
    }

    /// Makes a coordinate pair that contains the margin dimensions that are
    /// "greater".
    pub fn make_margin_above(&self) -> Vec2 {
        Vec2 { x: self.right_margin, y: self.bottom_margin }
    }

    /// Makes a coordinate pair that contains the minima sizes.
    pub fn make_min_size(&self) -> Vec2 {
        Vec2 { x: self.min_width, y: self.min_height }
    }

    /// Makes a coordinate pair that contains the maxima sizes.
    pub fn make_max_size(&self) -> Vec2 {
        Vec2 { x: self.max_width, y: self.max_height }
    }

    /// Makes a coordinate pair that contains the actual sizes.
    pub fn make_size(&self, screen_size: Vec2) -> Vec2 {
        Vec2 {
            y: screen_size
                .y
                .saturating_sub(self.make_margin_below().y)
                .saturating_sub(self.make_margin_above().y)
                .min(self.make_max_size().y),
            x: screen_size
                .x
                .saturating_sub(self.make_margin_below().x)
                .saturating_sub(self.make_margin_above().x)
                .min(self.make_max_size().x),
        }
    }
}
