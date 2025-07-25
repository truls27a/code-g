use crate::chat::error::{ChatSessionError, ChatSessionErrorHandling};
use crate::chat::event::{ChatSessionAction, ChatSessionEvent};
use crate::chat::memory::ChatMemory;
use crate::chat::system_prompt::{SYSTEM_PROMPT, SystemPromptConfig};
use crate::openai::client::OpenAIClient;
use crate::openai::error::OpenAIError;
use crate::openai::model::{AssistantMessage, ChatMessage, ChatResult, OpenAiModel};
use crate::tools::registry::ToolRegistry;
use crate::tui::tui::Tui;

// Maximum number of iterations per message to prevent infinite loops
const MAX_ITERATIONS: usize = 10;

pub struct ChatSession {
    memory: ChatMemory,
    client: OpenAIClient,
    tools: ToolRegistry,
    tui: Tui,
}

impl ChatSession {
    pub fn new(
        client: OpenAIClient,
        tools: ToolRegistry,
        tui: Tui,
        system_prompt_config: SystemPromptConfig,
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
            tui,
        }
    }

    pub async fn send_message(&mut self, message: &str) -> Result<String, ChatSessionError> {
        // Add user message to memory
        self.memory.add_message(ChatMessage::User {
            content: message.to_string(),
        });

        // Track iterations to prevent infinite loops
        let mut iterations = 0;

        // Loop until the client returns a message or max iterations reached
        loop {
            iterations += 1;

            // 1. Check if we've exceeded the maximum number of iterations
            if iterations > MAX_ITERATIONS {
                return Err(ChatSessionError::MaxIterationsExceeded {
                    max_iterations: MAX_ITERATIONS,
                });
            }

            // 2. Set the status message to thinking
            self.tui.handle_event(ChatSessionEvent::AwaitingAssistantResponse);

            // 3. Get a response from the client
            let response = match self
                .client
                .create_chat_completion(
                    &OpenAiModel::Gpt4oMini, // TODO: Make this configurable
                    &self.memory.get_memory(),
                    &self.tools.to_openai_tools(),
                )
                .await
            {
                Ok(response) => response,
                Err(e) => match self.handle_openai_error(e, iterations) {
                    ChatSessionErrorHandling::Fatal(err) => return Err(err),
                    ChatSessionErrorHandling::Retry => continue,
                    ChatSessionErrorHandling::AddToMemoryAndRetry(message) => {
                        self.memory
                            .add_message(ChatMessage::System { content: message });
                        continue;
                    }
                },
            };

            // 4. Handle the response from the client
            match response {
                // 5. If the response is a message, add it to the memory and return it
                ChatResult::Message { content, turn_over } => {
                    // 5.1 Add assistant message with content
                    self.memory.add_message(ChatMessage::Assistant {
                        message: AssistantMessage::Content(content.clone()),
                    });

                    // 5.2 Render the memory to the TUI (only if not silent)
                    self.tui.handle_event(ChatSessionEvent::ReceivedAssistantMessage(content.clone()));

                    // 5.3 Return the content only if turn is over, otherwise continue
                    if turn_over {
                        return Ok(content);
                    }
                    // If turn is not over, continue the loop to get more responses
                }
                // 6. If the response is tool calls, add them to the memory and process them, add the tool responses to the memory, and then finally start over to get the assistants response
                ChatResult::ToolCalls(tool_calls) => {
                    // 6.1 Add assistant message with tool_calls
                    self.memory.add_message(ChatMessage::Assistant {
                        message: AssistantMessage::ToolCalls(tool_calls.clone()),
                    });


                    // 6.2 Call each tool and collect responses
                    for tool_call in &tool_calls {
                        // 6.2.1 Set the status message to the tool call name
                        self.tui.handle_event(ChatSessionEvent::ReceivedToolCall(tool_call.name.clone(), tool_call.arguments.clone()));
                        
                        // 6.2.2 Call the tool
                        let tool_response = self
                            .tools
                            .call_tool(tool_call.name.as_str(), tool_call.arguments.clone())
                            .unwrap_or_else(|e| e);

                        // 6.2.3 Add tool response to memory
                        self.memory.add_message(ChatMessage::Tool {
                            content: tool_response.clone(),
                            tool_call_id: tool_call.id.clone(),
                            tool_name: tool_call.name.clone(),
                        });

                        // 6.2.4 Send tool response event to the TUI
                        self.tui.handle_event(ChatSessionEvent::ReceivedToolResponse(tool_response.clone(), tool_call.name.clone(), tool_call.id.clone()));

                    }

                    // 6.3 Continue the loop to get the assistants response
                    continue;
                }
            }
        }
    }

    pub async fn run(&mut self) -> Result<(), ChatSessionError> {
        // Clear the terminal
        self.tui.handle_event(ChatSessionEvent::SessionStarted);

        loop {
            let user_input = self.tui.handle_action(ChatSessionAction::RequestUserInput).unwrap(); // TODO: Handle errors

            if user_input == "exit" {
                // Exit the loop
                break;
            }

            self.send_message(&user_input).await?;
        }

        self.tui.handle_event(ChatSessionEvent::SessionEnded);

        Ok(())
    }

    /// Categorizes and handles OpenAI errors appropriately
    fn handle_openai_error(
        &self,
        error: OpenAIError,
        iteration: usize,
    ) -> ChatSessionErrorHandling {
        use OpenAIError::*;

        match error {
            // Fatal errors - configuration or account issues that won't resolve by retrying
            InvalidApiKey | MissingApiKey | InsufficientCredits | InvalidModel
            | EmptyChatHistory => ChatSessionErrorHandling::Fatal(ChatSessionError::OpenAI(error)),

            // Network/service errors - might be temporary, retry a few times
            RateLimitExceeded | ServiceUnavailable | HttpError(_) => {
                if iteration <= 3 {
                    ChatSessionErrorHandling::Retry
                } else {
                    ChatSessionErrorHandling::Fatal(ChatSessionError::OpenAI(error))
                }
            }

            // Content/parsing errors - AI might have made a mistake, inform it and retry
            InvalidContentResponse
            | InvalidToolCallArguments
            | NoCompletionFound
            | NoChoicesFound
            | NoContentFound => {
                let message = format!(
                    "An error occurred with the AI response: {}. Please try again with a different approach.",
                    error
                );
                ChatSessionErrorHandling::AddToMemoryAndRetry(message)
            }

            // Request errors - likely a programming bug, but inform AI in case it can adapt
            InvalidChatMessageRequest => {
                let message = format!(
                    "Invalid request format: {}. Please ensure your response follows the correct format.",
                    error
                );
                ChatSessionErrorHandling::AddToMemoryAndRetry(message)
            }

            // Other errors - treat as potentially recoverable
            Other(_) => {
                let message = format!(
                    "An unexpected error occurred: {}. Please try a different approach.",
                    error
                );
                ChatSessionErrorHandling::AddToMemoryAndRetry(message)
            }
        }
    }
}

