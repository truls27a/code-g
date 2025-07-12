use crate::openai::model::{Parameters, Property};
use crate::tools::tool::Tool;

use std::collections::HashMap;

pub struct ReadFileTool;

impl Tool for ReadFileTool {
    fn name(&self) -> String {
        "read_file".to_string()
    }

    fn description(&self) -> String {
        "Read the content of a file".to_string()
    }

    fn parameters(&self) -> Parameters {
        Parameters {
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
        }
    }

    fn strict(&self) -> bool {
        true
    }

    fn call(&self, args: HashMap<String, String>) -> Result<String, String> {
        Ok(String::from("Hello, world!"))
    }

}
