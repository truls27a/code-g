use crate::client::error::{ChatClientError, ErrorRetryStrategy};
use crate::client::model::{AssistantMessage, ChatMessage, ChatResult, Model};
use crate::client::providers::openai::schema::Model as OpenAiModel;
use crate::client::traits::ChatClient;
use crate::session::error::{ChatSessionError, ChatSessionErrorHandling};
use crate::session::event::{Action, Event, EventHandler};
use crate::session::memory::ChatMemory;
use crate::session::system_prompt::{SYSTEM_PROMPT, SystemPromptConfig};
use crate::tools::traits::ToolRegistry;
use std::collections::HashMap;

// Maximum number of iterations per message to prevent infinite loops
const MAX_ITERATIONS: usize = 50;

/// Core component that orchestrates conversations between a user and an AI assistant.
///
/// ChatSession maintains conversation history, handles tool calls, manages errors,
/// and provides event notifications throughout the conversation lifecycle. It includes
/// memory management, tool integration, error handling with retry logic, and loop protection.
///
/// # Examples
///
/// ```rust
/// use code_g::session::session::ChatSession;
/// use code_g::client::providers::openai::client::OpenAIClient;
/// use code_g::tools::registry::Registry;
/// use code_g::session::system_prompt::SystemPromptConfig;
/// use code_g::tui::tui::Tui;
///
/// let client = Box::new(OpenAIClient::new("api_key".to_string()));
/// let tools = Box::new(Registry::new());
/// let event_handler = Box::new(Tui::new());
/// let session = ChatSession::new(client, tools, event_handler, SystemPromptConfig::Default);
/// ```
pub struct ChatSession {
    /// Manages conversation history and context
    memory: ChatMemory,
    /// Chat client for API communication
    client: Box<dyn ChatClient>,
    /// Registry of available tools for the AI to use
    tools: Box<dyn ToolRegistry>,
    /// Handler for events and user interactions
    event_handler: Box<dyn EventHandler>,
}

