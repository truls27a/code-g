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
        self.clear_terminal(writer)?;

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

    pub fn clear_terminal(&self, writer: &mut impl Write) -> Result<(), io::Error> {
        write!(writer, "\x1B[2J\x1B[1;1H")?; // clear screen
        Ok(())
    }

    fn render_message(
        &self,
        message: &ChatMessage,
        writer: &mut impl Write,
    ) -> Result<(), io::Error> {
        match message {
            ChatMessage::System { content: _ } => {}, // Do not render system messages
            ChatMessage::User { content } => {
                writeln!(writer, "> {}", content)?;
                writeln!(writer, "")?;
            }
            ChatMessage::Assistant { message } => match message {
                AssistantMessage::Content(content) => {
                    writeln!(writer, "* {}", content)?;
                    writeln!(writer, "")?;
                }
                AssistantMessage::ToolCalls(tool_calls) => {
                    for tool_call in tool_calls {
                        match tool_call.name.as_str() {
                            "read_file" => {
                                writeln!(
                                    writer,
                                    "* Reading {}",
                                    tool_call.arguments.get("path").unwrap_or(&"".to_string())
                                )?;
                            }
                            "write_file" => {
                                writeln!(
                                    writer,
                                    "* Writing {}",
                                    tool_call.arguments.get("path").unwrap_or(&"".to_string())
                                )?;
                            }
                            "search_files" => {
                                writeln!(
                                    writer,
                                    "* Searching for '{}'",
                                    tool_call
                                        .arguments
                                        .get("pattern")
                                        .unwrap_or(&"".to_string())
                                )?;
                            }
                            _ => {
                                writeln!(writer, "* Calling tool '{}'", tool_call.name)?;
                            }
                        }
                        writeln!(writer, "")?;
                    }
                }
            },
            ChatMessage::Tool {
                content,
                tool_call_id: _,
                tool_name,
            } => {
                match tool_name.as_str() {
                    "read_file" => {
                        writeln!(writer, "* Read {} lines", content.lines().count())?;
                    }
                    "write_file" => {
                        writeln!(writer, "* Wrote {} lines", content.lines().count())?;
                    }
                    "search_files" => {
                        writeln!(writer, "* Found {} files", content.lines().count())?;
                    }
                    _ => {
                        writeln!(writer, "* Tool '{}' returned: {}", tool_name, content)?;
                    }
                }
            }
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
        assert_eq!(result, "\x1B[2J\x1B[1;1H> Hello\n\n* Hello human!\n\n");
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
