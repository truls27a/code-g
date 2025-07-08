use reqwest::Client;
use std::collections::HashMap;
use crate::openai::schema::{ChatCompletionRequest, ChatCompletionResponse};
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
                // println!("Response: {:?}", response.text().await?); ->:
                /*
                {
                    "id": "chatcmpl-Br617zJWJrO4xzC414ma0d6toHLkf",
                    "object": "chat.completion",
                    "created": 1751994213,
                    "model": "gpt-4o-mini-2024-07-18",
                    "choices": [
                        {
                        "index": 0,
                        "message": {
                            "role": "assistant",
                            "content": null,
                            "tool_calls": [
                            {
                                "id": "call_FfysjRfMtcW08stTt7Nd52XS",
                                "type": "function",
                                "function": {
                                "name": "read_file",
                                "arguments": "{\"path\":\"poem.txt\"}"
                                }
                            }
                            ],
                            "refusal": null,
                            "annotations": []
                        },
                        "logprobs": null,
                        "finish_reason": "tool_calls"
                        }
                    ],
                    "usage": {
                        "prompt_tokens": 61,
                        "completion_tokens": 16,
                        "total_tokens": 77,
                        "prompt_tokens_details": {
                        "cached_tokens": 0,
                        "audio_tokens": 0
                        },
                        "completion_tokens_details": {
                        "reasoning_tokens": 0,
                        "audio_tokens": 0,
                        "accepted_prediction_tokens": 0,
                        "rejected_prediction_tokens": 0
                        }
                    },
                    "service_tier": "default",
                    "system_fingerprint": "fp_34a54ae93c"
                    }

                 */
                let completions: ChatCompletionResponse = response.json().await.map_err(|_| OpenAIError::NoCompletionFound)?; // -> Err(OpenAIError::NoCompletionFound)
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
                content: "Say 'hi' in Swedish in all lowercase. Do not add any other text.".to_string(),
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
        assert_eq!(ChatResult::Message("Yo bro, I feel great!".to_string()), response);
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
        assert_eq!(ChatResult::Message("bonjour".to_string()), response);
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