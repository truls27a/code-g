use crate::openai::model::{Parameters, Property};
use crate::tools::tool::Tool;
use std::collections::HashMap;
use std::fs;
use std::path::Path;

const MAX_FILES_RETURNED: usize = 50;

pub struct SearchFiles;

impl Tool for SearchFiles {
    fn name(&self) -> String {
        "search_files".to_string()
    }

    fn description(&self) -> String {
        "Search for files matching a pattern. The pattern can be a specific filename or use wildcards (e.g., '*.rs' for all Rust files, 'config.*' for files starting with 'config').".to_string()
    }

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

    fn strict(&self) -> bool {
        true
    }

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
    fn call_searches_for_rust_files() {
        let tool = SearchFiles;

        let result = tool.call(HashMap::from([("pattern".to_string(), "*.rs".to_string())]));

        assert!(result.is_ok());
        let output = result.unwrap();
        assert!(output.contains("main.rs"));
    }

    #[test]
    fn call_returns_error_when_pattern_is_not_provided() {
        let tool = SearchFiles;

        let result = tool.call(HashMap::from([]));

        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Pattern is required");
    }

    #[test]
    fn call_uses_current_directory_by_default() {
        let tool = SearchFiles;

        let result = tool.call(HashMap::from([(
            "pattern".to_string(),
            "Cargo.toml".to_string(),
        )]));

        assert!(result.is_ok());
        let output = result.unwrap();
        assert!(output.contains("Cargo.toml"));
    }

    #[test]
    fn search_files_returns_truncated_false_for_small_results() {
        let result = search_files("Cargo.toml", ".");

        assert!(result.is_ok());
        let (files, truncated) = result.unwrap();
        assert!(!truncated); // Should not be truncated for single file
        assert_eq!(files.len(), 1);
    }

    #[test]
    fn call_does_not_show_truncation_message_for_small_results() {
        let tool = SearchFiles;

        let result = tool.call(HashMap::from([(
            "pattern".to_string(),
            "Cargo.toml".to_string(),
        )]));

        assert!(result.is_ok());
        let output = result.unwrap();
        assert!(!output.contains("showing first"));
        assert!(!output.contains("results)"));
    }

    #[test]
    fn call_shows_truncation_message_when_limit_reached() {
        let tool = SearchFiles;

        // Search for all files (*) which should exceed the limit in a typical project
        let result = tool.call(HashMap::from([("pattern".to_string(), "*".to_string())]));

        assert!(result.is_ok());
        let output = result.unwrap();

        // Check if we got results
        if output.contains("Found") && !output.contains("No files found") {
            let lines: Vec<&str> = output.lines().collect();
            let file_count = lines.len() - 1; // Subtract 1 for the "Found X files" line

            // If we found MAX_FILES_RETURNED files, check for truncation message
            if file_count >= MAX_FILES_RETURNED {
                assert!(output.contains("showing first"));
                assert!(output.contains(&format!("{}", MAX_FILES_RETURNED)));
            }
        }
    }

    #[test]
    fn max_files_returned_constant_is_reasonable() {
        // Verify the constant is set to a reasonable value
        assert_eq!(MAX_FILES_RETURNED, 50);
        assert!(MAX_FILES_RETURNED > 0);
        assert!(MAX_FILES_RETURNED < 1000); // Not too large to overwhelm context
    }

    #[test]
    fn search_files_respects_max_limit() {
        // Test with a pattern that would normally return many files
        let result = search_files("*", ".");

        assert!(result.is_ok());
        let (files, _truncated) = result.unwrap();

        // Should never return more than MAX_FILES_RETURNED
        assert!(files.len() <= MAX_FILES_RETURNED);
    }
}
