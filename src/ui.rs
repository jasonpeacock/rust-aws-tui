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

use chrono::{DateTime, Local};

pub fn draw_date_selection(f: &mut Frame, date_selection: &DateSelection) {
    // Title bar at the top
    let layout_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Title
            Constraint::Min(0),    // Rest of content
        ])
        .margin(1)
        .split(f.size());

    let title = Paragraph::new(format!(
        "Log Viewer | Profile: {} | Function: {}",
        date_selection.profile_name, date_selection.function_name
    ))
    .style(Style::default().fg(Color::Cyan))
    .block(Block::default().borders(Borders::ALL))
    .alignment(Alignment::Center);

    f.render_widget(title, layout_chunks[0]);

    // Split into left and right panels
    let content_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Length(35), // Left panel
            Constraint::Min(1),     // Right panel
        ])
        .split(layout_chunks[1]);

    // Left panel with its border
    let left_panel = Block::default()
        .borders(Borders::ALL)
        .style(Style::default());
    f.render_widget(left_panel.clone(), content_chunks[0]);

    // Left panel inner layout
    let left_inner = left_panel.inner(content_chunks[0]);
    let left_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(12), // Quick ranges
            Constraint::Length(12), // Custom range
            Constraint::Min(0),     // Helper text
        ])
        .split(left_inner);

    // Quick ranges section with focus state
    let quick_ranges_style = if !date_selection.custom_selection {
        Style::default().fg(Color::Yellow).bg(Color::DarkGray)
    } else {
        Style::default()
    };

    let quick_ranges: Vec<ListItem> = date_selection
        .quick_ranges
        .iter()
        .enumerate()
        .map(|(i, range)| {
            let style = if Some(i) == date_selection.selected_quick_range
                && !date_selection.custom_selection
            {
                Style::default().fg(Color::Yellow).bg(Color::DarkGray)
            } else {
                Style::default()
            };
            ListItem::new(range.display_name()).style(style)
        })
        .collect();

    let quick_ranges_list = List::new(quick_ranges)
        .block(
            Block::default()
                .title("Quick Ranges")
                .title_style(quick_ranges_style)
                .borders(Borders::ALL),
        )
        .highlight_style(Style::default().fg(Color::Yellow).bg(Color::DarkGray));
    f.render_widget(quick_ranges_list, left_chunks[0]);

    // Custom range section with focus state
    let custom_range_style = if date_selection.custom_selection {
        Style::default().fg(Color::Yellow).bg(Color::DarkGray)
    } else {
        Style::default()
    };

    let custom_range_block = Block::default()
        .title("Custom Range")
        .title_style(custom_range_style)
        .borders(Borders::ALL);
    let custom_range_area = custom_range_block.inner(left_chunks[1]);
    f.render_widget(custom_range_block, left_chunks[1]);

    // From and To fields layout
    let date_fields = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1), // From label
            Constraint::Length(3), // From input
            Constraint::Length(1), // To label
            Constraint::Length(3), // To input
        ])
        .margin(1)
        .split(custom_range_area);

    // From label and input with focus state
    let from_style = if date_selection.is_selecting_from && date_selection.custom_selection {
        Style::default().fg(Color::Yellow)
    } else {
        Style::default()
    };
    let from_label = Paragraph::new("From").style(from_style);
    f.render_widget(from_label, date_fields[0]);

    let from_text = format_date_with_highlight(
        date_selection.from_date,
        date_selection.is_selecting_from && date_selection.custom_selection,
        &date_selection.current_field,
    );
    let from_input = Paragraph::new(from_text)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(from_style),
        )
        .alignment(Alignment::Left);
    f.render_widget(from_input, date_fields[1]);

    // To label and input with focus state
    let to_style = if !date_selection.is_selecting_from && date_selection.custom_selection {
        Style::default().fg(Color::Yellow)
    } else {
        Style::default()
    };
    let to_label = Paragraph::new("To").style(to_style);
    f.render_widget(to_label, date_fields[2]);

    let to_text = format_date_with_highlight(
        date_selection.to_date,
        !date_selection.is_selecting_from && date_selection.custom_selection,
        &date_selection.current_field,
    );
    let to_input = Paragraph::new(to_text)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(to_style),
        )
        .alignment(Alignment::Left);
    f.render_widget(to_input, date_fields[3]);

    // Update help text based on focus state
    let help_text = if date_selection.custom_selection {
        if date_selection.is_selecting_from {
            "Tab: Switch to To | ←→: Select Field | ↑↓: Adjust Value | C: Quick Ranges | Enter: Confirm | Esc: Back"
        } else {
            "Tab: Switch to From | ←→: Select Field | ↑↓: Adjust Value | C: Quick Ranges | Enter: Confirm | Esc: Back"
        }
    } else {
        "←→: Select Range | C: Custom | Enter: Confirm | Esc: Back"
    };

    let left_help = Paragraph::new(help_text)
        .style(Style::default().fg(Color::Green))
        .alignment(Alignment::Left);
    f.render_widget(left_help, left_chunks[2]);

    // Right panel with its border
    let right_panel = Block::default()
        .borders(Borders::ALL)
        .style(Style::default());
    f.render_widget(right_panel.clone(), content_chunks[1]);

    // Right panel inner layout
    let right_inner = right_panel.inner(content_chunks[1]);
    let right_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Filter
            Constraint::Min(1),    // Logs
            Constraint::Length(3), // Helper text
        ])
        .margin(1)
        .split(right_inner);

    // Filter
    let filter = Block::default().title("Filter").borders(Borders::ALL);
    f.render_widget(filter, right_chunks[0]);

    // Logs
    let logs = Block::default().title("Logs").borders(Borders::ALL);
    f.render_widget(logs, right_chunks[1]);

    // Right panel helper text
    let right_help = Paragraph::new("Helper Text")
        .style(Style::default().fg(Color::Green))
        .alignment(Alignment::Left);
    f.render_widget(right_help, right_chunks[2]);
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

