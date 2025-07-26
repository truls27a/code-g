use thiserror::Error;

/// Represents errors that can occur when interacting with the OpenAI API.
///
/// This enum encompasses all possible error conditions that may arise during
/// OpenAI API operations, from authentication and validation issues to network
/// problems and API limitations. It uses the `thiserror` crate to provide
/// detailed error messages and automatic error conversion from underlying types.
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

    /// The API rate limit has been exceeded
    #[error("Rate limit exceeded")]
    RateLimitExceeded,

    /// The OpenAI service is temporarily unavailable
    #[error("Service unavailable")]
    ServiceUnavailable,

    /// An HTTP request failed with the underlying reqwest error
    #[error("HTTP request failed: {0}")]
    HttpError(#[from] reqwest::Error),

    /// A catch-all for other errors with a custom message
    #[error("Other error: {0}")]
    Other(String),
}
