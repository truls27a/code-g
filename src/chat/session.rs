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
                // 1. Add assistant message with tool_calls
                let assistant_msg = ChatMessage::Assistant {
                    content: None,
                    tool_calls: Some(tool_calls.clone()),
                };
                self.memory.add_message(assistant_msg);

                // 2. Call each tool and collect responses
                for tool_call in &tool_calls {
                    let tool_response = "To be or not to be, that is the question".to_string(); // TODO: Implement this
                    let tool_msg = ChatMessage::Tool {
                        content: tool_response,
                        tool_call_id: tool_call.id.clone(),
                    };
                    self.memory.add_message(tool_msg);
                }

                // 3. Re-call OpenAI with tool responses in memory
                let followup_response = self
                    .client
                    .create_chat_completion(
                        &OpenAiModel::Gpt4oMini,
                        &self.memory.get_memory(),
                        &self.tools,
                    )
                    .await?;

                // 4. Push final assistant message
                if let ChatResult::Message(content) = followup_response {
                    self.memory.add_message(ChatMessage::Assistant {
                        content: Some(content.clone()),
                        tool_calls: None,
                    });
                    return Ok(content);
                } else {
                    Err("Expected final assistant message, got tool calls again".into()) // TODO: Allow for further tool calls
                }
            }
        }
    }
}
