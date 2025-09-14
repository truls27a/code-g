/// The message types for the TUI.
///
/// The `Message` enum is used to store the messages in the TUI state.
/// This enum is used instead of the [`ChatMessage`] enum in the [`openai::model::ChatMessage`]
/// because the TUI does not need to store as much information as the [`ChatSession`].
///
/// # Examples
///
/// ```rust
/// use code_g::tui::model::Message;
///
/// let message = Message::User { content: "Hello, how are you?".to_string() };
/// ```
#[derive(Debug, Clone, PartialEq)]
pub enum Message {
    /// A user message.
    User { content: String },
    /// An assistant message.
    Assistant { content: String },
    /// A tool response. Contains the summary of the tool response and whether it is an error.
    ToolResponse { summary: String, is_error: bool },
}

/// The status of the TUI.
///
/// The `Status` enum is used to store the current status of the TUI,
/// which is in turn used to display the current loading state in the TUI.
///
/// # Examples
///
/// ```rust
/// use code_g::tui::model::Status;
///
/// let status = Status::Thinking;
/// ```
#[derive(Debug, Clone, PartialEq)]
pub enum Status {
    /// The assistant is thinking.
    Thinking,
    /// The assistant is reading a file.
    ReadingFile { path: String },
    /// The assistant is writing a file.
    WritingFile { path: String },
    /// The assistant is searching for files.
    SearchingFiles { pattern: String },
    /// The assistant is editing a file.
    EditingFile { path: String },
    /// The assistant is executing a command.
    ExecutingCommand { command: String },
    /// The assistant is executing a miscellaneous tool.
    ExecutingTool { tool_name: String },
}

impl Status {
    /// Convert the status to a string.
    ///
    /// The `to_string` method is used to format the [`Status`] enum to a string.
    /// This string can then be displayed to the user in the TUI.
    ///
    /// # Returns
    ///
    /// - `String` The string representation of the status
    ///
    /// # Examples
    ///
    /// ```rust
    /// use code_g::tui::model::Status;
    ///
    /// let status = Status::Thinking;
    /// assert_eq!(status.to_string(), "Thinking...");
    /// ```
    pub fn to_string(&self) -> String {
        match self {
            Status::Thinking => "Thinking...".to_string(),
            Status::ReadingFile { path } => format!("Reading {}...", path),
            Status::WritingFile { path } => format!("Writing {}...", path),
            Status::SearchingFiles { pattern } => format!("Searching for '{}'...", pattern),
            Status::EditingFile { path } => format!("Editing {}...", path),
            Status::ExecutingCommand { command } => format!("Executing '{}'...", command),
            Status::ExecutingTool { tool_name } => format!("Calling tool '{}'", tool_name),
        }
    }
}
