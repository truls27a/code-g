use crate::client::model::{Function, Parameters, Tool as OpenAiTool, ToolType};
use std::collections::HashMap;


/// A trait defining the interface for tool registries.
/// 
/// The `ToolRegistry` trait provides a standardized interface for implementing
/// tool registries that can be used to manage and execute tools.
/// 
/// # Examples
/// ```rust
/// use code_g::tools::traits::ToolRegistry;
/// use code_g::tools::tool::Tool;
/// 
/// struct MyToolRegistry;
/// 
/// impl ToolRegistry for MyToolRegistry {
///     fn new() -> Self {
///         Self { tools: vec![] }
///     }
/// 
///     fn from(tools: Vec<Box<dyn Tool>>) -> Self {
///         Self { tools }
///     }
/// 
///     fn read_only_tools() -> Self {
///         Self { tools: vec![Box::new(ReadFile), Box::new(SearchFiles)] }
///     }
/// 
///     fn all_tools() -> Self {
///         Self { tools: vec![Box::new(ReadFile), Box::new(SearchFiles)] }
///     }
/// 
///     fn call_tool(&self, tool_name: &str, args: HashMap<String, String>) -> Result<String, String> {
///         // Implement the tool execution logic here
///         Ok("Tool executed successfully".to_string())
///     }
/// }
/// ```
pub trait ToolRegistry {
    /// Creates a new tool registry.
    /// 
    /// # Returns
    /// A new `ToolRegistry` instance.
    fn new() -> Self;
    
    /// Returns a tool registry with the provided tools.
    /// 
    /// # Arguments
    /// 
    /// * `tools` - A vector of tools to include in the registry.
    /// 
    /// # Returns
    /// A tool registry with the provided tools.
    fn from(tools: Vec<Box<dyn Tool>>) -> Self;

    /// Returns a read-only tool registry.
    /// 
    /// # Returns
    /// A read-only `ToolRegistry` instance.
    fn read_only_tools() -> Self;
    
    /// Returns a tool registry with all available tools.
    /// 
    /// # Returns
    /// A tool registry with all available tools.
    fn all_tools() -> Self;


    /// Executes a tool by name with the provided arguments.
    /// 
    /// Searches for a tool with the given name in the registry and executes it
    /// with the provided arguments. If the tool is not found, returns an error.
    /// 
    /// # Arguments
    /// 
    /// * `tool_name` - The name of the tool to execute.
    /// * `args` - A HashMap containing the arguments to pass to the tool.
    /// 
    /// # Returns
    /// The output from the tool execution as a String.
    /// 
    /// # Errors
    /// Returns an error if the tool is not found in the registry or if the tool
    /// execution fails.
    fn call_tool(&self, tool_name: &str, args: HashMap<String, String>) -> Result<String, String>;
}


/// A trait defining the interface for tools that can be called with arguments.
///
/// The `Tool` trait provides a standardized interface for implementing various tools
/// that can be executed with string-based arguments and return string results. Tools
/// can be converted to OpenAI tool format for integration with AI models.
///
/// # Examples
///
/// ```rust
/// use code_g::tools::tool::Tool;
/// use code_g::chat_client::model::Parameters;
/// use std::collections::HashMap;
///
/// struct MyTool;
///
/// impl Tool for MyTool {
///     fn name(&self) -> String {
///         "my_tool".to_string()
///     }
///
///     fn description(&self) -> String {
///         "A simple example tool".to_string()
///     }
///
///     fn parameters(&self) -> Parameters {
///         Parameters {
///             param_type: "object".to_string(),
///             properties: HashMap::new(),
///             required: vec![],
///             additional_properties: false,
///         }
///     }
///
///     fn strict(&self) -> bool {
///         true
///     }
///
///     fn requires_approval(&self) -> bool {
///         false
///     }
///
///     fn approval_message(&self, args: &HashMap<String, String>) -> (String, String) {
///         ("Simple Tool".to_string(), "Running my_tool".to_string())
///     }
///
///     fn call(&self, args: HashMap<String, String>) -> Result<String, String> {
///         // Implement the tool's logic here
///         Ok("Tool executed successfully".to_string())
///     }
/// }
/// ```
pub trait Tool {
    /// Returns the name of the tool.
    ///
    /// The name should be a unique identifier for the tool, typically in snake_case.
    /// This name is used for tool registration and identification in the system.
    ///
    /// # Returns
    ///
    /// A `String` containing the tool's name.
    fn name(&self) -> String;

    /// Returns a human-readable description of the tool.
    ///
    /// The description should explain what the tool does and when it should be used.
    /// This description may be shown to users or AI models to help them understand
    /// the tool's purpose.
    ///
    /// # Returns
    ///
    /// A `String` containing the tool's description.
    fn description(&self) -> String;

    /// Returns the parameters schema for the tool.
    ///
    /// Defines the expected parameters that can be passed to the tool when calling it.
    /// The parameters schema is used for validation and documentation purposes.
    ///
    /// # Returns
    ///
    /// A `Parameters` object describing the expected tool parameters.
    fn parameters(&self) -> Parameters;

    /// Returns whether the tool requires strict parameter validation.
    ///
    /// When strict mode is enabled, the tool will enforce stricter validation
    /// of the provided parameters according to the schema.
    ///
    /// # Returns
    ///
    /// `true` if strict validation is required, `false` otherwise.
    fn strict(&self) -> bool;

    /// Returns whether the tool requires user approval before execution.
    ///
    /// Tools that modify the filesystem or execute system commands should
    /// require user approval to prevent potentially dangerous operations.
    ///
    /// # Returns
    ///
    /// `true` if user approval is required, `false` otherwise.
    fn requires_approval(&self) -> bool;

    /// Generates the approval message for this tool with the given arguments.
    ///
    /// This method creates a user-friendly description of what the tool will do
    /// when executed with the provided arguments. It returns a tuple containing
    /// the operation name and detailed description.
    ///
    /// # Arguments
    ///
    /// * `args` - A HashMap containing the tool arguments as key-value string pairs.
    ///
    /// # Returns
    ///
    /// A tuple containing (operation_name, details) for display to the user.
    fn approval_message(&self, args: &HashMap<String, String>) -> (String, String);

    /// Executes the tool with the provided arguments.
    ///
    /// This is the main execution method for the tool. It receives a map of
    /// string arguments and performs the tool's operation, returning either
    /// a success result or an error message.
    ///
    /// # Arguments
    ///
    /// * `args` - A HashMap containing the tool arguments as key-value string pairs.
    ///
    /// # Returns
    ///
    /// A `Result` containing either the tool's output as a `String` on success,
    /// or an error message as a `String` on failure.
    ///
    /// # Errors
    ///
    /// Returns an error string if the tool execution fails for any reason,
    /// such as invalid arguments, I/O errors, or internal processing errors.
    fn call(&self, args: HashMap<String, String>) -> Result<String, String>;

    /// Converts the tool to OpenAI tool format.
    ///
    /// Creates an OpenAI-compatible tool representation that can be used
    /// with OpenAI's function calling API. This method provides a default
    /// implementation that constructs the tool using the other trait methods.
    ///
    /// # Returns
    ///
    /// An `OpenAiTool` object representing this tool in OpenAI's format.
    fn to_openai_tool(&self) -> OpenAiTool {
        OpenAiTool {
            tool_type: ToolType::Function,
            function: Function {
                name: self.name(),
                description: self.description(),
                parameters: self.parameters(),
                strict: self.strict(),
            },
        }
    }
}
