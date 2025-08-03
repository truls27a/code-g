use crate::chat_client::error::ChatClientError;
use crate::chat_client::providers::openai::error::OpenAIError;
use crate::chat_client::model::{ChatMessage, ChatResult, OpenAiModel, Tool, ToolCall};
use crate::chat_client::providers::openai::schema::{
    ChatCompletionRequest, ChatCompletionResponse, ChatMessageRequest, ContentResponse, JsonSchema,
    ResponseFormat,
};
use crate::chat_client::traits::ChatClient;
use async_trait::async_trait;
use reqwest::Client;
use std::collections::HashMap;

/// HTTP client for interacting with the OpenAI Chat Completions API.
///
/// This struct provides a high-level interface for making chat completion requests
/// to the OpenAI API. It handles authentication, request formatting, response parsing,
/// and error handling, abstracting away the low-level HTTP details and providing
/// a convenient Rust API for OpenAI interactions.
///
/// # Fields
///
/// * `client` - The underlying HTTP client for making requests
/// * `api_key` - The OpenAI API key for authentication
///
/// # Examples
///
/// ```rust,no_run
/// use code_g::chat_client::providers::openai::client::OpenAIClient;
/// use code_g::chat_client::model::{ChatMessage, OpenAiModel};
/// use code_g::chat_client::traits::ChatClient;
/// use tokio::runtime::Runtime;
///
/// let client = OpenAIClient::new("your-api-key".to_string());
///
/// let chat_history = vec![
///     ChatMessage::User {
///         content: "Hello, how are you?".to_string(),
///     }
/// ];
///
/// let rt = Runtime::new().unwrap();
/// let result = rt.block_on(client
///     .create_chat_completion(&OpenAiModel::Gpt4oMini, &chat_history, &[]));
/// ```
pub struct OpenAIClient {
    client: Client,
    api_key: String,
}

impl OpenAIClient {
    /// Creates a new OpenAI client with the provided API key.
    ///
    /// This constructor initializes a new HTTP client and stores the API key
    /// for use in subsequent requests. The API key should be a valid OpenAI
    /// API key obtained from the OpenAI platform.
    ///
    /// # Arguments
    ///
    /// * `api_key` - A valid OpenAI API key for authentication
    ///
    /// # Returns
    ///
    /// A new [`OpenAIClient`] instance ready to make API requests.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use code_g::chat_client::providers::openai::client::OpenAIClient;
    ///
    /// let client = OpenAIClient::new("sk-your-api-key-here".to_string());
    /// ```
    pub fn new(api_key: String) -> Self {
        Self {
            client: Client::new(),
            api_key,
        }
    }

}

