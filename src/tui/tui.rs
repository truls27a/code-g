use super::formatting::{Formatter, Terminal};
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

    pub fn clear_terminal(&self, writer: &mut impl Write) -> Result<(), io::Error> {
        write!(writer, "{}", Terminal::clear_screen())?;
        Ok(())
    }

    fn render_message(
        &self,
        message: &ChatMessage,
        writer: &mut impl Write,
    ) -> Result<(), io::Error> {
        match message {
            ChatMessage::System { content: _ } => {} // Do not render system messages
            ChatMessage::User { content } => self.render_user_message(content, writer)?,
            ChatMessage::Assistant { message } => self.render_assistant_message(message, writer)?,
            ChatMessage::Tool {
                content,
                tool_call_id: _,
                tool_name,
            } => self.render_tool_message(content, tool_name, writer)?,
        }

        Ok(())
    }

    fn render_user_message(&self, content: &str, writer: &mut impl Write) -> Result<(), io::Error> {
        writeln!(writer, "> {}", content)?;
        writeln!(writer, "")?;
        Ok(())
    }

    fn render_assistant_message(
        &self,
        message: &AssistantMessage,
        writer: &mut impl Write,
    ) -> Result<(), io::Error> {
        match message {
            AssistantMessage::Content(content) => {
                writeln!(writer, "* {}", content)?;
                writeln!(writer, "")?;
            }
            AssistantMessage::ToolCalls(tool_calls) => {
                self.render_tool_calls(tool_calls, writer)?;
            }
        }
        Ok(())
    }

    fn render_tool_calls(
        &self,
        tool_calls: &[crate::openai::model::ToolCall],
        writer: &mut impl Write,
    ) -> Result<(), io::Error> {
        for tool_call in tool_calls {
            match tool_call.name.as_str() {
                "read_file" => {
                    writeln!(
                        writer,
                        "{}",
                        Formatter::gray_italic(&format!(
                            "* Reading {}...",
                            tool_call.arguments.get("path").unwrap_or(&"".to_string())
                        ))
                    )?;
                }
                "write_file" => {
                    writeln!(
                        writer,
                        "{}",
                        Formatter::gray_italic(&format!(
                            "* Writing {}...",
                            tool_call.arguments.get("path").unwrap_or(&"".to_string())
                        ))
                    )?;
                }
                "search_files" => {
                    writeln!(
                        writer,
                        "{}",
                        Formatter::gray_italic(&format!(
                            "* Searching for '{}'...",
                            tool_call
                                .arguments
                                .get("pattern")
                                .unwrap_or(&"".to_string())
                        ))
                    )?;
                }
                "edit_file" => {
                    writeln!(
                        writer,
                        "{}",
                        Formatter::gray_italic(&format!(
                            "* Editing {}...",
                            tool_call.arguments.get("path").unwrap_or(&"".to_string())
                        ))
                    )?;
                }
                _ => {
                    writeln!(
                        writer,
                        "{}",
                        Formatter::gray_italic(&format!("* Calling tool '{}'", tool_call.name))
                    )?;
                }
            }
        }
        Ok(())
    }

    fn render_tool_message(
        &self,
        content: &str,
        tool_name: &str,
        writer: &mut impl Write,
    ) -> Result<(), io::Error> {
        // Check if this is an error response
        if self.is_tool_error(content, tool_name) {
            writeln!(writer, "{}", Formatter::red_italic(content))?;
        } else {
            self.render_successful_tool_response(content, tool_name, writer)?;
        }
        writeln!(writer, "")?;
        Ok(())
    }

    fn render_successful_tool_response(
        &self,
        content: &str,
        tool_name: &str,
        writer: &mut impl Write,
    ) -> Result<(), io::Error> {
        match tool_name {
            "read_file" => {
                writeln!(
                    writer,
                    "{}{}{}",
                    Formatter::gray_italic("* Read "),
                    content.lines().count(),
                    Formatter::gray_italic(" lines")
                )?;
            }
            "write_file" => {
                writeln!(
                    writer,
                    "{}{}{}",
                    Formatter::gray_italic("* Wrote "),
                    content.lines().count(),
                    Formatter::gray_italic(" lines")
                )?;
            }
            "search_files" => {
                writeln!(
                    writer,
                    "{}{}{}",
                    Formatter::gray_italic("* Found "),
                    content.lines().count(),
                    Formatter::gray_italic(" files")
                )?;
            }
            "edit_file" => {
                writeln!(
                    writer,
                    "{}{}{}",
                    Formatter::gray_italic("* Edited "),
                    content.lines().count(),
                    Formatter::gray_italic(" lines")
                )?;
            }
            _ => {
                writeln!(
                    writer,
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
        let expected = format!("{}> Hello\n\n* Hello human!\n\n", Terminal::clear_screen());
        assert_eq!(result, expected);
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

    #[test]
    fn is_tool_error_detects_read_file_errors() {
        let tui = Tui::new();

        // Error cases
        assert!(tui.is_tool_error("Error: Permission denied", "read_file"));
        assert!(tui.is_tool_error("File not found", "read_file"));
        assert!(tui.is_tool_error("File does not exist or not found", "read_file"));

        // Non-error cases
        assert!(!tui.is_tool_error("File content here", "read_file"));
        assert!(!tui.is_tool_error("This file contains not found text", "read_file"));
        assert!(!tui.is_tool_error("", "read_file"));
    }

    #[test]
    fn is_tool_error_detects_write_file_errors() {
        let tui = Tui::new();

        // Error cases
        assert!(tui.is_tool_error("Error: Permission denied", "write_file"));
        assert!(tui.is_tool_error("Error writing file", "write_file"));

        // Non-error cases
        assert!(!tui.is_tool_error("File written successfully", "write_file"));
        assert!(!tui.is_tool_error("Successfully wrote 5 lines", "write_file"));
        assert!(!tui.is_tool_error("", "write_file"));
    }

    #[test]
    fn is_tool_error_detects_edit_file_errors() {
        let tui = Tui::new();

        // Error cases
        assert!(tui.is_tool_error("Error: Invalid edit", "edit_file"));
        assert!(tui.is_tool_error("Pattern not found in file", "edit_file"));
        assert!(tui.is_tool_error("Pattern appears 3 times in file", "edit_file"));

        // Non-error cases
        assert!(!tui.is_tool_error("File edited successfully", "edit_file"));
        assert!(!tui.is_tool_error("Replaced 1 occurrence", "edit_file"));
        assert!(!tui.is_tool_error("", "edit_file"));
        assert!(!tui.is_tool_error("File contains the word appears", "edit_file"));
    }

    #[test]
    fn is_tool_error_detects_search_files_errors() {
        let tui = Tui::new();

        // Error cases
        assert!(tui.is_tool_error("Error: Invalid pattern", "search_files"));
        assert!(tui.is_tool_error("No files found matching pattern", "search_files"));

        // Non-error cases
        assert!(!tui.is_tool_error("Found 5 files", "search_files"));
        assert!(!tui.is_tool_error("file1.txt\nfile2.txt", "search_files"));
        assert!(!tui.is_tool_error("", "search_files"));
    }

    #[test]
    fn is_tool_error_detects_unknown_tool_errors() {
        let tui = Tui::new();

        // Error cases
        assert!(tui.is_tool_error("Error: Something went wrong", "unknown_tool"));
        assert!(tui.is_tool_error("Error in processing", "custom_tool"));

        // Non-error cases
        assert!(!tui.is_tool_error("Success", "unknown_tool"));
        assert!(!tui.is_tool_error("Tool completed", "custom_tool"));
        assert!(!tui.is_tool_error("", "unknown_tool"));
    }

    #[test]
    fn is_tool_error_handles_edge_cases() {
        let tui = Tui::new();

        // Empty tool name
        assert!(tui.is_tool_error("Error: test", ""));
        assert!(!tui.is_tool_error("Success", ""));

        // Case sensitivity
        assert!(!tui.is_tool_error("error: lowercase", "read_file"));
        assert!(!tui.is_tool_error("ERROR: uppercase", "read_file"));

        // Whitespace
        assert!(!tui.is_tool_error(" Error: leading space", "read_file"));
        assert!(tui.is_tool_error("Error: trailing space ", "read_file"));
    }

    #[test]
    fn render_message_displays_tool_errors_with_red_formatting() {
        let tui = Tui::new();
        let mut output = Vec::new();

        // Test read_file error
        let error_message = ChatMessage::Tool {
            content: "Error: Permission denied".to_string(),
            tool_call_id: "call_123".to_string(),
            tool_name: "read_file".to_string(),
        };

        tui.render_message(&error_message, &mut output).unwrap();
        let result = String::from_utf8(output).unwrap();

        // Should contain red italic formatting
        assert!(result.contains(&format!("{}{}", Formatter::red(), Formatter::italic())));
        assert!(result.contains("Error: Permission denied"));
        assert!(result.contains(Formatter::reset()));
    }

    #[test]
    fn render_message_displays_successful_tool_responses_normally() {
        let tui = Tui::new();
        let mut output = Vec::new();

        // Test successful read_file response
        let success_message = ChatMessage::Tool {
            content: "File content here\nLine 2\nLine 3".to_string(),
            tool_call_id: "call_123".to_string(),
            tool_name: "read_file".to_string(),
        };

        tui.render_message(&success_message, &mut output).unwrap();
        let result = String::from_utf8(output).unwrap();

        // Should contain gray italic formatting for "* Read " and " lines" separately
        assert!(result.contains(&Formatter::gray_italic("* Read ")));
        assert!(result.contains(&Formatter::gray_italic(" lines")));
        assert!(result.contains("3")); // Plain number
        assert!(!result.contains(&format!("{}{}", Formatter::red(), Formatter::italic()))); // Should not contain red formatting
    }

    #[test]
    fn render_message_handles_write_file_errors() {
        let tui = Tui::new();
        let mut output = Vec::new();

        let error_message = ChatMessage::Tool {
            content: "Error: Cannot write to read-only file".to_string(),
            tool_call_id: "call_456".to_string(),
            tool_name: "write_file".to_string(),
        };

        tui.render_message(&error_message, &mut output).unwrap();
        let result = String::from_utf8(output).unwrap();

        assert!(result.contains(&format!("{}{}", Formatter::red(), Formatter::italic()))); // Red italic
        assert!(result.contains("Error: Cannot write to read-only file"));
    }

    #[test]
    fn render_message_handles_edit_file_errors() {
        let tui = Tui::new();
        let mut output = Vec::new();

        let error_message = ChatMessage::Tool {
            content: "Pattern not found in file".to_string(),
            tool_call_id: "call_789".to_string(),
            tool_name: "edit_file".to_string(),
        };

        tui.render_message(&error_message, &mut output).unwrap();
        let result = String::from_utf8(output).unwrap();

        assert!(result.contains(&format!("{}{}", Formatter::red(), Formatter::italic()))); // Red italic
        assert!(result.contains("Pattern not found in file"));
    }

    #[test]
    fn render_message_handles_search_files_errors() {
        let tui = Tui::new();
        let mut output = Vec::new();

        let error_message = ChatMessage::Tool {
            content: "No files found matching pattern".to_string(),
            tool_call_id: "call_101".to_string(),
            tool_name: "search_files".to_string(),
        };

        tui.render_message(&error_message, &mut output).unwrap();
        let result = String::from_utf8(output).unwrap();

        assert!(result.contains(&format!("{}{}", Formatter::red(), Formatter::italic()))); // Red italic
        assert!(result.contains("No files found matching pattern"));
    }

    #[test]
    fn render_message_handles_successful_tool_responses_for_all_tools() {
        let tui = Tui::new();

        // Test write_file success
        let mut output1 = Vec::new();
        let write_success = ChatMessage::Tool {
            content: "Line 1\nLine 2".to_string(),
            tool_call_id: "call_1".to_string(),
            tool_name: "write_file".to_string(),
        };
        tui.render_message(&write_success, &mut output1).unwrap();
        let result1 = String::from_utf8(output1).unwrap();
        assert!(result1.contains(&Formatter::gray_italic("* Wrote ")));
        assert!(result1.contains(&Formatter::gray_italic(" lines")));
        assert!(result1.contains("2"));
        assert!(!result1.contains(&format!("{}{}", Formatter::red(), Formatter::italic())));

        // Test search_files success
        let mut output2 = Vec::new();
        let search_success = ChatMessage::Tool {
            content: "file1.txt\nfile2.txt\nfile3.txt".to_string(),
            tool_call_id: "call_2".to_string(),
            tool_name: "search_files".to_string(),
        };
        tui.render_message(&search_success, &mut output2).unwrap();
        let result2 = String::from_utf8(output2).unwrap();
        assert!(result2.contains(&Formatter::gray_italic("* Found ")));
        assert!(result2.contains(&Formatter::gray_italic(" files")));
        assert!(result2.contains("3"));
        assert!(!result2.contains(&format!("{}{}", Formatter::red(), Formatter::italic())));

        // Test edit_file success
        let mut output3 = Vec::new();
        let edit_success = ChatMessage::Tool {
            content: "Successfully edited".to_string(),
            tool_call_id: "call_3".to_string(),
            tool_name: "edit_file".to_string(),
        };
        tui.render_message(&edit_success, &mut output3).unwrap();
        let result3 = String::from_utf8(output3).unwrap();
        assert!(result3.contains(&Formatter::gray_italic("* Edited ")));
        assert!(result3.contains(&Formatter::gray_italic(" lines")));
        assert!(result3.contains("1"));
        assert!(!result3.contains(&format!("{}{}", Formatter::red(), Formatter::italic())));
    }

    #[test]
    fn render_message_handles_unknown_tool_errors() {
        let tui = Tui::new();
        let mut output = Vec::new();

        let error_message = ChatMessage::Tool {
            content: "Error: Unknown tool failure".to_string(),
            tool_call_id: "call_999".to_string(),
            tool_name: "custom_tool".to_string(),
        };

        tui.render_message(&error_message, &mut output).unwrap();
        let result = String::from_utf8(output).unwrap();

        assert!(result.contains(&format!("{}{}", Formatter::red(), Formatter::italic()))); // Red italic
        assert!(result.contains("Error: Unknown tool failure"));
    }
}
