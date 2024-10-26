use ratatui::prelude::{Color, Line, Span, Style};

pub fn format_json(value: &serde_json::Value, indent: usize) -> Vec<Line<'static>> {
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
                            if let Some(last) = lines.last_mut() {
                                last.spans.push(Span::raw(comma.to_string()));
                            }
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
                            if let Some(last) = lines.last_mut() {
                                last.spans.push(Span::raw(comma.to_string()));
                            }
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
