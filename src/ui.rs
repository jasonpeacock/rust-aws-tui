use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span, Text},
    widgets::{Block, Borders, List, ListItem, Paragraph},
    Frame,
};

use crate::app_state::{
    date_selection::{DateField, DateSelection},
    function_selection::FunctionSelection,
    profile_selection::ProfileSelection,
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

use chrono::{DateTime, Local};

pub fn draw_date_selection(f: &mut Frame, date_selection: &DateSelection) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Title
            Constraint::Length(3), // Quick ranges
            Constraint::Length(5), // From date
            Constraint::Length(5), // To date
            Constraint::Length(3), // Controls
        ])
        .margin(2)
        .split(f.size());

    // Title
    let title = Paragraph::new("Select Time Range")
        .style(Style::default().add_modifier(Modifier::BOLD))
        .alignment(Alignment::Center);
    f.render_widget(title, chunks[0]);

    // Quick ranges
    let quick_ranges: Vec<Span> = date_selection
        .quick_ranges
        .iter()
        .enumerate()
        .map(|(i, range)| {
            let style = if Some(i) == date_selection.selected_quick_range
                && !date_selection.custom_selection
            {
                Style::default().fg(Color::Yellow)
            } else {
                Style::default()
            };
            Span::styled(range.display_name(), style)
        })
        .collect();

    let quick_ranges_text = Text::from(Line::from(quick_ranges));
    let quick_ranges_widget = Paragraph::new(quick_ranges_text)
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::ALL));
    f.render_widget(quick_ranges_widget, chunks[1]);

    // From date
    let from_block = Block::default()
        .title("From")
        .borders(Borders::ALL)
        .border_style(
            if date_selection.is_selecting_from && date_selection.custom_selection {
                Style::default().fg(Color::Yellow)
            } else {
                Style::default()
            },
        );

    let from_text = format_date_with_highlight(
        date_selection.from_date,
        date_selection.is_selecting_from && date_selection.custom_selection,
        &date_selection.current_field,
    );
    let from = Paragraph::new(from_text)
        .block(from_block)
        .alignment(Alignment::Center);
    f.render_widget(from, chunks[2]);

    // To date
    let to_block = Block::default()
        .title("To")
        .borders(Borders::ALL)
        .border_style(
            if !date_selection.is_selecting_from && date_selection.custom_selection {
                Style::default().fg(Color::Yellow)
            } else {
                Style::default()
            },
        );

    let to_text = format_date_with_highlight(
        date_selection.to_date,
        !date_selection.is_selecting_from && date_selection.custom_selection,
        &date_selection.current_field,
    );
    let to = Paragraph::new(to_text)
        .block(to_block)
        .alignment(Alignment::Center);
    f.render_widget(to, chunks[3]);

    // Controls
    let controls = if date_selection.custom_selection {
        "Tab: Switch Date | ←→: Select Field | ↑↓: Adjust Value | C: Quick Ranges | Enter: Confirm | Esc: Back"
    } else {
        "←→: Select Range | C: Custom | Enter: Confirm | Esc: Back"
    };

    let controls_widget = Paragraph::new(controls)
        .style(Style::default().fg(Color::Green))
        .block(Block::default().borders(Borders::ALL));
    f.render_widget(controls_widget, chunks[4]);
}

fn format_date_with_highlight(
    date: DateTime<Local>,
    is_selected: bool,
    current_field: &DateField,
) -> Text {
    if !is_selected {
        return Text::raw(date.format("%Y-%m-%d %H:%M").to_string());
    }

    let year = date.format("%Y").to_string();
    let month = date.format("%m").to_string();
    let day = date.format("%d").to_string();
    let hour = date.format("%H").to_string();
    let minute = date.format("%M").to_string();

    let highlight = Style::default().fg(Color::Yellow);

    Text::from(vec![Line::from(vec![
        if matches!(current_field, DateField::Year) {
            Span::styled(year, highlight)
        } else {
            Span::raw(year)
        },
        Span::raw("-"),
        if matches!(current_field, DateField::Month) {
            Span::styled(month, highlight)
        } else {
            Span::raw(month)
        },
        Span::raw("-"),
        if matches!(current_field, DateField::Day) {
            Span::styled(day, highlight)
        } else {
            Span::raw(day)
        },
        Span::raw(" "),
        if matches!(current_field, DateField::Hour) {
            Span::styled(hour, highlight)
        } else {
            Span::raw(hour)
        },
        Span::raw(":"),
        if matches!(current_field, DateField::Minute) {
            Span::styled(minute, highlight)
        } else {
            Span::raw(minute)
        },
    ])])
}