#[async_trait]
impl ChatClient for OpenAIClient {
    /// Creates a chat completion request to the OpenAI API.
    ///
    /// This method sends a chat completion request with the specified model,
    /// conversation history, and available tools. It handles the complete
    /// request/response cycle, including authentication, JSON schema formatting
    /// for structured responses, and error handling. The response can be either
    /// a text message or tool calls that need to be executed.
    ///
    /// # Arguments
    ///
    /// * `model` - The OpenAI model to use for the completion
    /// * `chat_history` - The conversation history as a slice of chat messages
    /// * `tools` - Available tools/functions that the assistant can call
    ///
    /// # Returns
    ///
    /// A [`ChatResult`] containing either a message response or tool calls.
    ///
    /// # Errors
    ///
    /// Returns a [`ChatClientError`] in the following cases:
    /// - [`ChatClientError::EmptyChatHistory`] if the chat history is empty
    /// - [`ChatClientError::InvalidChatMessageRequest`] if message conversion fails
    /// - [`ChatClientError::InvalidApiKey`] if the API key is invalid (HTTP 401)
    /// - [`ChatClientError::InsufficientCredits`] if the account has no credits (HTTP 403)
    /// - [`ChatClientError::RateLimitExceeded`] if rate limits are hit (HTTP 429)
    /// - [`ChatClientError::InvalidModel`] if the model is not found (HTTP 404)
    /// - [`ChatClientError::ServiceUnavailable`] if OpenAI service is down (HTTP 500)
    /// - [`ChatClientError::Other`] for other network-related errors
    /// - [`ChatClientError::OpenAIError`] for OpenAI-specific errors
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use code_g::chat_client::providers::openai::client::OpenAIClient;
    /// use code_g::chat_client::model::{ChatMessage, ChatResult, OpenAiModel};
    /// use code_g::chat_client::traits::ChatClient;
    /// use tokio::runtime::Runtime;
    ///
    /// let client = OpenAIClient::new("your-api-key".to_string());
    ///
    /// let chat_history = vec![
    ///     ChatMessage::System {
    ///         content: "You are a helpful assistant.".to_string(),
    ///     },
    ///     ChatMessage::User {
    ///         content: "What's the capital of France?".to_string(),
    ///     },
    /// ];
    ///
    /// let rt = Runtime::new().unwrap();
    /// match rt.block_on(client
    ///     .create_chat_completion(&OpenAiModel::Gpt4oMini, &chat_history, &[]))
    /// {
    ///     Ok(ChatResult::Message { content, turn_over }) => {
    ///         println!("Assistant: {}", content);
    ///         println!("Turn over: {}", turn_over);
    ///     }
    ///     Ok(ChatResult::ToolCalls(calls)) => {
    ///         println!("Assistant wants to call {} tools", calls.len());
    ///     }
    ///     Err(e) => {
    ///         eprintln!("Error: {:?}", e);
    ///     }
    /// }
    /// ```
    async fn create_chat_completion(
        &self,
        model: &OpenAiModel,
        chat_history: &[ChatMessage],
        tools: &[Tool],
    ) -> Result<ChatResult, ChatClientError> {
        if chat_history.is_empty() {
            return Err(ChatClientError::EmptyChatHistory);
        }

        let request_body = ChatCompletionRequest {
            model: model.clone(),
            messages: chat_history
                .iter()
                .map(|m| ChatMessageRequest::try_from(m.clone()))
                .collect::<Result<Vec<ChatMessageRequest>, serde_json::Error>>()
                .map_err(|_| ChatClientError::InvalidChatMessageRequest)?,
            tools: Some(tools.to_vec()),
            response_format: Some(ResponseFormat {
                response_format_type: "json_schema".to_string(),
                json_schema: JsonSchema {
                    name: "structured_chat_response".to_string(),
                    schema: serde_json::json!({
                        "type": "object",
                        "properties": {
                            "message": { "type": "string" },
                            "turn_over": { "type": "boolean", "description": "Whether the turn is completely over and the user should respond. Set to false when you plan to use tools to complete the user's request. Set to true only when you have finished all work and are ready for the user to respond." },
                        },
                        "required": ["message", "turn_over"],
                        "additional_properties": false,
                    }),
                },
            }),
        };

        let response = self
            .client
            .post("https://api.openai.com/v1/chat/completions")
            .bearer_auth(&self.api_key)
            .json(&request_body)
            .send()
            .await?;

        match response.status() {
            reqwest::StatusCode::OK => {
                let completions: ChatCompletionResponse = response
                    .json()
                    .await
                    .map_err(|_| ChatClientError::OpenAIError(OpenAIError::NoCompletionFound))?;
                let choice = completions
                    .choices
                    .get(0)
                    .ok_or(ChatClientError::OpenAIError(OpenAIError::NoChoicesFound))?;

                let message = &choice.message;

                if let Some(content) = &message.content {
                    let content_response = ContentResponse::try_from(content.as_str())
                        .map_err(|_| ChatClientError::OpenAIError(OpenAIError::InvalidContentResponse))?;
                    return Ok(ChatResult::Message {
                        content: content_response.message,
                        turn_over: content_response.turn_over,
                    });
                }

                if let Some(tool_calls_response) = &message.tool_calls {
                    let tool_calls: Result<Vec<ToolCall>, ChatClientError> = tool_calls_response
                        .into_iter()
                        .map(|tool_call| {
                            let arguments: HashMap<String, String> =
                                serde_json::from_str(&tool_call.function.arguments)
                                    .map_err(|_| ChatClientError::OpenAIError(OpenAIError::InvalidToolCallArguments))?;
                            Ok(ToolCall {
                                id: tool_call.id.clone(),
                                name: tool_call.function.name.clone(),
                                arguments,
                            })
                        })
                        .collect();
                    return Ok(ChatResult::ToolCalls(tool_calls?));
                }

                Err(ChatClientError::OpenAIError(OpenAIError::NoContentFound))
            }
            reqwest::StatusCode::UNAUTHORIZED => Err(ChatClientError::InvalidApiKey),
            reqwest::StatusCode::FORBIDDEN => Err(ChatClientError::InsufficientCredits),
            reqwest::StatusCode::TOO_MANY_REQUESTS => Err(ChatClientError::RateLimitExceeded),
            reqwest::StatusCode::INTERNAL_SERVER_ERROR => Err(ChatClientError::ServiceUnavailable),
            reqwest::StatusCode::NOT_FOUND => Err(ChatClientError::InvalidModel),
            _ => {
                let status = response.status();
                println!("Unexpected HTTP status: {:?}", status);
                println!("Response: {:?}", response.text().await.unwrap());
                Err(ChatClientError::Other(format!(
                    "Unexpected HTTP status: {}",
                    status
                )))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_creates_a_client_with_the_provided_api_key() {
        let client = OpenAIClient::new("test-api-key".to_string());
        assert_eq!(client.api_key, "test-api-key");
    }

    #[tokio::test]
    async fn create_chat_completion_returns_error_when_chat_history_is_empty() {
        let client = OpenAIClient::new("test-api-key".to_string());
        let result = client.create_chat_completion(&OpenAiModel::Gpt4oMini, &[], &[]).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn create_chat_completion_returns_error_when_api_key_is_invalid() {
        let client = OpenAIClient::new("invalid-api-key".to_string());
        let result = client.create_chat_completion(&OpenAiModel::Gpt4oMini, &[], &[]).await;
        assert!(result.is_err());
    }
}
