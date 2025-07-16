use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// Chat
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub enum ChatResult {
    Message { content: String, turn_over: bool },
    ToolCalls(Vec<ToolCall>),
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub enum ChatMessage {
    System {
        content: String,
    },
    User {
        content: String,
    },
    Assistant {
        message: AssistantMessage,
    },
    Tool {
        content: String,
        tool_call_id: String,
    },
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub enum AssistantMessage {
    Content(String),
    ToolCalls(Vec<ToolCall>),
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

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub struct ToolCall {
    pub id: String,
    pub name: String,
    pub arguments: HashMap<String, String>,
}
