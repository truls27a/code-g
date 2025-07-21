use crate::openai::model::{AssistantMessage, ChatMessage};
use std::io::{self, BufRead, Write};

#[derive(Debug, PartialEq)]
pub struct Tui;

impl Tui {
    pub fn new() -> Self {
        Self {}
    }

    pub fn render(
        &self,
        messages: &[ChatMessage],
        writer: &mut impl Write,
    ) -> Result<(), io::Error> {
        write!(writer, "\x1B[2J\x1B[1;1H")?; // clear screen

        for message in messages {
            self.render_message(message, writer)?;
        }

        writer.flush()?;
        Ok(())
    }

    pub fn read_user_input(&self, reader: &mut impl BufRead) -> Result<String, io::Error> {
        // Save current cursor position and move to bottom to show prompt
        print!("\x1B[s"); // Save cursor position
        print!("\x1B[999;1H"); // Move to bottom of terminal
        print!("> "); // Print prompt
        io::stdout().flush()?;

        // Capture the user's input
        let mut input = String::new();
        reader.read_line(&mut input)?;

        // Clear the prompt line and restore cursor position
        print!("\x1B[999;1H\x1B[K"); // Move to bottom and clear line
        print!("\x1B[u"); // Restore cursor position
        io::stdout().flush()?;

        Ok(input.trim().to_string())
    }

    fn render_message(
        &self,
        message: &ChatMessage,
        writer: &mut impl Write,
    ) -> Result<(), io::Error> {
        match message {
            ChatMessage::User { content } => {
                writeln!(writer, "User: {}", content)?;
            }
            ChatMessage::Assistant { message } => match message {
                AssistantMessage::Content(content) => writeln!(writer, "Assistant: {}", content)?,
                AssistantMessage::ToolCalls(tool_calls) => {
                    for tool_call in tool_calls {
                        writeln!(writer, "Assistant is calling tool: {}", tool_call.name)?;
                    }
                }
            },
            _ => {}
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::BufReader;

    #[test]
    fn new_creates_a_tui() {
        let tui = Tui::new();

        assert_eq!(tui, Tui);
    }

    #[test]
    fn render_prints_messages() {
        let tui = Tui::new();
        let messages = vec![
            ChatMessage::User {
                content: "Hello".to_string(),
            },
            ChatMessage::Assistant {
                message: AssistantMessage::Content("Hello human!".to_string()),
            },
        ];

        let mut output = Vec::new();
        tui.render(&messages, &mut output).unwrap();

        let result = String::from_utf8(output).unwrap();
        assert_eq!(
            result,
            "\x1B[2J\x1B[1;1HUser: Hello\nAssistant: Hello human!\n"
        );
    }

    #[test]
    fn render_does_not_print_system_messages() {
        let tui = Tui::new();
        let messages = vec![ChatMessage::System {
            content: "Hello".to_string(),
        }];

        let mut output = Vec::new();
        tui.render(&messages, &mut output).unwrap();

        let result = String::from_utf8(output).unwrap();
        assert_eq!(result, "\x1B[2J\x1B[1;1H");
    }

    #[test]
    fn read_user_input_reads_input() {
        let tui = Tui::new();
        let input = "Hello world!\n";
        let mut reader = BufReader::new(input.as_bytes());

        let result = tui.read_user_input(&mut reader).unwrap();

        assert_eq!(result, "Hello world!");
    }

    #[test]
    fn read_user_input_only_reads_first_line() {
        let tui = Tui::new();
        let input = "Hello world!\nHello again!\n";
        let mut reader = BufReader::new(input.as_bytes());

        let result = tui.read_user_input(&mut reader).unwrap();

        assert_eq!(result, "Hello world!");
    }

    #[test]
    fn read_user_input_works_with_empty_input() {
        let tui = Tui::new();
        let input = "";
        let mut reader = BufReader::new(input.as_bytes());

        let result = tui.read_user_input(&mut reader).unwrap();

        assert_eq!(result, "");
    }
}
