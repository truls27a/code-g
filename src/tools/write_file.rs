use crate::openai::model::{Parameters, Property};
use crate::tools::tool::Tool;
use std::collections::HashMap;
use std::fs;

pub struct WriteFileTool;

impl Tool for WriteFileTool {
    fn name(&self) -> String {
        "write_file".to_string()
    }

    fn description(&self) -> String {
        "Write to a file".to_string()
    }

    fn parameters(&self) -> Parameters {
        Parameters {
            param_type: "object".to_string(),
            properties: HashMap::from([
                (
                    "path".to_string(),
                    Property {
                        prop_type: "string".to_string(),
                        description: "The path to the file to write to".to_string(),
                    },
                ),
                (
                    "content".to_string(),
                    Property {
                        prop_type: "string".to_string(),
                        description: "The content to write to the file".to_string(),
                    },
                ),
            ]),
            required: vec!["path".to_string(), "content".to_string()],
            additional_properties: false,
        }
    }

    fn strict(&self) -> bool {
        true
    }

    fn call(&self, args: HashMap<String, String>) -> Result<String, String> {
        let path = args.get("path").ok_or("Path is required")?;
        let content = args.get("content").ok_or("Content is required")?;
        match fs::write(path, content) {
            Ok(_) => Ok(format!("File '{}' written successfully", path)),
            Err(e) => Err(format!("Error writing file: '{}': {}", path, e)),
        }
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn call_writes_to_file() {
        let path = "tmp_file.txt";
        let content = "Hello, world!";
        let tool = WriteFileTool;
        
        let result = tool.call(HashMap::from([("path".to_string(), path.to_string()), ("content".to_string(), content.to_string())]));
        
        assert_eq!(result.unwrap(), "File 'tmp_file.txt' written successfully");
        
        let read_result = fs::read_to_string(path).unwrap();
        assert_eq!(read_result, content);

        fs::remove_file(path).unwrap();
    }

    #[test]
    fn call_returns_error_when_path_is_not_provided() {
        let tool = WriteFileTool;
        
        let result = tool.call(HashMap::from([("content".to_string(), "Hello, world!".to_string())]));
        
        assert!(result.is_err());
    }

    #[test]
    fn call_returns_error_when_content_is_not_provided() {
        let tool = WriteFileTool;
        
        let result = tool.call(HashMap::from([("path".to_string(), "tmp_file.txt".to_string())]));
        
        assert!(result.is_err());
    }

    #[test]
    fn call_overwrites_file() {
        let path = "tmp_file.txt";
        fs::write(path, "Hello, world!").unwrap();

        let tool = WriteFileTool;
        let result = tool.call(HashMap::from([("path".to_string(), path.to_string()), ("content".to_string(), "Hej på dig!".to_string())]));
        assert_eq!(result.unwrap(), "File 'tmp_file.txt' written successfully");

        let read_result = fs::read_to_string(path).unwrap();
        assert_eq!(read_result, "Hej på dig!");

        fs::remove_file(path).unwrap();
    }
}