// #[cfg(test)]
// mod tests {
//     use super::*;
//     use crate::openai::model::{Parameters, Property};
//     use crate::tools::tool::Tool;
//     use std::collections::HashMap;

//     #[test]
//     fn new_creates_a_chat_session_with_empty_memory() {
//         let openai_client = OpenAIClient::new("any_api_key".to_string());
//         let chat_session = ChatSession::new(
//             openai_client,
//             ToolRegistry::new(),
//             SystemPromptConfig::None,
//             true,
//         );
//         assert_eq!(chat_session.memory.get_memory().len(), 0);
//     }

//     #[test]
//     fn new_creates_a_chat_session_with_empty_tools() {
//         let openai_client = OpenAIClient::new("any_api_key".to_string());
//         let chat_session = ChatSession::new(
//             openai_client,
//             ToolRegistry::new(),
//             SystemPromptConfig::None,
//             true,
//         );
//         assert_eq!(chat_session.tools.len(), 0);
//     }

//     #[tokio::test]
//     async fn send_message_adds_user_message_to_memory() {
//         let openai_client = OpenAIClient::new(std::env::var("OPENAI_API_KEY").unwrap());
//         let mut chat_session = ChatSession::new(
//             openai_client,
//             ToolRegistry::new(),
//             SystemPromptConfig::None,
//             true,
//         );
//         chat_session.send_message("Hello").await.unwrap();
//         assert_eq!(
//             chat_session.memory.get_memory()[0],
//             ChatMessage::User {
//                 content: "Hello".to_string()
//             }
//         );
//     }

