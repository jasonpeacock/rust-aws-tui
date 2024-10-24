mod config;
mod toml_parser;

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
use toml_parser::{read_aws_profiles, Profile};

enum AppState {
    ProfileSelection,
    FunctionList,
}

struct App {
    state: AppState,
    profile_list_state: ListState,
    selected_profile: Option<Profile>,
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
        let mut profile_list_state = ListState::default();
        if !config.aws_profiles.is_empty() {
            profile_list_state.select(Some(0));
        }

        Ok(App {
            state: AppState::ProfileSelection,
            profile_list_state,
            selected_profile: None,
            config,
            lambda_functions: Vec::new(),
            filtered_functions: Vec::new(),
            selected_index: 0,
            filter_input: String::new(),
            list_state: ListState::default(),
        })
    }

    async fn select_profile(&mut self, profile: Profile) -> Result<()> {
        self.selected_profile = Some(profile.clone());
        self.lambda_functions =
            Self::fetch_lambda_functions(profile.name.clone(), profile.region.clone()).await?;
        self.filtered_functions = self.lambda_functions.clone();
        self.state = AppState::FunctionList;
        self.list_state.select(Some(0));
        Ok(())
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
                    Constraint::Length(3), // Title
                    Constraint::Min(0),    // Main content
                    Constraint::Length(3), // Controls
                ])
                .split(f.size());

            match app.state {
                AppState::ProfileSelection => {
                    // Title
                    let title = Paragraph::new("AWS Profile Selection")
                        .style(Style::default().fg(Color::Cyan))
                        .block(Block::default().borders(Borders::ALL));
                    f.render_widget(title, chunks[0]);

                    // Profile List
                    let profiles: Vec<ListItem> = app
                        .config
                        .aws_profiles
                        .iter()
                        .map(|profile| {
                            ListItem::new(format!("{} ({})", profile.name, profile.region))
                        })
                        .collect();

                    let profiles_list = List::new(profiles)
                        .block(Block::default().title("AWS Profiles").borders(Borders::ALL))
                        .highlight_style(Style::default().fg(Color::Yellow).bg(Color::DarkGray));
                    f.render_stateful_widget(profiles_list, chunks[1], &mut app.profile_list_state);

                    // Controls
                    let controls =
                        Paragraph::new("↑↓: Navigate profiles | Enter: Select | q: Quit")
                            .style(Style::default().fg(Color::Green))
                            .block(Block::default().borders(Borders::ALL));
                    f.render_widget(controls, chunks[2]);
                }
                AppState::FunctionList => {
                    // Title with selected profile
                    let profile = app.selected_profile.as_ref().unwrap();
                    let title_text = format!(
                        "AWS Lambda Functions | Profile: {} | Region: {}",
                        profile.name, profile.region
                    );
                    let title = Paragraph::new(title_text)
                        .style(Style::default().fg(Color::Cyan))
                        .block(Block::default().borders(Borders::ALL));
                    f.render_widget(title, chunks[0]);

                    // Function list layout
                    let inner_chunks = Layout::default()
                        .direction(Direction::Vertical)
                        .constraints([Constraint::Length(3), Constraint::Min(0)])
                        .split(chunks[1]);

                    // Filter input
                    let filter_input = Paragraph::new(app.filter_input.as_str())
                        .block(Block::default().title("Filter").borders(Borders::ALL));
                    f.render_widget(filter_input, inner_chunks[0]);

                    // Functions list
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

                    // Controls
                    let controls =
                        Paragraph::new("↑↓: Navigate functions | Esc: Back to profiles | q: Quit")
                            .style(Style::default().fg(Color::Green))
                            .block(Block::default().borders(Borders::ALL));
                    f.render_widget(controls, chunks[2]);
                }
            }
        })?;

        // Handle input
        if event::poll(std::time::Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                match app.state {
                    AppState::ProfileSelection => match key.code {
                        KeyCode::Char('q') => break,
                        KeyCode::Up => {
                            if !app.config.aws_profiles.is_empty() {
                                let current = app.profile_list_state.selected().unwrap_or(0);
                                let next = current.saturating_sub(1);
                                app.profile_list_state.select(Some(next));
                            }
                        }
                        KeyCode::Down => {
                            if !app.config.aws_profiles.is_empty() {
                                let current = app.profile_list_state.selected().unwrap_or(0);
                                let next = (current + 1).min(app.config.aws_profiles.len() - 1);
                                app.profile_list_state.select(Some(next));
                            }
                        }
                        KeyCode::Enter => {
                            if let Some(selected) = app.profile_list_state.selected() {
                                let profile = app.config.aws_profiles[selected].clone();
                                app.select_profile(profile).await?;
                            }
                        }
                        _ => {}
                    },
                    AppState::FunctionList => match key.code {
                        KeyCode::Char('q') => break,
                        KeyCode::Esc => {
                            app.state = AppState::ProfileSelection;
                        }
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
                    },
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
