use code_g::session::event::{Action, Event, EventHandler};
use std::io;

/// Mock implementation of EventHandler for testing purposes.
///
/// This mock event handler allows you to configure predefined responses
/// and capture events for testing without requiring actual user interaction.
/// It supports configuring responses for user input requests and approval
/// requests, and records all events that occur during testing.
///
/// # Examples
///
/// ```rust
/// use code_g::tui::mock::MockEventHandler;
/// use code_g::session::event::{Event, Action};
///
/// // Create a mock that provides user inputs
/// let inputs = vec!["Hello".to_string(), "exit".to_string()];
/// let mock = MockEventHandler::new_with_inputs(inputs);
/// ```
#[derive(Debug)]
pub struct MockEventHandler {
    events: Vec<Event>,
    input_responses: Vec<String>,
    current_input_index: usize,
    approval_response: String,
}

impl MockEventHandler {
    /// Creates a new mock event handler with no predefined responses.
    ///
    /// User input requests will return "exit" by default.
    ///
    /// # Returns
    ///
    /// A new `MockEventHandler` with default behavior.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use code_g::tui::mock::MockEventHandler;
    ///
    /// let mock = MockEventHandler::new();
    /// ```
    pub fn new() -> Self {
        Self {
            events: Vec::new(),
            input_responses: Vec::new(),
            current_input_index: 0,
            approval_response: "approved".to_string(),
        }
    }

    /// Creates a new mock event handler with predefined input responses.
    ///
    /// The handler will provide each input in sequence, then return "exit"
    /// for any subsequent input requests.
    ///
    /// # Arguments
    ///
    /// * `responses` - The input responses to provide in sequence
    ///
    /// # Returns
    ///
    /// A new `MockEventHandler` configured with the specified inputs.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use code_g::tui::mock::MockEventHandler;
    ///
    /// let inputs = vec!["Hello".to_string(), "How are you?".to_string()];
    /// let mock = MockEventHandler::new_with_inputs(inputs);
    /// ```
    pub fn new_with_inputs(responses: Vec<String>) -> Self {
        Self {
            events: Vec::new(),
            input_responses: responses,
            current_input_index: 0,
            approval_response: "approved".to_string(),
        }
    }

    /// Creates a new mock event handler that declines approval requests.
    ///
    /// # Returns
    ///
    /// A new `MockEventHandler` that will decline approval requests.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use code_g::tui::mock::MockEventHandler;
    ///
    /// let mock = MockEventHandler::new_declining_approval();
    /// ```
    pub fn new_declining_approval() -> Self {
        Self {
            events: Vec::new(),
            input_responses: Vec::new(),
            current_input_index: 0,
            approval_response: "declined".to_string(),
        }
    }

    /// Returns all events that have been recorded during testing.
    ///
    /// This is useful for verifying that the expected events occurred
    /// during a test session.
    ///
    /// # Returns
    ///
    /// A reference to the vector of recorded events.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use code_g::tui::mock::MockEventHandler;
    /// use code_g::session::event::Event;
    ///
    /// let mut mock = MockEventHandler::new();
    /// mock.handle_event(Event::SessionStarted);
    ///
    /// let events = mock.get_events();
    /// assert_eq!(events.len(), 1);
    /// ```
    pub fn get_events(&self) -> &Vec<Event> {
        &self.events
    }

    /// Clears all recorded events.
    ///
    /// This is useful for resetting the mock state between tests.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use code_g::tui::mock::MockEventHandler;
    ///
    /// let mut mock = MockEventHandler::new();
    /// mock.clear_events();
    /// ```
    pub fn clear_events(&mut self) {
        self.events.clear();
    }

    /// Sets the response for approval requests.
    ///
    /// # Arguments
    ///
    /// * `response` - The response to return for approval requests
    ///
    /// # Examples
    ///
    /// ```rust
    /// use code_g::tui::mock::MockEventHandler;
    ///
    /// let mut mock = MockEventHandler::new();
    /// mock.set_approval_response("declined".to_string());
    /// ```
    pub fn set_approval_response(&mut self, response: String) {
        self.approval_response = response;
    }

    /// Adds additional input responses that will be returned in sequence.
    ///
    /// # Arguments
    ///
    /// * `responses` - Additional responses to add to the queue
    ///
    /// # Examples
    ///
    /// ```rust
    /// use code_g::tui::mock::MockEventHandler;
    ///
    /// let mut mock = MockEventHandler::new();
    /// mock.add_input_responses(vec!["Hello".to_string()]);
    /// ```
    pub fn add_input_responses(&mut self, mut responses: Vec<String>) {
        self.input_responses.append(&mut responses);
    }
}

impl EventHandler for MockEventHandler {
    fn handle_event(&mut self, event: Event) {
        self.events.push(event);
    }

