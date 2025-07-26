use std::collections::HashMap;
use std::fs;

#[derive(Debug, Clone, PartialEq)]
pub enum ChangeStatus {
    Pending,
    Accepted,
    Declined,
}

#[derive(Debug, Clone)]
pub struct PendingChange {
    pub id: u64,
    pub file_path: String,
    pub original_content: String,
    pub new_content: String,
    pub diff: String,
    pub status: ChangeStatus,
    pub timestamp: std::time::SystemTime,
}

impl PendingChange {
    pub fn new(
        id: u64,
        file_path: String,
        original_content: String,
        new_content: String,
    ) -> Self {
        let diff = generate_diff(&original_content, &new_content, &file_path);
        
        Self {
            id,
            file_path,
            original_content,
            new_content,
            diff,
            status: ChangeStatus::Pending,
            timestamp: std::time::SystemTime::now(),
        }
    }
}

#[derive(Debug)]
pub struct ChangeManager {
    changes: HashMap<u64, PendingChange>,
    next_id: u64,
    // Track the current state of files (after applying all accepted/pending changes)
    current_file_states: HashMap<String, String>,
}

impl ChangeManager {
    pub fn new() -> Self {
        Self {
            changes: HashMap::new(),
            next_id: 1,
            current_file_states: HashMap::new(),
        }
    }

    /// Add a new pending change and return its ID
    pub fn add_change(
        &mut self,
        file_path: String,
        new_content: String,
    ) -> Result<u64, String> {
        // Get the current state of the file (either from disk or our tracked state)
        let original_content = self.get_current_file_content(&file_path)?;
        
        let change_id = self.next_id;
        self.next_id += 1;

        let change = PendingChange::new(change_id, file_path.clone(), original_content, new_content.clone());
        
        // Update our tracked state immediately (AI sees the change)
        self.current_file_states.insert(file_path, new_content);
        
        self.changes.insert(change_id, change);
        
        Ok(change_id)
    }

    /// Accept a change and all previous changes (by timestamp)
    pub fn accept_change(&mut self, change_id: u64) -> Result<Vec<u64>, String> {
        let change = self.changes.get(&change_id)
            .ok_or_else(|| format!("Change {} not found", change_id))?;
        
        let cutoff_time = change.timestamp;
        
        // Find all changes that should be accepted (all pending changes up to this timestamp)
        let mut changes_to_accept: Vec<u64> = self.changes.iter()
            .filter(|(_, c)| c.status == ChangeStatus::Pending && c.timestamp <= cutoff_time)
            .map(|(id, _)| *id)
            .collect();
        
        changes_to_accept.sort_by(|a, b| {
            let time_a = self.changes.get(a).unwrap().timestamp;
            let time_b = self.changes.get(b).unwrap().timestamp;
            time_a.cmp(&time_b)
        });

        // Apply all changes to disk in chronological order
        for id in &changes_to_accept {
            let change = self.changes.get_mut(id).unwrap();
            fs::write(&change.file_path, &change.new_content)
                .map_err(|e| format!("Failed to write file '{}': {}", change.file_path, e))?;
            change.status = ChangeStatus::Accepted;
        }

        Ok(changes_to_accept)
    }

    /// Decline a change
    pub fn decline_change(&mut self, change_id: u64) -> Result<(), String> {
        let file_path = {
            let change = self.changes.get_mut(&change_id)
                .ok_or_else(|| format!("Change {} not found", change_id))?;
            
            if change.status != ChangeStatus::Pending {
                return Err(format!("Change {} is not pending", change_id));
            }

            change.status = ChangeStatus::Declined;
            change.file_path.clone()
        };
        
        // Revert our tracked state for this file
        self.recompute_file_state(&file_path)?;
        
        Ok(())
    }

    /// Get a change by ID
    pub fn get_change(&self, change_id: u64) -> Option<&PendingChange> {
        self.changes.get(&change_id)
    }

    /// Get all pending changes
    pub fn get_pending_changes(&self) -> Vec<&PendingChange> {
        let mut pending: Vec<&PendingChange> = self.changes.values()
            .filter(|c| c.status == ChangeStatus::Pending)
            .collect();
        
        pending.sort_by(|a, b| a.timestamp.cmp(&b.timestamp));
        pending
    }