fn format_date_with_highlight(
    date: DateTime<Local>,
    is_selected: bool,
    current_field: &DateField,
) -> Text<'static> {
    let date_str = date.format("%Y-%m-%d %H:%M").to_string();
    let mut spans = Vec::new();

    if !is_selected {
        spans.push(Span::raw(date_str));
    } else {
        // Create owned strings first
        let date_parts = (
            date_str[0..4].to_string(),   // Year
            date_str[5..7].to_string(),   // Month
            date_str[8..10].to_string(),  // Day
            date_str[11..13].to_string(), // Hour
            date_str[14..16].to_string(), // Minute
        );

        // Styles for different states
        let highlight_style = Style::default()
            .fg(Color::Black)
            .bg(Color::Yellow)
            .add_modifier(Modifier::BOLD);
        let active_style = Style::default().fg(Color::Yellow);
        let normal_style = Style::default();

        spans.extend(vec![
            Span::styled(
                date_parts.0,
                if matches!(current_field, DateField::Year) {
                    highlight_style
                } else {
                    active_style
                },
            ),
            Span::styled("-", normal_style),
            Span::styled(
                date_parts.1,
                if matches!(current_field, DateField::Month) {
                    highlight_style
                } else {
                    active_style
                },
            ),
            Span::styled("-", normal_style),
            Span::styled(
                date_parts.2,
                if matches!(current_field, DateField::Day) {
                    highlight_style
                } else {
                    active_style
                },
            ),
            Span::styled(" ", normal_style),
            Span::styled(
                date_parts.3,
                if matches!(current_field, DateField::Hour) {
                    highlight_style
                } else {
                    active_style
                },
            ),
            Span::styled(":", normal_style),
            Span::styled(
                date_parts.4,
                if matches!(current_field, DateField::Minute) {
                    highlight_style
                } else {
                    active_style
                },
            ),
        ]);
    }

    // Convert spans to owned data
    let owned_spans: Vec<Span<'static>> = spans
        .into_iter()
        .map(|span| Span::styled(span.content.to_string(), span.style))
        .collect();

    Text::from(Line::from(owned_spans))
}
