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

    pub fn set_status(&mut self, status: Option<TuiStatus>) {
        self.current_status = status;
    }

    pub fn clear(&mut self) {
        self.messages.clear();
        self.current_status = None;
    }
}
