mod helpers;

use code_g::client::providers::openai::client::OpenAIClient;
use code_g::session::event::Event;
use code_g::session::session::ChatSession;
use code_g::session::system_prompt::SystemPromptConfig;
use code_g::tools::registry::Registry;
use helpers::mocks::event_handler::MockEventHandler;
use std::sync::{Arc, Mutex};

/// These are live end-to-end tests that exercise a real chat session against the
/// OpenAI Chat Completions API using the actual tool registry and event handler.
/// They are ignored by default. To run them locally on Windows PowerShell:
///
///   $env:OPENAI_API_KEY = "sk-..."
///   cargo test --test chat_session_live -- --ignored
///
/// Note: These tests will create and edit files in the working directory when the
/// assistant decides to use tools. They are meant for local, manual verification.

#[tokio::test]
#[ignore]
async fn chat_session_live_handles_basic_message() {
    let api_key = std::env::var("OPENAI_API_KEY")
        .expect("Set OPENAI_API_KEY to run this ignored integration test");

    let client = Box::new(OpenAIClient::new(api_key));
    let tools = Box::new(Registry::read_only_tools());

    let events = Arc::new(Mutex::new(vec![]));
    let event_handler = Box::new(MockEventHandler::new(
        events.clone(),
        vec!["Hello".to_string()],
        vec![],
    ));

    let mut session = ChatSession::new(client, tools, event_handler, SystemPromptConfig::Default);

    let _ = session.run().await;

    let events_vec = events.lock().unwrap().clone();
    assert!(
        !events_vec.is_empty(),
        "expected some events to be recorded"
    );
    assert!(matches!(events_vec.first(), Some(Event::SessionStarted)));
    assert!(matches!(events_vec.last(), Some(Event::SessionEnded)));

    // Ensure our user message is captured
    assert!(events_vec.iter().any(|e| matches!(
        e,
        Event::ReceivedUserMessage { message } if message == "Hello"
    )));

    // Ensure we received at least one assistant message with non-empty content
    let assistant_messages: Vec<&String> = events_vec
        .iter()
        .filter_map(|e| match e {
            Event::ReceivedAssistantMessage { message } => Some(message),
            _ => None,
        })
        .collect();
    assert!(
        assistant_messages.iter().any(|m| !m.trim().is_empty()),
        "expected at least one non-empty assistant message"
    );
}

#[tokio::test]
#[ignore]
async fn chat_session_live_handles_full_workflow_with_tools() {
    use std::fs;

    let api_key = std::env::var("OPENAI_API_KEY")
        .expect("Set OPENAI_API_KEY to run this ignored integration test");

    let client = Box::new(OpenAIClient::new(api_key));
    let tools = Box::new(Registry::all_tools());

    let events = Arc::new(Mutex::new(vec![]));
    // Provide sufficient approvals for any write/edit/execute operations
    let approvals = vec!["approved".to_string(); 10];
    let inputs = vec![
        "I need a function to calculate the factorial of a number. Please implement it in a new file called math_utils.rs".to_string(),
        "Great! Can you add input validation to make sure the number is non-negative?".to_string(),
    ];
    let event_handler = Box::new(MockEventHandler::new(events.clone(), inputs, approvals));

    let mut session = ChatSession::new(client, tools, event_handler, SystemPromptConfig::Default);

    let _ = session.run().await;

    let events_vec = events.lock().unwrap().clone();
    assert!(matches!(events_vec.first(), Some(Event::SessionStarted)));
    assert!(matches!(events_vec.last(), Some(Event::SessionEnded)));

    // Expect at least one tool call was made during the workflow
    let tool_calls_count = events_vec
        .iter()
        .filter(|e| matches!(e, Event::ReceivedToolCall { .. }))
        .count();
    assert!(tool_calls_count > 0, "expected at least one tool call");

    // If a write_file tool succeeded, verify the file exists
    let maybe_written_path = events_vec.iter().find_map(|e| match e {
        Event::ReceivedToolResponse {
            tool_name,
            response,
            ..
        } if tool_name == "write_file" => {
            // Response format from WriteFile: "File '<path>' written successfully"
            let needle = "File '";
            if let Some(start) = response.find(needle) {
                let rest = &response[start + needle.len()..];
                if let Some(end) = rest.find("' written successfully") {
                    let path = &rest[..end];
                    return Some(path.to_string());
                }
            }
            None
        }
        _ => None,
    });

    if let Some(path) = maybe_written_path {
        assert!(
            fs::metadata(&path).is_ok(),
            "expected written file to exist: {}",
            path
        );

        // Clean up the file created during the test
        let _ = fs::remove_file(&path);
    }

    // Ensure we received at least one assistant message
    assert!(
        events_vec
            .iter()
            .any(|e| matches!(e, Event::ReceivedAssistantMessage { .. }))
    );
}
