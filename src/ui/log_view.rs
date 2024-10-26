use chrono::{DateTime, Local};
use ratatui::{
    layout::{Alignment, Constraint, Corner, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span, Text},
    widgets::{Block, Borders, List, ListItem, Paragraph},
    Frame,
};
use ratatui::widgets::ListState;
use crate::app_state::{
    date_selection::{DateField, DateSelection},
    log_viewer::LogViewer,
    FocusedPanel,
};

pub fn draw_log_view(
    f: &mut Frame,
    date_selection: &DateSelection,
    log_viewer: Option<&LogViewer>,
    is_loading: bool,
    focused_panel: FocusedPanel,
) {
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
            Constraint::Length(35), // Left panel (Date Selection)
            Constraint::Min(1),     // Right panel (Logs)
        ])
        .split(layout_chunks[1]);

    // Left panel (Date Selection)
    draw_date_selection_panel(f, date_selection, content_chunks[0], focused_panel);

    // Right panel (Logs)
    draw_logs_panel(f, log_viewer, is_loading, content_chunks[1], focused_panel);
}

fn draw_date_selection_panel(
    f: &mut Frame,
    date_selection: &DateSelection,
    area: ratatui::layout::Rect,
    focused_panel: FocusedPanel,
) {
    // Title bar at the top
    let layout_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Title
            Constraint::Min(0),    // Rest of content
        ])
        .margin(1)
        .split(area);

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
        "↑↓: Select Range | C: Custom | Enter: Confirm | Esc: Back"
    };

    // Helper text
    let left_help = Paragraph::new(help_text)
        .style(Style::default().fg(Color::Green))
        .alignment(Alignment::Left)
        .wrap(ratatui::widgets::Wrap { trim: true });
    f.render_widget(left_help, left_chunks[2]);
}

fn draw_logs_panel(
    f: &mut Frame,
    log_viewer: Option<&LogViewer>,
    is_loading: bool,
    area: ratatui::layout::Rect,
    focused_panel: FocusedPanel,
) {
    let right_panel = Block::default()
        .title(format!("2. Logs{}", 
            if focused_panel == FocusedPanel::Right { " [Active]" } else { "" }))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(
            if focused_panel == FocusedPanel::Right { Color::Yellow } else { Color::White }
        ));
    f.render_widget(right_panel.clone(), area);

    let inner_area = right_panel.inner(area);

    if is_loading {
        // Show loading indicator
        let loading_text = Paragraph::new("Loading logs...")
            .style(Style::default().fg(Color::Yellow))
            .alignment(Alignment::Center);
        f.render_widget(loading_text, inner_area);
        return;
    }

    let log_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Filter
            Constraint::Min(1),    // Logs
            Constraint::Length(3), // Helper text
        ])
        .margin(1)
        .split(inner_area);

    if let Some(log_viewer) = log_viewer {
        // Filter input
        let filter_input = Paragraph::new(log_viewer.filter_input.as_str())
            .block(Block::default().title("Filter").borders(Borders::ALL));
        f.render_widget(filter_input, log_layout[0]);

        // Logs content
        if log_viewer.expanded {
            draw_expanded_log(f, log_viewer, log_layout[1]);
        } else {
            draw_log_list(f, log_viewer, log_layout[1]);
        }

        // Controls
        let controls = if log_viewer.expanded {
            "Enter: Collapse | Esc: Back | q: Quit"
        } else {
            "↑↓: Navigate | Enter: Expand | Filter: Type to filter | Esc: Back | q: Quit"
        };

        let controls_widget = Paragraph::new(controls)
            .style(Style::default().fg(Color::Green))
            .block(Block::default().borders(Borders::ALL));
        f.render_widget(controls_widget, log_layout[2]);
    } else {
        // Show placeholder when no logs are loaded
        let placeholder = Paragraph::new("Select date range and press Enter to load logs")
            .style(Style::default().fg(Color::DarkGray))
            .alignment(Alignment::Center);
        f.render_widget(placeholder, inner_area);
    }
}

