use crate::openai::model::{Parameters, Property};
use crate::tools::tool::Tool;
use std::collections::HashMap;
use std::process::Command;

/// A tool for executing shell commands and returning their output.
///
/// This tool provides a way to run shell commands and capture both their
/// standard output and standard error. It's useful for automating tasks,
/// running system utilities, or integrating with external tools. The command
/// is executed in the current working directory.
///
/// # Examples
///
/// ```rust,no_run
/// use code_g::tools::execute_command::ExecuteCommand;
/// use code_g::tools::tool::Tool;
/// use std::collections::HashMap;
///
/// let tool = ExecuteCommand;
/// let mut args = HashMap::new();
/// args.insert("command".to_string(), "echo 'Hello, World!'".to_string());
///
/// let result = tool.call(args);
/// ```
///
/// # Notes
///
/// - Commands are executed in the current working directory
/// - Both stdout and stderr are captured and returned
/// - Long-running commands may cause timeouts
/// - Be cautious with commands that modify the filesystem
pub struct ExecuteCommand;

impl Tool for ExecuteCommand {
    /// Returns the name identifier for this tool.
    ///
    /// # Returns
    ///
    /// The string "execute_command" which identifies this tool in the tool registry.
    fn name(&self) -> String {
        "execute_command".to_string()
    }

    /// Returns a human-readable description of what this tool does.
    ///
    /// # Returns
    ///
    /// A description explaining that this tool executes shell commands.
    fn description(&self) -> String {
        "Execute a shell command and return its output".to_string()
    }

    /// Returns the parameter schema for this tool.
    ///
    /// Defines the required parameter for the execute_command tool: command.
    /// The command parameter is required and must be a string.
    ///
    /// # Returns
    ///
    /// A Parameters struct containing the schema definition for tool arguments.
    fn parameters(&self) -> Parameters {
        Parameters {
            param_type: "object".to_string(),
            properties: HashMap::from([(
                "command".to_string(),
                Property {
                    prop_type: "string".to_string(),
                    description: "The shell command to execute".to_string(),
                },
            )]),
            required: vec!["command".to_string()],
            additional_properties: false,
        }
    }

    /// Returns whether this tool uses strict parameter validation.
    ///
    /// # Returns
    ///
    /// Always returns true, indicating that this tool requires strict adherence
    /// to the parameter schema.
    fn strict(&self) -> bool {
        true
    }

    /// Returns whether this tool requires user approval before execution.
    ///
    /// # Returns
    ///
    /// Always returns true, as executing commands can potentially modify the system.
    fn requires_approval(&self) -> bool {
        true
    }

    /// Generates the approval message for this tool with the given arguments.
    ///
    /// # Arguments
    ///
    /// * `args` - A HashMap containing the tool arguments as key-value string pairs.
    ///
    /// # Returns
    ///
    /// A tuple containing (operation_name, details) for display to the user.
    fn approval_message(&self, args: &HashMap<String, String>) -> (String, String) {
        let command = args.get("command").map(|s| s.as_str()).unwrap_or("unknown");
        (
            "Execute Command".to_string(),
            format!("Command: {}", command),
        )
    }

    /// Executes the shell command and returns its output.
    ///
    /// Runs the specified command using the system shell and captures both
    /// standard output and standard error. The command is executed in the
    /// current working directory.
    ///
    /// # Arguments
    ///
    /// * `args` - A HashMap containing the required parameter:
    ///   - "command": The shell command to execute
    ///
    /// # Returns
    ///
    /// The combined output from stdout and stderr of the executed command.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The command parameter is missing
    /// - The command fails to execute
    /// - The command returns a non-zero exit code
    fn call(&self, args: HashMap<String, String>) -> Result<String, String> {
        let command = args.get("command").ok_or("Command is required")?;

        // Determine the shell based on the operating system
        let (shell, flag) = if cfg!(target_os = "windows") {
            ("cmd", "/C")
        } else {
            ("sh", "-c")
        };

        // Execute the command
        let output = Command::new(shell)
            .arg(flag)
            .arg(command)
            .output()
            .map_err(|e| format!("Failed to execute command '{}': {}", command, e))?;

        // Combine stdout and stderr
        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);

        let mut result = String::new();
        if !stdout.is_empty() {
            result.push_str(&stdout);
        }
        if !stderr.is_empty() {
            if !result.is_empty() {
                result.push('\n');
            }
            result.push_str("STDERR:\n");
            result.push_str(&stderr);
        }

        // Check if the command was successful
        if !output.status.success() {
            let exit_code = output.status.code().unwrap_or(-1);
            return Err(format!(
                "Command '{}' failed with exit code {}\nOutput: {}",
                command, exit_code, result
            ));
        }

        Ok(if result.is_empty() {
            "Command executed successfully with no output".to_string()
        } else {
            result
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn call_returns_error_when_command_parameter_missing() {
        let tool = ExecuteCommand;
        let result = tool.call(HashMap::new());

        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Command is required");
    }
}
