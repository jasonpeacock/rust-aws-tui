use anyhow::Result;
use aws_config::{BehaviorVersion, Region};
use aws_sdk_lambda::Client as LambdaClient;
use ratatui::widgets::ListState;
use std::fs;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use tokio::spawn;

use crate::toml_parser::Profile;

#[derive(Debug)]
pub struct FunctionSelection {
    pub profile: Profile,
    pub lambda_functions: Arc<Mutex<Vec<String>>>,
    pub filtered_functions: Vec<String>,
    pub selected_index: usize,
    pub filter_input: String,
    pub list_state: ListState,
    aws_client: Option<LambdaClient>,
}

impl FunctionSelection {
    pub fn new(profile: Profile) -> Self {
        Self {
            profile,
            lambda_functions: Arc::new(Mutex::new(Vec::new())),
            filtered_functions: Vec::new(),
            selected_index: 0,
            filter_input: String::new(),
            list_state: ListState::default(),
            aws_client: None,
        }
    }

    fn get_cache_path(&self) -> PathBuf {
        let cache_dir = dirs::cache_dir()
            .unwrap_or_else(|| PathBuf::from("/tmp"))
            .join("lambda-tui");
        fs::create_dir_all(&cache_dir).unwrap_or_default();
        cache_dir.join(format!("functions_{}.cache", self.profile.name))
    }

    async fn fetch_all_functions(&self, client: &LambdaClient) -> Result<Vec<String>> {
        let mut functions = Vec::new();
        let mut next_marker = None;

        loop {
            let mut request = client.list_functions();
            if let Some(marker) = &next_marker {
                request = request.marker(marker);
            }

            let resp = request.send().await?;

            if let Some(func_list) = &resp.functions {
                let function_names: Vec<String> = func_list
                    .iter()
                    .filter_map(|f| f.function_name().map(String::from))
                    .collect();
                functions.extend(function_names);
            }

            next_marker = resp.next_marker().map(ToString::to_string);
            if next_marker.is_none() {
                break;
            }
        }

        Ok(functions)
    }

    pub async fn load_functions(&mut self) -> Result<()> {
        // Initialize AWS client
        let aws_config = aws_config::defaults(BehaviorVersion::v2024_03_28())
            .profile_name(&self.profile.name)
            .region(Region::new(self.profile.region.clone()))
            .load()
            .await;

        let client = LambdaClient::new(&aws_config);
        self.aws_client = Some(client.clone());

        // Try to load from cache first
        let cache_path = self.get_cache_path();
        if let Ok(cached_content) = fs::read_to_string(&cache_path) {
            let cached_functions: Vec<String> = cached_content.lines().map(String::from).collect();
            if !cached_functions.is_empty() {
                *self.lambda_functions.lock().unwrap() = cached_functions;
                self.filtered_functions = self.lambda_functions.lock().unwrap().clone();
                self.list_state.select(Some(0));
            }
        }

        // Fetch functions before spawning
        let fresh_functions = self.fetch_all_functions(&client).await?;
        let lambda_functions = Arc::clone(&self.lambda_functions);
        let cache_path = self.get_cache_path();

        spawn(async move {
            // Update the functions list
            *lambda_functions.lock().unwrap() = fresh_functions.clone();

            // Update cache file
            let content = fresh_functions.join("\n");
            fs::write(cache_path, content).unwrap_or_default();
        });

        Ok(())
    }

    pub async fn update_filter(&mut self) -> Result<()> {
        let lambda_functions = self.lambda_functions.lock().unwrap().clone();

        if self.filter_input.is_empty() {
            self.filtered_functions = lambda_functions;
        } else {
            let filter_lower = self.filter_input.to_lowercase();
            let keywords: Vec<&str> = filter_lower.split_whitespace().collect();

            self.filtered_functions = lambda_functions
                .iter()
                .filter(|name| {
                    let function_name = name.to_lowercase();
                    keywords
                        .iter()
                        .all(|&keyword| function_name.contains(keyword))
                })
                .cloned()
                .collect();
        }

        self.selected_index = 0;
        self.list_state.select(Some(0));
        Ok(())
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
