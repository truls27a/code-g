use std::env;
use code_g::openai::client::OpenAIClient;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let api_key = env::var("OPENAI_API_KEY")?;
    let client = OpenAIClient::new(api_key);
    Ok(())
}
