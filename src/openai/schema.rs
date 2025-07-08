use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use crate::openai::model::{ChatMessage, Role, OpenAiModel, Tool, ToolType};


// Request

#[derive(Debug, Serialize, Deserialize)]
pub struct ChatCompletionRequest  {
    pub model: OpenAiModel,
    pub messages: Vec<ChatMessage>,
    pub tools: Vec<Tool>,
}

// Response
#[derive(Deserialize, Debug)]
pub struct ChatCompletionResponse {
    pub id: String,
    pub object: String,
    pub created: u64,
    pub model: String, // Different format than the model enum
    pub choices: Vec<ChoiceResponse>,

}

#[derive(Deserialize, Debug)]
pub struct ChoiceResponse {
    pub index: u64,
    pub message: MessageResponse,
    pub finish_reason: String,
}

#[derive(Deserialize, Debug)]
pub struct MessageResponse {
    pub role: Role,
    pub content: Option<String>,
    pub tool_calls: Option<Vec<ToolCallResponse>>,
}

#[derive(Deserialize, Debug)]
pub struct ToolCallResponse {
    pub id: String,
    #[serde(rename = "type")]
    pub tool_type: ToolType,
    pub function: FunctionResponse,
}

#[derive(Deserialize, Debug)]
pub struct FunctionResponse {
    pub name: String,
    pub arguments: String,
}
