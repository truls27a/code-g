use crate::chat::memory::ChatMemory;
use crate::openai::client::OpenAIClient;
use crate::openai::error::OpenAIError;
use crate::openai::model::{AssistantMessage, ChatMessage, ChatResult, OpenAiModel};
use crate::tools::registry::ToolRegistry;

pub struct ChatSession {
    memory: ChatMemory,
    client: OpenAIClient,
    tools: ToolRegistry,
}

impl ChatSession {
    pub fn new(client: OpenAIClient, tools: ToolRegistry) -> Self {
        Self {
            memory: ChatMemory::new(),
            client,
            tools,
        }
    }

    pub async fn send_message(&mut self, message: &str) -> Result<String, OpenAIError> {
        let user_message = ChatMessage::User {
            content: message.to_string(),
        };

        self.memory.add_message(user_message);

        // TODO: Handle scenario where it does to many tool calls
        loop {
            let response = match self
                .client
                .create_chat_completion(
                    &OpenAiModel::Gpt4oMini, // TODO: Make this configurable
                    &self.memory.get_memory(),
                    &self.tools.to_openai_tools(),
                )
                .await
            {
                Ok(res) => res,
                Err(e) => return Err(e), // Just bubble up the error here
            };

            match response {
                ChatResult::Message(content) => {
                    self.memory.add_message(ChatMessage::Assistant {
                        message: AssistantMessage::Content(content.clone()),
                    });
                    return Ok(content);
                }
                ChatResult::ToolCalls(tool_calls) => {
                    // 1. Add assistant message with tool_calls
                    let assistant_msg = ChatMessage::Assistant {
                        message: AssistantMessage::ToolCalls(tool_calls.clone()),
                    };
                    self.memory.add_message(assistant_msg);

                    // 2. Call each tool and collect responses
                    for tool_call in &tool_calls {
                        let tool_response = self
                            .tools
                            .call_tool(tool_call.name.as_str(), tool_call.arguments.clone())
                            .unwrap_or_else(|e| format!("Error calling tool: {}", e));
                        let tool_msg = ChatMessage::Tool {
                            content: tool_response,
                            tool_call_id: tool_call.id.clone(),
                        };
                        self.memory.add_message(tool_msg);
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::openai::model::{Parameters, Property};
    use crate::tools::tool::Tool;
    use std::collections::HashMap;

    #[test]
    fn new_creates_a_chat_session_with_empty_memory() {
        let openai_client = OpenAIClient::new("any_api_key".to_string());
        let chat_session = ChatSession::new(openai_client, ToolRegistry::new());
        assert_eq!(chat_session.memory.get_memory().len(), 0);
    }

    #[test]
    fn new_creates_a_chat_session_with_empty_tools() {
        let openai_client = OpenAIClient::new("any_api_key".to_string());
        let chat_session = ChatSession::new(openai_client, ToolRegistry::new());
        assert_eq!(chat_session.tools.len(), 0);
    }

    #[tokio::test]
    async fn send_message_adds_user_message_to_memory() {
        let openai_client = OpenAIClient::new(std::env::var("OPENAI_API_KEY").unwrap());
        let mut chat_session = ChatSession::new(openai_client, ToolRegistry::new());
        chat_session.send_message("Hello").await.unwrap();
        assert_eq!(
            chat_session.memory.get_memory()[0],
            ChatMessage::User {
                content: "Hello".to_string()
            }
        );
    }

    #[tokio::test]
    async fn send_message_adds_assistant_message_to_memory() {
        let openai_client = OpenAIClient::new(std::env::var("OPENAI_API_KEY").unwrap());
        let mut chat_session = ChatSession::new(openai_client, ToolRegistry::new());
        chat_session
            .send_message("Respond with 'Hello', nothing else.")
            .await
            .unwrap();
        assert_eq!(
            chat_session.memory.get_memory()[1],
            ChatMessage::Assistant {
                message: AssistantMessage::Content("Hello".to_string())
            }
        );
    }

    #[tokio::test]
    async fn send_message_returns_message_when_message_is_sent() {
        let openai_client = OpenAIClient::new(std::env::var("OPENAI_API_KEY").unwrap());
        let mut chat_session = ChatSession::new(openai_client, ToolRegistry::new());
        let response = chat_session
            .send_message("Say 'Hello', nothing else.")
            .await
            .unwrap();
        assert!(response.contains("Hello"));
    }

    #[tokio::test]
    async fn send_message_uses_tools_when_tools_are_provided() {
        let openai_client = OpenAIClient::new(std::env::var("OPENAI_API_KEY").unwrap());

        struct TestTool;

        impl Tool for TestTool {
            fn name(&self) -> String {
                "read_file".to_string()
            }

            fn description(&self) -> String {
                "Read the content of a file".to_string()
            }

            fn parameters(&self) -> Parameters {
                Parameters {
                    param_type: "object".to_string(),
                    properties: HashMap::from([(
                        "path".to_string(),
                        Property {
                            prop_type: "string".to_string(),
                            description: "The path to the file to read".to_string(),
                        },
                    )]),
                    required: vec!["path".to_string()],
                    additional_properties: false,
                }
            }

            fn strict(&self) -> bool {
                true
            }

            fn call(&self, _args: HashMap<String, String>) -> Result<String, String> {
                Ok("Hello, world!".to_string())
            }
        }

        let mut chat_session = ChatSession::new(openai_client, ToolRegistry::from(vec![Box::new(TestTool)]));

        chat_session
            .send_message("Read the content of the poem.txt file")
            .await
            .unwrap();
        if let ChatMessage::Tool { .. } = chat_session.memory.get_memory()[2] {
            // It's a tool message
        } else {
            panic!("Expected a tool message");
        }
    }
}
