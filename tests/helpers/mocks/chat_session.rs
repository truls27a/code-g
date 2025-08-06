use code_g::session::error::ChatSessionError;
use code_g::session::event::{Action, Event};

/// Mock implementation of ChatSession for testing purposes.
///
/// This mock session allows you to configure predefined responses and behaviors
/// for testing without running actual chat sessions. It supports configuring
/// whether the session should succeed, fail, or simulate specific behaviors
/// like user inputs and events.
///
/// # Examples
///
/// ```rust
/// use code_g::session::mock::MockChatSession;
/// use code_g::session::error::ChatSessionError;
/// use tokio::runtime::Runtime;
///
/// // Create a mock that succeeds
/// let mock = MockChatSession::new_success();
///
/// // Use it in tests
/// let rt = Runtime::new().unwrap();
/// rt.block_on(async {
///     let result = mock.run().await;
///     assert!(result.is_ok());
/// });
/// ```
#[derive(Debug)]
pub struct MockChatSession {
    behavior: MockSessionBehavior,
    events: Vec<Event>,
    actions: Vec<Action>,
}

/// Represents the different types of behaviors the mock session can exhibit.
#[derive(Debug, Clone)]
pub enum MockSessionBehavior {
    /// Session completes successfully
    Success,
    /// Session fails with the specified error
    Error(ChatSessionError),
    /// Session succeeds with specific user inputs
    SuccessWithInputs(Vec<String>),
    /// Session handles specific number of messages then exits
    HandleMessages(usize),
}

impl MockChatSession {
    /// Creates a new mock session that succeeds immediately.
    ///
    /// # Returns
    ///
    /// A new `MockChatSession` configured to succeed.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use code_g::session::mock::MockChatSession;
    ///
    /// let mock = MockChatSession::new_success();
    /// ```
    pub fn new_success() -> Self {
        Self {
            behavior: MockSessionBehavior::Success,
            events: Vec::new(),
            actions: Vec::new(),
        }
    }

    /// Creates a new mock session that fails with the specified error.
    ///
    /// # Arguments
    ///
    /// * `error` - The error the session should return
    ///
    /// # Returns
    ///
    /// A new `MockChatSession` configured to fail with the specified error.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use code_g::session::mock::MockChatSession;
    /// use code_g::session::error::ChatSessionError;
    ///
    /// let error = ChatSessionError::MaxIterationsExceeded { max_iterations: 10 };
    /// let mock = MockChatSession::new_error(error);
    /// ```
    pub fn new_error(error: ChatSessionError) -> Self {
        Self {
            behavior: MockSessionBehavior::Error(error),
            events: Vec::new(),
            actions: Vec::new(),
        }
    }

    /// Creates a new mock session that simulates user inputs.
    ///
    /// The session will process each input in sequence and then exit.
    ///
    /// # Arguments
    ///
    /// * `inputs` - The user inputs to simulate
    ///
    /// # Returns
    ///
    /// A new `MockChatSession` configured to handle the specified inputs.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use code_g::session::mock::MockChatSession;
    ///
    /// let inputs = vec!["Hello".to_string(), "How are you?".to_string()];
    /// let mock = MockChatSession::new_with_inputs(inputs);
    /// ```
    pub fn new_with_inputs(inputs: Vec<String>) -> Self {
        Self {
            behavior: MockSessionBehavior::SuccessWithInputs(inputs),
            events: Vec::new(),
            actions: Vec::new(),
        }
    }

    /// Creates a new mock session that handles a specific number of messages.
    ///
    /// The session will simulate processing the specified number of messages
    /// and then exit successfully.
    ///
    /// # Arguments
    ///
    /// * `message_count` - The number of messages to handle
    ///
    /// # Returns
    ///
    /// A new `MockChatSession` configured to handle the specified number of messages.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use code_g::session::mock::MockChatSession;
    ///
    /// let mock = MockChatSession::new_handle_messages(3);
    /// ```
    pub fn new_handle_messages(message_count: usize) -> Self {
        Self {
            behavior: MockSessionBehavior::HandleMessages(message_count),
            events: Vec::new(),
            actions: Vec::new(),
        }
    }

