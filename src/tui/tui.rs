use super::model::{TuiMessage, TuiStatus};
use super::state::TuiState;
use super::formatter::{
    text_formatter::TextFormatter, terminal_formatter::TerminalFormatter,
    tool_formatter::ToolFormatter,
};
use crate::chat::event::{ChatSessionAction, ChatSessionEvent, ChatSessionEventHandler};
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
                let status = ToolFormatter::create_status(&tool_name, &arguments);
                self.state.set_status(Some(status));
            }
            ChatSessionEvent::ReceivedToolResponse(tool_response, tool_name, arguments) => {
                let (summary, is_error) =
                    ToolFormatter::create_summary(&tool_response, &tool_name, arguments);
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

    fn read_user_input(&self, reader: &mut impl BufRead) -> Result<String, io::Error> {
        // Save current cursor position and move to bottom to show prompt
        print!("{}", TerminalFormatter::save_cursor());
        print!("{}", TerminalFormatter::move_to_bottom());
        print!("> ");
        io::stdout().flush()?;

        // Capture the user's input
        let mut input = String::new();
        reader.read_line(&mut input)?;

        // Clear the prompt line and restore cursor position
        print!("{}", TerminalFormatter::move_to_bottom_and_clear());
        print!("{}", TerminalFormatter::restore_cursor());
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
