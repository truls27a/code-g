use std::collections::HashMap;
use std::io;

/// The events that can occur in a chat session.
/// Each event is either a system event, a user event, or an assistant event.
#[derive(Debug, PartialEq)]
pub enum Event {
    // System events
    SessionStarted,
    SessionEnded,

    // User events
    ReceivedUserMessage(String),

    // Assistant events
    ReceivedAssistantMessage(String),
    ReceivedToolCall(String, HashMap<String, String>),
    ReceivedToolResponse(String, String, HashMap<String, String>),
    AwaitingAssistantResponse,
}

/// The actions that can occur in a chat session.
/// Right now, the only action is to request user input.
pub enum Action {
    RequestUserInput,
}

/// Trait for handling chat session events and actions
pub trait EventHandler {
    /// Handle a chat session event
    fn handle_event(&mut self, event: Event);

    /// Handle a chat session action and return the result
    fn handle_action(&mut self, action: Action) -> Result<String, io::Error>;
}
