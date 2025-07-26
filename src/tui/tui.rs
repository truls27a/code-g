use super::formatter::{terminal::TerminalFormatter, text::TextFormatter, tool::ToolFormatter};
use super::model::{TuiMessage, TuiStatus};
use super::state::TuiState;
use crate::chat::event::{Action, Event, EventHandler};
use std::io::{self, BufRead, Write};

pub struct Tui {
    state: TuiState,
    writer: Box<dyn Write>,
}

impl EventHandler for Tui {
    fn handle_event(&mut self, event: Event) {
        match event {
            Event::SessionStarted => {
                self.state.clear();
                self.clear_terminal().unwrap();
            }
            Event::SessionEnded => {
                self.clear_terminal().unwrap();
            }
            Event::ReceivedUserMessage(message) => {
                self.state.add_user_message(message);
            }
            Event::ReceivedAssistantMessage(message) => {
                self.state.add_assistant_message(message);
            }
            Event::ReceivedToolCall(tool_name, arguments) => {
                let status = ToolFormatter::create_status(&tool_name, &arguments);
                self.state.set_status(Some(status));
            }
            Event::ReceivedToolResponse(tool_response, tool_name, arguments) => {
                let (summary, is_error) =
                    ToolFormatter::create_summary(&tool_response, &tool_name, arguments);
                self.state.add_tool_response(summary, is_error);
            }
            Event::AwaitingAssistantResponse => {
                self.state.set_status(Some(TuiStatus::Thinking));
            }
            Event::PendingFileChange { change_id, file_path, diff } => {
                self.state.add_pending_change(change_id, file_path, diff);
                self.state.set_status(None); // Clear any status when showing pending change
            }
            Event::ChangeAccepted { change_id, accepted_changes } => {
                self.state.add_change_accepted(change_id, accepted_changes.len());
                self.state.set_status(None);
            }
            Event::ChangeDeclined { change_id } => {
                self.state.add_change_declined(change_id);
                self.state.set_status(None);
            }
            Event::ChangeError { change_id, error } => {
                self.state.add_change_error(change_id, error);
                self.state.set_status(None);
            }
        }
        self.render().unwrap();
    }

    fn handle_action(&mut self, action: Action) -> Result<String, io::Error> {
        match action {
            Action::RequestUserInput => self.read_user_input(&mut io::stdin().lock()),
            Action::AcceptChange(change_id) => {
                self.state.set_status(Some(TuiStatus::ProcessingChange { change_id }));
                self.render().unwrap();
                Ok(format!("accept:{}", change_id))
            }
            Action::DeclineChange(change_id) => {
                self.state.set_status(Some(TuiStatus::ProcessingChange { change_id }));
                self.render().unwrap();
                Ok(format!("decline:{}", change_id))
            }
            Action::ListPendingChanges => {
                Ok("list_changes".to_string())
            }
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

        let input = input.trim().to_string();
        
        // Parse special commands
        if input.starts_with("/accept ") || input.starts_with("/a ") {
            if let Some(id_str) = input.split_whitespace().nth(1) {
                if let Ok(change_id) = id_str.parse::<u64>() {
                    return Ok(format!("accept:{}", change_id));
                }
            }
            return Ok("Invalid accept command. Use: /accept <id> or /a <id>".to_string());
        }
        
        if input.starts_with("/decline ") || input.starts_with("/d ") {
            if let Some(id_str) = input.split_whitespace().nth(1) {
                if let Ok(change_id) = id_str.parse::<u64>() {
                    return Ok(format!("decline:{}", change_id));
                }
            }
            return Ok("Invalid decline command. Use: /decline <id> or /d <id>".to_string());
        }
        
        if input == "/changes" || input == "/c" {
            return Ok("list_changes".to_string());
        }
        
        if input == "/help" || input == "/h" {
            return Ok("help:commands".to_string());
        }

        Ok(input)
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
            TuiMessage::PendingChange { change_id, file_path, diff } => {
                writeln!(
                    self.writer,
                    "{}",
                    TextFormatter::yellow_bold(&format!("📝 PENDING CHANGE #{} - {}", change_id, file_path))
                )?;
                writeln!(self.writer, "{}", TextFormatter::gray_text(&self.format_diff(diff)))?;
                writeln!(
                    self.writer,
                    "{}",
                    TextFormatter::cyan_text(&format!("Use /accept {} or /decline {} to respond", change_id, change_id))
                )?;
                writeln!(self.writer)?;
            }
            TuiMessage::ChangeAccepted { change_id, accepted_count } => {
                let message = if *accepted_count == 1 {
                    format!("✅ Change #{} accepted", change_id)
                } else {
                    format!("✅ Change #{} accepted ({} total changes applied)", change_id, accepted_count)
                };
                writeln!(self.writer, "{}", TextFormatter::green_bold(&message))?;
                writeln!(self.writer)?;
            }
            TuiMessage::ChangeDeclined { change_id } => {
                writeln!(
                    self.writer,
                    "{}",
                    TextFormatter::red_bold(&format!("❌ Change #{} declined", change_id))
                )?;
                writeln!(self.writer)?;
            }
            TuiMessage::ChangeError { change_id, error } => {
                writeln!(
                    self.writer,
                    "{}",
                    TextFormatter::red_bold(&format!("💥 Error with change #{}: {}", change_id, error))
                )?;
                writeln!(self.writer)?;
            }
        }
        Ok(())
    }

    fn format_diff(&self, diff: &str) -> String {
        let mut formatted = String::new();
        for line in diff.lines() {
            if line.starts_with('+') && !line.starts_with("+++") {
                formatted.push_str(&format!("  {}\n", TextFormatter::green_text(line)));
            } else if line.starts_with('-') && !line.starts_with("---") {
                formatted.push_str(&format!("  {}\n", TextFormatter::red_text(line)));
            } else if line.starts_with("@@") {
                formatted.push_str(&format!("  {}\n", TextFormatter::cyan_text(line)));
            } else {
                formatted.push_str(&format!("  {}\n", line));
            }
        }
        formatted
    }

    fn clear_terminal(&mut self) -> Result<(), io::Error> {
        write!(self.writer, "{}", TerminalFormatter::clear_screen())?;
        Ok(())
    }
}
