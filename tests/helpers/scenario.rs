#![allow(dead_code)]

use crate::helpers::mocks::{
    chat_client::MockChatClient,
    event_handler::MockEventHandler,
    tool_registry::{MockTool, MockToolRegistry},
};
use code_g::client::error::ChatClientError;
use code_g::client::models::{ChatMessage, ChatResult, Model, Parameters, Tool, ToolCall};
use code_g::session::event::Event;
use code_g::session::session::ChatSession;
use code_g::session::system_prompt::SystemPromptConfig;
use code_g::tools::traits::Tool as ToolTrait;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

/// A lightweight builder for end-to-end chat session scenarios.
pub struct ScenarioBuilder {
    system_prompt_config: SystemPromptConfig,
    user_inputs: Vec<String>,
    approval_inputs: Vec<String>,
    queued_results: Vec<Result<ChatResult, ChatClientError>>,
    tools: Vec<Box<dyn ToolTrait>>,
}

impl Default for ScenarioBuilder {
    fn default() -> Self {
        Self {
            system_prompt_config: SystemPromptConfig::Default,
            user_inputs: Vec::new(),
            approval_inputs: Vec::new(),
            queued_results: Vec::new(),
            tools: Vec::new(),
        }
    }
}

impl ScenarioBuilder {
    /// Create a new ScenarioBuilder.
    ///
    /// # Returns
    ///
    /// A new ScenarioBuilder.
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the system prompt config.
    ///
    /// # Arguments
    ///
    /// * `config` - The system prompt config to set.
    ///
    /// # Returns
    ///
    /// A ScenarioBuilder with the system prompt config set.
    pub fn with_system_prompt_config(mut self, config: SystemPromptConfig) -> Self {
        self.system_prompt_config = config;
        self
    }

    /// Queue a user message.
    ///
    /// # Arguments
    ///
    /// * `inputs` - The inputs to queue.
    ///
    /// # Returns
    ///
    /// A ScenarioBuilder with the user inputs queued.
    pub fn inputs<I, S>(mut self, inputs: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        self.user_inputs = inputs.into_iter().map(Into::into).collect();
        self
    }

    pub fn approvals<I, S>(mut self, approvals: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        self.approval_inputs = approvals.into_iter().map(Into::into).collect();
        self
    }

    /// Queue a plain assistant message.
    ///
    /// # Arguments
    ///
    /// * `content` - The content of the message.
    /// * `turn_over` - Whether the turn is over.
    ///
    /// # Returns
    ///
    /// A ScenarioBuilder with the message queued.
    pub fn then_message<S: Into<String>>(mut self, content: S, turn_over: bool) -> Self {
        self.queued_results.push(Ok(ChatResult::Message {
            content: content.into(),
            turn_over,
        }));
        self
    }

    /// Queue an assistant tool call.
    ///
    /// # Arguments
    ///
    /// * `id` - The id of the tool call.
    /// * `name` - The name of the tool.
    /// * `arguments` - The arguments of the tool call.
    ///
    /// # Returns
    ///
    /// A ScenarioBuilder with the tool call queued.
    pub fn then_tool_call(
        mut self,
        id: impl Into<String>,
        name: impl Into<String>,
        arguments: HashMap<String, String>,
    ) -> Self {
        self.queued_results
            .push(Ok(ChatResult::ToolCalls(vec![ToolCall {
                id: id.into(),
                name: name.into(),
                arguments,
            }])));
        self
    }

    /// Add a mock tool available to the registry.
    ///
    /// # Arguments
    ///
    /// * `name` - The name of the tool.
    /// * `description` - The description of the tool.
    /// * `parameters` - The parameters of the tool.
    /// * `strict` - Whether the tool is strict.
    /// * `requires_approval` - Whether the tool requires approval.
    /// * `approval_message` - The message to return if the tool requires approval.
    /// * `return_value` - The value to return if the tool is called.
    ///
    /// # Returns
    ///
    /// A ScenarioBuilder with the mock tool added.
    pub fn add_mock_tool(
        mut self,
        name: impl Into<String>,
        description: impl Into<String>,
        parameters: Parameters,
        strict: bool,
        requires_approval: bool,
        approval_message: impl Into<String>,
        declined_message: impl Into<String>,
        return_value: impl Into<String>,
    ) -> Self {
        let tool = Box::new(MockTool::new(
            name.into(),
            description.into(),
            parameters,
            strict,
            requires_approval,
            approval_message.into(),
            declined_message.into(),
            return_value.into(),
        ));
        self.tools.push(tool);
        self
    }

    /// Runs the scenario end-to-end and returns artifacts for assertions.
    ///
    /// # Returns
    ///
    /// A ScenarioResult containing the events, client calls, and tool calls.
    ///
    /// # Panics
    ///
    /// Panics if the events queue is locked.
    pub async fn run(self) -> ScenarioResult {
        let events = Arc::new(Mutex::new(vec![]));
        let event_handler =
            MockEventHandler::new(events.clone(), self.user_inputs, self.approval_inputs);

        let client_calls: Arc<Mutex<Vec<(Model, Vec<ChatMessage>, Vec<Tool>)>>> =
            Arc::new(Mutex::new(vec![]));

        let chat_client = MockChatClient::new(self.queued_results, client_calls.clone());

        let registry_calls: Arc<Mutex<Vec<(String, HashMap<String, String>)>>> =
            Arc::new(Mutex::new(vec![]));
        let tool_registry = MockToolRegistry::new(self.tools, registry_calls.clone());

        let mut session = ChatSession::new(
            Box::new(chat_client.clone()),
            Box::new(tool_registry),
            Box::new(event_handler),
            self.system_prompt_config,
        );

        // Drive the session by running the loop until "exit" (MockEventHandler appends it).
        let _ = session.run().await;

        ScenarioResult {
            events: events.lock().unwrap().clone(),
            client_calls,
            tool_calls: registry_calls,
        }
    }
}

#[derive(Clone)]
pub struct ScenarioResult {
    pub events: Vec<Event>,
    pub client_calls: Arc<Mutex<Vec<(Model, Vec<ChatMessage>, Vec<Tool>)>>>,
    pub tool_calls: Arc<Mutex<Vec<(String, HashMap<String, String>)>>>,
}

impl ScenarioResult {
    /// Returns the last client call.
    ///
    /// # Returns
    ///
    /// A tuple containing the model, chat messages, and tools that the client called last.
    ///
    /// # Panics
    ///
    /// Panics if the client calls are empty.
    pub fn last_client_call(&self) -> (Model, Vec<ChatMessage>, Vec<Tool>) {
        self.client_calls.lock().unwrap().last().unwrap().clone()
    }
}
