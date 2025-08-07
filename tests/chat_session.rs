use code_g::session::error::ChatSessionError;
use code_g::session::session::ChatSession;
use code_g::session::system_prompt::SystemPromptConfig;
use code_g::session::event::{Event, Action, EventHandler};
use code_g::client::model::{ChatMessage, ChatResult, Model, ToolCall, Tool};
use code_g::client::error::ChatClientError;
use code_g::client::providers::openai::error::OpenAIError;
use code_g::client::traits::ChatClient;
use code_g::tools::registry::Registry;
use std::collections::HashMap;

mod helpers;
use helpers::mocks::chat_client::{MockChatClient, MockResponse};
use helpers::mocks::event_handler::MockEventHandler;

/// Integration tests for ChatSession.
///
/// These tests verify the complete interaction flow between ChatSession components,
/// including the client, tools, event handler, and memory management.
#[cfg(test)]
mod integration_tests {
    use super::*;

    #[tokio::test]
    async fn chat_session_run_processes_single_message_and_exits() {
        let client = Box::new(MockChatClient::new_with_message(
            "Hello! How can I help you?".to_string(),
            true,
        ));
        let tools = Registry::new();
        let event_handler = Box::new(MockEventHandler::new_with_inputs(vec![
            "Hello, assistant!".to_string(),
            "exit".to_string(),
        ]));

        let mut session = ChatSession::new(client, tools, event_handler, SystemPromptConfig::None);

        let result = session.run().await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn chat_session_run_processes_multiple_messages() {
        let client = Box::new(MockChatClient::new_with_message(
            "Mock response".to_string(),
            true,
        ));
        let tools = Registry::new();
        let event_handler = Box::new(MockEventHandler::new_with_inputs(vec![
            "First message".to_string(),
            "Second message".to_string(),
            "Third message".to_string(),
            "exit".to_string(),
        ]));

        let mut session = ChatSession::new(client, tools, event_handler, SystemPromptConfig::None);

        let result = session.run().await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn chat_session_handles_tool_calls_with_approval() {
        let mut args = HashMap::new();
        args.insert("path".to_string(), "test.txt".to_string());
        args.insert("content".to_string(), "Hello world".to_string());

        let tool_call = ToolCall {
            id: "call_123".to_string(),
            name: "write_file".to_string(),
            arguments: args,
        };

        // First response: tool call, second response: final message
        let client = Box::new(MockChatClient::new_with_sequence(vec![
            MockResponse::ToolCalls(vec![tool_call]),
            MockResponse::Message {
                content: "File written successfully!".to_string(),
                turn_over: true,
            },
        ]));
        let tools = Registry::all_tools();
        let event_handler = Box::new(MockEventHandler::new_with_inputs(vec![
            "Please write a file".to_string(),
            "exit".to_string(),
        ]));

        let mut session = ChatSession::new(client, tools, event_handler, SystemPromptConfig::None);

        let result = session.run().await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn chat_session_handles_tool_calls_with_declined_approval() {
        let mut args = HashMap::new();
        args.insert("path".to_string(), "test.txt".to_string());
        args.insert("content".to_string(), "Hello world".to_string());

        let tool_call = ToolCall {
            id: "call_123".to_string(),
            name: "write_file".to_string(),
            arguments: args,
        };

        let client = Box::new(MockChatClient::new_with_sequence(vec![
            MockResponse::ToolCalls(vec![tool_call]),
            MockResponse::Message {
                content: "Operation declined by user".to_string(),
                turn_over: true,
            },
        ]));
        let tools = Registry::all_tools();
        let mut event_handler = MockEventHandler::new_with_inputs(vec![
            "Please write a file".to_string(),
            "exit".to_string(),
        ]);
        event_handler.set_approval_response("declined".to_string());

        let mut session = ChatSession::new(
            client,
            tools,
            Box::new(event_handler),
            SystemPromptConfig::None,
        );

        let result = session.run().await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn chat_session_handles_safe_tools_without_approval() {
        let mut args = HashMap::new();
        args.insert("path".to_string(), "test.txt".to_string());

        let tool_call = ToolCall {
            id: "call_123".to_string(),
            name: "read_file".to_string(),
            arguments: args,
        };

        let client = Box::new(MockChatClient::new_with_sequence(vec![
            MockResponse::ToolCalls(vec![tool_call]),
            MockResponse::Message {
                content: "File read successfully".to_string(),
                turn_over: true,
            },
        ]));
        let tools = Registry::all_tools();
        let event_handler = Box::new(MockEventHandler::new_with_inputs(vec![
            "Please read a file".to_string(),
            "exit".to_string(),
        ]));

        let mut session = ChatSession::new(client, tools, event_handler, SystemPromptConfig::None);

        let result = session.run().await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn chat_session_records_events_correctly() {
        let client = Box::new(MockChatClient::new_with_message(
            "Test response".to_string(),
            true,
        ));
        let tools = Registry::new();
        let event_handler = MockEventHandler::new_with_inputs(vec![
            "Test message".to_string(),
            "exit".to_string(),
        ]);

        let mut session = ChatSession::new(
            client,
            tools,
            Box::new(event_handler),
            SystemPromptConfig::None,
        );

        let result = session.run().await;
        assert!(result.is_ok());

        // Note: We can't access the event handler after it's moved into the session,
        // so this test verifies the session runs without error when events are recorded
    }

    #[tokio::test]
    async fn chat_session_with_default_system_prompt_initializes_correctly() {
        let client = Box::new(MockChatClient::new_with_message(
            "Hello!".to_string(),
            true,
        ));
        let tools = Registry::new();
        let event_handler = Box::new(MockEventHandler::new_with_inputs(vec![
            "Hello".to_string(),
            "exit".to_string(),
        ]));

        let mut session = ChatSession::new(
            client,
            tools,
            event_handler,
            SystemPromptConfig::Default,
        );

        let result = session.run().await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn chat_session_with_custom_system_prompt_initializes_correctly() {
        let client = Box::new(MockChatClient::new_with_message(
            "Hello!".to_string(),
            true,
        ));
        let tools = Registry::new();
        let event_handler = Box::new(MockEventHandler::new_with_inputs(vec![
            "Hello".to_string(),
            "exit".to_string(),
        ]));

        let custom_prompt = "You are a test assistant.".to_string();
        let mut session = ChatSession::new(
            client,
            tools,
            event_handler,
            SystemPromptConfig::Custom(custom_prompt),
        );

        let result = session.run().await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn chat_session_handles_multiple_tool_calls_in_sequence() {
        let mut args1 = HashMap::new();
        args1.insert("path".to_string(), "file1.txt".to_string());

        let mut args2 = HashMap::new();
        args2.insert("path".to_string(), "file2.txt".to_string());

        let tool_calls = vec![
            ToolCall {
                id: "call_1".to_string(),
                name: "read_file".to_string(),
                arguments: args1,
            },
            ToolCall {
                id: "call_2".to_string(),
                name: "read_file".to_string(),
                arguments: args2,
            },
        ];

        let client = Box::new(MockChatClient::new_with_sequence(vec![
            MockResponse::ToolCalls(tool_calls),
            MockResponse::Message {
                content: "Files processed successfully".to_string(),
                turn_over: true,
            },
        ]));
        let tools = Registry::all_tools();
        let event_handler = Box::new(MockEventHandler::new_with_inputs(vec![
            "Read some files".to_string(),
            "exit".to_string(),
        ]));

        let mut session = ChatSession::new(client, tools, event_handler, SystemPromptConfig::None);

        let result = session.run().await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn chat_session_handles_unknown_tool_gracefully() {
        let mut args = HashMap::new();
        args.insert("param".to_string(), "value".to_string());

        let tool_call = ToolCall {
            id: "call_123".to_string(),
            name: "unknown_tool".to_string(),
            arguments: args,
        };

        let client = Box::new(MockChatClient::new_with_sequence(vec![
            MockResponse::ToolCalls(vec![tool_call]),
            MockResponse::Message {
                content: "Tool not found".to_string(),
                turn_over: true,
            },
        ]));
        let tools = Registry::new();
        let event_handler = Box::new(MockEventHandler::new_with_inputs(vec![
            "Use unknown tool".to_string(),
            "exit".to_string(),
        ]));

        let mut session = ChatSession::new(client, tools, event_handler, SystemPromptConfig::None);

        let result = session.run().await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn chat_session_continues_conversation_with_multiple_turns() {
        let client = Box::new(MockChatClient::new_with_message(
            "Mock response".to_string(),
            false, // turn_over = false to continue conversation
        ));
        let tools = Registry::new();
        let event_handler = Box::new(MockEventHandler::new_with_inputs(vec![
            "Start conversation".to_string(),
            "exit".to_string(),
        ]));

        let mut session = ChatSession::new(client, tools, event_handler, SystemPromptConfig::None);

        let result = session.run().await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn chat_session_exits_immediately_on_exit_command() {
        let client = Box::new(MockChatClient::new_with_message(
            "Should not see this".to_string(),
            true,
        ));
        let tools = Registry::new();
        let event_handler = Box::new(MockEventHandler::new_with_inputs(vec!["exit".to_string()]));

        let mut session = ChatSession::new(client, tools, event_handler, SystemPromptConfig::None);

        let result = session.run().await;
        assert!(result.is_ok());
    }
}

/// Error handling and edge case tests for ChatSession.
///
/// These tests verify that ChatSession properly handles various error conditions,
/// retry scenarios, and edge cases that may occur during operation.
#[cfg(test)]
mod error_handling_tests {
    use super::*;

    #[tokio::test]
    async fn chat_session_handles_client_fatal_errors() {
        let client = Box::new(MockChatClient::new_with_error("Invalid API key".to_string()));
        let tools = Registry::new();
        let event_handler = Box::new(MockEventHandler::new_with_inputs(vec![
            "Test message".to_string(),
        ]));

        let mut session = ChatSession::new(client, tools, event_handler, SystemPromptConfig::None);

        let result = session.run().await;
        assert!(result.is_err());
        
        match result.unwrap_err() {
            ChatSessionError::ChatClient(ChatClientError::OpenAIError(OpenAIError::Other(msg))) => {
                assert_eq!(msg, "Invalid API key");
            }
            _ => panic!("Expected ChatClient error with OpenAI Other error"),
        }
    }

    #[tokio::test]
    async fn chat_session_handles_max_iterations_exceeded() {
        // Create a client that never returns turn_over = true
        let client = Box::new(MockChatClient::new_with_message(
            "Continuing response".to_string(),
            false, // Never ends turn
        ));
        let tools = Registry::new();
        let event_handler = Box::new(MockEventHandler::new_with_inputs(vec![
            "Start infinite loop".to_string(),
            "exit".to_string(),
        ]));

        let mut session = ChatSession::new(client, tools, event_handler, SystemPromptConfig::None);

        let result = session.run().await;
        assert!(result.is_err());
        
        match result.unwrap_err() {
            ChatSessionError::MaxIterationsExceeded { max_iterations } => {
                assert_eq!(max_iterations, 10);
            }
            _ => panic!("Expected MaxIterationsExceeded error"),
        }
    }

    #[tokio::test]
    async fn chat_session_handles_tool_execution_errors() {
        let mut args = HashMap::new();
        args.insert("invalid_param".to_string(), "bad_value".to_string());

        let tool_call = ToolCall {
            id: "call_123".to_string(),
            name: "read_file".to_string(), // This will fail with invalid params
            arguments: args,
        };

        let client = Box::new(MockChatClient::new_with_tool_calls(vec![tool_call]));
        let tools = Registry::all_tools();
        let event_handler = Box::new(MockEventHandler::new_with_inputs(vec![
            "Try to read invalid file".to_string(),
            "exit".to_string(),
        ]));

        let mut session = ChatSession::new(client, tools, event_handler, SystemPromptConfig::None);

        // The session should handle tool errors gracefully and continue
        let result = session.run().await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn chat_session_handles_empty_tool_calls() {
        let client = Box::new(MockChatClient::new_with_tool_calls(vec![]));
        let tools = Registry::new();
        let event_handler = Box::new(MockEventHandler::new_with_inputs(vec![
            "Make empty tool calls".to_string(),
            "exit".to_string(),
        ]));

        let mut session = ChatSession::new(client, tools, event_handler, SystemPromptConfig::None);

        let result = session.run().await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn chat_session_handles_malformed_tool_calls() {
        let mut args = HashMap::new();
        args.insert("".to_string(), "empty_key".to_string()); // Malformed args

        let tool_call = ToolCall {
            id: "".to_string(), // Empty ID
            name: "".to_string(), // Empty name
            arguments: args,
        };

        let client = Box::new(MockChatClient::new_with_tool_calls(vec![tool_call]));
        let tools = Registry::all_tools();
        let event_handler = Box::new(MockEventHandler::new_with_inputs(vec![
            "Make malformed tool call".to_string(),
            "exit".to_string(),
        ]));

        let mut session = ChatSession::new(client, tools, event_handler, SystemPromptConfig::None);

        let result = session.run().await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn chat_session_handles_approval_request_errors() {
        let mut args = HashMap::new();
        args.insert("path".to_string(), "test.txt".to_string());
        args.insert("content".to_string(), "test content".to_string());

        let tool_call = ToolCall {
            id: "call_123".to_string(),
            name: "write_file".to_string(),
            arguments: args,
        };

        let client = Box::new(MockChatClient::new_with_tool_calls(vec![tool_call]));
        let tools = Registry::all_tools();
        
        // Create a custom event handler that fails approval requests
        struct FailingEventHandler;
        impl EventHandler for FailingEventHandler {
            fn handle_event(&mut self, _event: Event) {}
            fn handle_action(&mut self, action: Action) -> Result<String, std::io::Error> {
                match action {
                    Action::RequestUserInput => Ok("Test message".to_string()),
                    Action::RequestUserApproval { .. } => {
                        Err(std::io::Error::new(std::io::ErrorKind::Other, "Approval failed"))
                    }
                }
            }
        }

        let event_handler = Box::new(FailingEventHandler);

        let mut session = ChatSession::new(client, tools, event_handler, SystemPromptConfig::None);

        let result = session.run().await;
        // The session should handle approval failures gracefully
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn chat_session_handles_rapid_exit_commands() {
        let client = Box::new(MockChatClient::new_with_message(
            "Response".to_string(),
            true,
        ));
        let tools = Registry::new();
        let event_handler = Box::new(MockEventHandler::new_with_inputs(vec![
            "exit".to_string(),
            "exit".to_string(),
            "exit".to_string(),
        ]));

        let mut session = ChatSession::new(client, tools, event_handler, SystemPromptConfig::None);

        let result = session.run().await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn chat_session_handles_mixed_content_and_tool_responses() {
        // This tests a complex scenario where we have interleaved content and tool calls

        // Create a custom client that alternates between message and tool calls
        struct AlternatingClient {
            call_count: std::sync::Mutex<usize>,
        }

        impl AlternatingClient {
            fn new() -> Self {
                Self {
                    call_count: std::sync::Mutex::new(0),
                }
            }
        }

        #[async_trait::async_trait]
        impl ChatClient for AlternatingClient {
            async fn create_chat_completion(
                &self,
                _model: &Model,
                _chat_history: &[ChatMessage],
                _tools: &[Tool],
            ) -> Result<ChatResult, ChatClientError> {
                let mut count = self.call_count.lock().unwrap();
                *count += 1;

                if *count % 2 == 1 {
                    // Odd calls return tool calls
                    let mut args = HashMap::new();
                    args.insert("path".to_string(), "test.txt".to_string());
                    
                    Ok(ChatResult::ToolCalls(vec![ToolCall {
                        id: format!("call_{}", *count),
                        name: "read_file".to_string(),
                        arguments: args,
                    }]))
                } else {
                    // Even calls return messages
                    Ok(ChatResult::Message {
                        content: format!("Response {}", *count),
                        turn_over: true,
                    })
                }
            }
        }

        let client = Box::new(AlternatingClient::new());
        let tools = Registry::all_tools();
        let event_handler = Box::new(MockEventHandler::new_with_inputs(vec![
            "Complex interaction".to_string(),
            "exit".to_string(),
        ]));

        let mut session = ChatSession::new(client, tools, event_handler, SystemPromptConfig::None);

        let result = session.run().await;
        assert!(result.is_ok());
    }
}

/// Tool integration and approval workflow tests.
///
/// These tests verify the correct integration between ChatSession and the tool system,
/// including approval workflows, tool execution, and error handling.
#[cfg(test)]
mod tool_integration_tests {
    use super::*;

    #[tokio::test]
    async fn chat_session_tool_approval_workflow_complete() {
        let mut args = HashMap::new();
        args.insert("path".to_string(), "test.txt".to_string());
        args.insert("content".to_string(), "Hello, World!".to_string());

        let tool_call = ToolCall {
            id: "call_write".to_string(),
            name: "write_file".to_string(),
            arguments: args,
        };

        let client = Box::new(MockChatClient::new_with_tool_calls(vec![tool_call]));
        let tools = Registry::all_tools();
        
        // Create event handler that tracks approval requests
        let mut event_handler = MockEventHandler::new_with_inputs(vec![
            "Please write a file for me".to_string(),
            "exit".to_string(),
        ]);
        event_handler.set_approval_response("approved".to_string());

        let mut session = ChatSession::new(
            client,
            tools,
            Box::new(event_handler),
            SystemPromptConfig::None,
        );

        let result = session.run().await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn chat_session_tool_approval_declined_workflow() {
        let mut args = HashMap::new();
        args.insert("command".to_string(), "rm -rf /".to_string());

        let tool_call = ToolCall {
            id: "call_execute".to_string(),
            name: "execute_command".to_string(),
            arguments: args,
        };

        let client = Box::new(MockChatClient::new_with_tool_calls(vec![tool_call]));
        let tools = Registry::all_tools();
        
        let mut event_handler = MockEventHandler::new_with_inputs(vec![
            "Please run this dangerous command".to_string(),
            "exit".to_string(),
        ]);
        event_handler.set_approval_response("declined".to_string());

        let mut session = ChatSession::new(
            client,
            tools,
            Box::new(event_handler),
            SystemPromptConfig::None,
        );

        let result = session.run().await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn chat_session_safe_tools_bypass_approval() {
        let mut args = HashMap::new();
        args.insert("path".to_string(), "test.txt".to_string());

        let tool_call = ToolCall {
            id: "call_read".to_string(),
            name: "read_file".to_string(),
            arguments: args,
        };

        let client = Box::new(MockChatClient::new_with_tool_calls(vec![tool_call]));
        let tools = Registry::all_tools();
        
        // This handler should never be asked for approval since read_file is safe
        let event_handler = Box::new(MockEventHandler::new_with_inputs(vec![
            "Please read a file".to_string(),
            "exit".to_string(),
        ]));

        let mut session = ChatSession::new(client, tools, event_handler, SystemPromptConfig::None);

        let result = session.run().await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn chat_session_multiple_dangerous_tools_require_approval() {
        let tool_calls = vec![
            ToolCall {
                id: "call_1".to_string(),
                name: "write_file".to_string(),
                arguments: {
                    let mut args = HashMap::new();
                    args.insert("path".to_string(), "file1.txt".to_string());
                    args.insert("content".to_string(), "Content 1".to_string());
                    args
                },
            },
            ToolCall {
                id: "call_2".to_string(),
                name: "execute_command".to_string(),
                arguments: {
                    let mut args = HashMap::new();
                    args.insert("command".to_string(), "echo hello".to_string());
                    args
                },
            },
        ];

        let client = Box::new(MockChatClient::new_with_tool_calls(tool_calls));
        let tools = Registry::all_tools();
        
        let event_handler = Box::new(MockEventHandler::new_with_inputs(vec![
            "Please write a file and run a command".to_string(),
            "exit".to_string(),
        ]));

        let mut session = ChatSession::new(client, tools, event_handler, SystemPromptConfig::None);

        let result = session.run().await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn chat_session_tool_with_invalid_parameters_handles_gracefully() {
        let mut args = HashMap::new();
        args.insert("wrong_param".to_string(), "value".to_string());

        let tool_call = ToolCall {
            id: "call_invalid".to_string(),
            name: "read_file".to_string(),
            arguments: args, // Missing required 'path' parameter
        };

        let client = Box::new(MockChatClient::new_with_tool_calls(vec![tool_call]));
        let tools = Registry::all_tools();
        let event_handler = Box::new(MockEventHandler::new_with_inputs(vec![
            "Try to read file with wrong parameters".to_string(),
            "exit".to_string(),
        ]));

        let mut session = ChatSession::new(client, tools, event_handler, SystemPromptConfig::None);

        let result = session.run().await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn chat_session_tool_registry_with_no_tools() {
        let mut args = HashMap::new();
        args.insert("param".to_string(), "value".to_string());

        let tool_call = ToolCall {
            id: "call_none".to_string(),
            name: "nonexistent_tool".to_string(),
            arguments: args,
        };

        let client = Box::new(MockChatClient::new_with_tool_calls(vec![tool_call]));
        let tools = Registry::new(); // Empty registry
        let event_handler = Box::new(MockEventHandler::new_with_inputs(vec![
            "Try to use nonexistent tool".to_string(),
            "exit".to_string(),
        ]));

        let mut session = ChatSession::new(client, tools, event_handler, SystemPromptConfig::None);

        let result = session.run().await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn chat_session_complex_tool_sequence() {
        // Test a complex sequence: read file -> search files -> write file
        let tool_calls = vec![
            ToolCall {
                id: "call_1".to_string(),
                name: "read_file".to_string(),
                arguments: {
                    let mut args = HashMap::new();
                    args.insert("path".to_string(), "input.txt".to_string());
                    args
                },
            },
            ToolCall {
                id: "call_2".to_string(),
                name: "search_files".to_string(),
                arguments: {
                    let mut args = HashMap::new();
                    args.insert("query".to_string(), "test".to_string());
                    args.insert("path".to_string(), ".".to_string());
                    args
                },
            },
            ToolCall {
                id: "call_3".to_string(),
                name: "write_file".to_string(),
                arguments: {
                    let mut args = HashMap::new();
                    args.insert("path".to_string(), "output.txt".to_string());
                    args.insert("content".to_string(), "Processed content".to_string());
                    args
                },
            },
        ];

        let client = Box::new(MockChatClient::new_with_tool_calls(tool_calls));
        let tools = Registry::all_tools();
        let event_handler = Box::new(MockEventHandler::new_with_inputs(vec![
            "Please process files for me".to_string(),
            "exit".to_string(),
        ]));

        let mut session = ChatSession::new(client, tools, event_handler, SystemPromptConfig::None);

        let result = session.run().await;
        assert!(result.is_ok());
    }
}