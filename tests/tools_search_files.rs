mod helpers;

use code_g::tools::search_files::SearchFiles;
use code_g::tools::traits::Tool;
use std::collections::HashMap;

#[test]
fn search_files_tool_returns_files_matching_pattern() {
    let tool = SearchFiles;
    let args = HashMap::from([("pattern".to_string(), "sample.json".to_string())]);

    let result = tool.call(args);

    assert!(result.is_ok());
    assert!(result.unwrap().contains("tests\\fixtures\\sample.json"));
}

#[test]
fn search_files_tool_returns_files_matching_pattern_with_wildcard() {
    let tool = SearchFiles;
    let args = HashMap::from([("pattern".to_string(), "sample.*".to_string())]);

    let result = tool.call(args);

    assert!(result.is_ok());
    let result_content = result.unwrap();
    assert!(result_content.contains("tests\\fixtures\\sample.json"));
    assert!(result_content.contains("tests\\fixtures\\sample.rs"));
    assert!(result_content.contains("tests\\fixtures\\sample.txt"));
}

#[test]
fn search_files_tool_returns_files_matching_question_mark() {
    let tool = SearchFiles;
    let args = HashMap::from([("pattern".to_string(), "sample.???".to_string())]);

    let result = tool.call(args);

    assert!(result.is_ok());
    let result_content = result.unwrap();
    assert!(result_content.contains("tests\\fixtures\\sample.txt"));
}

#[test]
fn search_files_tool_returns_error_when_pattern_is_not_provided() {
    let tool = SearchFiles;
    let args = HashMap::new();

    let result = tool.call(args);

    assert!(result.is_err());
}

#[test]
fn search_files_tool_returns_error_when_pattern_is_empty() {
    let tool = SearchFiles;
    let args = HashMap::from([("pattern".to_string(), "".to_string())]);

    let result = tool.call(args);

    assert!(result.is_err());
}

#[test]
fn search_files_tool_returns_error_when_no_files_are_found() {
    let tool = SearchFiles;
    let args = HashMap::from([(
        "pattern".to_string(),
        "This pattern does not match any files".to_string(),
    )]);

    let result = tool.call(args);

    assert!(result.is_err());
}
