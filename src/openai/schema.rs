use crate::openai::model::{ChatMessage, OpenAiModel, Role, Tool, ToolType, ToolCall};
use serde::{Deserialize, Serialize};

// Request
#[derive(Debug, Serialize, Deserialize)]
pub struct ChatCompletionRequest {
    pub model: OpenAiModel,
    pub messages: Vec<ChatMessageRequest>,
    pub tools: Option<Vec<Tool>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ChatMessageRequest {
    pub role: Role,
    pub content: Option<String>,
    pub tool_calls: Option<Vec<ToolCallResponse>>,
}

impl From<ChatMessage> for ChatMessageRequest {
    fn from(chat_message: ChatMessage) -> Self {
        Self {
            role: chat_message.role,
            content: chat_message.content,
            tool_calls: chat_message.tool_calls.map(|tool_calls| {
                tool_calls
                    .into_iter()
                    .map(|tool_call| ToolCallResponse::from(tool_call))
                    .collect()
            }),
        }
    }
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
