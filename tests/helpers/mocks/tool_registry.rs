use code_g::tools::traits::{Tool, ToolRegistry};
use code_g::client::model::{Tool as ToolModel, Parameters, Property};
use std::{collections::HashMap, sync::{Arc, Mutex}};

/// A mock tool for testing purposes.
/// 
/// This tool is used to test the tool registry and tool execution.
/// It is not intended to be used in production.
pub struct MockTool {
    calls: Arc<Mutex<Vec<HashMap<String, String>>>>,
    return_value: Arc<Mutex<String>>,
}

impl MockTool {
    pub fn new(return_value: String) -> Self {
        Self { calls: Arc::new(Mutex::new(vec![])), return_value: Arc::new(Mutex::new(return_value)) }
    }

    pub fn calls(&self) -> Vec<HashMap<String, String>> {
        self.calls.lock().unwrap().clone()
    }

    pub fn return_value(&self) -> String {
        self.return_value.lock().unwrap().clone()
    }
}

impl Tool for MockTool {
    fn name(&self) -> String {
        "mock_tool".to_string()
    }

    fn description(&self) -> String {
        "Mock tool".to_string()
    }

    fn parameters(&self) -> Parameters {
        Parameters {
            param_type: "object".to_string(),
            properties: HashMap::from([(
                "mock_param".to_string(),
                Property {
                    prop_type: "string".to_string(),
                    description: "Mock parameter".to_string(),
                },
            )]),
            required: vec!["mock_param".to_string()],
            additional_properties: false,
        }
    }

    fn strict(&self) -> bool {
        true
    }

    fn requires_approval(&self) -> bool {
        false
    }

    fn approval_message(&self, _args: &HashMap<String, String>) -> (String, String) {
        ("Mock Tool".to_string(), "Mock approval message".to_string())
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
    pub fn new(tools: Vec<Box<dyn Tool>>) -> Self {
        Self { tools, calls: Arc::new(Mutex::new(vec![])) }
    }

    pub fn calls(&self) -> Vec<(String, HashMap<String, String>)> {
        self.calls.lock().unwrap().clone()
    }
}

impl ToolRegistry for MockToolRegistry {
    fn call_tool(&self, tool_name: &str, args: HashMap<String, String>) -> Result<String, String> {
        // Record the call
        self.calls.lock().unwrap().push((tool_name.to_string(), args.clone()));

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