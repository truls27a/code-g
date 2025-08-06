use crate::client::model::{AssistantMessage, ChatMessage, Tool, ToolCall, ToolType};

use serde::de::Error;
use serde::{Deserialize, Serialize};
use std::convert::TryFrom;

/// Represents the available OpenAI models for chat completions.
///
/// This enum defines the different OpenAI models that can be used for chat
/// completions, each with different capabilities, performance characteristics,
/// and pricing. The enum uses serde renaming to match the exact model names
/// expected by the OpenAI API.
///
/// # Examples
///
/// ```rust
/// use code_g::chat_client::providers::openai::schema::Model;
///
/// let model = Model::Gpt4o;
/// let mini_model = Model::Gpt4oMini;
/// let latest_model = Model::GptO3;
/// ```
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub enum Model {
    /// GPT-4o - Latest high-performance model with vision capabilities
    #[serde(rename = "gpt-4o")]
    Gpt4o,
    /// GPT-4o Mini - Smaller, faster, and more cost-effective variant
    #[serde(rename = "gpt-4o-mini")]
    Gpt4oMini,
    /// GPT-o3 - Next-generation model with enhanced reasoning capabilities
    #[serde(rename = "gpt-o3")]
    GptO3,
    /// GPT-o4 Mini - Compact version of the o4 model family
    #[serde(rename = "gpt-o4-mini")]
    GptO4Mini,
    /// GPT-o4 Mini High - High-performance variant of the o4 mini model
    #[serde(rename = "gpt-o4-mini-high")]
    GptO4MiniHigh,
}

/// Represents a chat completion request to the OpenAI API.
///
/// This struct contains all the necessary information to make a chat completion
/// request, including the model to use, conversation messages, available tools,
/// and response format preferences.
///
/// # Fields
///
/// * `model` - The OpenAI model to use for the completion
/// * `messages` - Vector of messages that make up the conversation history
/// * `tools` - Optional list of tools available for the assistant to call
/// * `response_format` - Optional format specification for structured responses
///
/// # Examples
///
/// ```rust
/// use code_g::chat_client::providers::openai::schema::ChatCompletionRequest;
/// use code_g::chat_client::model::Model;
///
/// let request = ChatCompletionRequest {
///     model: Model::Gpt4o,
///     messages: vec![],
///     tools: None,
///     response_format: None,
/// };
/// ```
#[derive(Debug, Serialize, Deserialize)]
pub struct ChatCompletionRequest {
    pub model: Model,
    pub messages: Vec<ChatMessageRequest>,
    pub tools: Option<Vec<Tool>>,
    pub response_format: Option<ResponseFormat>,
}

/// Represents a single message in a chat completion request.
///
/// This struct is used to serialize chat messages for sending to the OpenAI API.
/// It supports different message types including system, user, assistant, and tool messages,
/// with appropriate fields for each type.
///
/// # Fields
///
/// * `role` - The role of the message sender (system, user, assistant, or tool)
/// * `content` - Optional text content of the message
/// * `tool_calls` - Optional list of tool calls made by the assistant
/// * `tool_call_id` - Optional ID linking tool responses to their calls
///
/// # Examples
///
/// ```rust
/// use code_g::chat_client::providers::openai::schema::{ChatMessageRequest, Role};
///
/// let message = ChatMessageRequest {
///     role: Role::User,
///     content: Some("Hello, AI!".to_string()),
///     tool_calls: None,
///     tool_call_id: None,
/// };
/// ```
#[derive(Debug, Serialize, Deserialize)]
pub struct ChatMessageRequest {
    pub role: Role,
    pub content: Option<String>,
    pub tool_calls: Option<Vec<ToolCallResponse>>,
    pub tool_call_id: Option<String>,
}

impl TryFrom<ChatMessage> for ChatMessageRequest {
    type Error = serde_json::Error;

