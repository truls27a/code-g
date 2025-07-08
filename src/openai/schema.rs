use serde::{Deserialize, Serialize};
use crate::openai::model::{ChatMessage, Role, OpenAiModel};


#[derive(Debug, Serialize, Deserialize)]
pub struct ChatCompletionRequest  {
    pub model: OpenAiModel,
    pub messages: Vec<ChatMessage>,
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