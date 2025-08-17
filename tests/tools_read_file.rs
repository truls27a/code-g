mod helpers;

use code_g::tools::read_file::ReadFile;
use code_g::tools::traits::Tool;
use std::collections::HashMap;

#[test]
fn read_file_tool_returns_file_contents() {
    let tool = ReadFile;
    let args = HashMap::from([("path".to_string(), "tests/fixtures/sample.txt".to_string())]);

    let result = tool.call(args);

    assert_eq!(result, Ok("Hello, World!".to_string()));
}

#[test]
fn read_file_tool_returns_error_when_file_does_not_exist() {
    let tool = ReadFile;
    let args = HashMap::from([("path".to_string(), "tests/fixtures/nonexistent.txt".to_string())]);

    let result = tool.call(args);

    assert!(result.is_err());
}