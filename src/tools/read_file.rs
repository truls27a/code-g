use crate::client::model::{Parameters, Property};
use crate::tools::tool::Tool;
use std::collections::HashMap;
use std::fs;

/// A tool for reading content from files in the filesystem.
///
/// ReadFile provides functionality to read the entire content of a file as a string.
/// It implements the [`Tool`] trait to be used within the tool system for file
/// reading operations.
///
/// # Examples
///
/// ```rust,no_run
/// use code_g::tools::read_file::ReadFile;
/// use code_g::tools::tool::Tool;
/// use std::collections::HashMap;
///
/// let tool = ReadFile;
/// let args = HashMap::from([
///     ("path".to_string(), "example.txt".to_string()),
/// ]);
/// let result = tool.call(args);
/// ```
///
/// # Notes
/// - The tool reads the entire file content into memory as a string.
///   This is not efficient for large files and can cause issues when passed to the AI.
/// - The file must exist and be readable by the current process.
pub struct ReadFile;

impl Tool for ReadFile {
    /// Returns the name identifier for this tool.
    ///
    /// # Returns
    ///
    /// A string containing "read_file" as the tool identifier.
    fn name(&self) -> String {
        "read_file".to_string()
    }

    /// Returns a human-readable description of what this tool does.
    ///
    /// # Returns
    ///
    /// A string describing the tool's functionality for reading files.
    fn description(&self) -> String {
        "Read the content of a file".to_string()
    }

    /// Returns the parameter schema for this tool.
    ///
    /// Defines the required parameter for the read_file tool: path.
    /// The path parameter is a required string value.
    ///
    /// # Returns
    ///
    /// A Parameters object containing the schema for the path argument.
    fn parameters(&self) -> Parameters {
        Parameters {
            param_type: "object".to_string(),
            properties: HashMap::from([(
                "path".to_string(),
                Property {
                    prop_type: "string".to_string(),
                    description: "The path to the file to read".to_string(),
                },
            )]),
            required: vec!["path".to_string()],
            additional_properties: false,
        }
    }

    /// Returns whether this tool uses strict parameter validation.
    ///
    /// # Returns
    ///
    /// Always returns true, indicating strict parameter validation is enabled.
    fn strict(&self) -> bool {
        true
    }

    /// Returns whether this tool requires user approval before execution.
    ///
    /// # Returns
    ///
    /// Always returns false, as reading files is a safe operation.
    fn requires_approval(&self) -> bool {
        false
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
        (
            "Read File".to_string(),
            format!("File: {}", path),
        )
    }

    /// Executes the read file operation with the provided arguments.
    ///
    /// Reads the entire content of the file at the specified path and returns
    /// it as a string. The file must exist and be readable by the current process.
    ///
    /// # Arguments
    ///
    /// * `args` - A HashMap containing the "path" string value.
    ///
    /// # Returns
    ///
    /// The entire content of the file as a string.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The "path" argument is missing
    /// - The file does not exist
    /// - The file cannot be read due to permissions or other I/O errors
    /// - The file contains invalid UTF-8 content
    fn call(&self, args: HashMap<String, String>) -> Result<String, String> {
        let path = args.get("path").ok_or("Path is required")?;

        match fs::read_to_string(path) {
            Ok(content) => Ok(content),
            Err(e) => match e.kind() {
                std::io::ErrorKind::NotFound => Err(format!("File '{}' not found", path)),
                _ => Err(format!("Error reading file: '{}': {}", path, e)),
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn call_returns_error_when_path_is_not_provided() {
        let tool = ReadFile;

        let result = tool.call(HashMap::new());

        assert!(result.is_err());
    }
}
