mod helpers;

use code_g::client::model::ChatMessage;
use code_g::session::event::Event;
use code_g::session::system_prompt::{SYSTEM_PROMPT, SystemPromptConfig};
use helpers::assertions::{assert_chat_history, assert_events, assert_tool_calls};
use helpers::scenario::ScenarioBuilder;

#[tokio::test]
async fn system_prompt_default_is_included() {
    let scenario = ScenarioBuilder::new()
        .with_system_prompt_config(SystemPromptConfig::Default)
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
        &scenario.last_client_call().1,
        &[
            ChatMessage::System {
                content: SYSTEM_PROMPT.to_string(),
            },
            ChatMessage::User {
                content: "Hello".to_string(),
            },
        ],
    );

    assert_tool_calls(&scenario.tool_calls, &[]);
}

#[tokio::test]
async fn system_prompt_none_is_excluded() {
    let scenario = ScenarioBuilder::new()
        .with_system_prompt_config(SystemPromptConfig::None)
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
        &scenario.last_client_call().1,
        &[ChatMessage::User {
            content: "Hello".to_string(),
        }],
    );

    assert_tool_calls(&scenario.tool_calls, &[]);
}

#[tokio::test]
async fn system_prompt_custom_is_used() {
    let custom = "You are a custom assistant.".to_string();
    let scenario = ScenarioBuilder::new()
        .with_system_prompt_config(SystemPromptConfig::Custom(custom.clone()))
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
        &scenario.last_client_call().1,
        &[
            ChatMessage::System { content: custom },
            ChatMessage::User {
                content: "Hello".to_string(),
            },
        ],
    );

    assert_tool_calls(&scenario.tool_calls, &[]);
}
