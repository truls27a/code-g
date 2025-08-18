mod helpers;

use code_g::client::model::{AssistantMessage, ChatMessage, Parameters, ToolCall};
use code_g::session::event::Event;
use code_g::session::system_prompt::SYSTEM_PROMPT;
use helpers::assertions::{assert_chat_history, assert_events, assert_tool_calls};
use helpers::scenario::ScenarioBuilder;
use std::collections::HashMap;

#[tokio::test]
async fn chat_session_handles_tool_call_with_approval() {
    let scenario = ScenarioBuilder::new()
        .inputs(["Execute a command in my terminal: echo 'Hello, world!'"])
        .approvals(["approved"])
        .add_mock_tool(
            "execute_command",
            "Execute a command in the terminal",
            Parameters {
                param_type: "object".to_string(),
                properties: HashMap::new(),
                required: vec!["command".to_string()],
                additional_properties: false,
            },
            true,
            true,
            "AI wants to execute a command in the terminal. Do you approve?",
            "Execute command {} was declined by user",
            "Hello, world!",
        )
        .then_tool_call(
            "1",
            "execute_command",
            HashMap::from([("command".to_string(), "echo 'Hello, world!'".to_string())]),
        )
        .then_message("The command was executed successfully", true)
        .run()
        .await;

    assert_events(
        &scenario.events,
        &[
            Event::SessionStarted,
            Event::ReceivedUserMessage {
                message: "Execute a command in my terminal: echo 'Hello, world!'".to_string(),
            },
            Event::AwaitingAssistantResponse,
            Event::ReceivedToolCall {
                tool_name: "execute_command".to_string(),
                parameters: HashMap::from([(
                    "command".to_string(),
                    "echo 'Hello, world!'".to_string(),
                )]),
            },
            Event::ReceivedToolResponse {
                tool_name: "execute_command".to_string(),
                response: "Hello, world!".to_string(),
                parameters: HashMap::from([(
                    "command".to_string(),
                    "echo 'Hello, world!'".to_string(),
                )]),
                approved: true,
            },
            Event::AwaitingAssistantResponse,
            Event::ReceivedAssistantMessage {
                message: "The command was executed successfully".to_string(),
            },
            Event::SessionEnded,
        ],
    );

    assert_tool_calls(
        &scenario.tool_calls,
        &[(
            "execute_command".to_string(),
            HashMap::from([("command".to_string(), "echo 'Hello, world!'".to_string())]),
        )],
    );

    assert_chat_history(
        &scenario.last_client_call().1,
        &[
            ChatMessage::System {
                content: SYSTEM_PROMPT.to_string(),
            },
            ChatMessage::User {
                content: "Execute a command in my terminal: echo 'Hello, world!'".to_string(),
            },
            ChatMessage::Assistant {
                message: AssistantMessage::ToolCalls(vec![ToolCall {
                    id: "1".to_string(),
                    name: "execute_command".to_string(),
                    arguments: HashMap::from([(
                        "command".to_string(),
                        "echo 'Hello, world!'".to_string(),
                    )]),
                }]),
            },
            ChatMessage::Tool {
                content: "Hello, world!".to_string(),
                tool_call_id: "1".to_string(),
                tool_name: "execute_command".to_string(),
            },
        ],
    );
}

