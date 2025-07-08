use reqwest::Client;
use crate::openai::schema::{ChatCompletionRequest, ChatMessage, ChatCompletionResponse, Role};

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

    pub async fn create_chat_completion(&self, model: &str, chat_history: Vec<ChatMessage>) -> Result<String, Box<dyn std::error::Error>> {
        let request_body = ChatCompletionRequest {
            model: model.to_string(),
            messages: chat_history,
        };

        let response = self.client.post("https://api.openai.com/v1/chat/completions")
            .bearer_auth(&self.api_key)
            .json(&request_body)
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(Box::new(std::io::Error::new(std::io::ErrorKind::Other, "Failed to create chat completion"))); // TODO: Add custom error type
        }

        let completions: ChatCompletionResponse = response.json().await?;

        println!("Completions: {:?}", completions);

        let response = match completions.choices.get(0) {
            Some(choice) => choice.message.content.clone(),
            None => return Err(Box::new(std::io::Error::new(std::io::ErrorKind::Other, "No completion found"))),
        };

        Ok(response)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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