use crate::chat::error::{ChatSessionError, ChatSessionErrorHandling};
use crate::chat::event::{Action, Event, EventHandler};
use crate::chat::memory::ChatMemory;
use crate::chat::system_prompt::{SYSTEM_PROMPT, SystemPromptConfig};
use crate::openai::error::OpenAIError;
use crate::openai::model::{AssistantMessage, ChatMessage, ChatResult, OpenAiModel};
use crate::openai::traits::ChatClient;
use crate::tools::registry::Registry;
use std::collections::HashMap;

// Maximum number of iterations per message to prevent infinite loops
const MAX_ITERATIONS: usize = 10;

/// Core component that orchestrates conversations between a user and an AI assistant.
///
/// ChatSession maintains conversation history, handles tool calls, manages errors,
/// and provides event notifications throughout the conversation lifecycle. It includes
/// memory management, tool integration, error handling with retry logic, and loop protection.
///
/// # Examples
///
/// ```rust
/// use code_g::chat::session::ChatSession;
/// use code_g::openai::client::OpenAIClient;
/// use code_g::tools::registry::Registry;
/// use code_g::chat::system_prompt::SystemPromptConfig;
/// use code_g::tui::tui::Tui;
///
/// let client = Box::new(OpenAIClient::new("api_key".to_string()));
/// let tools = Registry::new();
/// let event_handler = Box::new(Tui::new());
/// let session = ChatSession::new(client, tools, event_handler, SystemPromptConfig::Default);
/// ```
pub struct ChatSession {
    /// Manages conversation history and context
    memory: ChatMemory,
    /// Chat client for API communication
    client: Box<dyn ChatClient>,
    /// Registry of available tools for the AI to use
    tools: Registry,
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
        tools: Registry,
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
        let (operation, details) = if let Some(tool) = self.tools.get_tool(tool_name) {
            tool.approval_message(parameters)
        } else {
            (
                "Unknown Operation".to_string(),
                format!("Tool: {}", tool_name),
            )
        };

        let response = self
            .event_handler
            .handle_action(Action::RequestUserApproval {
                operation,
                details,
                tool_name: tool_name.to_string(),
            })
            .map_err(|e| {
                ChatSessionError::ToolError(format!("Failed to request approval: {}", e))
            })?;

