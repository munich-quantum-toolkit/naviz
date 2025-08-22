// Test support utilities (only compiled during tests)
// Shared helpers to reduce duplication across lexer/parser tests.

/// Convert a byte offset into (line, column) 1-based coordinates.
/// Counts Unicode scalar values; advances line on '\n' only.
pub fn byte_offset_to_line_column(text: &str, offset: usize) -> (usize, usize) {
    let mut line = 1;
    let mut column = 1;
    for (i, ch) in text.char_indices() {
        if i >= offset {
            break;
        }
        if ch == '\n' {
            line += 1;
            column = 1;
        } else {
            column += 1;
        }
    }
    (line, column)
}

/// Collect stringified context frames from a winnow ParseError.
pub fn collect_context<I>(
    err: winnow::error::ParseError<I, winnow::error::ContextError>,
) -> Vec<String> {
    err.into_inner().context().map(|c| c.to_string()).collect()
}
