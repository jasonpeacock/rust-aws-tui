use crate::app_state::{
    date_selection::{DateField, DateSelection},
    log_viewer::LogViewer,
    FocusedPanel,
};
use chrono::{DateTime, Local};
use ratatui::{
    layout::{Alignment, Constraint, Corner, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span, Text},
    widgets::{Block, Borders, Clear, List, ListItem, Paragraph},
    Frame,
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
        "Step {}: {} | Profile: {} | Function: {}",
        if log_viewer.is_some() { "2" } else { "1" },
        if log_viewer.is_some() {
            "Log Viewer"
        } else {
            "Date Selection"
        },
        date_selection.profile_name,
        date_selection.function_name
    ))
    .style(Style::default().fg(Color::Cyan))
    .block(Block::default().borders(Borders::ALL))
    .alignment(Alignment::Center);

    f.render_widget(title, layout_chunks[0]);

    draw_logs_panel(f, log_viewer, is_loading, layout_chunks[1], focused_panel);
}

fn draw_logs_panel(
    f: &mut Frame,
    log_viewer: Option<&LogViewer>,
    is_loading: bool,
    area: ratatui::layout::Rect,
    focused_panel: FocusedPanel,
) {
    let right_panel = Block::default()
        .title(format!(
            "2. Logs{}",
            if focused_panel == FocusedPanel::Right {
                " [Active]"
            } else {
                ""
            }
        ))
        .borders(Borders::ALL)
        .border_style(
            Style::default().fg(if focused_panel == FocusedPanel::Right {
                Color::Yellow
            } else {
                Color::White
            }),
        );
    f.render_widget(right_panel.clone(), area);

    let inner_area = right_panel.inner(area);

    if is_loading {
        let loading_text = Paragraph::new("Loading logs...")
            .style(Style::default().fg(Color::Yellow))
            .alignment(Alignment::Center);
        f.render_widget(loading_text, inner_area);
        return;
    }

    if let Some(log_viewer) = log_viewer {
        let log_layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3), // Filter
                Constraint::Min(1),    // Logs
                Constraint::Length(3), // Helper text
            ])
            .margin(1)
            .split(inner_area);

        // Filter input
        let filter_input = Paragraph::new(log_viewer.filter_input.as_str())
            .block(Block::default().title("Filter").borders(Borders::ALL));
        f.render_widget(filter_input, log_layout[0]);

        // Clear the area before rendering new content
        let clear_widget = ratatui::widgets::Clear;
        f.render_widget(clear_widget, log_layout[1]);

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
        let placeholder = Paragraph::new("Select date range and press Enter to load logs")
            .style(Style::default().fg(Color::DarkGray))
            .alignment(Alignment::Center);
        f.render_widget(placeholder, inner_area);
    }
}

fn draw_expanded_log(f: &mut Frame, log_viewer: &LogViewer, area: ratatui::layout::Rect) {
    // Clear the entire area with spaces

    if let Some(log) = log_viewer.get_selected_log() {
        let message = log.message.as_deref().unwrap_or("");
        let timestamp = DateTime::<Local>::from(
            std::time::UNIX_EPOCH
                + std::time::Duration::from_millis(log.timestamp.unwrap_or(0) as u64),
        );

        let layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3), // Header
                Constraint::Min(1),    // Content
            ])
            .split(area);

        // Header
        let header = Paragraph::new(vec![Line::from(vec![
            Span::styled("Timestamp: ", Style::default().add_modifier(Modifier::BOLD)),
            Span::styled(
                timestamp.format("%Y-%m-%d %H:%M:%S%.3f").to_string(),
                Style::default().fg(Color::Cyan),
            ),
        ])])
        .block(Block::default().borders(Borders::ALL).title("Log Details"));
        f.render_widget(header, layout[0]);

        // Content
        let formatted_content = format_log_message(message);
        let content = Paragraph::new(formatted_content)
            .block(Block::default().borders(Borders::ALL).title("Message"))
            .wrap(ratatui::widgets::Wrap { trim: false })
            .scroll((log_viewer.scroll_position as u16, 0));
        f.render_widget(Clear, layout[1]);
        f.render_widget(content, layout[1]); // Uncomment this line
    }
}

