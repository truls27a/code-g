use crate::client::models::{Function, Parameters, Tool as ToolModel, ToolType};
use crate::tui::models::Status;
use std::collections::HashMap;


/// A trait defining the interface for tool registries.
///
/// The `ToolRegistry` trait provides a standardized interface for implementing
/// tool registries that can be used to manage and execute tools.
///
/// # Examples
/// ```rust
/// use code_g::tools::traits::ToolRegistry;
/// use code_g::tools::traits::Tool;
/// use code_g::client::model::Tool as ToolModel;
/// use std::collections::HashMap;
///
/// struct MyToolRegistry;
///
/// impl ToolRegistry for MyToolRegistry {
///     fn call_tool(&self, tool_name: &str, args: HashMap<String, String>) -> Result<String, String> {
///         // Implement the tool execution logic here
///         Ok("Tool executed successfully".to_string())
///     }
///
///     fn to_tools(&self) -> Vec<ToolModel> {
///         // Implement the conversion logic here
///         vec![]
///     }
///
///     fn len(&self) -> usize {
///         // Implement the logic to return the number of tools
///         0
///     }
///
///     fn get_tool(&self, tool_name: &str) -> Option<&Box<dyn Tool>> {
///         // Implement the logic to return a tool by name
///         None
///     }
///
///     fn get_tools(&self) -> &[Box<dyn Tool>] {
///         // Implement the logic to return all tools  
///         &[]
///     }
/// }
/// ```
pub trait ToolRegistry {
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

    /// Converts all tools in the registry to tool format.
    ///
    /// This is useful when integrating with AI models that support tool calling.
    ///
    /// # Returns
    /// A vector of tool definitions.
    fn to_tools(&self) -> Vec<ToolModel>;

    /// Returns the number of tools in the registry.
    ///
    /// # Returns
    /// The count of tools currently registered.
    fn len(&self) -> usize;

    /// Returns a tool by name.
    ///
    /// # Arguments
    /// * `tool_name` - The name of the tool to find.
    ///
    /// # Returns
    /// An `Option` containing a reference to the tool if found, or `None` if not found.
    fn get_tool(&self, tool_name: &str) -> Option<&Box<dyn Tool>>;

    /// Returns a reference to all tools in the registry.
    ///
    /// # Returns
    /// A slice of references to all tools in the registry.
    fn get_tools(&self) -> &[Box<dyn Tool>];
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
/// use code_g::tools::traits::Tool;
/// use code_g::client::model::Parameters;
/// use code_g::tui::model::Status;
/// use std::collections::HashMap;
///
/// #[derive(Clone)]
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
///     fn approval_message(&self, args: &HashMap<String, String>) -> String {
///         "CodeG wants to use tool".to_string()
///     }
///
///     fn declined_message(&self, args: &HashMap<String, String>) -> String {
///         "Tool was declined by user".to_string()
///     }
///
///     fn status(&self, args: &HashMap<String, String>) -> Status {
///         Status::ExecutingTool { tool_name: self.name() }
///     }
///
///
///     fn summary_message(&self, args: &HashMap<String, String>, result: &str) -> String {
///         format!("Tool '{}' completed with result: {}", self.name(), result)
///     }
///
///     fn call(&self, args: HashMap<String, String>) -> Result<String, String> {
///         // Implement the tool's logic here
///         Ok("Tool executed successfully".to_string())
///     }
/// }
/// ```
pub trait Tool: ToolClone {
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
    /// A `String` containing the approval message.
    fn approval_message(&self, args: &HashMap<String, String>) -> String;

    /// Generates the declined message for this tool with the given arguments.
    ///
    /// This method creates a user-friendly description indicating that the tool
    /// was declined by the user and what it was trying to do.
    ///
    /// # Arguments
    ///
    /// * `args` - A HashMap containing the tool arguments as key-value string pairs.
    ///
    /// # Returns
    ///
    /// A `String` containing the declined message.
    fn declined_message(&self, args: &HashMap<String, String>) -> String;

    /// Generates the TUI status for this tool with the given arguments.
    ///
    /// This method creates a user-friendly description of what the tool is doing
    /// when executed with the provided arguments. It returns a status that can
    /// be displayed to the user to provide feedback on the tool's progress.
    ///
    /// # Arguments
    ///
    /// * `args` - A HashMap containing the tool arguments as key-value string pairs.
    ///
    /// # Returns
    ///
    /// A `Status` containing the status.
    fn status(&self, args: &HashMap<String, String>) -> Status;

    /// Generates the summary message for this tool with the given arguments.
    ///
    /// This method creates a user-friendly description of what the tool did
    /// when executed with the provided arguments. It returns a string that can
    /// be displayed to the user to provide feedback on the tool's result.
    ///
    /// # Arguments
    ///
    /// * `args` - A HashMap containing the tool arguments as key-value string pairs.
    /// * `result` - A `String` containing the result of the tool execution.
    ///
    /// # Returns
    ///
    /// A `String` containing the summary message.
    fn summary_message(&self, args: &HashMap<String, String>, result: &str) -> String;

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

    /// Converts the tool to tool format.
    ///
    /// This is useful when integrating with AI models that support tool calling.
    ///
    /// # Returns
    ///
    /// A `Tool` object representing this tool.
    fn to_tool(&self) -> ToolModel {
        ToolModel {
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

/// Enables cloning of boxed trait objects implementing `Tool`.
pub trait ToolClone {
    fn box_clone(&self) -> Box<dyn Tool>;
}

impl<T> ToolClone for T
where
    T: 'static + Tool + Clone,
{
    fn box_clone(&self) -> Box<dyn Tool> {
        Box::new(self.clone())
    }
}

impl Clone for Box<dyn Tool> {
    fn clone(&self) -> Box<dyn Tool> {
        self.box_clone()
    }
}
