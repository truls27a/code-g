use code_g::client::error::ChatClientError;
use code_g::client::model::{ChatMessage, ChatResult, Model};
use code_g::client::providers::openai::client::OpenAIClient;
use code_g::client::providers::openai::schema::Model as OpenAiModel;
use code_g::client::traits::ChatClient;

/// These are live integration tests that hit the real OpenAI Chat Completions API.
/// They are ignored by default. To run them locally:
///
/// 1) Set an environment variable with your API key:
///    PowerShell:
///      $env:OPENAI_API_KEY = "sk-..."
///
/// 2) Run the ignored tests explicitly:
///      cargo test --test openai_client_basic -- --ignored

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