    /// Converts a [`ChatMessage`] into a [`ChatMessageRequest`] for API serialization.
    ///
    /// This conversion handles the different types of chat messages and maps them
    /// to the appropriate request format expected by the OpenAI API.
    ///
    /// # Arguments
    ///
    /// * `chat_message` - The chat message to convert
    ///
    /// # Returns
    ///
    /// A [`ChatMessageRequest`] ready for API serialization.
    ///
    /// # Errors
    ///
    /// Returns a [`serde_json::Error`] if tool call serialization fails.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use std::convert::TryFrom;
    /// use code_g::chat_client::model::ChatMessage;
    /// use code_g::chat_client::providers::openai::schema::ChatMessageRequest;
    ///
    /// let chat_msg = ChatMessage::User { content: "Hello".to_string() };
    /// let request = ChatMessageRequest::try_from(chat_msg);
    /// ```
    fn try_from(chat_message: ChatMessage) -> Result<Self, Self::Error> {
        match chat_message {
            ChatMessage::System { content } => Ok(ChatMessageRequest {
                role: Role::System,
                content: Some(content),
                tool_calls: None,
                tool_call_id: None,
            }),
            ChatMessage::User { content } => Ok(ChatMessageRequest {
                role: Role::User,
                content: Some(content),
                tool_calls: None,
                tool_call_id: None,
            }),
            ChatMessage::Assistant { message } => match message {
                AssistantMessage::Content(content) => Ok(ChatMessageRequest {
                    role: Role::Assistant,
                    content: Some(content),
                    tool_calls: None,
                    tool_call_id: None,
                }),
                AssistantMessage::ToolCalls(tool_calls) => {
                    let tool_calls = tool_calls
                        .into_iter()
                        .map(ToolCallResponse::try_from)
                        .collect::<Result<Vec<ToolCallResponse>, serde_json::Error>>(
                    )?;
                    Ok(ChatMessageRequest {
                        role: Role::Assistant,
                        content: None,
                        tool_calls: Some(tool_calls),
                        tool_call_id: None,
                    })
                }
            },
            ChatMessage::Tool {
                content,
                tool_call_id,
                tool_name: _,
            } => Ok(ChatMessageRequest {
                role: Role::Tool,
                content: Some(content),
                tool_calls: None,
                tool_call_id: Some(tool_call_id),
            }),
        }
    }
}

impl TryFrom<ChatMessageRequest> for ChatMessage {
    type Error = serde_json::Error;

    /// Converts a [`ChatMessageRequest`] back into a [`ChatMessage`] for internal use.
    ///
    /// This conversion handles deserialization from the API request format back
    /// to the internal chat message representation used by the application.
    ///
    /// # Arguments
    ///
    /// * `chat_message_request` - The request message to convert
    ///
    /// # Returns
    ///
    /// A [`ChatMessage`] for internal application use.
    ///
    /// # Errors
    ///
    /// Returns a [`serde_json::Error`] if required fields are missing or tool call
    /// deserialization fails.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use std::convert::TryFrom;
    /// use code_g::chat_client::providers::openai::schema::{ChatMessageRequest, Role};
    /// use code_g::chat_client::model::ChatMessage;
    ///
    /// let request = ChatMessageRequest {
    ///     role: Role::User,
    ///     content: Some("Hello".to_string()),
    ///     tool_calls: None,
    ///     tool_call_id: None,
    /// };
    /// let chat_msg = ChatMessage::try_from(request);
    /// ```
    fn try_from(chat_message_request: ChatMessageRequest) -> Result<Self, Self::Error> {
        match chat_message_request.role {
            Role::System => {
                let content = chat_message_request
                    .content
                    .ok_or(serde_json::Error::custom(
                        "System message must have content",
                    ))?;
                Ok(ChatMessage::System { content })
            }
            Role::User => Ok(ChatMessage::User {
                content: chat_message_request
                    .content
                    .ok_or(serde_json::Error::custom("User message must have content"))?,
            }),
            Role::Assistant => Ok(ChatMessage::Assistant {
                message: if let Some(content) = chat_message_request.content {
                    AssistantMessage::Content(content)
                } else if let Some(tool_calls) = chat_message_request.tool_calls {
                    AssistantMessage::ToolCalls(
                        tool_calls
                            .into_iter()
                            .map(ToolCall::try_from)
                            .collect::<Result<Vec<ToolCall>, serde_json::Error>>()?,
                    )
                } else {
                    return Err(serde_json::Error::custom(
                        "Assistant message must have content or tool_calls",
                    ));
                },
            }),
            Role::Tool => {
                let content = chat_message_request
                    .content
                    .ok_or(serde_json::Error::custom("Tool message must have content"))?;
                let tool_call = chat_message_request
                    .tool_calls
                    .and_then(|mut calls| calls.pop())
                    .ok_or(serde_json::Error::custom(
                        "Tool message must have a tool_call",
                    ))?;
                let tool_call_id = tool_call.id.clone();
                let tool_name = tool_call.function.name;

                Ok(ChatMessage::Tool {
                    content,
                    tool_call_id,
                    tool_name,
                })
            }
        }
    }
}

