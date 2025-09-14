use crate::client::models::{Parameters, Property};
use crate::tools::traits::Tool;
use crate::tui::diff::Diff;
use crate::tui::models::Status;
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

        let preview = match fs::read_to_string(path) {
            Ok(content) => {
                Diff::build_colored_unified_diff(path, &content, old_string, new_string, 3)
            }
            Err(e) => Diff::build_colored_unified_diff_error(
                path,
                &format!("Note: failed to read file for preview: {}", e),
                old_string,
                new_string,
            ),
        };

        format!(
            "CodeG wants to edit {path}\n\n{preview}",
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

// no helper; preview is built inline in approval_message using TUI diff utilities

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
