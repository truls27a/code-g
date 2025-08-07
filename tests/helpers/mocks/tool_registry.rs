use code_g::client::model::Tool as OpenAiTool;
use std::collections::HashMap;

/// Mock implementation of Registry for testing purposes.
///
/// This mock registry allows you to configure predefined tools and responses for testing
/// without using the actual Registry implementation. It supports configuring tool responses
/// and can be used to simulate various tool execution scenarios.
///
/// # Examples
///
/// ```rust
/// use tests::helpers::mocks::tool_registry::MockToolRegistry;
/// use std::collections::HashMap;
///
/// // Create a mock registry with predefined response
/// let mut mock = MockToolRegistry::new();
/// mock.set_tool_response("read_file", Ok("File contents".to_string()));
///
/// // Use it like a real registry
/// let mut args = HashMap::new();
/// args.insert("path".to_string(), "test.txt".to_string());
/// let result = mock.call_tool("read_file", args).unwrap();
/// assert_eq!(result, "File contents");
/// ```
#[derive(Debug, Clone)]
pub struct MockToolRegistry {
    tool_responses: HashMap<String, Result<String, String>>,
    tools: Vec<MockTool>,
    call_log: std::sync::Arc<std::sync::Mutex<Vec<(String, HashMap<String, String>)>>>,
}

/// Mock tool implementation for testing
#[derive(Debug, Clone)]
pub struct MockTool {
    name: String,
    description: String,
    requires_approval: bool,
}

impl MockToolRegistry {
    /// Creates a new mock tool registry with no predefined responses.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use tests::helpers::mocks::tool_registry::MockToolRegistry;
    ///
    /// let mock = MockToolRegistry::new();
    /// assert_eq!(mock.len(), 0);
    /// ```
    pub fn new() -> Self {
        Self {
            tool_responses: HashMap::new(),
            tools: vec![],
            call_log: std::sync::Arc::new(std::sync::Mutex::new(vec![])),
        }
    }

    /// Creates a mock registry with common read-only tools pre-configured.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use tests::helpers::mocks::tool_registry::MockToolRegistry;
    ///
    /// let mock = MockToolRegistry::with_read_only_tools();
    /// assert_eq!(mock.len(), 2);
    /// ```
    pub fn with_read_only_tools() -> Self {
        let mut mock = Self::new();
        mock.add_tool("read_file", "Read file contents", false);
        mock.add_tool("search_files", "Search for files", false);
        mock
    }

    /// Creates a mock registry with all common tools pre-configured.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use tests::helpers::mocks::tool_registry::MockToolRegistry;
    ///
    /// let mock = MockToolRegistry::with_all_tools();
    /// assert_eq!(mock.len(), 5);
    /// ```
    pub fn with_all_tools() -> Self {
        let mut mock = Self::new();
        mock.add_tool("read_file", "Read file contents", false);
        mock.add_tool("search_files", "Search for files", false);
        mock.add_tool("write_file", "Write file contents", true);
        mock.add_tool("edit_file", "Edit file contents", true);
        mock.add_tool("execute_command", "Execute shell command", true);
        mock
    }

    /// Adds a mock tool to the registry.
    ///
    /// # Arguments
    ///
    /// * `name` - The name of the tool
    /// * `description` - A description of what the tool does
    /// * `requires_approval` - Whether the tool requires user approval
    ///
    /// # Examples
    ///
    /// ```rust
    /// use tests::helpers::mocks::tool_registry::MockToolRegistry;
    ///
    /// let mut mock = MockToolRegistry::new();
    /// mock.add_tool("my_tool", "Does something useful", false);
    /// assert_eq!(mock.len(), 1);
    /// ```
    pub fn add_tool(&mut self, name: &str, description: &str, requires_approval: bool) {
        self.tools.push(MockTool {
            name: name.to_string(),
            description: description.to_string(),
            requires_approval,
        });
    }

