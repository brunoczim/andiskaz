//! This module provides basic color utilies.

use std::{error::Error, fmt};

/// Error returned when a [`BasicColor`](crate::color::BasicColor) is attempted
/// to be created with an invalid code.
#[derive(Debug, Clone, PartialEq)]
pub struct BadBasicColor {
    /// The code given to [`BasicColor`](crate::color::BasicColor).
    pub code: u8,
}

impl fmt::Display for BadBasicColor {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        write!(fmt, "Bad basic color code {}", self.code)
    }
}

impl Error for BadBasicColor {}

/// Error returned when a [`GrayColor`](crate::color::GrayColor) is attempted to
/// be created with an invalid brightness.
#[derive(Debug, Clone, PartialEq)]
pub struct BadGrayColor {
    /// The code given to [`GrayColor`](crate::color::GrayColor).
    pub brightness: u8,
}

impl fmt::Display for BadGrayColor {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        write!(fmt, "Bad gray color brightness {}", self.brightness)
    }
}

impl Error for BadGrayColor {}

/// Error returned when a [`CmyColor`](crate::color::CmyColor) is attempted to
/// be created with invalid channels.
#[derive(Debug, Clone, PartialEq)]
pub struct BadCmyColor {
    /// The cyan channel given to [`CmyColor`](crate::color::CmyColor).
    pub cyan: u8,
    /// The magenta channel given to [`CmyColor`](crate::color::CmyColor).
    pub magenta: u8,
    /// The yellow channel given to [`CmyColor`](crate::color::CmyColor).
    pub yellow: u8,
}

impl fmt::Display for BadCmyColor {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        write!(
            fmt,
            "Bad CMY color, cyan={}, magenta={}, yellow={}",
            self.cyan, self.magenta, self.yellow
        )
    }
}

impl Error for BadCmyColor {}
