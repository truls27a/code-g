use crate::openai::model::{Parameters, Property};
use crate::tools::tool::Tool;
use crate::tools::change_manager::ChangeManager;
use std::collections::HashMap;
use std::fs;

pub struct WriteFileTool;

impl Tool for WriteFileTool {
    fn name(&self) -> String {
        "write_file".to_string()
    }

    fn description(&self) -> String {
        "Write to a file. If the file does not exist, it will be created. If the file exists, it will be overwritten.".to_string()
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
            Err(e) => match e.kind() {
                std::io::ErrorKind::NotFound => Err(format!("File '{}' not found", path)),
                _ => Err(format!("Error writing file: '{}': {}", path, e)),
            },
        }
    }
}

pub struct ManagedWriteFileTool;

impl ManagedWriteFileTool {
    pub fn call_with_manager(
        &self,
        args: HashMap<String, String>,
        change_manager: &mut ChangeManager,
    ) -> Result<(String, u64), String> {
        let path = args.get("path").ok_or("Path is required")?;
        let content = args.get("content").ok_or("Content is required")?;

        // Add the change to the change manager
        let change_id = change_manager.add_change(path.clone(), content.clone())?;

        Ok((
            format!(
                "File write queued for '{}' (Change ID: {})",
                path, change_id
            ),
            change_id,
        ))
    }

    pub fn name(&self) -> String {
        "write_file".to_string()
    }

    pub fn description(&self) -> String {
        "Write to a file. If the file does not exist, it will be created. If the file exists, it will be overwritten.".to_string()
    }

    pub fn parameters(&self) -> Parameters {
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

    pub fn strict(&self) -> bool {
        true
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn call_writes_to_file() {
        let path = "call_writes_to_file_tmp_file.txt";
        let content = "Hello, world!";
        let tool = WriteFileTool;
        
        let result = tool.call(HashMap::from([("path".to_string(), path.to_string()), ("content".to_string(), content.to_string())]));
        
        assert_eq!(result.unwrap(), format!("File '{}' written successfully", path));
        
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
        let path = "call_overwrites_file_tmp_file.txt";
        fs::write(path, "Hello, world!").unwrap();

        let tool = WriteFileTool;
        let result = tool.call(HashMap::from([("path".to_string(), path.to_string()), ("content".to_string(), "Hej på dig!".to_string())]));
        assert_eq!(result.unwrap(), format!("File '{}' written successfully", path));

        let read_result = fs::read_to_string(path).unwrap();
        assert_eq!(read_result, "Hej på dig!");

        fs::remove_file(path).unwrap();
    }

    #[test]
    fn managed_tool_creates_pending_change() {
        let mut change_manager = ChangeManager::new();
        let tool = ManagedWriteFileTool;
        
        let path = "managed_write_test.txt";
        let content = "Hello, managed world!";

        let result = tool.call_with_manager(
            HashMap::from([
                ("path".to_string(), path.to_string()),
                ("content".to_string(), content.to_string()),
            ]),
            &mut change_manager,
        );

        assert!(result.is_ok());
        let (message, change_id) = result.unwrap();
        assert!(message.contains("File write queued"));
        assert_eq!(change_id, 1);

        let pending_changes = change_manager.get_pending_changes();
        assert_eq!(pending_changes.len(), 1);
        assert_eq!(pending_changes[0].file_path, path);
        assert_eq!(pending_changes[0].new_content, content);
    }
}