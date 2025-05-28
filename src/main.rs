use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::env;
use std::fs;
use std::io::{self, Write};
use log::{debug, error, info, warn};
use env_logger;

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
struct ChatMessage {
    role: String,
    // content is optional because tool calls don't include content
    content: Option<String>,
    // the name of the tool when the role is "tool"
    name: Option<String>,
    // capture any tool calls the assistant makes
    function_call: Option<FunctionCall>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
struct FunctionCall {
    name: String,
    arguments: String, // JSON string in the new API
}

#[derive(Serialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
struct ChatRequest {
    model: String,
    messages: Vec<ChatMessage>,
    // list of tools we offer
    functions: Option<Vec<Tool>>,
}

#[derive(Serialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
struct Tool {
    #[serde(rename = "type")]
    tool_type: String,
    function: FunctionDefinition,
}

#[derive(Serialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
struct FunctionDefinition {
    name: String,
    description: String,
    parameters: serde_json::Value,
}

#[derive(Deserialize, Debug)]
struct ChatResponse {
    choices: Vec<Choice>,
}

#[derive(Deserialize, Debug)]
struct Choice {
    message: ChatMessage,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();
    dotenv::dotenv().ok();
    let api_key = match env::var("OPENAI_API_KEY") {
        Ok(key) => {
            debug!("API key loaded successfully");
            key
        },
        Err(e) => {
            error!("Failed to load API key: {}", e);
            return Err(e.into());
        }
    };
    
    let client = Client::new();
    let mut conversation_history = Vec::new();

    // define our tools using the new format
    let tools = vec![
        Tool {
            tool_type: "function".to_string(),
            function: FunctionDefinition {
                name: "read_file".to_string(),
                description: "Read a file from the local filesystem".to_string(),
                parameters: json!({
                    "type": "object",
                    "properties": {
                        "path": {
                            "type": "string",
                            "description": "The file system path to read"
                        }
                    },
                    "required": ["path"]
                }),
            }
        }
    ];

    info!("Starting code-g CLI");
    println!("Welcome to the code-g CLI! (Type 'quit' to exit)");

    loop {
        print!("> ");
        io::stdout().flush()?;
        let mut user_input = String::new();
        io::stdin().read_line(&mut user_input)?;
        let user_input = user_input.trim();
        if user_input.eq_ignore_ascii_case("quit") {
            info!("User requested to quit");
            println!("Goodbye!");
            break;
        }

        debug!("Received user input: {}", user_input);

        // user message
        conversation_history.push(ChatMessage {
            role: "user".into(),
            content: Some(user_input.to_string()),
            name: None,
            function_call: None,
        });

        debug!("Making API request to OpenAI");
        // first call: with tools enabled
        let body = ChatRequest {
            model: "gpt-4.1".into(),
            messages: conversation_history.clone(),
            functions: Some(tools.clone()),
        };

        debug!("Body: {:?}", body);

        let resp: ChatResponse = match client
            .post("https://api.openai.com/v1/chat/completions")
            .bearer_auth(&api_key)
            .json(&body)
            .send()
            .await {
                Ok(response) => {
                    debug!("Received response from OpenAI");
                    match response.json().await {
                        Ok(json) => json,
                        Err(e) => {
                            error!("Failed to parse OpenAI response: {}", e);
                            return Err(e.into());
                        }
                    }
                },
                Err(e) => {
                    error!("Failed to send request to OpenAI: {}", e);
                    return Err(e.into());
                }
            };

        debug!("Response: {:?}", resp);

        let mut assistant_msg: ChatMessage = resp.choices[0].message.clone();

        debug!("Assistant message: {:?}", assistant_msg);

        // if the model made tool calls…
        if let Some(function_call) = &assistant_msg.function_call {
            debug!("Assistant requested {} tool call(s)", function_call.name);
            
            // Add the assistant's message with tool calls to history
            conversation_history.push(assistant_msg.clone());
            
            // Process each tool call
            if function_call.name == "read_file" {
                debug!("Processing read_file tool call with id: {}", function_call.name);
                
                    // parse the arguments (now a JSON string)
                    let args: serde_json::Value = match serde_json::from_str(&function_call.arguments) {
                        Ok(args) => args,
                        Err(e) => {
                            error!("Failed to parse tool call arguments: {}", e);
                            continue;
                        }
                    };
                    
                    let path = args
                        .get("path")
                        .and_then(|v| v.as_str())
                        .unwrap_or("");
                    debug!("Attempting to read file: {}", path);
                    
                    // actually read the file
                    let file_content = match fs::read_to_string(path) {
                        Ok(txt) => {
                            debug!("Successfully read file: {}", path);
                            txt
                        },
                        Err(e) => {
                            warn!("Error reading file {}: {}", path, e);
                            format!("Error reading {}: {}", path, e)
                        }
                    };

                    // push our tool response
                    conversation_history.push(ChatMessage {
                        role: "tool".into(),
                        name: None,
                        content: Some(file_content),
                        function_call: None,
                    });
            }

            debug!("Making follow-up API request to OpenAI");
            // now ask the model to continue, without re-including tools
            let followup = ChatRequest {
                model: "gpt-4.1".into(),
                messages: conversation_history.clone(),
                functions: None, // no need to supply again
            };

            let resp2: ChatResponse = match client
                .post("https://api.openai.com/v1/chat/completions")
                .bearer_auth(&api_key)
                .json(&followup)
                .send()
                .await {
                    Ok(response) => {
                        debug!("Received follow-up response from OpenAI");
                        match response.json().await {
                            Ok(json) => json,
                            Err(e) => {
                                error!("Failed to parse follow-up OpenAI response: {}", e);
                                return Err(e.into());
                            }
                        }
                    },
                    Err(e) => {
                        error!("Failed to send follow-up request to OpenAI: {}", e);
                        return Err(e.into());
                    }
                };

            assistant_msg = resp2.choices[0].message.clone();
        }

        // print assistant output
        if let Some(content) = &assistant_msg.content {
            debug!("Printing assistant response");
            println!("\nAssistant: {}\n", content);
        }

        // add assistant to history
        conversation_history.push(assistant_msg);
        debug!("Conversation history updated, length: {}", conversation_history.len());
    }

    Ok(())
}
