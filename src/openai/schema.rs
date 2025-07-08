use serde::{Deserialize, Serialize};
use crate::openai::model::{ChatMessage, Role};


#[derive(Debug, Serialize, Deserialize)]
pub struct ChatCompletionRequest  {
    pub model: String,
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