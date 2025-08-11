mod helpers;

use code_g::client::model::ChatResult;
use code_g::session::session::ChatSession;
use code_g::session::system_prompt::SystemPromptConfig;
use helpers::mocks::{
    chat_client::MockChatClient, event_handler::MockEventHandler, tool_registry::MockToolRegistry,
};
use std::sync::{Arc, Mutex};

#[tokio::test]
async fn test() {
    let event_handler = MockEventHandler::new(vec!["Hello".to_string()], vec![]);

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
}
