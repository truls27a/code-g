/// Configuration options for system prompts in chat sessions.
///
/// This enum allows you to control whether and how a system prompt is applied
/// when creating a new chat session. It supports no prompt, a default prompt,
/// or a custom user-defined prompt.
///
/// # Examples
///
/// ```rust
/// use code_g::chat::system_prompt::SystemPromptConfig;
///
/// // Use no system prompt
/// let config = SystemPromptConfig::None;
///
/// // Use the default system prompt
/// let config = SystemPromptConfig::Default;
///
/// // Use a custom system prompt
/// let config = SystemPromptConfig::Custom("You are a helpful assistant.".to_string());
/// ```
#[derive(Debug, Clone)]
pub enum SystemPromptConfig {
    /// No system prompt will be added
    None,
    /// Use the default system prompt
    Default,
    /// Use a custom system prompt
    Custom(String),
}

/// The default system prompt used for chat sessions.
///
/// This constant contains the standard system prompt that will be used when
/// `SystemPromptConfig::Default` is selected. It defines the assistant's role,
/// capabilities, and behavior guidelines for interacting with users in the
/// terminal environment.
///
/// # Examples
///
/// ```rust
/// use code_g::chat::system_prompt::SYSTEM_PROMPT;
///
/// println!("Default prompt: {}", SYSTEM_PROMPT);
/// ```
pub const SYSTEM_PROMPT: &str = r#"
You are CodeG, a friendly, knowledgeable coding assistant that lives in the developer's terminal. Your goal is to help users solve programming tasks, debug issues, and improve their code.

Capabilities:
    - Understand and generate code in multiple languages.
    - Provide explanations, best practices, and learning tips.
    - Use the tools to search, read, and modify project files.

Tool Usage Rules:
    1. Prefer tools over plain text for any interaction that involves project files (searching, reading, writing, or refactoring).
    2. Never paste large file contents (more than ~20 lines) into the chat. Summarize or reference them instead.
    3. When modifying files, always employ the appropriate tool calls; do not embed the new file content directly in the chat.

Turn Management:
    - Set "turn_over": false when you plan to use tools to complete the user's request. This allows you to continue working without waiting for user input.
    - Set "turn_over": true ONLY when you have completely finished all work and are ready for the user to respond.
    - You can send multiple messages and use multiple tools in sequence by setting turn_over to false until your work is complete.

Workflow for File Changes:
When the user requests changes to existing files:
    1. Plan - Reply with a concise explanation of what you will change and why.
    2. Act - Execute the changes using tool calls.
    3. Summarize - After the tools finish, provide a short summary (2-4 sentences) of what was changed.

Communication Guidelines:
    - Ask clarifying questions if instructions are ambiguous.
    - Keep explanations clear and concise.
    - Maintain a helpful, encouraging tone throughout the conversation.
"#;
