use crossterm::Command;
use std::{
    fmt,
    fmt::Write,
    pin::Pin,
    task::{Context, Poll},
};
use tokio::{
    io,
    io::{AsyncWrite, AsyncWriteExt, Stdout as TokioStdout},
    sync::{Mutex, MutexGuard},
};

#[derive(Debug)]
pub struct Stdout {
    inner: Mutex<TokioStdout>,
}

impl Stdout {
    pub fn new() -> Stdout {
        Self { inner: Mutex::new(io::stdout()) }
    }

    pub async fn lock<'stdout>(&'stdout self) -> LockedStdout<'stdout> {
        LockedStdout { guard: self.inner.lock().await }
    }

    pub async fn write_and_flush(&self, buf: &[u8]) -> Result<(), io::Error> {
        self.lock().await.write_and_flush(buf).await
    }
}

#[derive(Debug)]
pub struct LockedStdout<'stdout> {
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
    write!(buf, "{}", crossterm::terminal::EnterAlternateScreen.ansi_code())?;
    Ok(())
}

/// Restores the screen previous the application.
#[cfg(windows)]
pub fn restore_screen(buf: &mut String) -> fmt::Result {
    if crossterm::terminal::LeaveAlternateScreen.is_ansi_code_supported() {
        write!(
            buf,
            "{}",
            crossterm::terminal::LeaveAlternateScreen.ansi_code()
        )?;
    }
    Ok(())
}

/// Restores the screen previous the application.
#[cfg(unix)]
pub fn restore_screen(buf: &mut String) -> fmt::Result {
    write!(buf, "{}", crossterm::terminal::LeaveAlternateScreen.ansi_code())?;
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
    println!("{}", crossterm::terminal::LeaveAlternateScreen.ansi_code());
}