    /// Simulates running the chat session.
    ///
    /// This method mimics the behavior of the real `ChatSession::run()` method
    /// based on the configured behavior.
    ///
    /// # Returns
    ///
    /// Returns [`Ok(())`] for success behaviors or the configured error for error behaviors.
    ///
    /// # Errors
    ///
    /// Returns [`ChatSessionError`] if configured to return an error.
    pub async fn run(&mut self) -> Result<(), ChatSessionError> {
        // Record that the session started
        self.events.push(Event::SessionStarted);

        match self.behavior.clone() {
            MockSessionBehavior::Success => {
                self.events.push(Event::SessionEnded);
                Ok(())
            }
            MockSessionBehavior::Error(error) => Err(error),
            MockSessionBehavior::SuccessWithInputs(inputs) => {
                for input in inputs {
                    self.actions.push(Action::RequestUserInput);
                    self.events.push(Event::ReceivedUserMessage {
                        message: input.clone(),
                    });
                    self.events.push(Event::AwaitingAssistantResponse);
                    self.events.push(Event::ReceivedAssistantMessage {
                        message: format!("Mock response to: {}", input),
                    });
                }
                self.events.push(Event::SessionEnded);
                Ok(())
            }
            MockSessionBehavior::HandleMessages(count) => {
                for i in 0..count {
                    let message = format!("Message {}", i + 1);
                    self.actions.push(Action::RequestUserInput);
                    self.events.push(Event::ReceivedUserMessage {
                        message: message.clone(),
                    });
                    self.events.push(Event::AwaitingAssistantResponse);
                    self.events.push(Event::ReceivedAssistantMessage {
                        message: format!("Mock response to: {}", message),
                    });
                }
                self.events.push(Event::SessionEnded);
                Ok(())
            }
        }
    }

    /// Simulates sending a message to the session.
    ///
    /// This method mimics the behavior of the real `ChatSession::send_message()` method.
    ///
    /// # Arguments
    ///
    /// * `message` - The message to send
    ///
    /// # Returns
    ///
    /// A mock response to the message.
    ///
    /// # Errors
    ///
    /// Returns [`ChatSessionError`] if configured to return an error.
    pub async fn send_message(&mut self, message: &str) -> Result<String, ChatSessionError> {
        self.events.push(Event::ReceivedUserMessage {
            message: message.to_string(),
        });
        self.events.push(Event::AwaitingAssistantResponse);

        match self.behavior.clone() {
            MockSessionBehavior::Error(error) => Err(error),
            _ => {
                let response = format!("Mock response to: {}", message);
                self.events.push(Event::ReceivedAssistantMessage {
                    message: response.clone(),
                });
                Ok(response)
            }
        }
    }

    /// Returns all events that have been recorded during the session.
    ///
    /// This is useful for testing to verify that the expected events occurred.
    ///
    /// # Returns
    ///
    /// A reference to the vector of recorded events.
    pub fn get_events(&self) -> &Vec<Event> {
        &self.events
    }

    /// Returns all actions that have been recorded during the session.
    ///
    /// This is useful for testing to verify that the expected actions were requested.
    ///
    /// # Returns
    ///
    /// A reference to the vector of recorded actions.
    pub fn get_actions(&self) -> &Vec<Action> {
        &self.actions
    }

    /// Clears all recorded events and actions.
    ///
    /// This is useful for resetting the mock state between tests.
    pub fn reset(&mut self) {
        self.events.clear();
        self.actions.clear();
    }