// Role
/// Represents the different roles that can send messages in a chat conversation.
///
/// This enum defines the four types of participants in an OpenAI chat completion:
/// system (for instructions), user (for human input), assistant (for AI responses),
/// and tool (for function call results). Each role has specific behaviors and
/// constraints in the conversation flow.
///
/// # Examples
///
/// ```rust
/// use code_g::chat_client::providers::openai::schema::Role;
///
/// let user_role = Role::User;
/// let system_role = Role::System;
/// ```
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum Role {
    /// System role for providing instructions and context to the assistant
    System,
    /// User role for human-generated messages and queries
    User,
    /// Assistant role for AI-generated responses and tool calls
    Assistant,
    /// Tool role for responses from function/tool executions
    Tool,
}

// Response
/// Represents the complete response from an OpenAI chat completion API call.
///
/// This struct contains all the metadata and choices returned by the OpenAI API
/// after processing a chat completion request. It includes timing information,
/// model details, and the actual response choices.
///
/// # Fields
///
/// * `id` - Unique identifier for this completion
/// * `object` - Type of object returned (typically "chat.completion")
/// * `created` - Unix timestamp when the completion was created
/// * `model` - The model that generated this response
/// * `choices` - Array of completion choices (usually contains one choice)
///
/// # Examples
///
/// ```rust
/// use code_g::chat_client::providers::openai::schema::ChatCompletionResponse;
///
/// // Typically received from API deserialization
/// let json_response = r#"{"id":"test","object":"chat.completion","created":123,"model":"gpt-4o","choices":[]}"#;
/// let response: ChatCompletionResponse = serde_json::from_str(&json_response).unwrap();
/// ```
#[derive(Deserialize, Debug, Serialize)]
pub struct ChatCompletionResponse {
    pub id: String,
    pub object: String,
    pub created: u64,
    pub model: String, // Different format than the model enum
    pub choices: Vec<ChoiceResponse>,
}

/// Represents a single completion choice from the OpenAI API response.
///
/// Each choice contains the actual response message and metadata about why
/// the completion finished. Most API calls return a single choice, but the
/// API supports multiple choices for comparison.
///
/// # Fields
///
/// * `index` - Zero-based index of this choice in the choices array
/// * `message` - The actual response message from the assistant
/// * `finish_reason` - Why the completion stopped (e.g., "stop", "length", "tool_calls")
///
/// # Examples
///
/// ```rust
/// use code_g::chat_client::providers::openai::schema::{ChoiceResponse, MessageResponse, Role};
///
/// let choice = ChoiceResponse {
///     index: 0,
///     message: MessageResponse {
///         role: Role::Assistant,
///         content: Some("Hello!".to_string()),
///         tool_calls: None,
///     },
///     finish_reason: "stop".to_string(),
/// };
/// println!("Finish reason: {}", choice.finish_reason);
/// ```
#[derive(Deserialize, Debug, Serialize)]
pub struct ChoiceResponse {
    pub index: u64,
    pub message: MessageResponse,
    pub finish_reason: String,
}

/// Represents a message within a completion choice response.
///
/// This struct contains the actual content of the assistant's response,
/// including any text content and tool calls that the assistant wants to make.
/// It's similar to [`ChatMessageRequest`] but used for API responses.
///
/// # Fields
///
/// * `role` - The role of the message sender (typically Assistant)
/// * `content` - Optional text content of the response
/// * `tool_calls` - Optional list of tool calls the assistant wants to make
///
/// # Examples
///
/// ```rust
/// use code_g::chat_client::providers::openai::schema::{MessageResponse, Role};
///
/// let message = MessageResponse {
///     role: Role::Assistant,
///     content: Some("Hello, how can I help?".to_string()),
///     tool_calls: None,
/// };
/// if let Some(content) = &message.content {
///     println!("Assistant said: {}", content);
/// }
/// ```
#[derive(Deserialize, Debug, Serialize)]
pub struct MessageResponse {
    pub role: Role,
    pub content: Option<String>,
    pub tool_calls: Option<Vec<ToolCallResponse>>,
}

