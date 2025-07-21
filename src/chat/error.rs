use crate::openai::error::OpenAIError;
use thiserror::Error;

#[derive(Debug)]
pub enum ChatSessionErrorHandling {
    /// Fatal error that should immediately stop processing
    Fatal(ChatSessionError),
    /// Retry the request without adding anything to memory
    Retry,
    /// Add an error message to memory and retry
    AddToMemoryAndRetry(String),
}

#[derive(Error, Debug)]
pub enum ChatSessionError {
    #[error("OpenAI API error: {0}")]
    OpenAI(#[from] OpenAIError),

    #[error(
        "Maximum iterations ({max_iterations}) exceeded. The AI may be stuck in a tool-calling loop."
    )]
    MaxIterationsExceeded { max_iterations: usize },

    #[error("Tool execution error: {0}")]
    ToolError(String),
}
