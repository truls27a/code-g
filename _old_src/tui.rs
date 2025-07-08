use anyhow::Result;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEventKind, KeyModifiers},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout, Margin},
    style::{Color, Modifier, Style},
    text::{Line, Span, Text},
    widgets::{Block, Borders, Paragraph, Wrap},
    Frame, Terminal,
};
use std::{io, time::{Duration, Instant}};
use tokio::sync::mpsc;

#[derive(Clone)]
pub struct ChatMessage {
    pub role: String,
    pub content: String,
    pub message_type: MessageType,
}

#[derive(Clone)]
pub enum MessageType {
    Regular,
    ToolCall { tool_name: String },
}

pub struct App {
    pub messages: Vec<ChatMessage>,
    pub input: String,
    pub should_quit: bool,
    pub is_loading: bool,
    pub loading_spinner_frame: usize,
    pub last_spinner_update: Instant,
    pub scroll_offset: usize,
    pub current_tool_call: Option<String>,
}

pub enum TuiEvent {
    UserInput(String),
    Quit,
}

pub enum TuiCommand {
    AddMessage(ChatMessage),
    SetToolCall(String),
}

const SPINNER_FRAMES: &[&str] = &["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"];
const SPINNER_INTERVAL: Duration = Duration::from_millis(150);





impl Default for App {
    fn default() -> App {
        App {
            messages: vec![ChatMessage {
                role: "system".to_string(),
                content: "Welcome! Start typing your message and press Enter to send.".to_string(),
                message_type: MessageType::Regular,
            }],
            input: String::new(),
            should_quit: false,
            is_loading: false,
            loading_spinner_frame: 0,
            last_spinner_update: Instant::now(),
            scroll_offset: 0,
            current_tool_call: None,
        }
    }
}

impl App {
    pub fn add_message(&mut self, message: ChatMessage) {
        self.messages.push(message);
        // Auto-scroll to bottom when new message is added
        self.scroll_to_bottom();
    }

    pub fn set_loading(&mut self, loading: bool) {
        self.is_loading = loading;
        if loading {
            self.loading_spinner_frame = 0;
            self.last_spinner_update = Instant::now();
        } else {
            // Clear tool call info when loading stops
            self.current_tool_call = None;
        }
    }

    pub fn set_tool_call_info(&mut self, tool_info: String) {
        self.current_tool_call = Some(tool_info);
    }

    pub fn update_spinner(&mut self) {
        if self.is_loading && self.last_spinner_update.elapsed() >= SPINNER_INTERVAL {
            self.loading_spinner_frame = (self.loading_spinner_frame + 1) % SPINNER_FRAMES.len();
            self.last_spinner_update = Instant::now();
        }
    }

    pub fn submit_message(&mut self) -> Option<String> {
        if !self.input.trim().is_empty() {
            let message = self.input.clone();
            self.add_message(ChatMessage {
                role: "user".to_string(),
                content: message.clone(),
                message_type: MessageType::Regular,
            });
            self.input.clear();
            Some(message)
        } else {
            None
        }
    }

    pub fn scroll_to_bottom(&mut self) {
        self.scroll_offset = 0; // 0 means show from the bottom
    }

    pub fn scroll_up(&mut self) {
        // Scroll up by 3 lines for better user experience with wrapped text
        self.scroll_offset += 3;
    }

    pub fn scroll_down(&mut self) {
        // Scroll down by 3 lines, ensuring we don't go below 0
        if self.scroll_offset >= 3 {
            self.scroll_offset -= 3;
        } else {
            self.scroll_offset = 0;
        }
    }

    pub fn scroll_page_up(&mut self) {
        // Scroll up by a full page (15 lines)
        self.scroll_offset += 15;
    }

    pub fn scroll_page_down(&mut self) {
        // Scroll down by a full page (15 lines)
        if self.scroll_offset >= 15 {
            self.scroll_offset -= 15;
        } else {
            self.scroll_offset = 0;
        }
    }


}

