use super::models::{Message, Status};

/// The state of the TUI.
///
/// Stores the TUI messages and current status. The status is used to display the current loading state.
///
/// # Fields
///
/// - `messages`: [`Vec<Message>`] The messages to display
/// - `current_status`: [`Option<Status>`] The current status of the TUI
///
/// # Examples
///
/// ```rust
/// use code_g::tui::state::TuiState;
///
/// let mut state = TuiState::new();
///
/// state.add_user_message("Hello, how are you?".to_string());
/// state.add_assistant_message("I'm doing well, thank you!".to_string());
/// state.add_tool_response("Tool executed successfully".to_string(), false);
/// ```
#[derive(Debug, Clone)]
pub struct TuiState {
    pub messages: Vec<Message>,
    pub current_status: Option<Status>,
}

impl TuiState {
    /// Create a new TUI state.
    ///
    /// # Returns
    ///
    /// - `TuiState` The new TUI state
    ///
    /// # Examples
    ///
    /// ```rust
    /// use code_g::tui::state::TuiState;
    ///
    /// let state = TuiState::new();
    /// ```
    pub fn new() -> Self {
        Self {
            messages: Vec::new(),
            current_status: None,
        }
    }

    /// Add a user message to the state.
    ///
    /// # Arguments
    ///
    /// - `content`: [`String`] The content of the user message
    ///
    /// # Examples
    ///
    /// ```rust
    /// use code_g::tui::state::TuiState;
    ///
    /// let mut state = TuiState::new();
    /// state.add_user_message("Hello, how are you?".to_string());
    /// ```
    pub fn add_user_message(&mut self, content: String) {
        self.messages.push(Message::User { content });
        self.current_status = None;
    }

    /// Add an assistant message to the state.
    ///
    /// # Arguments
    ///
    /// - `content`: [`String`] The content of the assistant message
    ///
    /// # Examples
    ///
    /// ```rust
    /// use code_g::tui::state::TuiState;
    ///
    /// let mut state = TuiState::new();
    /// state.add_assistant_message("I'm doing well, thank you!".to_string());
    /// ```
    pub fn add_assistant_message(&mut self, content: String) {
        self.messages.push(Message::Assistant { content });
        self.current_status = None;
    }

    /// Add a tool response to the state.
    ///
    /// # Arguments
    ///
    /// - `summary`: [`String`] The summary of the tool response
    /// - `is_error`: [`bool`] Whether the tool response is an error
    ///
    /// # Examples
    ///
    /// ```rust
    /// use code_g::tui::state::TuiState;
    ///
    /// let mut state = TuiState::new();
    /// state.add_tool_response("Read 23 lines from test.txt".to_string(), false);
    /// ```
    pub fn add_tool_response(&mut self, summary: String, is_error: bool) {
        self.messages
            .push(Message::ToolResponse { summary, is_error });
        self.current_status = None;
    }

    /// Set the current status of the TUI.
    ///
    /// # Arguments
    ///
    /// - `status`: [`Option<Status>`] The status to set
    ///
    /// # Examples
    ///
    /// ```rust
    /// use code_g::tui::state::TuiState;
    /// use code_g::tui::models::Status;
    ///
    /// let mut state = TuiState::new();
    /// state.set_status(Some(Status::Thinking));
    /// ```
    pub fn set_status(&mut self, status: Option<Status>) {
        self.current_status = status;
    }

