use crate::client::model::Tool as OpenAiTool;
use crate::tools::edit_file::EditFile;
use crate::tools::execute_command::ExecuteCommand;
use crate::tools::read_file::ReadFile;
use crate::tools::search_files::SearchFiles;
use crate::tools::traits::Tool;
use crate::tools::write_file::WriteFile;
use std::collections::HashMap;

/// A registry for managing and executing tools.
///
/// The Registry acts as a central container for different tools that can be called
/// dynamically by name. It provides convenient factory methods for creating registries
/// with different tool combinations (read-only tools, all tools, or custom sets).
/// Tools can be executed through the registry and converted to OpenAI-compatible formats.
///
/// # Examples
///
/// ```rust
/// use code_g::tools::registry::Registry;
///
/// // Create a registry with all available tools
/// let registry = Registry::all_tools();
///
/// // Create a registry with only read-only tools
/// let read_only = Registry::read_only_tools();
///
/// // Execute a tool
/// let mut args = std::collections::HashMap::new();
/// args.insert("path".to_string(), "example.txt".to_string());
/// let result = registry.call_tool("read_file", args);
/// ```
pub struct Registry {
    tools: Vec<Box<dyn Tool>>,
}

impl Registry {
    /// Creates a new empty registry with no tools.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use code_g::tools::registry::Registry;
    ///
    /// let registry = Registry::new();
    ///
    /// assert_eq!(registry.len(), 0);
    /// ```
    pub fn new() -> Self {
        Self { tools: vec![] }
    }

    /// Creates a registry with the provided tools.
    ///
    /// # Arguments
    ///
    /// * `tools` - A vector of boxed tools to include in the registry.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use code_g::tools::registry::Registry;
    /// use code_g::tools::read_file::ReadFile;
    /// use code_g::tools::tool::Tool;
    ///
    /// let tools: Vec<Box<dyn Tool>> = vec![Box::new(ReadFile)];
    /// let registry = Registry::from(tools);
    ///
    /// assert_eq!(registry.len(), 1);
    /// ```
    pub fn from(tools: Vec<Box<dyn Tool>>) -> Self {
        Self { tools }
    }

    /// Creates a Registry with read-only tools (search files and read file).
    ///
    /// This is useful for scenarios where you want to restrict the registry to
    /// tools that only read data without making any modifications to the filesystem.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use code_g::tools::registry::Registry;
    ///
    /// let registry = Registry::read_only_tools();
    ///
    /// assert_eq!(registry.len(), 2);
    /// ```
    pub fn read_only_tools() -> Self {
        let tools: Vec<Box<dyn Tool>> = vec![Box::new(ReadFile), Box::new(SearchFiles)];
        Self { tools }
    }

    /// Creates a Registry with all available tools (read-only + write file + edit file + execute command).
    ///
    /// This includes ReadFile, SearchFiles, WriteFile, EditFile, and ExecuteCommand tools, providing
    /// full filesystem access and command execution capabilities.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use code_g::tools::registry::Registry;
    ///
    /// let registry = Registry::all_tools();
    ///
    /// assert_eq!(registry.len(), 5);
    /// ```
    pub fn all_tools() -> Self {
        let tools: Vec<Box<dyn Tool>> = vec![
            Box::new(ReadFile),
            Box::new(SearchFiles),
            Box::new(WriteFile),
            Box::new(EditFile),
            Box::new(ExecuteCommand),
        ];
        Self { tools }
    }

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
    ///
    /// The output from the tool execution as a String.
    ///
    /// # Errors
    ///
    /// Returns an error if the tool is not found in the registry or if the tool
    /// execution fails.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use code_g::tools::registry::Registry;
    /// use std::collections::HashMap;
    ///
    /// let registry = Registry::all_tools();
    /// let mut args = HashMap::new();
    /// args.insert("path".to_string(), "example.txt".to_string());
    ///
    /// let result = registry.call_tool("read_file", args);
    /// ```
    pub fn call_tool(
        &self,
        tool_name: &str,
        args: HashMap<String, String>,
    ) -> Result<String, String> {
        let tool = self.tools.iter().find(|t| t.name() == tool_name);
        if let Some(tool) = tool {
            tool.call(args)
        } else {
            Err(format!("Tool {} not found", tool_name))
        }
    }

