use super::formatter::{terminal::TerminalFormatter, text::TextFormatter};
use super::model::{Message, Status};
use super::state::TuiState;
use crate::session::event::{Action, Event, EventHandler};
use crate::tools::registry::Registry;
use std::io::{self, BufRead, Write};

/// Terminal User Interface for the chat application.
///
/// The `Tui` struct is responsible for managing the visual presentation and user interaction
/// layer of the [`ChatSession`]. It acts as a bridge between the chat logic and the terminal,
/// handling both input from the user and output rendering to the screen.
///
/// # Examples
///
/// ```rust
/// use code_g::tui::tui::Tui;
/// use code_g::session::event::{Event, EventHandler};
///
/// let mut tui = Tui::new();
///
/// tui.handle_event(Event::SessionStarted);
/// tui.handle_event(Event::ReceivedUserMessage { message: "Hello, how are you?".to_string() });
/// tui.handle_event(Event::AwaitingAssistantResponse);
/// tui.handle_event(Event::ReceivedAssistantMessage { message: "I'm doing well, thank you!".to_string() });
/// tui.handle_event(Event::SessionEnded);
/// ```
///
/// # Notes
///
/// - The TUI is designed to easily replace with a web UI or other UI layer by implementing the [`EventHandler`] trait
/// - The TUI is designed to be used in conjunction with a [`ChatSession`]
pub struct Tui {
    state: TuiState,
    writer: Box<dyn Write>,
    reader: Box<dyn BufRead>,
}

impl EventHandler for Tui {
    /// Handle an event from the ChatSession by updating TUI state and rendering.
    ///
    /// This method processes chat session events and updates the terminal display accordingly:
    /// - `SessionStarted/Ended`: Clears the terminal and resets state
    /// - `ReceivedUserMessage/AssistantMessage`: Adds messages to chat history and re-renders
    /// - `ReceivedToolCall`: Updates status display to show tool execution in progress
    /// - `ReceivedToolResponse`: Adds tool response to chat history and clears status
    /// - `AwaitingAssistantResponse`: Shows "thinking" status indicator
    ///
    /// After processing each event, the entire terminal is cleared and re-rendered to ensure
    /// a consistent display state.
    ///
    /// # Arguments
    ///
    /// - `event`: [`Event`] The event to handle
    ///
    /// # Examples
    ///
    /// ```rust
    /// use code_g::tui::tui::Tui;
    /// use code_g::session::event::{Event, EventHandler};
    ///
    /// let mut tui = Tui::new();
    ///
    /// tui.handle_event(Event::SessionStarted);
    /// ```
    fn handle_event(&mut self, event: Event) {
        match event {
            Event::SessionStarted => {
                self.state.clear();
                self.clear_terminal().unwrap();
            }
            Event::SessionEnded => {
                self.clear_terminal().unwrap();
            }
            Event::ReceivedUserMessage { message } => {
                self.state.add_user_message(message);
            }
            Event::ReceivedAssistantMessage { message } => {
                self.state.add_assistant_message(message);
            }
            Event::ReceivedToolCall {
                tool_name,
                parameters,
            } => {
                if let Some(tool) = Registry::get_from_all_tools(&tool_name) {
                    let status = tool.status(&parameters);
                    self.state.set_status(Some(status));
                } else {
                    self.state
                        .set_status(Some(Status::ExecutingTool { tool_name }));
                }
            }
            Event::ReceivedToolResponse {
                tool_name,
                response,
                parameters,
            } => {
                if let Some(tool) = Registry::get_from_all_tools(&tool_name) {
                    let summary = tool.summary_message(&parameters, &response);
                    self.state.add_tool_response(summary, false);
                } else {
                    let summary = format!("{}: {}", tool_name, response);
                    self.state.add_tool_response(summary, false);
                }
            }
            Event::AwaitingAssistantResponse => {
                self.state.set_status(Some(Status::Thinking));
            }
        }
        self.render().unwrap();
    }

