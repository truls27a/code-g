use std::collections::HashMap;
use serde::{Deserialize, Serialize};

// Chat
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub enum ChatResult {
    Message(String),
    ToolCalls(Vec<ToolCall>),
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct ChatMessage {
    pub role: Role,
    pub content: String,
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
