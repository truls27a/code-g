/// The message types for the TUI.
/// Each message is either a user message, an assistant message, or a tool response.
#[derive(Debug, Clone, PartialEq)]
pub enum Message {
    User { content: String },
    Assistant { content: String },
    ToolResponse { summary: String, is_error: bool },
}

/// The status of the TUI.
/// The status is used to display the current status of the TUI.
/// Example: "Thinking...", "Reading file...", "Writing file...", "Searching files...", "Editing file...", "Executing tool...".
#[derive(Debug, Clone, PartialEq)]
pub enum Status {
    Thinking,
    ReadingFile { path: String },
    WritingFile { path: String },
    SearchingFiles { pattern: String },
    EditingFile { path: String },
    ExecutingTool { tool_name: String },
}

impl Status {
    /// Convert the status to a string.
    /// Example: Thinking -> "Thinking..."
    pub fn to_string(&self) -> String {
        match self {
            Status::Thinking => "Thinking...".to_string(),
            Status::ReadingFile { path } => format!("Reading {}...", path),
            Status::WritingFile { path } => format!("Writing {}...", path),
            Status::SearchingFiles { pattern } => format!("Searching for '{}'...", pattern),
            Status::EditingFile { path } => format!("Editing {}...", path),
            Status::ExecutingTool { tool_name } => format!("Calling tool '{}'", tool_name),
        }
    }
}
