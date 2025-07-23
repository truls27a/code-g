use super::formatting::{Formatter, Terminal};
use crate::chat::event::{ChatSessionAction, ChatSessionEvent};
use crate::openai::model::{AssistantMessage, ChatMessage};
use std::io::{self, BufRead, BufReader, Write};

#[derive(Debug, PartialEq)]
pub struct Tui {
    messages: Vec<ChatMessage>,
    status_messages: Option<Vec<String>>,
}

impl Tui {
    pub fn new() -> Self {
        Self {
            messages: Vec::new(),
            status_messages: None,
        }
    }
    pub fn handle_event(&mut self, event: ChatSessionEvent) {
        match event {
            ChatSessionEvent::SessionStarted => {
                self.clear_terminal(&mut io::stdout()).unwrap();
            }
            ChatSessionEvent::SessionEnded => {
                self.clear_terminal(&mut io::stdout()).unwrap();
            }
            _ => {}
        }
    }

    pub fn handle_action(&mut self, action: ChatSessionAction) -> Result<String, io::Error> {
        let result = match action {
            ChatSessionAction::RequestUserInput => {
                self.read_user_input(&mut io::stdin().lock())
            }
        };

        result
    }

    fn clear_terminal(&self, writer: &mut impl Write) -> Result<(), io::Error> {
        write!(writer, "{}", Terminal::clear_screen())?;
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

