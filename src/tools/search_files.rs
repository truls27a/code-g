use crate::client::model::{Parameters, Property};
use crate::tools::traits::Tool;
use std::collections::HashMap;
use std::fs;
use std::path::Path;

const MAX_FILES_RETURNED: usize = 50;

/// A tool for searching files using glob-style patterns.
///
/// SearchFiles provides functionality to find files matching wildcard patterns
/// within the filesystem. It implements the [`Tool`] trait to be used within
/// the tool system for file discovery operations. The search is performed
/// recursively starting from the current directory.
///
/// # Examples
///
/// ```rust,no_run
/// use code_g::tools::search_files::SearchFiles;
/// use code_g::tools::traits::Tool;
/// use std::collections::HashMap;
///
/// let tool = SearchFiles;
/// let args = HashMap::from([
///     ("pattern".to_string(), "*.rs".to_string()),
/// ]);
/// let result = tool.call(args);
/// ```
///
/// # Notes
/// - Results are limited to 50 files to prevent overwhelming output.
/// - Supports wildcard patterns: `*` (any characters) and `?` (single character).
/// - Search is performed recursively from the current directory.
/// - Files are returned in sorted order.
pub struct SearchFiles;

impl Tool for SearchFiles {
    /// Returns the name identifier for this tool.
    ///
    /// # Returns
    ///
    /// A string containing "search_files" as the tool identifier.
    fn name(&self) -> String {
        "search_files".to_string()
    }

    /// Returns a human-readable description of what this tool does.
    ///
    /// # Returns
    ///
    /// A string describing the tool's functionality for searching files with patterns.
    fn description(&self) -> String {
        "Search for files matching a pattern. The pattern can be a specific filename or use wildcards (e.g., '*.rs' for all Rust files, 'config.*' for files starting with 'config').".to_string()
    }

    /// Returns the parameter schema for this tool.
    ///
    /// Defines the required parameter for the search_files tool: pattern.
    /// The pattern parameter is a required string value that supports wildcards.
    ///
    /// # Returns
    ///
    /// A Parameters object containing the schema for the pattern argument.
    fn parameters(&self) -> Parameters {
        Parameters {
            param_type: "object".to_string(),
            properties: HashMap::from([
                (
                    "pattern".to_string(),
                    Property {
                        prop_type: "string".to_string(),
                        description: "The search pattern. Can be a filename (e.g., 'main.rs') or use wildcards (e.g., '*.rs', 'test_*', 'config.*'). Searches in the current directory.".to_string(),
                    },
                ),
            ]),
            required: vec!["pattern".to_string()],
            additional_properties: false,
        }
    }

    /// Returns whether this tool uses strict parameter validation.
    ///
    /// # Returns
    ///
    /// Always returns true, indicating strict parameter validation is enabled.
    fn strict(&self) -> bool {
        true
    }

    /// Returns whether this tool requires user approval before execution.
    ///
    /// # Returns
    ///
    /// Always returns false, as searching files is a safe, read-only operation.
    fn requires_approval(&self) -> bool {
        false
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
        let pattern = args.get("pattern").map(|s| s.as_str()).unwrap_or("unknown");
        (
            "Search Files".to_string(),
            format!("Pattern: {}", pattern),
        )
    }

    /// Executes the file search operation with the provided arguments.
    ///
    /// Searches recursively from the current directory for files matching the
    /// specified pattern. Results are limited to 50 files and returned in
    /// sorted order. Supports wildcards `*` (any characters) and `?` (single character).
    ///
    /// # Arguments
    ///
    /// * `args` - A HashMap containing the "pattern" string value.
    ///
    /// # Returns
    ///
    /// A formatted string listing all matching files with their paths.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The "pattern" argument is missing
    /// - No files match the specified pattern
    /// - The directory cannot be read due to permissions or other I/O errors
    fn call(&self, args: HashMap<String, String>) -> Result<String, String> {
        let pattern = args.get("pattern").ok_or("Pattern is required")?;
        let directory = ".";

        match search_files(pattern, directory) {
            Ok((files, truncated)) => {
                if files.is_empty() {
                    Err(format!("No files found matching pattern '{}'", pattern))
                } else {
                    let truncation_note = if truncated {
                        format!(" (showing first {} results)", MAX_FILES_RETURNED)
                    } else {
                        String::new()
                    };

                    Ok(format!(
                        "Found {} file(s) matching pattern '{}'{}:\n{}",
                        files.len(),
                        pattern,
                        truncation_note,
                        files.join("\n")
                    ))
                }
            }
            Err(e) => Err(format!("Error searching for files: {}", e)),
        }
    }
}

