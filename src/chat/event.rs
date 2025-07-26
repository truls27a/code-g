use std::collections::HashMap;
use std::io;

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

/// Trait for handling chat session events and actions
pub trait ChatSessionEventHandler {
    /// Handle a chat session event
    fn handle_event(&mut self, event: ChatSessionEvent);
    
    /// Handle a chat session action and return the result
    fn handle_action(&mut self, action: ChatSessionAction) -> Result<String, io::Error>;
}
