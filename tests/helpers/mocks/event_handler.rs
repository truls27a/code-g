use std::sync::{Arc, Mutex};

use code_g::session::event::Event;

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
    /// * `events` - The events to be handled.
    /// * `inputs` - The inputs to be handled.
    /// * `approvals` - The approvals to be handled.
    /// 
    /// # Returns
    /// 
    /// A new `MockEventHandler` instance.
    pub fn new(events: Vec<Event>, inputs: Vec<String>, approvals: Vec<String>) -> Self {
        Self {
            events: Arc::new(Mutex::new(events)),
            inputs: Arc::new(Mutex::new(inputs)),
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
                self.inputs.lock().unwrap().push(input);
                Ok(input)
            }
            Action::RequestUserApproval { operation, details, tool_name } => {
                self.approvals.lock().unwrap().push(format!("{}: {}", operation, details));
                Ok(format!("{}: {}", operation, details))
            }
        }
    }
}

