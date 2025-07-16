use code_g::chat::session::{ChatSession, SystemPromptConfig};
use code_g::openai::client::OpenAIClient;
use code_g::tools::registry::ToolRegistry;
use std::env;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let api_key = env::var("OPENAI_API_KEY")?;
    let openai_client = OpenAIClient::new(api_key);

    let tools = ToolRegistry::all_tools();

    let mut chat_session =
        ChatSession::new(openai_client, tools, SystemPromptConfig::Default, false);

    chat_session.run().await?;

    Ok(())
}