#[tokio::test]
async fn chat_session_handles_tool_call_with_approval_and_denied() {
    let scenario = ScenarioBuilder::new()
        .inputs(["Execute a command in my terminal: echo 'Hello, world!'"])
        .approvals(["denied"])
        .add_mock_tool(
            "execute_command",
            "Execute a command in the terminal",
            Parameters {
                param_type: "object".to_string(),
                properties: HashMap::new(),
                required: vec!["command".to_string()],
                additional_properties: false,
            },
            true,
            true,
            "AI wants to execute a command in the terminal. Do you approve?",
            "Execute command {} was declined by user",
            "Hello, world!",
        )
        .then_tool_call(
            "1",
            "execute_command",
            HashMap::from([("command".to_string(), "echo 'Hello, world!'".to_string())]),
        )
        .then_message("The command was executed successfully", true)
        .run()
        .await;

    assert_events(
        &scenario.events,
        &[
            Event::SessionStarted,
            Event::ReceivedUserMessage {
                message: "Execute a command in my terminal: echo 'Hello, world!'".to_string(),
            },
            Event::AwaitingAssistantResponse,
            Event::ReceivedToolCall {
                tool_name: "execute_command".to_string(),
                parameters: HashMap::from([(
                    "command".to_string(),
                    "echo 'Hello, world!'".to_string(),
                )]),
            },
            Event::ReceivedToolResponse {
                tool_name: "execute_command".to_string(),
                response: "Operation cancelled by user: execute_command with parameters {\"command\": \"echo 'Hello, world!'\"}".to_string(),
                parameters: HashMap::from([(
                    "command".to_string(),
                    "echo 'Hello, world!'".to_string(),
                )]),
                approved: false,
            },
            Event::AwaitingAssistantResponse,
            Event::ReceivedAssistantMessage {
                message: "The command was executed successfully".to_string(),
            },
            Event::SessionEnded,
        ],
    );

    // No tool should be called when approval is denied
    assert_tool_calls(&scenario.tool_calls, &[]);

    assert_chat_history(
        &scenario.last_client_call().1,
        &[
            ChatMessage::System {
                content: SYSTEM_PROMPT.to_string(),
            },
            ChatMessage::User {
                content: "Execute a command in my terminal: echo 'Hello, world!'".to_string(),
            },
            ChatMessage::Assistant {
                message: AssistantMessage::ToolCalls(vec![ToolCall {
                    id: "1".to_string(),
                    name: "execute_command".to_string(),
                    arguments: HashMap::from([(
                        "command".to_string(),
                        "echo 'Hello, world!'".to_string(),
                    )]),
                }]),
            },
            ChatMessage::Tool {
                content: "Operation cancelled by user: execute_command with parameters {\"command\": \"echo 'Hello, world!'\"}".to_string(),
                tool_call_id: "1".to_string(),
                tool_name: "execute_command".to_string(),
            },
        ],
    );
}

#[tokio::test]
async fn chat_session_handles_tool_call_with_approval_and_invalid_approval() {
    let scenario = ScenarioBuilder::new()
        .inputs(["Execute a command in my terminal: echo 'Hello, world!'"])
        .approvals([":)".to_string()])
        .add_mock_tool(
            "execute_command",
            "Execute a command in the terminal",
            Parameters {
                param_type: "object".to_string(),
                properties: HashMap::new(),
                required: vec!["command".to_string()],
                additional_properties: false,
            },
            true,
            true,
            "AI wants to execute a command in the terminal. Do you approve?",
            "Execute command {} was declined by user",
            "Hello, world!",
        )
        .then_tool_call(
            "1",
            "execute_command",
            HashMap::from([("command".to_string(), "echo 'Hello, world!'".to_string())]),
        )
        .then_message("The command was executed successfully", true)
        .run()
        .await;

    assert_events(
        &scenario.events,
        &[
            Event::SessionStarted,
            Event::ReceivedUserMessage {
                message: "Execute a command in my terminal: echo 'Hello, world!'".to_string(),
            },
            Event::AwaitingAssistantResponse,
            Event::ReceivedToolCall {
                tool_name: "execute_command".to_string(),
                parameters: HashMap::from([(
                    "command".to_string(),
                    "echo 'Hello, world!'".to_string(),
                )]),
            },
            Event::ReceivedToolResponse {
                tool_name: "execute_command".to_string(),
                response: "Operation cancelled by user: execute_command with parameters {\"command\": \"echo 'Hello, world!'\"}".to_string(),
                parameters: HashMap::from([(
                    "command".to_string(),
                    "echo 'Hello, world!'".to_string(),
                )]),
                approved: false,
            },
            Event::AwaitingAssistantResponse,
            Event::ReceivedAssistantMessage {
                message: "The command was executed successfully".to_string(),
            },
            Event::SessionEnded,
        ],
    );

    // No tool should be called when approval is denied
    assert_tool_calls(&scenario.tool_calls, &[]);

    assert_chat_history(
        &scenario.last_client_call().1,
        &[
            ChatMessage::System {
                content: SYSTEM_PROMPT.to_string(),
            },
            ChatMessage::User {
                content: "Execute a command in my terminal: echo 'Hello, world!'".to_string(),
            },
            ChatMessage::Assistant {
                message: AssistantMessage::ToolCalls(vec![ToolCall {
                    id: "1".to_string(),
                    name: "execute_command".to_string(),
                    arguments: HashMap::from([(
                        "command".to_string(),
                        "echo 'Hello, world!'".to_string(),
                    )]),
                }]),
            },
            ChatMessage::Tool {
                content: "Operation cancelled by user: execute_command with parameters {\"command\": \"echo 'Hello, world!'\"}".to_string(),
                tool_call_id: "1".to_string(),
                tool_name: "execute_command".to_string(),
            },
        ],
    );
}
