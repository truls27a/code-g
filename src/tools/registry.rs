use crate::openai::model::Tool as OpenAiTool;
use crate::tools::tool::Tool;
use std::collections::HashMap;

pub struct ToolRegistry {
    tools: Vec<Box<dyn Tool>>,
}

impl ToolRegistry {
    pub fn new() -> Self {
        Self {
            tools: vec![],
        }
    }

    pub fn from(tools: Vec<Box<dyn Tool>>) -> Self {
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
