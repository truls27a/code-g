#[derive(Debug, PartialEq)]
pub enum ChatSessionEvent {
    // System events
    SessionStarted,
    SessionEnded,

    // User events
    ReceivedUserMessage(String),
    ReceivedAssistantMessage(String),
    ReceivedToolCall(String),
    ReceivedToolResponse(String),
    SetStatusMessages(Vec<StatusMessage>),
}


pub enum ChatSessionAction {
    RequestUserInput,
}

#[derive(Debug, PartialEq)]
pub enum StatusMessage {
    Thinking,
    ToolCallName(String),
}