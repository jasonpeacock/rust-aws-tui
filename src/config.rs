use anyhow::Result;
use std::env;

#[derive(Debug, Clone)]
pub struct Config {
    pub aws_profile_name: String,
    pub aws_region: String,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            aws_profile_name: String::from("resola-staging"),
            aws_region: String::from("ap-northeast-1"),
        }
    }
}

impl Config {
    pub fn new() -> Result<Self> {
        let aws_profile_name =
            env::var("AWS_PROFILE").unwrap_or_else(|_| String::from("resola-staging"));

        let aws_region = env::var("AWS_REGION")
            .or_else(|_| env::var("AWS_DEFAULT_REGION"))
            .unwrap_or_else(|_| String::from("ap-northeast-1"));

        Ok(Self {
            aws_profile_name,
            aws_region,
        })
    }
}