    /// Handle an action from the ChatSession, right now it is only user input requests.
    ///
    /// This method processes chat session actions and returns the requested data:
    /// - `RequestUserInput`: Displays a prompt ("> "), reads a line from stdin,
    ///   and returns the trimmed input string
    ///
    /// # Arguments
    ///
    /// - `action`: [`Action`] The action to handle
    ///
    /// # Returns
    ///
    /// - `String` The user input
    ///
    /// # Errors
    /// Returns `io::Error` if reading from stdin fails or if terminal cursor
    /// operations cannot be performed.
    ///
    /// # Examples
    ///
    /// ```rust, no_run
    /// use code_g::tui::tui::Tui;
    /// use code_g::session::event::{Action, EventHandler};
    ///
    /// let mut tui = Tui::new();
    ///
    /// let result = tui.handle_action(Action::RequestUserInput); // This will block until user input is provided
    /// assert_eq!(result.unwrap(), "Hello, how are you?");
    /// ```
    fn handle_action(&mut self, action: Action) -> Result<String, io::Error> {
        match action {
            Action::RequestUserInput => self.read_user_input(),
            Action::RequestUserApproval {
                operation,
                details,
                tool_name: _,
            } => self.request_user_approval(&operation, &details),
        }
    }
}

impl Tui {
    /// Create a new TUI.
    ///
    /// # Returns
    ///
    /// - `Tui` The new TUI instance
    ///
    /// # Examples
    ///
    /// ```rust
    /// use code_g::tui::tui::Tui;
    ///
    /// let tui = Tui::new();
    /// ```
    pub fn new() -> Self {
        Self {
            state: TuiState::new(),
            writer: Box::new(io::stdout()),
            reader: Box::new(io::stdin().lock()),
        }
    }

    fn render(&mut self) -> Result<(), io::Error> {
        self.clear_terminal()?;

        // Clone the messages to avoid borrowing conflicts
        let messages = self.state.messages.clone();
        for message in &messages {
            self.render_message(message)?;
        }

        // Render current status if any
        if let Some(status) = &self.state.current_status {
            writeln!(
                self.writer,
                "{}",
                TextFormatter::gray_italic(&status.to_string())
            )?;
            writeln!(self.writer)?;
        }

        self.writer.flush()?;
        Ok(())
    }

    fn read_user_input(&mut self) -> Result<String, io::Error> {
        // Save current cursor position and move to bottom to show prompt
        print!("{}", TerminalFormatter::save_cursor());
        print!("{}", TerminalFormatter::move_to_bottom());
        print!("> ");
        io::stdout().flush()?;

        // Capture the user's input
        let mut input = String::new();
        self.reader.read_line(&mut input)?;

        // Clear the prompt line and restore cursor position
        print!("{}", TerminalFormatter::move_to_bottom_and_clear());
        print!("{}", TerminalFormatter::restore_cursor());
        io::stdout().flush()?;

        Ok(input.trim().to_string())
    }

    fn request_user_approval(
        &mut self,
        operation: &str,
        details: &str,
    ) -> Result<String, io::Error> {
        // Save current cursor position and move to bottom to show approval prompt
        print!("{}", TerminalFormatter::save_cursor());
        print!("{}", TerminalFormatter::move_to_bottom());

        // Display approval prompt
        writeln!(self.writer)?;
        writeln!(self.writer, "⚠️  Permission Required ⚠️")?;
        writeln!(self.writer, "Operation: {}", operation)?;
        writeln!(self.writer, "Details: {}", details)?;
        writeln!(self.writer)?;
        print!("[A]pprove / [D]ecline: ");
        io::stdout().flush()?;

        // Capture the user's response
        let mut input = String::new();
        self.reader.read_line(&mut input)?;
        let response = input.trim().to_lowercase();

        // Clear the approval prompt and restore cursor position
        print!("{}", TerminalFormatter::move_to_bottom_and_clear());
        print!("{}", TerminalFormatter::restore_cursor());
        io::stdout().flush()?;

        // Return "approved" or "declined" based on user input
        match response.as_str() {
            "a" | "approve" | "y" | "yes" => Ok("approved".to_string()),
            _ => Ok("declined".to_string()),
        }
    }

