use crate::openai::error::OpenAIError;
use thiserror::Error;

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
