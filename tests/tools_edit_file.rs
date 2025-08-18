mod helpers;

use code_g::tools::edit_file::EditFile;
use code_g::tools::traits::Tool;
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
        std::env::temp_dir().join(format!("code_g_edit_test_{}_{}", std::process::id(), nanos));
    fs::create_dir_all(&temp_dir).expect("Failed to create temp directory");
    temp_dir
}

fn cleanup_temp_dir(temp_dir: &PathBuf) {
    if temp_dir.exists() {
        fs::remove_dir_all(temp_dir).ok();
    }
}

#[test]
fn edit_file_tool_replaces_single_occurrence_successfully() {
    let temp_dir = create_temp_dir();
    let file_path = temp_dir.join("sample.txt");

    fs::write(&file_path, "The quick brown fox jumps over the lazy dog.")
        .expect("Failed to create initial file");

    let tool = EditFile;
    let args = HashMap::from([
        ("path".to_string(), file_path.to_string_lossy().to_string()),
        ("old_string".to_string(), "brown fox".to_string()),
        ("new_string".to_string(), "red fox".to_string()),
    ]);

    let result = tool.call(args);

    assert!(result.is_ok());
    let content = fs::read_to_string(&file_path).expect("Failed to read file");
    assert_eq!(content, "The quick red fox jumps over the lazy dog.");

    cleanup_temp_dir(&temp_dir);
}

#[test]
fn edit_file_tool_deletes_content_when_new_string_empty() {
    let temp_dir = create_temp_dir();
    let file_path = temp_dir.join("delete.txt");

    fs::write(&file_path, "Hello, DELETE_ME world!").expect("Failed to create initial file");

    let tool = EditFile;
    let args = HashMap::from([
        ("path".to_string(), file_path.to_string_lossy().to_string()),
        ("old_string".to_string(), "DELETE_ME ".to_string()),
        ("new_string".to_string(), "".to_string()),
    ]);

    let result = tool.call(args);

    assert!(result.is_ok());
    let content = fs::read_to_string(&file_path).expect("Failed to read file");
    assert_eq!(content, "Hello, world!");

    cleanup_temp_dir(&temp_dir);
}

#[test]
fn edit_file_tool_replaces_multiline_content() {
    let temp_dir = create_temp_dir();
    let file_path = temp_dir.join("multiline.txt");
    let original = "Line 1\nREPLACE THIS BLOCK\nLine 3\n";
    let target = "REPLACE THIS BLOCK";
    let replacement = "Line 2 (updated)\nAnd a bit more";

    fs::write(&file_path, original).expect("Failed to create initial file");

    let tool = EditFile;
    let args = HashMap::from([
        ("path".to_string(), file_path.to_string_lossy().to_string()),
        ("old_string".to_string(), target.to_string()),
        ("new_string".to_string(), replacement.to_string()),
    ]);

    let result = tool.call(args);

    assert!(result.is_ok());
    let content = fs::read_to_string(&file_path).expect("Failed to read file");
    assert_eq!(content, format!("Line 1\n{}\nLine 3\n", replacement));

    cleanup_temp_dir(&temp_dir);
}

#[test]
fn edit_file_tool_errors_when_string_not_found() {
    let temp_dir = create_temp_dir();
    let file_path = temp_dir.join("not_found.txt");

    fs::write(&file_path, "Nothing to see here").expect("Failed to create initial file");

    let tool = EditFile;
    let args = HashMap::from([
        ("path".to_string(), file_path.to_string_lossy().to_string()),
        ("old_string".to_string(), "missing".to_string()),
        ("new_string".to_string(), "replacement".to_string()),
    ]);

    let result = tool.call(args);

    assert!(result.is_err());
    let error = result.unwrap_err();
    assert!(error.contains("String 'missing' not found"));

    cleanup_temp_dir(&temp_dir);
}

#[test]
fn edit_file_tool_errors_when_multiple_occurrences_found() {
    let temp_dir = create_temp_dir();
    let file_path = temp_dir.join("multiple.txt");

    fs::write(&file_path, "alpha beta alpha gamma").expect("Failed to create initial file");

    let tool = EditFile;
    let args = HashMap::from([
        ("path".to_string(), file_path.to_string_lossy().to_string()),
        ("old_string".to_string(), "alpha".to_string()),
        ("new_string".to_string(), "ALPHA".to_string()),
    ]);

    let result = tool.call(args);

    assert!(result.is_err());
    let error = result.unwrap_err();
    assert!(error.contains("appears 2 times"));
    assert!(error.contains("Please provide a more specific string"));

    cleanup_temp_dir(&temp_dir);
}

#[test]
fn edit_file_tool_returns_error_when_file_does_not_exist() {
    let tool = EditFile;
    let args = HashMap::from([
        (
            "path".to_string(),
            "tests/fixtures/does_not_exist.txt".to_string(),
        ),
        ("old_string".to_string(), "x".to_string()),
        ("new_string".to_string(), "y".to_string()),
    ]);

    let result = tool.call(args);

    assert!(result.is_err());
    let error = result.unwrap_err();
    assert!(error.contains("File 'tests/fixtures/does_not_exist.txt' not found"));
}

#[test]
fn edit_file_tool_returns_error_when_path_is_not_provided() {
    let tool = EditFile;
    let args = HashMap::from([
        ("old_string".to_string(), "a".to_string()),
        ("new_string".to_string(), "b".to_string()),
    ]);

    let result = tool.call(args);

    assert!(result.is_err());
    assert_eq!(result.unwrap_err(), "Path is required");
}

#[test]
fn edit_file_tool_returns_error_when_old_string_is_not_provided() {
    let tool = EditFile;
    let args = HashMap::from([
        ("path".to_string(), "tmp.txt".to_string()),
        ("new_string".to_string(), "b".to_string()),
    ]);

    let result = tool.call(args);

    assert!(result.is_err());
    assert_eq!(result.unwrap_err(), "Old string is required");
}

#[test]
fn edit_file_tool_returns_error_when_new_string_is_not_provided() {
    let tool = EditFile;
    let args = HashMap::from([
        ("path".to_string(), "tmp.txt".to_string()),
        ("old_string".to_string(), "a".to_string()),
    ]);

    let result = tool.call(args);

    assert!(result.is_err());
    assert_eq!(result.unwrap_err(), "New string is required");
}

#[test]
fn edit_file_tool_success_message_contains_details() {
    let temp_dir = create_temp_dir();
    let file_path = temp_dir.join("message.txt");

    fs::write(&file_path, "foo bar baz").expect("Failed to create initial file");

    let tool = EditFile;
    let args = HashMap::from([
        ("path".to_string(), file_path.to_string_lossy().to_string()),
        ("old_string".to_string(), "bar".to_string()),
        ("new_string".to_string(), "BAR".to_string()),
    ]);

    let result = tool.call(args);

    assert!(result.is_ok());
    let message = result.unwrap();
    assert!(message.contains(file_path.to_string_lossy().as_ref()));
    assert!(message.contains("replaced 'bar' with 'BAR'"));

    cleanup_temp_dir(&temp_dir);
}