    fn handle_action(&mut self, action: Action) -> Result<String, io::Error> {
        match action {
            Action::RequestUserInput => {
                if self.current_input_index < self.input_responses.len() {
                    let response = self.input_responses[self.current_input_index].clone();
                    self.current_input_index += 1;
                    Ok(response)
                } else {
                    Ok("exit".to_string())
                }
            }
            Action::RequestUserApproval { .. } => Ok(self.approval_response.clone()),
        }
    }
}

impl Default for MockEventHandler {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_creates_empty_mock_event_handler() {
        let mock = MockEventHandler::new();
        assert_eq!(mock.get_events().len(), 0);
        assert_eq!(mock.input_responses.len(), 0);
        assert_eq!(mock.approval_response, "approved");
    }

    #[test]
    fn new_with_inputs_stores_responses() {
        let inputs = vec!["Hello".to_string(), "World".to_string()];
        let mock = MockEventHandler::new_with_inputs(inputs.clone());
        assert_eq!(mock.input_responses, inputs);
    }

    #[test]
    fn new_declining_approval_sets_declined_response() {
        let mock = MockEventHandler::new_declining_approval();
        assert_eq!(mock.approval_response, "declined");
    }

    #[test]
    fn handle_event_records_events() {
        let mut mock = MockEventHandler::new();

        mock.handle_event(Event::SessionStarted);
        mock.handle_event(Event::ReceivedUserMessage {
            message: "Test".to_string(),
        });

        let events = mock.get_events();
        assert_eq!(events.len(), 2);
        assert_eq!(events[0], Event::SessionStarted);
        assert_eq!(
            events[1],
            Event::ReceivedUserMessage {
                message: "Test".to_string(),
            }
        );
    }

    #[test]
    fn handle_action_request_user_input_returns_configured_responses() {
        let inputs = vec!["First".to_string(), "Second".to_string()];
        let mut mock = MockEventHandler::new_with_inputs(inputs);

        let response1 = mock.handle_action(Action::RequestUserInput).unwrap();
        assert_eq!(response1, "First");

        let response2 = mock.handle_action(Action::RequestUserInput).unwrap();
        assert_eq!(response2, "Second");

        // After exhausting responses, should return "exit"
        let response3 = mock.handle_action(Action::RequestUserInput).unwrap();
        assert_eq!(response3, "exit");
    }

    #[test]
    fn handle_action_request_user_approval_returns_configured_response() {
        let mut mock = MockEventHandler::new();

        let result = mock
            .handle_action(Action::RequestUserApproval {
                operation: "Test Operation".to_string(),
                details: "Test Details".to_string(),
                tool_name: "test_tool".to_string(),
            })
            .unwrap();

        assert_eq!(result, "approved");
    }

    #[test]
    fn handle_action_request_user_approval_returns_declined_when_configured() {
        let mut mock = MockEventHandler::new_declining_approval();

        let result = mock
            .handle_action(Action::RequestUserApproval {
                operation: "Test Operation".to_string(),
                details: "Test Details".to_string(),
                tool_name: "test_tool".to_string(),
            })
            .unwrap();

        assert_eq!(result, "declined");
    }

    #[test]
    fn clear_events_removes_all_events() {
        let mut mock = MockEventHandler::new();
        mock.handle_event(Event::SessionStarted);
        mock.handle_event(Event::SessionEnded);

        assert_eq!(mock.get_events().len(), 2);

        mock.clear_events();
        assert_eq!(mock.get_events().len(), 0);
    }

    #[test]
    fn set_approval_response_changes_approval_behavior() {
        let mut mock = MockEventHandler::new();
        mock.set_approval_response("custom_response".to_string());

        let result = mock
            .handle_action(Action::RequestUserApproval {
                operation: "Test".to_string(),
                details: "Test".to_string(),
                tool_name: "test".to_string(),
            })
            .unwrap();

        assert_eq!(result, "custom_response");
    }

    #[test]
    fn add_input_responses_extends_response_queue() {
        let mut mock = MockEventHandler::new_with_inputs(vec!["First".to_string()]);
        mock.add_input_responses(vec!["Second".to_string(), "Third".to_string()]);

        assert_eq!(mock.input_responses.len(), 3);
        assert_eq!(
            mock.handle_action(Action::RequestUserInput).unwrap(),
            "First"
        );
        assert_eq!(
            mock.handle_action(Action::RequestUserInput).unwrap(),
            "Second"
        );
        assert_eq!(
            mock.handle_action(Action::RequestUserInput).unwrap(),
            "Third"
        );
    }

    #[test]
    fn default_creates_same_as_new() {
        let mock1 = MockEventHandler::new();
        let mock2 = MockEventHandler::default();

        assert_eq!(mock1.get_events().len(), mock2.get_events().len());
        assert_eq!(mock1.input_responses.len(), mock2.input_responses.len());
        assert_eq!(mock1.approval_response, mock2.approval_response);
    }
}
