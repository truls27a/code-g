#![allow(dead_code)]

use async_trait::async_trait;
use code_g::client::{
    error::ChatClientError,
    model::{ChatMessage, ChatResult, Model, Tool},
    traits::ChatClient,
};
use std::sync::{Arc, Mutex};

#[derive(Clone)]
pub struct MockChatClient {
    queue: Arc<Mutex<Vec<Result<ChatResult, ChatClientError>>>>,
    calls: Arc<Mutex<Vec<(Model, Vec<ChatMessage>, Vec<Tool>)>>>,
}

impl MockChatClient {
    /// Creates a new mock chat client with the given queue.
    ///
    /// # Arguments
    ///
    /// * `queue` - A vector of results that will be returned when the chat client is called.
    /// * `calls` - A vector of calls that will be returned when the chat client is called.
    ///
    /// # Returns
    ///
    /// A new mock chat client with the given queue.
    pub fn new(
        queue: Vec<Result<ChatResult, ChatClientError>>,
        calls: Arc<Mutex<Vec<(Model, Vec<ChatMessage>, Vec<Tool>)>>>,
    ) -> Self {
        Self {
            queue: Arc::new(Mutex::new(queue)),
            calls,
        }
    }

    /// Pushes a result to the queue.
    ///
    /// # Arguments
    ///
    /// * `result` - The result to push to the queue.
    pub fn push(&self, result: Result<ChatResult, ChatClientError>) {
        self.queue.lock().unwrap().push(result);
    }

    /// Returns the calls that the chat client has made.
    ///
    /// # Returns
    ///
    /// A vector of tuples that contain the model, chat messages, and tools that the chat client has made.
    pub fn calls(&self) -> Vec<(Model, Vec<ChatMessage>, Vec<Tool>)> {
        self.calls.lock().unwrap().clone()
    }
}

#[async_trait]
impl ChatClient for MockChatClient {
    async fn create_chat_completion(
        &self,
        model: &Model,
        chat_history: &[ChatMessage],
        tools: &[Tool],
    ) -> Result<ChatResult, ChatClientError> {
        // Record the call
        self.calls
            .lock()
            .unwrap()
            .push((model.clone(), chat_history.to_vec(), tools.to_vec()));

        // Return the next result from the queue
        match self.queue.lock().unwrap().remove(0) {
            Ok(result) => Ok(result),
            Err(e) => Err(e),
        }
    }
}
