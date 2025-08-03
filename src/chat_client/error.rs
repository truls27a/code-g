use crate::chat_client::providers::openai::error::OpenAIError;
use thiserror::Error;

/// Represents errors that can occur when interacting with chat client providers.
///
/// This enum encompasses all possible error conditions that may arise during
/// chat client operations across different providers (OpenAI, Claude, etc.),
/// from authentication and validation issues to network problems and API limitations.
/// It uses the `thiserror` crate to provide detailed error messages and automatic
/// error conversion from underlying types.
///
/// # Examples
///
/// ```rust
/// use code_g::chat_client::error::ChatClientError;
///
/// // Handle different types of errors
/// fn handle_chat_error(error: ChatClientError) {
///     match error {
///         ChatClientError::InvalidApiKey => {
///             eprintln!("Please check your API key");
///         }
///         ChatClientError::RateLimitExceeded => {
///             eprintln!("Rate limit hit, please wait before retrying");
///         }
///         ChatClientError::HttpError(e) => {
///             eprintln!("Network error: {}", e);
///         }
///         _ => {
///             eprintln!("Other chat client error: {}", error);
///         }
///     }
/// }
/// ```
#[derive(Error, Debug)]
pub enum ChatClientError {
    /// The specified model is not valid or supported
    #[error("Invalid model")]
    InvalidModel,

    /// The chat history provided is empty when it should contain messages
    #[error("Chat history cannot be empty")]
    EmptyChatHistory,

    /// The chat message request format is invalid or malformed
    #[error("Invalid chat message request")]
    InvalidChatMessageRequest,

    /// The provided API key is invalid or malformed
    #[error("Invalid API key")]
    InvalidApiKey,

    /// No API key was provided when one is required
    #[error("Missing API key")]
    MissingApiKey,

    /// The account has insufficient credits to complete the request
    #[error("Not enough credits")]
    InsufficientCredits,

    /// The API rate limit has been exceeded
    #[error("Rate limit exceeded")]
    RateLimitExceeded,

    /// The chat service is temporarily unavailable
    #[error("Service unavailable")]
    ServiceUnavailable,

    /// An HTTP request failed with the underlying reqwest error
    #[error("HTTP request failed: {0}")]
    HttpError(#[from] reqwest::Error),

    /// An error occurred while interacting with the OpenAI API
    #[error("OpenAI API error: {0}")]
    OpenAIError(#[from] OpenAIError),

    /// A catch-all for other errors with a custom message
    #[error("Other error: {0}")]
    Other(String),
}
