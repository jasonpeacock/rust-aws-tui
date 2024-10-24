mod app_state;
mod config;
mod toml_parser;
mod ui;

use anyhow::Result;
use app_state::{
    date_selection::DateSelection, function_selection::FunctionSelection, log_viewer::LogViewer,
    profile_selection::ProfileSelection, AppState,
};
use chrono::Local;
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
    date_selection: Option<DateSelection>,
    log_viewer: Option<LogViewer>,
}

impl App {
    async fn new() -> Result<Self> {
        let profiles = read_aws_profiles()?;
        Ok(App {
            state: AppState::ProfileSelection,
            profile_selection: ProfileSelection::new(profiles),
            function_selection: None,
            date_selection: None,
            log_viewer: None,
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

    fn enter_date_selection(&mut self) {
        self.date_selection = Some(DateSelection::new());
        self.state = AppState::DateSelection;
    }

    async fn enter_log_viewer(&mut self) -> Result<()> {
        if let (Some(function_selection), Some(date_selection)) =
            (&self.function_selection, &self.date_selection)
        {
            let function_name =
                function_selection.filtered_functions[function_selection.selected_index].clone();
            let mut log_viewer = LogViewer::new(
                function_name,
                date_selection.from_date,
                date_selection.to_date,
            );

            log_viewer
                .initialize(
                    function_selection.profile.name.clone(),
                    function_selection.profile.region.clone(),
                )
                .await?;

            self.log_viewer = Some(log_viewer);
            self.state = AppState::LogViewer;
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
            AppState::DateSelection => {
                if let Some(ref mut date_selection) = app.date_selection {
                    ui::draw_date_selection(f, date_selection)
                }
            }
            AppState::LogViewer => {
                if let Some(ref mut log_viewer) = app.log_viewer {
                    ui::draw_log_viewer(f, log_viewer)
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
                                KeyCode::Enter => {
                                    app.enter_date_selection();
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
                    AppState::DateSelection => {
                        if let Some(ref mut date_selection) = app.date_selection {
                            match key.code {
                                KeyCode::Char('q') => break,
                                KeyCode::Esc => {
                                    app.state = AppState::FunctionList;
                                    app.date_selection = None;
                                }
                                KeyCode::Char('c') => date_selection.toggle_custom(),
                                KeyCode::Tab if date_selection.custom_selection => {
                                    date_selection.toggle_selection()
                                }
                                KeyCode::Left => {
                                    if date_selection.custom_selection {
                                        date_selection.previous_field()
                                    } else {
                                        date_selection.previous_quick_range()
                                    }
                                }
                                KeyCode::Right => {
                                    if date_selection.custom_selection {
                                        date_selection.next_field()
                                    } else {
                                        date_selection.next_quick_range()
                                    }
                                }
                                KeyCode::Up if date_selection.custom_selection => {
                                    date_selection.adjust_current_field(true)
                                }
                                KeyCode::Down if date_selection.custom_selection => {
                                    date_selection.adjust_current_field(false)
                                }
                                KeyCode::Enter => {
                                    // Handle final selection
                                    app.enter_log_viewer().await?;
                                }
                                _ => {}
                            }
                        }
                    }
                    AppState::LogViewer => {
                        if let Some(ref mut log_viewer) = app.log_viewer {
                            match key.code {
                                KeyCode::Char('q') => break,
                                KeyCode::Esc => {
                                    app.state = AppState::DateSelection;
                                    app.log_viewer = None;
                                }
                                KeyCode::Up if !log_viewer.expanded => log_viewer.scroll_up(),
                                KeyCode::Down if !log_viewer.expanded => log_viewer.scroll_down(),
                                KeyCode::Enter => log_viewer.toggle_expand(),
                                KeyCode::Char(c) if !log_viewer.expanded => {
                                    log_viewer.filter_input.push(c);
                                    log_viewer.update_filter();
                                }
                                KeyCode::Backspace if !log_viewer.expanded => {
                                    log_viewer.filter_input.pop();
                                    log_viewer.update_filter();
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
