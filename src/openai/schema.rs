use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use crate::openai::model::{ChatMessage, Role, OpenAiModel, Tool};


#[derive(Debug, Serialize, Deserialize)]
pub struct ChatCompletionRequest  {
    pub model: OpenAiModel,
    pub messages: Vec<ChatMessage>,
    pub tools: Vec<Tool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_choice: Option<String>,
}

#[derive(Deserialize, Debug)]
pub struct ChatCompletionResponse {
    pub choices: Vec<ChoiceResponse>,
}

#[derive(Deserialize, Debug)]
pub struct ChoiceResponse {
    pub message: MessageResponse,
}

#[derive(Deserialize, Debug)]
pub struct MessageResponse {
    pub role: Role,
    pub content: String,
}

