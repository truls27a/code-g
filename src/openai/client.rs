use reqwest::Client;
use crate::openai::schema::{ChatCompletionRequest, ChatCompletionResponse};
use crate::openai::model::ChatMessage;
use crate::openai::error::OpenAIError;

pub struct OpenAIClient {
    client: Client,
    api_key: String,
}

impl OpenAIClient {
    pub fn new(api_key: String) -> Self {
        Self {
            client: Client::new(),
            api_key,
        }
    }

    pub async fn create_chat_completion(&self, model: &str, chat_history: Vec<ChatMessage>) -> Result<String, OpenAIError> {
        let request_body = ChatCompletionRequest {
            model: model.to_string(),
            messages: chat_history,
        };

        let response = self.client.post("https://api.openai.com/v1/chat/completions")
            .bearer_auth(&self.api_key)
            .json(&request_body)
            .send()
            .await?;

        match response.status() {
            reqwest::StatusCode::OK => {
                let completions: ChatCompletionResponse = match response.json().await {
                    Ok(completions) => completions,
                    Err(_) => return Err(OpenAIError::NoCompletionFound),
                };
                let choice_response = match completions.choices.get(0) {
                    Some(choice) => choice,
                    None => return Err(OpenAIError::NoChoicesFound),
                };
                
                let message_content = &choice_response.message.content;
                Ok(message_content.to_string())
            }
            reqwest::StatusCode::UNAUTHORIZED => Err(OpenAIError::InvalidApiKey),
            reqwest::StatusCode::FORBIDDEN => Err(OpenAIError::InsufficientCredits),
            reqwest::StatusCode::TOO_MANY_REQUESTS => Err(OpenAIError::RateLimitExceeded),
            reqwest::StatusCode::INTERNAL_SERVER_ERROR => Err(OpenAIError::ServiceUnavailable),
            _ => Err(OpenAIError::Other(format!("Unexpected HTTP status: {}", response.status()))),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::openai::model::Role;

    #[tokio::test]
    async fn create_chat_completion_responds_to_user_message() {
        let client = OpenAIClient::new(std::env::var("OPENAI_API_KEY").unwrap());
        let chat_history = vec![
            ChatMessage {
                role: Role::User,
                content: "Say 'hi' in Swedish in all lowercase, nothing else.".to_string(),
            },
        ];
        let response = client.create_chat_completion("gpt-4o-mini", chat_history).await.unwrap();
        assert_eq!("hej", response);
    }
}