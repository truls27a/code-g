use crate::chat::error::ChatSessionError;
use crate::chat::memory::ChatMemory;
use crate::chat::system_prompt::SYSTEM_PROMPT;
use crate::openai::client::OpenAIClient;
use crate::openai::model::{AssistantMessage, ChatMessage, ChatResult, OpenAiModel};
use crate::tools::registry::ToolRegistry;
use crate::tui::tui::Tui;
use std::io;

// Maximum number of iterations per message to prevent infinite loops
const MAX_ITERATIONS: usize = 10;

#[derive(Debug, Clone)]
pub enum SystemPromptConfig {
    /// No system prompt will be added
    None,
    /// Use the default system prompt
    Default,
    /// Use a custom system prompt
    Custom(String),
}

pub struct ChatSession {
    memory: ChatMemory,
    client: OpenAIClient,
    tools: ToolRegistry,
    tui: Tui,
    silent: bool,
}

impl ChatSession {
    pub fn new(
        client: OpenAIClient,
        tools: ToolRegistry,
        system_prompt_config: SystemPromptConfig,
        silent: bool,
    ) -> Self {
        let memory = match system_prompt_config {
            SystemPromptConfig::None => ChatMemory::from(vec![]),
            SystemPromptConfig::Default => ChatMemory::from(vec![ChatMessage::System {
                content: SYSTEM_PROMPT.to_string(),
            }]),
            SystemPromptConfig::Custom(custom_prompt) => {
                ChatMemory::from(vec![ChatMessage::System {
                    content: custom_prompt,
                }])
            }
        };

        Self {
            memory,
            client,
            tools,
            tui: Tui::new(),
            silent,
        }
    }

    pub async fn send_message(&mut self, message: &str) -> Result<String, ChatSessionError> {
        // Add user message to memory
        self.memory.add_message(ChatMessage::User {
            content: message.to_string(),
        });

        // Render the memory to the TUI (only if not silent)
        if !self.silent {
            self.tui
                .render(&self.memory.get_memory(), &mut io::stdout())
                .unwrap();
        }

        // Track iterations to prevent infinite loops
        let mut iterations = 0;

        // Loop until the client returns a message or max iterations reached
        loop {
            iterations += 1;

            // Check if we've exceeded the maximum number of iterations
            if iterations > MAX_ITERATIONS {
                return Err(ChatSessionError::MaxIterationsExceeded {
                    max_iterations: MAX_ITERATIONS,
                });
            }

            // 1. Get a response from the client
            let response = self
                .client
                .create_chat_completion(
                    &OpenAiModel::Gpt4oMini, // TODO: Make this configurable
                    &self.memory.get_memory(),
                    &self.tools.to_openai_tools(),
                )
                .await?;

            // 2. Handle the response from the client
            match response {
                // 3. If the response is a message, add it to the memory and return it
                ChatResult::Message { content, turn_over } => {
                    // 3.1 Add assistant message with content
                    self.memory.add_message(ChatMessage::Assistant {
                        message: AssistantMessage::Content(content.clone()),
                    });

                    // 3.2 Render the memory to the TUI (only if not silent)
                    if !self.silent {
                        self.tui
                            .render(&self.memory.get_memory(), &mut io::stdout())
                            .unwrap(); // TODO: Handle errors
                    }

                    // 3.3 Return the content only if turn is over, otherwise continue
                    if turn_over {
                        return Ok(content);
                    }
                    // If turn is not over, continue the loop to get more responses
                }
                // 3. If the response is tool calls, add them to the memory and process them, add the tool responses to the memory, and then finally start over to get the assistants response
                ChatResult::ToolCalls(tool_calls) => {
                    // 3.1 Add assistant message with tool_calls
                    self.memory.add_message(ChatMessage::Assistant {
                        message: AssistantMessage::ToolCalls(tool_calls.clone()),
                    });

                    // 3.2 Render the memory to the TUI (only if not silent)
                    if !self.silent {
                        self.tui
                            .render(&self.memory.get_memory(), &mut io::stdout())
                            .unwrap(); // TODO: Handle errors
                    }

                    // 3.2 Call each tool and collect responses
                    for tool_call in &tool_calls {
                        // 3.2.1 Call the tool
                        let tool_response = self
                            .tools
                            .call_tool(tool_call.name.as_str(), tool_call.arguments.clone())
                            .unwrap_or_else(|e| format!("Error calling tool: {}", e));

                        // 3.2.2 Add tool response to memory
                        self.memory.add_message(ChatMessage::Tool {
                            content: tool_response,
                            tool_call_id: tool_call.id.clone(),
                        });

                        // 3.2.3 Render the memory to the TUI (only if not silent)
                        if !self.silent {
                            self.tui
                                .render(&self.memory.get_memory(), &mut io::stdout())
                                .unwrap(); // TODO: Handle errors
                        }
                    }
                    continue;
                }
            }
        }
    }

    pub async fn run(&mut self) -> Result<(), ChatSessionError> {
        loop {
            let user_input = self.tui.read_user_input(&mut io::stdin().lock()).unwrap(); // TODO: Handle errors

            if user_input == "exit" {
                break;
            }

            self.send_message(&user_input).await?;
        }
        Ok(())
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
        let chat_session = ChatSession::new(
            openai_client,
            ToolRegistry::new(),
            SystemPromptConfig::None,
            true,
        );
        assert_eq!(chat_session.memory.get_memory().len(), 0);
    }

    #[test]
    fn new_creates_a_chat_session_with_empty_tools() {
        let openai_client = OpenAIClient::new("any_api_key".to_string());
        let chat_session = ChatSession::new(
            openai_client,
            ToolRegistry::new(),
            SystemPromptConfig::None,
            true,
        );
        assert_eq!(chat_session.tools.len(), 0);
    }

    #[tokio::test]
    async fn send_message_adds_user_message_to_memory() {
        let openai_client = OpenAIClient::new(std::env::var("OPENAI_API_KEY").unwrap());
        let mut chat_session = ChatSession::new(
            openai_client,
            ToolRegistry::new(),
            SystemPromptConfig::None,
            true,
        );
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
        let mut chat_session = ChatSession::new(
            openai_client,
            ToolRegistry::new(),
            SystemPromptConfig::None,
            true,
        );
        chat_session
            .send_message("Respond with 'Hello', nothing else.")
            .await
            .unwrap();
        // Check that assistant message was added (structure may vary based on turn_over)
        if let ChatMessage::Assistant {
            message: AssistantMessage::Content(content),
        } = &chat_session.memory.get_memory()[1]
        {
            assert!(content.contains("Hello"));
        } else {
            panic!("Expected assistant message with content");
        }
    }

    #[tokio::test]
    async fn send_message_returns_message_when_message_is_sent() {
        let openai_client = OpenAIClient::new(std::env::var("OPENAI_API_KEY").unwrap());
        let mut chat_session = ChatSession::new(
            openai_client,
            ToolRegistry::new(),
            SystemPromptConfig::None,
            true,
        );
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

        let mut chat_session = ChatSession::new(
            openai_client,
            ToolRegistry::from(vec![Box::new(TestTool)]),
            SystemPromptConfig::None,
            true,
        );

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
