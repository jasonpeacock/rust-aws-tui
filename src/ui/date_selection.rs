use crate::app_state::{date_selection::DateSelection, FocusedPanel};
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span, Text},
    widgets::{Block, Borders, List, ListItem, Paragraph},
    Frame,
};

use crate::app_state::date_selection::DateField;
use chrono::{DateTime, Local};

pub fn draw_date_selection_panel(f: &mut Frame, date_selection: &DateSelection) {
    let layout_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Title
            Constraint::Min(0),    // Rest of content
        ])
        .margin(1)
        .split(f.size());
    let panel = Block::default()
        .title("Date Selection")
        .borders(Borders::ALL);
    f.render_widget(panel.clone(), layout_chunks[1]);

    let inner_area = panel.inner(layout_chunks[1]);
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(12), // Quick ranges
            Constraint::Length(12), // Custom range
            Constraint::Min(0),     // Helper text
        ])
        .margin(1)
        .split(inner_area);

    // Title bar at the top

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
        .block(Block::default().title("Quick Ranges").borders(Borders::ALL))
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
