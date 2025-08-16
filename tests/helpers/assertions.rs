use code_g::client::model::ChatMessage;
use code_g::session::event::Event;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

pub fn assert_events(actual_events: &[Event], expected_events: &[Event]) {
    assert_eq!(actual_events, expected_events, "event mismatch");
}

pub fn assert_chat_history(chat_history: &[ChatMessage], expected_chat_history: &[ChatMessage]) {
    assert_eq!(chat_history, expected_chat_history, "chat history mismatch");
}

pub fn assert_tool_calls(
    tool_calls: &Arc<Mutex<Vec<(String, HashMap<String, String>)>>>,
    expected_tool_calls: &[(String, HashMap<String, String>)],
) {
    let calls = tool_calls.lock().unwrap().clone();
    assert_eq!(calls, expected_tool_calls, "tool calls mismatch");
}
