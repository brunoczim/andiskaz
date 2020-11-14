use crossterm::ErrorKind as CrosstermError;
use std::{
    error::Error as ErrorTrait,
    fmt,
    num::ParseIntError,
    string::FromUtf8Error,
};
use tokio::{io, task::JoinError};

#[derive(Debug, Clone)]
pub struct RendererOff;

impl fmt::Display for RendererOff {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        fmt.pad("terminal renderer disconnected")
    }
}

impl ErrorTrait for RendererOff {}

#[derive(Debug, Clone)]
pub struct EventsOff;

impl fmt::Display for EventsOff {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        fmt.pad("terminal event listener disconnected")
    }
}

impl ErrorTrait for EventsOff {}

/// An error that may happen when executing a terminal.
#[derive(Debug)]
#[non_exhaustive]
pub enum ErrorKind {
    RendererOff(RendererOff),
    EventsOff(EventsOff),
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
    Custom(Box<dyn ErrorTrait>),
}

impl ErrorKind {
    pub fn as_dyn(&self) -> &(dyn ErrorTrait + 'static) {
        match self {
            ErrorKind::RendererOff(error) => error,
            ErrorKind::EventsOff(error) => error,
            ErrorKind::IO(error) => error,
            ErrorKind::Fmt(error) => error,
            ErrorKind::ParseInt(error) => error,
            ErrorKind::Utf8(error) => error,
            ErrorKind::Join(error) => error,
            ErrorKind::Custom(error) => &**error,
        }
    }

    /// Converts a Crossterm error to Andiskaz error kind.
    pub(crate) fn from_crossterm(error: CrosstermError) -> Self {
        match error {
            CrosstermError::IoError(error) => ErrorKind::IO(error),
            CrosstermError::FmtError(error) => ErrorKind::Fmt(error),
            CrosstermError::Utf8Error(error) => ErrorKind::Utf8(error),
            CrosstermError::ParseIntError(error) => ErrorKind::ParseInt(error),
            error => ErrorKind::Custom(Box::new(error)),
        }
    }
}

impl From<RendererOff> for ErrorKind {
    fn from(error: RendererOff) -> Self {
        ErrorKind::RendererOff(error)
    }
}

impl From<EventsOff> for ErrorKind {
    fn from(error: EventsOff) -> Self {
        ErrorKind::EventsOff(error)
    }
}

impl From<io::Error> for ErrorKind {
    fn from(error: io::Error) -> Self {
        ErrorKind::IO(error)
    }
}

impl From<ParseIntError> for ErrorKind {
    fn from(error: ParseIntError) -> Self {
        ErrorKind::ParseInt(error)
    }
}

impl From<FromUtf8Error> for ErrorKind {
    fn from(error: FromUtf8Error) -> Self {
        ErrorKind::Utf8(error)
    }
}

impl From<fmt::Error> for ErrorKind {
    fn from(error: fmt::Error) -> Self {
        ErrorKind::Fmt(error)
    }
}

impl From<JoinError> for ErrorKind {
    fn from(error: JoinError) -> Self {
        ErrorKind::Join(error)
    }
}

impl From<Box<dyn ErrorTrait>> for ErrorKind {
    fn from(error: Box<dyn ErrorTrait>) -> Self {
        ErrorKind::Custom(error)
    }
}

#[derive(Debug)]
pub struct Error {
    kind: Box<ErrorKind>,
}

impl Error {
    pub fn new(kind: ErrorKind) -> Self {
        Self { kind: Box::new(kind) }
    }

    pub fn as_dyn(&self) -> &(dyn ErrorTrait + 'static) {
        self.kind.as_dyn()
    }

    pub fn kind(&self) -> &ErrorKind {
        &self.kind
    }

    /// Converts a Crossterm error to Andiskaz error.
    pub(crate) fn from_crossterm(error: CrosstermError) -> Self {
        Self::new(ErrorKind::from_crossterm(error))
    }
}

impl fmt::Display for Error {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        write!(fmt, "{}", self.as_dyn())
    }
}

impl ErrorTrait for Error {
    fn source(&self) -> Option<&(dyn ErrorTrait + 'static)> {
        Some(self.as_dyn())
    }
}

impl From<ErrorKind> for Error {
    fn from(kind: ErrorKind) -> Self {
        Self::new(kind)
    }
}

impl From<RendererOff> for Error {
    fn from(error: RendererOff) -> Self {
        Self::new(ErrorKind::from(error))
    }
}

impl From<EventsOff> for Error {
    fn from(error: EventsOff) -> Self {
        Self::new(ErrorKind::from(error))
    }
}

impl From<io::Error> for Error {
    fn from(error: io::Error) -> Self {
        Self::new(ErrorKind::from(error))
    }
}

impl From<ParseIntError> for Error {
    fn from(error: ParseIntError) -> Self {
        Self::new(ErrorKind::from(error))
    }
}

impl From<FromUtf8Error> for Error {
    fn from(error: FromUtf8Error) -> Self {
        Self::new(ErrorKind::from(error))
    }
}

impl From<fmt::Error> for Error {
    fn from(error: fmt::Error) -> Self {
        Self::new(ErrorKind::from(error))
    }
}

impl From<JoinError> for Error {
    fn from(error: JoinError) -> Self {
        Self::new(ErrorKind::from(error))
    }
}

impl From<Box<dyn ErrorTrait>> for Error {
    fn from(error: Box<dyn ErrorTrait>) -> Self {
        Self::new(ErrorKind::from(error))
    }
}
