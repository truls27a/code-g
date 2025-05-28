use reqwest::Client;
use serde_json::{json, Value};
use std::env;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Read your API key from the environment
    let api_key = env::var("OPENAI_API_KEY")
        .expect("Please set the OPENAI_API_KEY environment variable");

    // Build the HTTP client
    let client = Client::new();

    // Define the function schema
    let functions = json!([{
        "name": "get_weather",
        "description": "Get current temperature for a given location.",
        "parameters": {
            "type": "object",
            "properties": {
                "location": {
                    "type": "string",
                    "description": "City and country e.g. Bogotá, Colombia"
                }
            },
            "required": ["location"],
            "additionalProperties": false
        }
    }]);

    // Build the request payload
    let body = json!({
        "model": "gpt-4.1",
        "messages": [
            { "role": "user", "content": "What is the weather like in Paris today?" }
        ],
        "functions": functions
    });

    // Send the request
    let res = client
        .post("https://api.openai.com/v1/chat/completions")
        .bearer_auth(api_key)
        .json(&body)
        .send()
        .await?
        .error_for_status()? // make sure we catch HTTP errors
        .json::<Value>()
        .await?;

    // Drill down into the response to find any function calls
    if let Some(choice) = res["choices"].as_array().and_then(|arr| arr.get(0)) {
        if let Some(tool_call) = choice["message"]["function_call"].as_object() {
            println!("Function call detected:");
            println!("  name: {}", tool_call["name"].as_str().unwrap_or(""));
            println!("  arguments: {}", tool_call["arguments"]);
        } else {
            println!("No function call in the response. Full message:");
            println!("{}", choice["message"]["content"]);
        }
    } else {
        eprintln!("Unexpected response format: {}", res);
    }

    Ok(())
}
