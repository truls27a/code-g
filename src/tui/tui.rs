use super::formatting::{Formatter, Terminal};
use super::model::{TuiMessage, TuiStatus};
use super::state::TuiState;
use crate::chat::event::{ChatSessionAction, ChatSessionEvent, ChatSessionEventHandler};
use std::collections::HashMap;
use std::io::{self, BufRead, Write};

pub struct Tui {
    state: TuiState,
    writer: Box<dyn Write>,
}

impl ChatSessionEventHandler for Tui {
    fn handle_event(&mut self, event: ChatSessionEvent) {
        match event {
            ChatSessionEvent::SessionStarted => {
                self.state.clear();
                self.clear_terminal().unwrap();
            }
            ChatSessionEvent::SessionEnded => {
                self.clear_terminal().unwrap();
            }
            ChatSessionEvent::ReceivedUserMessage(message) => {
                self.state.add_user_message(message);
            }
            ChatSessionEvent::ReceivedAssistantMessage(message) => {
                self.state.add_assistant_message(message);
            }
            ChatSessionEvent::ReceivedToolCall(tool_name, arguments) => {
                let status = self.create_tool_status(&tool_name, &arguments);
                self.state.set_status(Some(status));
            }
            ChatSessionEvent::ReceivedToolResponse(tool_response, tool_name, arguments) => {
                let (summary, is_error) =
                    self.create_tool_summary(&tool_response, &tool_name, arguments);
                self.state.add_tool_response(summary, is_error);
            }
            ChatSessionEvent::AwaitingAssistantResponse => {
                self.state.set_status(Some(TuiStatus::Thinking));
            }
        }
        self.render().unwrap();
    }

    fn handle_action(&mut self, action: ChatSessionAction) -> Result<String, io::Error> {
        match action {
            ChatSessionAction::RequestUserInput => self.read_user_input(&mut io::stdin().lock()),
        }
    }
}

impl Tui {
    pub fn new() -> Self {
        Self {
            state: TuiState::new(),
            writer: Box::new(io::stdout()),
        }
    }
   
   fn create_tool_status(
        &self,
        tool_name: &str,
        arguments: &std::collections::HashMap<String, String>,
    ) -> TuiStatus {
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

    fn create_tool_summary(
        &self,
        content: &str,
        tool_name: &str,
        arguments: HashMap<String, String>,
    ) -> (String, bool) {
        let is_error = self.is_tool_error(content, tool_name);

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
                Formatter::gray_italic(&status.to_string())
            )?;
            writeln!(self.writer)?;
        }

        self.writer.flush()?;
        Ok(())
    }

    fn read_user_input(&self, reader: &mut impl BufRead) -> Result<String, io::Error> {
        // Save current cursor position and move to bottom to show prompt
        print!("{}", Terminal::save_cursor());
        print!("{}", Terminal::move_to_bottom());
        print!("> ");
        io::stdout().flush()?;

        // Capture the user's input
        let mut input = String::new();
        reader.read_line(&mut input)?;

        // Clear the prompt line and restore cursor position
        print!("{}", Terminal::move_to_bottom_and_clear());
        print!("{}", Terminal::restore_cursor());
        io::stdout().flush()?;

        Ok(input.trim().to_string())
    }

    fn render_message(&mut self, message: &TuiMessage) -> Result<(), io::Error> {
        match message {
            TuiMessage::User { content } => {
                writeln!(self.writer, "> {}", content)?;
                writeln!(self.writer)?;
            }
            TuiMessage::Assistant { content } => {
                writeln!(self.writer, "* {}", content)?;
                writeln!(self.writer)?;
            }
            TuiMessage::ToolResponse { summary, is_error } => {
                if *is_error {
                    writeln!(self.writer, "{}", Formatter::red_italic(summary))?;
                } else {
                    writeln!(
                        self.writer,
                        "{}",
                        Formatter::gray_italic(&format!("* {}", summary))
                    )?;
                }
                writeln!(self.writer)?;
            }
        }
        Ok(())
    }

    fn clear_terminal(&mut self) -> Result<(), io::Error> {
        write!(self.writer, "{}", Terminal::clear_screen())?;
        Ok(())
    }

    fn is_tool_error(&self, content: &str, tool_name: &str) -> bool {
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