fn draw_log_list(f: &mut Frame, log_viewer: &LogViewer, area: ratatui::layout::Rect) {
    // Clear the area first
    let clear_text = " ".repeat(area.width as usize);
    for y in 0..area.height {
        let clear_line =
            Paragraph::new(clear_text.clone()).style(Style::default().bg(Color::Reset));
        f.render_widget(
            clear_line,
            Rect {
                x: area.x,
                y: area.y + y,
                width: area.width,
                height: 1,
            },
        );
    }

    let available_width = area.width.saturating_sub(2) as usize;
    let timestamp_width = "YYYY-MM-DD HH:MM:SS ".len();
    let message_width = available_width.saturating_sub(timestamp_width);

    // Calculate visible range
    let visible_height = area.height.saturating_sub(2) as usize; // Subtract 2 for borders
    let total_logs = log_viewer.filtered_logs.len();
    let (start_idx, end_idx) = log_viewer.get_visible_range(visible_height);

    // Get visible logs
    let visible_logs = log_viewer
        .filtered_logs
        .iter()
        .enumerate()
        .skip(start_idx)
        .take(end_idx - start_idx);

    let logs: Vec<ListItem> = visible_logs
        .map(|(i, log)| {
            let message = log.message.as_deref().unwrap_or("");
            let timestamp = DateTime::<Local>::from(
                std::time::UNIX_EPOCH
                    + std::time::Duration::from_millis(log.timestamp.unwrap_or(0) as u64),
            );

            // Add a marker for the selected log
            let timestamp_prefix = if Some(i) == log_viewer.selected_log {
                "→ "
            } else {
                "  "
            };

            let timestamp_span = Span::styled(
                format!(
                    "{}{} ",
                    timestamp_prefix,
                    timestamp.format("%Y-%m-%d %H:%M:%S")
                ),
                Style::default().fg(Color::Gray),
            );

            let wrapped_message = textwrap::wrap(message, message_width);
            let mut lines = Vec::new();

            // First line with timestamp
            let mut first_line_spans = vec![timestamp_span];
            if let Some(first_msg) = wrapped_message.first() {
                if log_viewer.filter_input.is_empty() {
                    first_line_spans.push(Span::raw(first_msg.to_string()));
                } else {
                    add_highlighted_message_spans(
                        &mut first_line_spans,
                        first_msg,
                        &log_viewer.filter_input,
                    );
                }
            }
            lines.push(Line::from(first_line_spans));

            // Remaining lines with proper indentation
            for msg in wrapped_message.iter().skip(1) {
                let mut line_spans = vec![
                    Span::raw(" ".repeat(timestamp_width + 2)), // +2 for the arrow/space prefix
                ];
                if log_viewer.filter_input.is_empty() {
                    line_spans.push(Span::raw(msg.to_string()));
                } else {
                    add_highlighted_message_spans(&mut line_spans, msg, &log_viewer.filter_input);
                }
                lines.push(Line::from(line_spans));
            }

            let style = if Some(i) == log_viewer.selected_log {
                Style::default().fg(Color::Yellow).bg(Color::DarkGray)
            } else {
                Style::default()
            };

            ListItem::new(lines).style(style)
        })
        .collect();

    // Calculate scroll percentage
    let scroll_percentage = if total_logs > visible_height {
        (start_idx as f64 / (total_logs - visible_height) as f64 * 100.0) as u16
    } else {
        100
    };

    let logs_list = List::new(logs)
        .block(
            Block::default()
                .title(format!(
                    "Logs ({}/{}) {}%",
                    log_viewer.selected_log.map_or(0, |i| i + 1),
                    total_logs,
                    scroll_percentage
                ))
                .borders(Borders::ALL),
        )
        .start_corner(Corner::TopLeft);

    f.render_widget(logs_list, area);
}

fn add_highlighted_message_spans(spans: &mut Vec<Span<'static>>, text: &str, filter: &str) {
    let keywords: Vec<&str> = filter.split_whitespace().collect();
    let mut last_pos = 0;
    let mut positions: Vec<(usize, usize)> = Vec::new();

    // Find all keyword positions
    for keyword in keywords {
        let text_lower = text.to_lowercase();
        let keyword_lower = keyword.to_lowercase();

        let mut start = 0;
        while let Some(pos) = text_lower[start..].find(&keyword_lower) {
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
            spans.push(Span::raw(text[last_pos..start].to_string()));
        }
        spans.push(Span::styled(
            text[start..end].to_string(),
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        ));
        last_pos = end;
    }

    if last_pos < text.len() {
        spans.push(Span::raw(text[last_pos..].to_string()));
    }
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
                    Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
                )));
            } else if line.contains("WARN") || line.contains("warn") {
                lines.push(Line::from(Span::styled(
                    line_string,
                    Style::default().fg(Color::Yellow),
                )));
            } else if line.contains("DEBUG") || line.contains("debug") {
                lines.push(Line::from(Span::styled(
                    line_string,
                    Style::default().fg(Color::Blue),
                )));
            } else if line.contains("INFO") || line.contains("info") {
                lines.push(Line::from(Span::styled(
                    line_string,
                    Style::default().fg(Color::Green),
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
                            lines
                                .last_mut()
                                .map(|line| line.spans.push(Span::raw(comma)));
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
                            lines
                                .last_mut()
                                .map(|line| line.spans.push(Span::raw(comma)));
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
        serde_json::Value::String(s) => {
            Span::styled(format!("\"{}\"", s), Style::default().fg(Color::Green))
        }
        serde_json::Value::Number(n) => {
            Span::styled(n.to_string(), Style::default().fg(Color::Yellow))
        }
        serde_json::Value::Bool(b) => {
            Span::styled(b.to_string(), Style::default().fg(Color::Magenta))
        }
        serde_json::Value::Null => Span::styled("null", Style::default().fg(Color::DarkGray)),
        _ => Span::raw(value.to_string()),
    }
}
