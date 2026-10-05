//! UTF-8-safe truncation by terminal display width.
use unicode_segmentation::UnicodeSegmentation;
use unicode_width::UnicodeWidthStr;
pub fn truncate(text: &str, width: usize) -> String {
    if text.width() <= width {
        return text.into();
    }
    if width == 0 {
        return String::new();
    }
    let mut result = String::new();
    let mut used = 0;
    for g in text.graphemes(true) {
        if used + g.width() > width - 1 {
            break;
        }
        used += g.width();
        result.push_str(g);
    }
    result.push('…');
    result
}
