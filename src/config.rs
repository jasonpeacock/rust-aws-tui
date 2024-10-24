use anyhow::Result;

use crate::toml_parser::{read_aws_profiles, Profile};

#[derive(Debug, Clone)]
pub struct Config {
    pub aws_profiles: Vec<Profile>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            aws_profiles: vec![],
        }
    }
}

impl Config {
    pub fn new() -> Result<Self> {
        let aws_profiles = read_aws_profiles()?;

        Ok(Self {
            aws_profiles: aws_profiles,
        })
    }
}
