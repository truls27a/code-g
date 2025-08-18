mod helpers;

use code_g::tools::traits::Tool;
use code_g::tools::write_file::WriteFile;
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

fn create_temp_dir() -> PathBuf {
    use std::time::{SystemTime, UNIX_EPOCH};
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let temp_dir =
        std::env::temp_dir().join(format!("code_g_test_{}_{}", std::process::id(), nanos));
    fs::create_dir_all(&temp_dir).expect("Failed to create temp directory");
    temp_dir
}

fn cleanup_temp_dir(temp_dir: &PathBuf) {
    if temp_dir.exists() {
        fs::remove_dir_all(temp_dir).ok();
    }
}

#[test]
fn write_file_tool_creates_new_file_successfully() {
    let temp_dir = create_temp_dir();
    let file_path = temp_dir.join("test_file.txt");

    let tool = WriteFile;
    let args = HashMap::from([
        ("path".to_string(), file_path.to_string_lossy().to_string()),
        ("content".to_string(), "Hello, World!".to_string()),
    ]);

    let result = tool.call(args);

    assert!(result.is_ok());
    assert!(file_path.exists());
    let content = fs::read_to_string(&file_path).expect("Failed to read file");
    assert_eq!(content, "Hello, World!");

    cleanup_temp_dir(&temp_dir);
}

#[test]
fn write_file_tool_overwrites_existing_file() {
    let temp_dir = create_temp_dir();
    let file_path = temp_dir.join("existing_file.txt");

    // Create initial file
    fs::write(&file_path, "Original content").expect("Failed to create initial file");

    let tool = WriteFile;
    let args = HashMap::from([
        ("path".to_string(), file_path.to_string_lossy().to_string()),
        ("content".to_string(), "New content".to_string()),
    ]);

    let result = tool.call(args);

    assert!(result.is_ok());
    let content = fs::read_to_string(&file_path).expect("Failed to read file");
    assert_eq!(content, "New content");

    cleanup_temp_dir(&temp_dir);
}

#[test]
fn write_file_tool_writes_empty_content() {
    let temp_dir = create_temp_dir();
    let file_path = temp_dir.join("empty_file.txt");

    let tool = WriteFile;
    let args = HashMap::from([
        ("path".to_string(), file_path.to_string_lossy().to_string()),
        ("content".to_string(), "".to_string()),
    ]);

    let result = tool.call(args);

    assert!(result.is_ok());
    assert!(file_path.exists());
    let content = fs::read_to_string(&file_path).expect("Failed to read file");
    assert_eq!(content, "");

    cleanup_temp_dir(&temp_dir);
}

#[test]
fn write_file_tool_writes_multiline_content() {
    let temp_dir = create_temp_dir();
    let file_path = temp_dir.join("multiline_file.txt");
    let multiline_content = "Line 1\nLine 2\nLine 3\n";

    let tool = WriteFile;
    let args = HashMap::from([
        ("path".to_string(), file_path.to_string_lossy().to_string()),
        ("content".to_string(), multiline_content.to_string()),
    ]);

    let result = tool.call(args);

    assert!(result.is_ok());
    let content = fs::read_to_string(&file_path).expect("Failed to read file");
    assert_eq!(content, multiline_content);

    cleanup_temp_dir(&temp_dir);
}

#[test]
fn write_file_tool_creates_file_in_nested_directory() {
    let temp_dir = create_temp_dir();
    let nested_dir = temp_dir.join("nested").join("deep");
    fs::create_dir_all(&nested_dir).expect("Failed to create nested directory");
    let file_path = nested_dir.join("nested_file.txt");

    let tool = WriteFile;
    let args = HashMap::from([
        ("path".to_string(), file_path.to_string_lossy().to_string()),
        ("content".to_string(), "Nested content".to_string()),
    ]);

    let result = tool.call(args);

    assert!(result.is_ok());
    assert!(file_path.exists());
    let content = fs::read_to_string(&file_path).expect("Failed to read file");
    assert_eq!(content, "Nested content");

    cleanup_temp_dir(&temp_dir);
}

#[test]
fn write_file_tool_returns_error_when_path_is_not_provided() {
    let tool = WriteFile;
    let args = HashMap::from([("content".to_string(), "Hello, world!".to_string())]);

    let result = tool.call(args);

    assert!(result.is_err());
    assert_eq!(result.unwrap_err(), "Path is required");
}

#[test]
fn write_file_tool_returns_error_when_content_is_not_provided() {
    let tool = WriteFile;
    let args = HashMap::from([("path".to_string(), "tmp_file.txt".to_string())]);

    let result = tool.call(args);

    assert!(result.is_err());
    assert_eq!(result.unwrap_err(), "Content is required");
}

#[test]
fn write_file_tool_returns_error_when_both_args_are_missing() {
    let tool = WriteFile;
    let args = HashMap::new();

    let result = tool.call(args);

    assert!(result.is_err());
}

#[test]
fn write_file_tool_returns_error_for_invalid_path() {
    let tool = WriteFile;
    // Use an invalid path that should cause an error
    let invalid_path = if cfg!(windows) {
        "Z:\\nonexistent\\path\\file.txt"
    } else {
        "/root/nonexistent/path/file.txt"
    };

    let args = HashMap::from([
        ("path".to_string(), invalid_path.to_string()),
        ("content".to_string(), "content".to_string()),
    ]);

    let result = tool.call(args);

    assert!(result.is_err());
    let error_msg = result.unwrap_err();
    // The error message should contain either "Error writing file" or "File" and "not found"
    assert!(
        error_msg.contains("Error writing file")
            || (error_msg.contains("File") && error_msg.contains("not found"))
    );
}

#[test]
fn write_file_tool_success_message_contains_file_path() {
    let temp_dir = create_temp_dir();
    let file_path = temp_dir.join("message_test.txt");

    let tool = WriteFile;
    let args = HashMap::from([
        ("path".to_string(), file_path.to_string_lossy().to_string()),
        ("content".to_string(), "test content".to_string()),
    ]);

    let result = tool.call(args);

    assert!(result.is_ok());
    let success_message = result.unwrap();
    assert!(success_message.contains(file_path.to_string_lossy().as_ref()));
    assert!(success_message.contains("written successfully"));

    cleanup_temp_dir(&temp_dir);
}
