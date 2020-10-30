//! This module provides some raw IO utilites for the terminal.

use crossterm::Command;
use std::{fmt, fmt::Write};
use tokio::{io, io::AsyncWriteExt, sync::MutexGuard};

/// Writes and flushes data to the standard output.
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