fn draw_expanded_log(f: &mut Frame, log_viewer: &LogViewer, area: ratatui::layout::Rect) {
    if let Some(log) = log_viewer.get_selected_log() {
        let message = log.message.as_deref().unwrap_or("");
        let timestamp = DateTime::<Local>::from(
            std::time::UNIX_EPOCH + std::time::Duration::from_millis(log.timestamp.unwrap_or(0) as u64),
        );

        // Create a more structured layout for the expanded log
        let layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3),  // Header
                Constraint::Min(1),     // Content
            ])
            .split(area);

        // Header with timestamp and metadata
        let header = Paragraph::new(vec![
            Line::from(vec![
                Span::styled("Timestamp: ", Style::default().add_modifier(Modifier::BOLD)),
                Span::styled(
                    timestamp.format("%Y-%m-%d %H:%M:%S%.3f").to_string(),
                    Style::default().fg(Color::Cyan)
                ),
            ]),
        ])
        .block(Block::default().borders(Borders::ALL).title("Log Details"));
        f.render_widget(header, layout[0]);

        // Format the message content
        let formatted_content = format_log_message(message);
        
        let content = Paragraph::new(formatted_content)
            .block(Block::default().borders(Borders::ALL).title("Message"))
            .wrap(ratatui::widgets::Wrap { trim: false })
            .scroll((log_viewer.scroll_position as u16, 0));

        f.render_widget(content, layout[1]);
    }
}

fn draw_log_list(f: &mut Frame, log_viewer: &LogViewer, area: ratatui::layout::Rect) {
    let logs_list_block = Block::default()
        .title("Logs")
        .borders(Borders::ALL);

    let logs: Vec<ListItem> = log_viewer
        .filtered_logs
        .iter()
        .enumerate()
        .map(|(i, log)| {
            let message = log.message.as_deref().unwrap_or("");
            let timestamp = DateTime::<Local>::from(
                std::time::UNIX_EPOCH + std::time::Duration::from_millis(log.timestamp.unwrap_or(0) as u64),
            );

            let mut spans = vec![Span::styled(
                format!("{} ", timestamp.format("%Y-%m-%d %H:%M:%S")),
                Style::default().fg(Color::Gray),
            )];

            // Add message with highlighting if filter is active
            if log_viewer.filter_input.is_empty() {
                spans.push(Span::raw(message));
            } else {
                add_highlighted_message(&mut spans, message, &log_viewer.filter_input);
            }

            let style = if Some(i) == log_viewer.selected_log {
                Style::default().fg(Color::Yellow).bg(Color::DarkGray)
            } else {
                Style::default()
            };
 
            ListItem::new(Line::from(spans)).style(style)
        })
        .collect();

    let logs_list = List::new(logs)
    .block(logs_list_block)
    .start_corner(Corner::TopLeft)
        .highlight_style(Style::default().fg(Color::Yellow).bg(Color::DarkGray))
        .highlight_symbol(">> ");// Optional: adds an indicator for the selected item

    f.render_widget(logs_list, area);
}

fn add_highlighted_message<'a>(spans: &mut Vec<Span<'a>>, message: &'a str, filter: &str) {
    let keywords: Vec<&str> = filter.split_whitespace().collect();
    let mut last_pos = 0;
    let mut positions: Vec<(usize, usize)> = Vec::new();

    // Find all keyword positions
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

    // Sort and deduplicate positions
    positions.sort_by_key(|k| k.0);
    positions.dedup();

    // Build spans with highlighting
    for (start, end) in positions {
        if start > last_pos {
            spans.push(Span::raw(message[last_pos..start].to_string()));
        }
        spans.push(Span::styled(
            message[start..end].to_string(),
            Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD),
        ));
        last_pos = end;
    }

    if last_pos < message.len() {
        spans.push(Span::raw(message[last_pos..].to_string()));
    }
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

