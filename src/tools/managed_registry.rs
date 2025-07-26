use crate::openai::model::Tool as OpenAiTool;
use crate::tools::edit_file::ManagedEditFileTool;
use crate::tools::read_file::ReadFileTool;
use crate::tools::search_files::SearchFilesTool;
use crate::tools::tool::Tool;
use crate::tools::write_file::ManagedWriteFileTool;
use crate::tools::change_manager::ChangeManager;
use crate::chat::event::{Event, EventHandler};
use std::collections::HashMap;

pub struct ManagedToolRegistry {
    regular_tools: Vec<Box<dyn Tool>>,
    edit_tool: ManagedEditFileTool,
    write_tool: ManagedWriteFileTool,
    change_manager: ChangeManager,
}

impl ManagedToolRegistry {
    pub fn new() -> Self {
        Self {
            regular_tools: vec![],
            edit_tool: ManagedEditFileTool,
            write_tool: ManagedWriteFileTool,
            change_manager: ChangeManager::new(),
        }
    }

    /// Creates a ManagedToolRegistry with read-only tools only
    pub fn read_only_tools() -> Self {
        let regular_tools: Vec<Box<dyn Tool>> = vec![
            Box::new(ReadFileTool), 
            Box::new(SearchFilesTool)
        ];
        Self {
            regular_tools,
            edit_tool: ManagedEditFileTool,
            write_tool: ManagedWriteFileTool,
            change_manager: ChangeManager::new(),
        }
    }

    /// Creates a ManagedToolRegistry with all available tools
    pub fn all_tools() -> Self {
        let regular_tools: Vec<Box<dyn Tool>> = vec![
            Box::new(ReadFileTool),
            Box::new(SearchFilesTool),
        ];
        Self {
            regular_tools,
            edit_tool: ManagedEditFileTool,
            write_tool: ManagedWriteFileTool,
            change_manager: ChangeManager::new(),
        }
    }

    /// Creates a ManagedToolRegistry with custom regular tools
    pub fn from(regular_tools: Vec<Box<dyn Tool>>) -> Self {
        Self {
            regular_tools,
            edit_tool: ManagedEditFileTool,
            write_tool: ManagedWriteFileTool,
            change_manager: ChangeManager::new(),
        }
    }

    pub fn call_tool(
        &mut self,
        tool_name: &str,
        args: HashMap<String, String>,
        event_handler: &mut dyn EventHandler,
    ) -> Result<String, String> {
        match tool_name {
            "edit_file" => {
                match self.edit_tool.call_with_manager(args, &mut self.change_manager) {
                    Ok((message, change_id)) => {
                        // Get the change to send the diff
                        if let Some(change) = self.change_manager.get_change(change_id) {
                            event_handler.handle_event(Event::PendingFileChange {
                                change_id,
                                file_path: change.file_path.clone(),
                                diff: change.diff.clone(),
                            });
                        }
                        Ok(message)
                    }
                    Err(e) => Err(e),
                }
            }
            "write_file" => {
                match self.write_tool.call_with_manager(args, &mut self.change_manager) {
                    Ok((message, change_id)) => {
                        // Get the change to send the diff
                        if let Some(change) = self.change_manager.get_change(change_id) {
                            event_handler.handle_event(Event::PendingFileChange {
                                change_id,
                                file_path: change.file_path.clone(),
                                diff: change.diff.clone(),
                            });
                        }
                        Ok(message)
                    }
                    Err(e) => Err(e),
                }
            }
            _ => {
                // Try regular tools
                let tool = self.regular_tools.iter().find(|t| t.name() == tool_name);
                if let Some(tool) = tool {
                    tool.call(args)
                } else {
                    Err(format!("Tool {} not found", tool_name))
                }
            }
        }
    }

    pub fn accept_change(
        &mut self,
        change_id: u64,
        event_handler: &mut dyn EventHandler,
    ) -> Result<String, String> {
        match self.change_manager.accept_change(change_id) {
            Ok(accepted_changes) => {
                event_handler.handle_event(Event::ChangeAccepted {
                    change_id,
                    accepted_changes: accepted_changes.clone(),
                });
                Ok(format!(
                    "Accepted change {} and {} previous changes",
                    change_id,
                    accepted_changes.len() - 1
                ))
            }
            Err(e) => {
                event_handler.handle_event(Event::ChangeError {
                    change_id,
                    error: e.clone(),
                });
                Err(e)
            }
        }
    }

    pub fn decline_change(
        &mut self,
        change_id: u64,
        event_handler: &mut dyn EventHandler,
    ) -> Result<String, String> {
        match self.change_manager.decline_change(change_id) {
            Ok(_) => {
                event_handler.handle_event(Event::ChangeDeclined { change_id });
                Ok(format!("Declined change {}", change_id))
            }
            Err(e) => {
                event_handler.handle_event(Event::ChangeError {
                    change_id,
                    error: e.clone(),
                });
                Err(e)
            }
        }
    }

    pub fn list_pending_changes(&self) -> String {
        let pending = self.change_manager.get_pending_changes();
        if pending.is_empty() {
            "No pending changes".to_string()
        } else {
            let mut result = format!("Pending changes ({}):\n", pending.len());
            for change in pending {
                result.push_str(&format!(
                    "  {} - {} (ID: {})\n",
                    change.id, change.file_path, change.id
                ));
            }
            result
        }
    }

