use crate::openai::error::OpenAIError;
use thiserror::Error;

/// Represents different strategies for handling chat session errors.
///
/// This enum defines how the chat session should respond to errors that occur
/// during processing. It allows for different recovery strategies depending on
/// the type and severity of the error encountered.
///
/// # Examples
///
/// ```rust
/// use code_g::chat::error::ChatSessionErrorHandling;
///
/// let handling = ChatSessionErrorHandling::Retry;
/// ```
#[derive(Debug)]
pub enum ChatSessionErrorHandling {
    /// Fatal error that should immediately stop processing
    Fatal(ChatSessionError),
    /// Retry the request without adding anything to memory
    Retry,
    /// Add an error message to memory and retry
    AddToMemoryAndRetry(String),
}

/// Represents errors that can occur during chat session operations.
///
/// This enum encompasses all possible error conditions that may arise when
/// executing chat sessions, including OpenAI API failures, iteration limits,
/// and tool execution problems. Each variant provides specific context about
/// the nature of the error to enable appropriate error handling.
///
/// # Examples
///
/// ```rust
/// use code_g::chat::error::ChatSessionError;
///
/// let error = ChatSessionError::ToolError("Failed to execute command".to_string());
/// ```
#[derive(Error, Debug)]
pub enum ChatSessionError {
    /// Error originating from the OpenAI API
    #[error("OpenAI API error: {0}")]
    OpenAI(#[from] OpenAIError),

    /// Maximum iterations exceeded.
    #[error(
        "Maximum iterations ({max_iterations}) exceeded. The AI may be stuck in a tool-calling loop."
    )]
    MaxIterationsExceeded { max_iterations: usize },

    /// Error during tool execution
    #[error("Tool execution error: {0}")]
    ToolError(String),
}
