use super::formatter::text::TextFormatter;

/// Build a unified diff for replacing a single occurrence of old_string with new_string,
/// with ANSI colors applied for TUI rendering.
pub fn build_colored_unified_diff(
    path: &str,
    content: &str,
    old_string: &str,
    new_string: &str,
    context: usize,
) -> String {
    let plain = crate::diff::build_unified_diff(path, content, old_string, new_string, context);
    colorize_unified_diff(&plain)
}

/// Build a colored unified diff error preview when file can't be read or inputs missing.
pub fn build_colored_unified_diff_error(
    path: &str,
    message: &str,
    old_string: &str,
    new_string: &str,
) -> String {
    let plain = crate::diff::build_unified_diff_error(path, message, old_string, new_string);
    colorize_unified_diff(&plain)
}

/// Build a colored unified diff representing an overwrite of a file's contents.
pub fn build_colored_unified_diff_overwrite(
    path: &str,
    old_content: &str,
    new_content: &str,
) -> String {
    let plain = crate::diff::build_unified_diff_overwrite(path, old_content, new_content);
    colorize_unified_diff(&plain)
}

/// Apply colors to unified diff text: headers cyan, hunks yellow, removals red, additions green, notes yellow.
pub fn colorize_unified_diff(plain: &str) -> String {
    let mut colored = String::new();
    for line in plain.lines() {
        let c = if line.starts_with("--- ") || line.starts_with("+++ ") {
            TextFormatter::colored_text(line, TextFormatter::cyan())
        } else if line.starts_with("@@") {
            TextFormatter::colored_text(line, TextFormatter::yellow())
        } else if line.starts_with("! ") {
            TextFormatter::colored_text(line, TextFormatter::yellow())
        } else if line.starts_with("- ") || (line.starts_with('-') && !line.starts_with("--- ")) {
            TextFormatter::colored_text(line, TextFormatter::red())
        } else if line.starts_with("+ ") || (line.starts_with('+') && !line.starts_with("+++ ")) {
            TextFormatter::colored_text(line, TextFormatter::green())
        } else {
            line.to_string()
        };
        colored.push_str(&c);
        colored.push('\n');
    }
    colored
}
