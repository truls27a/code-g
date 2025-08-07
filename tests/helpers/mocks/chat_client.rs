use async_trait::async_trait;
use code_g::client::error::ChatClientError;
use code_g::client::model::{ChatMessage, ChatResult, Model, Tool, ToolCall};
use code_g::client::providers::openai::error::OpenAIError;
use code_g::client::traits::ChatClient;

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
/// use code_g::chat_client::providers::openai::schema::Model as OpenAiModel;
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
    call_count: std::sync::Arc<std::sync::Mutex<usize>>,
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
    /// Return a sequence of responses in order
    Sequence(Vec<MockResponse>),
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
            call_count: std::sync::Arc::new(std::sync::Mutex::new(0)),
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
            call_count: std::sync::Arc::new(std::sync::Mutex::new(0)),
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
            call_count: std::sync::Arc::new(std::sync::Mutex::new(0)),
        }
    }

    /// Creates a new mock client that returns a sequence of responses.
    ///
    /// The client will return responses in the order they are provided.
    /// After all responses are exhausted, it will repeat the last response.
    ///
    /// # Arguments
    ///
    /// * `responses` - The sequence of responses to return
    ///
    /// # Returns
    ///
    /// A new `MockChatClient` configured to return the sequence of responses.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use code_g::chat_client::mock::{MockChatClient, MockResponse};
    ///
    /// let responses = vec![
    ///     MockResponse::ToolCalls(vec![/* tool calls */]),
    ///     MockResponse::Message {
    ///         content: "Final response".to_string(),
    ///         turn_over: true
    ///     },
    /// ];
    /// let mock = MockChatClient::new_with_sequence(responses);
    /// ```
    pub fn new_with_sequence(responses: Vec<MockResponse>) -> Self {
        Self {
            response: MockResponse::Sequence(responses),
            call_count: std::sync::Arc::new(std::sync::Mutex::new(0)),
        }
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
            MockResponse::Sequence(responses) => {
                let mut count = self.call_count.lock().unwrap();
                let index = *count;
                *count += 1;

                if index < responses.len() {
                    match &responses[index] {
                        MockResponse::Message { content, turn_over } => Ok(ChatResult::Message {
                            content: content.clone(),
                            turn_over: *turn_over,
                        }),
                        MockResponse::ToolCalls(tool_calls) => {
                            Ok(ChatResult::ToolCalls(tool_calls.clone()))
                        }
                        MockResponse::Error(error_message) => Err(ChatClientError::OpenAIError(
                            OpenAIError::Other(error_message.clone()),
                        )),
                        MockResponse::Sequence(_) => {
                            // Nested sequences not supported, fall back to last response
                            if let Some(last) = responses.last() {
                                match last {
                                    MockResponse::Message { content, turn_over } => {
                                        Ok(ChatResult::Message {
                                            content: content.clone(),
                                            turn_over: *turn_over,
                                        })
                                    }
                                    MockResponse::ToolCalls(tool_calls) => {
                                        Ok(ChatResult::ToolCalls(tool_calls.clone()))
                                    }
                                    MockResponse::Error(error_message) => {
                                        Err(ChatClientError::OpenAIError(OpenAIError::Other(
                                            error_message.clone(),
                                        )))
                                    }
                                    MockResponse::Sequence(_) => Ok(ChatResult::Message {
                                        content: "Default response".to_string(),
                                        turn_over: true,
                                    }),
                                }
                            } else {
                                Ok(ChatResult::Message {
                                    content: "Default response".to_string(),
                                    turn_over: true,
                                })
                            }
                        }
                    }
                } else {
                    // Repeat the last response
                    if let Some(last) = responses.last() {
                        match last {
                            MockResponse::Message { content, turn_over } => {
                                Ok(ChatResult::Message {
                                    content: content.clone(),
                                    turn_over: *turn_over,
                                })
                            }
                            MockResponse::ToolCalls(tool_calls) => {
                                Ok(ChatResult::ToolCalls(tool_calls.clone()))
                            }
                            MockResponse::Error(error_message) => {
                                Err(ChatClientError::OpenAIError(OpenAIError::Other(
                                    error_message.clone(),
                                )))
                            }
                            MockResponse::Sequence(_) => Ok(ChatResult::Message {
                                content: "Default response".to_string(),
                                turn_over: true,
                            }),
                        }
                    } else {
                        Ok(ChatResult::Message {
                            content: "Default response".to_string(),
                            turn_over: true,
                        })
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use code_g::client::model::{ChatMessage, Model};
    use code_g::client::providers::openai::schema::Model as OpenAiModel;
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

}
