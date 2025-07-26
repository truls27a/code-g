use super::model::{TuiMessage, TuiStatus};

#[derive(Debug, Clone)]
pub struct TuiState {
    pub messages: Vec<TuiMessage>,
    pub current_status: Option<TuiStatus>,
}

impl TuiState {
    pub fn new() -> Self {
        Self {
            messages: Vec::new(),
            current_status: None,
        }
    }

    pub fn add_user_message(&mut self, content: String) {
        self.messages.push(TuiMessage::User { content });
        self.current_status = None;
    }

    pub fn add_assistant_message(&mut self, content: String) {
        self.messages.push(TuiMessage::Assistant { content });
        self.current_status = None;
    }

    pub fn add_tool_response(&mut self, summary: String, is_error: bool) {
        self.messages
            .push(TuiMessage::ToolResponse { summary, is_error });
        self.current_status = None;
    }

    pub fn add_pending_change(&mut self, change_id: u64, file_path: String, diff: String) {
        self.messages.push(TuiMessage::PendingChange { change_id, file_path, diff });
        self.current_status = None;
    }

    pub fn add_change_accepted(&mut self, change_id: u64, accepted_count: usize) {
        self.messages.push(TuiMessage::ChangeAccepted { change_id, accepted_count });
        self.current_status = None;
    }

    pub fn add_change_declined(&mut self, change_id: u64) {
        self.messages.push(TuiMessage::ChangeDeclined { change_id });
        self.current_status = None;
    }

    pub fn add_change_error(&mut self, change_id: u64, error: String) {
        self.messages.push(TuiMessage::ChangeError { change_id, error });
        self.current_status = None;
    }

    pub fn set_status(&mut self, status: Option<TuiStatus>) {
        self.current_status = status;
    }

    pub fn clear(&mut self) {
        self.messages.clear();
        self.current_status = None;
    }
}