    /// Clear the state.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use code_g::tui::state::TuiState;
    ///
    /// let mut state = TuiState::new();
    /// state.clear();
    /// ```
    pub fn clear(&mut self) {
        self.messages.clear();
        self.current_status = None;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_creates_empty_state() {
        let state = TuiState::new();

        assert_eq!(state.messages.len(), 0);
        assert!(state.current_status.is_none());
    }

    #[test]
    fn add_user_message_adds_message_and_clears_status() {
        let mut state = TuiState::new();
        state.current_status = Some(Status::Thinking);

        state.add_user_message("Hello world".to_string());

        assert_eq!(state.messages.len(), 1);
        assert_eq!(
            state.messages[0],
            Message::User {
                content: "Hello world".to_string()
            }
        );
        assert!(state.current_status.is_none());
    }

    #[test]
    fn add_assistant_message_adds_message_and_clears_status() {
        let mut state = TuiState::new();
        state.current_status = Some(Status::Thinking);

        state.add_assistant_message("Hello there!".to_string());

        assert_eq!(state.messages.len(), 1);
        assert_eq!(
            state.messages[0],
            Message::Assistant {
                content: "Hello there!".to_string()
            }
        );
        assert!(state.current_status.is_none());
    }

    #[test]
    fn add_tool_response_adds_message_and_clears_status() {
        let mut state = TuiState::new();
        state.current_status = Some(Status::ExecutingTool {
            tool_name: "test_tool".to_string(),
        });

        state.add_tool_response("Tool executed successfully".to_string(), false);

        assert_eq!(state.messages.len(), 1);
        assert_eq!(
            state.messages[0],
            Message::ToolResponse {
                summary: "Tool executed successfully".to_string(),
                is_error: false
            }
        );
        assert!(state.current_status.is_none());
    }

    #[test]
    fn add_tool_response_with_error_adds_tool_response_with_error() {
        let mut state = TuiState::new();

        state.add_tool_response("Tool failed".to_string(), true);

        assert_eq!(state.messages.len(), 1);
        assert_eq!(
            state.messages[0],
            Message::ToolResponse {
                summary: "Tool failed".to_string(),
                is_error: true
            }
        );
    }

    #[test]
    fn set_status_updates_current_status() {
        let mut state = TuiState::new();

        let status = Status::ReadingFile {
            path: "/test/file.txt".to_string(),
        };
        state.set_status(Some(status.clone()));

        assert_eq!(state.current_status, Some(status));
    }

    #[test]
    fn set_status_can_clear_status() {
        let mut state = TuiState::new();
        state.current_status = Some(Status::Thinking);

        state.set_status(None);

        assert!(state.current_status.is_none());
    }

    #[test]
    fn clear_removes_all_messages_and_status() {
        let mut state = TuiState::new();
        state.add_user_message("Message 1".to_string());
        state.add_assistant_message("Message 2".to_string());
        state.add_tool_response("Tool response".to_string(), false);
        state.set_status(Some(Status::Thinking));

        state.clear();

        assert_eq!(state.messages.len(), 0);
        assert!(state.current_status.is_none());
    }

    #[test]
    fn multiple_messages_are_stored_in_order() {
        let mut state = TuiState::new();

        state.add_user_message("First message".to_string());
        state.add_assistant_message("Second message".to_string());
        state.add_tool_response("Third message".to_string(), false);

        assert_eq!(state.messages.len(), 3);
        assert_eq!(
            state.messages[0],
            Message::User {
                content: "First message".to_string()
            }
        );
        assert_eq!(
            state.messages[1],
            Message::Assistant {
                content: "Second message".to_string()
            }
        );
        assert_eq!(
            state.messages[2],
            Message::ToolResponse {
                summary: "Third message".to_string(),
                is_error: false
            }
        );
    }

    #[test]
    fn adding_message_after_status_clears_status() {
        let mut state = TuiState::new();

        // Set various statuses and verify they get cleared
        state.set_status(Some(Status::WritingFile {
            path: "test.txt".to_string(),
        }));
        state.add_user_message("User message".to_string());
        assert!(state.current_status.is_none());

        state.set_status(Some(Status::SearchingFiles {
            pattern: "*.rs".to_string(),
        }));
        state.add_assistant_message("Assistant message".to_string());
        assert!(state.current_status.is_none());

        state.set_status(Some(Status::EditingFile {
            path: "main.rs".to_string(),
        }));
        state.add_tool_response("Tool response".to_string(), true);
        assert!(state.current_status.is_none());
    }

    #[test]
    fn empty_content_messages_are_handled() {
        let mut state = TuiState::new();

        state.add_user_message("".to_string());
        state.add_assistant_message("".to_string());
        state.add_tool_response("".to_string(), false);

        assert_eq!(state.messages.len(), 3);
        assert_eq!(
            state.messages[0],
            Message::User {
                content: "".to_string()
            }
        );
        assert_eq!(
            state.messages[1],
            Message::Assistant {
                content: "".to_string()
            }
        );
        assert_eq!(
            state.messages[2],
            Message::ToolResponse {
                summary: "".to_string(),
                is_error: false
            }
        );
    }
}
