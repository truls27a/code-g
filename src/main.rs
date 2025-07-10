use std::{collections::HashMap, env};
use code_g::openai::client::OpenAIClient;
use code_g::openai::model::{ChatMessage, OpenAiModel, Tool, ToolType, Parameters, Property, Function, ChatResult};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let api_key = env::var("OPENAI_API_KEY")?;
    let client = OpenAIClient::new(api_key);

    let mut messages = vec![
        ChatMessage::User {
            content: "What is in the poem.txt file?".to_string(),
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

    println!("Response: {:?}", response.clone());

    match response {
        ChatResult::Message(content) => {
            messages.push(ChatMessage::Assistant {
                content: Some(content),
                tool_calls: None,
            });
        }
        ChatResult::ToolCalls(tool_calls) => {
            messages.push(ChatMessage::Assistant {
                content: None,
                tool_calls: Some(tool_calls.clone()),
            });
            messages.push(ChatMessage::Tool { content: "To be or not to be, that is the question!".to_string(), tool_call_id: tool_calls[0].id.clone() });
        }
    };

    println!("Messages: {:?}", messages.clone());

    let response = client.create_chat_completion(
        &OpenAiModel::Gpt4oMini,
        &messages,
        &tools,
    ).await?;

    println!("Response: {:?}", response.clone());




    Ok(())
}
