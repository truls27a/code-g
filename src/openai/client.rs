use reqwest::Client;
use crate::openai::schema::{ChatCompletionRequest, ChatCompletionResponse};
use crate::openai::model::{ChatMessage, OpenAiModel, Tool};
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

    pub async fn create_chat_completion(&self, model: &OpenAiModel, chat_history: &[ChatMessage], tools: &[Tool]) -> Result<String, OpenAIError> {
        if chat_history.is_empty() {
            return Err(OpenAIError::EmptyChatHistory);
        }

        let request_body = ChatCompletionRequest {
            model: model.clone(),
            messages: chat_history.to_vec(),
            tools: tools.to_vec(),
        };

        let response = self.client.post("https://api.openai.com/v1/chat/completions")
            .bearer_auth(&self.api_key)
            .json(&request_body)
            .send()
            .await?;


        match response.status() {
            reqwest::StatusCode::OK => {
                let completions: ChatCompletionResponse = response.json().await.map_err(|_| OpenAIError::NoCompletionFound)?;
                let choice_response = completions.choices.get(0).ok_or(OpenAIError::NoChoicesFound)?;
                
                let message_content = choice_response.message.content.as_ref().ok_or(OpenAIError::NoContentFound)?;
                Ok(message_content.to_string())
            }
            reqwest::StatusCode::UNAUTHORIZED => Err(OpenAIError::InvalidApiKey),
            reqwest::StatusCode::FORBIDDEN => Err(OpenAIError::InsufficientCredits),
            reqwest::StatusCode::TOO_MANY_REQUESTS => Err(OpenAIError::RateLimitExceeded),
            reqwest::StatusCode::INTERNAL_SERVER_ERROR => Err(OpenAIError::ServiceUnavailable),
            reqwest::StatusCode::NOT_FOUND => Err(OpenAIError::InvalidModel),
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
        let chat_history = &[
            ChatMessage {
                role: Role::User,
                content: "Say 'hi' in Swedish in all lowercase. Do not add any other text.".to_string(),
            },
        ];
        let response = client.create_chat_completion(&OpenAiModel::Gpt4oMini, chat_history, &[]).await.unwrap();
        assert_eq!("hej", response);
    }

    #[tokio::test]
    async fn create_chat_completion_responds_to_multiple_messages() {
        let client = OpenAIClient::new(std::env::var("OPENAI_API_KEY").unwrap());
        let chat_history = &[
            ChatMessage {
                role: Role::User,
                content: "How are you dude?".to_string(),
            },
            ChatMessage {
                role: Role::Assistant,
                content: "Yo bro, I feel great!".to_string(),
            },
            ChatMessage {
                role: Role::User,
                content: "What did you say? I didn't hear you. Repeat what you said exactly like you said it. Do not add any other text.".to_string(),
            },
        ];
        let response = client.create_chat_completion(&OpenAiModel::Gpt4oMini, chat_history, &[]).await.unwrap();
        assert_eq!("Yo bro, I feel great!", response);
    }

    #[tokio::test]
    async fn create_chat_completion_adheres_to_system_message() {
        let client = OpenAIClient::new(std::env::var("OPENAI_API_KEY").unwrap());
        let chat_history = &[
            ChatMessage {
                role: Role::System,
                content: "Always respond in french with all lowercase. Do not add any other text.".to_string(),
            },
            ChatMessage {
                role: Role::User,
                content: "How do you say 'hello' in french?".to_string(),
            },
        ];
        let response = client.create_chat_completion(&OpenAiModel::Gpt4oMini, chat_history, &[]).await.unwrap();
        assert_eq!("bonjour", response);
    }


    #[tokio::test]
    async fn create_chat_completion_returns_invalid_api_key_error_when_api_key_is_invalid() {
        let client = OpenAIClient::new("invalid_api_key".to_string());
        let chat_history = &[
            ChatMessage {
                role: Role::User,
                content: "I am too broke for api key".to_string(),
            },
        ];
        let response = client.create_chat_completion(&OpenAiModel::Gpt4oMini, chat_history, &[]).await.unwrap_err();
        assert!(matches!(response, OpenAIError::InvalidApiKey));
    }

    #[tokio::test]
    async fn create_chat_completion_returns_empty_chat_history_error_when_chat_history_is_empty() {
        let client = OpenAIClient::new("any_api_key".to_string());
        let chat_history: &[ChatMessage] = &[];
        let response = client.create_chat_completion(&OpenAiModel::Gpt4oMini, chat_history, &[]).await.unwrap_err();
        assert!(matches!(response, OpenAIError::EmptyChatHistory));
    }
}