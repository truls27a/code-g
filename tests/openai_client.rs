use code_g::client::error::ChatClientError;
use code_g::client::model::{ChatMessage, ChatResult, Model};
use code_g::client::model::{Function, Parameters, Property, Tool, ToolType};
use code_g::client::providers::openai::client::OpenAIClient;
use code_g::client::providers::openai::schema::Model as OpenAiModel;
use code_g::client::traits::ChatClient;
use std::collections::HashMap;

/// These are live integration tests that hit the real OpenAI Chat Completions API.
/// They are ignored by default. To run them locally:
///
/// 1) Set an environment variable with your API key:
///    PowerShell:
///      $env:OPENAI_API_KEY = "sk-..."
///
/// 2) Run the ignored tests explicitly:
///      cargo test --test openai_client -- --ignored

#[tokio::test]
#[ignore]
async fn openai_client_returns_message_for_simple_prompt() {
    let api_key = std::env::var("OPENAI_API_KEY")
        .expect("Set OPENAI_API_KEY to run this ignored integration test");

    let client = OpenAIClient::new(api_key);

    let chat_history = vec![ChatMessage::User {
        content: "Respond with a short greeting. Set turn_over to true.".to_string(),
    }];

    let result = client
        .create_chat_completion(&Model::OpenAi(OpenAiModel::Gpt4oMini), &chat_history, &[])
        .await;

    match result {
        Ok(ChatResult::Message { content, turn_over }) => {
            assert!(!content.trim().is_empty(), "expected non-empty content");
            assert!(turn_over, "expected turn_over to be true");
        }
        Ok(ChatResult::ToolCalls(_)) => panic!("unexpected tool calls when no tools were provided"),
        Err(err) => panic!("unexpected error: {:?}", err),
    }
}

#[tokio::test]
#[ignore]
async fn openai_client_returns_tool_call_when_tool_available() {
    let api_key = std::env::var("OPENAI_API_KEY")
        .expect("Set OPENAI_API_KEY to run this ignored integration test");

    let client = OpenAIClient::new(api_key);

    let tool = Tool {
        tool_type: ToolType::Function,
        function: Function {
            name: "echo".to_string(),
            description: "Echo back the provided text".to_string(),
            parameters: Parameters {
                param_type: "object".to_string(),
                properties: HashMap::from([(
                    "text".to_string(),
                    Property {
                        prop_type: "string".to_string(),
                        description: "The text to echo".to_string(),
                    },
                )]),
                required: vec!["text".to_string()],
                additional_properties: false,
            },
            strict: true,
        },
    };

    let chat_history = vec![
        ChatMessage::System {
            content: "You can call a tool named 'echo' that echoes the provided text. If possible, respond only by calling this tool.".to_string(),
        },
        ChatMessage::User {
            content: "Please call the echo tool with text 'hello'. Do not provide a direct textual answer.".to_string(),
        },
    ];

    let result = client
        .create_chat_completion(
            &Model::OpenAi(OpenAiModel::Gpt4oMini),
            &chat_history,
            &[tool],
        )
        .await;

    match result {
        Ok(ChatResult::ToolCalls(calls)) => {
            assert!(!calls.is_empty(), "expected at least one tool call");
            let call = &calls[0];
            assert_eq!(call.name, "echo");
            assert_eq!(
                call.arguments.get("text").map(|s| s.as_str()),
                Some("hello")
            );
        }
        Ok(ChatResult::Message { content, .. }) => {
            panic!("expected tool calls, got message: {}", content);
        }
        Err(err) => panic!("unexpected error: {:?}", err),
    }
}

#[tokio::test]
#[ignore]
async fn openai_client_returns_message_with_system_and_user() {
    let api_key = std::env::var("OPENAI_API_KEY")
        .expect("Set OPENAI_API_KEY to run this ignored integration test");

    let client = OpenAIClient::new(api_key);

    let chat_history = vec![
        ChatMessage::System {
            content: "You are a helpful assistant. Provide concise answers. Always set turn_over to true.".to_string(),
        },
        ChatMessage::User {
            content: "Give a 3-word greeting".to_string(),
        },
    ];

    let result = client
        .create_chat_completion(&Model::OpenAi(OpenAiModel::Gpt4oMini), &chat_history, &[])
        .await;

    match result {
        Ok(ChatResult::Message { content, turn_over }) => {
            assert!(!content.trim().is_empty(), "expected non-empty content");
            assert!(turn_over, "expected turn_over to be true");
        }
        Ok(ChatResult::ToolCalls(_)) => panic!("unexpected tool calls when no tools were provided"),
        Err(err) => panic!("unexpected error: {:?}", err),
    }
}

#[tokio::test]
#[ignore]
async fn openai_client_invalid_api_key_returns_unauthorized() {
    let client = OpenAIClient::new("invalid-api-key".to_string());

    let chat_history = vec![ChatMessage::User {
        content: "Say hi".to_string(),
    }];

    let result = client
        .create_chat_completion(&Model::OpenAi(OpenAiModel::Gpt4oMini), &chat_history, &[])
        .await;

    match result {
        Err(ChatClientError::InvalidApiKey) => {}
        other => panic!("expected InvalidApiKey error, got: {:?}", other),
    }
}