    /// Sets the response for a specific tool call.
    ///
    /// # Arguments
    ///
    /// * `tool_name` - The name of the tool to configure
    /// * `response` - The response to return when this tool is called
    ///
    /// # Examples
    ///
    /// ```rust
    /// use tests::helpers::mocks::tool_registry::MockToolRegistry;
    ///
    /// let mut mock = MockToolRegistry::new();
    /// mock.set_tool_response("read_file", Ok("File contents".to_string()));
    /// mock.set_tool_response("bad_tool", Err("Tool failed".to_string()));
    /// ```
    pub fn set_tool_response(&mut self, tool_name: &str, response: Result<String, String>) {
        self.tool_responses.insert(tool_name.to_string(), response);
    }

    /// Executes a tool by name with the provided arguments.
    ///
    /// Returns the predefined response for the tool, or an error if the tool
    /// is not found or no response was configured.
    ///
    /// # Arguments
    ///
    /// * `tool_name` - The name of the tool to execute
    /// * `args` - A HashMap containing the arguments to pass to the tool
    ///
    /// # Returns
    ///
    /// The configured response for the tool
    ///
    /// # Examples
    ///
    /// ```rust
    /// use tests::helpers::mocks::tool_registry::MockToolRegistry;
    /// use std::collections::HashMap;
    ///
    /// let mut mock = MockToolRegistry::new();
    /// mock.set_tool_response("read_file", Ok("File contents".to_string()));
    ///
    /// let mut args = HashMap::new();
    /// args.insert("path".to_string(), "test.txt".to_string());
    /// let result = mock.call_tool("read_file", args).unwrap();
    /// assert_eq!(result, "File contents");
    /// ```
    pub fn call_tool(
        &self,
        tool_name: &str,
        args: HashMap<String, String>,
    ) -> Result<String, String> {
        // Log the call
        if let Ok(mut log) = self.call_log.lock() {
            log.push((tool_name.to_string(), args.clone()));
        }

        // Return the configured response
        if let Some(response) = self.tool_responses.get(tool_name) {
            response.clone()
        } else {
            Err(format!(
                "Tool {} not found or no response configured",
                tool_name
            ))
        }
    }

    /// Returns a reference to the mock tool by name if it exists.
    ///
    /// # Arguments
    ///
    /// * `tool_name` - The name of the tool to find
    ///
    /// # Returns
    ///
    /// An `Option` containing a reference to the mock tool if found
    ///
    /// # Examples
    ///
    /// ```rust
    /// use tests::helpers::mocks::tool_registry::MockToolRegistry;
    ///
    /// let mock = MockToolRegistry::with_all_tools();
    /// if let Some(tool) = mock.get_tool("read_file") {
    ///     assert_eq!(tool.name(), "read_file");
    /// }
    /// ```
    pub fn get_tool(&self, tool_name: &str) -> Option<&MockTool> {
        self.tools.iter().find(|t| t.name() == tool_name)
    }

    /// Returns all tool names in the registry.
    ///
    /// # Returns
    ///
    /// A vector of tool names
    ///
    /// # Examples
    ///
    /// ```rust
    /// use tests::helpers::mocks::tool_registry::MockToolRegistry;
    ///
    /// let mock = MockToolRegistry::with_read_only_tools();
    /// let names = mock.get_tool_names();
    /// assert_eq!(names, vec!["read_file", "search_files"]);
    /// ```
    pub fn get_tool_names(&self) -> Vec<String> {
        self.tools.iter().map(|t| t.name()).collect()
    }

    /// Converts all tools to OpenAI-compatible tool format.
    ///
    /// # Returns
    ///
    /// A vector of OpenAI tool definitions
    ///
    /// # Examples
    ///
    /// ```rust
    /// use tests::helpers::mocks::tool_registry::MockToolRegistry;
    ///
    /// let mock = MockToolRegistry::with_all_tools();
    /// let openai_tools = mock.to_openai_tools();
    /// assert_eq!(openai_tools.len(), 5);
    /// ```
    pub fn to_openai_tools(&self) -> Vec<OpenAiTool> {
        self.tools
            .iter()
            .map(|tool| tool.to_openai_tool())
            .collect()
    }

