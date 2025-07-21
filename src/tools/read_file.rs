use crate::openai::model::{Parameters, Property};
use crate::tools::tool::Tool;
use std::collections::HashMap;
use std::fs;

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
            properties: HashMap::from([
                (
                    "path".to_string(),
                    Property {
                        prop_type: "string".to_string(),
                        description: "The path to the file to read".to_string(),
                    },
                ),
                (
                    "include_line_numbers".to_string(),
                    Property {
                        prop_type: "boolean".to_string(),
                        description: "Whether to include line numbers in the output".to_string(),
                    },
                ),
            ]),
            required: vec!["path".to_string()],
            additional_properties: false,
        }
    }

    fn strict(&self) -> bool {
        true
    }

    fn call(&self, args: HashMap<String, String>) -> Result<String, String> {
        let path = args.get("path").ok_or("Path is required")?;
        let include_line_numbers = args
            .get("include_line_numbers")
            .map(|s| s.parse::<bool>().unwrap_or(false))
            .unwrap_or(false);

        match fs::read_to_string(path) {
            Ok(content) => {
                if include_line_numbers {
                    let numbered_content = content
                        .lines()
                        .enumerate()
                        .map(|(i, line)| format!("{:4}: {}", i + 1, line))
                        .collect::<Vec<String>>()
                        .join("\n");
                    Ok(numbered_content)
                } else {
                    Ok(content)
                }
            }
            Err(e) => Err(format!("Error reading file: '{}': {}", path, e)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn call_reads_file() {
        let path = "call_reads_file_tmp_file.txt";
        let content = "Hello, world!";
        fs::write(path, content).unwrap();

        let tool = ReadFileTool;

        let result = tool.call(HashMap::from([("path".to_string(), path.to_string())]));

        assert_eq!(result.unwrap(), "Hello, world!");

        fs::remove_file(path).unwrap();
    }

    #[test]
    fn call_reads_file_with_line_numbers() {
        let path = "call_reads_file_with_line_numbers_tmp_file.txt";
        let content = "First line\nSecond line\nThird line";
        fs::write(path, content).unwrap();

        let tool = ReadFileTool;

        let result = tool.call(HashMap::from([
            ("path".to_string(), path.to_string()),
            ("include_line_numbers".to_string(), "true".to_string()),
        ]));

        let expected = "   1: First line\n   2: Second line\n   3: Third line";
        assert_eq!(result.unwrap(), expected);

        fs::remove_file(path).unwrap();
    }

    #[test]
    fn call_reads_file_without_line_numbers_when_false() {
        let path = "call_reads_file_without_line_numbers_tmp_file.txt";
        let content = "First line\nSecond line";
        fs::write(path, content).unwrap();

        let tool = ReadFileTool;

        let result = tool.call(HashMap::from([
            ("path".to_string(), path.to_string()),
            ("include_line_numbers".to_string(), "false".to_string()),
        ]));

        assert_eq!(result.unwrap(), "First line\nSecond line");

        fs::remove_file(path).unwrap();
    }

    #[test]
    fn call_returns_error_when_file_does_not_exist() {
        let tool = ReadFileTool;

        let result = tool.call(HashMap::from([(
            "path".to_string(),
            "non_existent_file.txt".to_string(),
        )]));

        assert!(result.is_err());
    }

    #[test]
    fn call_returns_error_when_path_is_not_provided() {
        let tool = ReadFileTool;

        let result = tool.call(HashMap::new());

        assert!(result.is_err());
    }
}
