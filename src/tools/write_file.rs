use crate::openai::model::{Parameters, Property};
use crate::tools::tool::Tool;
use std::collections::HashMap;
use std::fs;

/// A tool for writing content to files in the filesystem.
///
/// WriteFile provides functionality to create new files or overwrite existing files
/// with specified content. It implements the [`Tool`] trait to be used within the
/// tool system for file manipulation operations.
///
/// # Examples
///
/// ```rust, no_run
/// use code_g::tools::write_file::WriteFile;
/// use code_g::tools::tool::Tool;
/// use std::collections::HashMap;
///
/// let tool = WriteFile;
/// let args = HashMap::from([
///     ("path".to_string(), "example.txt".to_string()),
///     ("content".to_string(), "Hello, world!".to_string()),
/// ]);
/// let result = tool.call(args);
/// ```
/// 
/// # Notes
/// - The tool overwrites the file if it already exists.
/// - The tool creates the file if it does not exist.
pub struct WriteFile;

impl Tool for WriteFile {
    /// Returns the name identifier for this tool.
    ///
    /// # Returns
    ///
    /// A string containing "write_file" as the tool identifier.
    fn name(&self) -> String {
        "write_file".to_string()
    }

    /// Returns a human-readable description of what this tool does.
    ///
    /// # Returns
    ///
    /// A string describing the tool's functionality for writing files.
    fn description(&self) -> String {
        "Write to a file. If the file does not exist, it will be created. If the file exists, it will be overwritten.".to_string()
    }

    /// Returns the parameter schema for this tool.
    ///
    /// Defines the required parameters for the write_file tool: path and content.
    /// Both parameters are required string values.
    ///
    /// # Returns
    ///
    /// A Parameters object containing the schema for path and content arguments.
    fn parameters(&self) -> Parameters {
        Parameters {
            param_type: "object".to_string(),
            properties: HashMap::from([
                (
                    "path".to_string(),
                    Property {
                        prop_type: "string".to_string(),
                        description: "The path to the file to write to".to_string(),
                    },
                ),
                (
                    "content".to_string(),
                    Property {
                        prop_type: "string".to_string(),
                        description: "The content to write to the file".to_string(),
                    },
                ),
            ]),
            required: vec!["path".to_string(), "content".to_string()],
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

    /// Executes the write file operation with the provided arguments.
    ///
    /// Creates a new file or overwrites an existing file at the specified path
    /// with the given content. The operation will create any necessary parent
    /// directories if they don't exist.
    ///
    /// # Arguments
    ///
    /// * `args` - A HashMap containing "path" and "content" string values.
    ///
    /// # Returns
    ///
    /// A success message indicating the file was written successfully.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The "path" argument is missing
    /// - The "content" argument is missing  
    /// - The file cannot be written due to permissions or other I/O errors
    fn call(&self, args: HashMap<String, String>) -> Result<String, String> {
        let path = args.get("path").ok_or("Path is required")?;
        let content = args.get("content").ok_or("Content is required")?;
        match fs::write(path, content) {
            Ok(_) => Ok(format!("File '{}' written successfully", path)),
            Err(e) => match e.kind() {
                std::io::ErrorKind::NotFound => Err(format!("File '{}' not found", path)),
                _ => Err(format!("Error writing file: '{}': {}", path, e)),
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn call_writes_to_file() {
        let path = "call_writes_to_file_tmp_file.txt";
        let content = "Hello, world!";
        let tool = WriteFile;

        let result = tool.call(HashMap::from([
            ("path".to_string(), path.to_string()),
            ("content".to_string(), content.to_string()),
        ]));

        assert_eq!(
            result.unwrap(),
            format!("File '{}' written successfully", path)
        );

        let read_result = fs::read_to_string(path).unwrap();
        assert_eq!(read_result, content);

        fs::remove_file(path).unwrap();
    }

    #[test]
    fn call_returns_error_when_path_is_not_provided() {
        let tool = WriteFile;

        let result = tool.call(HashMap::from([(
            "content".to_string(),
            "Hello, world!".to_string(),
        )]));

        assert!(result.is_err());
    }

    #[test]
    fn call_returns_error_when_content_is_not_provided() {
        let tool = WriteFile;

        let result = tool.call(HashMap::from([(
            "path".to_string(),
            "tmp_file.txt".to_string(),
        )]));

        assert!(result.is_err());
    }

    #[test]
    fn call_overwrites_file() {
        let path = "call_overwrites_file_tmp_file.txt";
        fs::write(path, "Hello, world!").unwrap();

        let tool = WriteFile;
        let result = tool.call(HashMap::from([
            ("path".to_string(), path.to_string()),
            ("content".to_string(), "Hej på dig!".to_string()),
        ]));
        assert_eq!(
            result.unwrap(),
            format!("File '{}' written successfully", path)
        );

        let read_result = fs::read_to_string(path).unwrap();
        assert_eq!(read_result, "Hej på dig!");

        fs::remove_file(path).unwrap();
    }
}
