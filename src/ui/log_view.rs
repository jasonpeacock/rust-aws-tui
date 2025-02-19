use crate::{
    app_state::{
        date_selection::{DateField, DateSelection},
        log_viewer::LogViewer,
        FocusedPanel,
    },
    utils::ui_utils::format_json,
};
use chrono::{DateTime, Local};
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Margin, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span, Text},
    widgets::{
        Block, Borders, Clear, List, ListItem, Paragraph, Scrollbar, ScrollbarOrientation,
        ScrollbarState,
    },
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
    f.render_widget(Clear, area);
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

        // Header with timestamp
        let header = Paragraph::new(vec![Line::from(vec![
            Span::styled("Timestamp: ", Style::default().add_modifier(Modifier::BOLD)),
            Span::styled(
                timestamp.format("%Y-%m-%d %H:%M:%S%.3f").to_string(),
                Style::default().fg(Color::Cyan),
            ),
        ])])
        .block(Block::default().borders(Borders::ALL).title("Log Details"));
        f.render_widget(header, layout[0]);

        // Format message content
        let formatted_content =
            if let Ok(json_value) = serde_json::from_str::<serde_json::Value>(message) {
                // If it's valid JSON, format it nicely
                let formatted_lines = format_json(&json_value, 0);
                Text::from(formatted_lines)
            } else {
                // If it's not JSON, format as regular log message
                Text::from(format_log_message(message))
            };

        // Content area with scrollbar
        let content_area = layout[1];
        let inner_area = content_area.inner(Margin {
            vertical: 1,
            horizontal: 1,
        });

        // Count actual lines after formatting
        let line_count = formatted_content.lines.len();
        let viewport_height = inner_area.height as usize;

        // Create content paragraph with scroll
        let content = Paragraph::new(formatted_content)
            .block(Block::default().borders(Borders::ALL).title(format!(
                "Message (Line {} of {})",
                log_viewer.scroll_position + 1,
                line_count
            )))
            .wrap(ratatui::widgets::Wrap { trim: false })
            .scroll((log_viewer.scroll_position as u16, 0));

        f.render_widget(content, content_area);

        // Only render scrollbar if content is scrollable
        if line_count > viewport_height {
            let mut scrollbar_state = ScrollbarState::default()
                .content_length(line_count)
                .position(log_viewer.scroll_position);

            f.render_stateful_widget(
                Scrollbar::new(ScrollbarOrientation::VerticalRight)
                    .begin_symbol(Some("↑"))
                    .end_symbol(Some("↓")),
                content_area.inner(Margin {
                    vertical: 1,
                    horizontal: 0,
                }),
                &mut scrollbar_state,
            );
        }
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

    let available_width = area.width.saturating_sub(4) as usize; // Subtract 4 for borders and scrollbar
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

            let mut lines = Vec::new();
            let message_lines: Vec<&str> = message.lines().collect();

            // Process first line with timestamp
            if let Some(first_msg) = message_lines.first() {
                let mut first_line_spans = vec![timestamp_span];
                let truncated_msg = truncate_to_width(first_msg, message_width);

                if log_viewer.filter_input.is_empty() {
                    first_line_spans.push(Span::raw(truncated_msg));
                } else {
                    add_highlighted_message_spans(
                        &mut first_line_spans,
                        &truncated_msg,
                        &log_viewer.filter_input,
                    );
                }
                lines.push(Line::from(first_line_spans));
            }

            // Process remaining lines with indentation
            for msg in message_lines.iter().skip(1).take(2) {
                // Show max 3 lines per log
                let mut line_spans = vec![Span::raw(" ".repeat(timestamp_width + 2))];
                let truncated_msg = truncate_to_width(msg, message_width);

                if log_viewer.filter_input.is_empty() {
                    line_spans.push(Span::raw(truncated_msg));
                } else {
                    add_highlighted_message_spans(
                        &mut line_spans,
                        &truncated_msg,
                        &log_viewer.filter_input,
                    );
                }
                lines.push(Line::from(line_spans));
            }

            // Add ellipsis if there are more lines
            if message_lines.len() > 3 {
                lines.push(Line::from(vec![
                    Span::raw(" ".repeat(timestamp_width + 2)),
                    Span::styled("...", Style::default().fg(Color::DarkGray)),
                ]));
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

    let logs_list = List::new(logs).block(
        Block::default()
            .title(format!(
                "Logs ({}/{}) {}%",
                log_viewer.selected_log.map_or(0, |i| i + 1),
                total_logs,
                scroll_percentage
            ))
            .borders(Borders::ALL),
    );

    f.render_widget(Clear, area);
    f.render_widget(logs_list, area);

    // Add scrollbar if there are more logs than visible space
    if total_logs > visible_height {
        // Update scrollbar position to follow selected item
        let scrollbar_position = if let Some(selected_idx) = log_viewer.selected_log {
            // Ensure selected item is always visible in the scrollbar viewport
            if selected_idx >= start_idx && selected_idx < end_idx {
                selected_idx // Use selected index as position when it's in view
            } else {
                start_idx // Otherwise use the current scroll position
            }
        } else {
            start_idx
        };

        let mut scrollbar_state = ScrollbarState::default()
            .content_length(total_logs)
            .position(scrollbar_position);

        f.render_stateful_widget(
            Scrollbar::new(ScrollbarOrientation::VerticalRight)
                .begin_symbol(Some("↑"))
                .end_symbol(Some("↓")),
            area.inner(Margin {
                vertical: 1,
                horizontal: 0,
            }),
            &mut scrollbar_state,
        );
    }
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

// Add this helper function at the end of the file
fn wrap_text(text: &str, width: usize) -> Vec<String> {
    let mut wrapped = Vec::new();
    let mut line = String::new();
    let mut line_length = 0;

    for word in text.split_whitespace() {
        let word_length = word.len();

        if line_length + word_length + 1 <= width {
            // Add word to current line
            if !line.is_empty() {
                line.push(' ');
                line_length += 1;
            }
            line.push_str(word);
            line_length += word_length;
        } else {
            // Start new line
            if !line.is_empty() {
                wrapped.push(line);
            }
            line = word.to_string();
            line_length = word_length;
        }
    }

    if !line.is_empty() {
        wrapped.push(line);
    }

    // Handle empty input or single long words
    if wrapped.is_empty() {
        wrapped.push(text.to_string());
    }

    wrapped
}

// Add this helper function to truncate text
fn truncate_to_width(text: &str, width: usize) -> String {
    if text.len() <= width {
        text.to_string()
    } else {
        let mut truncated = text.chars().take(width - 3).collect::<String>();
        truncated.push_str("...");
        truncated
    }
}
