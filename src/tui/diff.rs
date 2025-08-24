use super::formatter::text::TextFormatter;

/// Unified diff utilities for the Terminal UI.
///
/// Provides helpers to build unified diff strings for previewing changes in the
/// TUI, with both plain (no color) and minimally colorized variants. All
/// functions are pure string builders – they do not perform any filesystem I/O.
///
/// # Examples
///
/// ```rust
/// use code_g::tui::diff::Diff;
///
/// let path = "example.txt";
/// let before = "line 1\nold\nline 3";
/// let after = "line 1\nnew\nline 3";
///
/// // Plain unified diff for a single replacement
/// let plain = Diff::build_unified_diff(path, before, "old", "new", 2);
/// assert!(plain.contains("--- example.txt"));
/// assert!(plain.contains("+++ example.txt"));
/// assert!(plain.contains("-old"));
/// assert!(plain.contains("+new"));
///
/// // Colorized unified diff for displaying in the TUI
/// let colored = Diff::build_colored_unified_diff(path, before, "old", "new", 2);
/// assert!(colored.contains("\u{1b}[91m") || colored.contains("-old")); // red for removals, or plain if stripped
/// ```
pub struct Diff;

impl Diff {
    /// Build a colored unified diff for replacing a single occurrence of `old_string` with `new_string`.
    ///
    /// Produces a minimally colorized unified diff (red for removals, green for additions)
    /// suitable for direct rendering in the TUI. If the `old_string` does not exist or
    /// appears multiple times, a helpful note and a minimal hunk header are included.
    ///
    /// # Arguments
    ///
    /// - `path` - Path label shown in the diff headers (no file I/O is performed).
    /// - `content` - The full original file content.
    /// - `old_string` - The exact string to replace (must appear exactly once for a normal hunk).
    /// - `new_string` - The replacement string.
    /// - `context` - Number of context lines to include before and after the change.
    ///
    /// # Returns
    ///
    /// A unified diff string with color escape sequences for removals/additions.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use code_g::tui::diff::Diff;
    ///
    /// let diff = Diff::build_colored_unified_diff(
    ///     "example.txt",
    ///     "a\nX\nc\n",
    ///     "X",
    ///     "Y",
    ///     1,
    /// );
    /// assert!(diff.contains("--- example.txt"));
    /// assert!(diff.contains("+++ example.txt"));
    /// assert!(diff.contains("-X"));
    /// assert!(diff.contains("+Y"));
    /// ```
    pub fn build_colored_unified_diff(
        path: &str,
        content: &str,
        old_string: &str,
        new_string: &str,
        context: usize,
    ) -> String {
        let plain = Self::build_unified_diff(path, content, old_string, new_string, context);
        Self::colorize_unified_diff(&plain)
    }

    /// Build a colored unified diff error preview when file content can't be obtained.
    ///
    /// Intended for cases where reading the file failed or inputs are incomplete.
    /// It emits a minimal diff with a note line explaining the issue.
    ///
    /// # Arguments
    ///
    /// - `path` - Path label shown in the diff headers.
    /// - `message` - A human-readable error note displayed in the diff body.
    /// - `old_string` - The string that would have been removed.
    /// - `new_string` - The string that would have been added.
    ///
    /// # Returns
    ///
    /// A minimal unified diff with a note, colorized for removals/additions.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use code_g::tui::diff::Diff;
    ///
    /// let diff = Diff::build_colored_unified_diff_error(
    ///     "missing.txt",
    ///     "Note: failed to read file for preview",
    ///     "OLD",
    ///     "NEW",
    /// );
    /// assert!(diff.contains("--- missing.txt"));
    /// assert!(diff.contains("! Note:"));
    /// ```
    pub fn build_colored_unified_diff_error(
        path: &str,
        message: &str,
        old_string: &str,
        new_string: &str,
    ) -> String {
        let plain = Self::build_unified_diff_error(path, message, old_string, new_string);
        Self::colorize_unified_diff(&plain)
    }

