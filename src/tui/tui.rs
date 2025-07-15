use crate::openai::model::{AssistantMessage, ChatMessage};
use std::io;

pub struct Tui;

impl Tui {
    pub fn new() -> Self {
        Self {}
    }

    pub fn render(&self, messages: &[ChatMessage]) {
        println!("\x1B[2J\x1B[1;1H"); // clear screen

        for message in messages {
            match message {
                ChatMessage::User { content } => println!("User: {}", content),
                ChatMessage::Assistant { message } => match message {
                    AssistantMessage::Content(content) => println!("Assistant: {}", content),
                    AssistantMessage::ToolCalls(tool_calls) => println!("Assistant: {:?}", tool_calls),
                },
                ChatMessage::Tool { content, tool_call_id: _ } => println!("Tool: {}", content),
                ChatMessage::System { content } => println!("System: {}", content),
            }
        }
    }

    pub fn read_user_input(&self) -> Result<String, io::Error> {
        println!("> ");
        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        Ok(input.trim().to_string())
    }
}