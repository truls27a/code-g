use crate::openai::model::{Function, Parameters, Tool as OpenAiTool, ToolType};
use std::collections::HashMap;

pub trait Tool {
    fn name(&self) -> String;
    fn description(&self) -> String;
    fn parameters(&self) -> Parameters;
    fn strict(&self) -> bool;

    fn call(&self, args: HashMap<String, String>) -> Result<String, String>;
    
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
