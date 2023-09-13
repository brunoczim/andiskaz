use std::{error::Error, fmt};

/// Error generated when validating a `TermString` or a grapheme
/// ([`TermGrapheme`](crate::string::TermGrapheme)) and the string starts with a
/// diacrtic.
#[derive(Debug, Clone, Default)]
pub struct DiacriticAtStart;

impl Error for DiacriticAtStart {}

impl fmt::Display for DiacriticAtStart {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        write!(fmt, "The given string with a diacritic")
    }
}

/// Error generated when validating a grapheme
/// ([`TermGrapheme`](crate::string::TermGrapheme)) and the string does not
/// containing exactly one grapheme cluster
/// ([`TermGrapheme`](crate::string::TermGrapheme)).
#[derive(Debug, Clone, Default)]
pub struct NotAGrapheme;

impl Error for NotAGrapheme {}

impl fmt::Display for NotAGrapheme {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        write!(fmt, "The given string is not made of only one grapheme cluster",)
    }
}

/// Error generated when validating a `TermString` and the string contains a
/// control byte.
#[derive(Debug, Clone)]
pub struct InvalidControl {
    /// The position in bytes of the invalid character.
    pub position: usize,
}

impl Error for InvalidControl {}

impl fmt::Display for InvalidControl {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        write!(
            fmt,
            "The string contains a control character in position {}",
            self.position
        )
    }
}

/// Possible errors when creating a
/// [`TermGrapheme`](crate::string::TermGrapheme).
#[derive(Debug, Clone)]
pub enum TermGraphemeError {
    /// Invalid control character found in the given input string.
    InvalidControl(InvalidControl),
    /// The given input string starts with a diacritic.
    DiacriticAtStart(DiacriticAtStart),
    /// The given input is not made of only one grapheme
    /// ([`TermGrapheme`](crate::string::TermGrapheme)).
    NotAGrapheme(NotAGrapheme),
}

impl Error for TermGraphemeError {}

impl fmt::Display for TermGraphemeError {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::InvalidControl(error) => write!(fmt, "{}", error),
            Self::DiacriticAtStart(error) => write!(fmt, "{}", error),
            Self::NotAGrapheme(error) => write!(fmt, "{}", error),
        }
    }
}

impl From<InvalidControl> for TermGraphemeError {
    fn from(error: InvalidControl) -> Self {
        TermGraphemeError::InvalidControl(error)
    }
}

impl From<DiacriticAtStart> for TermGraphemeError {
    fn from(error: DiacriticAtStart) -> Self {
        TermGraphemeError::DiacriticAtStart(error)
    }
}

impl From<NotAGrapheme> for TermGraphemeError {
    fn from(error: NotAGrapheme) -> Self {
        TermGraphemeError::NotAGrapheme(error)
    }
}

impl From<TermStringError> for TermGraphemeError {
    fn from(error: TermStringError) -> Self {
        match error {
            TermStringError::InvalidControl(error) => {
                TermGraphemeError::InvalidControl(error)
            },
            TermStringError::DiacriticAtStart(error) => {
                TermGraphemeError::DiacriticAtStart(error)
            },
        }
    }
}

/// Possible errors when creating a
/// [`TermGrapheme`](crate::string::TermGrapheme).
#[derive(Debug, Clone)]
pub enum TermStringError {
    /// Invalid control character found in the given input string.
    InvalidControl(InvalidControl),
    /// The given input string starts with a diacritic.
    DiacriticAtStart(DiacriticAtStart),
}

impl Error for TermStringError {}

impl fmt::Display for TermStringError {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::InvalidControl(error) => write!(fmt, "{}", error),
            Self::DiacriticAtStart(error) => write!(fmt, "{}", error),
        }
    }
}

impl From<InvalidControl> for TermStringError {
    fn from(error: InvalidControl) -> Self {
        TermStringError::InvalidControl(error)
    }
}

impl From<DiacriticAtStart> for TermStringError {
    fn from(error: DiacriticAtStart) -> Self {
        TermStringError::DiacriticAtStart(error)
    }
}
