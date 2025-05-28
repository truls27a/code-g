// src/main.rs
use anyhow::{anyhow, Result};
use log::{debug, error, info, warn};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::{env, fs};
use tokio::io::{self, AsyncBufReadExt};

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    env_logger::init();
    info!("Starting chat application");

    dotenvy::dotenv().ok();
    debug!("Loaded environment variables");
    
    let api_key = env::var("OPENAI_API_KEY")
        .map_err(|_| {
            error!("OPENAI_API_KEY environment variable not set");
            anyhow!("Set the OPENAI_API_KEY environment variable")
        })?;
    info!("OpenAI API key loaded successfully");

    let client = Client::new();
    debug!("HTTP client initialized");
    
    let mut messages: Vec<Message> = vec![Message::system(
        "You are a helpful assistant. Feel free to call the read_file function when useful.",
    )];
    debug!("Initial system message added to conversation");

    let tools = [Tool::read_file_schema()];
    debug!("Tools schema initialized");

    let stdin = io::BufReader::new(io::stdin());
    let mut lines = stdin.lines();

    println!("👋  Type your message (Ctrl-C to quit):");

    while let Ok(Some(user_input)) = lines.next_line().await {
        if user_input.trim().is_empty() {
            continue;
        }
        
        info!("User input received: {}", user_input.chars().take(100).collect::<String>());
        messages.push(Message::user(user_input));

        // --- 1. Ask the model (may or may not include a tool call) --------------------------
        debug!("Sending chat completion request to OpenAI");
        let mut response = create_chat_completion(&client, &api_key, &messages, &tools).await?;

        // --- 2. If the model decided to call tools, execute them and loop once -------------
        if let Some(tool_calls) = response.tool_calls.take() {
            info!("Model requested {} tool call(s)", tool_calls.len());
            
            // First, add the assistant message with tool calls to the conversation
            messages.push(Message::assistant_with_tool_calls(tool_calls.clone()));
            
            for tc in tool_calls {
                debug!("Processing tool call: {} ({})", tc.function.name, tc.id);
                
                if tc.function.name == "read_file" {
                    // Parse {"path":"..."}
                    #[derive(Deserialize)]
                    struct Args {
                        path: String,
                    }
                    let Args { path } = serde_json::from_str(&tc.function.arguments)
                        .map_err(|e| {
                            error!("Failed to parse tool call arguments: {}", e);
                            anyhow!("Invalid tool call arguments: {}", e)
                        })?;
                    
                    info!("Reading file: {}", path);
                    let file_contents = fs::read_to_string(&path)
                        .map_err(|e| {
                            error!("Failed to read file '{}': {}", path, e);
                            anyhow!("read_file error on {path:?}: {e}")
                        })?;

                    let truncated_content = file_contents
                        .chars()
                        .take(8_000) // keep context small – truncate big files
                        .collect::<String>();
                    
                    if file_contents.len() > 8_000 {
                        warn!("File '{}' was truncated from {} to 8000 characters", path, file_contents.len());
                    }
                    
                    debug!("File read successfully, content length: {} characters", truncated_content.len());

                    // Append tool result so the model can craft its answer
                    messages.push(Message::tool(&tc.id, &truncated_content));
                }
            }
            
            // Ask the model again, this time including the tool results
            debug!("Sending follow-up chat completion request with tool results");
            response = create_chat_completion(&client, &api_key, &messages, &tools).await?;
        }

        // --- 3. Show final assistant answer and store it in the chat history ---------------
        if let Some(content) = response.content {
            info!("Assistant response received, length: {} characters", content.len());
            println!("\n🤖 {content}\n");
            messages.push(Message::assistant(content));
        } else {
            warn!("No assistant content returned in response");
            println!("(No assistant content returned)\n");
        }
        print!("💬  ");
    }
    
    info!("Chat application shutting down");
    Ok(())
}

// ------------------------------------------------------------------------------------------
// OpenAI helper -----------------------------------------------------------------------------
// ------------------------------------------------------------------------------------------

#[derive(Serialize)]
struct ChatRequest<'a> {
    model: &'static str,
    messages: &'a [Message],
    tools: &'a [Tool],
    #[serde(skip_serializing_if = "Option::is_none")]
    tool_choice: Option<&'static str>,
}

#[derive(Deserialize)]
struct ChatResponse {
    choices: Vec<Choice>,
}

