use reqwest::Client;
use std::collections::HashMap;
use crate::openai::schema::{ChatCompletionRequest, ChatCompletionResponse, ChatMessageRequest};
use crate::openai::model::{ChatMessage, OpenAiModel, Tool, ChatResult, ToolCall};
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

    pub async fn create_chat_completion(&self, model: &OpenAiModel, chat_history: &[ChatMessage], tools: &[Tool]) -> Result<ChatResult, OpenAIError> {
        if chat_history.is_empty() {
            return Err(OpenAIError::EmptyChatHistory);
        }

        let request_body = ChatCompletionRequest {
            model: model.clone(),
            messages: chat_history.iter().map(|m| ChatMessageRequest::from(m.clone())).collect(),
            tools: Some(tools.to_vec()),
        };

        let response = self.client.post("https://api.openai.com/v1/chat/completions")
            .bearer_auth(&self.api_key)
            .json(&request_body)
            .send()
            .await?;


        match response.status() {
            reqwest::StatusCode::OK => {
                let completions: ChatCompletionResponse = response.json().await.map_err(|_| OpenAIError::NoCompletionFound)?;
                let choice = completions.choices.get(0).ok_or(OpenAIError::NoChoicesFound)?;
                
                let message = &choice.message;

                if let Some(content) = &message.content {
                    return Ok(ChatResult::Message(content.clone()));
                }
                
                if let Some(tool_calls_response) = &message.tool_calls {
                    let tool_calls: Result<Vec<ToolCall>, OpenAIError> = tool_calls_response
                        .into_iter()
                        .map(|tool_call| {
                            let arguments: HashMap<String, String> = serde_json::from_str(&tool_call.function.arguments)
                                .map_err(|_| OpenAIError::InvalidToolCallArguments)?;
                            Ok(ToolCall {
                                id: tool_call.id.clone(),
                                name: tool_call.function.name.clone(),
                                arguments,
                            })
                        })
                        .collect();
                    return Ok(ChatResult::ToolCalls(tool_calls?));
                }

                Err(OpenAIError::NoContentFound)

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
                content: Some("Say 'hi' in Swedish in all lowercase. Do not add any other text.".to_string()),
                tool_calls: None,
            },
        ];
        let response = client.create_chat_completion(&OpenAiModel::Gpt4oMini, chat_history, &[]).await.unwrap();
        assert_eq!(ChatResult::Message("hej".to_string()), response);
    }

    #[tokio::test]
    async fn create_chat_completion_responds_to_multiple_messages() {
        let client = OpenAIClient::new(std::env::var("OPENAI_API_KEY").unwrap());
        let chat_history = &[
            ChatMessage {
                role: Role::User,
                content: Some("How are you dude?".to_string()),
                tool_calls: None,
            },
            ChatMessage {
                role: Role::Assistant,
                content: Some("Yo bro, I feel great!".to_string()),
                tool_calls: None,
            },
            ChatMessage {
                role: Role::User,
                content: Some("What did you say? I didn't hear you. Repeat what you said exactly like you said it. Do not add any other text.".to_string()),
                tool_calls: None,
            },
        ];
        let response = client.create_chat_completion(&OpenAiModel::Gpt4oMini, chat_history, &[]).await.unwrap();
        assert_eq!(ChatResult::Message("Yo bro, I feel great!".to_string()), response);
    }

    #[tokio::test]
    async fn create_chat_completion_adheres_to_system_message() {
        let client = OpenAIClient::new(std::env::var("OPENAI_API_KEY").unwrap());
        let chat_history = &[
            ChatMessage {
                role: Role::System,
                content: Some("Always respond in french with all lowercase. Do not add any other text.".to_string()),
                tool_calls: None,
            },
            ChatMessage {
                role: Role::User,
                content: Some("How do you say 'hello' in french?".to_string()),
                tool_calls: None,
            },
        ];
        let response = client.create_chat_completion(&OpenAiModel::Gpt4oMini, chat_history, &[]).await.unwrap();
        assert_eq!(ChatResult::Message("bonjour".to_string()), response);
    }


    #[tokio::test]
    async fn create_chat_completion_returns_invalid_api_key_error_when_api_key_is_invalid() {
        let client = OpenAIClient::new("invalid_api_key".to_string());
        let chat_history = &[
            ChatMessage {
                role: Role::User,
                content: Some("I am too broke for api key".to_string()),
                tool_calls: None,
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