pub async fn run_tui() -> Result<(mpsc::UnboundedSender<TuiCommand>, mpsc::UnboundedReceiver<TuiEvent>)> {
    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Create channels for communication
    let (event_tx, event_rx) = mpsc::unbounded_channel::<TuiEvent>();
    let (command_tx, mut command_rx) = mpsc::unbounded_channel::<TuiCommand>();

    // Clone the event sender for the input task
    let event_tx_clone = event_tx.clone();

    // Spawn the TUI task
    let _tui_handle = tokio::spawn(async move {
        let mut app = App::default();
        
        loop {
            // Check for incoming commands
            while let Ok(command) = command_rx.try_recv() {
                match command {
                    TuiCommand::AddMessage(message) => {
                        app.add_message(message);
                        app.set_loading(false); // Stop loading when we receive a message
                    }
                    TuiCommand::SetToolCall(tool_info) => {
                        app.set_tool_call_info(tool_info);
                    }
                }
            }

            // Update spinner animation
            app.update_spinner();

            // Draw the interface
            if let Err(_) = terminal.draw(|f| ui(f, &app)) {
                let _ = event_tx_clone.send(TuiEvent::Quit);
                break;
            }

            // Handle input events with a shorter timeout for smoother animation
            if let Ok(has_event) = event::poll(Duration::from_millis(50)) {
                if has_event {
                    if let Ok(Event::Key(key)) = event::read() {
                        if key.kind == KeyEventKind::Press {
                            // Always in editing mode - handle all keys in one place
                            match key.code {
                                KeyCode::Enter => {
                                    if !app.is_loading {
                                        if let Some(message) = app.submit_message() {
                                            app.set_loading(true); // Start loading when sending message
                                            if event_tx_clone.send(TuiEvent::UserInput(message)).is_err() {
                                                break;
                                            }
                                        }
                                    }
                                }
                                KeyCode::Char(c) => {
                                    // Handle Ctrl+C to quit
                                    if c == 'c' && key.modifiers.contains(KeyModifiers::CONTROL) {
                                        app.should_quit = true;
                                    } else if !app.is_loading {
                                        app.input.push(c);
                                    }
                                }
                                KeyCode::Backspace => {
                                    if !app.is_loading {
                                        app.input.pop();
                                    }
                                }
                                KeyCode::Up => {
                                    app.scroll_up();
                                }
                                KeyCode::Down => {
                                    app.scroll_down();
                                }
                                KeyCode::End => {
                                    app.scroll_to_bottom();
                                }
                                KeyCode::PageUp => {
                                    app.scroll_page_up();
                                }
                                KeyCode::PageDown => {
                                    app.scroll_page_down();
                                }
                                _ => {}
                            }
                        }
                    }
                }
            }

            if app.should_quit {
                let _ = event_tx_clone.send(TuiEvent::Quit);
                break;
            }
        }

        // Restore terminal
        let _ = disable_raw_mode();
        let _ = execute!(
            terminal.backend_mut(),
            LeaveAlternateScreen,
            DisableMouseCapture
        );
        let _ = terminal.show_cursor();
    });

    // Return the sender for commands and receiver for events
    Ok((command_tx, event_rx))
}

