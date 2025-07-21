use crate::openai::model::{AssistantMessage, ChatMessage, OpenAiModel, Tool, ToolCall, ToolType};
use serde::de::Error;
use serde::{Deserialize, Serialize};
use std::convert::TryFrom;

// Request
#[derive(Debug, Serialize, Deserialize)]
pub struct ChatCompletionRequest {
    pub model: OpenAiModel,
    pub messages: Vec<ChatMessageRequest>,
    pub tools: Option<Vec<Tool>>,
    pub response_format: Option<ResponseFormat>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ChatMessageRequest {
    pub role: Role,
    pub content: Option<String>,
    pub tool_calls: Option<Vec<ToolCallResponse>>,
    pub tool_call_id: Option<String>,
}

impl TryFrom<ChatMessage> for ChatMessageRequest {
    type Error = serde_json::Error;

    fn try_from(chat_message: ChatMessage) -> Result<Self, Self::Error> {
        match chat_message {
            ChatMessage::System { content } => Ok(ChatMessageRequest {
                role: Role::System,
                content: Some(content),
                tool_calls: None,
                tool_call_id: None,
            }),
            ChatMessage::User { content } => Ok(ChatMessageRequest {
                role: Role::User,
                content: Some(content),
                tool_calls: None,
                tool_call_id: None,
            }),
            ChatMessage::Assistant { message } => match message {
                AssistantMessage::Content(content) => Ok(ChatMessageRequest {
                    role: Role::Assistant,
                    content: Some(content),
                    tool_calls: None,
                    tool_call_id: None,
                }),
                AssistantMessage::ToolCalls(tool_calls) => {
                    let tool_calls = tool_calls
                        .into_iter()
                        .map(ToolCallResponse::try_from)
                        .collect::<Result<Vec<ToolCallResponse>, serde_json::Error>>()?;
                    Ok(ChatMessageRequest {
                    role: Role::Assistant,
                    content: None,
                    tool_calls: Some(tool_calls),
                    tool_call_id: None,
                    })
                }
            },
            ChatMessage::Tool {
                content,
                tool_call_id,
                tool_name: _,
            } => Ok(ChatMessageRequest {
                role: Role::Tool,
                content: Some(content),
                tool_calls: None,
                tool_call_id: Some(tool_call_id),
            }),
        }
    }
}

impl TryFrom<ChatMessageRequest> for ChatMessage {
    type Error = serde_json::Error;

    fn try_from(chat_message_request: ChatMessageRequest) -> Result<Self, Self::Error> {
        match chat_message_request.role {
            Role::System => {
                let content = chat_message_request.content.ok_or(serde_json::Error::custom(
                    "System message must have content",
                ))?;
                Ok(ChatMessage::System { content })
            }
            Role::User => Ok(ChatMessage::User {
                content: chat_message_request
                    .content
                    .ok_or(serde_json::Error::custom("User message must have content"))?,
            }),
            Role::Assistant => Ok(ChatMessage::Assistant {
                message: if let Some(content) = chat_message_request.content {
                    AssistantMessage::Content(content)
                } else if let Some(tool_calls) = chat_message_request.tool_calls {
                    AssistantMessage::ToolCalls(
                        tool_calls
                            .into_iter()
                            .map(ToolCall::try_from)
                            .collect::<Result<Vec<ToolCall>, serde_json::Error>>()?,
                    )
                } else {
                    return Err(serde_json::Error::custom(
                        "Assistant message must have content or tool_calls",
                    ));
                },
            }),
            Role::Tool => {
                let content = chat_message_request
                    .content
                    .ok_or(serde_json::Error::custom("Tool message must have content"))?;
                let tool_call = chat_message_request.tool_calls.and_then(|mut calls| calls.pop()).ok_or(
                    serde_json::Error::custom("Tool message must have a tool_call"),
                )?;
                let tool_call_id = tool_call.id.clone();
                let tool_name = tool_call.function.name;

                Ok(ChatMessage::Tool {
                    content,
                    tool_call_id,
                    tool_name,
                })
            }
        }
    }
}

// Role
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum Role {
    System,
    User,
    Assistant,
    Tool,
}

// Response
#[derive(Deserialize, Debug, Serialize)]
pub struct ChatCompletionResponse {
    pub id: String,
    pub object: String,
    pub created: u64,
    pub model: String, // Different format than the model enum
    pub choices: Vec<ChoiceResponse>,
}

#[derive(Deserialize, Debug, Serialize)]
pub struct ChoiceResponse {
    pub index: u64,
    pub message: MessageResponse,
    pub finish_reason: String,
}

#[derive(Deserialize, Debug, Serialize)]
pub struct MessageResponse {
    pub role: Role,
    pub content: Option<String>,
    pub tool_calls: Option<Vec<ToolCallResponse>>,
}

#[derive(Deserialize, Debug, Serialize)]
pub struct ContentResponse {
    pub message: String,
    pub turn_over: bool,
}

impl TryFrom<&str> for ContentResponse {
    type Error = serde_json::Error;

    fn try_from(content: &str) -> Result<Self, Self::Error> {
        serde_json::from_str(content)
    }
}

#[derive(Deserialize, Debug, Serialize)]
pub struct ToolCallResponse {
    pub id: String,
    #[serde(rename = "type")]
    pub tool_type: ToolType,
    pub function: FunctionResponse,
}

impl TryFrom<ToolCall> for ToolCallResponse {
    type Error = serde_json::Error;

    fn try_from(tool_call: ToolCall) -> Result<Self, Self::Error> {
        let arguments = serde_json::to_string(&tool_call.arguments)?;
        Ok(Self {
            id: tool_call.id,
            tool_type: ToolType::Function,
            function: FunctionResponse {
                name: tool_call.name,
                arguments,
            },
        })
    }
}

impl TryFrom<ToolCallResponse> for ToolCall {
    type Error = serde_json::Error;

    fn try_from(tool_call_response: ToolCallResponse) -> Result<Self, Self::Error> {
        let arguments = serde_json::from_str(&tool_call_response.function.arguments)?;
        Ok(Self {
            id: tool_call_response.id,
            name: tool_call_response.function.name,
            arguments,
        })
    }
}

#[derive(Deserialize, Debug, Serialize)]
pub struct FunctionResponse {
    pub name: String,
    pub arguments: String,
}

// Response Format
#[derive(Deserialize, Debug, Serialize)]
pub struct ResponseFormat {
    #[serde(rename = "type")]
    pub response_format_type: String, // "json_schema"
    pub json_schema: JsonSchema,
}

#[derive(Deserialize, Debug, Serialize)]
pub struct JsonSchema {
    pub name: String,
    pub schema: serde_json::Value,
}
