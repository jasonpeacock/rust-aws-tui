use anyhow::Result;
use std::fs;
use std::path::PathBuf;

pub fn get_cache_dir() -> Result<PathBuf> {
    let cache_dir = dirs::cache_dir()
        .ok_or_else(|| anyhow::anyhow!("Could not find cache directory"))?
        .join("aws-logs-viewer");

    if !cache_dir.exists() {
        fs::create_dir_all(&cache_dir)?;
    }

    Ok(cache_dir)
}

pub fn get_functions_cache_path(profile_name: &str, region: &str) -> Result<PathBuf> {
    let cache_dir = get_cache_dir()?;
    Ok(cache_dir.join(format!("functions_{}_{}.cache", profile_name, region)))
}

pub fn cache_functions(profile_name: &str, region: &str, functions: &[String]) -> Result<()> {
    let cache_path = get_functions_cache_path(profile_name, region)?;
    let cache_content = serde_json::to_string(functions)?;
    fs::write(cache_path, cache_content)?;
    Ok(())
}

pub fn load_cached_functions(profile_name: &str, region: &str) -> Result<Option<Vec<String>>> {
    let cache_path = get_functions_cache_path(profile_name, region)?;

    if !cache_path.exists() {
        return Ok(None);
    }

    let cache_content = fs::read_to_string(cache_path)?;
    let functions: Vec<String> = serde_json::from_str(&cache_content)?;
    Ok(Some(functions))
}