    /// Get the current content of a file (as the AI sees it)
    pub fn get_current_file_content(&self, file_path: &str) -> Result<String, String> {
        // First check if we have a tracked state
        if let Some(content) = self.current_file_states.get(file_path) {
            return Ok(content.clone());
        }
        
        // Otherwise read from disk
        match fs::read_to_string(file_path) {
            Ok(content) => Ok(content),
            Err(e) => match e.kind() {
                std::io::ErrorKind::NotFound => Ok(String::new()), // New file
                _ => Err(format!("Error reading file '{}': {}", file_path, e)),
            }
        }
    }

    /// Recompute the current state of a file based on accepted changes
    fn recompute_file_state(&mut self, file_path: &str) -> Result<(), String> {
        // Start with the original file content
        let original_content = match fs::read_to_string(file_path) {
            Ok(content) => content,
            Err(e) => match e.kind() {
                std::io::ErrorKind::NotFound => String::new(),
                _ => return Err(format!("Error reading file '{}': {}", file_path, e)),
            }
        };

        // Apply all accepted changes in chronological order
        let mut current_content = original_content;
        let mut accepted_changes: Vec<&PendingChange> = self.changes.values()
            .filter(|c| c.file_path == file_path && c.status == ChangeStatus::Accepted)
            .collect();
        
        accepted_changes.sort_by(|a, b| a.timestamp.cmp(&b.timestamp));

        for change in accepted_changes {
            current_content = change.new_content.clone();
        }

        // Then apply pending changes in chronological order
        let mut pending_changes: Vec<&PendingChange> = self.changes.values()
            .filter(|c| c.file_path == file_path && c.status == ChangeStatus::Pending)
            .collect();
        
        pending_changes.sort_by(|a, b| a.timestamp.cmp(&b.timestamp));

        for change in pending_changes {
            current_content = change.new_content.clone();
        }

        self.current_file_states.insert(file_path.to_string(), current_content);
        Ok(())
    }

    /// Check if there are any pending changes for a file
    pub fn has_pending_changes(&self, file_path: &str) -> bool {
        self.changes.values()
            .any(|c| c.file_path == file_path && c.status == ChangeStatus::Pending)
    }
}

