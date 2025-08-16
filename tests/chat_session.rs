mod helpers;

use code_g::client::model::{
    AssistantMessage, ChatMessage, ChatResult, Model, Parameters, ToolCall,
};
use code_g::client::providers::openai::schema::Model as OpenAiModel;
use code_g::session::event::Event;
use code_g::session::session::ChatSession;
use code_g::session::system_prompt::{SYSTEM_PROMPT, SystemPromptConfig};
use helpers::assertions::{assert_chat_history, assert_events, assert_tool_calls};
use helpers::mocks::{
    chat_client::MockChatClient, event_handler::MockEventHandler, tool_registry::MockTool,
    tool_registry::MockToolRegistry,
};
use helpers::scenario::ScenarioBuilder;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

#[tokio::test]
async fn chat_session_handles_message() {
    let scenario = ScenarioBuilder::new()
        .inputs(["Hello"])
        .then_message("Hello human", true)
        .run()
        .await;

    assert_events(
        &scenario.events,
        &[
            Event::SessionStarted,
            Event::ReceivedUserMessage {
                message: "Hello".to_string(),
            },
            Event::AwaitingAssistantResponse,
            Event::ReceivedAssistantMessage {
                message: "Hello human".to_string(),
            },
            Event::SessionEnded,
        ],
    );

    assert_chat_history(
        &scenario.client_calls.lock().unwrap().clone()[0].1,
        &[
            ChatMessage::System {
                content: SYSTEM_PROMPT.to_string(),
            },
            ChatMessage::User {
                content: "Hello".to_string(),
            },
        ],
    );
}

#[tokio::test]
async fn chat_session_handles_multiple_messages() {
    let scenario = ScenarioBuilder::new()
        .inputs(["Hello", "How are you?", "I'm good, thank you!"])
        .then_message("Hello human", true)
        .then_message("Oh, I feel great. What about you?", true)
        .then_message("Thats nice to hear!", true)
        .run()
        .await;

    assert_events(
        &scenario.events,
        &[
            Event::SessionStarted,
            Event::ReceivedUserMessage {
                message: "Hello".to_string(),
            },
            Event::AwaitingAssistantResponse,
            Event::ReceivedAssistantMessage {
                message: "Hello human".to_string(),
            },
            Event::ReceivedUserMessage {
                message: "How are you?".to_string(),
            },
            Event::AwaitingAssistantResponse,
            Event::ReceivedAssistantMessage {
                message: "Oh, I feel great. What about you?".to_string(),
            },
            Event::ReceivedUserMessage {
                message: "I'm good, thank you!".to_string(),
            },
            Event::AwaitingAssistantResponse,
            Event::ReceivedAssistantMessage {
                message: "Thats nice to hear!".to_string(),
            },
            Event::SessionEnded,
        ],
    );

    assert_chat_history(
        &scenario.client_calls.lock().unwrap().clone()[2].1,
        &[
            ChatMessage::System {
                content: SYSTEM_PROMPT.to_string(),
            },
            ChatMessage::User {
                content: "Hello".to_string(),
            },
            ChatMessage::Assistant {
                message: AssistantMessage::Content("Hello human".to_string()),
            },
            ChatMessage::User {
                content: "How are you?".to_string(),
            },
            ChatMessage::Assistant {
                message: AssistantMessage::Content("Oh, I feel great. What about you?".to_string()),
            },
            ChatMessage::User {
                content: "I'm good, thank you!".to_string(),
            },
        ],
    );
}

