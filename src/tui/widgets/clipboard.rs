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
        thread_local! { static CLIPBOARD: std::cell::RefCell<Option<arboard::Clipboard>> = const { std::cell::RefCell::new(None) }; }
        CLIPBOARD.with(|slot| {
            let mut slot = slot.borrow_mut();
            if slot.is_none() {
                *slot = Some(
                    arboard::Clipboard::new()
                        .map_err(|e| ClipboardError::Unavailable(e.to_string()))?,
                );
            }
            slot.as_mut()
                .unwrap()
                .set_text(text.to_string())
                .map_err(|e| ClipboardError::Copy(e.to_string()))
        })
    }
}
