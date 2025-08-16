use code_g::client::model::{AssistantMessage, ChatMessage, Model};
use code_g::session::event::Event;
use std::sync::{Arc, Mutex};

/// Convert events into a compact, human-readable transcript for assertions.
pub fn events_to_transcript(events: &[Event]) -> Vec<String> {
    events
        .iter()
        .map(|e| match e {
            Event::SessionStarted => "SessionStarted".to_string(),
            Event::SessionEnded => "SessionEnded".to_string(),
            Event::AwaitingAssistantResponse => "AwaitingAssistantResponse".to_string(),
            Event::ReceivedUserMessage { message } => {
                format!("User: {}", message)
            }
            Event::ReceivedAssistantMessage { message } => {
                format!("Assistant: {}", message)
            }
            Event::ReceivedToolCall {
                tool_name,
                parameters,
            } => {
                format!("ToolCall: {} {:?}", tool_name, parameters)
            }
            Event::ReceivedToolResponse {
                tool_name,
                response,
                parameters,
            } => {
                format!(
                    "ToolResponse: {} {:?} -> {}",
                    tool_name, parameters, response
                )
            }
        })
        .collect()
}

pub fn assert_events_transcript(actual_events: &[Event], expected_lines: &[&str]) {
    let actual = events_to_transcript(actual_events);
    let expected: Vec<String> = expected_lines.iter().map(|s| s.to_string()).collect();
    assert_eq!(actual, expected, "event transcript mismatch");
}

pub fn assert_client_calls_len(
    client_calls: &Arc<Mutex<Vec<(Model, Vec<ChatMessage>, Vec<code_g::client::model::Tool>)>>>,
    expected_len: usize,
) {
    let calls = client_calls.lock().unwrap().clone();
    assert_eq!(
        calls.len(),
        expected_len,
        "unexpected number of client calls"
    );
}

pub fn assert_chat_history_at<F>(
    client_calls: &Arc<Mutex<Vec<(Model, Vec<ChatMessage>, Vec<code_g::client::model::Tool>)>>>,
    call_index: usize,
    assert_fn: F,
) where
    F: FnOnce(&[ChatMessage]),
{
    let calls = client_calls.lock().unwrap().clone();
    let (_, chat_history, _) = &calls[call_index];
    assert_fn(chat_history);
}

/// Handy helper for matching assistant content in history.
pub fn assistant_content_at(chat_history: &[ChatMessage], index: usize) -> &str {
    match &chat_history[index] {
        ChatMessage::Assistant {
            message: AssistantMessage::Content(c),
        } => c.as_str(),
        other => panic!(
            "expected assistant content at {}, found: {:?}",
            index, other
        ),
    }
}