/// Represents a structured content response with turn management.
///
/// This struct is used for parsing structured responses that include both
/// message content and information about whether the conversation turn
/// should continue or end.
///
/// # Fields
///
/// * `message` - The actual message content
/// * `turn_over` - Whether this response ends the current turn
///
/// # Examples
///
/// ```rust
/// use code_g::chat_client::providers::openai::schema::ContentResponse;
/// use std::convert::TryFrom;
///
/// let json_content = r#"{"message": "Hello!", "turn_over": true}"#;
/// let response = ContentResponse::try_from(json_content);
/// ```
#[derive(Deserialize, Debug, Serialize)]
pub struct ContentResponse {
    pub message: String,
    pub turn_over: bool,
}

impl TryFrom<&str> for ContentResponse {
    type Error = serde_json::Error;

    /// Parses a JSON string into a [`ContentResponse`].
    ///
    /// This method deserializes a JSON string representation of a content
    /// response, typically used when the API returns structured JSON content
    /// instead of plain text.
    ///
    /// # Arguments
    ///
    /// * `content` - JSON string to parse
    ///
    /// # Returns
    ///
    /// A [`ContentResponse`] parsed from the JSON string.
    ///
    /// # Errors
    ///
    /// Returns a [`serde_json::Error`] if the JSON is malformed or doesn't
    /// match the expected structure.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use std::convert::TryFrom;
    /// use code_g::chat_client::providers::openai::schema::ContentResponse;
    ///
    /// let json = r#"{"message": "Hello!", "turn_over": false}"#;
    /// let response = ContentResponse::try_from(json).unwrap();
    /// assert_eq!(response.message, "Hello!");
    /// ```
    fn try_from(content: &str) -> Result<Self, Self::Error> {
        serde_json::from_str(content)
    }
}

/// Represents a tool call in an API response.
///
/// This struct contains information about a tool/function that the assistant
/// wants to call, including the tool's ID, type, and the specific function
/// call details. It's used in API responses when the assistant decides to
/// use available tools.
///
/// # Fields
///
/// * `id` - Unique identifier for this tool call
/// * `tool_type` - The type of tool being called (typically "function")
/// * `function` - Details about the specific function being called
///
/// # Examples
///
/// ```rust
/// use code_g::chat_client::providers::openai::schema::{ToolCallResponse, FunctionResponse};
/// use code_g::chat_client::model::ToolType;
///
/// let tool_call = ToolCallResponse {
///     id: "call_123".to_string(),
///     tool_type: ToolType::Function,
///     function: FunctionResponse {
///         name: "get_weather".to_string(),
///         arguments: r#"{"location": "London"}"#.to_string(),
///     },
/// };
/// println!("Calling function: {}", tool_call.function.name);
/// ```
#[derive(Deserialize, Debug, Serialize)]
pub struct ToolCallResponse {
    pub id: String,
    #[serde(rename = "type")]
    pub tool_type: ToolType,
    pub function: FunctionResponse,
}

impl TryFrom<ToolCall> for ToolCallResponse {
    type Error = serde_json::Error;

    /// Converts a [`ToolCall`] into a [`ToolCallResponse`] for API serialization.
    ///
    /// This conversion prepares tool call data for sending to the OpenAI API
    /// by serializing the arguments into a JSON string format expected by the API.
    ///
    /// # Arguments
    ///
    /// * `tool_call` - The tool call to convert
    ///
    /// # Returns
    ///
    /// A [`ToolCallResponse`] ready for API serialization.
    ///
    /// # Errors
    ///
    /// Returns a [`serde_json::Error`] if the tool call arguments cannot be
    /// serialized to JSON.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use std::convert::TryFrom;
    /// use std::collections::HashMap;
    /// use code_g::chat_client::model::ToolCall;
    /// use code_g::chat_client::providers::openai::schema::ToolCallResponse;
    ///
    /// let mut arguments = HashMap::new();
    /// arguments.insert("location".to_string(), "London".to_string());
    /// let tool_call = ToolCall {
    ///     id: "call_123".to_string(),
    ///     name: "get_weather".to_string(),
    ///     arguments,
    /// };
    /// let response = ToolCallResponse::try_from(tool_call);
    /// ```
    fn try_from(tool_call: ToolCall) -> Result<Self, Self::Error> {
        let arguments = serde_json::to_string(&tool_call.arguments)?;
        Ok(Self {
            id: tool_call.id,
            tool_type: ToolType::Function,
            function: FunctionResponse {
                name: tool_call.name,
                arguments,
            },
        })
    }
}

