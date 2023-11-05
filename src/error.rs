//! This module exports error types used by the terminal handles.

use std::{
    error::Error as ErrorTrait,
    fmt,
    num::ParseIntError,
    panic,
    string::FromUtf8Error,
};
use tokio::{io, task::JoinError};

/// Error returned by the terminal handle initialization when there is already
/// an instance of terminal services running.
#[derive(Debug, Clone)]
pub struct AlreadyRunning;

impl fmt::Display for AlreadyRunning {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        fmt.pad("there already is an instance of terminal services running")
    }
}

impl ErrorTrait for AlreadyRunning {}

/// Error returned by the events handler when the listener disconnects.
#[derive(Debug, Clone)]
pub struct ServicesOff;

impl fmt::Display for ServicesOff {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        fmt.pad("terminal event listener and/or screen render disconnected")
    }
}

impl ErrorTrait for ServicesOff {}

/// Error returned when accessing the clipboard fails.
#[cfg(feature = "clipboard")]
#[derive(Debug)]
pub struct ClipboardError {
    inner: anyhow::Error,
}

#[cfg(feature = "clipboard")]
impl ClipboardError {
    pub(crate) fn new<E>(inner: E) -> Self
    where
        E: fmt::Display,
    {
        Self { inner: anyhow::anyhow!("{}", inner) }
    }
}

#[cfg(feature = "clipboard")]
impl fmt::Display for ClipboardError {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        write!(fmt, "Clipboard error: {}", self.inner)
    }
}

/// Error that might happen joining a task.
#[derive(Debug)]
pub struct TaskJoinError {
    /// Actual error.
    inner: JoinError,
}

impl TaskJoinError {
    /// Creates an error from tokio's join error.
    pub(crate) fn new(inner: JoinError) -> Self {
        match inner.try_into_panic() {
            Ok(payload) => panic::resume_unwind(payload),
            Err(inner) => Self { inner },
        }
    }
}

impl fmt::Display for TaskJoinError {
    fn fmt(&self, fmtr: &mut fmt::Formatter) -> fmt::Result {
        fmt::Display::fmt(&self.inner, fmtr)
    }
}

impl ErrorTrait for TaskJoinError {
    fn source(&self) -> Option<&(dyn ErrorTrait + 'static)> {
        Some(&self.inner)
    }
}

#[cfg(feature = "clipboard")]
impl ErrorTrait for ClipboardError {}

/// The kind of an error that may happen when executing a terminal operation.
#[derive(Debug)]
#[non_exhaustive]
pub enum ErrorKind {
    /// Terminal services already running.
    AlreadyRunning(AlreadyRunning),
    /// Event listener and/or renderer disconnected.
    ServicesOff(ServicesOff),
    /// This is an IO error.
    IO(io::Error),
    /// This is a formatting error.
    Fmt(fmt::Error),
    /// This is an error from a bad integer parse attempt.
    ParseInt(ParseIntError),
    /// This is an string from UTF-8 conversion error.
    Utf8(FromUtf8Error),
    /// This is an error from failing to join a task, either because it was
    /// cancelled, or because it panicked.
    TaskJoin(TaskJoinError),
    /// This is an error that may happen accessing clipboard.
    #[cfg(feature = "clipboard")]
    Clipboard(ClipboardError),
    /// A custom error, stored in a trait object.
    Custom(Box<dyn ErrorTrait + Send + Sync>),
}

impl ErrorKind {
    /// Returns this error kind as a trait object.
    pub fn as_dyn(&self) -> &(dyn ErrorTrait + 'static + Send + Sync) {
        match self {
            ErrorKind::AlreadyRunning(error) => error,
            ErrorKind::ServicesOff(error) => error,
            ErrorKind::IO(error) => error,
            ErrorKind::Fmt(error) => error,
            ErrorKind::ParseInt(error) => error,
            ErrorKind::Utf8(error) => error,
            ErrorKind::TaskJoin(error) => error,
            #[cfg(feature = "clipboard")]
            ErrorKind::Clipboard(error) => error,
            ErrorKind::Custom(error) => &**error,
        }
    }
}

impl From<AlreadyRunning> for ErrorKind {
    fn from(error: AlreadyRunning) -> Self {
        ErrorKind::AlreadyRunning(error)
    }
}

impl From<ServicesOff> for ErrorKind {
    fn from(error: ServicesOff) -> Self {
        ErrorKind::ServicesOff(error)
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

impl From<TaskJoinError> for ErrorKind {
    fn from(error: TaskJoinError) -> Self {
        ErrorKind::TaskJoin(error)
    }
}

impl From<fmt::Error> for ErrorKind {
    fn from(error: fmt::Error) -> Self {
        ErrorKind::Fmt(error)
    }
}

#[cfg(feature = "clipboard")]
impl From<ClipboardError> for ErrorKind {
    fn from(error: ClipboardError) -> Self {
        ErrorKind::Clipboard(error)
    }
}

impl From<Box<dyn ErrorTrait + Send + Sync>> for ErrorKind {
    fn from(error: Box<dyn ErrorTrait + Send + Sync>) -> Self {
        ErrorKind::Custom(error)
    }
}

/// An error that may happen when executing an operation on the terminal.
#[derive(Debug)]
pub struct Error {
    /// Error kind. Wrapped in a Box to reduce the stack size.
    kind: Box<ErrorKind>,
}

impl Error {
    /// Creates an error from its kind.
    pub fn new(kind: ErrorKind) -> Self {
        Self { kind: Box::new(kind) }
    }

    /// Returns this error as a trait object.
    pub fn as_dyn(&self) -> &(dyn ErrorTrait + 'static + Send + Sync) {
        self.kind.as_dyn()
    }

    /// Returns the kind of the error.
    pub fn kind(&self) -> &ErrorKind {
        &self.kind
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

impl From<AlreadyRunning> for Error {
    fn from(error: AlreadyRunning) -> Self {
        Self::new(ErrorKind::from(error))
    }
}

impl From<ServicesOff> for Error {
    fn from(error: ServicesOff) -> Self {
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

impl From<TaskJoinError> for Error {
    fn from(error: TaskJoinError) -> Self {
        Self::new(ErrorKind::from(error))
    }
}

#[cfg(feature = "clipboard")]
impl From<ClipboardError> for Error {
    fn from(error: ClipboardError) -> Self {
        Self::new(ErrorKind::from(error))
    }
}

impl From<Box<dyn ErrorTrait + Send + Sync>> for Error {
    fn from(error: Box<dyn ErrorTrait + Send + Sync>) -> Self {
        Self::new(ErrorKind::from(error))
    }
}
