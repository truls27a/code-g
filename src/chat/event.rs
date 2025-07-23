#[derive(Debug, PartialEq)]
pub enum ChatSessionEvent {
    // System events
    SessionStarted,
    SessionEnded,

    // User events
    ReceivedUserMessage(String),
    ReceivedAssistantMessage(String),
    ReceivedToolCall(String),
    ReceivedToolResponse(String, String),
    AwaitingAssistantResponse,
}


pub enum ChatSessionAction {
    RequestUserInput,
}