/// Searches for files matching a pattern in the specified directory.
///
/// Performs a recursive search starting from the given directory for files
/// that match the specified glob-style pattern. Results are limited and sorted.
///
/// # Arguments
///
/// * `pattern` - The glob pattern to match against filenames
/// * `directory` - The directory to start searching from
///
/// # Returns
///
/// A tuple containing a vector of matching file paths and a boolean indicating
/// if results were truncated due to the limit.
///
/// # Errors
///
/// Returns an error if the directory doesn't exist, isn't a directory, or
/// cannot be read.
fn search_files(pattern: &str, directory: &str) -> Result<(Vec<String>, bool), String> {
    let mut found_files = Vec::new();
    let search_path = Path::new(directory);

    if !search_path.exists() {
        return Err(format!("Directory '{}' does not exist", directory));
    }

    if !search_path.is_dir() {
        return Err(format!("'{}' is not a directory", directory));
    }

    search_directory_recursive(search_path, pattern, &mut found_files)?;
    found_files.sort();

    // Check if we hit the limit (indicating there might be more files)
    let truncated = found_files.len() >= MAX_FILES_RETURNED;

    Ok((found_files, truncated))
}

/// Recursively searches a directory for files matching a pattern.
///
/// This function traverses the directory tree depth-first, collecting files
/// that match the given pattern. Search stops when the maximum file limit
/// is reached to prevent excessive results.
///
/// # Arguments
///
/// * `dir` - The directory path to search in
/// * `pattern` - The glob pattern to match against filenames
/// * `found_files` - Mutable vector to collect matching file paths
///
/// # Errors
///
/// Returns an error if directory entries cannot be read or accessed.
fn search_directory_recursive(
    dir: &Path,
    pattern: &str,
    found_files: &mut Vec<String>,
) -> Result<(), String> {
    // Stop searching if we've reached the limit
    if found_files.len() >= MAX_FILES_RETURNED {
        return Ok(());
    }

    let entries = fs::read_dir(dir)
        .map_err(|e| format!("Failed to read directory '{}': {}", dir.display(), e))?;

    for entry in entries {
        // Check limit again in case we reached it during this iteration
        if found_files.len() >= MAX_FILES_RETURNED {
            break;
        }

        let entry = entry.map_err(|e| format!("Failed to read directory entry: {}", e))?;
        let path = entry.path();

        if path.is_dir() {
            // Recursively search subdirectories
            search_directory_recursive(&path, pattern, found_files)?;
        } else if path.is_file() {
            if let Some(filename) = path.file_name().and_then(|n| n.to_str()) {
                if matches_pattern(filename, pattern) {
                    found_files.push(path.to_string_lossy().to_string());
                }
            }
        }
    }

    Ok(())
}

/// Determines if a filename matches a glob-style pattern.
///
/// Supports wildcard matching with `*` (any number of characters) and
/// `?` (exactly one character). Also handles exact string matching when
/// no wildcards are present.
///
/// # Arguments
///
/// * `filename` - The filename to test against the pattern
/// * `pattern` - The glob pattern with optional wildcards
///
/// # Returns
///
/// True if the filename matches the pattern, false otherwise.
fn matches_pattern(filename: &str, pattern: &str) -> bool {
    if pattern == "*" {
        return true;
    }

    // Handle exact match
    if !pattern.contains('*') && !pattern.contains('?') {
        return filename == pattern;
    }

    // Convert glob pattern to regex-like matching
    // This is a simple implementation that handles * and ?
    let pattern_chars: Vec<char> = pattern.chars().collect();
    let filename_chars: Vec<char> = filename.chars().collect();

    matches_with_wildcards(&filename_chars, &pattern_chars, 0, 0)
}