//     #[tokio::test]
//     async fn send_message_adds_assistant_message_to_memory() {
//         let openai_client = OpenAIClient::new(std::env::var("OPENAI_API_KEY").unwrap());
//         let mut chat_session = ChatSession::new(
//             openai_client,
//             ToolRegistry::new(),
//             SystemPromptConfig::None,
//             true,
//         );
//         chat_session
//             .send_message("Respond with 'Hello', nothing else.")
//             .await
//             .unwrap();
//         // Check that assistant message was added (structure may vary based on turn_over)
//         if let ChatMessage::Assistant {
//             message: AssistantMessage::Content(content),
//         } = &chat_session.memory.get_memory()[1]
//         {
//             assert!(content.contains("Hello"));
//         } else {
//             panic!("Expected assistant message with content");
//         }
//     }

//     #[tokio::test]
//     async fn send_message_returns_message_when_message_is_sent() {
//         let openai_client = OpenAIClient::new(std::env::var("OPENAI_API_KEY").unwrap());
//         let mut chat_session = ChatSession::new(
//             openai_client,
//             ToolRegistry::new(),
//             SystemPromptConfig::None,
//             true,
//         );
//         let response = chat_session
//             .send_message("Say 'Hello', nothing else.")
//             .await
//             .unwrap();
//         assert!(response.contains("Hello"));
//     }

//     #[tokio::test]
//     async fn send_message_uses_tools_when_tools_are_provided() {
//         let openai_client = OpenAIClient::new(std::env::var("OPENAI_API_KEY").unwrap());

//         struct TestTool;

//         impl Tool for TestTool {
//             fn name(&self) -> String {
//                 "read_file".to_string()
//             }

//             fn description(&self) -> String {
//                 "Read the content of a file".to_string()
//             }

//             fn parameters(&self) -> Parameters {
//                 Parameters {
//                     param_type: "object".to_string(),
//                     properties: HashMap::from([(
//                         "path".to_string(),
//                         Property {
//                             prop_type: "string".to_string(),
//                             description: "The path to the file to read".to_string(),
//                         },
//                     )]),
//                     required: vec!["path".to_string()],
//                     additional_properties: false,
//                 }
//             }

//             fn strict(&self) -> bool {
//                 true
//             }

//             fn call(&self, _args: HashMap<String, String>) -> Result<String, String> {
//                 Ok("Hello, world!".to_string())
//             }
//         }

//         let mut chat_session = ChatSession::new(
//             openai_client,
//             ToolRegistry::from(vec![Box::new(TestTool)]),
//             SystemPromptConfig::None,
//             true,
//         );

//         chat_session
//             .send_message("Read the content of the poem.txt file")
//             .await
//             .unwrap();
//         if let ChatMessage::Tool { .. } = chat_session.memory.get_memory()[2] {
//             // It's a tool message
//         } else {
//             panic!("Expected a tool message");
//         }
//     }

//     #[test]
//     fn handle_openai_error_fatal_errors_return_fatal() {
//         let openai_client = OpenAIClient::new("test_key".to_string());
//         let chat_session = ChatSession::new(
//             openai_client,
//             ToolRegistry::new(),
//             SystemPromptConfig::None,
//             true,
//         );

//         // Test all fatal error types
//         let fatal_errors = vec![
//             OpenAIError::InvalidApiKey,
//             OpenAIError::MissingApiKey,
//             OpenAIError::InsufficientCredits,
//             OpenAIError::InvalidModel,
//             OpenAIError::EmptyChatHistory,
//         ];

//         for error in fatal_errors {
//             let result = chat_session.handle_openai_error(error, 1);
//             match result {
//                 ChatSessionErrorHandling::Fatal(_) => (), // Expected
//                 _ => panic!("Expected Fatal error handling for fatal error"),
//             }
//         }
//     }

//     #[test]
//     fn handle_openai_error_retryable_errors_retry_then_fatal() {
//         let openai_client = OpenAIClient::new("test_key".to_string());
//         let chat_session = ChatSession::new(
//             openai_client,
//             ToolRegistry::new(),
//             SystemPromptConfig::None,
//             true,
//         );

//         // Test RateLimitExceeded
//         for iteration in 1..=3 {
//             let result =
//                 chat_session.handle_openai_error(OpenAIError::RateLimitExceeded, iteration);
//             match result {
//                 ChatSessionErrorHandling::Retry => (), // Expected
//                 _ => panic!(
//                     "Expected Retry for RateLimitExceeded at iteration {}",
//                     iteration
//                 ),
//             }
//         }
//         let result = chat_session.handle_openai_error(OpenAIError::RateLimitExceeded, 4);
//         match result {
//             ChatSessionErrorHandling::Fatal(_) => (), // Expected
//             _ => panic!("Expected Fatal for RateLimitExceeded at iteration 4"),
//         }

