use crate::chat_client::error::ChatClientError;
use crate::chat_client::model::{ChatMessage, ChatResult, Model, Tool, ToolCall};
use crate::chat_client::providers::openai::error::OpenAIError;
use crate::chat_client::traits::ChatClient;
use async_trait::async_trait;

/// Mock implementation of ChatClient for testing purposes.
///
/// This mock client allows you to configure predefined responses for testing
/// without making actual API calls to OpenAI. It supports both message responses
/// and tool call responses, and can be configured to return errors for testing
/// error handling scenarios.
///
/// # Examples
///
/// ```rust
/// use code_g::chat_client::mock::MockChatClient;
/// use code_g::chat_client::model::{ChatResult, ChatMessage};
/// use code_g::chat_client::providers::openai::schema::OpenAiModel;
/// use code_g::chat_client::traits::ChatClient;
/// use tokio::runtime::Runtime;
///
/// // Create a mock that returns a simple message
/// let mock = MockChatClient::new_with_message("Hello from mock!".to_string(), true);
///
/// // Use it like any other ChatClient
/// let rt = Runtime::new().unwrap();
/// rt.block_on(async {
///     let result = mock.create_chat_completion(
///         &Model::OpenAi(OpenAiModel::Gpt4oMini),
///         &[ChatMessage::User { content: "Hi".to_string() }],
///         &[],
///     ).await.unwrap();
/// });
/// ```
#[derive(Debug, Clone)]
pub struct MockChatClient {
    response: MockResponse,
}

/// Represents the different types of responses the mock client can return.
#[derive(Debug, Clone)]
pub enum MockResponse {
    /// Return a message response
    Message { content: String, turn_over: bool },
    /// Return tool calls
    ToolCalls(Vec<ToolCall>),
    /// Return an error with error message
    Error(String),
}

impl MockChatClient {
    /// Creates a new mock client that returns a message response.
    ///
    /// # Arguments
    ///
    /// * `content` - The message content to return
    /// * `turn_over` - Whether the turn should be marked as over
    ///
    /// # Returns
    ///
    /// A new `MockChatClient` configured to return the specified message.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use code_g::chat_client::mock::MockChatClient;
    ///
    /// let mock = MockChatClient::new_with_message("Test response".to_string(), true);
    /// ```
    pub fn new_with_message(content: String, turn_over: bool) -> Self {
        Self {
            response: MockResponse::Message { content, turn_over },
        }
    }

    /// Creates a new mock client that returns tool calls.
    ///
    /// # Arguments
    ///
    /// * `tool_calls` - The tool calls to return
    ///
    /// # Returns
    ///
    /// A new `MockChatClient` configured to return the specified tool calls.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use code_g::chat_client::mock::MockChatClient;
    /// use code_g::chat_client::model::ToolCall;
    /// use std::collections::HashMap;
    ///
    /// let mut args = HashMap::new();
    /// args.insert("file".to_string(), "test.txt".to_string());
    ///
    /// let tool_call = ToolCall {
    ///     id: "call_123".to_string(),
    ///     name: "read_file".to_string(),
    ///     arguments: args,
    /// };
    ///
    /// let mock = MockChatClient::new_with_tool_calls(vec![tool_call]);
    /// ```
    pub fn new_with_tool_calls(tool_calls: Vec<ToolCall>) -> Self {
        Self {
            response: MockResponse::ToolCalls(tool_calls),
        }
    }

    /// Creates a new mock client that returns an error.
    ///
    /// # Arguments
    ///
    /// * `error_message` - The error message to return
    ///
    /// # Returns
    ///
    /// A new `MockChatClient` configured to return the specified error.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use code_g::chat_client::mock::MockChatClient;
    ///
    /// let mock = MockChatClient::new_with_error("Invalid API key".to_string());
    /// ```
    pub fn new_with_error(error_message: String) -> Self {
        Self {
            response: MockResponse::Error(error_message),
        }
    }

