use ratatui::{
    layout::{Constraint, Direction, Layout},
    style::{Color, Style},
    widgets::{Block, Borders, List, ListItem, Paragraph},
    Frame,
};

use crate::app_state::function_selection::FunctionSelection;

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

    // Functions list with scroll state
    let total_functions = state.filtered_functions.len();
    let items_per_page = inner_chunks[1].height as usize - 2; // Subtract 2 for borders

    // Calculate scroll position
    let selected_index = state.list_state.selected().unwrap_or(0);
    let scroll_threshold = items_per_page / 2;
    let scroll_offset = if selected_index > scroll_threshold {
        selected_index - scroll_threshold
    } else {
        0
    };

    // Create visible items
    let visible_items: Vec<ListItem> = state
        .filtered_functions
        .iter()
        .skip(scroll_offset)
        .take(items_per_page)
        .enumerate()
        .map(|(i, name)| {
            let display_text = if name.len() > inner_chunks[1].width as usize - 4 {
                format!("{}...", &name[..inner_chunks[1].width as usize - 7])
            } else {
                name.clone()
            };

            let style = if i + scroll_offset == selected_index {
                Style::default().fg(Color::Yellow).bg(Color::DarkGray)
            } else {
                Style::default()
            };

            ListItem::new(display_text).style(style)
        })
        .collect();

    // Create scroll indicator
    let scroll_indicator = if total_functions > items_per_page {
        let progress =
            (scroll_offset as f64 / (total_functions - items_per_page) as f64 * 100.0) as u16;
        format!(
            " ({}/{}) {}%",
            selected_index + 1,
            total_functions,
            progress
        )
    } else {
        format!(" ({}/{})", selected_index + 1, total_functions)
    };

    let functions_list = List::new(visible_items).block(
        Block::default()
            .title(format!("Lambda Functions{}", scroll_indicator))
            .borders(Borders::ALL),
    ); // Removed highlight_style
    f.render_stateful_widget(functions_list, inner_chunks[1], &mut state.list_state);

    // Controls
    let controls = if total_functions > items_per_page {
        "↑↓: Navigate | PgUp/PgDn: Scroll | Enter: Select | Esc: Back to profiles | q: Quit"
    } else {
        "↑↓: Navigate | Enter: Select | Esc: Back to profiles | q: Quit"
    };

    let controls_widget = Paragraph::new(controls)
        .style(Style::default().fg(Color::Green))
        .block(Block::default().borders(Borders::ALL));
    f.render_widget(controls_widget, chunks[2]);
}
