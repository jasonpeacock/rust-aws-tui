use anyhow::Result;
use aws_config::{BehaviorVersion, Region};
use aws_sdk_lambda::Client as LambdaClient;
use ratatui::widgets::ListState;

use crate::toml_parser::Profile;

#[derive(Debug)]
pub struct FunctionSelection {
    pub profile: Profile,
    pub lambda_functions: Vec<String>,
    pub filtered_functions: Vec<String>,
    pub selected_index: usize,
    pub filter_input: String,
    pub list_state: ListState,
}

impl FunctionSelection {
    pub fn new(profile: Profile) -> Self {
        Self {
            profile,
            lambda_functions: Vec::new(),
            filtered_functions: Vec::new(),
            selected_index: 0,
            filter_input: String::new(),
            list_state: ListState::default(),
        }
    }

    pub async fn load_functions(&mut self) -> Result<()> {
        self.lambda_functions =
            Self::fetch_lambda_functions(self.profile.name.clone(), self.profile.region.clone())
                .await?;
        self.filtered_functions = self.lambda_functions.clone();
        self.list_state.select(Some(0));
        Ok(())
    }

    async fn fetch_lambda_functions(profile_name: String, region: String) -> Result<Vec<String>> {
        let aws_config = aws_config::defaults(BehaviorVersion::v2024_03_28())
            .profile_name(&profile_name)
            .region(Region::new(region))
            .load()
            .await;

        let client = LambdaClient::new(&aws_config);
        let mut functions = Vec::new();
        let mut next_marker = None;

        loop {
            let mut request = client.list_functions();
            if let Some(marker) = next_marker {
                request = request.marker(marker);
            }

            let resp = request.send().await?;

            if let Some(func_list) = resp.functions.as_ref() {
                functions.extend(
                    func_list
                        .iter()
                        .filter_map(|f| f.function_name().map(String::from)),
                );
            }

            next_marker = resp.next_marker().map(ToString::to_string);

            if next_marker.is_none() {
                break;
            }
        }

        Ok(functions)
    }

    pub fn update_filter(&mut self) {
        let keywords: Vec<String> = self
            .filter_input
            .to_lowercase()
            .split_whitespace()
            .map(String::from)
            .collect();
        self.filtered_functions = self
            .lambda_functions
            .iter()
            .filter(|&f| {
                let function_name = f.to_lowercase();
                keywords
                    .iter()
                    .all(|keyword| function_name.contains(keyword))
            })
            .cloned()
            .collect();
        self.selected_index = 0;
        self.list_state.select(Some(0));
    }

    pub fn next(&mut self) {
        if !self.filtered_functions.is_empty() {
            self.selected_index = (self.selected_index + 1).min(self.filtered_functions.len() - 1);
            self.list_state.select(Some(self.selected_index));
        }
    }

    pub fn previous(&mut self) {
        if !self.filtered_functions.is_empty() {
            self.selected_index = self.selected_index.saturating_sub(1);
            self.list_state.select(Some(self.selected_index));
        }
    }
}