    /// Build a colored unified diff representing an overwrite of a file's contents.
    ///
    /// Shows all previous lines as removals and all new lines as additions in a
    /// single hunk. Useful for write operations where the entire file is replaced.
    ///
    /// # Arguments
    ///
    /// - `path` - Path label shown in the diff headers.
    /// - `old_content` - The previous file contents (may be empty if file did not exist).
    /// - `new_content` - The new file contents to be written.
    ///
    /// # Returns
    ///
    /// A unified diff string colorized for removals/additions.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use code_g::tui::diff::Diff;
    ///
    /// let diff = Diff::build_colored_unified_diff_overwrite(
    ///     "out.txt",
    ///     "old line\n",
    ///     "new line\n",
    /// );
    /// assert!(diff.contains("-old line"));
    /// assert!(diff.contains("+new line"));
    /// ```
    pub fn build_colored_unified_diff_overwrite(
        path: &str,
        old_content: &str,
        new_content: &str,
    ) -> String {
        let plain = Self::build_unified_diff_overwrite(path, old_content, new_content);
        Self::colorize_unified_diff(&plain)
    }

    /// Build a plain unified diff (no colors) for a single replacement.
    ///
    /// Emits a standard unified diff with a single hunk that replaces exactly one
    /// occurrence of `old_string` with `new_string`, including up to `context`
    /// lines of surrounding context. If the target string is missing or appears
    /// more than once, a minimal diff with a note is produced instead.
    ///
    /// # Arguments
    ///
    /// - `path` - Path label shown in the diff headers.
    /// - `content` - The full original file content.
    /// - `old_string` - The exact string to replace.
    /// - `new_string` - The replacement string.
    /// - `context` - Number of context lines to include.
    ///
    /// # Returns
    ///
    /// A unified diff string without color escape sequences.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use code_g::tui::diff::Diff;
    ///
    /// let diff = Diff::build_unified_diff("f.txt", "A\nB\nC\n", "B", "X", 1);
    /// assert!(diff.contains("@@"));
    /// assert!(diff.contains("-B"));
    /// assert!(diff.contains("+X"));
    /// ```
    pub fn build_unified_diff(
        path: &str,
        content: &str,
        old_string: &str,
        new_string: &str,
        context: usize,
    ) -> String {
        use std::cmp::min;

        let mut diff = String::new();
        diff.push_str(&format!("--- {}\n", path));
        diff.push_str(&format!("+++ {}\n", path));

        let occurrences = content.matches(old_string).count();
        if occurrences == 0 {
            diff.push_str("@@ -0,0 +0,0 @@\n");
            diff.push_str(
                "! Note: the specified old_string was not found; the operation will fail.\n",
            );
            diff.push_str(&format!("- {}\n+ {}\n", old_string, new_string));
            return diff;
        }
        if occurrences > 1 {
            diff.push_str("@@ -0,0 +0,0 @@\n");
            diff.push_str(&format!(
                "! Note: the specified old_string appears {} times; operation requires a unique match.\n",
                occurrences
            ));
            diff.push_str(&format!("- {}\n+ {}\n", old_string, new_string));
            return diff;
        }

        let idx = content.find(old_string).unwrap_or(0);
        let before = &content[..idx];
        let lines: Vec<&str> = content.split('\n').collect();

        let start_line = before.bytes().filter(|&b| b == b'\n').count();
        let old_lines = old_string.split('\n').count();
        let end_line = start_line + old_lines.saturating_sub(1);

        let total_lines = lines.len();
        let hunk_start = start_line.saturating_sub(context);
        let hunk_end = min(total_lines.saturating_sub(1), end_line + context);

        let count_before = start_line - hunk_start;
        let count_after = hunk_end.saturating_sub(end_line);
        let new_lines_count = new_string.split('\n').count();
        let old_count = count_before + old_lines + count_after;
        let new_count = count_before + new_lines_count + count_after;
        let old_start = hunk_start + 1; // 1-based
        let new_start = hunk_start + 1;

        diff.push_str(&format!(
            "@@ -{},{} +{},{} @@\n",
            old_start, old_count, new_start, new_count
        ));

        for i in hunk_start..start_line {
            diff.push_str(" ");
            diff.push_str(lines.get(i).unwrap_or(&""));
            diff.push('\n');
        }
        for line in old_string.split('\n') {
            diff.push_str("-");
            diff.push_str(line);
            diff.push('\n');
        }
        for line in new_string.split('\n') {
            diff.push_str("+");
            diff.push_str(line);
            diff.push('\n');
        }
        for i in (end_line + 1)..=hunk_end {
            diff.push_str(" ");
            diff.push_str(lines.get(i).unwrap_or(&""));
            diff.push('\n');
        }

        diff
    }

