use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::env;

#[derive(Serialize, Deserialize)]
struct ChatMessage {
    role: String,
    content: String,
}

#[derive(Serialize)]
struct ChatRequest {
    model: String,
    messages: Vec<ChatMessage>,
}

#[derive(Deserialize)]
struct ChatResponse {
    choices: Vec<Choice>,
}

#[derive(Deserialize)]
struct Choice {
    message: ChatMessage,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv::dotenv().ok();
    let api_key = env::var("OPENAI_API_KEY")?;

    let client = Client::new();

    let body = ChatRequest {
        model: "gpt-4o".to_string(),
        messages: vec![
            ChatMessage {
                role: "user".to_string(),
                content: "Hello, who are you?".to_string(),
            },
        ],
    };

    let res = client
        .post("https://api.openai.com/v1/chat/completions")
        .bearer_auth(api_key)
        .json(&body)
        .send()
        .await?
        .json::<ChatResponse>()
        .await?;

    println!("Response: {}", res.choices[0].message.content);

    Ok(())
}
