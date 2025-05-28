// src/main.rs
use anyhow::{anyhow, Result};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::{env, fs};
use tokio::io::{self, AsyncBufReadExt};

#[tokio::main]
async fn main() -> Result<()> {
    dotenvy::dotenv().ok();
    let api_key = env::var("OPENAI_API_KEY")
        .map_err(|_| anyhow!("Set the OPENAI_API_KEY environment variable"))?;

    let client = Client::new();
    let mut messages: Vec<Message> = vec![Message::system(
        "You are a helpful assistant. Feel free to call the read_file function when useful.",
    )];

    let tools = [Tool::read_file_schema()];

    let stdin = io::BufReader::new(io::stdin());
    let mut lines = stdin.lines();

    println!("👋  Type your message (Ctrl-C to quit):");

    while let Ok(Some(user_input)) = lines.next_line().await {
        if user_input.trim().is_empty() {
            continue;
        }
        messages.push(Message::user(user_input));

        // --- 1. Ask the model (may or may not include a tool call) --------------------------
        let mut response = create_chat_completion(&client, &api_key, &messages, &tools).await?;

        // --- 2. If the model decided to call tools, execute them and loop once -------------
        if let Some(tool_calls) = response.tool_calls.take() {
            for tc in tool_calls {
                if tc.function.name == "read_file" {
                    // Parse {"path":"..."}
                    #[derive(Deserialize)]
                    struct Args {
                        path: String,
                    }
                    let Args { path } = serde_json::from_str(&tc.function.arguments)?;
                    let file_contents = fs::read_to_string(&path)
                        .map_err(|e| anyhow!("read_file error on {path:?}: {e}"))?;

                    // Append tool result so the model can craft its answer
                    messages.push(Message::tool(
                        &tc.id,
                        &file_contents
                            .chars()
                            .take(8_000) // keep context small – truncate big files
                            .collect::<String>(),
                    ));
                }
            }
            // Ask the model again, this time including the tool results
            response = create_chat_completion(&client, &api_key, &messages, &tools).await?;
        }

        // --- 3. Show final assistant answer and store it in the chat history ---------------
        if let Some(content) = response.content {
            println!("\n🤖 {content}\n");
            messages.push(Message::assistant(content));
        } else {
            println!("(No assistant content returned)\n");
        }
        print!("💬  ");
    }
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

#[derive(Deserialize)]
struct ToolCall {
    id: String,
    #[serde(rename = "type")]
    _type: String,
    function: ToolCallFunction,
}

#[derive(Deserialize)]
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
    let req_body = ChatRequest {
        model: "gpt-4o-mini", // or any chat-model that supports function calling
        messages,
        tools,
        tool_choice: Some("auto"),
    };
    let res = client
        .post("https://api.openai.com/v1/chat/completions")
        .bearer_auth(api_key)
        .json(&req_body)
        .send()
        .await?
        .error_for_status()?;

    let chat_res: ChatResponse = res.json().await?;
    Ok(chat_res.choices.into_iter().next().unwrap().message)
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
}

impl Message {
    fn system<S: Into<String>>(text: S) -> Self {
        Message {
            role: "system",
            content: Some(text.into()),
            name: None,
            tool_call_id: None,
        }
    }
    fn user<S: Into<String>>(text: S) -> Self {
        Message {
            role: "user",
            content: Some(text.into()),
            name: None,
            tool_call_id: None,
        }
    }
    fn assistant<S: Into<String>>(text: S) -> Self {
        Message {
            role: "assistant",
            content: Some(text.into()),
            name: None,
            tool_call_id: None,
        }
    }
    fn tool<S: Into<String>>(call_id: &str, result: S) -> Self {
        Message {
            role: "tool",
            content: Some(result.into()),
            name: None,
            tool_call_id: Some(call_id.to_string()),
        }
    }
}
