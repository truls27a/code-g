use crate::openai::model::{Parameters, Property};
use crate::tools::tool::Tool;
use std::collections::HashMap;
use std::fs;

pub struct EditFileTool;

impl Tool for EditFileTool {
    fn name(&self) -> String {
        "edit_file".to_string()
    }

    fn description(&self) -> String {
        "Edit a file by replacing a specific string with new content".to_string()
    }

    fn parameters(&self) -> Parameters {
        Parameters {
            param_type: "object".to_string(),
            properties: HashMap::from([
                (
                    "path".to_string(),
                    Property {
                        prop_type: "string".to_string(),
                        description: "The path to the file to edit".to_string(),
                    },
                ),
                (
                    "old_string".to_string(),
                    Property {
                        prop_type: "string".to_string(),
                        description: "The string to find and replace in the file".to_string(),
                    },
                ),
                (
                    "new_string".to_string(),
                    Property {
                        prop_type: "string".to_string(),
                        description: "The replacement string".to_string(),
                    },
                ),
            ]),
            required: vec![
                "path".to_string(),
                "old_string".to_string(),
                "new_string".to_string(),
            ],
            additional_properties: false,
        }
    }

    fn strict(&self) -> bool {
        true
    }

    fn call(&self, args: HashMap<String, String>) -> Result<String, String> {
        let path = args.get("path").ok_or("Path is required")?;
        let old_string = args.get("old_string").ok_or("Old string is required")?;
        let new_string = args.get("new_string").ok_or("New string is required")?;

        // Read the current file content
        let content = fs::read_to_string(path)
            .map_err(|e| format!("Error reading file '{}': {}", path, e))?;

        // Check if the old string exists in the file
        if !content.contains(old_string) {
            return Err(format!(
                "String '{}' not found in file '{}'",
                old_string, path
            ));
        }

        // Count occurrences to warn about multiple matches
        let occurrence_count = content.matches(old_string).count();
        if occurrence_count > 1 {
            return Err(format!(
                "String '{}' appears {} times in file '{}'. Please provide a more specific string that appears only once",
                old_string, occurrence_count, path
            ));
        }

        // Replace the string
        let new_content = content.replace(old_string, new_string);

        // Write the modified content back to the file
        fs::write(path, &new_content)
            .map_err(|e| format!("Error writing to file '{}': {}", path, e))?;

        Ok(format!(
            "Successfully edited file '{}': replaced '{}' with '{}'",
            path, old_string, new_string
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn call_replaces_string_successfully() {
        let path = "replace_string_tmp_file.txt";
        let content = "Line 1\nLine 2\nLine 3";
        fs::write(path, content).unwrap();

        let tool = EditFileTool;
        let result = tool.call(HashMap::from([
            ("path".to_string(), path.to_string()),
            ("old_string".to_string(), "Line 2".to_string()),
            ("new_string".to_string(), "Modified Line 2".to_string()),
        ]));

        assert!(result.is_ok());
        let file_content = fs::read_to_string(path).unwrap();
        assert_eq!(file_content, "Line 1\nModified Line 2\nLine 3");

        fs::remove_file(path).unwrap();
    }

    #[test]
    fn call_replaces_multiline_string_successfully() {
        let path = "replace_multiline_tmp_file.txt";
        let content = "Line 1\nLine 2\nLine 3\nLine 4";
        fs::write(path, content).unwrap();

        let tool = EditFileTool;
        let result = tool.call(HashMap::from([
            ("path".to_string(), path.to_string()),
            ("old_string".to_string(), "Line 2\nLine 3".to_string()),
            (
                "new_string".to_string(),
                "New Line A\nNew Line B".to_string(),
            ),
        ]));

        assert!(result.is_ok());
        let file_content = fs::read_to_string(path).unwrap();
        assert_eq!(file_content, "Line 1\nNew Line A\nNew Line B\nLine 4");

        fs::remove_file(path).unwrap();
    }

    #[test]
    fn call_deletes_string_when_given_empty_replacement() {
        let path = "delete_string_tmp_file.txt";
        let content = "Line 1\nLine 2\nLine 3";
        fs::write(path, content).unwrap();

        let tool = EditFileTool;
        let result = tool.call(HashMap::from([
            ("path".to_string(), path.to_string()),
            ("old_string".to_string(), "\nLine 2".to_string()),
            ("new_string".to_string(), "".to_string()),
        ]));

        assert!(result.is_ok());
        let file_content = fs::read_to_string(path).unwrap();
        assert_eq!(file_content, "Line 1\nLine 3");

        fs::remove_file(path).unwrap();
    }

    #[test]
    fn call_returns_error_when_file_does_not_exist() {
        let tool = EditFileTool;
        let result = tool.call(HashMap::from([
            ("path".to_string(), "non_existent_file.txt".to_string()),
            ("old_string".to_string(), "test".to_string()),
            ("new_string".to_string(), "replacement".to_string()),
        ]));

        assert!(result.is_err());
    }

    #[test]
    fn call_returns_error_when_string_not_found() {
        let path = "string_not_found_tmp_file.txt";
        let content = "Line 1\nLine 2";
        fs::write(path, content).unwrap();

        let tool = EditFileTool;
        let result = tool.call(HashMap::from([
            ("path".to_string(), path.to_string()),
            ("old_string".to_string(), "Line 3".to_string()),
            ("new_string".to_string(), "test".to_string()),
        ]));

        assert!(result.is_err());
        fs::remove_file(path).unwrap();
    }

    #[test]
    fn call_returns_error_when_string_appears_multiple_times() {
        let path = "multiple_occurrences_tmp_file.txt";
        let content = "Line\nLine\nLine 3";
        fs::write(path, content).unwrap();

        let tool = EditFileTool;
        let result = tool.call(HashMap::from([
            ("path".to_string(), path.to_string()),
            ("old_string".to_string(), "Line".to_string()),
            ("new_string".to_string(), "test".to_string()),
        ]));

        assert!(result.is_err());
        fs::remove_file(path).unwrap();
    }

    #[test]
    fn call_returns_error_when_required_parameters_missing() {
        let tool = EditFileTool;
        let result = tool.call(HashMap::from([(
            "path".to_string(),
            "test.txt".to_string(),
        )]));

        assert!(result.is_err());
    }
}
