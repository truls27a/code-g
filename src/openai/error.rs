use thiserror::Error;
use crate::openai::model::OpenAiModel;

#[derive(Error, Debug)]
pub enum OpenAIError {
    #[error("Invalid model: {0}")]
    InvalidModel(OpenAiModel),

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

    #[error("Rate limit exceeded")]
    RateLimitExceeded,

    #[error("Service unavailable")]
    ServiceUnavailable,

    #[error("HTTP request failed: {0}")]
    HttpError(#[from] reqwest::Error),

    #[error("Other error: {0}")]
    Other(String),
}