//         // Test ServiceUnavailable
//         for iteration in 1..=3 {
//             let result =
//                 chat_session.handle_openai_error(OpenAIError::ServiceUnavailable, iteration);
//             match result {
//                 ChatSessionErrorHandling::Retry => (), // Expected
//                 _ => panic!(
//                     "Expected Retry for ServiceUnavailable at iteration {}",
//                     iteration
//                 ),
//             }
//         }
//         let result = chat_session.handle_openai_error(OpenAIError::ServiceUnavailable, 4);
//         match result {
//             ChatSessionErrorHandling::Fatal(_) => (), // Expected
//             _ => panic!("Expected Fatal for ServiceUnavailable at iteration 4"),
//         }
//     }

//     #[test]
//     fn handle_openai_error_content_errors_add_to_memory_and_retry() {
//         let openai_client = OpenAIClient::new("test_key".to_string());
//         let chat_session = ChatSession::new(
//             openai_client,
//             ToolRegistry::new(),
//             SystemPromptConfig::None,
//             true,
//         );

//         let content_errors = vec![
//             OpenAIError::InvalidContentResponse,
//             OpenAIError::InvalidToolCallArguments,
//             OpenAIError::NoCompletionFound,
//             OpenAIError::NoChoicesFound,
//             OpenAIError::NoContentFound,
//         ];

//         for error in content_errors {
//             let result = chat_session.handle_openai_error(error, 1);
//             match result {
//                 ChatSessionErrorHandling::AddToMemoryAndRetry(message) => {
//                     assert!(message.contains("error occurred"));
//                     assert!(message.contains("try again"));
//                 }
//                 _ => panic!("Expected AddToMemoryAndRetry for content error"),
//             }
//         }
//     }

//     #[test]
//     fn handle_openai_error_request_errors_add_to_memory_and_retry() {
//         let openai_client = OpenAIClient::new("test_key".to_string());
//         let chat_session = ChatSession::new(
//             openai_client,
//             ToolRegistry::new(),
//             SystemPromptConfig::None,
//             true,
//         );

//         let result = chat_session.handle_openai_error(OpenAIError::InvalidChatMessageRequest, 1);
//         match result {
//             ChatSessionErrorHandling::AddToMemoryAndRetry(message) => {
//                 assert!(message.contains("Invalid request format"));
//                 assert!(message.contains("correct format"));
//             }
//             _ => panic!("Expected AddToMemoryAndRetry for InvalidChatMessageRequest"),
//         }
//     }

//     #[test]
//     fn handle_openai_error_other_errors_add_to_memory_and_retry() {
//         let openai_client = OpenAIClient::new("test_key".to_string());
//         let chat_session = ChatSession::new(
//             openai_client,
//             ToolRegistry::new(),
//             SystemPromptConfig::None,
//             true,
//         );

//         let result = chat_session
//             .handle_openai_error(OpenAIError::Other("Some unexpected error".to_string()), 1);
//         match result {
//             ChatSessionErrorHandling::AddToMemoryAndRetry(message) => {
//                 assert!(message.contains("unexpected error"));
//                 assert!(message.contains("different approach"));
//             }
//             _ => panic!("Expected AddToMemoryAndRetry for Other error"),
//         }
//     }

//     #[test]
//     fn handle_openai_error_preserves_original_error_in_fatal_cases() {
//         let openai_client = OpenAIClient::new("test_key".to_string());
//         let chat_session = ChatSession::new(
//             openai_client,
//             ToolRegistry::new(),
//             SystemPromptConfig::None,
//             true,
//         );

//         let original_error = OpenAIError::InvalidApiKey;
//         let result = chat_session.handle_openai_error(original_error, 1);

//         match result {
//             ChatSessionErrorHandling::Fatal(ChatSessionError::OpenAI(preserved_error)) => {
//                 match preserved_error {
//                     OpenAIError::InvalidApiKey => (), // Expected
//                     _ => panic!("Original error not preserved correctly"),
//                 }
//             }
//             _ => panic!("Expected Fatal with preserved OpenAI error"),
//         }
//     }
// }
