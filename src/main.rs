use std::env;
use code_g::openai::client::OpenAIClient;
use code_g::openai::schema::{ChatMessage, Role};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let api_key = env::var("OPENAI_API_KEY")?;
    let client = OpenAIClient::new(api_key);
    let response = client.create_chat_completion("gpt-4o-mini", vec![
        ChatMessage {
            role: Role::User,
            content: "Say 'hi' in Swedish in all lowercase, nothing else.".to_string(),
        },
    ]).await?;
    println!("Response: {}", response);
    Ok(())
}
