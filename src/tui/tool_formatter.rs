use super::model::TuiStatus;
use std::collections::HashMap;

pub struct ToolFormatter;

impl ToolFormatter {
    pub fn create_status(tool_name: &str, arguments: &HashMap<String, String>) -> TuiStatus {
        match tool_name {
            "read_file" => {
                let path = arguments.get("path").unwrap_or(&"".to_string()).clone();
                TuiStatus::ReadingFile { path }
            }
            "write_file" => {
                let path = arguments.get("path").unwrap_or(&"".to_string()).clone();
                TuiStatus::WritingFile { path }
            }
            "search_files" => {
                let pattern = arguments.get("pattern").unwrap_or(&"".to_string()).clone();
                TuiStatus::SearchingFiles { pattern }
            }
            "edit_file" => {
                let path = arguments.get("path").unwrap_or(&"".to_string()).clone();
                TuiStatus::EditingFile { path }
            }
            _ => TuiStatus::ExecutingTool {
                tool_name: tool_name.to_string(),
            },
        }
    }

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
                _ => format!("Tool '{}' completed", tool_name),
            };
            (summary, false)
        }
    }

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
            _ => content.starts_with("Error"),
        }
    }
}