    /// Returns a reference to all tools in the registry.
    ///
    /// # Returns
    ///
    /// A slice containing references to all tools in the registry.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use code_g::tools::registry::Registry;
    ///
    /// let registry = Registry::all_tools();
    /// let tools = registry.get_tools();
    ///
    /// println!("Registry contains {} tools", tools.len());
    /// ```
    pub fn get_tools(&self) -> &[Box<dyn Tool>] {
        &self.tools
    }

    /// Returns a reference to a tool by name if it exists in the registry.
    ///
    /// # Arguments
    ///
    /// * `tool_name` - The name of the tool to find.
    ///
    /// # Returns
    ///
    /// An `Option` containing a reference to the tool if found, or `None` if not found.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use code_g::tools::registry::Registry;
    ///
    /// let registry = Registry::all_tools();
    /// if let Some(tool) = registry.get_tool("read_file") {
    ///     println!("Tool requires approval: {}", tool.requires_approval());
    /// }
    /// ```
    pub fn get_tool(&self, tool_name: &str) -> Option<&Box<dyn Tool>> {
        self.tools.iter().find(|t| t.name() == tool_name)
    }

    /// Converts all tools in the registry to OpenAI-compatible tool format.
    ///
    /// This is useful when integrating with OpenAI's function calling capabilities,
    /// as it provides the tool definitions in the expected format.
    ///
    /// # Returns
    ///
    /// A vector of OpenAI tool definitions.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use code_g::tools::registry::Registry;
    ///
    /// let registry = Registry::all_tools();
    /// let openai_tools = registry.to_openai_tools();
    /// // Use openai_tools with OpenAI API
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
    /// The count of tools currently registered.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use code_g::tools::registry::Registry;
    ///
    /// let registry = Registry::new();
    /// assert_eq!(registry.len(), 0);
    ///
    /// let registry = Registry::all_tools();
    /// assert_eq!(registry.len(), 5);
    /// ```
    pub fn len(&self) -> usize {
        self.tools.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_creates_a_tool_registry_with_no_tools() {
        let registry = Registry::new();
        assert_eq!(registry.len(), 0);
        assert_eq!(registry.get_tools().len(), 0);
    }

    #[test]
    fn from_creates_a_tool_registry_with_the_given_tools() {
        let tools: Vec<Box<dyn Tool>> = vec![Box::new(ReadFile), Box::new(WriteFile)];
        let registry = Registry::from(tools);
        assert_eq!(registry.len(), 2);
        assert_eq!(registry.get_tools().len(), 2);

        let tool_names: Vec<String> = registry.get_tools().iter().map(|t| t.name()).collect();
        assert_eq!(
            tool_names,
            vec!["read_file".to_string(), "write_file".to_string()]
        );
    }

    #[test]
    fn read_only_tools_creates_a_tool_registry_with_read_only_tools() {
        let registry = Registry::read_only_tools();
        assert_eq!(registry.len(), 2);

        let tool_names: Vec<String> = registry.get_tools().iter().map(|t| t.name()).collect();
        assert_eq!(
            tool_names,
            vec!["read_file".to_string(), "search_files".to_string()]
        );
    }

    #[test]
    fn all_tools_creates_a_tool_registry_with_all_tools() {
        let registry = Registry::all_tools();
        assert_eq!(registry.len(), 5);

        let tool_names: Vec<String> = registry.get_tools().iter().map(|t| t.name()).collect();
        assert_eq!(
            tool_names,
            vec![
                "read_file".to_string(),
                "search_files".to_string(),
                "write_file".to_string(),
                "edit_file".to_string(),
                "execute_command".to_string()
            ]
        );
    }
}
