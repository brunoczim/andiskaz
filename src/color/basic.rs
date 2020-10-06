use crate::color::{ApproxBrightness, BadBasicColor, Brightness};
use crossterm::style::Color as CrosstermColor;
use std::{convert::TryFrom, ops::Not};

/// A basic color used by the terminal.
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum BasicColor {
    /// Black.
    Black = 0,
    /// Dark red/red.
    DarkRed = 1,
    /// Dark green/green.
    DarkGreen = 2,
    /// Dark yellow/yellow.
    DarkYellow = 3,
    /// Dark blue/blue.
    DarkBlue = 4,
    /// Dark magenta/magenta.
    DarkMagenta = 5,
    /// Dark cyan/cyan.
    DarkCyan = 6,
    /// Light gray/dark white.
    LightGray = 7,
    /// Dark gray/light black.
    DarkGray = 8,
    /// Light red.
    LightRed = 9,
    /// Light green.
    LightGreen = 10,
    /// Light yellow.
    LightYellow = 11,
    /// Light blue.
    LightBlue = 12,
    /// Light magenta.
    LightMagenta = 13,
    /// Light cyan.
    LightCyan = 14,
    /// White
    White = 15,
}

impl BasicColor {
    /// Translates this color to a crossterm color.
    pub(crate) fn to_crossterm(self) -> CrosstermColor {
        match self {
            BasicColor::Black => CrosstermColor::Black,
            BasicColor::DarkRed => CrosstermColor::DarkRed,
            BasicColor::DarkGreen => CrosstermColor::DarkGreen,
            BasicColor::DarkYellow => CrosstermColor::DarkYellow,
            BasicColor::DarkBlue => CrosstermColor::DarkBlue,
            BasicColor::DarkMagenta => CrosstermColor::DarkMagenta,
            BasicColor::DarkCyan => CrosstermColor::DarkCyan,
            BasicColor::LightGray => CrosstermColor::DarkGrey,
            BasicColor::DarkGray => CrosstermColor::Grey,
            BasicColor::LightRed => CrosstermColor::Red,
            BasicColor::LightGreen => CrosstermColor::Green,
            BasicColor::LightYellow => CrosstermColor::Yellow,
            BasicColor::LightBlue => CrosstermColor::Blue,
            BasicColor::LightMagenta => CrosstermColor::Magenta,
            BasicColor::LightCyan => CrosstermColor::Cyan,
            BasicColor::White => CrosstermColor::White,
        }
    }
}

impl TryFrom<u8> for BasicColor {
    type Error = BadBasicColor;

    fn try_from(code: u8) -> Result<Self, Self::Error> {
        if code == BasicColor::Black as u8 {
            Ok(BasicColor::Black)
        } else if code == BasicColor::DarkRed as u8 {
            Ok(BasicColor::DarkRed)
        } else if code == BasicColor::DarkGreen as u8 {
            Ok(BasicColor::DarkGreen)
        } else if code == BasicColor::DarkYellow as u8 {
            Ok(BasicColor::DarkYellow)
        } else if code == BasicColor::DarkBlue as u8 {
            Ok(BasicColor::DarkBlue)
        } else if code == BasicColor::DarkMagenta as u8 {
            Ok(BasicColor::DarkMagenta)
        } else if code == BasicColor::DarkCyan as u8 {
            Ok(BasicColor::DarkCyan)
        } else if code == BasicColor::LightGray as u8 {
            Ok(BasicColor::LightGray)
        } else if code == BasicColor::DarkGray as u8 {
            Ok(BasicColor::DarkGray)
        } else if code == BasicColor::LightRed as u8 {
            Ok(BasicColor::LightRed)
        } else if code == BasicColor::LightGreen as u8 {
            Ok(BasicColor::LightGreen)
        } else if code == BasicColor::LightYellow as u8 {
            Ok(BasicColor::LightYellow)
        } else if code == BasicColor::LightBlue as u8 {
            Ok(BasicColor::LightBlue)
        } else if code == BasicColor::LightMagenta as u8 {
            Ok(BasicColor::LightMagenta)
        } else if code == BasicColor::LightCyan as u8 {
            Ok(BasicColor::LightCyan)
        } else if code == BasicColor::White as u8 {
            Ok(BasicColor::White)
        } else {
            Err(BadBasicColor { code })
        }
    }
}

impl ApproxBrightness for BasicColor {
    fn approx_brightness(&self) -> Brightness {
        let min = Brightness { level: 0 };
        let max = Brightness { level: u8::max_value() };
        match self {
            BasicColor::Black => min,
            BasicColor::White => max,
            BasicColor::DarkGray => min,
            BasicColor::LightGray => max,
            BasicColor::DarkRed => min,
            BasicColor::LightRed => max,
            BasicColor::DarkGreen => min,
            BasicColor::LightGreen => max,
            BasicColor::DarkYellow => min,
            BasicColor::LightYellow => max,
            BasicColor::DarkBlue => min,
            BasicColor::LightBlue => max,
            BasicColor::DarkMagenta => min,
            BasicColor::LightMagenta => max,
            BasicColor::DarkCyan => min,
            BasicColor::LightCyan => max,
        }
    }

    fn set_approx_brightness(&mut self, brightness: Brightness) {
        let half = Brightness { level: u8::max_value() / 2 };
        let self_white = self.approx_brightness() >= half;
        let other_white = brightness >= half;

        *self = if self_white == other_white { *self } else { !*self };
    }
}

impl Not for BasicColor {
    type Output = Self;

    fn not(self) -> Self::Output {
        match self {
            BasicColor::Black => BasicColor::White,
            BasicColor::White => BasicColor::Black,
            BasicColor::DarkGray => BasicColor::LightGray,
            BasicColor::LightGray => BasicColor::DarkGray,
            BasicColor::DarkRed => BasicColor::LightRed,
            BasicColor::LightRed => BasicColor::DarkRed,
            BasicColor::DarkGreen => BasicColor::LightGreen,
            BasicColor::LightGreen => BasicColor::DarkGreen,
            BasicColor::DarkYellow => BasicColor::LightYellow,
            BasicColor::LightYellow => BasicColor::DarkYellow,
            BasicColor::DarkBlue => BasicColor::LightBlue,
            BasicColor::LightBlue => BasicColor::DarkBlue,
            BasicColor::DarkMagenta => BasicColor::LightMagenta,
            BasicColor::LightMagenta => BasicColor::DarkMagenta,
            BasicColor::DarkCyan => BasicColor::LightCyan,
            BasicColor::LightCyan => BasicColor::DarkCyan,
        }
    }
}
