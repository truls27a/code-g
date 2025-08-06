use crate::client::model::{Parameters, Property};
use crate::tools::tool::Tool;
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
/// use code_g::tools::tool::Tool;
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
pub struct EditFile;

impl Tool for EditFile {
    /// Returns the name identifier for this tool.
    ///
    /// # Returns
    ///
    /// The string "edit_file" which identifies this tool in the tool registry.
    fn name(&self) -> String {
        "edit_file".to_string()
    }

    /// Returns a human-readable description of what this tool does.
    ///
    /// # Returns
    ///
    /// A description explaining that this tool edits files by replacing strings.
    fn description(&self) -> String {
        "Edit a file by replacing a specific string with new content".to_string()
    }

    /// Returns the parameter schema for this tool.
    ///
    /// Defines the required parameters for the edit_file tool: path, old_string,
    /// and new_string. All parameters are required and must be strings.
    ///
    /// # Returns
    ///
    /// A Parameters struct containing the schema definition for tool arguments.
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

    /// Returns whether this tool uses strict parameter validation.
    ///
    /// # Returns
    ///
    /// Always returns true, indicating that this tool requires strict adherence
    /// to the parameter schema.
    fn strict(&self) -> bool {
        true
    }

    /// Returns whether this tool requires user approval before execution.
    ///
    /// # Returns
    ///
    /// Always returns true, as editing files modifies the filesystem.
    fn requires_approval(&self) -> bool {
        true
    }

    /// Generates the approval message for this tool with the given arguments.
    ///
    /// # Arguments
    ///
    /// * `args` - A HashMap containing the tool arguments as key-value string pairs.
    ///
    /// # Returns
    ///
    /// A tuple containing (operation_name, details) for display to the user.
    fn approval_message(&self, args: &HashMap<String, String>) -> (String, String) {
        let path = args.get("path").map(|s| s.as_str()).unwrap_or("unknown");
        let old_string = args.get("old_string").map(|s| s.as_str()).unwrap_or("");
        let new_string = args.get("new_string").map(|s| s.as_str()).unwrap_or("");
        (
            "Edit File".to_string(),
            format!(
                "File: {}\nReplace: {:?}\nWith: {:?}",
                path, old_string, new_string
            ),
        )
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
