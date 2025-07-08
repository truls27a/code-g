use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct ChatMessage {
    pub role: Role,
    pub content: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ChatCompletionRequest  {
    pub model: String,
    pub messages: Vec<ChatMessage>,
}

#[derive(Deserialize, Debug)]
pub struct ChatCompletionResponse {
    pub choices: Vec<Choice>,
}

#[derive(Deserialize, Debug)]
pub struct Choice {
    pub message: MessageResponse,
}

#[derive(Deserialize, Debug)]
pub struct MessageResponse {
    pub role: Role,
    pub content: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Role {
    System,
    User,
    Assistant,
}