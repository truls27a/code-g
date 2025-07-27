use crate::tui::model::Status;
use std::collections::HashMap;

/// Formats tool responses and calls for the TUI.
pub struct ToolFormatter;

impl ToolFormatter {
    /// Create status for the TUI based on a tool call.
    pub fn create_status(tool_name: &str, arguments: &HashMap<String, String>) -> Status {
        match tool_name {
            "read_file" => {
                let path = arguments.get("path").unwrap_or(&"".to_string()).clone();
                Status::ReadingFile { path }
            }
            "write_file" => {
                let path = arguments.get("path").unwrap_or(&"".to_string()).clone();
                Status::WritingFile { path }
            }
            "search_files" => {
                let pattern = arguments.get("pattern").unwrap_or(&"".to_string()).clone();
                Status::SearchingFiles { pattern }
            }
            "edit_file" => {
                let path = arguments.get("path").unwrap_or(&"".to_string()).clone();
                Status::EditingFile { path }
            }
            "execute_command" => {
                let command = arguments.get("command").unwrap_or(&"".to_string()).clone();
                Status::ExecutingCommand { command }
            }
            _ => Status::ExecutingTool {
                tool_name: tool_name.to_string(),
            },
        }
    }

    /// Create a summary of a tool response.
    /// The summary is used to display the result of a tool call.
    /// It also checks if the tool response is an error, if it is, the summary is the error message.
    pub fn create_summary(
        content: &str,
        tool_name: &str,
        arguments: HashMap<String, String>,
    ) -> (String, bool) {
        let is_error = Self::is_error(content, tool_name);

        if is_error {
            (content.to_string(), true)
        } else {
            let summary = match tool_name {
                "read_file" => {
                    let path = arguments.get("path").unwrap_or(&"".to_string()).clone();
                    format!("Read {} lines from {}", content.lines().count(), path)
                }
                "write_file" => {
                    let path = arguments.get("path").unwrap_or(&"".to_string()).clone();
                    format!("Wrote {} lines to {}", content.lines().count(), path)
                }
                "search_files" => {
                    let pattern = arguments.get("pattern").unwrap_or(&"".to_string()).clone();
                    format!(
                        "Found {} files matching {}",
                        content.lines().count(),
                        pattern
                    )
                }
                "edit_file" => {
                    let path = arguments.get("path").unwrap_or(&"".to_string()).clone();
                    format!("Edited {} lines in {}", content.lines().count(), path)
                }
                "execute_command" => {
                    let command = arguments.get("command").unwrap_or(&"".to_string()).clone();
                    format!("Executed command '{}'", command)
                }
                _ => format!("Tool '{}' completed", tool_name),
            };
            (summary, false)
        }
    }

    /// Check if a tool response is an error.
    pub fn is_error(content: &str, tool_name: &str) -> bool {
        match tool_name {
            "read_file" => {
                content.starts_with("Error")
                    || content.contains("not found") && content.starts_with("File")
            }
            "write_file" => content.starts_with("Error"),
            "edit_file" => {
                content.starts_with("Error")
                    || content.contains("not found in file")
                    || content.contains("appears") && content.contains("times in file")
            }
            "search_files" => {
                content.starts_with("Error") || content.contains("No files found matching pattern")
            }
            "execute_command" => {
                content.starts_with("Command") && content.contains("failed with exit code")
                    || content.starts_with("Failed to execute command")
                    || content == "Command is required"
            }
            _ => content.starts_with("Error"),
        }
    }
}
