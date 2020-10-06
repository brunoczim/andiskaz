//! This module provides some raw IO utilites for the terminal.

use crossterm::{terminal, Command};
use std::{fmt, fmt::Write};
use tokio::{io, io::AsyncWriteExt, sync::MutexGuard};

pub async fn write_and_flush<'guard>(
    buf: &[u8],
    stdout: &mut MutexGuard<'guard, io::Stdout>,
) -> io::Result<()> {
    stdout.write_all(buf).await?;
    stdout.flush().await?;
    Ok(())
}

/// Saves the screen previous the application.
#[cfg(windows)]
pub fn save_screen(buf: &mut String) -> fmt::Result {
    if terminal::EnterAlternateScreen.is_ansi_code_supported() {
        write!(buf, "{}", terminal::EnterAlternateScreen.ansi_code())?;
    }
    Ok(())
}

/// Saves the screen previous the application.
#[cfg(unix)]
pub fn save_screen(buf: &mut String) -> fmt::Result {
    write!(buf, "{}", terminal::EnterAlternateScreen.ansi_code())?;
    Ok(())
}

/// Restores the screen previous the application.
#[cfg(windows)]
pub fn restore_screen(buf: &mut String) -> fmt::Result {
    if terminal::LeaveAlternateScreen.is_ansi_code_supported() {
        write!(buf, "{}", terminal::LeaveAlternateScreen.ansi_code())?;
    }
    Ok(())
}

/// Restores the screen previous the application.
#[cfg(unix)]
pub fn restore_screen(buf: &mut String) -> fmt::Result {
    write!(buf, "{}", terminal::LeaveAlternateScreen.ansi_code())?;
    Ok(())
}
