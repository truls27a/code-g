use crate::openai::error::OpenAIError;
use crate::openai::model::{ChatMessage, ChatResult, OpenAiModel, Tool};
use async_trait::async_trait;

/// Trait defining the interface for chat completion clients.
///
/// This trait abstracts the core functionality needed for chat completions,
/// allowing different implementations (real OpenAI client, mock client, etc.)
/// to be used interchangeably with the chat session.
///
/// # Examples
///
/// ```rust
/// use code_g::openai::traits::ChatClient;
/// use code_g::openai::model::{ChatMessage, ChatResult, OpenAiModel};
/// use async_trait::async_trait;
/// 
/// struct MockClient;
/// 
/// #[async_trait]
/// impl ChatClient for MockClient {
///     async fn create_chat_completion(
///         &self,
///         model: &OpenAiModel,
///         chat_history: &[ChatMessage],
///         tools: &[Tool],
///     ) -> Result<ChatResult, OpenAIError> {
///         Ok(ChatResult::Message {
///             content: "Mock response".to_string(),
///             turn_over: true,
///         })
///     }
/// }
/// ```
#[async_trait]
pub trait ChatClient: Send + Sync {
    /// Creates a chat completion request.
    ///
    /// This method sends a chat completion request with the specified model,
    /// conversation history, and available tools. The response can be either
    /// a text message or tool calls that need to be executed.
    ///
    /// # Arguments
    ///
    /// * `model` - The model to use for the completion
    /// * `chat_history` - The conversation history as a slice of chat messages
    /// * `tools` - Available tools/functions that the assistant can call
    ///
    /// # Returns
    ///
    /// A [`ChatResult`] containing either a message response or tool calls.
    ///
    /// # Errors
    ///
    /// Returns an [`OpenAIError`] for various failure conditions including
    /// network errors, API errors, parsing errors, etc.
    async fn create_chat_completion(
        &self,
        model: &OpenAiModel,
        chat_history: &[ChatMessage],
        tools: &[Tool],
    ) -> Result<ChatResult, OpenAIError>;
}