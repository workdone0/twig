//! Clipboard wrapper.
//!
//! Thin facade over `arboard` so the rest of the app can stay
//! clipboard-agnostic. On headless systems (no $DISPLAY on Linux,
//! sandboxed macOS, …) `arboard::Clipboard::new()` fails; we capture
//! that into a `ClipboardError` and let the caller surface a status
//! bar message instead of panicking.

#[derive(Debug, thiserror::Error)]
pub enum ClipboardError {
    #[error("clipboard unavailable on this system: {0}")]
    Unavailable(String),
    #[error("failed to copy text: {0}")]
    Copy(String),
}

pub struct Clipboard;

impl Clipboard {
    pub fn copy(text: &str) -> Result<(), ClipboardError> {
        let mut cb = arboard::Clipboard::new()
            .map_err(|e| ClipboardError::Unavailable(e.to_string()))?;
        cb.set_text(text.to_string())
            .map_err(|e| ClipboardError::Copy(e.to_string()))
    }
}