        Ok(response == "approved")
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
    pub async fn send_message(&mut self, message: &str) -> Result<String, ChatSessionError> {
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
                        let tool_response = if self
                            .tools
                            .get_tool(&tool_call.name)
                            .map(|tool| tool.requires_approval())
                            .unwrap_or(false)
                        {
                            match self.request_approval(&tool_call.name, &tool_call.arguments) {
                                Ok(true) => {
                                    // User approved, proceed with tool execution
                                    self.tools
                                        .call_tool(
                                            tool_call.name.as_str(),
                                            tool_call.arguments.clone(),
                                        )
                                        .unwrap_or_else(|e| e)
                                }
                                Ok(false) => {
                                    // User declined, return cancellation message
                                    format!(
                                        "Operation cancelled by user: {} with parameters {:?}",
                                        tool_call.name, tool_call.arguments
                                    )
                                }
                                Err(e) => {
                                    // Error requesting approval
                                    format!(
                                        "Failed to request approval for {}: {}",
                                        tool_call.name, e
                                    )
                                }
                            }
                        } else {
                            // Tool doesn't require approval, execute directly
                            self.tools
                                .call_tool(tool_call.name.as_str(), tool_call.arguments.clone())
                                .unwrap_or_else(|e| e)
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
                            });
                    }

                    // 6.3 Continue the loop to get the assistants response
                    continue;
                }
            }
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

    /// Categorizes and handles OpenAI errors with appropriate recovery strategies.
    ///
    /// This method implements error handling logic that categorizes OpenAI API errors
    /// and determines the best recovery strategy. Fatal errors (configuration/account issues)
    /// are returned immediately. Retryable errors (network/service issues) are retried up to 3 times.
    /// Content/request errors inform the AI of the issue and retry.
    ///
    /// # Arguments
    ///
    /// * `error` - The OpenAI error to handle
    /// * `iteration` - Current iteration number for retry logic
    ///
    /// # Returns
    ///
    /// A [`ChatSessionErrorHandling`] enum indicating the recovery strategy.
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::openai::client::OpenAIClient;
    use std::collections::HashMap;
    use std::io;

    // Mock event handler for testing
    struct MockEventHandler {
        events: Vec<Event>,
        input_responses: Vec<String>,
        current_input_index: usize,
    }

    impl MockEventHandler {
        fn new() -> Self {
            Self {
                events: Vec::new(),
                input_responses: Vec::new(),
                current_input_index: 0,
            }
        }

        fn with_input_responses(responses: Vec<String>) -> Self {
            Self {
                events: Vec::new(),
                input_responses: responses,
                current_input_index: 0,
            }
        }

        fn get_events(&self) -> &Vec<Event> {
            &self.events
        }
    }

    impl EventHandler for MockEventHandler {
        fn handle_event(&mut self, event: Event) {
            self.events.push(event);
        }

        fn handle_action(&mut self, action: Action) -> Result<String, io::Error> {
            match action {
                Action::RequestUserInput => {
                    if self.current_input_index < self.input_responses.len() {
                        let response = self.input_responses[self.current_input_index].clone();
                        self.current_input_index += 1;
                        Ok(response)
                    } else {
                        Ok("exit".to_string())
                    }
                }
                Action::RequestUserApproval { .. } => {
                    // For tests, always approve
                    Ok("approved".to_string())
                }
            }
        }
    }

    #[test]
    fn new_creates_a_chat_session_with_empty_memory() {
        let openai_client = Box::new(OpenAIClient::new("any_api_key".to_string()));
        let event_handler = Box::new(MockEventHandler::new());
        let chat_session = ChatSession::new(
            openai_client,
            Registry::new(),
            event_handler,
            SystemPromptConfig::None,
        );
        assert_eq!(chat_session.memory.get_memory().len(), 0);
    }

    #[test]
    fn new_creates_a_chat_session_with_system_prompt_when_default() {
        let openai_client = Box::new(OpenAIClient::new("any_api_key".to_string()));
        let event_handler = Box::new(MockEventHandler::new());
        let chat_session = ChatSession::new(
            openai_client,
            Registry::new(),
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
        let event_handler = Box::new(MockEventHandler::new());
        let custom_prompt = "You are a helpful assistant.".to_string();
        let chat_session = ChatSession::new(
            openai_client,
            Registry::new(),
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
    fn handle_openai_error_fatal_errors_return_fatal() {
        let openai_client = Box::new(OpenAIClient::new("test_key".to_string()));
        let event_handler = Box::new(MockEventHandler::new());
        let chat_session = ChatSession::new(
            openai_client,
            Registry::new(),
            event_handler,
            SystemPromptConfig::None,
        );

        // Test all fatal error types
        let fatal_errors = vec![
            OpenAIError::InvalidApiKey,
            OpenAIError::MissingApiKey,
            OpenAIError::InsufficientCredits,
            OpenAIError::InvalidModel,
            OpenAIError::EmptyChatHistory,
        ];

        for error in fatal_errors {
            let result = chat_session.handle_openai_error(error, 1);
            match result {
                ChatSessionErrorHandling::Fatal(_) => (), // Expected
                _ => panic!("Expected Fatal error handling for fatal error"),
            }
        }
    }

    #[test]
    fn handle_openai_error_retryable_errors_retry_then_fatal() {
        let openai_client = Box::new(OpenAIClient::new("test_key".to_string()));
        let event_handler = Box::new(MockEventHandler::new());
        let chat_session = ChatSession::new(
            openai_client,
            Registry::new(),
            event_handler,
            SystemPromptConfig::None,
        );

        // Test RateLimitExceeded
        for iteration in 1..=3 {
            let result =
                chat_session.handle_openai_error(OpenAIError::RateLimitExceeded, iteration);
            match result {
                ChatSessionErrorHandling::Retry => (), // Expected
                _ => panic!(
                    "Expected Retry for RateLimitExceeded at iteration {}",
                    iteration
                ),
            }
        }
        let result = chat_session.handle_openai_error(OpenAIError::RateLimitExceeded, 4);
        match result {
            ChatSessionErrorHandling::Fatal(_) => (), // Expected
            _ => panic!("Expected Fatal for RateLimitExceeded at iteration 4"),
        }

        // Test ServiceUnavailable
        for iteration in 1..=3 {
            let result =
                chat_session.handle_openai_error(OpenAIError::ServiceUnavailable, iteration);
            match result {
                ChatSessionErrorHandling::Retry => (), // Expected
                _ => panic!(
                    "Expected Retry for ServiceUnavailable at iteration {}",
                    iteration
                ),
            }
        }
        let result = chat_session.handle_openai_error(OpenAIError::ServiceUnavailable, 4);
        match result {
            ChatSessionErrorHandling::Fatal(_) => (), // Expected
            _ => panic!("Expected Fatal for ServiceUnavailable at iteration 4"),
        }
    }

    #[test]
    fn handle_openai_error_content_errors_add_to_memory_and_retry() {
        let openai_client = Box::new(OpenAIClient::new("test_key".to_string()));
        let event_handler = Box::new(MockEventHandler::new());
        let chat_session = ChatSession::new(
            openai_client,
            Registry::new(),
            event_handler,
            SystemPromptConfig::None,
        );

        let content_errors = vec![
            OpenAIError::InvalidContentResponse,
            OpenAIError::InvalidToolCallArguments,
            OpenAIError::NoCompletionFound,
            OpenAIError::NoChoicesFound,
            OpenAIError::NoContentFound,
        ];

        for error in content_errors {
            let result = chat_session.handle_openai_error(error, 1);
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
    fn handle_openai_error_request_errors_add_to_memory_and_retry() {
        let openai_client = Box::new(OpenAIClient::new("test_key".to_string()));
        let event_handler = Box::new(MockEventHandler::new());
        let chat_session = ChatSession::new(
            openai_client,
            Registry::new(),
            event_handler,
            SystemPromptConfig::None,
        );

        let result = chat_session.handle_openai_error(OpenAIError::InvalidChatMessageRequest, 1);
        match result {
            ChatSessionErrorHandling::AddToMemoryAndRetry(message) => {
                assert!(message.contains("Invalid request format"));
                assert!(message.contains("correct format"));
            }
            _ => panic!("Expected AddToMemoryAndRetry for InvalidChatMessageRequest"),
        }
    }

    #[test]
    fn handle_openai_error_other_errors_add_to_memory_and_retry() {
        let openai_client = Box::new(OpenAIClient::new("test_key".to_string()));
        let event_handler = Box::new(MockEventHandler::new());
        let chat_session = ChatSession::new(
            openai_client,
            Registry::new(),
            event_handler,
            SystemPromptConfig::None,
        );

        let result = chat_session
            .handle_openai_error(OpenAIError::Other("Some unexpected error".to_string()), 1);
        match result {
            ChatSessionErrorHandling::AddToMemoryAndRetry(message) => {
                assert!(message.contains("unexpected error"));
                assert!(message.contains("different approach"));
            }
            _ => panic!("Expected AddToMemoryAndRetry for Other error"),
        }
    }

    #[test]
    fn handle_openai_error_preserves_original_error_in_fatal_cases() {
        let openai_client = Box::new(OpenAIClient::new("test_key".to_string()));
        let event_handler = Box::new(MockEventHandler::new());
        let chat_session = ChatSession::new(
            openai_client,
            Registry::new(),
            event_handler,
            SystemPromptConfig::None,
        );

        let original_error = OpenAIError::InvalidApiKey;
        let result = chat_session.handle_openai_error(original_error, 1);

        match result {
            ChatSessionErrorHandling::Fatal(ChatSessionError::OpenAI(preserved_error)) => {
                match preserved_error {
                    OpenAIError::InvalidApiKey => (), // Expected
                    _ => panic!("Original error not preserved correctly"),
                }
            }
            _ => panic!("Expected Fatal with preserved OpenAI error"),
        }
    }

    #[test]
    fn event_handler_receives_events() {
        let mut mock_handler = MockEventHandler::new();

        // Test that we can add events to the mock handler
        mock_handler.handle_event(Event::SessionStarted);
        mock_handler.handle_event(Event::ReceivedUserMessage {
            message: "Hello".to_string(),
        });

        let events = mock_handler.get_events();
        assert_eq!(events.len(), 2);
        assert_eq!(events[0], Event::SessionStarted);
        assert_eq!(
            events[1],
            Event::ReceivedUserMessage {
                message: "Hello".to_string()
            }
        );
    }

    #[test]
    fn event_handler_handles_input_requests() {
        let mut mock_handler = MockEventHandler::with_input_responses(vec![
            "Hello".to_string(),
            "How are you?".to_string(),
        ]);

        let response1 = mock_handler.handle_action(Action::RequestUserInput);
        assert_eq!(response1.unwrap(), "Hello");

        let response2 = mock_handler.handle_action(Action::RequestUserInput);
        assert_eq!(response2.unwrap(), "How are you?");

        // After exhausting responses, should return "exit"
        let response3 = mock_handler.handle_action(Action::RequestUserInput);
        assert_eq!(response3.unwrap(), "exit");
    }

    #[test]
    fn requires_approval_identifies_dangerous_tools() {
        let registry = Registry::all_tools();

        // Dangerous tools should require approval
        assert!(registry.get_tool("edit_file").unwrap().requires_approval());
        assert!(registry.get_tool("write_file").unwrap().requires_approval());
        assert!(
            registry
                .get_tool("execute_command")
                .unwrap()
                .requires_approval()
        );

        // Safe tools should not require approval
        assert!(!registry.get_tool("read_file").unwrap().requires_approval());
        assert!(
            !registry
                .get_tool("search_files")
                .unwrap()
                .requires_approval()
        );
    }

    #[test]
    fn request_approval_returns_true_when_approved() {
        let openai_client = Box::new(OpenAIClient::new("test_key".to_string()));
        let event_handler = Box::new(MockEventHandler::new());
        let mut chat_session = ChatSession::new(
            openai_client,
            Registry::new(),
            event_handler,
            SystemPromptConfig::None,
        );

        let mut parameters = HashMap::new();
        parameters.insert("path".to_string(), "test.txt".to_string());
        parameters.insert("content".to_string(), "Hello world".to_string());

        let result = chat_session.request_approval("write_file", &parameters);
        assert!(result.is_ok());
        assert!(result.unwrap()); // MockEventHandler always approves
    }

    #[test]
    fn event_handler_handles_approval_requests() {
        let mut mock_handler = MockEventHandler::new();

        let response = mock_handler.handle_action(Action::RequestUserApproval {
            operation: "Edit File".to_string(),
            details: "File: test.txt\nReplace: old\nWith: new".to_string(),
            tool_name: "edit_file".to_string(),
        });

        assert_eq!(response.unwrap(), "approved");
    }
}
