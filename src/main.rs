mod app_state;
mod config;
mod toml_parser;
mod ui;

use anyhow::Result;
use app_state::{
    function_selection::FunctionSelection, profile_selection::ProfileSelection, AppState,
};
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};
use std::io;
use toml_parser::read_aws_profiles;

struct App {
    state: AppState,
    profile_selection: ProfileSelection,
    function_selection: Option<FunctionSelection>,
}

impl App {
    async fn new() -> Result<Self> {
        let profiles = read_aws_profiles()?;
        Ok(App {
            state: AppState::ProfileSelection,
            profile_selection: ProfileSelection::new(profiles),
            function_selection: None,
        })
    }

    async fn select_profile(&mut self) -> Result<()> {
        if let Some(profile) = self.profile_selection.selected_profile() {
            let mut function_selection = FunctionSelection::new(profile);
            function_selection.load_functions().await?;
            self.function_selection = Some(function_selection);
            self.state = AppState::FunctionList;
        }
        Ok(())
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
        terminal.draw(|f| match app.state {
            AppState::ProfileSelection => ui::draw_profile_selection(f, &mut app.profile_selection),
            AppState::FunctionList => {
                if let Some(ref mut function_selection) = app.function_selection {
                    ui::draw_function_selection(f, function_selection)
                }
            }
        })?;

        if event::poll(std::time::Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                match app.state {
                    AppState::ProfileSelection => match key.code {
                        KeyCode::Char('q') => break,
                        KeyCode::Up => app.profile_selection.previous(),
                        KeyCode::Down => app.profile_selection.next(),
                        KeyCode::Enter => {
                            app.select_profile().await?;
                        }
                        _ => {}
                    },
                    AppState::FunctionList => {
                        if let Some(ref mut function_selection) = app.function_selection {
                            match key.code {
                                KeyCode::Char('q') => break,
                                KeyCode::Esc => {
                                    app.state = AppState::ProfileSelection;
                                    app.function_selection = None;
                                }
                                KeyCode::Up => function_selection.previous(),
                                KeyCode::Down => function_selection.next(),
                                KeyCode::Char(c) => {
                                    function_selection.filter_input.push(c);
                                    function_selection.update_filter().await?;
                                }
                                KeyCode::Backspace => {
                                    function_selection.filter_input.pop();
                                    function_selection.update_filter().await?;
                                }
                                _ => {}
                            }
                        }
                    }
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
