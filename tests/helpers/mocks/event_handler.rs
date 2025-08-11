use std::sync::{Arc, Mutex};
use code_g::session::event::Event;

/// Mock event handler for testing.
/// 
/// This mock event handler is used to test the event handling logic in the `ChatSession`.
/// It allows for the simulation of user input and approval requests, and provides methods
/// to inspect the events and inputs that have been handled.
pub struct MockEventHandler {
    events: Arc<Mutex<Vec<Event>>>,
    inputs: Arc<Mutex<Vec<String>>>,
    approvals: Arc<Mutex<Vec<String>>>,
    max_inputs: usize,
}

impl MockEventHandler {
    /// Create a new MockEventHandler.
    /// 
    /// # Arguments
    /// 
    /// * `events` - The events to be handled.
    /// * `inputs` - The inputs to be handled.
    /// * `approvals` - The approvals to be handled.
    /// * `max_inputs` - The maximum number of inputs to handle before returning "exit".
    /// 
    /// # Returns
    /// 
    /// A new `MockEventHandler` instance.
    pub fn new(events: Vec<Event>, inputs: Vec<String>, approvals: Vec<String>, max_inputs: usize) -> Self {
        Self {
            events: Arc::new(Mutex::new(events)),
            inputs: Arc::new(Mutex::new(inputs)),
            approvals: Arc::new(Mutex::new(approvals)),
            max_inputs,
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

    /// Get the number of max inputs.
    /// 
    /// # Returns
    /// 
    /// The number of max inputs.
    pub fn max_inputs(&self) -> usize {
        self.max_inputs
    }
}

impl EventHandler for MockEventHandler {
    fn handle_event(&mut self, event: Event) {
        self.events.lock().unwrap().push(event);
    }

    fn handle_action(&mut self, action: Action) -> Result<String, io::Error> {
        match action {
            Action::RequestUserInput => {
                if self.inputs.lock().unwrap().len() >= self.max_inputs {
                    Ok("exit".to_string())
                } else {
                    let input = self.inputs.lock().unwrap().pop().unwrap();
                    Ok(input)
                }
            }
            Action::RequestUserApproval { operation, details, tool_name } => {
                self.approvals.lock().unwrap().push(format!("{}: {}", operation, details));
                Ok(format!("{}: {}", operation, details))
            }
        }
    }
}