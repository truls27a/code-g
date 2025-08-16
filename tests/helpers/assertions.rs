use code_g::client::model::{ChatMessage, Model};
use code_g::session::event::Event;
use std::sync::{Arc, Mutex};

pub fn assert_events_transcript(actual_events: &[Event], expected_events: &[Event]) {
    assert_eq!(actual_events, expected_events, "event transcript mismatch");
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