    /// Creates a simple mock client that returns "Mock response" with turn_over = true.
    ///
    /// This is a convenience method for quickly creating a basic mock for testing.
    ///
    /// # Returns
    ///
    /// A new `MockChatClient` with a default response.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use code_g::chat_client::mock::MockChatClient;
    ///
    /// let mock = MockChatClient::default();
    /// ```
    pub fn default() -> Self {
        Self::new_with_message("Mock response".to_string(), true)
    }
}

#[async_trait]
impl ChatClient for MockChatClient {
    async fn create_chat_completion(
        &self,
        _model: &Model,
        _chat_history: &[ChatMessage],
        _tools: &[Tool],
    ) -> Result<ChatResult, ChatClientError> {
        match &self.response {
            MockResponse::Message { content, turn_over } => Ok(ChatResult::Message {
                content: content.clone(),
                turn_over: *turn_over,
            }),
            MockResponse::ToolCalls(tool_calls) => Ok(ChatResult::ToolCalls(tool_calls.clone())),
            MockResponse::Error(error_message) => Err(ChatClientError::OpenAIError(
                OpenAIError::Other(error_message.clone()),
            )),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::chat_client::model::{ChatMessage, Model};
    use crate::chat_client::providers::openai::schema::OpenAiModel;
    use std::collections::HashMap;

    #[tokio::test]
    async fn mock_returns_configured_message() {
        let mock = MockChatClient::new_with_message("Test message".to_string(), true);

        let result = mock
            .create_chat_completion(
                &Model::OpenAi(OpenAiModel::Gpt4oMini),
                &[ChatMessage::User {
                    content: "Hello".to_string(),
                }],
                &[],
            )
            .await;

        assert!(result.is_ok());
        match result.unwrap() {
            ChatResult::Message { content, turn_over } => {
                assert_eq!(content, "Test message");
                assert_eq!(turn_over, true);
            }
            _ => panic!("Expected message result"),
        }
    }

    #[tokio::test]
    async fn mock_returns_configured_tool_calls() {
        let mut args = HashMap::new();
        args.insert("param".to_string(), "value".to_string());

        let tool_call = ToolCall {
            id: "call_test".to_string(),
            name: "test_tool".to_string(),
            arguments: args,
        };

        let mock = MockChatClient::new_with_tool_calls(vec![tool_call.clone()]);

        let result = mock
            .create_chat_completion(
                &Model::OpenAi(OpenAiModel::Gpt4oMini),
                &[ChatMessage::User {
                    content: "Hello".to_string(),
                }],
                &[],
            )
            .await;

        assert!(result.is_ok());
        match result.unwrap() {
            ChatResult::ToolCalls(calls) => {
                assert_eq!(calls.len(), 1);
                assert_eq!(calls[0], tool_call);
            }
            _ => panic!("Expected tool calls result"),
        }
    }

    #[tokio::test]
    async fn mock_returns_configured_error() {
        let mock = MockChatClient::new_with_error("Test error".to_string());

        let result = mock
            .create_chat_completion(
                &Model::OpenAi(OpenAiModel::Gpt4oMini),
                &[ChatMessage::User {
                    content: "Hello".to_string(),
                }],
                &[],
            )
            .await;

        assert!(result.is_err());
        match result.unwrap_err() {
            ChatClientError::OpenAIError(OpenAIError::Other(message)) => {
                assert_eq!(message, "Test error");
            }
            _ => panic!("Expected Other error"),
        }
    }

    #[tokio::test]
    async fn default_mock_returns_default_message() {
        let mock = MockChatClient::default();

        let result = mock
            .create_chat_completion(
                &Model::OpenAi(OpenAiModel::Gpt4oMini),
                &[ChatMessage::User {
                    content: "Hello".to_string(),
                }],
                &[],
            )
            .await;

        assert!(result.is_ok());
        match result.unwrap() {
            ChatResult::Message { content, turn_over } => {
                assert_eq!(content, "Mock response");
                assert_eq!(turn_over, true);
            }
            _ => panic!("Expected message result"),
        }
    }
}
