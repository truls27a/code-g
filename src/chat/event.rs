use std::collections::HashMap;
use std::io;

#[derive(Debug, PartialEq)]
pub enum Event {
    // System events
    SessionStarted,
    SessionEnded,

    // User events
    ReceivedUserMessage(String),
    ReceivedAssistantMessage(String),
    ReceivedToolCall(String, HashMap<String, String>),
    ReceivedToolResponse(String, String, HashMap<String, String>),
    AwaitingAssistantResponse,
    
    // Change management events
    PendingFileChange { change_id: u64, file_path: String, diff: String },
    ChangeAccepted { change_id: u64, accepted_changes: Vec<u64> },
    ChangeDeclined { change_id: u64 },
    ChangeError { change_id: u64, error: String },
}

#[derive(Debug)]
pub enum Action {
    RequestUserInput,
    AcceptChange(u64),
    DeclineChange(u64),
    ListPendingChanges,
}

/// Trait for handling chat session events and actions
pub trait EventHandler {
    /// Handle a chat session event
    fn handle_event(&mut self, event: Event);

    /// Handle a chat session action and return the result
    fn handle_action(&mut self, action: Action) -> Result<String, io::Error>;
}
