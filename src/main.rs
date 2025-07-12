use code_g::chat::session::ChatSession;
use code_g::openai::client::OpenAIClient;
use code_g::tools::registry::ToolRegistry;
use code_g::tools::read_file::ReadFileTool;
use std::env;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let api_key = env::var("OPENAI_API_KEY")?;
    let openai_client = OpenAIClient::new(api_key);
    
    let tools = ToolRegistry::from(vec![Box::new(ReadFileTool)]);

    let mut chat_session = ChatSession::new(
        openai_client,
        tools,
    );

    let response = chat_session
        .send_message("What is in the poem.txt file? Read through it 3 times (important!)")
        .await?;
    println!("{}", response);
    let response = chat_session
        .send_message("Do a litteraly analisys of the poem")
        .await?;
    println!("{}", response);

    Ok(())
}