    /// Build a plain unified diff error stub (no colors).
    ///
    /// Produces a minimal diff with a note line when a normal diff cannot be
    /// constructed (e.g., file could not be read or inputs are missing).
    ///
    /// # Arguments
    ///
    /// - `path` - Path label shown in the diff headers.
    /// - `message` - A human-readable error note displayed in the diff body.
    /// - `old_string` - The string that would have been removed.
    /// - `new_string` - The string that would have been added.
    ///
    /// # Returns
    ///
    /// A minimal unified diff string without color escape sequences.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use code_g::tui::diff::Diff;
    ///
    /// let diff = Diff::build_unified_diff_error("x.txt", "Note: error", "OLD", "NEW");
    /// assert!(diff.contains("! Note: error"));
    /// ```
    pub fn build_unified_diff_error(
        path: &str,
        message: &str,
        old_string: &str,
        new_string: &str,
    ) -> String {
        let mut diff = String::new();
        diff.push_str(&format!("--- {}\n", path));
        diff.push_str(&format!("+++ {}\n", path));
        diff.push_str("@@ -0,0 +0,0 @@\n");
        diff.push_str(&format!("! {}\n", message));
        diff.push_str(&format!("- {}\n+ {}\n", old_string, new_string));
        diff
    }

    /// Build a plain overwrite unified diff (no colors).
    ///
    /// Shows all previous lines as removals and all new lines as additions in a
    /// single hunk. Useful for write operations where the entire file is replaced.
    ///
    /// # Arguments
    ///
    /// - `path` - Path label shown in the diff headers.
    /// - `old_content` - The previous file contents (may be empty if file did not exist).
    /// - `new_content` - The new file contents to be written.
    ///
    /// # Returns
    ///
    /// A unified diff string without color escape sequences.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use code_g::tui::diff::Diff;
    ///
    /// let diff = Diff::build_unified_diff_overwrite("f.txt", "old\n", "new\n");
    /// assert!(diff.contains("-old"));
    /// assert!(diff.contains("+new"));
    /// ```
    pub fn build_unified_diff_overwrite(
        path: &str,
        old_content: &str,
        new_content: &str,
    ) -> String {
        let mut diff = String::new();
        diff.push_str(&format!("--- {}\n", path));
        diff.push_str(&format!("+++ {}\n", path));

        let old_count = if old_content.is_empty() {
            0
        } else {
            old_content.split('\n').count()
        };
        let new_count = if new_content.is_empty() {
            0
        } else {
            new_content.split('\n').count()
        };
        diff.push_str(&format!("@@ -1,{} +1,{} @@\n", old_count, new_count));

        if old_count > 0 {
            for line in old_content.split('\n') {
                diff.push_str("-");
                diff.push_str(line);
                diff.push('\n');
            }
        }
        if new_count > 0 {
            for line in new_content.split('\n') {
                diff.push_str("+");
                diff.push_str(line);
                diff.push('\n');
            }
        }

        diff
    }

    /// Apply colors to unified diff text: removals red, additions green; others uncolored.
    fn colorize_unified_diff(plain: &str) -> String {
        let mut colored = String::new();
        for line in plain.lines() {
            let c = if line.starts_with("--- ") || line.starts_with("+++ ") {
                line.to_string()
            } else if line.starts_with("@@") {
                line.to_string()
            } else if line.starts_with("! ") {
                line.to_string()
            } else if line.starts_with("- ") || (line.starts_with('-') && !line.starts_with("--- "))
            {
                TextFormatter::colored_text(line, TextFormatter::red())
            } else if line.starts_with("+ ") || (line.starts_with('+') && !line.starts_with("+++ "))
            {
                TextFormatter::colored_text(line, TextFormatter::green())
            } else {
                line.to_string()
            };
            colored.push_str(&c);
            colored.push('\n');
        }
        colored
    }
}
