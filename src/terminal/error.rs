use crossterm::ErrorKind as CrosstermError;
use std::{error::Error, fmt, num::ParseIntError, string::FromUtf8Error};
use tokio::{io, task::JoinError};

/// An error that may happen when executing a terminal.
#[derive(Debug)]
pub enum TermError {
    /// This is an IO error.
    IO(io::Error),
    /// This is a formatting error.
    Fmt(fmt::Error),
    /// This is an error from a bad integer parse attempt.
    ParseInt(ParseIntError),
    /// This is an string from UTF-8 conversion error.
    Utf8(FromUtf8Error),
    /// This is an error from a bad join.
    Join(JoinError),
}

impl TermError {
    /// Converts a Crossterm error to Andiskaz error.
    pub(crate) fn from_crossterm(error: CrosstermError) -> Self {
        match error {
            CrosstermError::IoError(error) => TermError::IO(error),
            CrosstermError::FmtError(error) => TermError::Fmt(error),
            CrosstermError::Utf8Error(error) => TermError::Utf8(error),
            CrosstermError::ParseIntError(error) => TermError::ParseInt(error),
            error => TermError::IO(io::Error::new(io::ErrorKind::Other, error)),
        }
    }
}

impl From<io::Error> for TermError {
    fn from(error: io::Error) -> Self {
        TermError::IO(error)
    }
}

impl From<ParseIntError> for TermError {
    fn from(error: ParseIntError) -> Self {
        TermError::ParseInt(error)
    }
}

impl From<FromUtf8Error> for TermError {
    fn from(error: FromUtf8Error) -> Self {
        TermError::Utf8(error)
    }
}

impl From<fmt::Error> for TermError {
    fn from(error: fmt::Error) -> Self {
        TermError::Fmt(error)
    }
}

impl From<JoinError> for TermError {
    fn from(error: JoinError) -> Self {
        TermError::Join(error)
    }
}

impl fmt::Display for TermError {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        fmt.write_str("terminal: ")?;
        match self {
            TermError::IO(error) => write!(fmt, "{}", error)?,
            TermError::Fmt(error) => write!(fmt, "{}", error)?,
            TermError::ParseInt(error) => write!(fmt, "{}", error)?,
            TermError::Utf8(error) => write!(fmt, "{}", error)?,
            TermError::Join(error) => write!(fmt, "{}", error)?,
        }
        Ok(())
    }
}

impl Error for TermError {}

/// Happens when the event listener fails and disconnects.
#[derive(Debug, Clone, Default)]
pub struct ListenerFailed;

impl fmt::Display for ListenerFailed {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        write!(fmt, "Event listener failed and disconnected")
    }
}

impl Error for ListenerFailed {}