// Add this new function to format log messages
fn format_log_message(message: &str) -> Vec<Line<'static>> {
    let mut lines = Vec::new();
    
    // Try to parse as JSON first
    if let Ok(json) = serde_json::from_str::<serde_json::Value>(message) {
        // Format JSON with pretty print
        let formatted = format_json(&json, 0);
        lines.extend(formatted);
    } else {
        // Handle non-JSON log messages
        for line in message.lines() {
            let line_string = line.to_string(); // Convert to owned String
            if line.contains("ERROR") || line.contains("error") {
                lines.push(Line::from(Span::styled(
                    line_string,
                    Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)
                )));
            } else if line.contains("WARN") || line.contains("warn") {
                lines.push(Line::from(Span::styled(
                    line_string,
                    Style::default().fg(Color::Yellow)
                )));
            } else if line.contains("DEBUG") || line.contains("debug") {
                lines.push(Line::from(Span::styled(
                    line_string,
                    Style::default().fg(Color::Blue)
                )));
            } else if line.contains("INFO") || line.contains("info") {
                lines.push(Line::from(Span::styled(
                    line_string,
                    Style::default().fg(Color::Green)
                )));
            } else {
                lines.push(Line::from(line_string));
            }
        }
    }

    lines
}

// Add this function to format JSON content
fn format_json(value: &serde_json::Value, indent: usize) -> Vec<Line<'static>> {
    let mut lines = Vec::new();
    let indent_str = " ".repeat(indent);

    match value {
        serde_json::Value::Object(map) => {
            lines.push(Line::from(format!("{}{{", indent_str)));
            let mut iter = map.iter().peekable();
            while let Some((key, value)) = iter.next() {
                let comma = if iter.peek().is_some() { "," } else { "" };
                match value {
                    serde_json::Value::Object(_) | serde_json::Value::Array(_) => {
                        lines.push(Line::from(vec![
                            Span::raw(format!("{}  ", indent_str)),
                            Span::styled(key.clone(), Style::default().fg(Color::Cyan)),
                            Span::raw(": "),
                        ]));
                        lines.extend(format_json(value, indent + 2));
                        if !comma.is_empty() {
                            lines.last_mut().map(|line| line.spans.push(Span::raw(comma)));
                        }
                    }
                    _ => {
                        lines.push(Line::from(vec![
                            Span::raw(format!("{}  ", indent_str)),
                            Span::styled(key.clone(), Style::default().fg(Color::Cyan)),
                            Span::raw(": "),
                            format_json_value(value),
                            Span::raw(comma),
                        ]));
                    }
                }
            }
            lines.push(Line::from(format!("{}}}", indent_str)));
        }
        serde_json::Value::Array(arr) => {
            lines.push(Line::from(format!("{}[", indent_str)));
            let mut iter = arr.iter().peekable();
            while let Some(value) = iter.next() {
                let comma = if iter.peek().is_some() { "," } else { "" };
                match value {
                    serde_json::Value::Object(_) | serde_json::Value::Array(_) => {
                        lines.extend(format_json(value, indent + 2));
                        if !comma.is_empty() {
                            lines.last_mut().map(|line| line.spans.push(Span::raw(comma)));
                        }
                    }
                    _ => {
                        lines.push(Line::from(vec![
                            Span::raw(format!("{}  ", indent_str)),
                            format_json_value(value),
                            Span::raw(comma),
                        ]));
                    }
                }
            }
            lines.push(Line::from(format!("{}]", indent_str)));
        }
        _ => {
            lines.push(Line::from(vec![format_json_value(value)]));
        }
    }

    lines
}

fn format_json_value(value: &serde_json::Value) -> Span<'static> {
    match value {
        serde_json::Value::String(s) => Span::styled(
            format!("\"{}\"", s),
            Style::default().fg(Color::Green)
        ),
        serde_json::Value::Number(n) => Span::styled(
            n.to_string(),
            Style::default().fg(Color::Yellow)
        ),
        serde_json::Value::Bool(b) => Span::styled(
            b.to_string(),
            Style::default().fg(Color::Magenta)
        ),
        serde_json::Value::Null => Span::styled(
            "null",
            Style::default().fg(Color::DarkGray)
        ),
        _ => Span::raw(value.to_string()),
    }
}
