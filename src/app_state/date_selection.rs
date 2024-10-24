use chrono::{DateTime, Datelike, Duration, Local};

#[derive(Debug, PartialEq, Clone)]
pub enum DateField {
    Year,
    Month,
    Day,
    Hour,
    Minute,
}

#[derive(Debug, PartialEq, Clone)]
pub enum QuickRange {
    LastHour,
    Last2Hours,
    Last3Hours,
    Last6Hours,
    Last12Hours,
    Last24Hours,
    Last3Days,
    LastWeek,
}

impl QuickRange {
    pub fn all() -> Vec<QuickRange> {
        vec![
            QuickRange::LastHour,
            QuickRange::Last2Hours,
            QuickRange::Last3Hours,
            QuickRange::Last6Hours,
            QuickRange::Last12Hours,
            QuickRange::Last24Hours,
            QuickRange::Last3Days,
            QuickRange::LastWeek,
        ]
    }

    pub fn to_duration(&self) -> Duration {
        match self {
            QuickRange::LastHour => Duration::hours(1),
            QuickRange::Last2Hours => Duration::hours(2),
            QuickRange::Last3Hours => Duration::hours(3),
            QuickRange::Last6Hours => Duration::hours(6),
            QuickRange::Last12Hours => Duration::hours(12),
            QuickRange::Last24Hours => Duration::hours(24),
            QuickRange::Last3Days => Duration::days(3),
            QuickRange::LastWeek => Duration::days(7),
        }
    }

    pub fn display_name(&self) -> &str {
        match self {
            QuickRange::LastHour => "Last Hour",
            QuickRange::Last2Hours => "Last 2 Hours",
            QuickRange::Last3Hours => "Last 3 Hours",
            QuickRange::Last6Hours => "Last 6 Hours",
            QuickRange::Last12Hours => "Last 12 Hours",
            QuickRange::Last24Hours => "Last 24 Hours",
            QuickRange::Last3Days => "Last 3 Days",
            QuickRange::LastWeek => "Last Week",
        }
    }
}

#[derive(Debug)]
pub struct DateSelection {
    pub from_date: DateTime<Local>,
    pub to_date: DateTime<Local>,
    pub is_selecting_from: bool,
    pub current_field: DateField,
    pub quick_ranges: Vec<QuickRange>,
    pub selected_quick_range: Option<usize>,
    pub custom_selection: bool,
}

impl DateSelection {
    pub fn new() -> Self {
        let now = Local::now();
        Self {
            from_date: now - Duration::hours(1),
            to_date: now,
            is_selecting_from: true,
            current_field: DateField::Day,
            quick_ranges: QuickRange::all(),
            selected_quick_range: Some(0), // Default to first quick range
            custom_selection: false,
        }
    }

    pub fn toggle_selection(&mut self) {
        if self.custom_selection {
            self.is_selecting_from = !self.is_selecting_from;
        }
    }

    pub fn toggle_custom(&mut self) {
        self.custom_selection = !self.custom_selection;
        if !self.custom_selection {
            self.selected_quick_range = Some(0);
            self.apply_quick_range(0);
        }
    }

    pub fn next_quick_range(&mut self) {
        if let Some(current) = self.selected_quick_range {
            let next = (current + 1) % self.quick_ranges.len();
            self.selected_quick_range = Some(next);
            self.apply_quick_range(next);
        }
    }

    pub fn previous_quick_range(&mut self) {
        if let Some(current) = self.selected_quick_range {
            let prev = if current == 0 {
                self.quick_ranges.len() - 1
            } else {
                current - 1
            };
            self.selected_quick_range = Some(prev);
            self.apply_quick_range(prev);
        }
    }

    fn apply_quick_range(&mut self, index: usize) {
        if let Some(range) = self.quick_ranges.get(index) {
            self.to_date = Local::now();
            self.from_date = self.to_date - range.to_duration();
        }
    }

    pub fn next_field(&mut self) {
        self.current_field = match self.current_field {
            DateField::Year => DateField::Month,
            DateField::Month => DateField::Day,
            DateField::Day => DateField::Hour,
            DateField::Hour => DateField::Minute,
            DateField::Minute => DateField::Year,
        };
    }

    pub fn previous_field(&mut self) {
        self.current_field = match self.current_field {
            DateField::Year => DateField::Minute,
            DateField::Month => DateField::Year,
            DateField::Day => DateField::Month,
            DateField::Hour => DateField::Day,
            DateField::Minute => DateField::Hour,
        };
    }

    pub fn adjust_current_field(&mut self, increment: bool) {
        let date = if self.is_selecting_from {
            &mut self.from_date
        } else {
            &mut self.to_date
        };

        match self.current_field {
            DateField::Year => {
                let years = if increment { 1 } else { -1 };
                *date = date.with_year(date.year() + years).unwrap_or(*date);
            }
            DateField::Month => {
                let months = if increment { 1 } else { -1 };
                let new_month = (date.month() as i32 + months).rem_euclid(12) as u32;
                *date = date
                    .with_month(if new_month == 0 { 12 } else { new_month })
                    .unwrap_or(*date);
            }
            DateField::Day => {
                let days = if increment { 1 } else { -1 };
                *date += Duration::days(days);
            }
            DateField::Hour => {
                let hours = if increment { 1 } else { -1 };
                *date += Duration::hours(hours);
            }
            DateField::Minute => {
                let minutes = if increment { 1 } else { -1 };
                *date += Duration::minutes(minutes);
            }
        }

        // Ensure dates stay in order
        if self.is_selecting_from && self.from_date > self.to_date {
            self.from_date = self.to_date;
        } else if !self.is_selecting_from && self.to_date < self.from_date {
            self.to_date = self.from_date;
        }
    }
}
