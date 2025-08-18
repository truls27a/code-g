# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Development Commands

**Build and Run:**
```bash
cargo build              # Build the project
cargo run                # Run the application
```

**Environment Setup:**
- Requires `OPENAI_API_KEY` environment variable to be set
- Use `.env` file for local development (dotenv support included)

## Project Architecture

CodeG is a terminal-based AI chat application built in Rust that provides an interactive coding assistant interface.

**Core Architecture:**
- **Entry Point:** `src/main.rs` - Initializes OpenAI client, tools registry, TUI, and chat session
- **Chat System:** `src/chat/` - Manages conversation state, memory, and AI interactions
  - `session.rs` - Main chat session orchestration
  - `memory.rs` - Conversation history management
  - `system_prompt.rs` - AI system prompt configuration
  - `event.rs` - Chat event handling
- **OpenAI Integration:** `src/openai/` - API client and data models for OpenAI communication
  - `client.rs` - HTTP client for OpenAI API
  - `schema.rs` - Request/response data structures
  - `model.rs` - AI model definitions
- **Tools System:** `src/tools/` - Extensible tool registry for AI capabilities
  - `registry.rs` - Central tool registration and management
  - Individual tools: `read_file.rs`, `write_file.rs`, `edit_file.rs`, `search_files.rs`, `execute_command.rs`
- **Terminal UI:** `src/tui/` - User interface rendering and interaction
  - `tui.rs` - Main TUI orchestration
  - `formatter/` - Text and output formatting for terminal display

**Key Design Patterns:**
- Tool-based architecture where AI capabilities are implemented as discrete tools
- Async/await pattern throughout for non-blocking operations
- Error handling with `thiserror` for structured error types
- Modular design with clear separation between chat logic, UI, and external integrations

**Dependencies:**
- `reqwest` - HTTP client for OpenAI API
- `tokio` - Async runtime
- `serde`/`serde_json` - JSON serialization
- `dotenv` - Environment variable management
- `thiserror` - Error handling

## Documentation Standards

Follow the Cursor rules in `.cursor/rules/documentation.mdc`:
- Document all public structs, functions, and enums with detailed rustdoc comments
- Include examples in documentation that compile and use proper imports
- Use structured sections: Arguments, Returns, Errors, Examples, Notes
- Reference other types using backticks in documentation