    /// Returns the number of tools in the registry.
    ///
    /// # Returns
    ///
    /// The count of tools currently registered
    ///
    /// # Examples
    ///
    /// ```rust
    /// use tests::helpers::mocks::tool_registry::MockToolRegistry;
    ///
    /// let mock = MockToolRegistry::new();
    /// assert_eq!(mock.len(), 0);
    ///
    /// let mock = MockToolRegistry::with_all_tools();
    /// assert_eq!(mock.len(), 5);
    /// ```
    pub fn len(&self) -> usize {
        self.tools.len()
    }

    /// Returns true if the registry has no tools.
    ///
    /// # Returns
    ///
    /// True if the registry is empty
    ///
    /// # Examples
    ///
    /// ```rust
    /// use tests::helpers::mocks::tool_registry::MockToolRegistry;
    ///
    /// let mock = MockToolRegistry::new();
    /// assert!(mock.is_empty());
    /// ```
    pub fn is_empty(&self) -> bool {
        self.tools.is_empty()
    }

    /// Returns a copy of the call log for testing verification.
    ///
    /// Each entry contains the tool name and arguments that were passed.
    ///
    /// # Returns
    ///
    /// A vector of tuples containing (tool_name, arguments)
    ///
    /// # Examples
    ///
    /// ```rust
    /// use tests::helpers::mocks::tool_registry::MockToolRegistry;
    /// use std::collections::HashMap;
    ///
    /// let mut mock = MockToolRegistry::new();
    /// mock.set_tool_response("read_file", Ok("contents".to_string()));
    ///
    /// let mut args = HashMap::new();
    /// args.insert("path".to_string(), "test.txt".to_string());
    /// mock.call_tool("read_file", args.clone()).unwrap();
    ///
    /// let log = mock.get_call_log();
    /// assert_eq!(log.len(), 1);
    /// assert_eq!(log[0].0, "read_file");
    /// assert_eq!(log[0].1, args);
    /// ```
    pub fn get_call_log(&self) -> Vec<(String, HashMap<String, String>)> {
        self.call_log.lock().unwrap().clone()
    }

    /// Clears the call log.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use tests::helpers::mocks::tool_registry::MockToolRegistry;
    /// use std::collections::HashMap;
    ///
    /// let mut mock = MockToolRegistry::new();
    /// mock.set_tool_response("read_file", Ok("contents".to_string()));
    /// mock.call_tool("read_file", HashMap::new()).unwrap();
    ///
    /// assert_eq!(mock.get_call_log().len(), 1);
    /// mock.clear_call_log();
    /// assert_eq!(mock.get_call_log().len(), 0);
    /// ```
    pub fn clear_call_log(&self) {
        self.call_log.lock().unwrap().clear();
    }

    /// Returns the number of times tools have been called.
    ///
    /// # Returns
    ///
    /// The total number of tool calls
    ///
    /// # Examples
    ///
    /// ```rust
    /// use tests::helpers::mocks::tool_registry::MockToolRegistry;
    /// use std::collections::HashMap;
    ///
    /// let mut mock = MockToolRegistry::new();
    /// mock.set_tool_response("read_file", Ok("contents".to_string()));
    ///
    /// assert_eq!(mock.call_count(), 0);
    /// mock.call_tool("read_file", HashMap::new()).unwrap();
    /// assert_eq!(mock.call_count(), 1);
    /// ```
    pub fn call_count(&self) -> usize {
        self.call_log.lock().unwrap().len()
    }
}

impl MockTool {
    /// Returns the name of the tool.
    pub fn name(&self) -> String {
        self.name.clone()
    }

    /// Returns the description of the tool.
    pub fn description(&self) -> String {
        self.description.clone()
    }

    /// Returns whether the tool requires approval.
    pub fn requires_approval(&self) -> bool {
        self.requires_approval
    }

