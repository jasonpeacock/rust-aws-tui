mod config;

use anyhow::Result;
use aws_config::{BehaviorVersion, Region};
use aws_sdk_lambda::Client as LambdaClient;
use config::Config;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::widgets::ListState;
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Style},
    widgets::{Block, Borders, List, ListItem, Paragraph},
    Terminal,
};
use std::io;

struct App {
    config: Config,
    lambda_functions: Vec<String>,
    filtered_functions: Vec<String>,
    selected_index: usize,
    filter_input: String,
    list_state: ListState,
}

impl App {
    async fn new() -> Result<Self> {
        let config = Config::new()?;
        let lambda_functions = Self::fetch_lambda_functions(
            config.aws_profile_name.clone(),
            config.aws_region.clone(),
        )
        .await?;

        Ok(App {
            config,
            lambda_functions: lambda_functions.clone(),
            filtered_functions: lambda_functions,
            selected_index: 0,
            filter_input: String::new(),
            list_state: ListState::default(),
        })
    }

    async fn fetch_lambda_functions(profile_name: String, region: String) -> Result<Vec<String>> {
        let aws_config = aws_config::defaults(BehaviorVersion::v2024_03_28())
            .profile_name(&profile_name)
            .region(Region::new(region))
            .load()
            .await;

        let client = LambdaClient::new(&aws_config);
        let mut functions = Vec::new();
        let mut next_marker = None;

        loop {
            let mut request = client.list_functions();
            if let Some(marker) = next_marker {
                request = request.marker(marker);
            }

            let resp = request.send().await?;

            if let Some(func_list) = resp.functions.as_ref() {
                functions.extend(
                    func_list
                        .iter()
                        .filter_map(|f| f.function_name().map(String::from)),
                );
            }

            next_marker = resp.next_marker().map(ToString::to_string);

            if next_marker.is_none() {
                break;
            }
        }

        Ok(functions)
    }

    fn update_filter(&mut self) {
        let keywords: Vec<String> = self
            .filter_input
            .to_lowercase()
            .split_whitespace()
            .map(String::from)
            .collect();
        self.filtered_functions = self
            .lambda_functions
            .iter()
            .filter(|&f| {
                let function_name = f.to_lowercase();
                keywords
                    .iter()
                    .all(|keyword| function_name.contains(keyword))
            })
            .cloned()
            .collect();
        self.selected_index = 0;
        self.list_state.select(Some(0));
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Create app state
    let mut app = App::new().await?;

    // Main loop
    loop {
        terminal.draw(|f| {
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .margin(1)
                .constraints([
                    Constraint::Length(3), // Title & config
                    Constraint::Min(0),    // Main content
                    Constraint::Length(3), // Controls
                ])
                .split(f.size());

            // Title with AWS Configuration
            let title_text = format!(
                "AWS TUI App (q: quit) | Profile: {} | Region: {}",
                app.config.aws_profile_name, app.config.aws_region
            );
            let title = Paragraph::new(title_text)
                .style(Style::default().fg(Color::Cyan))
                .block(Block::default().borders(Borders::ALL));
            f.render_widget(title, chunks[0]);

            // Two-column layout for Lambda Functions and Details
            let inner_chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Length(3), Constraint::Min(0)])
                .split(chunks[1]);

            // Filter input
            let filter_input = Paragraph::new(app.filter_input.as_str())
                .style(Style::default())
                .block(Block::default().title("Filter").borders(Borders::ALL));
            f.render_widget(filter_input, inner_chunks[0]);

            // Lambda Functions List (Left column)
            let functions: Vec<ListItem> = app
                .filtered_functions
                .iter()
                .map(|name| ListItem::new(name.as_str()))
                .collect();

            let functions_list = List::new(functions)
                .block(
                    Block::default()
                        .title("Lambda Functions")
                        .borders(Borders::ALL),
                )
                .highlight_style(Style::default().fg(Color::Yellow).bg(Color::DarkGray));
            f.render_stateful_widget(functions_list, inner_chunks[1], &mut app.list_state);

            // Function Details (Right column)
            let details = if let Some(selected) = app.filtered_functions.get(app.selected_index) {
                format!("Selected function: {}", selected)
            } else {
                "No function selected".to_string()
            };

            let details_widget = Paragraph::new(details)
                .style(Style::default().fg(Color::White))
                .block(Block::default().title("Details").borders(Borders::ALL));
            f.render_widget(details_widget, inner_chunks[1]);

            // Controls
            let controls = Paragraph::new("↑↓: Navigate functions | q: Quit")
                .style(Style::default().fg(Color::Green))
                .block(Block::default().borders(Borders::ALL));
            f.render_widget(controls, chunks[2]);
        })?;

        // Handle input
        if event::poll(std::time::Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Char('q') => break,
                    KeyCode::Up => {
                        if !app.filtered_functions.is_empty() {
                            app.selected_index = app.selected_index.saturating_sub(1);
                            app.list_state.select(Some(app.selected_index));
                        }
                    }
                    KeyCode::Down => {
                        if !app.filtered_functions.is_empty() {
                            app.selected_index =
                                (app.selected_index + 1).min(app.filtered_functions.len() - 1);
                            app.list_state.select(Some(app.selected_index));
                        }
                    }
                    KeyCode::Char(c) => {
                        app.filter_input.push(c);
                        app.update_filter();
                    }
                    KeyCode::Backspace => {
                        app.filter_input.pop();
                        app.update_filter();
                    }
                    _ => {}
                }
            }
        }
    }

    // Restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    Ok(())
}
