use code_g::chat::session::ChatSession;
use code_g::openai::client::OpenAIClient;
use code_g::openai::model::{Function, Parameters, Property, Tool, ToolType};
use std::collections::HashMap;
use std::env;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let api_key = env::var("OPENAI_API_KEY")?;
    let openai_client = OpenAIClient::new(api_key);
    let mut chat_session = ChatSession::new(
        openai_client,
        vec![Tool {
            tool_type: ToolType::Function,
            function: Function {
                name: "read_file".to_string(),
                description: "Read the content of a file".to_string(),
                parameters: Parameters {
                    param_type: "object".to_string(),
                    properties: HashMap::from([(
                        "path".to_string(),
                        Property {
                            prop_type: "string".to_string(),
                            description: "The path to the file to read".to_string(),
                        },
                    )]),
                    required: vec!["path".to_string()],
                    additional_properties: false,
                },
                strict: true,
            },
        }],
    );

    let response = chat_session.send_message("What is in the poem.txt file?").await?;
    println!("{}", response);
    let response = chat_session.send_message("Do a litteraly analisys of the poem").await?;
    println!("{}", response);

    Ok(())
}
