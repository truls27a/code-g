mod helpers;

use code_g::client::model::{AssistantMessage, ChatMessage, Parameters, Property, ToolCall};
use code_g::session::event::Event;
use code_g::session::system_prompt::SYSTEM_PROMPT;
use helpers::assertions::{assert_chat_history, assert_events, assert_tool_calls};
use helpers::scenario::ScenarioBuilder;
use std::collections::HashMap;

#[tokio::test]
async fn chat_session_handles_full_workflow() {
    let scenario = ScenarioBuilder::new()
        .inputs([
            "I need a function to calculate the factorial of a number. Please implement it in a new file called math_utils.rs",
            "Great! Can you add input validation to make sure the number is non-negative?"
        ])
        // Set up the tools that will be available
        .add_mock_tool(
            "search_files",
            "Search for files matching a pattern",
            Parameters {
                param_type: "object".to_string(),
                properties: HashMap::from([
                    (
                        "pattern".to_string(),
                        Property {
                            prop_type: "string".to_string(),
                            description: "The search pattern".to_string(),
                        },
                    ),
                ]),
                required: vec!["pattern".to_string()],
                additional_properties: false,
            },
            true,
            false,
            "AI wants to search for files",
            "src/lib.rs\nsrc/main.rs\nsrc/utils/mod.rs"
        )
        .add_mock_tool(
            "read_file",
            "Read the contents of a file",
            Parameters {
                param_type: "object".to_string(),
                properties: HashMap::from([
                    (
                        "file_path".to_string(),
                        Property {
                            prop_type: "string".to_string(),
                            description: "The path to the file to read".to_string(),
                        },
                    ),
                ]),
                required: vec!["file_path".to_string()],
                additional_properties: false,
            },
            true,
            false,
            "AI wants to read a file",
            "// This is the main library file\npub mod utils;\n"
        )
        .add_mock_tool(
            "write_file",
            "Write content to a file",
            Parameters {
                param_type: "object".to_string(),
                properties: HashMap::from([
                    (
                        "file_path".to_string(),
                        Property {
                            prop_type: "string".to_string(),
                            description: "The path to the file to write".to_string(),
                        },
                    ),
                    (
                        "content".to_string(),
                        Property {
                            prop_type: "string".to_string(),
                            description: "The content to write to the file".to_string(),
                        },
                    ),
                ]),
                required: vec!["file_path".to_string(), "content".to_string()],
                additional_properties: false,
            },
            true,
            false,
            "AI wants to write to a file",
            "File written successfully"
        )
        .add_mock_tool(
            "edit_file",
            "Edit an existing file",
            Parameters {
                param_type: "object".to_string(),
                properties: HashMap::from([
                    (
                        "file_path".to_string(),
                        Property {
                            prop_type: "string".to_string(),
                            description: "The path to the file to edit".to_string(),
                        },
                    ),
                    (
                        "search_text".to_string(),
                        Property {
                            prop_type: "string".to_string(),
                            description: "The text to search for".to_string(),
                        },
                    ),
                    (
                        "replacement_text".to_string(),
                        Property {
                            prop_type: "string".to_string(),
                            description: "The text to replace with".to_string(),
                        },
                    ),
                ]),
                required: vec!["file_path".to_string(), "search_text".to_string(), "replacement_text".to_string()],
                additional_properties: false,
            },
            true,
            false,
            "AI wants to edit a file",
            "File edited successfully"
        )
        // First AI response: understanding the request and searching for existing files
        .then_message("I'll help you implement a factorial function. Let me first search for existing math-related files to understand the project structure.", false)
        .then_tool_call(
            "1",
            "search_files",
            HashMap::from([("pattern".to_string(), "*math*".to_string())])
        )
        .then_message("Let me also search for any utility files to see how the project is organized.", false)
        .then_tool_call(
            "2",
            "search_files",
            HashMap::from([("pattern".to_string(), "*utils*".to_string())])
        )
        .then_message("Now let me check the main library file to understand the module structure.", false)
        .then_tool_call(
            "3",
            "read_file",
            HashMap::from([("file_path".to_string(), "src/lib.rs".to_string())])
        )
        .then_message("Perfect! I can see there's a utils module. I'll create the math_utils.rs file with the factorial function.", false)
        .then_tool_call(
            "4",
            "write_file",
            HashMap::from([
                ("file_path".to_string(), "src/math_utils.rs".to_string()),
                ("content".to_string(), "/// Calculate the factorial of a non-negative integer\n/// \n/// # Arguments\n/// \n/// * `n` - A non-negative integer\n/// \n/// # Returns\n/// \n/// The factorial of n\n/// \n/// # Examples\n/// \n/// ```\n/// assert_eq!(factorial(5), 120);\n/// assert_eq!(factorial(0), 1);\n/// ```\npub fn factorial(n: u32) -> u64 {\n    match n {\n        0 | 1 => 1,\n        _ => (2..=n as u64).product(),\n    }\n}\n\n#[cfg(test)]\nmod tests {\n    use super::*;\n\n    #[test]\n    fn test_factorial() {\n        assert_eq!(factorial(0), 1);\n        assert_eq!(factorial(1), 1);\n        assert_eq!(factorial(5), 120);\n        assert_eq!(factorial(10), 3628800);\n    }\n}".to_string())
            ])
        )
        .then_message("Great! I've created the math_utils.rs file with a factorial function. The function handles the base cases (0 and 1) and uses Rust's product iterator for efficiency. I've also included comprehensive tests.", true)
        
        // Second turn: User asks for input validation
        .then_message("Absolutely! I'll add input validation to ensure the function handles negative numbers appropriately. Let me update the function to include proper validation.", false)
        .then_tool_call(
            "5",
            "edit_file",
            HashMap::from([
                ("file_path".to_string(), "src/math_utils.rs".to_string()),
                ("search_text".to_string(), "pub fn factorial(n: u32) -> u64 {\n    match n {\n        0 | 1 => 1,\n        _ => (2..=n as u64).product(),\n    }\n}".to_string()),
                ("replacement_text".to_string(), "pub fn factorial(n: i32) -> Result<u64, String> {\n    if n < 0 {\n        return Err(\"Factorial is not defined for negative numbers\".to_string());\n    }\n    \n    let n = n as u32;\n    match n {\n        0 | 1 => Ok(1),\n        _ => Ok((2..=n as u64).product()),\n    }\n}".to_string())
            ])
        )
        .then_message("Perfect! I've updated the factorial function to include input validation. The function now:\n\n1. Takes an i32 parameter to allow negative inputs\n2. Returns a Result<u64, String> to handle errors gracefully\n3. Validates that the input is non-negative\n4. Returns an appropriate error message for negative inputs\n\nThe function is now more robust and follows Rust's error handling best practices!", true)
        .run()
        .await;

    // Verify the complete sequence of events
    assert_events(
        &scenario.events,
        &[
            Event::SessionStarted,
            // First user message
            Event::ReceivedUserMessage {
                message: "I need a function to calculate the factorial of a number. Please implement it in a new file called math_utils.rs".to_string(),
            },
            Event::AwaitingAssistantResponse,
            Event::ReceivedAssistantMessage {
                message: "I'll help you implement a factorial function. Let me first search for existing math-related files to understand the project structure.".to_string(),
            },
            Event::AwaitingAssistantResponse,
            Event::ReceivedToolCall {
                tool_name: "search_files".to_string(),
                parameters: HashMap::from([("pattern".to_string(), "*math*".to_string())]),
            },
            Event::ReceivedToolResponse {
                tool_name: "search_files".to_string(),
                response: "src/lib.rs\nsrc/main.rs\nsrc/utils/mod.rs".to_string(),
                parameters: HashMap::from([("pattern".to_string(), "*math*".to_string())]),
            },
            Event::AwaitingAssistantResponse,
            Event::ReceivedAssistantMessage {
                message: "Let me also search for any utility files to see how the project is organized.".to_string(),
            },
            Event::AwaitingAssistantResponse,
            Event::ReceivedToolCall {
                tool_name: "search_files".to_string(),
                parameters: HashMap::from([("pattern".to_string(), "*utils*".to_string())]),
            },
            Event::ReceivedToolResponse {
                tool_name: "search_files".to_string(),
                response: "src/lib.rs\nsrc/main.rs\nsrc/utils/mod.rs".to_string(),
                parameters: HashMap::from([("pattern".to_string(), "*utils*".to_string())]),
            },
            Event::AwaitingAssistantResponse,
            Event::ReceivedAssistantMessage {
                message: "Now let me check the main library file to understand the module structure.".to_string(),
            },
            Event::AwaitingAssistantResponse,
            Event::ReceivedToolCall {
                tool_name: "read_file".to_string(),
                parameters: HashMap::from([("file_path".to_string(), "src/lib.rs".to_string())]),
            },
            Event::ReceivedToolResponse {
                tool_name: "read_file".to_string(),
                response: "// This is the main library file\npub mod utils;\n".to_string(),
                parameters: HashMap::from([("file_path".to_string(), "src/lib.rs".to_string())]),
            },
            Event::AwaitingAssistantResponse,
            Event::ReceivedAssistantMessage {
                message: "Perfect! I can see there's a utils module. I'll create the math_utils.rs file with the factorial function.".to_string(),
            },
            Event::AwaitingAssistantResponse,
            Event::ReceivedToolCall {
                tool_name: "write_file".to_string(),
                parameters: HashMap::from([
                    ("file_path".to_string(), "src/math_utils.rs".to_string()),
                    ("content".to_string(), "/// Calculate the factorial of a non-negative integer\n/// \n/// # Arguments\n/// \n/// * `n` - A non-negative integer\n/// \n/// # Returns\n/// \n/// The factorial of n\n/// \n/// # Examples\n/// \n/// ```\n/// assert_eq!(factorial(5), 120);\n/// assert_eq!(factorial(0), 1);\n/// ```\npub fn factorial(n: u32) -> u64 {\n    match n {\n        0 | 1 => 1,\n        _ => (2..=n as u64).product(),\n    }\n}\n\n#[cfg(test)]\nmod tests {\n    use super::*;\n\n    #[test]\n    fn test_factorial() {\n        assert_eq!(factorial(0), 1);\n        assert_eq!(factorial(1), 1);\n        assert_eq!(factorial(5), 120);\n        assert_eq!(factorial(10), 3628800);\n    }\n}".to_string())
                ]),
            },
            Event::ReceivedToolResponse {
                tool_name: "write_file".to_string(),
                response: "File written successfully".to_string(),
                parameters: HashMap::from([
                    ("file_path".to_string(), "src/math_utils.rs".to_string()),
                    ("content".to_string(), "/// Calculate the factorial of a non-negative integer\n/// \n/// # Arguments\n/// \n/// * `n` - A non-negative integer\n/// \n/// # Returns\n/// \n/// The factorial of n\n/// \n/// # Examples\n/// \n/// ```\n/// assert_eq!(factorial(5), 120);\n/// assert_eq!(factorial(0), 1);\n/// ```\npub fn factorial(n: u32) -> u64 {\n    match n {\n        0 | 1 => 1,\n        _ => (2..=n as u64).product(),\n    }\n}\n\n#[cfg(test)]\nmod tests {\n    use super::*;\n\n    #[test]\n    fn test_factorial() {\n        assert_eq!(factorial(0), 1);\n        assert_eq!(factorial(1), 1);\n        assert_eq!(factorial(5), 120);\n        assert_eq!(factorial(10), 3628800);\n    }\n}".to_string())
                ]),
            },
            Event::AwaitingAssistantResponse,
            Event::ReceivedAssistantMessage {
                message: "Great! I've created the math_utils.rs file with a factorial function. The function handles the base cases (0 and 1) and uses Rust's product iterator for efficiency. I've also included comprehensive tests.".to_string(),
            },
            // Second user message (follow-up)
            Event::ReceivedUserMessage {
                message: "Great! Can you add input validation to make sure the number is non-negative?".to_string(),
            },
            Event::AwaitingAssistantResponse,
            Event::ReceivedAssistantMessage {
                message: "Absolutely! I'll add input validation to ensure the function handles negative numbers appropriately. Let me update the function to include proper validation.".to_string(),
            },
            Event::AwaitingAssistantResponse,
            Event::ReceivedToolCall {
                tool_name: "edit_file".to_string(),
                parameters: HashMap::from([
                    ("file_path".to_string(), "src/math_utils.rs".to_string()),
                    ("search_text".to_string(), "pub fn factorial(n: u32) -> u64 {\n    match n {\n        0 | 1 => 1,\n        _ => (2..=n as u64).product(),\n    }\n}".to_string()),
                    ("replacement_text".to_string(), "pub fn factorial(n: i32) -> Result<u64, String> {\n    if n < 0 {\n        return Err(\"Factorial is not defined for negative numbers\".to_string());\n    }\n    \n    let n = n as u32;\n    match n {\n        0 | 1 => Ok(1),\n        _ => Ok((2..=n as u64).product()),\n    }\n}".to_string())
                ]),
            },
            Event::ReceivedToolResponse {
                tool_name: "edit_file".to_string(),
                response: "File edited successfully".to_string(),
                parameters: HashMap::from([
                    ("file_path".to_string(), "src/math_utils.rs".to_string()),
                    ("search_text".to_string(), "pub fn factorial(n: u32) -> u64 {\n    match n {\n        0 | 1 => 1,\n        _ => (2..=n as u64).product(),\n    }\n}".to_string()),
                    ("replacement_text".to_string(), "pub fn factorial(n: i32) -> Result<u64, String> {\n    if n < 0 {\n        return Err(\"Factorial is not defined for negative numbers\".to_string());\n    }\n    \n    let n = n as u32;\n    match n {\n        0 | 1 => Ok(1),\n        _ => Ok((2..=n as u64).product()),\n    }\n}".to_string())
                ]),
            },
            Event::AwaitingAssistantResponse,
            Event::ReceivedAssistantMessage {
                message: "Perfect! I've updated the factorial function to include input validation. The function now:\n\n1. Takes an i32 parameter to allow negative inputs\n2. Returns a Result<u64, String> to handle errors gracefully\n3. Validates that the input is non-negative\n4. Returns an appropriate error message for negative inputs\n\nThe function is now more robust and follows Rust's error handling best practices!".to_string(),
            },
            Event::SessionEnded,
        ],
    );

    // Verify the chat history includes all messages and tool calls
    assert_chat_history(
        &scenario.last_client_call().1,
        &[
            ChatMessage::System {
                content: SYSTEM_PROMPT.to_string(),
            },
            ChatMessage::User {
                content: "I need a function to calculate the factorial of a number. Please implement it in a new file called math_utils.rs".to_string(),
            },
            ChatMessage::Assistant {
                message: AssistantMessage::Content("I'll help you implement a factorial function. Let me first search for existing math-related files to understand the project structure.".to_string()),
            },
            ChatMessage::Assistant {
                message: AssistantMessage::ToolCalls(vec![ToolCall {
                    id: "1".to_string(),
                    name: "search_files".to_string(),
                    arguments: HashMap::from([("pattern".to_string(), "*math*".to_string())]),
                }]),
            },
            ChatMessage::Tool {
                content: "src/lib.rs\nsrc/main.rs\nsrc/utils/mod.rs".to_string(),
                tool_call_id: "1".to_string(),
                tool_name: "search_files".to_string(),
            },
            ChatMessage::Assistant {
                message: AssistantMessage::Content("Let me also search for any utility files to see how the project is organized.".to_string()),
            },
            ChatMessage::Assistant {
                message: AssistantMessage::ToolCalls(vec![ToolCall {
                    id: "2".to_string(),
                    name: "search_files".to_string(),
                    arguments: HashMap::from([("pattern".to_string(), "*utils*".to_string())]),
                }]),
            },
            ChatMessage::Tool {
                content: "src/lib.rs\nsrc/main.rs\nsrc/utils/mod.rs".to_string(),
                tool_call_id: "2".to_string(),
                tool_name: "search_files".to_string(),
            },
            ChatMessage::Assistant {
                message: AssistantMessage::Content("Now let me check the main library file to understand the module structure.".to_string()),
            },
            ChatMessage::Assistant {
                message: AssistantMessage::ToolCalls(vec![ToolCall {
                    id: "3".to_string(),
                    name: "read_file".to_string(),
                    arguments: HashMap::from([("file_path".to_string(), "src/lib.rs".to_string())]),
                }]),
            },
            ChatMessage::Tool {
                content: "// This is the main library file\npub mod utils;\n".to_string(),
                tool_call_id: "3".to_string(),
                tool_name: "read_file".to_string(),
            },
            ChatMessage::Assistant {
                message: AssistantMessage::Content("Perfect! I can see there's a utils module. I'll create the math_utils.rs file with the factorial function.".to_string()),
            },
            ChatMessage::Assistant {
                message: AssistantMessage::ToolCalls(vec![ToolCall {
                    id: "4".to_string(),
                    name: "write_file".to_string(),
                    arguments: HashMap::from([
                        ("file_path".to_string(), "src/math_utils.rs".to_string()),
                        ("content".to_string(), "/// Calculate the factorial of a non-negative integer\n/// \n/// # Arguments\n/// \n/// * `n` - A non-negative integer\n/// \n/// # Returns\n/// \n/// The factorial of n\n/// \n/// # Examples\n/// \n/// ```\n/// assert_eq!(factorial(5), 120);\n/// assert_eq!(factorial(0), 1);\n/// ```\npub fn factorial(n: u32) -> u64 {\n    match n {\n        0 | 1 => 1,\n        _ => (2..=n as u64).product(),\n    }\n}\n\n#[cfg(test)]\nmod tests {\n    use super::*;\n\n    #[test]\n    fn test_factorial() {\n        assert_eq!(factorial(0), 1);\n        assert_eq!(factorial(1), 1);\n        assert_eq!(factorial(5), 120);\n        assert_eq!(factorial(10), 3628800);\n    }\n}".to_string())
                    ]),
                }]),
            },
            ChatMessage::Tool {
                content: "File written successfully".to_string(),
                tool_call_id: "4".to_string(),
                tool_name: "write_file".to_string(),
            },
            ChatMessage::Assistant {
                message: AssistantMessage::Content("Great! I've created the math_utils.rs file with a factorial function. The function handles the base cases (0 and 1) and uses Rust's product iterator for efficiency. I've also included comprehensive tests.".to_string()),
            },
            ChatMessage::User {
                content: "Great! Can you add input validation to make sure the number is non-negative?".to_string(),
            },
            ChatMessage::Assistant {
                message: AssistantMessage::Content("Absolutely! I'll add input validation to ensure the function handles negative numbers appropriately. Let me update the function to include proper validation.".to_string()),
            },
            ChatMessage::Assistant {
                message: AssistantMessage::ToolCalls(vec![ToolCall {
                    id: "5".to_string(),
                    name: "edit_file".to_string(),
                    arguments: HashMap::from([
                        ("file_path".to_string(), "src/math_utils.rs".to_string()),
                        ("search_text".to_string(), "pub fn factorial(n: u32) -> u64 {\n    match n {\n        0 | 1 => 1,\n        _ => (2..=n as u64).product(),\n    }\n}".to_string()),
                        ("replacement_text".to_string(), "pub fn factorial(n: i32) -> Result<u64, String> {\n    if n < 0 {\n        return Err(\"Factorial is not defined for negative numbers\".to_string());\n    }\n    \n    let n = n as u32;\n    match n {\n        0 | 1 => Ok(1),\n        _ => Ok((2..=n as u64).product()),\n    }\n}".to_string())
                    ]),
                }]),
            },
            ChatMessage::Tool {
                content: "File edited successfully".to_string(),
                tool_call_id: "5".to_string(),
                tool_name: "edit_file".to_string(),
            },
        ],
    );

    // Verify all the expected tool calls were made
    assert_tool_calls(
        &scenario.tool_calls,
        &[
            ("search_files".to_string(), HashMap::from([("pattern".to_string(), "*math*".to_string())])),
            ("search_files".to_string(), HashMap::from([("pattern".to_string(), "*utils*".to_string())])),
            ("read_file".to_string(), HashMap::from([("file_path".to_string(), "src/lib.rs".to_string())])),
            ("write_file".to_string(), HashMap::from([
                ("file_path".to_string(), "src/math_utils.rs".to_string()),
                ("content".to_string(), "/// Calculate the factorial of a non-negative integer\n/// \n/// # Arguments\n/// \n/// * `n` - A non-negative integer\n/// \n/// # Returns\n/// \n/// The factorial of n\n/// \n/// # Examples\n/// \n/// ```\n/// assert_eq!(factorial(5), 120);\n/// assert_eq!(factorial(0), 1);\n/// ```\npub fn factorial(n: u32) -> u64 {\n    match n {\n        0 | 1 => 1,\n        _ => (2..=n as u64).product(),\n    }\n}\n\n#[cfg(test)]\nmod tests {\n    use super::*;\n\n    #[test]\n    fn test_factorial() {\n        assert_eq!(factorial(0), 1);\n        assert_eq!(factorial(1), 1);\n        assert_eq!(factorial(5), 120);\n        assert_eq!(factorial(10), 3628800);\n    }\n}".to_string())
            ])),
            ("edit_file".to_string(), HashMap::from([
                ("file_path".to_string(), "src/math_utils.rs".to_string()),
                ("search_text".to_string(), "pub fn factorial(n: u32) -> u64 {\n    match n {\n        0 | 1 => 1,\n        _ => (2..=n as u64).product(),\n    }\n}".to_string()),
                ("replacement_text".to_string(), "pub fn factorial(n: i32) -> Result<u64, String> {\n    if n < 0 {\n        return Err(\"Factorial is not defined for negative numbers\".to_string());\n    }\n    \n    let n = n as u32;\n    match n {\n        0 | 1 => Ok(1),\n        _ => Ok((2..=n as u64).product()),\n    }\n}".to_string())
            ])),
        ],
    );
}