impl TryFrom<ToolCallResponse> for ToolCall {
    type Error = serde_json::Error;

    /// Converts a [`ToolCallResponse`] back into a [`ToolCall`] for internal use.
    ///
    /// This conversion handles deserialization from the API response format back
    /// to the internal tool call representation, parsing the JSON arguments
    /// string back into structured data.
    ///
    /// # Arguments
    ///
    /// * `tool_call_response` - The response tool call to convert
    ///
    /// # Returns
    ///
    /// A [`ToolCall`] for internal application use.
    ///
    /// # Errors
    ///
    /// Returns a [`serde_json::Error`] if the arguments JSON string cannot be
    /// parsed into the expected structure.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use std::convert::TryFrom;
    /// use code_g::chat_client::providers::openai::schema::{ToolCallResponse, FunctionResponse};
    /// use code_g::chat_client::model::{ToolCall, ToolType};
    ///
    /// let response = ToolCallResponse {
    ///     id: "call_123".to_string(),
    ///     tool_type: ToolType::Function,
    ///     function: FunctionResponse {
    ///         name: "get_weather".to_string(),
    ///         arguments: r#"{"location": "London"}"#.to_string(),
    ///     },
    /// };
    /// let tool_call = ToolCall::try_from(response);
    /// ```
    fn try_from(tool_call_response: ToolCallResponse) -> Result<Self, Self::Error> {
        let arguments = serde_json::from_str(&tool_call_response.function.arguments)?;
        Ok(Self {
            id: tool_call_response.id,
            name: tool_call_response.function.name,
            arguments,
        })
    }
}

/// Represents function call details within a tool call response.
///
/// This struct contains the specific information about a function that the
/// assistant wants to call, including the function name and its arguments
/// serialized as a JSON string.
///
/// # Fields
///
/// * `name` - The name of the function to call
/// * `arguments` - JSON string containing the function arguments
///
/// # Examples
///
/// ```rust
/// use code_g::chat_client::providers::openai::schema::FunctionResponse;
///
/// let function = FunctionResponse {
///     name: "calculate_sum".to_string(),
///     arguments: r#"{"a": 5, "b": 3}"#.to_string(),
/// };
/// ```
#[derive(Deserialize, Debug, Serialize)]
pub struct FunctionResponse {
    pub name: String,
    pub arguments: String,
}

// Response Format
/// Specifies the desired response format for structured API responses.
///
/// This struct is used to request structured responses from the OpenAI API,
/// particularly when you want the response to conform to a specific JSON schema
/// rather than returning free-form text.
///
/// # Fields
///
/// * `response_format_type` - The type of response format (typically "json_schema")
/// * `json_schema` - The specific JSON schema definition
///
/// # Examples
///
/// ```rust
/// use code_g::chat_client::providers::openai::schema::{ResponseFormat, JsonSchema};
///
/// let format = ResponseFormat {
///     response_format_type: "json_schema".to_string(),
///     json_schema: JsonSchema {
///         name: "weather_response".to_string(),
///         schema: serde_json::json!({
///             "type": "object",
///             "properties": {
///                 "temperature": {"type": "number"},
///                 "condition": {"type": "string"}
///             }
///         }),
///     },
/// };
/// ```
#[derive(Deserialize, Debug, Serialize)]
pub struct ResponseFormat {
    #[serde(rename = "type")]
    pub response_format_type: String, // "json_schema"
    pub json_schema: JsonSchema,
}

/// Defines a JSON schema for structured API responses.
///
/// This struct contains the schema definition that the OpenAI API should use
/// to structure its response. It includes a name for the schema and the actual
/// schema definition as a JSON value.
///
/// # Fields
///
/// * `name` - A descriptive name for this schema
/// * `schema` - The JSON schema definition as a serde_json::Value
///
/// # Examples
///
/// ```rust
/// use code_g::chat_client::providers::openai::schema::JsonSchema;
/// use serde_json::json;
///
/// let schema = JsonSchema {
///     name: "user_profile".to_string(),
///     schema: json!({
///         "type": "object",
///         "properties": {
///             "name": {"type": "string"},
///             "age": {"type": "integer", "minimum": 0}
///         },
///         "required": ["name"]
///     }),
/// };
/// ```
#[derive(Deserialize, Debug, Serialize)]
pub struct JsonSchema {
    pub name: String,
    pub schema: serde_json::Value,
}
