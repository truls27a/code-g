use crate::openai::model::ChatMessage;

#[derive(Debug, PartialEq, Clone)]
pub struct ChatMemory {
    memory: Vec<ChatMessage>,
}

impl ChatMemory {
    pub fn new() -> Self {
        Self { memory: vec![] }
    }

    pub fn from(memory: Vec<ChatMessage>) -> Self {
        Self { memory }
    }

    pub fn add_message(&mut self, message: ChatMessage) {
        self.memory.push(message);
    }

    pub fn get_memory(&self) -> &Vec<ChatMessage> {
        &self.memory
    }

    pub fn get_last_message(&self) -> Option<&ChatMessage> {
        self.memory.last()
    }

    pub fn remove_message(&mut self, message: &ChatMessage) {
        self.memory.retain(|m| m != message);
    }

    pub fn remove_last_message(&mut self) {
        self.memory.pop();
    }

    pub fn clear(&mut self) {
        self.memory.clear();
    }
}

#[cfg(test)]
mod tests {
    use crate::openai::model::{AssistantMessage, ToolCall};
    use std::collections::HashMap;

    use super::*;

    #[test]
    fn add_message_adds_message_to_memory() {
        let mut memory = ChatMemory::new();

        memory.add_message(ChatMessage::User {
            content: "Hello, world!".to_string(),
        });

        assert_eq!(memory.get_memory().len(), 1);
        assert_eq!(
            memory.get_memory()[0],
            ChatMessage::User {
                content: "Hello, world!".to_string()
            }
        );
    }

    #[test]
    fn get_memory_returns_memory() {
        let mut memory = ChatMemory::new();

        memory.add_message(ChatMessage::User {
            content: "Hello, world!".to_string(),
        });

        assert_eq!(
            memory.get_memory(),
            &[ChatMessage::User {
                content: "Hello, world!".to_string(),
            }]
        );
    }

    #[test]
    fn get_last_message_returns_last_message() {
        let mut memory = ChatMemory::new();

        memory.add_message(ChatMessage::User {
            content: "How are you?".to_string(),
        });
        memory.add_message(ChatMessage::Assistant {
            message: AssistantMessage::Content("I'm fine, thank you!".to_string()),
        });

        assert_eq!(
            memory.get_last_message(),
            Some(&ChatMessage::Assistant {
                message: AssistantMessage::Content("I'm fine, thank you!".to_string()),
            }),
        );
    }

    #[test]
    fn remove_message_removes_message_from_memory() {
        let mut memory = ChatMemory::new();

        memory.add_message(ChatMessage::User {
            content: "Hello, world!".to_string(),
        });
        memory.add_message(ChatMessage::Assistant {
            message: AssistantMessage::ToolCalls(vec![ToolCall {
                id: "1".to_string(),
                name: "read_file".to_string(),
                arguments: HashMap::new(),
            }]),
        });
        memory.remove_message(&ChatMessage::User {
            content: "Hello, world!".to_string(),
        });

        assert_eq!(memory.get_memory().len(), 1);
        assert_eq!(
            memory.get_memory()[0],
            ChatMessage::Assistant {
                message: AssistantMessage::ToolCalls(vec![ToolCall {
                    id: "1".to_string(),
                    name: "read_file".to_string(),
                    arguments: HashMap::new(),
                }]),
            }
        );
    }

    #[test]
    fn remove_last_message_removes_last_message_from_memory() {
        let mut memory = ChatMemory::new();

        memory.add_message(ChatMessage::User {
            content: "Hello, how are you?".to_string(),
        });

        memory.add_message(ChatMessage::Assistant {
            message: AssistantMessage::Content("I'm fine, thank you!".to_string()),
        });
        memory.add_message(ChatMessage::User {
            content: "Whats your name?".to_string(),
        });
        memory.add_message(ChatMessage::Assistant {
            message: AssistantMessage::ToolCalls(vec![ToolCall {
                id: "1".to_string(),
                name: "read_file".to_string(),
                arguments: HashMap::new(),
            }]),
        });
        memory.add_message(ChatMessage::Tool {
            content: "My name is John Doe".to_string(),
            tool_call_id: "1".to_string(),
            tool_name: "test_tool".to_string(),
        });
        memory.add_message(ChatMessage::Assistant {
            message: AssistantMessage::Content("My name is John Doe".to_string()),
        });

        memory.remove_last_message();

        assert_eq!(memory.get_memory().len(), 5);
        assert_eq!(
            memory.get_memory()[4],
            ChatMessage::Tool {
                content: "My name is John Doe".to_string(),
                tool_call_id: "1".to_string(),
                tool_name: "test_tool".to_string(),
            }
        );
    }

    #[test]
    fn clear_clears_memory() {
        let mut memory = ChatMemory::new();
        memory.add_message(ChatMessage::User {
            content: "Hello, world!".to_string(),
        });
        memory.clear();
        assert_eq!(memory.get_memory().len(), 0);
        assert_eq!(memory.get_memory(), &[]);
    }
}