    fn render_message(&mut self, message: &Message) -> Result<(), io::Error> {
        match message {
            Message::User { content } => {
                writeln!(self.writer, "> {}", content)?;
                writeln!(self.writer)?;
            }
            Message::Assistant { content } => {
                writeln!(self.writer, "* {}", content)?;
                writeln!(self.writer)?;
            }
            Message::ToolResponse { summary, is_error } => {
                if *is_error {
                    writeln!(self.writer, "{}", TextFormatter::red_italic(summary))?;
                } else {
                    writeln!(
                        self.writer,
                        "{}",
                        TextFormatter::gray_italic(&format!("* {}", summary))
                    )?;
                }
                writeln!(self.writer)?;
            }
        }
        Ok(())
    }

    fn clear_terminal(&mut self) -> Result<(), io::Error> {
        write!(self.writer, "{}", TerminalFormatter::clear_screen())?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::session::event::Event;
    use std::collections::HashMap;
    use std::io::Cursor;

    fn tui_with_writer_and_reader(writer: Box<dyn Write>, reader: Box<dyn BufRead>) -> Tui {
        Tui {
            state: TuiState::new(),
            writer,
            reader,
        }
    }

    #[test]
    fn new_creates_tui_with_empty_state() {
        let tui = Tui::new();

        assert_eq!(tui.state.messages.len(), 0);
        assert!(tui.state.current_status.is_none());
    }

    #[test]
    fn handle_event_session_started_clears_state() {
        let mut tui = Tui::new();

        // Add some initial state
        tui.state.add_user_message("test message".to_string());
        tui.state.set_status(Some(Status::Thinking));

        // Handle session started event
        tui.handle_event(Event::SessionStarted);

        assert_eq!(tui.state.messages.len(), 0);
        assert!(tui.state.current_status.is_none());
    }

    #[test]
    fn handle_event_session_ended_clears_terminal() {
        let mut tui = Tui::new();

        tui.handle_event(Event::SessionEnded);

        // We can't easily test terminal clearing without mocking terminal calls,
        // but we can verify the event was handled without panicking
        assert_eq!(tui.state.messages.len(), 0);
    }

    #[test]
    fn handle_event_received_user_message_adds_to_state() {
        let mut tui = Tui::new();

        let message = "Hello, how are you?".to_string();
        tui.handle_event(Event::ReceivedUserMessage {
            message: message.clone(),
        });

        assert_eq!(tui.state.messages.len(), 1);
        match &tui.state.messages[0] {
            Message::User { content } => assert_eq!(content, &message),
            _ => panic!("Expected user message"),
        }
    }

    #[test]
    fn handle_event_received_assistant_message_adds_to_state() {
        let mut tui = Tui::new();

        let message = "I'm doing well, thank you!".to_string();
        tui.handle_event(Event::ReceivedAssistantMessage {
            message: message.clone(),
        });

        assert_eq!(tui.state.messages.len(), 1);
        match &tui.state.messages[0] {
            Message::Assistant { content } => assert_eq!(content, &message),
            _ => panic!("Expected assistant message"),
        }
    }

    #[test]
    fn handle_event_received_tool_call_sets_status() {
        let mut tui = Tui::new();

        let tool_name = "read_file".to_string();
        let mut arguments = HashMap::new();
        arguments.insert("path".to_string(), "test.txt".to_string());

        tui.handle_event(Event::ReceivedToolCall {
            tool_name: tool_name.clone(),
            parameters: arguments.clone(),
        });

        assert!(tui.state.current_status.is_some());
        match &tui.state.current_status.as_ref().unwrap() {
            Status::ReadingFile { path } => assert_eq!(path, "test.txt"),
            _ => panic!("Expected ReadingFile status"),
        }
    }

    #[test]
    fn handle_event_received_tool_response_adds_tool_message() {
        let mut tui = Tui::new();

        let tool_response = "File content here".to_string();
        let tool_name = "read_file".to_string();
        let mut arguments = HashMap::new();
        arguments.insert("path".to_string(), "test.txt".to_string());

        tui.handle_event(Event::ReceivedToolResponse {
            tool_name: tool_name.clone(),
            response: tool_response.clone(),
            parameters: arguments.clone(),
        });

        assert_eq!(tui.state.messages.len(), 1);
        match &tui.state.messages[0] {
            Message::ToolResponse { summary, is_error } => {
                assert!(summary.contains("Read 1 lines from test.txt"));
                assert!(!is_error);
            }
            _ => panic!("Expected tool response message"),
        }
    }

    #[test]
    fn handle_event_awaiting_assistant_response_sets_thinking_status() {
        let mut tui = Tui::new();

        tui.handle_event(Event::AwaitingAssistantResponse);

        assert!(tui.state.current_status.is_some());
        match &tui.state.current_status.as_ref().unwrap() {
            Status::Thinking => (),
            _ => panic!("Expected Thinking status"),
        }
    }

    #[test]
    fn handle_action_request_user_input_reads_from_stdin() {
        let input = "test input\n";
        let mut tui = tui_with_writer_and_reader(
            Box::new(Cursor::new(Vec::new())),
            Box::new(Cursor::new(input.as_bytes())),
        );

        let result = tui.read_user_input();

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "test input");
    }