#[derive(Deserialize)]
struct Choice {
    message: AssistantMessage,
}

#[derive(Deserialize)]
struct AssistantMessage {
    #[serde(default)]
    content: Option<String>,
    #[serde(default)]
    tool_calls: Option<Vec<ToolCall>>,
}

#[derive(Deserialize, Clone, Serialize)]
struct ToolCall {
    id: String,
    #[serde(rename = "type")]
    _type: String,
    function: ToolCallFunction,
}

#[derive(Deserialize, Clone, Serialize)]
struct ToolCallFunction {
    name: String,
    arguments: String,
}

async fn create_chat_completion(
    client: &Client,
    api_key: &str,
    messages: &[Message],
    tools: &[Tool],
) -> Result<AssistantMessage> {
    debug!("Creating chat completion request with {} messages", messages.len());
    
    let req_body = ChatRequest {
        model: "gpt-4o-mini", // or any chat-model that supports function calling
        messages,
        tools,
        tool_choice: Some("auto"),
    };
    
    debug!("Sending request to OpenAI API");
    let res = client
        .post("https://api.openai.com/v1/chat/completions")
        .bearer_auth(api_key)
        .json(&req_body)
        .send()
        .await
        .map_err(|e| {
            error!("Failed to send request to OpenAI API: {}", e);
            e
        })?
        .error_for_status()
        .map_err(|e| {
            error!("OpenAI API returned error status: {}", e);
            e
        })?;

    debug!("Received response from OpenAI API");
    let chat_res: ChatResponse = res.json().await
        .map_err(|e| {
            error!("Failed to parse OpenAI API response: {}", e);
            e
        })?;
    
    let message = chat_res.choices.into_iter().next().unwrap().message;
    
    if message.content.is_some() {
        debug!("Response contains content");
    }
    if message.tool_calls.is_some() {
        debug!("Response contains tool calls");
    }
    
    Ok(message)
}

// ------------------------------------------------------------------------------------------
// Schema & message helpers -----------------------------------------------------------------
// ------------------------------------------------------------------------------------------

#[derive(Clone, Serialize, Deserialize)]
struct Tool {
    #[serde(rename = "type")]
    _type: &'static str,
    function: ToolSchema,
}

impl Tool {
    fn read_file_schema() -> Self {
        Tool {
            _type: "function",
            function: ToolSchema {
                name: "read_file",
                description: "Read the content of a UTF-8 text file given an absolute or relative path.",
                parameters: json!({
                    "type": "object",
                    "properties": {
                        "path": {
                            "type": "string",
                            "description": "Absolute or relative path to the text file on disk."
                        }
                    },
                    "required": ["path"],
                    "additionalProperties": false
                }),
                strict: true,
            },
        }
    }
}

#[derive(Clone, Serialize, Deserialize)]
struct ToolSchema {
    name: &'static str,
    description: &'static str,
    parameters: Value,
    strict: bool,
}

#[derive(Clone, Serialize)]
struct Message {
    role: &'static str,
    #[serde(skip_serializing_if = "Option::is_none")]
    content: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    name: Option<&'static str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    tool_call_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    tool_calls: Option<Vec<ToolCall>>,
}

impl Message {
    fn system<S: Into<String>>(text: S) -> Self {
        Message {
            role: "system",
            content: Some(text.into()),
            name: None,
            tool_call_id: None,
            tool_calls: None,
        }
    }
    fn user<S: Into<String>>(text: S) -> Self {
        Message {
            role: "user",
            content: Some(text.into()),
            name: None,
            tool_call_id: None,
            tool_calls: None,
        }
    }
    fn assistant<S: Into<String>>(text: S) -> Self {
        Message {
            role: "assistant",
            content: Some(text.into()),
            name: None,
            tool_call_id: None,
            tool_calls: None,
        }
    }
    fn assistant_with_tool_calls(tool_calls: Vec<ToolCall>) -> Self {
        Message {
            role: "assistant",
            content: None,
            name: None,
            tool_call_id: None,
            tool_calls: Some(tool_calls),
        }
    }
    fn tool<S: Into<String>>(call_id: &str, result: S) -> Self {
        Message {
            role: "tool",
            content: Some(result.into()),
            name: None,
            tool_call_id: Some(call_id.to_string()),
            tool_calls: None,
        }
    }
}
