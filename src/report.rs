use std::path::Path;

use crate::checker::Violation;

/// Formats a single violation in GCC/Clang-style: `path:line:col: non-ASCII byte 0xXX ('char')`.
#[must_use]
pub fn format_violation(path: &Path, v: &Violation) -> String {
    let char_part = v
        .char_display
        .map_or_else(String::new, |c| format!(" ('{c}')"));
    format!(
        "{}:{}:{}: non-ASCII byte 0x{:02X}{}",
        path.display(),
        v.line,
        v.column,
        v.byte,
        char_part,
    )
}
