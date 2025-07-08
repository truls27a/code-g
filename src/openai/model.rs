use crate::openai::schema::{ChatMessageRequest, Role, ToolCallResponse};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// Chat
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub enum ChatResult {
    Message(String),
    ToolCalls(Vec<ToolCall>),
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub enum ChatMessage {
    System {
        content: String,
    },
    User {
        content: String,
    },
    Assistant {
        content: Option<String>,
        tool_calls: Option<Vec<ToolCall>>,
    },
    Tool {
        content: String,
        tool_call_id: String,
    },
}


impl From<ChatMessageRequest> for ChatMessage {
    fn from(req: ChatMessageRequest) -> Self {
        match req.role {
            Role::System => ChatMessage::System {
                content: req.content.expect("System message must have content"),
            },
            Role::User => ChatMessage::User {
                content: req.content.expect("User message must have content"),
            },
            Role::Assistant => ChatMessage::Assistant {
                content: req.content,
                tool_calls: req.tool_calls.map(|calls| {
                    calls
                        .into_iter()
                        .map(ToolCall::from)
                        .collect()
                }),
            },
            Role::Tool => {
                let content = req.content.expect("Tool message must have content");
                let tool_call_id = req.tool_calls
                    .and_then(|mut calls| calls.pop())
                    .map(|call| call.id)
                    .expect("Tool message must have a tool_call_id");

                ChatMessage::Tool {
                    content,
                    tool_call_id,
                }
            }
        }
    }
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
