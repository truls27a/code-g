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
                approved: true,
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

#[tokio::test]
async fn chat_session_handles_multiple_tool_calls() {
    let scenario = ScenarioBuilder::new()
        .inputs(["Fix all errors in the main.rs file"])
        .add_mock_tool(
            "read_file",
            "Read a file",
            Parameters {
                param_type: "object".to_string(),
                properties: HashMap::new(),
                required: vec!["file_path".to_string()],
                additional_properties: false,
            },
            true,
            false,
            "AI wants to read a file. Do you approve?",
            "assert_eq!(1, 2);",
        )
        .add_mock_tool(
            "write_file",
            "Write to a file",
            Parameters {
                param_type: "object".to_string(),
                properties: HashMap::new(),
                required: vec!["file_path".to_string(), "content".to_string()],
                additional_properties: false,
            },
            true,
            false,
            "AI wants to write to a file. Do you approve?",
            "File written successfully",
        )
        .then_message("Sure, let me read the file and then fix the errors", false)
        .then_tool_call(
            "1",
            "read_file",
            HashMap::from([("file_path".to_string(), "main.rs".to_string())]),
        )
        .then_message("The file contains the following text: 'assert_eq!(1, 2). This is an error, it should be assert_eq!(1, 1);'", false)
        .then_tool_call(
            "2",
            "write_file",
            HashMap::from([("file_path".to_string(), "main.rs".to_string()), ("content".to_string(), "assert_eq!(1, 1);".to_string())]),
        )
        .then_message("The file was written successfully and the errors are fixed", true)
        .run()
        .await;

    assert_events(
        &scenario.events,
        &[
            Event::SessionStarted,
            Event::ReceivedUserMessage {
                message: "Fix all errors in the main.rs file".to_string(),
            },
            Event::AwaitingAssistantResponse,
            Event::ReceivedAssistantMessage {
                message: "Sure, let me read the file and then fix the errors".to_string(),
            },
            Event::AwaitingAssistantResponse,
            Event::ReceivedToolCall {
                tool_name: "read_file".to_string(),
                parameters: HashMap::from([("file_path".to_string(), "main.rs".to_string())]),
            },
            Event::ReceivedToolResponse {
                tool_name: "read_file".to_string(),
                response: "assert_eq!(1, 2);".to_string(),
                parameters: HashMap::from([("file_path".to_string(), "main.rs".to_string())]),
                approved: true,
            },
            Event::AwaitingAssistantResponse,
            Event::ReceivedAssistantMessage {
                message: "The file contains the following text: 'assert_eq!(1, 2). This is an error, it should be assert_eq!(1, 1);'".to_string(),
            },
            Event::AwaitingAssistantResponse,
            Event::ReceivedToolCall {
                tool_name: "write_file".to_string(),
                parameters: HashMap::from([("file_path".to_string(), "main.rs".to_string()), ("content".to_string(), "assert_eq!(1, 1);".to_string())]),
            },
            Event::ReceivedToolResponse {
                tool_name: "write_file".to_string(),
                response: "File written successfully".to_string(),
                parameters: HashMap::from([("file_path".to_string(), "main.rs".to_string()), ("content".to_string(), "assert_eq!(1, 1);".to_string())]),
                approved: true,
            },
            Event::AwaitingAssistantResponse,
            Event::ReceivedAssistantMessage {
                message: "The file was written successfully and the errors are fixed".to_string(),
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
                content: "Fix all errors in the main.rs file".to_string(),
            },
            ChatMessage::Assistant {
                message: AssistantMessage::Content("Sure, let me read the file and then fix the errors".to_string()),
            },
            ChatMessage::Assistant {
                message: AssistantMessage::ToolCalls(vec![ToolCall {
                    id: "1".to_string(),
                    name: "read_file".to_string(),
                    arguments: HashMap::from([("file_path".to_string(), "main.rs".to_string())]),
                }]),
            },
            ChatMessage::Tool {
                content: "assert_eq!(1, 2);".to_string(),
                tool_call_id: "1".to_string(),
                tool_name: "read_file".to_string(),
            },
            ChatMessage::Assistant {
                message: AssistantMessage::Content("The file contains the following text: 'assert_eq!(1, 2). This is an error, it should be assert_eq!(1, 1);'".to_string()),
            },
            ChatMessage::Assistant {
                message: AssistantMessage::ToolCalls(vec![ToolCall {
                    id: "2".to_string(),
                    name: "write_file".to_string(),
                    arguments: HashMap::from([("file_path".to_string(), "main.rs".to_string()), ("content".to_string(), "assert_eq!(1, 1);".to_string())]),
                }]),
            },
            ChatMessage::Tool {
                content: "File written successfully".to_string(),
                tool_call_id: "2".to_string(),
                tool_name: "write_file".to_string(),
            },
        ],
    );

    assert_tool_calls(
        &scenario.tool_calls,
        &[(
            "read_file".to_string(),
            HashMap::from([("file_path".to_string(), "main.rs".to_string())]),
        ), (
            "write_file".to_string(),
            HashMap::from([("file_path".to_string(), "main.rs".to_string()), ("content".to_string(), "assert_eq!(1, 1);".to_string())]),
        )],
    );
}

#[tokio::test]
async fn chat_session_handles_tool_call_when_no_tools_are_available() {
    let scenario = ScenarioBuilder::new()
        .inputs(["What is the weather in Tokyo?"])
        .then_tool_call(
            "1",
            "get_weather",
            HashMap::from([("city".to_string(), "Tokyo".to_string())]),
        )
        .then_message("I dont know bro", true)
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
                response: "Tool get_weather not found".to_string(),
                parameters: HashMap::from([("city".to_string(), "Tokyo".to_string())]),
                approved: true,
            },
            Event::AwaitingAssistantResponse,
            Event::ReceivedAssistantMessage {
                message: "I dont know bro".to_string(),
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
                content: "Tool get_weather not found".to_string(),
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

#[tokio::test]
async fn chat_session_handles_invalid_tool_call_parameters() {
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
            HashMap::from([("coordinates".to_string(), "123, 456".to_string())]),
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
                parameters: HashMap::from([("coordinates".to_string(), "123, 456".to_string())]),
            },
            Event::ReceivedToolResponse {
                tool_name: "get_weather".to_string(),
                response: "The weather in Tokyo is sunny".to_string(), // Mock tool doesnt validate arguments
                parameters: HashMap::from([("coordinates".to_string(), "123, 456".to_string())]),
                approved: true,
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
                    arguments: HashMap::from([("coordinates".to_string(), "123, 456".to_string())]),
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
            HashMap::from([("coordinates".to_string(), "123, 456".to_string())]),
        )],
    );
}