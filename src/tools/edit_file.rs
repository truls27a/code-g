use crate::client::model::{Parameters, Property};
use crate::tools::traits::Tool;
use crate::tui::model::Status;
use std::collections::HashMap;
use std::fs;

/// A tool for editing files by replacing specific strings with new content.
///
/// This tool provides a safe way to edit files by finding and replacing
/// exact string matches. It ensures that the target string appears exactly
/// once in the file to prevent unintended modifications. The tool can handle
/// both single-line and multi-line string replacements.
///
/// # Examples
///
/// ```rust,no_run
/// use code_g::tools::edit_file::EditFile;
/// use code_g::tools::traits::Tool;
/// use std::collections::HashMap;
///
/// let tool = EditFile;
/// let mut args = HashMap::new();
/// args.insert("path".to_string(), "example.txt".to_string());
/// args.insert("old_string".to_string(), "old text".to_string());
/// args.insert("new_string".to_string(), "new text".to_string());
///
/// let result = tool.call(args);
/// ```
///
/// # Notes
///
/// - The tool will fail if the target string appears multiple times in the file
/// - Empty replacement strings can be used to delete content
/// - The tool preserves file permissions and encoding
#[derive(Clone)]
pub struct EditFile;

impl Tool for EditFile {
    /// Returns the name identifier for the edit file tool.
    fn name(&self) -> String {
        "edit_file".to_string()
    }

    /// Returns a human-readable description of what the edit file tool does.
    fn description(&self) -> String {
        "Edit a file by replacing a specific string with new content".to_string()
    }

    /// Returns the parameter schema for the edit file tool.
    fn parameters(&self) -> Parameters {
        Parameters {
            param_type: "object".to_string(),
            properties: HashMap::from([
                (
                    "path".to_string(),
                    Property {
                        prop_type: "string".to_string(),
                        description: "The path to the file to edit".to_string(),
                    },
                ),
                (
                    "old_string".to_string(),
                    Property {
                        prop_type: "string".to_string(),
                        description: "The string to find and replace in the file".to_string(),
                    },
                ),
                (
                    "new_string".to_string(),
                    Property {
                        prop_type: "string".to_string(),
                        description: "The replacement string".to_string(),
                    },
                ),
            ]),
            required: vec![
                "path".to_string(),
                "old_string".to_string(),
                "new_string".to_string(),
            ],
            additional_properties: false,
        }
    }

    /// The edit file tool uses strict parameter validation.
    fn strict(&self) -> bool {
        true
    }

    /// The edit file tool requires user approval before execution.
    fn requires_approval(&self) -> bool {
        true
    }

    /// Generates the approval message for the edit file tool with the given arguments.
    fn approval_message(&self, args: &HashMap<String, String>) -> String {
        let path = args.get("path").map(|s| s.as_str()).unwrap_or("unknown");

        let old_string = args.get("old_string").map(|s| s.as_str());
        let new_string = args.get("new_string").map(|s| s.as_str());

        // If required args for preview are missing, fall back to simple message
        if old_string.is_none() || new_string.is_none() {
            return format!("CodeG wants to edit file {}", path);
        }

        let old_string = old_string.unwrap();
        let new_string = new_string.unwrap();

        let preview = Self::build_diff_preview(path, old_string, new_string);

        format!(
            "CodeG wants to edit file {path}\n\n{preview}",
            path = path,
            preview = preview
        )
    }

    /// Generates the declined message for the edit file tool with the given arguments.
    fn declined_message(&self, args: &HashMap<String, String>) -> String {
        let path = args.get("path").map(|s| s.as_str()).unwrap_or("unknown");
        format!("Edit file {} was declined by user", path)
    }

    /// Generates the TUI status for the edit file tool with the given arguments.
    fn status(&self, args: &HashMap<String, String>) -> Status {
        let path = args.get("path").map(|s| s.as_str()).unwrap_or("unknown");
        Status::EditingFile {
            path: path.to_string(),
        }
    }

    /// Generates the summary message for the edit file tool with the given arguments.
    fn summary_message(&self, args: &HashMap<String, String>, result: &str) -> String {
        let path = args.get("path").map(|s| s.as_str()).unwrap_or("unknown");
        let lines = result.lines().count();
        format!("Edited {} lines in {}", lines, path)
    }

