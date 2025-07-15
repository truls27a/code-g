use code_g::chat::session::ChatSession;
use code_g::openai::client::OpenAIClient;
use code_g::tools::registry::ToolRegistry;
use std::env;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let api_key = env::var("OPENAI_API_KEY")?;
    let openai_client = OpenAIClient::new(api_key);
    
    let tools = ToolRegistry::all_tools();

    let mut chat_session = ChatSession::new(
        openai_client,
        tools,
    );

    let response = chat_session
        .send_message("What is in the poem.txt file?")
        .await?;
    println!("{}", response);
    let response = chat_session
        .send_message("Rewrite it to swedish")
        .await?;
    println!("{}", response);

    Ok(())
}