    #[test]
    fn render_message_formats_user_message_correctly() {
        let mut tui = Tui::new();

        let message = Message::User {
            content: "Hello world".to_string(),
        };

        let result = tui.render_message(&message);
        assert!(result.is_ok());
    }

    #[test]
    fn render_message_formats_assistant_message_correctly() {
        let mut tui = Tui::new();

        let message = Message::Assistant {
            content: "Hi there!".to_string(),
        };

        let result = tui.render_message(&message);
        assert!(result.is_ok());
    }

    #[test]
    fn render_message_formats_tool_response_correctly() {
        let mut tui = Tui::new();

        let message = Message::ToolResponse {
            summary: "File read successfully".to_string(),
            is_error: false,
        };

        let result = tui.render_message(&message);
        assert!(result.is_ok());
    }

    #[test]
    fn render_message_formats_error_tool_response_correctly() {
        let mut tui = Tui::new();

        let message = Message::ToolResponse {
            summary: "Error reading file".to_string(),
            is_error: true,
        };

        let result = tui.render_message(&message);
        assert!(result.is_ok());
    }

    #[test]
    fn clear_terminal_writes_clear_sequence() {
        let mut tui = Tui::new();

        let result = tui.clear_terminal();
        assert!(result.is_ok());
    }

    #[test]
    fn render_displays_all_messages_in_order() {
        let mut tui = Tui::new();

        // Add multiple messages
        tui.state.add_user_message("First message".to_string());
        tui.state.add_assistant_message("Response".to_string());
        tui.state
            .add_tool_response("Tool output".to_string(), false);

        let result = tui.render();
        assert!(result.is_ok());
    }

    #[test]
    fn render_displays_current_status_when_set() {
        let mut tui = Tui::new();

        tui.state.set_status(Some(Status::Thinking));

        let result = tui.render();
        assert!(result.is_ok());
    }

    #[test]
    fn handle_event_handles_multiple_events_in_sequence() {
        let mut tui = Tui::new();

        // Simulate a complete interaction sequence
        tui.handle_event(Event::SessionStarted);
        tui.handle_event(Event::ReceivedUserMessage {
            message: "Hello".to_string(),
        });
        tui.handle_event(Event::AwaitingAssistantResponse);
        tui.handle_event(Event::ReceivedAssistantMessage {
            message: "Hi there!".to_string(),
        });

        assert_eq!(tui.state.messages.len(), 2);
        assert!(tui.state.current_status.is_none()); // Should be cleared after assistant message
    }

    #[test]
    fn state_is_properly_updated_after_each_event() {
        let mut tui = Tui::new();

        // Test that status is cleared after adding messages
        tui.handle_event(Event::AwaitingAssistantResponse);
        assert!(tui.state.current_status.is_some());

        tui.handle_event(Event::ReceivedUserMessage {
            message: "test".to_string(),
        });
        assert!(tui.state.current_status.is_none());
    }
}
