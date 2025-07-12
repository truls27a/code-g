use crate::openai::model::{Function, Parameters, Property, Tool as OpenAiTool, ToolType};
use crate::tools::tool::Tool;

use std::collections::HashMap;

pub struct ReadFileTool;

impl Tool for ReadFileTool {
    fn name(&self) -> String {
        "read_file".to_string()
    }

    fn call(&self, args: HashMap<String, String>) -> Result<String, String> {
        Ok(String::from("Hello, world!"))
    }

    fn to_openai_tool(&self) -> OpenAiTool {
        OpenAiTool {
            tool_type: ToolType::Function,
            function: Function {
                name: "read_file".to_string(),
                description: "Read the content of a file".to_string(),
                parameters: Parameters {
                    param_type: "object".to_string(),
                    properties: HashMap::from([(
                        "path".to_string(),
                        Property {
                            prop_type: "string".to_string(),
                            description: "The path to the file to read".to_string(),
                        },
                    )]),
                    required: vec!["path".to_string()],
                    additional_properties: false,
                },
                strict: true,
            },
        }
    }
}