impl Default for ChangeManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Generate a unified diff between two strings
fn generate_diff(original: &str, new: &str, file_path: &str) -> String {
    let original_lines: Vec<&str> = original.lines().collect();
    let new_lines: Vec<&str> = new.lines().collect();
    
    if original_lines == new_lines {
        return "No changes".to_string();
    }
    
    let mut diff = String::new();
    diff.push_str(&format!("--- {}\n", file_path));
    diff.push_str(&format!("+++ {}\n", file_path));
    
    // Simple line-by-line diff implementation
    let max_lines = original_lines.len().max(new_lines.len());
    let mut context_start = 0;
    let mut in_hunk = false;
    let mut hunk_lines = Vec::new();
    
    for i in 0..max_lines {
        let original_line = original_lines.get(i);
        let new_line = new_lines.get(i);
        
        match (original_line, new_line) {
            (Some(orig), Some(new_l)) if orig == new_l => {
                if in_hunk {
                    hunk_lines.push(format!(" {}", orig));
                }
            }
            _ => {
                if !in_hunk {
                    in_hunk = true;
                    context_start = i.saturating_sub(3);
                    
                    // Add context lines before the change
                    for j in context_start..i {
                        if let Some(line) = original_lines.get(j) {
                            hunk_lines.push(format!(" {}", line));
                        }
                    }
                }
                
                if let Some(orig) = original_line {
                    hunk_lines.push(format!("-{}", orig));
                }
                if let Some(new_l) = new_line {
                    hunk_lines.push(format!("+{}", new_l));
                }
            }
        }
    }
    
    if in_hunk {
        // Add hunk header
        let start_old = context_start + 1;
        let count_old = original_lines.len() - context_start;
        let start_new = context_start + 1;
        let count_new = new_lines.len() - context_start;
        
        diff.push_str(&format!("@@ -{},{} +{},{} @@\n", start_old, count_old, start_new, count_new));
        
        for line in hunk_lines {
            diff.push_str(&line);
            diff.push('\n');
        }
    }
    
    if diff.trim().is_empty() || diff.lines().count() <= 2 {
        format!("File content changed:\n--- Original:\n{}\n+++ New:\n{}", original, new)
    } else {
        diff
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn test_change_manager_new() {
        let manager = ChangeManager::new();
        assert_eq!(manager.next_id, 1);
        assert!(manager.changes.is_empty());
        assert!(manager.current_file_states.is_empty());
    }

    #[test]
    fn test_add_change() {
        let mut manager = ChangeManager::new();
        
        // Create a test file
        let test_file = "test_change_manager.txt";
        fs::write(test_file, "original content").unwrap();
        
        let change_id = manager.add_change(test_file.to_string(), "new content".to_string()).unwrap();
        
        assert_eq!(change_id, 1);
        assert_eq!(manager.next_id, 2);
        
        let change = manager.get_change(change_id).unwrap();
        assert_eq!(change.file_path, test_file);
        assert_eq!(change.original_content, "original content");
        assert_eq!(change.new_content, "new content");
        assert_eq!(change.status, ChangeStatus::Pending);
        
        fs::remove_file(test_file).unwrap();
    }

    #[test]
    fn test_accept_change() {
        let mut manager = ChangeManager::new();
        
        let test_file = "test_accept_change.txt";
        fs::write(test_file, "original").unwrap();
        
        let change_id = manager.add_change(test_file.to_string(), "modified".to_string()).unwrap();
        let accepted = manager.accept_change(change_id).unwrap();
        
        assert_eq!(accepted, vec![change_id]);
        
        let change = manager.get_change(change_id).unwrap();
        assert_eq!(change.status, ChangeStatus::Accepted);
        
        let file_content = fs::read_to_string(test_file).unwrap();
        assert_eq!(file_content, "modified");
        
        fs::remove_file(test_file).unwrap();
    }

    #[test]
    fn test_decline_change() {
        let mut manager = ChangeManager::new();
        
        let test_file = "test_decline_change.txt";
        fs::write(test_file, "original").unwrap();
        
        let change_id = manager.add_change(test_file.to_string(), "modified".to_string()).unwrap();
        manager.decline_change(change_id).unwrap();
        
        let change = manager.get_change(change_id).unwrap();
        assert_eq!(change.status, ChangeStatus::Declined);
        
        // File should remain unchanged
        let file_content = fs::read_to_string(test_file).unwrap();
        assert_eq!(file_content, "original");
        
        fs::remove_file(test_file).unwrap();
    }

    #[test]
    fn test_generate_diff() {
        let original = "line 1\nline 2\nline 3";
        let new = "line 1\nmodified line 2\nline 3";
        let diff = generate_diff(original, new, "test.txt");
        
        assert!(diff.contains("--- test.txt"));
        assert!(diff.contains("+++ test.txt"));
        assert!(diff.contains("-line 2"));
        assert!(diff.contains("+modified line 2"));
    }

    #[test]
    fn test_get_pending_changes() {
        let mut manager = ChangeManager::new();
        
        let test_file1 = "test_pending1.txt";
        let test_file2 = "test_pending2.txt";
        fs::write(test_file1, "content1").unwrap();
        fs::write(test_file2, "content2").unwrap();
        
        let id1 = manager.add_change(test_file1.to_string(), "new1".to_string()).unwrap();
        let id2 = manager.add_change(test_file2.to_string(), "new2".to_string()).unwrap();
        
        let pending = manager.get_pending_changes();
        assert_eq!(pending.len(), 2);
        
        manager.accept_change(id1).unwrap();
        let pending = manager.get_pending_changes();
        assert_eq!(pending.len(), 1);
        assert_eq!(pending[0].id, id2);
        
        fs::remove_file(test_file1).unwrap();
        fs::remove_file(test_file2).unwrap();
    }
} 