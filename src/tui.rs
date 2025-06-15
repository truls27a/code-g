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
    widgets::{Block, Borders, List, ListItem, Paragraph, Wrap},
    Frame, Terminal,
};
use std::{io, time::{Duration, Instant}};
use tokio::sync::mpsc;

#[derive(Clone)]
pub struct ChatMessage {
    pub role: String,
    pub content: String,
}

pub struct App {
    pub messages: Vec<ChatMessage>,
    pub input: String,
    pub should_quit: bool,
    pub is_loading: bool,
    pub loading_spinner_frame: usize,
    pub last_spinner_update: Instant,
    pub scroll_offset: usize,
}

pub enum TuiEvent {
    UserInput(String),
    Loading(bool),
    Quit,
}

const SPINNER_FRAMES: &[&str] = &["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"];
const SPINNER_INTERVAL: Duration = Duration::from_millis(150);

// Helper function to wrap text to a specific width
fn wrap_text(text: &str, width: usize) -> Vec<String> {
    if width == 0 {
        return vec![text.to_string()];
    }
    
    let mut lines = Vec::new();
    let mut current_line = String::new();
    let mut current_width = 0;
    
    for word in text.split_whitespace() {
        let word_len = word.len();
        
        // If adding this word would exceed the width, start a new line
        if current_width + word_len + (if current_line.is_empty() { 0 } else { 1 }) > width {
            if !current_line.is_empty() {
                lines.push(current_line);
                current_line = String::new();
                current_width = 0;
            }
        }
        
        // Add the word to the current line
        if !current_line.is_empty() {
            current_line.push(' ');
            current_width += 1;
        }
        current_line.push_str(word);
        current_width += word_len;
        
        // If a single word is longer than the width, we need to break it
        if current_width > width && current_line.len() == word_len {
            lines.push(current_line);
            current_line = String::new();
            current_width = 0;
        }
    }
    
    if !current_line.is_empty() {
        lines.push(current_line);
    }
    
    if lines.is_empty() {
        lines.push(String::new());
    }
    
    lines
}

// Helper function to calculate the rendered height of a message
fn calculate_message_height(message: &ChatMessage, message_width: usize) -> usize {
    let role_prefix_len = match message.role.as_str() {
        "user" => "You: ".len(),
        "assistant" => "Assistant: ".len(),
        "system" => "System: ".len(),
        _ => 0,
    };
    
    let content_width = message_width.saturating_sub(role_prefix_len);
    let wrapped_lines = wrap_text(&message.content, content_width);
    wrapped_lines.len()
}