    pub fn get_file_content(&self, file_path: &str) -> Result<String, String> {
        self.change_manager.get_current_file_content(file_path)
    }

    pub fn to_openai_tools(&self) -> Vec<OpenAiTool> {
        let mut tools: Vec<OpenAiTool> = self.regular_tools
            .iter()
            .map(|tool| tool.to_openai_tool())
            .collect();

        // Add managed tools
        tools.push(OpenAiTool {
            tool_type: crate::openai::model::ToolType::Function,
            function: crate::openai::model::Function {
                name: self.edit_tool.name(),
                description: self.edit_tool.description(),
                parameters: self.edit_tool.parameters(),
                strict: self.edit_tool.strict(),
            },
        });

        tools.push(OpenAiTool {
            tool_type: crate::openai::model::ToolType::Function,
            function: crate::openai::model::Function {
                name: self.write_tool.name(),
                description: self.write_tool.description(),
                parameters: self.write_tool.parameters(),
                strict: self.write_tool.strict(),
            },
        });

        tools
    }

    pub fn len(&self) -> usize {
        self.regular_tools.len() + 2 // +2 for edit_file and write_file
    }
}

impl Default for ManagedToolRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::chat::event::{Action, Event, EventHandler};
    use std::io;

    // Mock event handler for testing
    struct MockEventHandler {
        events: Vec<Event>,
    }

    impl MockEventHandler {
        fn new() -> Self {
            Self { events: Vec::new() }
        }

        fn get_events(&self) -> &Vec<Event> {
            &self.events
        }
    }

    impl EventHandler for MockEventHandler {
        fn handle_event(&mut self, event: Event) {
            self.events.push(event);
        }

        fn handle_action(&mut self, _action: Action) -> Result<String, io::Error> {
            Ok("test".to_string())
        }
    }

    #[test]
    fn new_creates_empty_managed_registry() {
        let registry = ManagedToolRegistry::new();
        assert_eq!(registry.len(), 2); // edit_file and write_file
    }

    #[test]
    fn read_only_tools_creates_registry_with_read_tools() {
        let registry = ManagedToolRegistry::read_only_tools();
        assert_eq!(registry.len(), 4); // read_file, search_files, edit_file, write_file
    }

    #[test]
    fn all_tools_creates_registry_with_all_tools() {
        let registry = ManagedToolRegistry::all_tools();
        assert_eq!(registry.len(), 4); // read_file, search_files, edit_file, write_file
    }

    #[test]
    fn call_tool_handles_edit_file() {
        use std::fs;
        
        let mut registry = ManagedToolRegistry::new();
        let mut event_handler = MockEventHandler::new();
        
        let test_file = "test_managed_edit.txt";
        fs::write(test_file, "original content").unwrap();

        let result = registry.call_tool(
            "edit_file",
            HashMap::from([
                ("path".to_string(), test_file.to_string()),
                ("old_string".to_string(), "original".to_string()),
                ("new_string".to_string(), "modified".to_string()),
            ]),
            &mut event_handler,
        );

        assert!(result.is_ok());
        assert!(result.unwrap().contains("File edit queued"));

        let events = event_handler.get_events();
        assert_eq!(events.len(), 1);
        match &events[0] {
            Event::PendingFileChange { change_id, file_path, diff: _ } => {
                assert_eq!(*change_id, 1);
                assert_eq!(file_path, test_file);
            }
            _ => panic!("Expected PendingFileChange event"),
        }

        fs::remove_file(test_file).unwrap();
    }

    #[test]
    fn accept_change_applies_change_to_disk() {
        use std::fs;
        
        let mut registry = ManagedToolRegistry::new();
        let mut event_handler = MockEventHandler::new();
        
        let test_file = "test_accept_change.txt";
        fs::write(test_file, "original").unwrap();

        // Add a change
        let _ = registry.call_tool(
            "edit_file",
            HashMap::from([
                ("path".to_string(), test_file.to_string()),
                ("old_string".to_string(), "original".to_string()),
                ("new_string".to_string(), "modified".to_string()),
            ]),
            &mut event_handler,
        );

        // Accept the change
        let result = registry.accept_change(1, &mut event_handler);
        assert!(result.is_ok());

        // Check that file was actually modified
        let file_content = fs::read_to_string(test_file).unwrap();
        assert_eq!(file_content, "modified");

        // Check events
        let events = event_handler.get_events();
        assert_eq!(events.len(), 2); // PendingFileChange + ChangeAccepted

        fs::remove_file(test_file).unwrap();
    }

    #[test]
    fn decline_change_does_not_apply_change() {
        use std::fs;
        
        let mut registry = ManagedToolRegistry::new();
        let mut event_handler = MockEventHandler::new();
        
        let test_file = "test_decline_change.txt";
        fs::write(test_file, "original").unwrap();

        // Add a change
        let _ = registry.call_tool(
            "edit_file",
            HashMap::from([
                ("path".to_string(), test_file.to_string()),
                ("old_string".to_string(), "original".to_string()),
                ("new_string".to_string(), "modified".to_string()),
            ]),
            &mut event_handler,
        );

        // Decline the change
        let result = registry.decline_change(1, &mut event_handler);
        assert!(result.is_ok());

        // Check that file was not modified
        let file_content = fs::read_to_string(test_file).unwrap();
        assert_eq!(file_content, "original");

        fs::remove_file(test_file).unwrap();
    }
} 