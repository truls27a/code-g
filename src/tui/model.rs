#[derive(Debug, Clone, PartialEq)]
pub enum TuiMessage {
    User { content: String },
    Assistant { content: String },
    ToolResponse { summary: String, is_error: bool },
}

#[derive(Debug, Clone, PartialEq)]
pub enum TuiStatus {
    Thinking,
    ReadingFile { path: String },
    WritingFile { path: String },
    SearchingFiles { pattern: String },
    EditingFile { path: String },
    ExecutingTool { tool_name: String },
}

impl TuiStatus {
    pub fn to_string(&self) -> String {
        match self {
            TuiStatus::Thinking => "Thinking...".to_string(),
            TuiStatus::ReadingFile { path } => format!("Reading {}...", path),
            TuiStatus::WritingFile { path } => format!("Writing {}...", path),
            TuiStatus::SearchingFiles { pattern } => format!("Searching for '{}'...", pattern),
            TuiStatus::EditingFile { path } => format!("Editing {}...", path),
            TuiStatus::ExecutingTool { tool_name } => format!("Calling tool '{}'", tool_name),
        }
    }
}
