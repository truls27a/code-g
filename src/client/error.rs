use crate::client::providers::openai::error::OpenAIError;
use thiserror::Error;

/// Defines the retry strategy for different types of errors.
///
/// This enum categorizes errors based on how they should be handled
/// in terms of retry logic, providing a clean abstraction for the
/// chat session to determine appropriate recovery strategies.
///
/// # Examples
///
/// ```rust
/// use code_g::chat_client::error::ErrorRetryStrategy;
///
/// let strategy = ErrorRetryStrategy::Retryable;
/// match strategy {
///     ErrorRetryStrategy::Fatal => println!("Cannot recover"),
///     ErrorRetryStrategy::Retryable => println!("Can retry"),
///     ErrorRetryStrategy::AddToMemoryAndRetry => println!("Inform AI and retry"),
/// }
/// ```
#[derive(Debug, Clone, PartialEq)]
pub enum ErrorRetryStrategy {
    /// Fatal errors that cannot be recovered from (e.g., invalid API key, insufficient credits)
    Fatal,
    /// Temporary errors that can be retried (e.g., rate limits, network issues)
    Retryable,
    /// Errors where the AI should be informed of the issue before retrying (e.g., invalid response format)
    AddToMemoryAndRetry,
}

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

impl Clone for ChatClientError {
    fn clone(&self) -> Self {
        match self {
            ChatClientError::InvalidModel => ChatClientError::InvalidModel,
            ChatClientError::EmptyChatHistory => ChatClientError::EmptyChatHistory,
            ChatClientError::InvalidChatMessageRequest => {
                ChatClientError::InvalidChatMessageRequest
            }
            ChatClientError::InvalidApiKey => ChatClientError::InvalidApiKey,
            ChatClientError::MissingApiKey => ChatClientError::MissingApiKey,
            ChatClientError::InsufficientCredits => ChatClientError::InsufficientCredits,
            ChatClientError::RateLimitExceeded => ChatClientError::RateLimitExceeded,
            ChatClientError::ServiceUnavailable => ChatClientError::ServiceUnavailable,
            ChatClientError::HttpError(e) => ChatClientError::Other(e.to_string()),
            ChatClientError::OpenAIError(e) => ChatClientError::Other(e.to_string()),
            ChatClientError::Other(e) => ChatClientError::Other(e.clone()),
        }
    }
}

impl ChatClientError {
    /// Determines the retry strategy for this error.
    ///
    /// This method categorizes the error based on its type and returns
    /// the appropriate retry strategy that should be used by the chat session.
    /// The categorization is based on whether the error is:
    /// - Fatal: Configuration or account issues that won't resolve by retrying
    /// - Retryable: Temporary network or service issues
    /// - AddToMemoryAndRetry: Response format issues where the AI should be informed
    ///
    /// # Returns
    ///
    /// An [`ErrorRetryStrategy`] indicating how this error should be handled.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use code_g::chat_client::error::{ChatClientError, ErrorRetryStrategy};
    ///
    /// let error = ChatClientError::InvalidApiKey;
    /// assert_eq!(error.retry_strategy(), ErrorRetryStrategy::Fatal);
    ///
    /// let error = ChatClientError::RateLimitExceeded;
    /// assert_eq!(error.retry_strategy(), ErrorRetryStrategy::Retryable);
    /// ```
    pub fn retry_strategy(&self) -> ErrorRetryStrategy {
        match self {
            // Fatal errors - configuration or account issues that won't resolve by retrying
            ChatClientError::InvalidApiKey
            | ChatClientError::MissingApiKey
            | ChatClientError::InsufficientCredits
            | ChatClientError::InvalidModel
            | ChatClientError::EmptyChatHistory => ErrorRetryStrategy::Fatal,

            // Network/service errors - might be temporary, can retry
            ChatClientError::RateLimitExceeded
            | ChatClientError::ServiceUnavailable
            | ChatClientError::HttpError(_) => ErrorRetryStrategy::Retryable,

            // Request errors - likely a programming bug, but inform AI in case it can adapt
            ChatClientError::InvalidChatMessageRequest => ErrorRetryStrategy::AddToMemoryAndRetry,

            // Provider-specific errors - delegate to the provider's error handling
            ChatClientError::OpenAIError(openai_error) => openai_error.retry_strategy(),

            // Other errors - treat as potentially recoverable by informing the AI
            ChatClientError::Other(_) => ErrorRetryStrategy::AddToMemoryAndRetry,
        }
    }
}