#[tokio::test]
async fn chat_session_handles_multiple_assistant_messages_per_turn() {
    let scenario = ScenarioBuilder::new()
        .inputs(["What is 1+1? Think about it real hard"])
        .then_message("Okay lets see. The user is asking me what 1+1 is. I need to think about it real hard", false)
        .then_message("I think the answer is 2. I'm not sure if I'm right, as one sand pile plus one sand pile is one big sand pile", false)
        .then_message("I'm going to return the answer 2", false)
        .then_message("1+1 is 2", true)
        .run()
        .await;

    assert_events(
        &scenario.events,
        &[
            Event::SessionStarted,
            Event::ReceivedUserMessage { message: "What is 1+1? Think about it real hard".to_string() },
            Event::AwaitingAssistantResponse,
            Event::ReceivedAssistantMessage { message: "Okay lets see. The user is asking me what 1+1 is. I need to think about it real hard".to_string() },
            Event::AwaitingAssistantResponse,
            Event::ReceivedAssistantMessage { message: "I think the answer is 2. I'm not sure if I'm right, as one sand pile plus one sand pile is one big sand pile".to_string() },
            Event::AwaitingAssistantResponse,
            Event::ReceivedAssistantMessage { message: "I'm going to return the answer 2".to_string() },
            Event::AwaitingAssistantResponse,
            Event::ReceivedAssistantMessage { message: "1+1 is 2".to_string() },
            Event::SessionEnded,
        ],
    );

    assert_chat_history(
        &scenario.client_calls.lock().unwrap().clone()[3].1,
        &[
            ChatMessage::System { content: SYSTEM_PROMPT.to_string() },
            ChatMessage::User { content: "What is 1+1? Think about it real hard".to_string() },
            ChatMessage::Assistant { message: AssistantMessage::Content("Okay lets see. The user is asking me what 1+1 is. I need to think about it real hard".to_string()) },
            ChatMessage::Assistant { message: AssistantMessage::Content("I think the answer is 2. I'm not sure if I'm right, as one sand pile plus one sand pile is one big sand pile".to_string()) },
            ChatMessage::Assistant { message: AssistantMessage::Content("I'm going to return the answer 2".to_string()) },
        ],
    );
}

#[tokio::test]
async fn chat_session_handles_tool_call() {
    let scenario = ScenarioBuilder::new()
        .inputs(["What is the weather in Tokyo?"])
        .add_mock_tool(
            "get_weather",
            "Get the weather in a city",
            Parameters {
                param_type: "object".to_string(),
                properties: HashMap::new(),
                required: vec!["city".to_string()],
                additional_properties: false,
            },
            true,
            false,
            "AI wants to check the weather in Tokyo. Do you approve?",
            "The weather in Tokyo is sunny",
        )
        .then_tool_call(
            "1",
            "get_weather",
            HashMap::from([("city".to_string(), "Tokyo".to_string())]),
        )
        .then_message("The weather in Tokyo is sunny", true)
        .run()
        .await;

    assert_events(
        &scenario.events,
        &[
            Event::SessionStarted,
            Event::ReceivedUserMessage {
                message: "What is the weather in Tokyo?".to_string(),
            },
            Event::AwaitingAssistantResponse,
            Event::ReceivedToolCall {
                tool_name: "get_weather".to_string(),
                parameters: HashMap::from([("city".to_string(), "Tokyo".to_string())]),
            },
            Event::ReceivedToolResponse {
                tool_name: "get_weather".to_string(),
                response: "The weather in Tokyo is sunny".to_string(),
                parameters: HashMap::from([("city".to_string(), "Tokyo".to_string())]),
            },
            Event::AwaitingAssistantResponse,
            Event::ReceivedAssistantMessage {
                message: "The weather in Tokyo is sunny".to_string(),
            },
            Event::SessionEnded,
        ],
    );

    assert_chat_history(
        &scenario.client_calls.lock().unwrap().clone()[1].1,
        &[
            ChatMessage::System {
                content: SYSTEM_PROMPT.to_string(),
            },
            ChatMessage::User {
                content: "What is the weather in Tokyo?".to_string(),
            },
            ChatMessage::Assistant {
                message: AssistantMessage::ToolCalls(vec![ToolCall {
                    id: "1".to_string(),
                    name: "get_weather".to_string(),
                    arguments: HashMap::from([("city".to_string(), "Tokyo".to_string())]),
                }]),
            },
            ChatMessage::Tool {
                content: "The weather in Tokyo is sunny".to_string(),
                tool_call_id: "1".to_string(),
                tool_name: "get_weather".to_string(),
            },
        ],
    );

    assert_tool_calls(
        &scenario.tool_calls,
        &(
            "get_weather".to_string(),
            HashMap::from([("city".to_string(), "Tokyo".to_string())]),
        ),
    );
}

