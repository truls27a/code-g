use super::formatting::{Formatter, Terminal};
use crate::chat::event::{ChatSessionAction, ChatSessionEvent};
use crate::openai::model::{AssistantMessage, ChatMessage};
use std::io::{self, BufRead, Write};

pub struct Tui {
    messages: Vec<ChatMessage>,
    status_messages: Option<String>,
    writer: Box<dyn Write>,
}

impl Tui {
    pub fn new() -> Self {
        Self {
            messages: Vec::new(),
            status_messages: None,
            writer: Box::new(io::stdout()),
        }
    }
    pub fn handle_event(&mut self, event: ChatSessionEvent) {
        match event {
            ChatSessionEvent::SessionStarted => {
                self.clear_terminal().unwrap();
            }
            ChatSessionEvent::SessionEnded => {
                self.clear_terminal().unwrap();
            }
            ChatSessionEvent::ReceivedUserMessage(message) => {
                println!("ReceivedUserMessage: {}", message);
                self.messages.push(ChatMessage::User { content: message });
            }
            ChatSessionEvent::ReceivedAssistantMessage(message) => {
                println!("ReceivedAssistantMessage: {}", message);
                self.status_messages = None;
                self.messages.push(ChatMessage::Assistant {
                    message: AssistantMessage::Content(message),
                });
            }
            ChatSessionEvent::ReceivedToolCall(tool_name, arguments) => match tool_name.as_str() {
                "read_file" => {
                    let path = arguments.get("path").unwrap_or(&"".to_string()).clone();
                    self.status_messages = Some(format!("Reading {}...", path));
                }
                "write_file" => {
                    let path = arguments.get("path").unwrap_or(&"".to_string()).clone();
                    self.status_messages = Some(format!("Writing {}...", path));
                }
                "search_files" => {
                    let pattern = arguments.get("pattern").unwrap_or(&"".to_string()).clone();
                    self.status_messages = Some(format!("Searching for '{}'...", pattern));
                }
                "edit_file" => {
                    let path = arguments.get("path").unwrap_or(&"".to_string()).clone();
                    self.status_messages = Some(format!("Editing {}...", path));
                }
                _ => {
                    self.status_messages = Some(format!("Calling tool '{}'", tool_name));
                }
            },
            ChatSessionEvent::ReceivedToolResponse(tool_response, tool_name, tool_call_id) => {
                self.status_messages = None;
                self.messages.push(ChatMessage::Tool {
                    content: tool_response,
                    tool_name,
                    tool_call_id,
                });
            }
            ChatSessionEvent::AwaitingAssistantResponse => {
                self.status_messages = Some("Thinking...".to_string());
            }
        }
        self.render().unwrap();
    }

    pub fn handle_action(&mut self, action: ChatSessionAction) -> Result<String, io::Error> {
        let result = match action {
            ChatSessionAction::RequestUserInput => self.read_user_input(&mut io::stdin().lock()),
        };

        result
    }

    fn render(&mut self) -> Result<(), io::Error> {
        self.clear_terminal()?;

        // Clone the messages to avoid borrowing conflicts
        let messages = self.messages.clone();
        for message in &messages {
            self.render_message(&message)?;
        }

        self.writer.flush()?;
        Ok(())
    }
    fn render_message(&mut self, message: &ChatMessage) -> Result<(), io::Error> {
        match message {
            ChatMessage::System { content: _ } => {} // Do not render system messages
            ChatMessage::User { content } => self.render_user_message(content)?,
            ChatMessage::Assistant { message } => self.render_assistant_message(message)?,
            ChatMessage::Tool {
                content,
                tool_call_id: _,
                tool_name,
            } => self.render_tool_message(content, tool_name)?,
        }

        Ok(())
    }

    fn render_user_message(&mut self, content: &str) -> Result<(), io::Error> {
        writeln!(self.writer, "> {}", content)?;
        writeln!(self.writer, "")?;
        Ok(())
    }

    fn render_assistant_message(&mut self, message: &AssistantMessage) -> Result<(), io::Error> {
        match message {
            AssistantMessage::Content(content) => {
                writeln!(self.writer, "* {}", content)?;
                writeln!(self.writer, "")?;
            }
            _ => {}
        }
        Ok(())
    }

    fn render_tool_message(&mut self, content: &str, tool_name: &str) -> Result<(), io::Error> {
        // Check if this is an error response
        if self.is_tool_error(content, tool_name) {
            writeln!(self.writer, "{}", Formatter::red_italic(content))?;
        } else {
            self.render_successful_tool_response(content, tool_name)?;
        }
        writeln!(self.writer, "")?;
        Ok(())
    }

    fn render_successful_tool_response(
        &mut self,
        content: &str,
        tool_name: &str,
    ) -> Result<(), io::Error> {
        match tool_name {
            "read_file" => {
                writeln!(
                    self.writer,
                    "{}{}{}",
                    Formatter::gray_italic("* Read "),
                    content.lines().count(),
                    Formatter::gray_italic(" lines")
                )?;
            }
            "write_file" => {
                writeln!(
                    self.writer,
                    "{}{}{}",
                    Formatter::gray_italic("* Wrote "),
                    content.lines().count(),
                    Formatter::gray_italic(" lines")
                )?;
            }
            "search_files" => {
                writeln!(
                    self.writer,
                    "{}{}{}",
                    Formatter::gray_italic("* Found "),
                    content.lines().count(),
                    Formatter::gray_italic(" files")
                )?;
            }
            "edit_file" => {
                writeln!(
                    self.writer,
                    "{}{}{}",
                    Formatter::gray_italic("* Edited "),
                    content.lines().count(),
                    Formatter::gray_italic(" lines")
                )?;
            }
            _ => {
                writeln!(
                    self.writer,
                    "{}",
                    Formatter::gray_italic(&format!(
                        "* Tool '{}' returned: {}",
                        tool_name, content
                    ))
                )?;
            }
        }
        Ok(())
    }

    /// Check if tool response content indicates an error
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

    fn clear_terminal(&mut self) -> Result<(), io::Error> {
        write!(self.writer, "{}", Terminal::clear_screen())?;
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
}