/// Performs wildcard matching using recursive backtracking.
///
/// This function implements glob-style pattern matching by recursively
/// comparing filename and pattern characters, handling wildcards appropriately.
///
/// # Arguments
///
/// * `filename` - Character array of the filename being matched
/// * `pattern` - Character array of the pattern with wildcards
/// * `f_idx` - Current index in the filename
/// * `p_idx` - Current index in the pattern
///
/// # Returns
///
/// True if the remaining filename matches the remaining pattern, false otherwise.
fn matches_with_wildcards(filename: &[char], pattern: &[char], f_idx: usize, p_idx: usize) -> bool {
    // If we've consumed the entire pattern
    if p_idx >= pattern.len() {
        return f_idx >= filename.len();
    }

    // If we've consumed the entire filename but still have pattern left
    if f_idx >= filename.len() {
        // Only valid if remaining pattern is all *
        return pattern[p_idx..].iter().all(|&c| c == '*');
    }

    match pattern[p_idx] {
        '*' => {
            // Try matching zero characters
            if matches_with_wildcards(filename, pattern, f_idx, p_idx + 1) {
                return true;
            }
            // Try matching one or more characters
            for i in f_idx..filename.len() {
                if matches_with_wildcards(filename, pattern, i + 1, p_idx + 1) {
                    return true;
                }
            }
            false
        }
        '?' => {
            // ? matches exactly one character
            matches_with_wildcards(filename, pattern, f_idx + 1, p_idx + 1)
        }
        c => {
            // Exact character match
            if filename[f_idx] == c {
                matches_with_wildcards(filename, pattern, f_idx + 1, p_idx + 1)
            } else {
                false
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn matches_pattern_returns_true_for_matching_patterns() {
        // Test * wildcard patterns
        assert!(matches_pattern("main.rs", "*.rs"));
        assert!(matches_pattern("test.rs", "*.rs"));
        assert!(!matches_pattern("main.py", "*.rs"));
        assert!(matches_pattern("main.rs", "main.rs"));
        assert!(!matches_pattern("test.rs", "main.rs"));
        assert!(matches_pattern("config.json", "config.*"));
        assert!(matches_pattern("config.toml", "config.*"));
        assert!(!matches_pattern("main.json", "config.*"));
        assert!(matches_pattern("test_file.txt", "test_*"));
        assert!(matches_pattern("any_file.txt", "*"));

        // Test ? wildcard patterns - matches exactly one character
        assert!(matches_pattern("a.rs", "?.rs"));
        assert!(matches_pattern("x.rs", "?.rs"));
        assert!(!matches_pattern("ab.rs", "?.rs")); // ? doesn't match multiple chars
        assert!(!matches_pattern(".rs", "?.rs")); // ? doesn't match zero chars

        assert!(matches_pattern("test1.txt", "test?.txt"));
        assert!(matches_pattern("testA.txt", "test?.txt"));
        assert!(!matches_pattern("test12.txt", "test?.txt")); // ? doesn't match multiple chars
        assert!(!matches_pattern("test.txt", "test?.txt")); // ? doesn't match zero chars

        // Test multiple ? wildcards
        assert!(matches_pattern("abc.txt", "???.txt"));
        assert!(matches_pattern("xyz.txt", "???.txt"));
        assert!(!matches_pattern("ab.txt", "???.txt")); // not enough chars
        assert!(!matches_pattern("abcd.txt", "???.txt")); // too many chars

        // Test combinations of ? and *
        assert!(matches_pattern("a_test.rs", "?_*.rs"));
        assert!(matches_pattern("x_anything.rs", "?_*.rs"));
        assert!(!matches_pattern("_test.rs", "?_*.rs")); // ? must match one char
        assert!(!matches_pattern("ab_test.rs", "?_*.rs")); // ? matches only one char
    }

    #[test]
    fn call_returns_error_when_pattern_is_not_provided() {
        let tool = SearchFiles;

        let result = tool.call(HashMap::from([]));

        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Pattern is required");
    }

    #[test]
    fn max_files_returned_constant_is_reasonable() {
        // Verify the constant is set to a reasonable value
        assert_eq!(MAX_FILES_RETURNED, 50);
        assert!(MAX_FILES_RETURNED > 0);
        assert!(MAX_FILES_RETURNED < 1000); // Not too large to overwhelm context
    }
}
