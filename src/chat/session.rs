use crate::chat::memory::ChatMemory;
use crate::openai::client::OpenAIClient;
use crate::openai::model::{ChatMessage, ChatResult, OpenAiModel, Tool};

pub struct ChatSession {
    memory: ChatMemory,
    client: OpenAIClient,
    tools: Vec<Tool>,
}

impl ChatSession {
    pub fn new(client: OpenAIClient, tools: Vec<Tool>) -> Self {
        Self {
            memory: ChatMemory::new(),
            client,
            tools,
        }
    }

    pub async fn send_message(
        &mut self,
        message: &str,
    ) -> Result<String, Box<dyn std::error::Error>> {
        // TODO: Add custom error type
        let user_message = ChatMessage::User {
            content: message.to_string(),
        };

        self.memory.add_message(user_message);

        let response = self
            .client
            .create_chat_completion(
                &OpenAiModel::Gpt4oMini, // TODO: Make this configurable
                &self.memory.get_memory(),
                &self.tools,
            )
            .await?;

        match response {
            ChatResult::Message(content) => {
                self.memory.add_message(ChatMessage::Assistant {
                    content: Some(content.clone()),
                    tool_calls: None,
                });
                return Ok(content);
            }
            ChatResult::ToolCalls(tool_calls) => {
                for tool_call in tool_calls {
                    let tool_call_response = ChatMessage::Assistant {
                        content: None,
                        tool_calls: Some(vec![tool_call]),
                    };
                    self.memory.add_message(tool_call_response);
                }

                // For each tool call:
                // 1. Call the tool
                // 2. Add the tool call response to the memory
                // Finally, call the chat completion again

                // TODO: Implement tool call response
                return Ok("".to_string());
            }
        };

    }
}
