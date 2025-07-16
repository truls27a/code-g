use crate::openai::model::{AssistantMessage, ChatMessage, OpenAiModel, Tool, ToolCall, ToolType};
use serde::{Deserialize, Serialize};

// Request
#[derive(Debug, Serialize, Deserialize)]
pub struct ChatCompletionRequest {
    pub model: OpenAiModel,
    pub messages: Vec<ChatMessageRequest>,
    pub tools: Option<Vec<Tool>>,
    pub response_format: Option<ResponseFormat>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ChatMessageRequest {
    pub role: Role,
    pub content: Option<String>,
    pub tool_calls: Option<Vec<ToolCallResponse>>,
    pub tool_call_id: Option<String>,
}

impl From<ChatMessage> for ChatMessageRequest {
    fn from(chat_message: ChatMessage) -> Self {
        match chat_message {
            ChatMessage::System { content } => ChatMessageRequest {
                role: Role::System,
                content: Some(content),
                tool_calls: None,
                tool_call_id: None,
            },
            ChatMessage::User { content } => ChatMessageRequest {
                role: Role::User,
                content: Some(content),
                tool_calls: None,
                tool_call_id: None,
            },
            ChatMessage::Assistant { message } => match message {
                AssistantMessage::Content(content) => ChatMessageRequest {
                    role: Role::Assistant,
                    content: Some(content),
                    tool_calls: None,
                    tool_call_id: None,
                },
                AssistantMessage::ToolCalls(tool_calls) => ChatMessageRequest {
                    role: Role::Assistant,
                    content: None,
                    tool_calls: Some(tool_calls.into_iter().map(ToolCallResponse::from).collect()),
                    tool_call_id: None,
                },
            },
            ChatMessage::Tool {
                content,
                tool_call_id,
            } => ChatMessageRequest {
                role: Role::Tool,
                content: Some(content),
                tool_calls: None,
                tool_call_id: Some(tool_call_id),
            },
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
    Tool,
}


// Response
#[derive(Deserialize, Debug, Serialize)]
pub struct ChatCompletionResponse {
    pub id: String,
    pub object: String,
    pub created: u64,
    pub model: String, // Different format than the model enum
    pub choices: Vec<ChoiceResponse>,
}

#[derive(Deserialize, Debug, Serialize)]
pub struct ChoiceResponse {
    pub index: u64,
    pub message: MessageResponse,
    pub finish_reason: String,
}

#[derive(Deserialize, Debug, Serialize)]
pub struct MessageResponse {
    pub role: Role,
    pub content: Option<String>,
    pub tool_calls: Option<Vec<ToolCallResponse>>,
}

#[derive(Deserialize, Debug, Serialize)]
pub struct ContentResponse {
    pub message: String,
    pub turn_over: bool,
}

impl From<&str> for ContentResponse {
    fn from(content: &str) -> Self {
        serde_json::from_str(content).unwrap()
    }
}

#[derive(Deserialize, Debug, Serialize)]
pub struct ToolCallResponse {
    pub id: String,
    #[serde(rename = "type")]
    pub tool_type: ToolType,
    pub function: FunctionResponse,
}

impl From<ToolCall> for ToolCallResponse {
    fn from(tool_call: ToolCall) -> Self {
        Self {
            id: tool_call.id,
            tool_type: ToolType::Function,
            function: FunctionResponse {
                name: tool_call.name,
                arguments: serde_json::to_string(&tool_call.arguments).unwrap(), // TODO: handle error
            },
        }
    }
}

#[derive(Deserialize, Debug, Serialize)]
pub struct FunctionResponse {
    pub name: String,
    pub arguments: String,
}

// Response Format
#[derive(Deserialize, Debug, Serialize)]
pub struct ResponseFormat {
    #[serde(rename = "type")]
    pub response_format_type: String, // "json_schema"
    pub json_schema: JsonSchema,
}

#[derive(Deserialize, Debug, Serialize)]
pub struct JsonSchema {
    pub name: String,
    pub schema: serde_json::Value,
}