use std::cmp::min;

/// Utilities to build unified diffs for file previews.
///
/// This module is UI-agnostic. It does not colorize; callers can color
/// their output as desired.

/// Build a unified diff for replacing a single occurrence of old_string with new_string.
///
/// - Adds file headers (---/+++)
/// - Adds a standard hunk header with both old and new ranges
/// - Includes up to `context` lines before and after
pub fn build_unified_diff(
    path: &str,
    content: &str,
    old_string: &str,
    new_string: &str,
    context: usize,
) -> String {
    let mut diff = String::new();
    diff.push_str(&format!("--- {}\n", path));
    diff.push_str(&format!("+++ {}\n", path));

    let occurrences = content.matches(old_string).count();
    if occurrences == 0 {
        diff.push_str("@@ -0,0 +0,0 @@\n");
        diff.push_str("! Note: the specified old_string was not found; the operation will fail.\n");
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

/// Convenience to build a diff when the file could not be read or inputs are missing.
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

/// Build a unified diff representing an overwrite of a file's contents.
/// Prints all old lines as removals and all new lines as additions in a single hunk.
pub fn build_unified_diff_overwrite(path: &str, old_content: &str, new_content: &str) -> String {
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
    // Start at line 1 for both since it's an overwrite preview
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
