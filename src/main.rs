use std::{collections::HashMap, env};
use code_g::openai::client::OpenAIClient;
use code_g::openai::model::{ChatMessage, Role, OpenAiModel, Tool, ToolType, Parameters, Property, Function};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let api_key = env::var("OPENAI_API_KEY")?;
    let client = OpenAIClient::new(api_key);

    let mut messages = vec![
        ChatMessage {
            role: Role::User,
            content: Some("What is in the poem.txt file?".to_string()),
            tool_calls: None,
        },
    ];

    let tools = vec![
        Tool {
            tool_type: ToolType::Function,
            function: Function {
                name: "read_file".to_string(),
                description: "Read the content of a file".to_string(),
                parameters: Parameters {
                param_type: "object".to_string(),
                properties: HashMap::from([
                    ("path".to_string(), Property {
                        prop_type: "string".to_string(),
                        description: "The path to the file to read".to_string(),
                    }),
                ]),
                required: vec!["path".to_string()],
                additional_properties: false,
                },
                strict: true,
            },
        }
    ];

    let response = client.create_chat_completion(
        &OpenAiModel::Gpt4oMini,
        &messages,
        &tools,
    ).await?;
    println!("Response: {:?}", response);


    Ok(())
}
