use crate::openai::model::ChatMessage;

pub struct ChatMemory {
    memory: Vec<ChatMessage>,
}

impl ChatMemory {
    pub fn new() -> Self {
        Self { memory: vec![] }
    }

    pub fn add_message(&mut self, message: ChatMessage) {
        self.memory.push(message);
    }

    pub fn get_memory(&self) -> &Vec<ChatMessage> {
        &self.memory
    }

    pub fn get_last_message(&self) -> Option<&ChatMessage> {
        self.memory.last()
    }

    pub fn remove_message(&mut self, message: &ChatMessage) {
        self.memory.retain(|m| m != message);
    }

    pub fn remove_last_message(&mut self) {
        self.memory.pop();
    }

    pub fn clear(&mut self) {
        self.memory.clear();
    }
}