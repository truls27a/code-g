use crate::openai::error::OpenAIError;
use crate::openai::model::{ChatMessage, ChatResult, OpenAiModel, Tool, ToolCall};
use crate::openai::schema::{
    ChatCompletionRequest, ChatCompletionResponse, ChatMessageRequest, ContentResponse, JsonSchema,
    ResponseFormat,
};
use reqwest::Client;
use std::collections::HashMap;

/// HTTP client for interacting with the OpenAI Chat Completions API.
///
/// This struct provides a high-level interface for making chat completion requests
/// to the OpenAI API. It handles authentication, request formatting, response parsing,
/// and error handling, abstracting away the low-level HTTP details and providing
/// a convenient Rust API for OpenAI interactions.
///
/// # Fields
///
/// * `client` - The underlying HTTP client for making requests
/// * `api_key` - The OpenAI API key for authentication
///
/// # Examples
///
/// ```rust
/// use code_g::openai::client::OpenAIClient;
/// use code_g::openai::model::{ChatMessage, OpenAiModel};
/// use tokio::runtime::Runtime;
///
/// let client = OpenAIClient::new("your-api-key".to_string());
///
/// let chat_history = vec![
///     ChatMessage::User {
///         content: "Hello, how are you?".to_string(),
///     }
/// ];
///
/// let rt = Runtime::new().unwrap();
/// let result = rt.block_on(client
///     .create_chat_completion(&OpenAiModel::Gpt4oMini, &chat_history, &[]));
/// ```
pub struct OpenAIClient {
    client: Client,
    api_key: String,
}

impl OpenAIClient {
    /// Creates a new OpenAI client with the provided API key.
    ///
    /// This constructor initializes a new HTTP client and stores the API key
    /// for use in subsequent requests. The API key should be a valid OpenAI
    /// API key obtained from the OpenAI platform.
    ///
    /// # Arguments
    ///
    /// * `api_key` - A valid OpenAI API key for authentication
    ///
    /// # Returns
    ///
    /// A new [`OpenAIClient`] instance ready to make API requests.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use code_g::openai::client::OpenAIClient;
    ///
    /// let client = OpenAIClient::new("sk-your-api-key-here".to_string());
    /// ```
    pub fn new(api_key: String) -> Self {
        Self {
            client: Client::new(),
            api_key,
        }
    }