impl Default for App {
    fn default() -> App {
        App {
            messages: vec![ChatMessage {
                role: "system".to_string(),
                content: "Welcome! Start typing your message and press Enter to send.".to_string(),
            }],
            input: String::new(),
            should_quit: false,
            is_loading: false,
            loading_spinner_frame: 0,
            last_spinner_update: Instant::now(),
            scroll_offset: 0,
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
        }
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

    pub fn get_visible_messages(&self, chat_height: usize, message_width: usize) -> (Vec<&ChatMessage>, bool) {
        let num_messages = self.messages.len();
        if num_messages == 0 {
            return (vec![], self.is_loading && self.scroll_offset == 0);
        }
        
        // Reserve space for loading indicator if at bottom and loading
        let available_height = if self.is_loading && self.scroll_offset == 0 {
            chat_height.saturating_sub(1)
        } else {
            chat_height
        };
        
        // Calculate the rendered height of each message
        let message_heights: Vec<usize> = self.messages
            .iter()
            .map(|msg| calculate_message_height(msg, message_width))
            .collect();
        
        // Calculate total rendered height of all messages
        let total_height: usize = message_heights.iter().sum();
        
        // If all messages fit, show all
        if total_height <= available_height {
            return (self.messages.iter().collect(), self.is_loading && self.scroll_offset == 0);
        }
        
        // Calculate the maximum valid scroll offset (in lines)
        let max_scroll_lines = total_height.saturating_sub(available_height);
        
        // Clamp scroll_offset to valid range
        let effective_scroll = self.scroll_offset.min(max_scroll_lines);
        
        // Determine which messages to show based on scroll offset
        let mut visible_messages = Vec::new();
        
        if effective_scroll == 0 {
            // Show from bottom (most recent messages)
            let mut accumulated_height = 0;
            for (i, &msg_height) in message_heights.iter().enumerate().rev() {
                if accumulated_height + msg_height <= available_height {
                    accumulated_height += msg_height;
                    visible_messages.insert(0, &self.messages[i]);
                } else {
                    break;
                }
            }
        } else {
            // Calculate which messages to show based on scroll offset
            let mut lines_to_skip = max_scroll_lines - effective_scroll;
            let mut start_message_idx = 0;
            
            // Find the first message to show (skip lines_to_skip lines from the top)
            for (i, &msg_height) in message_heights.iter().enumerate() {
                if lines_to_skip >= msg_height {
                    lines_to_skip -= msg_height;
                } else {
                    start_message_idx = i;
                    break;
                }
            }
            
            // Collect messages that fit in the available height
            let mut remaining_height = available_height;
            for i in start_message_idx..num_messages {
                let msg_height = message_heights[i];
                if remaining_height >= msg_height {
                    visible_messages.push(&self.messages[i]);
                    remaining_height -= msg_height;
                } else {
                    break;
                }
            }
        }
        
        let show_loading = self.is_loading && self.scroll_offset == 0;
        (visible_messages, show_loading)
    }
}

pub async fn run_tui() -> Result<(mpsc::UnboundedSender<ChatMessage>, mpsc::UnboundedReceiver<TuiEvent>)> {
    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Create channels for communication
    let (event_tx, event_rx) = mpsc::unbounded_channel::<TuiEvent>();
    let (message_tx, mut message_rx) = mpsc::unbounded_channel::<ChatMessage>();

    // Clone the event sender for the input task
    let event_tx_clone = event_tx.clone();

    // Spawn the TUI task
    let _tui_handle = tokio::spawn(async move {
        let mut app = App::default();
        
        loop {
            // Check for incoming chat messages
            while let Ok(message) = message_rx.try_recv() {
                app.add_message(message);
                app.set_loading(false); // Stop loading when we receive a message
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

    // Return the sender for messages and receiver for events
    Ok((message_tx, event_rx))
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
    
    let (visible_messages, show_loading) = app.get_visible_messages(chat_height, message_width);

    let mut messages: Vec<ListItem> = visible_messages
        .iter()
        .map(|m| {
            let (role_prefix, role_style) = match m.role.as_str() {
                "user" => ("You: ".to_string(), Style::default().fg(Color::Blue).add_modifier(Modifier::BOLD)),
                "assistant" => ("Assistant: ".to_string(), Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)),
                "system" => ("System: ".to_string(), Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
                _ => ("".to_string(), Style::default()),
            };

            // Calculate available width for content (subtract role prefix length)
            let content_width = message_width.saturating_sub(role_prefix.len());
            
            // Wrap the message content
            let wrapped_lines = wrap_text(&m.content, content_width);
            
            // Create lines with the role prefix on the first line only
            let mut lines = Vec::new();
            for (i, line) in wrapped_lines.iter().enumerate() {
                if i == 0 {
                    // First line includes the role prefix
                    lines.push(Line::from(vec![
                        Span::styled(role_prefix.clone(), role_style),
                        Span::raw(line.clone()),
                    ]));
                } else {
                    // Subsequent lines are indented to align with content
                    let indent = " ".repeat(role_prefix.len());
                    lines.push(Line::from(vec![
                        Span::raw(indent),
                        Span::raw(line.clone()),
                    ]));
                }
            }
            
            ListItem::new(Text::from(lines))
        })
        .collect();

    // Add animated loading indicator if needed
    if show_loading {
        let spinner = SPINNER_FRAMES[app.loading_spinner_frame];
        let loading_content = Text::from(vec![Line::from(vec![
            Span::styled("Assistant: ", Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)),
            Span::styled(spinner, Style::default().fg(Color::Green)),
        ])]);
        messages.push(ListItem::new(loading_content));
    }

    // Create title with scroll indicator
    let chat_title = "Chat".to_string();

    let messages_list = List::new(messages)
        .block(Block::default().borders(Borders::ALL).title(chat_title));
    f.render_widget(messages_list, chunks[0]);

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
        // Calculate cursor position considering text wrapping
        let input_width = chunks[1].width.saturating_sub(2) as usize; // Subtract borders
        let wrapped_input = wrap_text(&app.input, input_width);
        let cursor_line = wrapped_input.len().saturating_sub(1);
        let cursor_col = if wrapped_input.is_empty() {
            0
        } else {
            wrapped_input[cursor_line].len()
        };
        
        f.set_cursor_position((
            chunks[1].x + cursor_col as u16 + 1,
            chunks[1].y + cursor_line as u16 + 1,
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