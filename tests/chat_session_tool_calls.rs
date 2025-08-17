mod helpers;

use code_g::client::model::{AssistantMessage, ChatMessage, Parameters, ToolCall};
use code_g::session::event::Event;
use code_g::session::system_prompt::SYSTEM_PROMPT;
use helpers::assertions::{assert_chat_history, assert_events, assert_tool_calls};
use helpers::scenario::ScenarioBuilder;
use std::collections::HashMap;

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
        &scenario.last_client_call().1,
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
        &[(
            "get_weather".to_string(),
            HashMap::from([("city".to_string(), "Tokyo".to_string())]),
        )],
    );
}