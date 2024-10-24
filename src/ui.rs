use ratatui::{
    layout::{Alignment, Constraint, Corner, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span, Text},
    widgets::{Block, Borders, List, ListItem, Paragraph},
    Frame,
};

use crate::app_state::{
    date_selection::{DateField, DateSelection},
    function_selection::FunctionSelection,
    log_viewer::LogViewer,
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
    let title = Paragraph::new(format!(
        "Select Time Range | Profile: {} | Function: {}",
        date_selection.profile_name, date_selection.function_name
    ))
    .style(Style::default().fg(Color::Cyan))
    .block(Block::default().borders(Borders::ALL));
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

pub fn draw_log_viewer(f: &mut Frame, state: &LogViewer) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Title
            Constraint::Length(3), // Filter
            Constraint::Min(0),    // Logs
            Constraint::Length(3), // Controls
        ])
        .margin(1)
        .split(f.size());

    // Title
    let title = format!(
        "Logs for {} ({} to {})",
        state.function_name,
        state.from_date.format("%Y-%m-%d %H:%M"),
        state.to_date.format("%Y-%m-%d %H:%M")
    );
    let title_widget = Paragraph::new(title)
        .style(Style::default().fg(Color::Cyan))
        .block(Block::default().borders(Borders::ALL));
    f.render_widget(title_widget, chunks[0]);

    // Filter input
    let filter_input = Paragraph::new(state.filter_input.as_str())
        .block(Block::default().title("Filter").borders(Borders::ALL));
    f.render_widget(filter_input, chunks[1]);

    // Logs area
    if state.expanded {
        if let Some(log) = state.get_selected_log() {
            let message = log.message.as_deref().unwrap_or("");
            let timestamp = DateTime::<Local>::from(
                std::time::UNIX_EPOCH
                    + std::time::Duration::from_millis(log.timestamp.unwrap_or(0) as u64),
            );

            let header = format!("Timestamp: {}", timestamp.format("%Y-%m-%d %H:%M:%S%.3f"));
            let content = format!("\n{}", message);

            let log_detail = Paragraph::new(vec![
                Line::from(vec![Span::styled(
                    "Log Details",
                    Style::default().add_modifier(Modifier::BOLD),
                )]),
                Line::from(vec![Span::styled(header, Style::default().fg(Color::Gray))]),
                Line::from(vec![Span::raw(content)]),
            ])
            .block(
                Block::default()
                    .title("Expanded View")
                    .borders(Borders::ALL),
            )
            .wrap(ratatui::widgets::Wrap { trim: false });

            f.render_widget(log_detail, chunks[2]);
        }
    } else {
        let logs: Vec<ListItem> = state
            .filtered_logs
            .iter()
            .enumerate()
            .map(|(i, log)| {
                let message = log.message.as_deref().unwrap_or("");
                let timestamp = DateTime::<Local>::from(
                    std::time::UNIX_EPOCH
                        + std::time::Duration::from_millis(log.timestamp.unwrap_or(0) as u64),
                );

                let mut spans = vec![Span::styled(
                    format!("{} ", timestamp.format("%Y-%m-%d %H:%M:%S")),
                    Style::default().fg(Color::Gray),
                )];

                // Truncate message if it's too long
                let truncated_message = if message.len() > 100 {
                    format!("{}...", &message[..100])
                } else {
                    message.to_string()
                };

                if state.filter_input.is_empty() {
                    spans.push(Span::raw(truncated_message));
                } else {
                    // Highlight keywords in the message
                    let keywords: Vec<&str> = state.filter_input.split_whitespace().collect();
                    let mut last_pos = 0;
                    let mut positions: Vec<(usize, usize)> = Vec::new();

                    for keyword in keywords {
                        let message_lower = message.to_lowercase();
                        let keyword_lower = keyword.to_lowercase();

                        let mut start = 0;
                        while let Some(pos) = message_lower[start..].find(&keyword_lower) {
                            let abs_pos = start + pos;
                            positions.push((abs_pos, abs_pos + keyword.len()));
                            start = abs_pos + 1;
                        }
                    }

                    positions.sort_by_key(|k| k.0);
                    positions.dedup();

                    for (start, end) in positions {
                        if start > last_pos {
                            spans.push(Span::raw(&message[last_pos..start]));
                        }
                        spans.push(Span::styled(
                            &message[start..end],
                            Style::default()
                                .fg(Color::Yellow)
                                .add_modifier(Modifier::BOLD),
                        ));
                        last_pos = end;
                    }

                    if last_pos < message.len() {
                        spans.push(Span::raw(&message[last_pos..]));
                    }
                }

                let style = if Some(i) == state.selected_log {
                    Style::default().fg(Color::Yellow).bg(Color::DarkGray)
                } else {
                    Style::default()
                };

                ListItem::new(Line::from(spans)).style(style)
            })
            .collect();

        let logs_list = List::new(logs)
            .block(Block::default().title("Logs").borders(Borders::ALL))
            .start_corner(Corner::TopLeft);
        f.render_widget(logs_list, chunks[2]);
    }

    // Controls
    let controls = if state.expanded {
        "Enter: Collapse | Esc: Back | q: Quit"
    } else {
        "↑↓: Navigate | Enter: Expand | Filter: Type to filter | Esc: Back | q: Quit"
    };

    let controls_widget = Paragraph::new(controls)
        .style(Style::default().fg(Color::Green))
        .block(Block::default().borders(Borders::ALL));
    f.render_widget(controls_widget, chunks[3]);
}
