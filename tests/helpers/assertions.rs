use code_g::client::model::{ChatMessage, Model};
use code_g::session::event::Event;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

pub fn assert_events(actual_events: &[Event], expected_events: &[Event]) {
    assert_eq!(actual_events, expected_events, "event mismatch");
}

pub fn assert_client_calls(
    client_calls: &Arc<Mutex<Vec<(Model, Vec<ChatMessage>, Vec<code_g::client::model::Tool>)>>>,
    expected_client_calls: &(Model, Vec<ChatMessage>, Vec<code_g::client::model::Tool>),
) {
    let calls = client_calls.lock().unwrap().clone();
    assert_eq!(
        calls.last(),
        Some(expected_client_calls),
        "client calls mismatch"
    );
}

pub fn assert_chat_history(chat_history: &[ChatMessage], expected_chat_history: &[ChatMessage]) {
    assert_eq!(chat_history, expected_chat_history, "chat history mismatch");
}

pub fn assert_tool_calls(
    tool_calls: &Arc<Mutex<Vec<(String, HashMap<String, String>)>>>,
    expected_tool_call: &(String, HashMap<String, String>),
) {
    let calls = tool_calls.lock().unwrap().clone();
    assert_eq!(
        calls.last(),
        Some(expected_tool_call),
        "tool calls mismatch"
    );
}
