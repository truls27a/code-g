use crate::openai::model::{AssistantMessage, ChatMessage};
use std::io::{self, BufRead, Write};

#[derive(Debug, PartialEq)]
pub struct Tui;

impl Tui {
    pub fn new() -> Self {
        Self {}
    }

    pub fn render(&self, messages: &[ChatMessage], writer: &mut impl Write) -> Result<(), io::Error> {
        println!("\x1B[2J\x1B[1;1H"); // clear screen

        for message in messages {
            match message {
                ChatMessage::User { content } => writeln!(writer, "User: {}", content)?,
                ChatMessage::Assistant { message } => match message {
                    AssistantMessage::Content(content) => writeln!(writer, "Assistant: {}", content)?,
                    AssistantMessage::ToolCalls(tool_calls) => writeln!(writer, "Assistant: {:?}", tool_calls)?,
                },
                ChatMessage::Tool { content, tool_call_id: _ } => writeln!(writer, "Tool: {}", content)?,
                ChatMessage::System { content } => writeln!(writer, "System: {}", content)?,
            }
        }

        Ok(())
    }

    pub fn read_user_input(&self, reader: &mut impl BufRead) -> Result<String, io::Error> {
        println!("> ");
        let mut input = String::new();
        reader.read_line(&mut input)?;
        Ok(input.trim().to_string())
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
            ChatMessage::User { content: "Hello".to_string() },
            ChatMessage::Assistant { message: AssistantMessage::Content("Hello human!".to_string()) },
        ];

        let mut output = Vec::new();
        tui.render(&messages, &mut output).unwrap();

        let result = String::from_utf8(output).unwrap();
        assert_eq!(result, "User: Hello\nAssistant: Hello human!\n");
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