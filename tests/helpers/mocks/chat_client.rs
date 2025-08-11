use code_g::client::{
    error::ChatClientError,
    model::{ChatMessage, ChatResult, Model, Tool},
    traits::ChatClient,
};
use async_trait::async_trait;
use std::sync::{Arc, Mutex};

pub struct MockChatClient {
    queue: Arc<Mutex<Vec<Result<ChatResult, ChatClientError>>>>,
    calls: Arc<Mutex<Vec<(Model, Vec<ChatMessage>, Vec<Tool>)>>>,
}

impl MockChatClient {
    /// Creates a new mock chat client with the given queue and calls.
    ///
    /// # Arguments
    ///
    /// * `queue` - A vector of results that will be returned when the chat client is called.
    /// * `calls` - A vector of calls that the chat client has made.
    ///
    /// # Returns
    ///
    /// A new mock chat client with the given queue and calls.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use code_g::client::{ChatClient, ChatResult, ChatMessage, Model, Tool};
    /// use std::sync::{Arc, Mutex};
    ///
    /// let queue = vec![Ok(ChatResult::Message {
    ///     content: "Mock response".to_string(),
    ///     turn_over: true,
    /// })];
    /// let calls = vec![(Model::OpenAi(OpenAiModel::Gpt4o), vec![ChatMessage::User { content: "Hello, how are you?".to_string() }], vec![])];
    ///
    /// let mock_client = MockChatClient::new(queue, calls);
    /// ```
    pub fn new(queue: Vec<Result<ChatResult, ChatClientError>>, calls: Vec<(Model, Vec<ChatMessage>, Vec<Tool>)>) -> Self {
        Self {
            queue: Arc::new(Mutex::new(queue)),
            calls: Arc::new(Mutex::new(calls)),
        }
    }

    /// Pushes a result to the queue.
    ///
    /// # Arguments
    ///
    /// * `result` - The result to push to the queue.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use code_g::client::{ChatClient, ChatResult, ChatMessage, Model, Tool};
    /// use std::sync::{Arc, Mutex};
    ///
    /// let mock_client = MockChatClient::new(vec![], vec![]);
    /// mock_client.push(Ok(ChatResult::Message {
    ///     content: "Mock response".to_string(),
    ///     turn_over: true,
    /// }));
    /// ```
    pub fn push(&self, result: Result<ChatResult, ChatClientError>) {
        self.queue.lock().unwrap().push(result);
    }

    /// Returns the calls that the chat client has made.
    ///
    /// # Returns
    ///
    /// A vector of tuples that contain the model, chat messages, and tools that the chat client has made.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use code_g::client::{ChatClient, ChatResult, ChatMessage, Model, Tool};
    /// use std::sync::{Arc, Mutex};
    ///
    /// let mock_client = MockChatClient::new(vec![], vec![]);
    /// mock_client.push(Ok(ChatResult::Message {
    ///     content: "Mock response".to_string(),
    ///     turn_over: true,
    /// }));
    /// 
    /// let calls = mock_client.calls();
    ///
    /// assert_eq!(calls.len(), 1);
    /// assert_eq!(calls[0].0, Model::OpenAi(OpenAiModel::Gpt4o));
    /// assert_eq!(calls[0].1, vec![ChatMessage::User { content: "Hello, how are you?".to_string() }]);
    /// assert_eq!(calls[0].2, vec![]);
    /// ```
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
        self.calls.lock().unwrap().push((model.clone(), chat_history.to_vec(), tools.to_vec()));

        // Return the next result from the queue
        self.queue.lock().unwrap().pop().unwrap_or(Err(ChatClientError::Other("No more results in queue".to_string())))
    }
}