use anyhow::Result;
use aws_config::{BehaviorVersion, Region};
use aws_sdk_lambda::Client as LambdaClient;
use ratatui::widgets::ListState;
use std::sync::{Arc, Mutex};
use tokio::spawn;

use crate::toml_parser::Profile;
use crate::utils::file_utils::{cache_functions, load_cached_functions};

#[derive(Debug)]
pub struct FunctionSelection {
    pub profile: Profile,
    pub lambda_functions: Arc<Mutex<Vec<String>>>,
    pub filtered_functions: Vec<String>,
    pub selected_index: usize,
    pub filter_input: String,
    pub list_state: ListState,
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
        }
    }

    pub async fn load_functions(&mut self) -> Result<()> {
        // Try to load from cache first
        if let Some(cached_functions) =
            load_cached_functions(&self.profile.name, &self.profile.region)?
        {
            // Update UI immediately with cached data
            self.lambda_functions.lock().unwrap().clear();
            self.lambda_functions
                .lock()
                .unwrap()
                .extend(cached_functions);
            self.filtered_functions = self.lambda_functions.lock().unwrap().clone();
            self.list_state.select(Some(0));

            // Clone necessary data for background task
            let profile_name = self.profile.name.clone();
            let profile_region = self.profile.region.clone();
            let lambda_functions = Arc::clone(&self.lambda_functions);

            // Spawn background task to update cache
            spawn(async move {
                if let Err(e) = update_functions_in_background(
                    profile_name.clone(),
                    profile_region.clone(),
                    lambda_functions,
                )
                .await
                {
                    eprintln!("Background update failed: {}", e);
                }
            });

            return Ok(());
        }

        // If no cache exists, load directly from AWS
        self.load_functions_from_aws().await
    }

    async fn load_functions_from_aws(&mut self) -> Result<()> {
        let config = aws_config::defaults(BehaviorVersion::latest())
            .profile_name(&self.profile.name)
            .region(Region::new(self.profile.region.clone()))
            .load()
            .await;

        let client = LambdaClient::new(&config);
        let mut functions = Vec::new();
        let mut next_marker = None;

        loop {
            let mut request = client.list_functions();
            if let Some(marker) = next_marker {
                request = request.marker(marker);
            }

            let response = request.send().await?;
            let function_list = response.functions();
            for function in function_list {
                if let Some(name) = &function.function_name {
                    functions.push(name.clone())
                }
            }

            next_marker = response.next_marker().map(String::from);
            if next_marker.is_none() {
                break;
            }
        }

        functions.sort();

        // Cache the functions
        cache_functions(&self.profile.name, &self.profile.region, &functions)?;

        self.lambda_functions.lock().unwrap().clear();
        self.lambda_functions.lock().unwrap().extend(functions);
        self.filtered_functions = self.lambda_functions.lock().unwrap().clone();
        self.list_state.select(Some(0));
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

async fn update_functions_in_background(
    profile_name: String,
    profile_region: String,
    lambda_functions: Arc<Mutex<Vec<String>>>,
) -> Result<()> {
    let config = aws_config::defaults(BehaviorVersion::latest())
        .profile_name(&profile_name)
        .region(Region::new(profile_region.clone()))
        .load()
        .await;

    let client = LambdaClient::new(&config);
    let mut functions = Vec::new();
    let mut next_marker = None;

    loop {
        let mut request = client.list_functions();
        if let Some(marker) = next_marker {
            request = request.marker(marker);
        }

        let response = request.send().await?;
        let function_list = response.functions();
        for function in function_list {
            if let Some(name) = &function.function_name {
                functions.push(name.clone())
            }
        }

        next_marker = response.next_marker().map(String::from);
        if next_marker.is_none() {
            break;
        }
    }

    functions.sort();

    // Update cache
    cache_functions(&profile_name, &profile_region, &functions)?;

    // Update the shared functions list
    let mut functions_lock = lambda_functions.lock().unwrap();
    functions_lock.clear();
    functions_lock.extend(functions);

    Ok(())
}
