mod helpers;

use code_g::tools::execute_command::ExecuteCommand;
use code_g::tools::traits::Tool;
use std::collections::HashMap;

#[test]
fn execute_command_tool_runs_simple_command_and_returns_stdout() {
    let tool = ExecuteCommand;
    let cmd = if cfg!(windows) {
        "echo Hello, World!"
    } else {
        "echo Hello, World!"
    };

    let args = HashMap::from([("command".to_string(), cmd.to_string())]);
    let result = tool.call(args);

    assert!(result.is_ok());
    let output = result.unwrap();
    assert!(output.contains("Hello, World!"));
}

#[test]
fn execute_command_tool_captures_stderr_on_success() {
    let tool = ExecuteCommand;
    let cmd = "echo error 1>&2"; // Works on both Windows cmd and sh

    let args = HashMap::from([("command".to_string(), cmd.to_string())]);
    let result = tool.call(args);

    assert!(result.is_ok());
    let output = result.unwrap();
    assert!(output.starts_with("STDERR:\n"));
    assert!(output.contains("error"));
}

#[test]
fn execute_command_tool_returns_error_when_command_is_not_provided() {
    let tool = ExecuteCommand;
    let result = tool.call(HashMap::new());

    assert!(result.is_err());
    assert_eq!(result.unwrap_err(), "Command is required");
}

#[test]
fn execute_command_tool_returns_error_on_failure_with_exit_code() {
    let tool = ExecuteCommand;
    let cmd = if cfg!(windows) { "exit /b 3" } else { "exit 3" };

    let args = HashMap::from([("command".to_string(), cmd.to_string())]);
    let result = tool.call(args);

    assert!(result.is_err());
    let error = result.unwrap_err();
    assert!(error.contains("failed with exit code 3"));
}

#[test]
fn execute_command_tool_returns_message_when_no_output() {
    let tool = ExecuteCommand;
    let cmd = if cfg!(windows) { "ver > NUL" } else { ":" };

    let args = HashMap::from([("command".to_string(), cmd.to_string())]);
    let result = tool.call(args);

    assert!(result.is_ok());
    assert_eq!(
        result.unwrap(),
        "Command executed successfully with no output"
    );
}

#[test]
fn execute_command_tool_combines_stdout_and_stderr() {
    let tool = ExecuteCommand;
    let cmd = "echo out && echo err 1>&2"; // stdout then stderr

    let args = HashMap::from([("command".to_string(), cmd.to_string())]);
    let result = tool.call(args);

    assert!(result.is_ok());
    let output = result.unwrap();

    // Should contain both stdout and the STDERR section
    assert!(output.contains("out"));
    assert!(output.contains("STDERR:"));
    assert!(output.contains("err"));

    // Ensure order: stdout appears before STDERR section
    let pos_out = output.find("out").unwrap();
    let pos_stderr = output.find("STDERR:").unwrap();
    assert!(pos_out < pos_stderr);
}
