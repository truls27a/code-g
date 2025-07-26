#[derive(Debug, Clone, PartialEq)]
pub enum TuiMessage {
    User { content: String },
    Assistant { content: String },
    ToolResponse { summary: String, is_error: bool },
    PendingChange { change_id: u64, file_path: String, diff: String },
    ChangeAccepted { change_id: u64, accepted_count: usize },
    ChangeDeclined { change_id: u64 },
    ChangeError { change_id: u64, error: String },
}

#[derive(Debug, Clone, PartialEq)]
pub enum TuiStatus {
    Thinking,
    ReadingFile { path: String },
    WritingFile { path: String },
    SearchingFiles { pattern: String },
    EditingFile { path: String },
    ExecutingTool { tool_name: String },
    ProcessingChange { change_id: u64 },
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
            TuiStatus::ProcessingChange { change_id } => format!("Processing change {}...", change_id),
        }
    }
}
