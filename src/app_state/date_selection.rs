use chrono::{DateTime, Datelike, Duration, Local};

#[derive(Debug, PartialEq, Clone)]
pub enum DateField {
    Year,
    Month,
    Day,
    Hour,
    Minute,
}

#[derive(Debug)]
pub struct DateSelection {
    pub from_date: DateTime<Local>,
    pub to_date: DateTime<Local>,
    pub is_selecting_from: bool,
    pub current_field: DateField,
}

impl DateSelection {
    pub fn new() -> Self {
        let now = Local::now();
        Self {
            from_date: now - Duration::hours(1),
            to_date: now,
            is_selecting_from: true,
            current_field: DateField::Day,
        }
    }

    pub fn toggle_selection(&mut self) {
        self.is_selecting_from = !self.is_selecting_from;
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