    /// Creates a chat completion request to the OpenAI API.
    ///
    /// This method sends a chat completion request with the specified model,
    /// conversation history, and available tools. It handles the complete
    /// request/response cycle, including authentication, JSON schema formatting
    /// for structured responses, and error handling. The response can be either
    /// a text message or tool calls that need to be executed.
    ///
    /// # Arguments
    ///
    /// * `model` - The OpenAI model to use for the completion
    /// * `chat_history` - The conversation history as a slice of chat messages
    /// * `tools` - Available tools/functions that the assistant can call
    ///
    /// # Returns
    ///
    /// A [`ChatResult`] containing either a message response or tool calls.
    ///
    /// # Errors
    ///
    /// Returns an [`OpenAIError`] in the following cases:
    /// - [`OpenAIError::EmptyChatHistory`] if the chat history is empty
    /// - [`OpenAIError::InvalidChatMessageRequest`] if message conversion fails
    /// - [`OpenAIError::InvalidApiKey`] if the API key is invalid (HTTP 401)
    /// - [`OpenAIError::InsufficientCredits`] if the account has no credits (HTTP 403)
    /// - [`OpenAIError::RateLimitExceeded`] if rate limits are hit (HTTP 429)
    /// - [`OpenAIError::InvalidModel`] if the model is not found (HTTP 404)
    /// - [`OpenAIError::ServiceUnavailable`] if OpenAI service is down (HTTP 500)
    /// - [`OpenAIError::HttpError`] for other network-related errors
    /// - [`OpenAIError::NoCompletionFound`] if response parsing fails
    /// - [`OpenAIError::NoChoicesFound`] if no choices in the response
    /// - [`OpenAIError::NoContentFound`] if no content or tool calls found
    /// - [`OpenAIError::InvalidContentResponse`] if content JSON is malformed
    /// - [`OpenAIError::InvalidToolCallArguments`] if tool call args are invalid
    ///
    /// # Examples
    ///
    /// ```rust
    /// use code_g::openai::client::OpenAIClient;
    /// use code_g::openai::model::{ChatMessage, ChatResult, OpenAiModel};
    /// use tokio::runtime::Runtime;
    ///
    /// let client = OpenAIClient::new("your-api-key".to_string());
    ///
    /// let chat_history = vec![
    ///     ChatMessage::System {
    ///         content: "You are a helpful assistant.".to_string(),
    ///     },
    ///     ChatMessage::User {
    ///         content: "What's the capital of France?".to_string(),
    ///     },
    /// ];
    ///
    /// let rt = Runtime::new().unwrap();
    /// match rt.block_on(client
    ///     .create_chat_completion(&OpenAiModel::Gpt4oMini, &chat_history, &[]))
    /// {
    ///     Ok(ChatResult::Message { content, turn_over }) => {
    ///         println!("Assistant: {}", content);
    ///         println!("Turn over: {}", turn_over);
    ///     }
    ///     Ok(ChatResult::ToolCalls(calls)) => {
    ///         println!("Assistant wants to call {} tools", calls.len());
    ///     }
    ///     Err(e) => {
    ///         eprintln!("Error: {:?}", e);
    ///     }
    /// }
    /// ```
    pub async fn create_chat_completion(
        &self,
        model: &OpenAiModel,
        chat_history: &[ChatMessage],
        tools: &[Tool],
    ) -> Result<ChatResult, OpenAIError> {
        if chat_history.is_empty() {
            return Err(OpenAIError::EmptyChatHistory);
        }

        let request_body = ChatCompletionRequest {
            model: model.clone(),
            messages: chat_history
                .iter()
                .map(|m| ChatMessageRequest::try_from(m.clone()))
                .collect::<Result<Vec<ChatMessageRequest>, serde_json::Error>>()
                .map_err(|_| OpenAIError::InvalidChatMessageRequest)?,
            tools: Some(tools.to_vec()),
            response_format: Some(ResponseFormat {
                response_format_type: "json_schema".to_string(),
                json_schema: JsonSchema {
                    name: "structured_chat_response".to_string(),
                    schema: serde_json::json!({
                        "type": "object",
                        "properties": {
                            "message": { "type": "string" },
                            "turn_over": { "type": "boolean", "description": "Whether the turn is completely over and the user should respond. Set to false when you plan to use tools to complete the user's request. Set to true only when you have finished all work and are ready for the user to respond." },
                        },
                        "required": ["message", "turn_over"],
                        "additional_properties": false,
                    }),
                },
            }),
        };

        let response = self
            .client
            .post("https://api.openai.com/v1/chat/completions")
            .bearer_auth(&self.api_key)
            .json(&request_body)
            .send()
            .await?;

        match response.status() {
            reqwest::StatusCode::OK => {
                let completions: ChatCompletionResponse = response
                    .json()
                    .await
                    .map_err(|_| OpenAIError::NoCompletionFound)?;
                let choice = completions
                    .choices
                    .get(0)
                    .ok_or(OpenAIError::NoChoicesFound)?;

                let message = &choice.message;

                if let Some(content) = &message.content {
                    let content_response = ContentResponse::try_from(content.as_str())
                        .map_err(|_| OpenAIError::InvalidContentResponse)?;
                    return Ok(ChatResult::Message {
                        content: content_response.message,
                        turn_over: content_response.turn_over,
                    });
                }

                if let Some(tool_calls_response) = &message.tool_calls {
                    let tool_calls: Result<Vec<ToolCall>, OpenAIError> = tool_calls_response
                        .into_iter()
                        .map(|tool_call| {
                            let arguments: HashMap<String, String> =
                                serde_json::from_str(&tool_call.function.arguments)
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
            _ => {
                let status = response.status();
                println!("Unexpected HTTP status: {:?}", status);
                println!("Response: {:?}", response.text().await.unwrap());
                Err(OpenAIError::Other(format!(
                    "Unexpected HTTP status: {}",
                    status
                )))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::openai::model::{
        AssistantMessage, ChatMessage, ChatResult, Function, OpenAiModel, Parameters, Property,
        Tool, ToolType,
    };

    #[tokio::test]
    async fn create_chat_completion_responds_to_user_message() {
        let client = OpenAIClient::new(std::env::var("OPENAI_API_KEY").unwrap());
        let chat_history = &[ChatMessage::User {
            content: "Say 'hi' in Swedish in all lowercase. Do not add any other text.".to_string(),
        }];
        let response = client
            .create_chat_completion(&OpenAiModel::Gpt4oMini, chat_history, &[])
            .await
            .unwrap();

        match response {
            ChatResult::Message { content, .. } => assert!(content.contains("hej")),
            _ => panic!("Expected ChatResult::Message"),
        }
    }

    #[tokio::test]
    async fn create_chat_completion_responds_to_multiple_messages() {
        let client = OpenAIClient::new(std::env::var("OPENAI_API_KEY").unwrap());
        let chat_history = &[
            ChatMessage::User {
                content: "How are you dude?".to_string(),
            },
            ChatMessage::Assistant {
                message: AssistantMessage::Content("Yo bro, I feel great!".to_string()),
            },
            ChatMessage::User {
                content: "What did you say? I didn't hear you. Repeat what you said exactly like you said it. Do not add any other text.".to_string(),
            },
        ];
        let response = client
            .create_chat_completion(&OpenAiModel::Gpt4oMini, chat_history, &[])
            .await
            .unwrap();

        match response {
            ChatResult::Message { content, .. } => {
                assert!(content.contains("Yo bro, I feel great!"))
            }
            _ => panic!("Expected ChatResult::Message"),
        }
    }

    #[tokio::test]
    async fn create_chat_completion_adheres_to_system_message() {
        let client = OpenAIClient::new(std::env::var("OPENAI_API_KEY").unwrap());
        let chat_history = &[
            ChatMessage::System {
                content: "Always respond in french with all lowercase. Do not add any other text."
                    .to_string(),
            },
            ChatMessage::User {
                content: "How do you say 'hello' in french? Dont say anything else".to_string(),
            },
        ];
        let response = client
            .create_chat_completion(&OpenAiModel::Gpt4oMini, chat_history, &[])
            .await
            .unwrap();

        match response {
            ChatResult::Message { content, .. } => assert!(content.contains("bonjour")),
            _ => panic!("Expected ChatResult::Message"),
        }
    }

    #[tokio::test]
    async fn create_chat_completion_returns_invalid_api_key_error_when_api_key_is_invalid() {
        let client = OpenAIClient::new("invalid_api_key".to_string());
        let chat_history = &[ChatMessage::User {
            content: "I am too broke for api key".to_string(),
        }];
        let response = client
            .create_chat_completion(&OpenAiModel::Gpt4oMini, chat_history, &[])
            .await
            .unwrap_err();
        assert!(matches!(response, OpenAIError::InvalidApiKey));
    }

    #[tokio::test]
    async fn create_chat_completion_returns_empty_chat_history_error_when_chat_history_is_empty() {
        let client = OpenAIClient::new("any_api_key".to_string());
        let chat_history: &[ChatMessage] = &[];
        let response = client
            .create_chat_completion(&OpenAiModel::Gpt4oMini, chat_history, &[])
            .await
            .unwrap_err();
        assert!(matches!(response, OpenAIError::EmptyChatHistory));
    }

    #[tokio::test]
    async fn create_chat_completion_returns_tool_calls_when_tool_calls_are_present() {
        let client = OpenAIClient::new(std::env::var("OPENAI_API_KEY").unwrap());

        let chat_history = &[ChatMessage::User {
            content: "What is the weather in Tokyo?".to_string(),
        }];

        let tools = &[Tool {
            tool_type: ToolType::Function,
            function: Function {
                name: "get_weather".to_string(),
                description: "Get the weather in a given city".to_string(),
                parameters: Parameters {
                    param_type: "object".to_string(),
                    properties: HashMap::from([(
                        "city".to_string(),
                        Property {
                            prop_type: "string".to_string(),
                            description: "The city to get the weather of".to_string(),
                        },
                    )]),
                    required: vec!["city".to_string()],
                    additional_properties: false,
                },
                strict: true,
            },
        }];

        let response = client
            .create_chat_completion(&OpenAiModel::Gpt4oMini, chat_history, tools)
            .await
            .unwrap();

        assert!(matches!(response, ChatResult::ToolCalls(_)));
    }

    #[tokio::test]
    async fn create_chat_completion_returns_message_based_on_tool_call_response() {
        let client = OpenAIClient::new(std::env::var("OPENAI_API_KEY").unwrap());

        let chat_history = &[
            ChatMessage::User {
                content: "What is the weather in Tokyo?".to_string(),
            },
            ChatMessage::Assistant {
                message: AssistantMessage::ToolCalls(vec![ToolCall {
                    id: "tool_call_id".to_string(),
                    name: "get_weather".to_string(),
                    arguments: HashMap::from([("city".to_string(), "Tokyo".to_string())]),
                }]),
            },
            ChatMessage::Tool {
                content: "The weather in Tokyo is sunny".to_string(),
                tool_call_id: "tool_call_id".to_string(),
                tool_name: "get_weather".to_string(),
            },
        ];

        let tools = &[Tool {
            tool_type: ToolType::Function,
            function: Function {
                name: "get_weather".to_string(),
                description: "Get the weather in a given city".to_string(),
                parameters: Parameters {
                    param_type: "object".to_string(),
                    properties: HashMap::from([(
                        "city".to_string(),
                        Property {
                            prop_type: "string".to_string(),
                            description: "The city to get the weather of".to_string(),
                        },
                    )]),
                    required: vec!["city".to_string()],
                    additional_properties: false,
                },
                strict: true,
            },
        }];
        let response = client
            .create_chat_completion(&OpenAiModel::Gpt4oMini, chat_history, tools)
            .await
            .unwrap();

        match response {
            ChatResult::Message { content, .. } => assert!(content.contains("sunny")),
            _ => panic!("Expected ChatResult::Message"),
        }
    }
}
