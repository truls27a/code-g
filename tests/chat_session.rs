mod helpers;

use code_g::client::model::ChatResult;
use code_g::session::session::ChatSession;
use code_g::session::system_prompt::SystemPromptConfig;
use helpers::mocks::{
    chat_client::MockChatClient, event_handler::MockEventHandler, tool_registry::MockToolRegistry,
};
use std::sync::{Arc, Mutex};

#[tokio::test]
async fn chat_session_handles_message() {
    let events = Arc::new(Mutex::new(vec![]));
    let event_handler = MockEventHandler::new(events.clone(), vec!["Hello".to_string()], vec![]);

    let calls = Arc::new(Mutex::new(vec![]));
    let chat_client = MockChatClient::new(
        vec![Ok(ChatResult::Message {
            content: "Hello human".to_string(),
            turn_over: true,
        })],
        calls.clone(),
    );
    let tool_registry = MockToolRegistry::new(vec![]);

    let mut chat_session = ChatSession::new(
        Box::new(chat_client),
        Box::new(tool_registry),
        Box::new(event_handler),
        SystemPromptConfig::Default,
    );

    chat_session.run().await.unwrap();

    assert_eq!(calls.lock().unwrap().len(), 1);
    assert_eq!(events.lock().unwrap().len(), 5);
}

#[tokio::test]
async fn chat_session_handles_multiple_messages() {
    let events = Arc::new(Mutex::new(vec![]));
    let event_handler = MockEventHandler::new(
        events.clone(),
        vec![
            "Hello".to_string(),
            "How are you?".to_string(),
            "I'm good, thank you!
}"
            .to_string(),
        ],
        vec![],
    );

    let calls = Arc::new(Mutex::new(vec![]));
    let chat_client = MockChatClient::new(
        vec![Ok(ChatResult::Message {
            content: "Hello human".to_string(),
            turn_over: true,
        }),
        Ok(ChatResult::Message {
            content: "Oh, I feel great. What about you?".to_string(),
            turn_over: true,
        }),
        Ok(ChatResult::Message {
            content: "Thats nice to hear!".to_string(),
            turn_over: true,
        })],
        calls.clone(),
    );
    let tool_registry = MockToolRegistry::new(vec![]);

    let mut chat_session = ChatSession::new(
        Box::new(chat_client),
        Box::new(tool_registry),
        Box::new(event_handler),
        SystemPromptConfig::Default,
    );

    chat_session.run().await.unwrap();

    assert_eq!(calls.lock().unwrap().len(), 3);
    assert_eq!(events.lock().unwrap().len(), 11);
}
