mod helpers;

use code_g::client::models::{AssistantMessage, ChatMessage};
use code_g::session::event::Event;
use code_g::session::system_prompt::SYSTEM_PROMPT;
use helpers::assertions::{assert_chat_history, assert_events, assert_tool_calls};
use helpers::scenario::ScenarioBuilder;

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
        &scenario.last_client_call().1,
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

    assert_tool_calls(&scenario.tool_calls, &[]);
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
        &scenario.last_client_call().1,
        &[
            ChatMessage::System { content: SYSTEM_PROMPT.to_string() },
            ChatMessage::User { content: "What is 1+1? Think about it real hard".to_string() },
            ChatMessage::Assistant { message: AssistantMessage::Content("Okay lets see. The user is asking me what 1+1 is. I need to think about it real hard".to_string()) },
            ChatMessage::Assistant { message: AssistantMessage::Content("I think the answer is 2. I'm not sure if I'm right, as one sand pile plus one sand pile is one big sand pile".to_string()) },
            ChatMessage::Assistant { message: AssistantMessage::Content("I'm going to return the answer 2".to_string()) },
        ],
    );

    assert_tool_calls(&scenario.tool_calls, &[]);
}

