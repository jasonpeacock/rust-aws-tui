use ratatui::{
    backend::Backend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Style},
    widgets::{Block, Borders, List, ListItem, Paragraph},
    Frame,
};

use crate::app_state::{
    function_selection::FunctionSelection, profile_selection::ProfileSelection,
};

pub fn draw_profile_selection(f: &mut Frame, state: &mut ProfileSelection) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints([
            Constraint::Length(3), // Title
            Constraint::Min(0),    // Main content
            Constraint::Length(3), // Controls
        ])
        .split(f.size());

    // Title
    let title = Paragraph::new("AWS Profile Selection")
        .style(Style::default().fg(Color::Cyan))
        .block(Block::default().borders(Borders::ALL));
    f.render_widget(title, chunks[0]);

    // Profile List
    let profiles: Vec<ListItem> = state
        .profiles
        .iter()
        .map(|profile| ListItem::new(format!("{} ({})", profile.name, profile.region)))
        .collect();

    let profiles_list = List::new(profiles)
        .block(Block::default().title("AWS Profiles").borders(Borders::ALL))
        .highlight_style(Style::default().fg(Color::Yellow).bg(Color::DarkGray));
    f.render_stateful_widget(profiles_list, chunks[1], &mut state.list_state);

    // Controls
    let controls = Paragraph::new("↑↓: Navigate profiles | Enter: Select | q: Quit")
        .style(Style::default().fg(Color::Green))
        .block(Block::default().borders(Borders::ALL));
    f.render_widget(controls, chunks[2]);
}

pub fn draw_function_selection(f: &mut Frame, state: &mut FunctionSelection) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints([
            Constraint::Length(3), // Title
            Constraint::Min(0),    // Main content
            Constraint::Length(3), // Controls
        ])
        .split(f.size());

    // Title
    let title_text = format!(
        "AWS Lambda Functions | Profile: {} | Region: {}",
        state.profile.name, state.profile.region
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
    let filter_input = Paragraph::new(state.filter_input.as_str())
        .block(Block::default().title("Filter").borders(Borders::ALL));
    f.render_widget(filter_input, inner_chunks[0]);

    // Functions list
    let functions: Vec<ListItem> = state
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
    f.render_stateful_widget(functions_list, inner_chunks[1], &mut state.list_state);

    // Controls
    let controls = Paragraph::new("↑↓: Navigate functions | Esc: Back to profiles | q: Quit")
        .style(Style::default().fg(Color::Green))
        .block(Block::default().borders(Borders::ALL));
    f.render_widget(controls, chunks[2]);
}
