use code_g::chat::session::ChatSession;
use code_g::chat::system_prompt::SystemPromptConfig;
use code_g::openai::client::OpenAIClient;
use code_g::tools::registry::ToolRegistry;
use code_g::tui::tui::Tui;
use std::env;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let api_key = env::var("OPENAI_API_KEY")?;
    let openai_client = OpenAIClient::new(api_key);

    let tools = ToolRegistry::all_tools();

    let tui = Tui::new();

    let mut chat_session = ChatSession::new(
        openai_client,
        tools,
        Box::new(tui),
        SystemPromptConfig::Default,
    );

    chat_session.run().await?;

    Ok(())
}
