use crate::client::model::ChatMessage;

/// A storage container for managing chat conversation history.
///
/// ChatMemory maintains an ordered sequence of chat messages that represent
/// the conversation history between users, assistants, and tools. It provides
/// methods to add, retrieve, and manipulate messages while preserving the
/// chronological order of the conversation.
///
/// # Examples
///
/// ```rust
/// use code_g::session::memory::ChatMemory;
/// use code_g::client::model::ChatMessage;
///
/// let mut memory = ChatMemory::new();
/// memory.add_message(ChatMessage::User {
///     content: "Hello, world!".to_string(),
/// });
///
/// assert_eq!(memory.get_memory().len(), 1);
/// ```
#[derive(Debug, PartialEq, Clone)]
pub struct ChatMemory {
    memory: Vec<ChatMessage>,
}

impl ChatMemory {
    /// Creates a new empty chat memory.
    ///
    /// Initializes a ChatMemory instance with no messages. This is the
    /// standard way to begin a new conversation session.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use code_g::session::memory::ChatMemory;
    ///
    /// let memory = ChatMemory::new();
    /// assert_eq!(memory.get_memory().len(), 0);
    /// ```
    pub fn new() -> Self {
        Self { memory: vec![] }
    }

    /// Creates a chat memory from an existing vector of messages.
    ///
    /// This constructor allows you to initialize ChatMemory with a pre-existing
    /// conversation history, which is useful for restoring saved sessions or
    /// continuing conversations from a checkpoint.
    ///
    /// # Arguments
    ///
    /// * `memory` - A vector of ChatMessage instances representing the conversation history
    ///
    /// # Examples
    ///
    /// ```rust
    /// use code_g::session::memory::ChatMemory;
    /// use code_g::client::model::ChatMessage;
    ///
    /// let messages = vec![
    ///     ChatMessage::User { content: "Hello".to_string() }
    /// ];
    /// let memory = ChatMemory::from(messages);
    /// assert_eq!(memory.get_memory().len(), 1);
    /// ```
    pub fn from(memory: Vec<ChatMessage>) -> Self {
        Self { memory }
    }

    /// Adds a new message to the end of the conversation history.
    ///
    /// Messages are appended in chronological order, maintaining the sequence
    /// of the conversation. This method should be called whenever a new message
    /// is received from any participant (user, assistant, or tool).
    ///
    /// # Arguments
    ///
    /// * `message` - The ChatMessage to add to the conversation history
    ///
    /// # Examples
    ///
    /// ```rust
    /// use code_g::session::memory::ChatMemory;
    /// use code_g::client::model::ChatMessage;
    ///
    /// let mut memory = ChatMemory::new();
    /// memory.add_message(ChatMessage::User {
    ///     content: "Hello, world!".to_string(),
    /// });
    /// ```
    pub fn add_message(&mut self, message: ChatMessage) {
        self.memory.push(message);
    }

    /// Returns a reference to the complete conversation history.
    ///
    /// Provides read-only access to all messages in the conversation,
    /// ordered chronologically from first to last.
    ///
    /// # Returns
    ///
    /// A reference to the vector containing all ChatMessage instances
    ///
    /// # Examples
    ///
    /// ```rust
    /// use code_g::session::memory::ChatMemory;
    ///
    /// let memory = ChatMemory::new();
    /// let messages = memory.get_memory();
    /// assert_eq!(messages.len(), 0);
    /// ```
    pub fn get_memory(&self) -> &Vec<ChatMessage> {
        &self.memory
    }

    /// Removes all messages from the conversation history.
    ///
    /// This operation empties the entire conversation, resetting the memory
    /// to its initial state. Use this method to start fresh conversations
    /// or when implementing session reset functionality.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use code_g::session::memory::ChatMemory;
    /// use code_g::client::model::ChatMessage;
    ///
    /// let mut memory = ChatMemory::new();
    /// memory.add_message(ChatMessage::User {
    ///     content: "Hello".to_string(),
    /// });
    /// memory.clear();
    /// assert_eq!(memory.get_memory().len(), 0);
    /// ```
    pub fn clear(&mut self) {
        self.memory.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn add_message_adds_message_to_memory() {
        let mut memory = ChatMemory::new();

        memory.add_message(ChatMessage::User {
            content: "Hello, world!".to_string(),
        });

        assert_eq!(memory.get_memory().len(), 1);
        assert_eq!(
            memory.get_memory()[0],
            ChatMessage::User {
                content: "Hello, world!".to_string()
            }
        );
    }

    #[test]
    fn get_memory_returns_memory() {
        let mut memory = ChatMemory::new();

        memory.add_message(ChatMessage::User {
            content: "Hello, world!".to_string(),
        });

        assert_eq!(
            memory.get_memory(),
            &[ChatMessage::User {
                content: "Hello, world!".to_string(),
            }]
        );
    }

    #[test]
    fn clear_clears_memory() {
        let mut memory = ChatMemory::new();
        memory.add_message(ChatMessage::User {
            content: "Hello, world!".to_string(),
        });
        memory.clear();
        assert_eq!(memory.get_memory().len(), 0);
        assert_eq!(memory.get_memory(), &[]);
    }
}