    /// Converts the mock tool to OpenAI tool format.
    pub fn to_openai_tool(&self) -> OpenAiTool {
        use code_g::client::model::{Function, Parameters, ToolType};

        OpenAiTool {
            tool_type: ToolType::Function,
            function: Function {
                name: self.name.clone(),
                description: self.description.clone(),
                parameters: Parameters {
                    param_type: "object".to_string(),
                    properties: HashMap::new(),
                    required: vec![],
                    additional_properties: false,
                },
                strict: true,
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_creates_empty_registry() {
        let mock = MockToolRegistry::new();
        assert_eq!(mock.len(), 0);
        assert!(mock.is_empty());
    }

    #[test]
    fn with_read_only_tools_creates_registry_with_read_tools() {
        let mock = MockToolRegistry::with_read_only_tools();
        assert_eq!(mock.len(), 2);

        let names = mock.get_tool_names();
        assert!(names.contains(&"read_file".to_string()));
        assert!(names.contains(&"search_files".to_string()));
    }

    #[test]
    fn with_all_tools_creates_registry_with_all_tools() {
        let mock = MockToolRegistry::with_all_tools();
        assert_eq!(mock.len(), 5);

        let names = mock.get_tool_names();
        assert!(names.contains(&"read_file".to_string()));
        assert!(names.contains(&"search_files".to_string()));
        assert!(names.contains(&"write_file".to_string()));
        assert!(names.contains(&"edit_file".to_string()));
        assert!(names.contains(&"execute_command".to_string()));
    }

    #[test]
    fn add_tool_adds_tool_to_registry() {
        let mut mock = MockToolRegistry::new();
        mock.add_tool("test_tool", "A test tool", false);

        assert_eq!(mock.len(), 1);
        assert!(!mock.is_empty());

        let tool = mock.get_tool("test_tool").unwrap();
        assert_eq!(tool.name(), "test_tool");
        assert_eq!(tool.description(), "A test tool");
        assert!(!tool.requires_approval());
    }

    #[test]
    fn call_tool_returns_configured_response() {
        let mut mock = MockToolRegistry::new();
        mock.set_tool_response("test_tool", Ok("success".to_string()));

        let result = mock.call_tool("test_tool", HashMap::new()).unwrap();
        assert_eq!(result, "success");
    }

    #[test]
    fn call_tool_returns_error_for_unconfigured_tool() {
        let mock = MockToolRegistry::new();

        let result = mock.call_tool("unknown_tool", HashMap::new());
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Tool unknown_tool not found"));
    }

    #[test]
    fn call_tool_logs_calls() {
        let mut mock = MockToolRegistry::new();
        mock.set_tool_response("test_tool", Ok("success".to_string()));

        let mut args = HashMap::new();
        args.insert("key".to_string(), "value".to_string());

        mock.call_tool("test_tool", args.clone()).unwrap();

        let log = mock.get_call_log();
        assert_eq!(log.len(), 1);
        assert_eq!(log[0].0, "test_tool");
        assert_eq!(log[0].1, args);

        assert_eq!(mock.call_count(), 1);
    }

    #[test]
    fn clear_call_log_empties_log() {
        let mut mock = MockToolRegistry::new();
        mock.set_tool_response("test_tool", Ok("success".to_string()));

        mock.call_tool("test_tool", HashMap::new()).unwrap();
        assert_eq!(mock.call_count(), 1);

        mock.clear_call_log();
        assert_eq!(mock.call_count(), 0);
        assert!(mock.get_call_log().is_empty());
    }

    #[test]
    fn to_openai_tools_converts_all_tools() {
        let mock = MockToolRegistry::with_read_only_tools();
        let openai_tools = mock.to_openai_tools();

        assert_eq!(openai_tools.len(), 2);
        assert_eq!(openai_tools[0].function.name, "read_file");
        assert_eq!(openai_tools[1].function.name, "search_files");
    }
}
