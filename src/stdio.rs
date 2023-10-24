//! This module exports utilities related to terminal's standard input and
//! output at a raw level.

use crossterm::Command;
use std::{
    fmt,
    pin::Pin,
    task::{Context, Poll},
};
use tokio::{
    io,
    io::{AsyncWrite, AsyncWriteExt, Stdout as TokioStdout},
    sync::{Mutex, MutexGuard},
};

/// A centralized lock to a standard output handle.
#[derive(Debug)]
pub struct Stdout {
    /// A lock to tokio's stdout.
    inner: Mutex<TokioStdout>,
}

impl Stdout {
    /// Creates a new centralized stdout lock. You should NOT call this
    /// function if there is already an active centralized lock to the Stdout.
    /// You shouldn't also write to stdout through other means, such as
    /// std's `println!`.
    pub fn new() -> Stdout {
        Self { inner: Mutex::new(io::stdout()) }
    }

    /// Locks this centralized lock to the standard output, and acquire a locked
    /// stdout.
    pub async fn lock<'stdout>(&'stdout self) -> LockedStdout<'stdout> {
        LockedStdout { guard: self.inner.lock().await }
    }

    /// Tries to lock this centralized lock to the standard output, without
    /// blocking, and acquire a locked stdout. Returns `None` if lock failed.
    pub fn try_lock(&self) -> Option<LockedStdout> {
        self.inner.try_lock().ok().map(|guard| LockedStdout { guard })
    }

    /// Locks the stdout and writes and flushes everything in the given buffer.
    pub async fn write_and_flush(&self, buf: &[u8]) -> Result<(), io::Error> {
        self.lock().await.write_and_flush(buf).await
    }
}

/// A locked standard output handle.
#[derive(Debug)]
pub struct LockedStdout<'stdout> {
    /// Guard of tokio's Mutex.
    guard: MutexGuard<'stdout, TokioStdout>,
}

impl<'stdout> LockedStdout<'stdout> {
    pub async fn write_and_flush(
        &mut self,
        buf: &[u8],
    ) -> Result<(), io::Error> {
        self.write_all(buf).await?;
        self.flush().await?;
        Ok(())
    }
}

impl<'stdout> AsyncWrite for LockedStdout<'stdout> {
    fn poll_write(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &[u8],
    ) -> Poll<Result<usize, io::Error>> {
        Pin::new(&mut *self.as_mut().guard).poll_write(cx, buf)
    }

    fn poll_flush(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<Result<(), io::Error>> {
        Pin::new(&mut *self.as_mut().guard).as_mut().poll_flush(cx)
    }

    fn poll_shutdown(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<Result<(), io::Error>> {
        Pin::new(&mut *self.as_mut().guard).as_mut().poll_shutdown(cx)
    }
}

/// Saves the screen previous the application.
#[cfg(windows)]
pub fn save_screen(buf: &mut String) -> fmt::Result {
    if crossterm::terminal::EnterAlternateScreen.is_ansi_code_supported() {
        write!(
            buf,
            "{}",
            crossterm::terminal::EnterAlternateScreen.ansi_code()
        )?;
    }
    Ok(())
}

/// Saves the screen previous the application.
#[cfg(unix)]
pub fn save_screen(buf: &mut String) -> fmt::Result {
    crossterm::terminal::EnterAlternateScreen.write_ansi(buf)?;
    Ok(())
}

/// Restores the screen previous the application.
#[cfg(windows)]
pub fn restore_screen(buf: &mut String) -> fmt::Result {
    if crossterm::terminal::LeaveAlternateScreen.is_ansi_code_supported() {
        crossterm::terminal::LeaveAlternateScreen.write_ansi(buf)?;
    }
    Ok(())
}

/// Restores the screen previous the application.
#[cfg(unix)]
pub fn restore_screen(buf: &mut String) -> fmt::Result {
    crossterm::terminal::LeaveAlternateScreen.write_ansi(buf)?;
    Ok(())
}

#[cfg(windows)]
/// Best-effort function to restore the terminal in a panic.
pub fn emergency_restore() {
    let _ = crossterm::terminal::disable_raw_mode();
    print!("{}", crossterm::cursor::Show);
    print!(
        "{}",
        crossterm::style::SetBackgroundColor(crossterm::style::Color::Reset)
    );
    print!(
        "{}",
        crossterm::style::SetForegroundColor(crossterm::style::Color::Reset)
    );
    if crossterm::terminal::LeaveAlternateScreen.is_ansi_code_supported() {
        print!("{}", crossterm::terminal::LeaveAlternateScreen.ansi_code());
    }
    println!();
}

#[cfg(unix)]
/// Best-effort function to restore the terminal in a panic.
pub fn emergency_restore() {
    let _ = crossterm::terminal::disable_raw_mode();
    print!("{}", crossterm::cursor::Show);
    print!(
        "{}",
        crossterm::style::SetBackgroundColor(crossterm::style::Color::Reset)
    );
    print!(
        "{}",
        crossterm::style::SetForegroundColor(crossterm::style::Color::Reset)
    );
    let mut buf = String::new();
    let _ = crossterm::terminal::LeaveAlternateScreen.write_ansi(&mut buf);
    println!("{}", buf);
}