fn ui(f: &mut Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(2)
        .constraints([Constraint::Min(1), Constraint::Length(3)].as_ref())
        .split(f.area());

    // Calculate available height for chat messages
    let chat_height = chunks[0].height.saturating_sub(2) as usize; // Subtract 2 for borders
    
    // Calculate available width for message content (subtract borders and padding)
    let message_width = chunks[0].width.saturating_sub(4) as usize; // 2 for borders + 2 for padding
    
    // Build all chat content as a single scrollable text - let Paragraph handle wrapping
    let mut all_lines = Vec::new();
    
    for message in &app.messages {
        match &message.message_type {
            MessageType::ToolCall { tool_name } => {
                // Display tool calls with a special style
                all_lines.push(Line::from(vec![
                    Span::styled("Tool: ", Style::default().fg(Color::Magenta).add_modifier(Modifier::BOLD)),
                    Span::styled(tool_name, Style::default().fg(Color::Cyan).add_modifier(Modifier::ITALIC)),
                    Span::raw(" - "),
                    Span::styled(message.content.clone(), Style::default().fg(Color::Gray)),
                ]));
            }
            MessageType::Regular => {
                let (role_prefix, role_style) = match message.role.as_str() {
                    "user" => ("You: ", Style::default().fg(Color::Blue).add_modifier(Modifier::BOLD)),
                    "assistant" => ("Assistant: ", Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)),
                    "system" => ("System: ", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
                    _ => ("", Style::default()),
                };

                // Create a single line with role prefix and content, let Paragraph widget handle wrapping
                all_lines.push(Line::from(vec![
                    Span::styled(role_prefix, role_style),
                    Span::raw(message.content.clone()),
                ]));
            }
        }
        
        // Add a blank line between messages for readability
        if message.role != "system" {
            all_lines.push(Line::from(""));
        }
    }

    // Add animated loading indicator if needed
    if app.is_loading {
        let spinner = SPINNER_FRAMES[app.loading_spinner_frame];
        
        if let Some(tool_info) = &app.current_tool_call {
            // Show tool call information during loading
            all_lines.push(Line::from(vec![
                Span::styled("Assistant: ", Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)),
                Span::styled(spinner, Style::default().fg(Color::Green)),
                Span::raw(" "),
                Span::styled(tool_info, Style::default().fg(Color::Gray).add_modifier(Modifier::ITALIC)),
            ]));
        } else {
            // Show regular loading indicator
            all_lines.push(Line::from(vec![
                Span::styled("Assistant: ", Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)),
                Span::styled(spinner, Style::default().fg(Color::Green)),
            ]));
        }
    }

    // Calculate dynamic scroll based on actual content
    let total_logical_lines = all_lines.len();
    
    // Estimate the actual rendered height by accounting for text wrapping
    let mut estimated_rendered_lines = 0;
    for line in &all_lines {
        // Calculate the text content length for this line
        let line_text_length: usize = line.spans.iter()
            .map(|span| span.content.len())
            .sum();
        
        // Estimate how many wrapped lines this will create
        let available_width = message_width.max(20); // Ensure minimum width
        let wrapped_line_count = if line_text_length == 0 {
            1 // Empty lines still take 1 line
        } else {
            ((line_text_length + available_width - 1) / available_width).max(1)
        };
        
        estimated_rendered_lines += wrapped_line_count;
    }
    
    // Calculate scroll offset dynamically
    let scroll_offset = if app.scroll_offset == 0 {
        // Show from bottom - scroll to show the most recent content
        if estimated_rendered_lines > chat_height {
            estimated_rendered_lines.saturating_sub(chat_height)
        } else {
            0 // All content fits, no scrolling needed
        }
    } else {
        // When scrolling up from bottom
        let max_scroll = if estimated_rendered_lines > chat_height {
            estimated_rendered_lines.saturating_sub(chat_height)
        } else {
            0
        };
        
        if app.scroll_offset >= max_scroll {
            0 // Scrolled to top
        } else {
            max_scroll.saturating_sub(app.scroll_offset)
        }
    };
    
    // Debug logging
    if log::log_enabled!(log::Level::Debug) {
        log::debug!("Dynamic scroll: logical_lines={}, estimated_rendered={}, chat_height={}, app.scroll_offset={}, final_scroll={}", 
            total_logical_lines, estimated_rendered_lines, chat_height, app.scroll_offset, scroll_offset);
    }

    // Create the chat content as a paragraph
    let chat_content = Text::from(all_lines);
    let chat_paragraph = Paragraph::new(chat_content)
        .block(Block::default().borders(Borders::ALL).title("Chat"))
        .wrap(Wrap { trim: false })
        .scroll((scroll_offset as u16, 0));
    
    f.render_widget(chat_paragraph, chunks[0]);

    let input_title = if app.is_loading { "Input (AI is responding...)" } else { "Input" };
    let input = Paragraph::new(app.input.as_str())
        .style(if app.is_loading {
            Style::default().fg(Color::Gray)
        } else {
            Style::default().fg(Color::Yellow)
        })
        .block(Block::default().borders(Borders::ALL).title(input_title))
        .wrap(Wrap { trim: true });
    f.render_widget(input, chunks[1]);

    // Always show cursor when not loading
    if !app.is_loading {
        // Simple cursor positioning - place at end of input text
        let cursor_col = app.input.len().min(chunks[1].width.saturating_sub(2) as usize);
        
        f.set_cursor_position((
            chunks[1].x + cursor_col as u16 + 1,
            chunks[1].y + 1,
        ));
    }

    // Instructions
    let instruction_text = if app.is_loading {
        "Please wait while the assistant responds..."
    } else {
        "Type your message and press Enter to send. Use ↑/↓ to scroll, PgUp/PgDn for fast scroll, End to scroll to bottom, Ctrl+C to quit"
    };
    
    let instructions = Paragraph::new(instruction_text)
        .style(Style::default().fg(Color::Gray))
        .block(Block::default().borders(Borders::NONE))
        .wrap(Wrap { trim: true });
    
    let instruction_area = chunks[1].inner(Margin {
        vertical: 0,
        horizontal: 1,
    });
    let instruction_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(1), Constraint::Min(1)].as_ref())
        .split(instruction_area);
    
    f.render_widget(instructions, instruction_chunks[0]);
} 