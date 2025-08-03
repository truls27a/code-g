use crate::chat_client::error::ErrorRetryStrategy;
use thiserror::Error;

/// Represents errors that can occur when interacting with the OpenAI API.
///
/// This enum encompasses the specific errors that can occur when interacting with the OpenAI API.
/// Generally errors that are not specifly related to the OpenAI API should be handled by the [`ChatClientError`](crate::chat_client::error::ChatClientError) enum.
///
/// # Examples
///
/// ```rust
/// use code_g::openai::error::OpenAIError;
///
/// // Handle different types of errors
/// fn handle_openai_error(error: OpenAIError) {
///     match error {
///         OpenAIError::InvalidApiKey => {
///             eprintln!("Please check your API key");
///         }
///         OpenAIError::RateLimitExceeded => {
///             eprintln!("Rate limit hit, please wait before retrying");
///         }
///         OpenAIError::HttpError(e) => {
///             eprintln!("Network error: {}", e);
///         }
///         _ => {
///             eprintln!("Other OpenAI error: {}", error);
///         }
///     }
/// }
/// ```
#[derive(Error, Debug)]
pub enum OpenAIError {
    /// The API response did not contain a completion
    #[error("No completion found")]
    NoCompletionFound,

    /// The API response did not contain any choices
    #[error("No choices found")]
    NoChoicesFound,

    /// The API response did not contain any content
    #[error("No content found")]
    NoContentFound,

    /// The tool call arguments are invalid or cannot be parsed
    #[error("Invalid tool call arguments")]
    InvalidToolCallArguments,

    /// The content response format is invalid or cannot be parsed
    #[error("Invalid content response")]
    InvalidContentResponse,

    /// Other errors
    #[error("Other error: {0}")]
    Other(String),
}

impl OpenAIError {
    /// Determines the retry strategy for this OpenAI-specific error.
    ///
    /// This method categorizes OpenAI API errors based on their type and returns
    /// the appropriate retry strategy. The categorization follows OpenAI's
    /// error handling best practices and recommendations.
    ///
    /// # Returns
    ///
    /// An [`ErrorRetryStrategy`] indicating how this error should be handled.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use code_g::chat_client::providers::openai::error::OpenAIError;
    /// use code_g::chat_client::error::ErrorRetryStrategy;
    ///
    /// let error = OpenAIError::InvalidApiKey;
    /// assert_eq!(error.retry_strategy(), ErrorRetryStrategy::Fatal);
    ///
    /// let error = OpenAIError::RateLimitExceeded;
    /// assert_eq!(error.retry_strategy(), ErrorRetryStrategy::Retryable);
    /// ```
    pub fn retry_strategy(&self) -> ErrorRetryStrategy {
        match self {
            // Content/parsing errors - AI might have made a mistake, inform it and retry
            OpenAIError::InvalidContentResponse
            | OpenAIError::InvalidToolCallArguments
            | OpenAIError::NoCompletionFound
            | OpenAIError::NoChoicesFound
            | OpenAIError::NoContentFound => ErrorRetryStrategy::AddToMemoryAndRetry,

            // Other errors - treat as potentially recoverable
            OpenAIError::Other(_) => ErrorRetryStrategy::AddToMemoryAndRetry,
        }
    }
}
