use thiserror::Error;

#[derive(Error, Debug)]
pub enum OpenAIError {
    #[error("Invalid model")]
    InvalidModel,

    #[error("Chat history cannot be empty")]
    EmptyChatHistory,

    #[error("Invalid API key")]
    InvalidApiKey,

    #[error("Missing API key")]
    MissingApiKey,

    #[error("Not enough credits")]
    InsufficientCredits,

    #[error("No completion found")]
    NoCompletionFound,

    #[error("No choices found")]
    NoChoicesFound,

    #[error("No content found")]
    NoContentFound,

    #[error("Invalid tool call arguments")]
    InvalidToolCallArguments,

    #[error("Rate limit exceeded")]
    RateLimitExceeded,

    #[error("Service unavailable")]
    ServiceUnavailable,

    #[error("HTTP request failed: {0}")]
    HttpError(#[from] reqwest::Error),

    #[error("Other error: {0}")]
    Other(String),
}