    /// Creates a default mock session that succeeds.
    ///
    /// This is a convenience method for quickly creating a basic mock for testing.
    ///
    /// # Returns
    ///
    /// A new `MockChatSession` with success behavior.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use code_g::session::mock::MockChatSession;
    ///
    /// let mock = MockChatSession::default();
    /// ```
    pub fn default() -> Self {
        Self::new_success()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn mock_success_completes_successfully() {
        let mut mock = MockChatSession::new_success();

        let result = mock.run().await;
        assert!(result.is_ok());

        let events = mock.get_events();
        assert_eq!(events.len(), 2);
        assert_eq!(events[0], Event::SessionStarted);
        assert_eq!(events[1], Event::SessionEnded);
    }

    #[tokio::test]
    async fn mock_error_returns_configured_error() {
        let error = ChatSessionError::MaxIterationsExceeded { max_iterations: 10 };
        let mut mock = MockChatSession::new_error(error.clone());

        let result = mock.run().await;
        assert!(result.is_err());

        match result.unwrap_err() {
            ChatSessionError::MaxIterationsExceeded { max_iterations } => {
                assert_eq!(max_iterations, 10);
            }
            _ => panic!("Expected MaxIterationsExceeded error"),
        }
    }

    #[tokio::test]
    async fn mock_with_inputs_processes_all_inputs() {
        let inputs = vec!["Hello".to_string(), "How are you?".to_string()];
        let mut mock = MockChatSession::new_with_inputs(inputs.clone());

        let result = mock.run().await;
        assert!(result.is_ok());

        let events = mock.get_events();
        // Should have: SessionStarted + 2*(ReceivedUserMessage + AwaitingAssistantResponse + ReceivedAssistantMessage) + SessionEnded
        assert_eq!(events.len(), 8);
        assert_eq!(events[0], Event::SessionStarted);
        assert_eq!(events[7], Event::SessionEnded);

        // Check that user messages were recorded
        assert_eq!(
            events[1],
            Event::ReceivedUserMessage {
                message: "Hello".to_string()
            }
        );
        assert_eq!(
            events[4],
            Event::ReceivedUserMessage {
                message: "How are you?".to_string()
            }
        );
    }

    #[tokio::test]
    async fn mock_handle_messages_processes_correct_count() {
        let mut mock = MockChatSession::new_handle_messages(2);

        let result = mock.run().await;
        assert!(result.is_ok());

        let events = mock.get_events();
        // Should have: SessionStarted + 2*(ReceivedUserMessage + AwaitingAssistantResponse + ReceivedAssistantMessage) + SessionEnded
        assert_eq!(events.len(), 8);
    }

    #[tokio::test]
    async fn send_message_returns_mock_response() {
        let mut mock = MockChatSession::new_success();

        let result = mock.send_message("Hello").await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "Mock response to: Hello");

        let events = mock.get_events();
        assert_eq!(events.len(), 3);
        assert_eq!(
            events[0],
            Event::ReceivedUserMessage {
                message: "Hello".to_string()
            }
        );
        assert_eq!(events[1], Event::AwaitingAssistantResponse);
        assert_eq!(
            events[2],
            Event::ReceivedAssistantMessage {
                message: "Mock response to: Hello".to_string()
            }
        );
    }

    #[tokio::test]
    async fn send_message_returns_error_when_configured() {
        let error = ChatSessionError::MaxIterationsExceeded { max_iterations: 5 };
        let mut mock = MockChatSession::new_error(error.clone());

        let result = mock.send_message("Hello").await;
        assert!(result.is_err());

        match result.unwrap_err() {
            ChatSessionError::MaxIterationsExceeded { max_iterations } => {
                assert_eq!(max_iterations, 5);
            }
            _ => panic!("Expected MaxIterationsExceeded error"),
        }
    }

    #[test]
    fn reset_clears_events_and_actions() {
        let mut mock = MockChatSession::new_success();
        mock.events.push(Event::SessionStarted);
        mock.actions.push(Action::RequestUserInput);

        assert_eq!(mock.get_events().len(), 1);
        assert_eq!(mock.get_actions().len(), 1);

        mock.reset();

        assert_eq!(mock.get_events().len(), 0);
        assert_eq!(mock.get_actions().len(), 0);
    }

    #[test]
    fn default_creates_success_mock() {
        let mock = MockChatSession::default();
        match mock.behavior {
            MockSessionBehavior::Success => (), // Expected
            _ => panic!("Default should create success behavior"),
        }
    }
}