    /// Executes the file editing operation.
    ///
    /// Reads the specified file, finds the exact occurrence of the old string,
    /// and replaces it with the new string. The operation will fail if the
    /// old string is not found or appears multiple times to prevent unintended
    /// modifications.
    ///
    /// # Arguments
    ///
    /// * `args` - A HashMap containing the required parameters:
    ///   - "path": The file path to edit
    ///   - "old_string": The exact string to find and replace
    ///   - "new_string": The replacement string (can be empty to delete)
    ///
    /// # Returns
    ///
    /// A success message indicating the file was edited successfully.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Required parameters are missing
    /// - The file cannot be read or written
    /// - The old string is not found in the file
    /// - The old string appears multiple times in the file
    fn call(&self, args: HashMap<String, String>) -> Result<String, String> {
        let path = args.get("path").ok_or("Path is required")?;
        let old_string = args.get("old_string").ok_or("Old string is required")?;
        let new_string = args.get("new_string").ok_or("New string is required")?;

        // Read the current file content
        let content = fs::read_to_string(path).map_err(|e| match e.kind() {
            std::io::ErrorKind::NotFound => format!("File '{}' not found", path),
            _ => format!("Error reading file '{}': {}", path, e),
        })?;

        // Check if the old string exists in the file
        if !content.contains(old_string) {
            return Err(format!(
                "String '{}' not found in file '{}'",
                old_string, path
            ));
        }

        // Count occurrences to warn about multiple matches
        let occurrence_count = content.matches(old_string).count();
        if occurrence_count > 1 {
            return Err(format!(
                "String '{}' appears {} times in file '{}'. Please provide a more specific string that appears only once",
                old_string, occurrence_count, path
            ));
        }

        // Replace the string
        let new_content = content.replace(old_string, new_string);

        // Write the modified content back to the file
        fs::write(path, &new_content).map_err(|e| match e.kind() {
            std::io::ErrorKind::NotFound => format!("File '{}' not found", path),
            _ => format!("Error writing to file '{}': {}", path, e),
        })?;

        Ok(format!(
            "Successfully edited file '{}': replaced '{}' with '{}'",
            path, old_string, new_string
        ))
    }
}

impl EditFile {
    fn build_diff_preview(path: &str, old_string: &str, new_string: &str) -> String {
        match fs::read_to_string(path) {
            Ok(content) => {
                // Count occurrences to mirror the tool behavior and inform the user
                let occurrence_count = content.matches(old_string).count();

                if occurrence_count == 0 {
                    return format!(
                        "--- {path}\n+++ {path}\n! Note: the specified old_string was not found; the operation will fail.\n- {old}\n+ {new}\n",
                        path = path,
                        old = old_string,
                        new = new_string
                    );
                }

                if occurrence_count > 1 {
                    return format!(
                        "--- {path}\n+++ {path}\n! Note: the specified old_string appears {n} times; operation requires a unique match.\n- {old}\n+ {new}\n",
                        path = path,
                        n = occurrence_count,
                        old = old_string,
                        new = new_string
                    );
                }

                // Build a compact preview with a small context window
                let idx = content.find(old_string).unwrap_or(0);
                let before = &content[..idx];

                // Determine context lines: capture up to 3 lines before and after
                let context_before = 3usize;
                let context_after = 3usize;

                let lines: Vec<&str> = content.split('\n').collect();

                // Compute line indexes
                let start_line = before.bytes().filter(|&b| b == b'\n').count();
                let old_lines = old_string.split('\n').count();
                let end_line = start_line + old_lines.saturating_sub(1);

                let total_lines = lines.len();
                let hunk_start = start_line.saturating_sub(context_before);
                let hunk_end = usize::min(total_lines.saturating_sub(1), end_line + context_after);

                // Compose unified-like diff
                let mut diff = String::new();
                diff.push_str(&format!("--- {}\n", path));
                diff.push_str(&format!("+++ {}\n", path));
                diff.push_str(&format!(
                    "@@ -{},{} @@\n",
                    start_line + 1,
                    (hunk_end - hunk_start + 1)
                ));

                for i in hunk_start..start_line {
                    diff.push_str(" ");
                    diff.push_str(lines.get(i).unwrap_or(&""));
                    diff.push('\n');
                }

                // Removed block (old_string), split by lines
                for line in old_string.split('\n') {
                    diff.push_str("-");
                    diff.push_str(line);
                    diff.push('\n');
                }

                // Added block (new_string), split by lines
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

                // Truncate overly large previews
                const MAX_PREVIEW_LEN: usize = 8000;
                if diff.len() > MAX_PREVIEW_LEN {
                    let mut truncated = diff;
                    truncated.truncate(MAX_PREVIEW_LEN);
                    truncated.push_str("\n... (diff truncated)\n");
                    truncated
                } else {
                    diff
                }
            }
            Err(e) => format!(
                "--- {path}\n+++ {path}\n! Note: failed to read file for preview: {err}\n- {old}\n+ {new}\n",
                path = path,
                err = e,
                old = old_string,
                new = new_string
            ),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn call_returns_error_when_file_does_not_exist() {
        let tool = EditFile;
        let result = tool.call(HashMap::from([
            ("path".to_string(), "non_existent_file.txt".to_string()),
            ("old_string".to_string(), "test".to_string()),
            ("new_string".to_string(), "replacement".to_string()),
        ]));

        assert!(result.is_err());
    }

    #[test]
    fn call_returns_error_when_required_parameters_missing() {
        let tool = EditFile;
        let result = tool.call(HashMap::from([(
            "path".to_string(),
            "test.txt".to_string(),
        )]));

        assert!(result.is_err());
    }
}
