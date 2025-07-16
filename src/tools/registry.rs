use crate::openai::model::Tool as OpenAiTool;
use crate::tools::read_file::ReadFileTool;
use crate::tools::search_files::SearchFilesTool;
use crate::tools::tool::Tool;
use crate::tools::write_file::WriteFileTool;
use std::collections::HashMap;

pub struct ToolRegistry {
    tools: Vec<Box<dyn Tool>>,
}

impl ToolRegistry {
    pub fn new() -> Self {
        Self { tools: vec![] }
    }

    pub fn from(tools: Vec<Box<dyn Tool>>) -> Self {
        Self { tools }
    }

    /// Creates a ToolRegistry with read-only tools (search files and read file)
    pub fn read_only_tools() -> Self {
        let tools: Vec<Box<dyn Tool>> = vec![Box::new(ReadFileTool), Box::new(SearchFilesTool)];
        Self { tools }
    }

    /// Creates a ToolRegistry with all available tools (read-only + write file)
    pub fn all_tools() -> Self {
        let tools: Vec<Box<dyn Tool>> = vec![
            Box::new(ReadFileTool),
            Box::new(SearchFilesTool),
            Box::new(WriteFileTool),
        ];
        Self { tools }
    }

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

    pub fn get_tools(&self) -> &[Box<dyn Tool>] {
        &self.tools
    }

    pub fn to_openai_tools(&self) -> Vec<OpenAiTool> {
        self.tools
            .iter()
            .map(|tool| tool.to_openai_tool())
            .collect()
    }

    pub fn len(&self) -> usize {
        self.tools.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_creates_a_tool_registry_with_no_tools() {
        let registry = ToolRegistry::new();
        assert_eq!(registry.len(), 0);
        assert_eq!(registry.get_tools().len(), 0);
    }

    #[test]
    fn from_creates_a_tool_registry_with_the_given_tools() {
        let tools: Vec<Box<dyn Tool>> = vec![Box::new(ReadFileTool), Box::new(WriteFileTool)];
        let registry = ToolRegistry::from(tools);
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
        let registry = ToolRegistry::read_only_tools();
        assert_eq!(registry.len(), 2);

        let tool_names: Vec<String> = registry.get_tools().iter().map(|t| t.name()).collect();
        assert_eq!(
            tool_names,
            vec!["read_file".to_string(), "search_files".to_string()]
        );
    }

    #[test]
    fn all_tools_creates_a_tool_registry_with_all_tools() {
        let registry = ToolRegistry::all_tools();
        assert_eq!(registry.len(), 3);

        let tool_names: Vec<String> = registry.get_tools().iter().map(|t| t.name()).collect();
        assert_eq!(
            tool_names,
            vec![
                "read_file".to_string(),
                "search_files".to_string(),
                "write_file".to_string()
            ]
        );
    }
}
