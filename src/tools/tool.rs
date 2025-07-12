use crate::openai::model::{Function, Parameters, Tool as OpenAiTool, ToolType};
use std::collections::HashMap;

pub trait Tool {
    fn name(&self) -> String;
    fn call(&self, args: HashMap<String, String>) -> Result<String, String>;
    fn to_openai_tool(&self) -> OpenAiTool;
}
