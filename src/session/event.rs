use std::collections::HashMap;
use std::io;

/// Events that can occur during a chat session.
///
/// This enum represents all possible events that can happen in a chat session,
/// including system lifecycle events, user interactions, and assistant responses.
/// Events are used to track the state and flow of the conversation.
///
/// # Examples
///
/// ```rust
/// use code_g::session::event::Event;
///
/// let event = Event::ReceivedUserMessage { message: "Hello".to_string() };
/// ```
#[derive(Debug, PartialEq, Clone)]
pub enum Event {
    /// The chat session has been started
    SessionStarted,
    /// The chat session has been ended
    SessionEnded,

    /// A message was received from the user
    ReceivedUserMessage { message: String },

    /// A message was received from the assistant
    ReceivedAssistantMessage { message: String },
    /// A tool call was received from the assistant with tool name and parameters
    ReceivedToolCall {
        tool_name: String,
        parameters: HashMap<String, String>,
    },
    /// A tool response was received with tool name, response, and parameters
    ReceivedToolResponse {
        tool_name: String,
        response: String,
        parameters: HashMap<String, String>,
    },
    /// The system is waiting for the assistant to respond
    AwaitingAssistantResponse,
}

/// Actions that can be requested during a chat session.
///
/// This enum represents actions that need to be performed during the chat session,
/// typically requiring external input or interaction. Actions are used to coordinate
/// between the chat system and the user interface.
///
/// # Examples
///
/// ```rust
/// use code_g::session::event::Action;
///
/// let action = Action::RequestUserInput;
/// ```
#[derive(Debug, PartialEq)]
pub enum Action {
    /// Request input from the user
    RequestUserInput,
    /// Request user approval for a potentially dangerous operation
    RequestUserApproval {
        operation: String,
        details: String,
        tool_name: String,
    },
}

/// Trait for handling chat session events and actions.
///
/// This trait defines the interface for handling chat events and actions in a chat session.
/// Implementors can define custom behavior for processing events and handling actions
/// that require external interaction.
///
/// # Examples
///
/// ```rust
/// use code_g::session::event::{Event, Action, EventHandler};
/// use std::io;
///
/// struct MyHandler;
///
/// impl EventHandler for MyHandler {
///     fn handle_event(&mut self, event: Event) {
///         // Handle the event
///     }
///
///     fn handle_action(&mut self, action: Action) -> Result<String, io::Error> {
///         Ok("result".to_string())
///     }
/// }
/// ```
pub trait EventHandler {
    /// Handle a chat session event.
    ///
    /// This method is called whenever an event occurs in the chat session.
    /// The implementation should process the event according to the desired behavior.
    /// For example, render the event in the terminal.
    ///
    /// # Arguments
    ///
    /// * `event` - The event that occurred in the chat session
    fn handle_event(&mut self, event: Event);

    /// Handle a chat session action and return the result.
    ///
    /// This method is called when an action needs to be performed that requires
    /// external interaction or input. The implementation should perform the
    /// requested action and return the result. For example, request user input.
    ///
    /// # Arguments
    ///
    /// * `action` - The action that needs to be performed
    ///
    /// # Returns
    ///
    /// A `String` containing the result of the action.
    ///
    /// # Errors
    ///
    /// Returns an `io::Error` if the action cannot be completed due to I/O issues.
    fn handle_action(&mut self, action: Action) -> Result<String, io::Error>;
}
