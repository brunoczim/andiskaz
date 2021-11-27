//! Provides an interface to system clipboard. Supports Windows, MacOS, Linux
//! (X11), Linux (Wayland).

use crate::error::ClipboardError;

/// Gets the content of the clipboard. May fail if clipboard is not an UTF-8
/// string or for another low-level cause.
pub fn get() -> Result<String, ClipboardError> {
    cli_clipboard::get_contents().map_err(ClipboardError::new)
}

/// Sets the content of the clipboard to the given string. May fail if clipboard
/// is not an UTF-8 string or for another low-level cause.
pub fn set(value: String) -> Result<(), ClipboardError> {
    cli_clipboard::set_contents(value).map_err(ClipboardError::new)
}
