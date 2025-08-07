use code_g::client::model::{Function, Parameters, Tool as OpenAiTool, ToolType};
use code_g::tools::traits::{Tool, ToolRegistry};
use std::cell::RefCell;
use std::collections::HashMap;

/// A mock implementation of ToolRegistry for testing purposes.
///
/// MockToolRegistry allows tests to control the behavior of tool execution
/// and verify that tools are called with the expected arguments.
///
/// # Examples
///
/// ```rust
/// use code_g::tests::helpers::mocks::tool_registry::MockToolRegistry;
/// use std::collections::HashMap;
///
/// let mut mock = MockToolRegistry::new();
/// mock.expect_call_tool("read_file", Ok("file content".to_string()));
///
/// let mut args = HashMap::new();
/// args.insert("path".to_string(), "test.txt".to_string());
/// let result = mock.call_tool("read_file", args);
/// assert_eq!(result, Ok("file content".to_string()));
/// ```
pub struct MockToolRegistry {
    /// Expected tool calls with their return values
    expected_calls: HashMap<String, Result<String, String>>,
    /// Actual calls made to the registry
    actual_calls: RefCell<Vec<(String, HashMap<String, String>)>>,
    /// Tools to return from get_tools and get_tool
    tools: Vec<Box<dyn Tool>>,
    /// OpenAI tools to return from to_openai_tools
    openai_tools: Vec<OpenAiTool>,
}

impl MockToolRegistry {
    /// Creates a new empty mock tool registry.
    pub fn new() -> Self {
        Self {
            expected_calls: HashMap::new(),
            actual_calls: RefCell::new(Vec::new()),
            tools: Vec::new(),
            openai_tools: Vec::new(),
        }
    }

    /// Sets up an expectation for a tool call.
    ///
    /// # Arguments
    ///
    /// * `tool_name` - The name of the tool expected to be called
    /// * `result` - The result to return when the tool is called
    pub fn expect_call_tool(&mut self, tool_name: &str, result: Result<String, String>) {
        self.expected_calls.insert(tool_name.to_string(), result);
    }

    /// Returns the actual calls made to the registry.
    ///
    /// This is useful for verifying that the expected tool calls were made
    /// with the correct arguments.
    ///
    /// # Returns
    ///
    /// A vector of tuples containing (tool_name, arguments) for each call made.
    pub fn get_actual_calls(&self) -> Vec<(String, HashMap<String, String>)> {
        self.actual_calls.borrow().clone()
    }

    pub fn set_openai_tools(&mut self, tools: Vec<OpenAiTool>) {
        self.openai_tools = tools;
    }
}

impl ToolRegistry for MockToolRegistry {
    fn call_tool(&self, tool_name: &str, args: HashMap<String, String>) -> Result<String, String> {
        // Record the call
        self.actual_calls
            .borrow_mut()
            .push((tool_name.to_string(), args));

        // Return the expected result
        if let Some(result) = self.expected_calls.get(tool_name) {
            result.clone()
        } else {
            Err(format!("Unexpected tool call: {}", tool_name))
        }
    }

    fn to_openai_tools(&self) -> Vec<OpenAiTool> {
        self.openai_tools.clone()
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_creates_empty_mock() {
        let mock = MockToolRegistry::new();
        assert_eq!(mock.len(), 0);
        assert_eq!(mock.get_actual_calls().len(), 0);
        assert_eq!(mock.to_openai_tools().len(), 0);
    }

    #[test]
    fn expect_call_tool_sets_up_expectation() {
        let mut mock = MockToolRegistry::new();
        mock.expect_call_tool("test_tool", Ok("success".to_string()));

        let args = HashMap::new();
        let result = mock.call_tool("test_tool", args);
        assert_eq!(result, Ok("success".to_string()));
    }

    #[test]
    fn call_tool_returns_error_for_unexpected_calls() {
        let mock = MockToolRegistry::new();
        let args = HashMap::new();
        let result = mock.call_tool("unknown_tool", args);

        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Unexpected tool call: unknown_tool");
    }

    #[test]
    fn set_openai_tools_sets_tools() {
        let mut mock = MockToolRegistry::new();
        let tools = vec![OpenAiTool {
            tool_type: ToolType::Function,
            function: Function {
                name: "test_tool".to_string(),
                description: "Test tool".to_string(),
                parameters: Parameters {
                    param_type: "object".to_string(),
                    properties: HashMap::new(),
                    required: vec![],
                    additional_properties: false,
                },
                strict: false,
            },
        }];
        mock.set_openai_tools(tools);
        assert_eq!(mock.to_openai_tools().len(), 1);
        assert_eq!(mock.to_openai_tools()[0].function.name, "test_tool");
        assert_eq!(mock.to_openai_tools()[0].function.description, "Test tool");
        assert_eq!(mock.to_openai_tools()[0].function.parameters.param_type, "object");
        assert_eq!(mock.to_openai_tools()[0].function.parameters.required, vec!["test_tool".to_string()]);
        assert_eq!(mock.to_openai_tools()[0].function.parameters.additional_properties, false);
        assert_eq!(mock.to_openai_tools()[0].function.strict, false);
    }
}
