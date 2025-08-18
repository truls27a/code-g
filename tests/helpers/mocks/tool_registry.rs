#![allow(dead_code)]

use code_g::client::model::{Parameters, Tool as ToolModel};
use code_g::tools::traits::{Tool, ToolRegistry};
use code_g::tui::model::Status;
use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

/// A mock tool for testing purposes.
///
/// This tool is used to test the tool registry and tool execution.
/// It is not intended to be used in production.
#[derive(Clone)]
pub struct MockTool {
    name: String,
    description: String,
    parameters: Parameters,
    strict: bool,
    requires_approval: bool,
    approval_message: String,
    calls: Arc<Mutex<Vec<HashMap<String, String>>>>,
    return_value: Arc<Mutex<String>>,
}

impl MockTool {
    /// Create a new MockTool.
    ///
    /// # Arguments
    ///
    /// * `return_value` - The value to return when the tool is called.
    ///
    /// # Returns
    ///
    /// A new `MockTool` instance.
    pub fn new(
        name: String,
        description: String,
        parameters: Parameters,
        strict: bool,
        requires_approval: bool,
        approval_message: String,
        return_value: String,
    ) -> Self {
        Self {
            name,
            description,
            parameters,
            strict,
            requires_approval,
            approval_message,
            calls: Arc::new(Mutex::new(vec![])),
            return_value: Arc::new(Mutex::new(return_value)),
        }
    }

    /// Get the calls made to the tool.
    ///
    /// # Returns
    ///
    /// A vector of `HashMap`s, each containing the arguments passed to the tool.
    pub fn calls(&self) -> Vec<HashMap<String, String>> {
        self.calls.lock().unwrap().clone()
    }

    /// Get the return value of the tool.
    ///
    /// # Returns
    ///
    /// The return value of the tool.
    pub fn return_value(&self) -> String {
        self.return_value.lock().unwrap().clone()
    }
}

impl Tool for MockTool {
    fn name(&self) -> String {
        self.name.clone()
    }

    fn description(&self) -> String {
        self.description.clone()
    }

    fn parameters(&self) -> Parameters {
        self.parameters.clone()
    }

    fn strict(&self) -> bool {
        self.strict
    }

    fn requires_approval(&self) -> bool {
        self.requires_approval
    }

    fn approval_message(&self, _args: &HashMap<String, String>) -> (String, String) {
        (self.name.clone(), self.approval_message.clone())
    }

    fn status(&self, _args: &HashMap<String, String>) -> Status {
        Status::ExecutingTool { tool_name: self.name.clone() }
    }

    fn summary_message(&self, _args: &HashMap<String, String>, result: &str) -> String {
        format!("Tool '{}' completed with result: {}", self.name, result)
    }

    fn call(&self, args: HashMap<String, String>) -> Result<String, String> {
        // Record the call
        self.calls.lock().unwrap().push(args);

        // Return the return value
        Ok(self.return_value.lock().unwrap().clone())
    }
}

/// A mock tool registry for testing purposes.
///
/// This tool registry is used to test the tool registry and tool execution.
/// It is not intended to be used in production.
///
/// # Fields
///
/// * `tools` - A vector of tools that are available in the registry.
/// * `calls` - A vector of calls to the registry.
pub struct MockToolRegistry {
    tools: Vec<Box<dyn Tool>>,
    calls: Arc<Mutex<Vec<(String, HashMap<String, String>)>>>,
}

impl MockToolRegistry {
    /// Create a new MockToolRegistry.
    ///
    /// # Arguments
    ///
    /// * `tools` - A vector of tools that are available in the registry.
    ///
    /// # Returns
    ///
    /// A new `MockToolRegistry` instance.
    pub fn new(
        tools: Vec<Box<dyn Tool>>,
        calls: Arc<Mutex<Vec<(String, HashMap<String, String>)>>>,
    ) -> Self {
        Self {
            tools,
            calls,
        }
    }

    /// Get the calls made to the registry.
    ///
    /// # Returns
    ///
    /// A vector of tuples, each containing the tool name and the arguments passed to the tool.
    pub fn calls(&self) -> Vec<(String, HashMap<String, String>)> {
        self.calls.lock().unwrap().clone()
    }
}

impl ToolRegistry for MockToolRegistry {
    fn call_tool(&self, tool_name: &str, args: HashMap<String, String>) -> Result<String, String> {
        // Record the call
        self.calls
            .lock()
            .unwrap()
            .push((tool_name.to_string(), args.clone()));

        // Find the tool
        let tool = self.tools.iter().find(|t| t.name() == tool_name);
        if let Some(tool) = tool {
            // Call the tool
            tool.call(args)
        } else {
            // Return an error if the tool is not found
            Err(format!("Tool {} not found", tool_name))
        }
    }

    fn to_tools(&self) -> Vec<ToolModel> {
        self.tools.iter().map(|t| t.to_tool()).collect()
    }

    fn len(&self) -> usize {
        self.tools.len()
    }

    fn get_tool(&self, tool_name: &str) -> Option<&Box<dyn Tool>> {
        self.tools.iter().find(|t| t.name() == tool_name)
    }

    fn get_tools(&self) -> &[Box<dyn Tool>] {
        &self.tools
    }
}
