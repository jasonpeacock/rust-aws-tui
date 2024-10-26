use anyhow::Result;
use aws_config::Region;
use aws_sdk_cloudwatchlogs::types::OutputLogEvent;
use aws_sdk_cloudwatchlogs::Client as CloudWatchLogsClient;
use chrono::{DateTime, Local};
use std::sync::{Arc, Mutex};

#[derive(Debug)]
pub struct LogViewer {
    pub function_name: String,
    pub from_date: DateTime<Local>,
    pub to_date: DateTime<Local>,
    pub logs: Arc<Mutex<Vec<OutputLogEvent>>>,
    pub filtered_logs: Vec<OutputLogEvent>,
    pub filter_input: String,
    pub scroll_offset: usize,  // Changed from scroll_position
    pub selected_log: Option<usize>,
    pub expanded: bool,
    cloudwatch_client: Option<CloudWatchLogsClient>,
    pub scroll_position: usize,
}

impl LogViewer {
    pub fn new(
        function_name: String,
        from_date: DateTime<Local>,
        to_date: DateTime<Local>,
    ) -> Self {
        Self {
            function_name,
            from_date,
            to_date,
            logs: Arc::new(Mutex::new(Vec::new())),
            filtered_logs: Vec::new(),
            filter_input: String::new(),
            scroll_offset: 0,
            selected_log: None,
            expanded: false,
            cloudwatch_client: None,
            scroll_position: 0,
        }
    }

    pub async fn initialize(&mut self, profile_name: String, region: String) -> Result<()> {
        let aws_config = aws_config::defaults(aws_config::BehaviorVersion::v2024_03_28())
            .profile_name(profile_name)
            .region(Region::new(region.clone()))
            .load()
            .await;

        self.cloudwatch_client = Some(CloudWatchLogsClient::new(&aws_config));
        self.load_logs().await?;
        Ok(())
    }

    async fn load_logs(&mut self) -> Result<()> {
        let client = self.cloudwatch_client.as_ref().unwrap();
        let log_group_name = format!("/aws/lambda/{}", self.function_name);

        let start_time = self.from_date.timestamp_millis();
        let end_time = self.to_date.timestamp_millis();

        let mut logs = Vec::new();
        let mut next_token = None;

        loop {
            let mut request = client
                .filter_log_events()
                .log_group_name(&log_group_name)
                .start_time(start_time as i64)
                .end_time(end_time as i64)
                .limit(100);

            if let Some(token) = &next_token {
                request = request.next_token(token);
            }

            let response = request.send().await?;

            if let Some(events) = response.events {
                logs.extend(events.into_iter().map(|e| {
                    OutputLogEvent::builder()
                        .timestamp(e.timestamp.unwrap_or(0))
                        .message(e.message.unwrap_or(String::new()))
                        .ingestion_time(e.ingestion_time.unwrap_or(0))
                        .build()
                }));
            }

            next_token = response.next_token;
            if next_token.is_none() {
                break;
            }
        }

        *self.logs.lock().unwrap() = logs;
        self.update_filter();
        Ok(())
    }

    pub fn update_filter(&mut self) {
        let logs = self.logs.lock().unwrap();

        if self.filter_input.is_empty() {
            self.filtered_logs = logs.clone();
        } else {
            let filter_lower = self.filter_input.to_lowercase();
            let keywords: Vec<&str> = filter_lower.split_whitespace().collect();

            self.filtered_logs = logs
                .iter()
                .filter(|log| {
                    if let Some(message) = log.message.as_ref() {
                        let message_lower = message.to_lowercase();
                        keywords
                            .iter()
                            .all(|&keyword| message_lower.contains(keyword))
                    } else {
                        false
                    }
                })
                .cloned()
                .collect();
        }

        // Reset selection when filter changes
        self.selected_log = if self.filtered_logs.is_empty() {
            None
        } else {
            Some(0)
        };
        self.expanded = false;
    }

    pub fn scroll_up(&mut self) {
        if self.expanded {
            self.scroll_offset = self.scroll_offset.saturating_sub(1);
        } else if let Some(selected) = self.selected_log {
            if selected > 0 {
                self.selected_log = Some(selected - 1);
                // Adjust scroll offset to keep selection in view
                if selected <= self.scroll_offset {
                    self.scroll_offset = self.scroll_offset.saturating_sub(1);
                }
            }
        } else if !self.filtered_logs.is_empty() {
            self.selected_log = Some(0);
            self.scroll_offset = 0;
        }
    }

    pub fn scroll_down(&mut self) {
        if self.expanded {
            self.scroll_offset = self.scroll_offset.saturating_add(1);
        } else if let Some(selected) = self.selected_log {
            if selected < self.filtered_logs.len().saturating_sub(1) {
                self.selected_log = Some(selected + 1);
                // Adjust scroll offset to keep selection in view
                self.scroll_offset = self.scroll_offset.saturating_add(1);
            }
        } else if !self.filtered_logs.is_empty() {
            self.selected_log = Some(0);
            self.scroll_offset = 0;
        }
    }

    pub fn update_scroll(&mut self, visible_height: usize) {
        if let Some(selected) = self.selected_log {
            // Keep selection in the middle of the visible area when possible
            let middle = visible_height / 2;
            
            if selected >= middle {
                self.scroll_offset = selected.saturating_sub(middle);
            } else {
                self.scroll_offset = 0;
            }

            // Don't scroll past the end
            let max_scroll = self.filtered_logs.len().saturating_sub(visible_height);
            self.scroll_offset = self.scroll_offset.min(max_scroll);
        }
    }

    pub fn toggle_expand(&mut self) {
        self.expanded = !self.expanded;
        self.scroll_offset = 0;
    }

    pub fn get_selected_log(&self) -> Option<&OutputLogEvent> {
        self.selected_log.and_then(|i| self.filtered_logs.get(i))
    }

    pub fn page_up(&mut self, page_size: usize) {
        self.scroll_offset = self.scroll_offset.saturating_sub(page_size);
    }

    pub fn page_down(&mut self, page_size: usize) {
        if !self.filtered_logs.is_empty() {
            self.scroll_offset =
                (self.scroll_offset + page_size).min(self.filtered_logs.len() - 1);
        }
    }

    pub fn get_visible_range(&self, visible_height: usize) -> (usize, usize) {
        let total_logs = self.filtered_logs.len();
        let half_height = visible_height / 2;

        if let Some(selected) = self.selected_log {
            // Calculate the ideal start position that would center the selected item
            let ideal_start = selected.saturating_sub(half_height);
            
            // Adjust start position if we're too close to the end
            let start = if selected + half_height >= total_logs {
                total_logs.saturating_sub(visible_height)
            } else {
                ideal_start
            };

            // Calculate end position
            let end = (start + visible_height).min(total_logs);

            (start, end)
        } else {
            (0, visible_height.min(total_logs))
        }
    }
}
