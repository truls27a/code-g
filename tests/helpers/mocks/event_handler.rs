#![allow(dead_code)]

use code_g::session::event::{Action, Event, EventHandler};
use std::io;
use std::sync::{Arc, Mutex};

/// Mock event handler for testing.
///
/// This mock event handler is used to test the event handling logic in the `ChatSession`.
/// It allows for the simulation of user input and approval requests, and provides methods
/// to inspect the events and inputs that have been handled.
pub struct MockEventHandler {
    events: Arc<Mutex<Vec<Event>>>,
    inputs: Arc<Mutex<Vec<String>>>,
    approvals: Arc<Mutex<Vec<String>>>,
}

impl MockEventHandler {
    /// Create a new MockEventHandler.
    ///
    /// # Arguments
    ///
    /// * `inputs` - The message to be returned on handle_action for RequestUserInput.
    /// * `approvals` - The approval message to be returned on handle_action for RequestUserApproval.
    ///
    /// # Returns
    ///
    /// A new `MockEventHandler` instance.
    pub fn new(events: Arc<Mutex<Vec<Event>>>, inputs: Vec<String>, approvals: Vec<String>) -> Self {
        // Add "exit" to the inputs to simulate the user exiting the chat.
        let mut inputs_with_exit = inputs;
        inputs_with_exit.push("exit".to_string());

        Self {
            events,
            inputs: Arc::new(Mutex::new(inputs_with_exit)),
            approvals: Arc::new(Mutex::new(approvals)),
        }
    }

    /// Get the events that have been handled.
    ///
    /// # Returns
    ///
    /// A vector of events that have been handled.
    pub fn events(&self) -> Vec<Event> {
        self.events.lock().unwrap().clone()
    }

    /// Get the inputs that have been handled.
    ///
    /// # Returns
    ///
    /// A vector of inputs that have been handled.
    pub fn inputs(&self) -> Vec<String> {
        self.inputs.lock().unwrap().clone()
    }

    /// Get the approvals that have been handled.
    ///
    /// # Returns
    ///
    /// A vector of approvals that have been handled.
    pub fn approvals(&self) -> Vec<String> {
        self.approvals.lock().unwrap().clone()
    }
}

impl EventHandler for MockEventHandler {
    fn handle_event(&mut self, event: Event) {
        self.events.lock().unwrap().push(event);
    }

    fn handle_action(&mut self, action: Action) -> Result<String, io::Error> {
        match action {
            Action::RequestUserInput => {
                let input = self.inputs.lock().unwrap().remove(0);
                Ok(input)
            }
            Action::RequestUserApproval { .. } => {
                let approval = self.approvals.lock().unwrap().remove(0);
                Ok(approval)
            }
        }
    }
}
