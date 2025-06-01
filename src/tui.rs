use anyhow::Result;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEventKind},
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
    pub input_mode: InputMode,
    pub should_quit: bool,
    pub is_loading: bool,
    pub loading_spinner_frame: usize,
    pub last_spinner_update: Instant,
}

#[derive(PartialEq)]
pub enum InputMode {
    Normal,
    Editing,
}

pub enum TuiEvent {
    UserInput(String),
    Loading(bool),
    Quit,
}

const SPINNER_FRAMES: &[&str] = &["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"];
const SPINNER_INTERVAL: Duration = Duration::from_millis(150);

impl Default for App {
    fn default() -> App {
        App {
            messages: vec![ChatMessage {
                role: "system".to_string(),
                content: "Welcome! Start typing your message and press Enter to send.".to_string(),
            }],
            input: String::new(),
            input_mode: InputMode::Editing,
            should_quit: false,
            is_loading: false,
            loading_spinner_frame: 0,
            last_spinner_update: Instant::now(),
        }
    }
}

impl App {
    pub fn add_message(&mut self, message: ChatMessage) {
        self.messages.push(message);
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
                            match app.input_mode {
                                InputMode::Normal => match key.code {
                                    KeyCode::Char('e') => {
                                        app.input_mode = InputMode::Editing;
                                    }
                                    KeyCode::Char('q') => {
                                        app.should_quit = true;
                                    }
                                    _ => {}
                                },
                                InputMode::Editing => match key.code {
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
                                        if !app.is_loading {
                                            app.input.push(c);
                                        }
                                    }
                                    KeyCode::Backspace => {
                                        if !app.is_loading {
                                            app.input.pop();
                                        }
                                    }
                                    KeyCode::Esc => {
                                        app.input_mode = InputMode::Normal;
                                    }
                                    _ => {}
                                },
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

    let mut messages: Vec<ListItem> = app
        .messages
        .iter()
        .enumerate()
        .map(|(_, m)| {
            let content = match m.role.as_str() {
                "user" => {
                    let lines = vec![Line::from(vec![
                        Span::styled("You: ", Style::default().fg(Color::Blue).add_modifier(Modifier::BOLD)),
                        Span::raw(&m.content),
                    ])];
                    Text::from(lines)
                }
                "assistant" => {
                    let lines = vec![Line::from(vec![
                        Span::styled("Assistant: ", Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)),
                        Span::raw(&m.content),
                    ])];
                    Text::from(lines)
                }
                "system" => {
                    let lines = vec![Line::from(vec![
                        Span::styled("System: ", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
                        Span::raw(&m.content),
                    ])];
                    Text::from(lines)
                }
                _ => Text::raw(&m.content),
            };
            ListItem::new(content)
        })
        .collect();

    // Add animated loading indicator if assistant is responding
    if app.is_loading {
        let spinner = SPINNER_FRAMES[app.loading_spinner_frame];
        let loading_content = Text::from(vec![Line::from(vec![
            Span::styled("Assistant: ", Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)),
            Span::styled(spinner, Style::default().fg(Color::Green)),
        ])]);
        messages.push(ListItem::new(loading_content));
    }

    let messages_list = List::new(messages)
        .block(Block::default().borders(Borders::ALL).title("Chat"));
    f.render_widget(messages_list, chunks[0]);

    let input_title = if app.is_loading { "Input (AI is responding...)" } else { "Input" };
    let input = Paragraph::new(app.input.as_str())
        .style(match app.input_mode {
            InputMode::Normal => Style::default(),
            InputMode::Editing => if app.is_loading {
                Style::default().fg(Color::Gray)
            } else {
                Style::default().fg(Color::Yellow)
            },
        })
        .block(Block::default().borders(Borders::ALL).title(input_title));
    f.render_widget(input, chunks[1]);

    match app.input_mode {
        InputMode::Normal => {}
        InputMode::Editing => {
            if !app.is_loading {
                f.set_cursor_position((
                    chunks[1].x + app.input.len() as u16 + 1,
                    chunks[1].y + 1,
                ));
            }
        }
    }

    // Instructions
    let instruction_text = if app.is_loading {
        "Please wait while the assistant responds..."
    } else {
        "Press Esc to stop editing, Enter to send message, 'q' to quit"
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