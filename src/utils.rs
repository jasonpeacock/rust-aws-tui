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
            date_str[0..4].to_string(),
            date_str[5..7].to_string(),
            date_str[8..10].to_string(),
            date_str[11..13].to_string(),
            date_str[14..16].to_string(),
        );

        let highlight_style = Style::default().fg(Color::Yellow).bg(Color::DarkGray);
        let normal_style = Style::default();

        spans.extend(vec![
            Span::styled(
                date_parts.0,
                if matches!(current_field, DateField::Year) {
                    highlight_style
                } else {
                    normal_style
                },
            ),
            Span::raw("-"),
            Span::styled(
                date_parts.1,
                if matches!(current_field, DateField::Month) {
                    highlight_style
                } else {
                    normal_style
                },
            ),
            Span::raw("-"),
            Span::styled(
                date_parts.2,
                if matches!(current_field, DateField::Day) {
                    highlight_style
                } else {
                    normal_style
                },
            ),
            Span::raw(" "),
            Span::styled(
                date_parts.3,
                if matches!(current_field, DateField::Hour) {
                    highlight_style
                } else {
                    normal_style
                },
            ),
            Span::raw(":"),
            Span::styled(
                date_parts.4,
                if matches!(current_field, DateField::Minute) {
                    highlight_style
                } else {
                    normal_style
                },
            ),
        ]);
    }

    // Convert spans to owned data by cloning the strings
    let owned_spans: Vec<Span<'static>> = spans
        .into_iter()
        .map(|span| Span::raw(span.content.to_string()))
        .collect();

    Text::from(Line::from(owned_spans))
}
