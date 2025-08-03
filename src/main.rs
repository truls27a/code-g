use code_g::chat::session::ChatSession;
use code_g::chat::system_prompt::SystemPromptConfig;
use code_g::chat_client::client::OpenAIClient;
use code_g::tools::registry::Registry;
use code_g::tui::tui::Tui;
use std::env;

// Entry point for the CodeG terminal chat application.
//
// Initializes the chat session and TUI.
// Responsible for starting the async runtime and wiring
// together the OpenAI client, tools, and TUI renderer.
//
// Panics if required environment variables (e.g. OPENAI_API_KEY) are missing.
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let api_key = env::var("OPENAI_API_KEY")?;
    let openai_client = OpenAIClient::new(api_key);

    let tools = Registry::all_tools();

    let tui = Tui::new();

    let mut chat_session = ChatSession::new(
        Box::new(openai_client),
        tools,
        Box::new(tui),
        SystemPromptConfig::Default,
    );

    chat_session.run().await?;

    Ok(())
}
