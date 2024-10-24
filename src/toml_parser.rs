use anyhow::Result;
use serde::Deserialize;
use std::fs;
use toml;

#[derive(Debug, Deserialize)]
pub struct AwsConfig {
    pub profiles: Vec<Profile>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Profile {
    pub name: String,
    pub region: String,
}

pub fn read_aws_profiles() -> Result<Vec<Profile>> {
    let config_path = "config.toml";

    if !std::path::Path::new(config_path).exists() {
        return Ok(Vec::new());
    }

    let content = fs::read_to_string(config_path)?;
    let config: AwsConfig = toml::from_str(&content)?;

    Ok(config.profiles)
}
