mod config;

use anyhow::Result;
use aws_config::{BehaviorVersion, Region};
use aws_sdk_lambda::Client as LambdaClient;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Style},
    widgets::{Block, Borders, List, ListItem, Paragraph},
    Terminal,
};
use std::io;
use config::Config;

struct App {
    config: Config,
    lambda_functions: Vec<String>,
    selected_index: usize,
}

impl App {
    async fn new() -> Result<Self> {
        let config = Config::new()?;
        let lambda_functions = Self::fetch_lambda_functions(config.aws_profile_name.clone(), config.aws_region.clone()).await?;
        
        Ok(App {
            config,
            lambda_functions,
            selected_index: 0,
        })
    }

    async fn fetch_lambda_functions(profile_name: String, region: String) -> Result<Vec<String>> {
        let aws_config = aws_config::defaults(BehaviorVersion::v2024_03_28())
            .profile_name(&profile_name)
            .region(Region::new(region))
            .load()
            .await;

        let client = LambdaClient::new(&aws_config);
        let resp = client.list_functions().send().await?;
        Ok(resp.functions()
            .iter()
            .filter_map(|f| f.function_name().map(|name| name.to_string()))
            .collect())
    }

    fn next_function(&mut self) {
        if !self.lambda_functions.is_empty() {
            self.selected_index = (self.selected_index + 1) % self.lambda_functions.len();
        }
    }

    fn previous_function(&mut self) {
        if !self.lambda_functions.is_empty() {
            self.selected_index = self.selected_index.checked_sub(1)
                .unwrap_or(self.lambda_functions.len() - 1);
        }
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
                    Constraint::Length(3),  // Title & config
                    Constraint::Min(0),     // Main content
                    Constraint::Length(3),  // Controls
                ])
                .split(f.size());

            // Title with AWS Configuration
            let title_text = format!(
                "AWS TUI App (q: quit) | Profile: {} | Region: {}", 
                app.config.aws_profile_name, 
                app.config.aws_region
            );
            let title = Paragraph::new(title_text)
                .style(Style::default().fg(Color::Cyan))
                .block(Block::default().borders(Borders::ALL));
            f.render_widget(title, chunks[0]);

            // Two-column layout for Lambda Functions and Details
            let inner_chunks = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
                .split(chunks[1]);

            // Lambda Functions List (Left column)
            let functions: Vec<ListItem> = app.lambda_functions
                .iter()
                .enumerate()
                .map(|(i, name)| {
                    let style = if i == app.selected_index {
                        Style::default().fg(Color::Yellow)
                    } else {
                        Style::default()
                    };
                    ListItem::new(name.as_str()).style(style)
                })
                .collect();

            let functions_list = List::new(functions)
                .block(Block::default().title("Lambda Functions").borders(Borders::ALL));
            f.render_widget(functions_list, inner_chunks[0]);

            // Function Details (Right column)
            let details = if let Some(selected) = app.lambda_functions.get(app.selected_index) {
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
                    KeyCode::Up => app.previous_function(),
                    KeyCode::Down => app.next_function(),
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