impl ChatSession {
    /// Creates a new chat session with the specified configuration.
    ///
    /// # Arguments
    ///
    /// * `client` - [`ChatClient`] implementation for API communication
    /// * `tools` - [`Registry`] containing tools available to the AI assistant
    /// * `event_handler` - [`EventHandler`] for processing events and user interactions
    /// * `system_prompt_config` - [`SystemPromptConfig`] for the initial system prompt
    ///
    /// # Returns
    ///
    /// A new `ChatSession` instance ready for conversation.
    pub fn new(
        client: Box<dyn ChatClient>,
        tools: Box<dyn ToolRegistry>,
        event_handler: Box<dyn EventHandler>,
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
            event_handler,
        }
    }

    /// Runs an interactive chat loop that continues until the user exits.
    ///
    /// Provides a complete interactive chat experience by continuously prompting for
    /// user input, processing each message, and displaying responses. The loop exits
    /// when the user types "exit".
    ///
    /// # Returns
    ///
    /// Returns [`Ok(())`] when session completes normally.
    ///
    /// # Errors
    ///
    /// Returns [`ChatSessionError`] if an error occurs during conversation.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use code_g::session::session::ChatSession;
    /// use code_g::client::providers::openai::client::OpenAIClient;
    /// use code_g::tools::registry::Registry;
    /// use code_g::session::system_prompt::SystemPromptConfig;
    /// use code_g::tui::tui::Tui;
    /// use tokio::runtime::Runtime;
    ///
    /// let client = Box::new(OpenAIClient::new("api_key".to_string()));
    /// let tools = Box::new(Registry::new());
    /// let event_handler = Box::new(Tui::new());
    ///
    /// let mut session = ChatSession::new(client, tools, event_handler, SystemPromptConfig::Default);
    ///
    /// // Run the session in a tokio runtime
    /// let rt = Runtime::new().unwrap();
    ///
    /// rt.block_on(session.run());
    /// ```
    pub async fn run(&mut self) -> Result<(), ChatSessionError> {
        // Clear the terminal
        self.event_handler.handle_event(Event::SessionStarted);

        loop {
            let user_input = self
                .event_handler
                .handle_action(Action::RequestUserInput)
                .unwrap(); // TODO: Handle errors

            if user_input == "exit" {
                // Exit the loop
                break;
            }

            self.send_message(&user_input).await?;
        }

        self.event_handler.handle_event(Event::SessionEnded);

        Ok(())
    }

    /// Sends a message to the AI assistant and returns the response.
    ///
    /// This method handles the complete conversation flow: adds the user message to memory,
    /// requests a response from the AI, processes any tool calls, handles errors with
    /// retry logic, update event handler with events, and returns the final assistant response.
    /// The method continues until the AI returns a final message or maximum iterations are reached.
    ///
    /// # Arguments
    ///
    /// * `message` - The user's message to send to the assistant
    ///
    /// # Returns
    ///
    /// The assistant's response message.
    ///
    /// # Errors
    ///
    /// Returns [`ChatSessionError`] for API errors, maximum iteration limit exceeded,
    /// or tool execution failures.
    async fn send_message(&mut self, message: &str) -> Result<String, ChatSessionError> {
        // Add user message to memory
        self.memory.add_message(ChatMessage::User {
            content: message.to_string(),
        });

        // Notify event handler about the user message
        self.event_handler.handle_event(Event::ReceivedUserMessage {
            message: message.to_string(),
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
            self.event_handler
                .handle_event(Event::AwaitingAssistantResponse);

            // 3. Get a response from the client
            let response = match self
                .client
                .create_chat_completion(
                    &Model::OpenAi(OpenAiModel::Gpt4oMini), // TODO: Make this configurable
                    &self.memory.get_memory(),
                    &self.tools.to_tools(),
                )
                .await
            {
                Ok(response) => response,
                Err(e) => match self.handle_chat_client_error(e, iterations) {
                    ChatSessionErrorHandling::Fatal(err) => {
                        return Err(err);
                    }
                    ChatSessionErrorHandling::Retry => {
                        continue;
                    }
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

                    // 5.2 Render the memory to the event handler (only if not silent)
                    self.event_handler
                        .handle_event(Event::ReceivedAssistantMessage {
                            message: content.clone(),
                        });

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
                        self.event_handler.handle_event(Event::ReceivedToolCall {
                            tool_name: tool_call.name.clone(),
                            parameters: tool_call.arguments.clone(),
                        });

                        // 6.2.2 Check if tool requires approval and request if needed
                        let (tool_response, approved) = if self
                            .tools
                            .get_tool(&tool_call.name)
                            .map(|tool| tool.requires_approval())
                            .unwrap_or(false)
                        {
                            match self.request_approval(&tool_call.name, &tool_call.arguments) {
                                Ok(true) => {
                                    // User approved, proceed with tool execution
                                    let response = self
                                        .tools
                                        .call_tool(
                                            tool_call.name.as_str(),
                                            tool_call.arguments.clone(),
                                        )
                                        .unwrap_or_else(|e| e);
                                    (response, true)
                                }
                                Ok(false) => {
                                    // User declined, return cancellation message
                                    let response = format!(
                                        "Operation cancelled by user: {} with parameters {:?}",
                                        tool_call.name, tool_call.arguments
                                    );
                                    (response, false)
                                }
                                Err(e) => {
                                    // Error requesting approval
                                    let response = format!(
                                        "Failed to request approval for {}: {}",
                                        tool_call.name, e
                                    );
                                    (response, false)
                                }
                            }
                        } else {
                            // Tool doesn't require approval, execute directly
                            let response = self
                                .tools
                                .call_tool(tool_call.name.as_str(), tool_call.arguments.clone())
                                .unwrap_or_else(|e| e);
                            (response, true)
                        };

                        // 6.2.3 Add tool response to memory
                        self.memory.add_message(ChatMessage::Tool {
                            content: tool_response.clone(),
                            tool_call_id: tool_call.id.clone(),
                            tool_name: tool_call.name.clone(),
                        });

                        // 6.2.4 Send tool response event to the event handler
                        self.event_handler
                            .handle_event(Event::ReceivedToolResponse {
                                tool_name: tool_call.name.clone(),
                                response: tool_response.clone(),
                                parameters: tool_call.arguments.clone(),
                                approved,
                            });
                    }

                    // 6.3 Continue the loop to get the assistants response
                    continue;
                }
            }
        }
    }

    /// Requests user approval for a potentially dangerous operation.
    ///
    /// This method prompts the user to approve or decline the execution of a tool
    /// that could modify the filesystem or execute system commands.
    ///
    /// # Arguments
    ///
    /// * `tool_name` - The name of the tool requiring approval
    /// * `parameters` - The parameters being passed to the tool
    ///
    /// # Returns
    ///
    /// `true` if the user approved the operation, `false` if declined.
    ///
    /// # Errors
    ///
    /// Returns [`ChatSessionError`] if the approval request fails.
    fn request_approval(
        &mut self,
        tool_name: &str,
        parameters: &HashMap<String, String>,
    ) -> Result<bool, ChatSessionError> {
        let approval_message = if let Some(tool) = self.tools.get_tool(tool_name) {
            tool.approval_message(parameters)
        } else {
            format!("CodeG wants to use tool: {}", tool_name)
        };

        let response = self
            .event_handler
            .handle_action(Action::RequestUserApproval {
                approval_message,
                tool_name: tool_name.to_string(),
            })
            .map_err(|e| {
                ChatSessionError::ToolError(format!("Failed to request approval: {}", e))
            })?;

        Ok(response == "approved")
    }

    /// Categorizes and handles chat client errors with appropriate recovery strategies.
    ///
    /// This method implements error handling logic that uses the error's retry strategy
    /// to determine the best recovery approach. Fatal errors (configuration/account issues)
    /// are returned immediately. Retryable errors (network/service issues) are retried up to 3 times.
    /// Content/request errors inform the AI of the issue and retry.
    ///
    /// # Arguments
    ///
    /// * `error` - The chat client error to handle
    /// * `iteration` - Current iteration number for retry logic
    ///
    /// # Returns
    ///
    /// A [`ChatSessionErrorHandling`] enum indicating the recovery strategy.
    fn handle_chat_client_error(
        &self,
        error: ChatClientError,
        iteration: usize,
    ) -> ChatSessionErrorHandling {
        match error.retry_strategy() {
            ErrorRetryStrategy::Fatal => {
                ChatSessionErrorHandling::Fatal(ChatSessionError::ChatClient(error))
            }
            ErrorRetryStrategy::Retryable => {
                if iteration <= 3 {
                    ChatSessionErrorHandling::Retry
                } else {
                    ChatSessionErrorHandling::Fatal(ChatSessionError::ChatClient(error))
                }
            }
            ErrorRetryStrategy::AddToMemoryAndRetry => {
                let message = format!(
                    "An error occurred: {}. Please try again with a different approach.",
                    error
                );
                ChatSessionErrorHandling::AddToMemoryAndRetry(message)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::client::providers::openai::client::OpenAIClient;
    use crate::tools::registry::Registry;
    use crate::tui::tui::Tui;

    #[test]
    fn new_creates_a_chat_session_with_empty_memory() {
        let openai_client = Box::new(OpenAIClient::new("any_api_key".to_string()));
        let event_handler = Box::new(Tui::new());
        let chat_session = ChatSession::new(
            openai_client,
            Box::new(Registry::new()),
            event_handler,
            SystemPromptConfig::None,
        );
        assert_eq!(chat_session.memory.get_memory().len(), 0);
    }

    #[test]
    fn new_creates_a_chat_session_with_system_prompt_when_default() {
        let openai_client = Box::new(OpenAIClient::new("any_api_key".to_string()));
        let event_handler = Box::new(Tui::new());
        let chat_session = ChatSession::new(
            openai_client,
            Box::new(Registry::new()),
            event_handler,
            SystemPromptConfig::Default,
        );
        assert_eq!(chat_session.memory.get_memory().len(), 1);
        if let ChatMessage::System { content } = &chat_session.memory.get_memory()[0] {
            assert_eq!(content, SYSTEM_PROMPT);
        } else {
            panic!("Expected system message");
        }
    }

    #[test]
    fn new_creates_a_chat_session_with_custom_system_prompt() {
        let openai_client = Box::new(OpenAIClient::new("any_api_key".to_string()));
        let event_handler = Box::new(Tui::new());
        let custom_prompt = "You are a helpful assistant.".to_string();
        let chat_session = ChatSession::new(
            openai_client,
            Box::new(Registry::new()),
            event_handler,
            SystemPromptConfig::Custom(custom_prompt.clone()),
        );
        assert_eq!(chat_session.memory.get_memory().len(), 1);
        if let ChatMessage::System { content } = &chat_session.memory.get_memory()[0] {
            assert_eq!(content, &custom_prompt);
        } else {
            panic!("Expected system message");
        }
    }

    #[test]
    fn handle_chat_client_error_fatal_errors_return_fatal() {
        let openai_client = Box::new(OpenAIClient::new("test_key".to_string()));
        let event_handler = Box::new(Tui::new());
        let chat_session = ChatSession::new(
            openai_client,
            Box::new(Registry::new()),
            event_handler,
            SystemPromptConfig::None,
        );

        // Test all fatal error types
        let fatal_errors = vec![
            ChatClientError::InvalidApiKey,
            ChatClientError::MissingApiKey,
            ChatClientError::InsufficientCredits,
            ChatClientError::InvalidModel,
            ChatClientError::EmptyChatHistory,
        ];

        for error in fatal_errors {
            let result = chat_session.handle_chat_client_error(error, 1);
            match result {
                ChatSessionErrorHandling::Fatal(_) => (), // Expected
                _ => panic!("Expected Fatal error handling for fatal error"),
            }
        }
    }

    #[test]
    fn handle_chat_client_error_retryable_errors_retry_then_fatal() {
        let openai_client = Box::new(OpenAIClient::new("test_key".to_string()));
        let event_handler = Box::new(Tui::new());
        let chat_session = ChatSession::new(
            openai_client,
            Box::new(Registry::new()),
            event_handler,
            SystemPromptConfig::None,
        );

        // Test RateLimitExceeded
        for iteration in 1..=3 {
            let result = chat_session
                .handle_chat_client_error(ChatClientError::RateLimitExceeded, iteration);
            match result {
                ChatSessionErrorHandling::Retry => (), // Expected
                _ => panic!(
                    "Expected Retry for RateLimitExceeded at iteration {}",
                    iteration
                ),
            }
        }
        let result = chat_session.handle_chat_client_error(ChatClientError::RateLimitExceeded, 4);
        match result {
            ChatSessionErrorHandling::Fatal(_) => (), // Expected
            _ => panic!("Expected Fatal for RateLimitExceeded at iteration 4"),
        }

        // Test ServiceUnavailable
        for iteration in 1..=3 {
            let result = chat_session
                .handle_chat_client_error(ChatClientError::ServiceUnavailable, iteration);
            match result {
                ChatSessionErrorHandling::Retry => (), // Expected
                _ => panic!(
                    "Expected Retry for ServiceUnavailable at iteration {}",
                    iteration
                ),
            }
        }
        let result = chat_session.handle_chat_client_error(ChatClientError::ServiceUnavailable, 4);
        match result {
            ChatSessionErrorHandling::Fatal(_) => (), // Expected
            _ => panic!("Expected Fatal for ServiceUnavailable at iteration 4"),
        }
    }

    #[test]
    fn handle_chat_client_error_content_errors_add_to_memory_and_retry() {
        let openai_client = Box::new(OpenAIClient::new("test_key".to_string()));
        let event_handler = Box::new(Tui::new());
        let chat_session = ChatSession::new(
            openai_client,
            Box::new(Registry::new()),
            event_handler,
            SystemPromptConfig::None,
        );

        // These errors are now wrapped in OpenAIError, so we need to create them properly
        use crate::client::providers::openai::error::OpenAIError;

        let content_errors = vec![
            ChatClientError::OpenAIError(OpenAIError::InvalidContentResponse),
            ChatClientError::OpenAIError(OpenAIError::InvalidToolCallArguments),
            ChatClientError::OpenAIError(OpenAIError::NoCompletionFound),
            ChatClientError::OpenAIError(OpenAIError::NoChoicesFound),
            ChatClientError::OpenAIError(OpenAIError::NoContentFound),
        ];

        for error in content_errors {
            let result = chat_session.handle_chat_client_error(error, 1);
            match result {
                ChatSessionErrorHandling::AddToMemoryAndRetry(message) => {
                    assert!(message.contains("error occurred"));
                    assert!(message.contains("try again"));
                }
                _ => panic!("Expected AddToMemoryAndRetry for content error"),
            }
        }
    }

    #[test]
    fn handle_chat_client_error_request_errors_add_to_memory_and_retry() {
        let openai_client = Box::new(OpenAIClient::new("test_key".to_string()));
        let event_handler = Box::new(Tui::new());
        let chat_session = ChatSession::new(
            openai_client,
            Box::new(Registry::new()),
            event_handler,
            SystemPromptConfig::None,
        );

        let result =
            chat_session.handle_chat_client_error(ChatClientError::InvalidChatMessageRequest, 1);
        match result {
            ChatSessionErrorHandling::AddToMemoryAndRetry(message) => {
                assert!(message.contains("error occurred"));
                assert!(message.contains("try again"));
            }
            _ => panic!("Expected AddToMemoryAndRetry for InvalidChatMessageRequest"),
        }
    }

    #[test]
    fn handle_chat_client_error_other_errors_add_to_memory_and_retry() {
        let openai_client = Box::new(OpenAIClient::new("test_key".to_string()));
        let event_handler = Box::new(Tui::new());
        let chat_session = ChatSession::new(
            openai_client,
            Box::new(Registry::new()),
            event_handler,
            SystemPromptConfig::None,
        );

        let result = chat_session.handle_chat_client_error(
            ChatClientError::Other("Some unexpected error".to_string()),
            1,
        );
        match result {
            ChatSessionErrorHandling::AddToMemoryAndRetry(message) => {
                assert!(message.contains("error occurred"));
                assert!(message.contains("try again"));
            }
            _ => panic!("Expected AddToMemoryAndRetry for Other error"),
        }
    }

    #[test]
    fn handle_chat_client_error_preserves_original_error_in_fatal_cases() {
        let openai_client = Box::new(OpenAIClient::new("test_key".to_string()));
        let event_handler = Box::new(Tui::new());
        let chat_session = ChatSession::new(
            openai_client,
            Box::new(Registry::new()),
            event_handler,
            SystemPromptConfig::None,
        );

        let original_error = ChatClientError::InvalidApiKey;
        let result = chat_session.handle_chat_client_error(original_error, 1);

        match result {
            ChatSessionErrorHandling::Fatal(ChatSessionError::ChatClient(preserved_error)) => {
                match preserved_error {
                    ChatClientError::InvalidApiKey => (), // Expected
                    _ => panic!("Original error not preserved correctly"),
                }
            }
            _ => panic!("Expected Fatal with preserved chat client error"),
        }
    }
}
