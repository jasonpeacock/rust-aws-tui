use ratatui::{
    layout::{Constraint, Direction, Layout},
    style::{Color, Style},
    widgets::{Block, Borders, List, ListItem, Paragraph},
    Frame,
};

use crate::app_state::profile_selection::ProfileSelection;

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
    let controls = Paragraph::new("↑↓ or j/k: Navigate profiles | Enter: Select | q: Quit")
        .style(Style::default().fg(Color::Green))
        .block(Block::default().borders(Borders::ALL));
    f.render_widget(controls, chunks[2]);
}
