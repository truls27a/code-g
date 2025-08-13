mod helpers;

use code_g::client::model::{ChatResult, Function, Parameters, Property, Tool, ToolCall, ToolType};
use code_g::session::session::ChatSession;
use code_g::session::system_prompt::SystemPromptConfig;
use helpers::mocks::{
    chat_client::MockChatClient, event_handler::MockEventHandler, tool_registry::MockTool,
    tool_registry::MockToolRegistry,
};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

#[tokio::test]
async fn chat_session_handles_message() {
    let events = Arc::new(Mutex::new(vec![]));
    let event_handler = MockEventHandler::new(events.clone(), vec!["Hello".to_string()], vec![]);

    let client_calls = Arc::new(Mutex::new(vec![]));
    let chat_client = MockChatClient::new(
        vec![Ok(ChatResult::Message {
            content: "Hello human".to_string(),
            turn_over: true,
        })],
        client_calls.clone(),
    );

    let tool_calls = Arc::new(Mutex::new(vec![]));
    let tool_registry = MockToolRegistry::new(vec![], tool_calls.clone());

    let mut chat_session = ChatSession::new(
        Box::new(chat_client),
        Box::new(tool_registry),
        Box::new(event_handler),
        SystemPromptConfig::Default,
    );

    chat_session.run().await.unwrap();

    assert_eq!(client_calls.lock().unwrap().len(), 1);
    assert_eq!(tool_calls.lock().unwrap().len(), 0);
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
            "I'm good, thank you!".to_string(),
        ],
        vec![],
    );

    let client_calls = Arc::new(Mutex::new(vec![]));
    let chat_client = MockChatClient::new(
        vec![
            Ok(ChatResult::Message {
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
            }),
        ],
        client_calls.clone(),
    );
    let tool_calls = Arc::new(Mutex::new(vec![]));
    let tool_registry = MockToolRegistry::new(vec![], tool_calls.clone());

    let mut chat_session = ChatSession::new(
        Box::new(chat_client),
        Box::new(tool_registry),
        Box::new(event_handler),
        SystemPromptConfig::Default,
    );

    chat_session.run().await.unwrap();

    assert_eq!(client_calls.lock().unwrap().len(), 3);
    assert_eq!(tool_calls.lock().unwrap().len(), 0);
    assert_eq!(events.lock().unwrap().len(), 11);
}

#[tokio::test]
async fn chat_session_handles_multiple_assistant_messages_per_turn() {
    let events = Arc::new(Mutex::new(vec![]));
    let event_handler = MockEventHandler::new(
        events.clone(),
        vec!["What is 1+1? Think about it real hard".to_string()],
        vec![],
    );

    let client_calls = Arc::new(Mutex::new(vec![]));
    let chat_client = MockChatClient::new(
        vec![
            Ok(ChatResult::Message {
                content: "Okay lets see. The user is asking me what 1+1 is. I need to think about it real hard".to_string(),
                turn_over: false,
            }),
            Ok(ChatResult::Message {
                content: "I think the answer is 2. I'm not sure if I'm right, as one sand pile plus one sand pile is one big sand pile".to_string(),
                turn_over: false,
            }),
            Ok(ChatResult::Message {
                content: "I'm going to return the answer 2".to_string(),
                turn_over: false,
            }),
            Ok(ChatResult::Message {
                content: "1+1 is 2".to_string(),
                turn_over: true,
            }),
        ],
        client_calls.clone(),
    );
    let tool_calls = Arc::new(Mutex::new(vec![]));
    let tool_registry = MockToolRegistry::new(vec![], tool_calls.clone());

    let mut chat_session = ChatSession::new(
        Box::new(chat_client),
        Box::new(tool_registry),
        Box::new(event_handler),
        SystemPromptConfig::Default,
    );

    chat_session.run().await.unwrap();

    assert_eq!(client_calls.lock().unwrap().len(), 4);
    assert_eq!(tool_calls.lock().unwrap().len(), 0);
    assert_eq!(events.lock().unwrap().len(), 11);
}

#[tokio::test]
async fn chat_session_handles_tool_calls() {
    let events = Arc::new(Mutex::new(vec![]));
    let event_handler = MockEventHandler::new(
        events.clone(),
        vec!["What is the weather in Tokyo?".to_string()],
        vec![],
    );

    let client_calls = Arc::new(Mutex::new(vec![]));
    let chat_client = MockChatClient::new(
        vec![
            Ok(ChatResult::ToolCalls(vec![ToolCall {
                id: "1".to_string(),
                name: "get_weather".to_string(),
                arguments: HashMap::from([("city".to_string(), "Tokyo".to_string())]),
            }])),
            Ok(ChatResult::Message {
                content: "The weather in Tokyo is sunny".to_string(),
                turn_over: true,
            }),
        ],
        client_calls.clone(),
    );

    let tool_calls = Arc::new(Mutex::new(vec![]));
    let tool_registry = MockToolRegistry::new(
        vec![Box::new(MockTool::new(
            "get_weather".to_string(),
            "Get the weather in a city".to_string(),
            Parameters {
                param_type: "object".to_string(),
                properties: HashMap::new(),
                required: vec!["city".to_string()],
                additional_properties: false,
            },
            true,
            false,
            "AI wants to check the weather in Tokyo. Do you approve?".to_string(),
            "The weather in Tokyo is sunny".to_string(),
        ))],
        tool_calls.clone(),
    );

    let mut chat_session = ChatSession::new(
        Box::new(chat_client),
        Box::new(tool_registry),
        Box::new(event_handler),
        SystemPromptConfig::Default,
    );

    chat_session.run().await.unwrap();

    assert_eq!(client_calls.lock().unwrap().len(), 2);
    assert_eq!(tool_calls.lock().unwrap().len(), 1);
    assert_eq!(events.lock().unwrap().len(), 8);
}
