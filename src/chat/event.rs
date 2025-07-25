use std::collections::HashMap;

#[derive(Debug, PartialEq)]
pub enum ChatSessionEvent {
    // System events
    SessionStarted,
    SessionEnded,

    // User events
    ReceivedUserMessage(String),
    ReceivedAssistantMessage(String),
    ReceivedToolCall(String, HashMap<String, String>),
    ReceivedToolResponse(String, String, HashMap<String, String>),
    AwaitingAssistantResponse,
}


pub enum ChatSessionAction {
    RequestUserInput,
}
