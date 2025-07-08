use crate::openai::schema::{ChatMessageRequest, ToolCallResponse};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// Chat
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub enum ChatResult {
    Message(String),
    ToolCalls(Vec<ToolCall>),
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct ChatMessage {
    pub role: Role,
    pub content: Option<String>,
    pub tool_calls: Option<Vec<ToolCall>>,
}

impl From<ChatMessageRequest> for ChatMessage {
    fn from(chat_message_request: ChatMessageRequest) -> Self {
        Self {
            role: chat_message_request.role,
            content: chat_message_request.content,
            tool_calls: chat_message_request.tool_calls.map(|tool_calls| {
                tool_calls
                    .into_iter()
                    .map(|tool_call| ToolCall::from(tool_call))
                    .collect()
            }),
        }
    }
}

// Role
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum Role {
    System,
    User,
    Assistant,
}

// Model
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub enum OpenAiModel {
    #[serde(rename = "gpt-4o")]
    Gpt4o,
    #[serde(rename = "gpt-4o-mini")]
    Gpt4oMini,
    #[serde(rename = "gpt-o3")]
    GptO3,
    #[serde(rename = "gpt-o4-mini")]
    GptO4Mini,
    #[serde(rename = "gpt-o4-mini-high")]
    GptO4MiniHigh,
}

// Tool
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Tool {
    #[serde(rename = "type")]
    pub tool_type: ToolType,
    pub function: Function,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "snake_case")]
pub enum ToolType {
    Function,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Function {
    pub name: String,
    pub description: String,
    pub parameters: Parameters,
    pub strict: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Parameters {
    #[serde(rename = "type")]
    pub param_type: String, // usually "object"
    pub properties: HashMap<String, Property>,
    pub required: Vec<String>,
    pub additional_properties: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Property {
    #[serde(rename = "type")]
    pub prop_type: String,
    pub description: String,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct ToolCall {
    pub id: String,
    pub name: String,
    pub arguments: HashMap<String, String>,
}

impl From<ToolCallResponse> for ToolCall {
    fn from(tool_call_response: ToolCallResponse) -> Self {
        Self {
            id: tool_call_response.id,
            name: tool_call_response.function.name,
            arguments: serde_json::from_str(&tool_call_response.function.arguments).unwrap(), // TODO: handle error
        }
    }
}