#[tokio::test]
async fn chat_session_handles_tool_call_with_approval() {
    let events = Arc::new(Mutex::new(vec![]));
    let event_handler = MockEventHandler::new(
        events.clone(),
        vec!["Execute a command in my terminal: echo 'Hello, world!'".to_string()],
        vec!["approved".to_string()],
    );

    let client_calls = Arc::new(Mutex::new(vec![]));
    let chat_client = MockChatClient::new(
        vec![
            Ok(ChatResult::ToolCalls(vec![ToolCall {
                id: "1".to_string(),
                name: "execute_command".to_string(),
                arguments: HashMap::from([(
                    "command".to_string(),
                    "echo 'Hello, world!'".to_string(),
                )]),
            }])),
            Ok(ChatResult::Message {
                content: "The command was executed successfully".to_string(),
                turn_over: true,
            }),
        ],
        client_calls.clone(),
    );

    let tool_calls = Arc::new(Mutex::new(vec![]));
    let tool_registry = MockToolRegistry::new(
        vec![Box::new(MockTool::new(
            "execute_command".to_string(),
            "Execute a command in the terminal".to_string(),
            Parameters {
                param_type: "object".to_string(),
                properties: HashMap::new(),
                required: vec!["command".to_string()],
                additional_properties: false,
            },
            true,
            true,
            "AI wants to execute a command in the terminal. Do you approve?".to_string(),
            "Hello, world!".to_string(),
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

    let tool_calls = tool_calls.lock().unwrap().clone();
    assert_eq!(tool_calls.len(), 1);
    assert_eq!(
        tool_calls[0].clone(),
        (
            "execute_command".to_string(),
            HashMap::from([("command".to_string(), "echo 'Hello, world!'".to_string())]),
        )
    );

    let events = events.lock().unwrap().clone();
    assert_eq!(events.len(), 8);
    assert_eq!(events[0], Event::SessionStarted);
    assert_eq!(
        events[1],
        Event::ReceivedUserMessage {
            message: "Execute a command in my terminal: echo 'Hello, world!'".to_string()
        }
    );
    assert_eq!(events[2], Event::AwaitingAssistantResponse);
    assert_eq!(
        events[3],
        Event::ReceivedToolCall {
            tool_name: "execute_command".to_string(),
            parameters: HashMap::from([(
                "command".to_string(),
                "echo 'Hello, world!'".to_string()
            )])
        }
    );
    assert_eq!(
        events[4],
        Event::ReceivedToolResponse {
            tool_name: "execute_command".to_string(),
            response: "Hello, world!".to_string(),
            parameters: HashMap::from([(
                "command".to_string(),
                "echo 'Hello, world!'".to_string()
            )])
        }
    );
    assert_eq!(events[5], Event::AwaitingAssistantResponse);
    assert_eq!(
        events[6],
        Event::ReceivedAssistantMessage {
            message: "The command was executed successfully".to_string()
        }
    );
    assert_eq!(events[7], Event::SessionEnded);

    let client_calls = client_calls.lock().unwrap().clone();
    assert_eq!(client_calls.len(), 2);
    // Call 1
    let (model, chat_history, tools) = &client_calls[0];
    assert_eq!(model, &Model::OpenAi(OpenAiModel::Gpt4oMini));
    assert_eq!(tools.len(), 1);
    assert_eq!(chat_history.len(), 2);
    if let ChatMessage::User { content } = &chat_history[1] {
        assert_eq!(
            content,
            "Execute a command in my terminal: echo 'Hello, world!'"
        );
    } else {
        panic!("expected user message");
    }
    // Call 2
    let (model, chat_history, tools) = &client_calls[1];
    assert_eq!(model, &Model::OpenAi(OpenAiModel::Gpt4oMini));
    assert_eq!(tools.len(), 1);
    assert_eq!(chat_history.len(), 4);
    if let ChatMessage::Assistant {
        message: AssistantMessage::ToolCalls(tool_calls),
    } = &chat_history[2]
    {
        assert_eq!(tool_calls.len(), 1);
    } else {
        panic!("expected assistant tool calls");
    }
    if let ChatMessage::Tool {
        content,
        tool_call_id,
        tool_name,
    } = &chat_history[3]
    {
        assert_eq!(content, "Hello, world!");
        assert_eq!(tool_call_id, "1");
        assert_eq!(tool_name, "execute_command");
    } else {
        panic!("expected tool message");
    }
}

#[tokio::test]
async fn chat_session_handles_tool_call_with_approval_and_denied() {
    let events = Arc::new(Mutex::new(vec![]));
    let event_handler = MockEventHandler::new(
        events.clone(),
        vec!["Execute a command in my terminal: echo 'Hello, world!'".to_string()],
        vec!["denied".to_string()],
    );

    let client_calls = Arc::new(Mutex::new(vec![]));
    let chat_client = MockChatClient::new(
        vec![
            Ok(ChatResult::ToolCalls(vec![ToolCall {
                id: "1".to_string(),
                name: "execute_command".to_string(),
                arguments: HashMap::from([(
                    "command".to_string(),
                    "echo 'Hello, world!'".to_string(),
                )]),
            }])),
            Ok(ChatResult::Message {
                content: "The command was executed successfully".to_string(),
                turn_over: true,
            }),
        ],
        client_calls.clone(),
    );

    let tool_calls = Arc::new(Mutex::new(vec![]));
    let tool_registry = MockToolRegistry::new(
        vec![Box::new(MockTool::new(
            "execute_command".to_string(),
            "Execute a command in the terminal".to_string(),
            Parameters {
                param_type: "object".to_string(),
                properties: HashMap::new(),
                required: vec!["command".to_string()],
                additional_properties: false,
            },
            true,
            true,
            "AI wants to execute a command in the terminal. Do you approve?".to_string(),
            "Hello, world!".to_string(),
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

    let tool_calls = tool_calls.lock().unwrap().clone();
    assert_eq!(tool_calls.len(), 0);

    let events = events.lock().unwrap().clone();
    assert_eq!(events.len(), 8);
    assert_eq!(events[0], Event::SessionStarted);
    assert_eq!(
        events[1],
        Event::ReceivedUserMessage {
            message: "Execute a command in my terminal: echo 'Hello, world!'".to_string()
        }
    );
    assert_eq!(events[2], Event::AwaitingAssistantResponse);
    assert_eq!(
        events[3],
        Event::ReceivedToolCall {
            tool_name: "execute_command".to_string(),
            parameters: HashMap::from([(
                "command".to_string(),
                "echo 'Hello, world!'".to_string()
            )])
        }
    );
    assert_eq!(
        events[4],
        Event::ReceivedToolResponse {
            tool_name: "execute_command".to_string(),
            response: "Operation cancelled by user: execute_command with parameters {\"command\": \"echo 'Hello, world!'\"}".to_string(),
            parameters: HashMap::from([(
                "command".to_string(),
                "echo 'Hello, world!'".to_string()
            )])
        }
    );
    assert_eq!(events[5], Event::AwaitingAssistantResponse);
    assert_eq!(
        events[6],
        Event::ReceivedAssistantMessage {
            message: "The command was executed successfully".to_string()
        }
    );
    assert_eq!(events[7], Event::SessionEnded);

    let client_calls = client_calls.lock().unwrap().clone();
    assert_eq!(client_calls.len(), 2);
    // Call 1
    let (model, chat_history, tools) = &client_calls[0];
    assert_eq!(model, &Model::OpenAi(OpenAiModel::Gpt4oMini));
    assert_eq!(tools.len(), 1);
    assert_eq!(chat_history.len(), 2);
    if let ChatMessage::User { content } = &chat_history[1] {
        assert_eq!(
            content,
            "Execute a command in my terminal: echo 'Hello, world!'"
        );
    } else {
        panic!("expected user message");
    }
    // Call 2
    let (model, chat_history, tools) = &client_calls[1];
    assert_eq!(model, &Model::OpenAi(OpenAiModel::Gpt4oMini));
    assert_eq!(tools.len(), 1);
    assert_eq!(chat_history.len(), 4);
    if let ChatMessage::Assistant {
        message: AssistantMessage::ToolCalls(tool_calls),
    } = &chat_history[2]
    {
        assert_eq!(tool_calls.len(), 1);
    } else {
        panic!("expected assistant tool calls");
    }
    if let ChatMessage::Tool {
        content,
        tool_call_id,
        tool_name,
    } = &chat_history[3]
    {
        assert_eq!(
            content,
            "Operation cancelled by user: execute_command with parameters {\"command\": \"echo 'Hello, world!'\"}"
        );
        assert_eq!(tool_call_id, "1");
        assert_eq!(tool_name, "execute_command");
    } else {
        panic!("expected tool message");
    }
}
