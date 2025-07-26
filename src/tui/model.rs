#[derive(Debug, Clone, PartialEq)]
pub enum Message {
    User { content: String },
    Assistant { content: String },
    ToolResponse { summary: String, is_error: bool },
}

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
