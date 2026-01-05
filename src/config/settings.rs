use directories::ProjectDirs;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ConfigError {
    #[error("Failed to determine config directory")]
    NoConfigDir,
    #[error("Failed to read config file: {0}")]
    ReadError(#[from] std::io::Error),
    #[error("Failed to parse config file: {0}")]
    ParseError(#[from] toml::de::Error),
    #[error("Failed to serialize config: {0}")]
    SerializeError(#[from] toml::ser::Error),
}

#[derive(Debug, Serialize, Deserialize, Default, Clone)]
pub struct Config {
    #[serde(default)]
    pub auth: AuthConfig,
    #[serde(default)]
    pub defaults: DefaultsConfig,
    #[serde(default)]
    pub display: DisplayConfig,
}

#[derive(Debug, Serialize, Deserialize, Default, Clone)]
pub struct AuthConfig {
    pub api_key: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DefaultsConfig {
    pub device: Option<String>,
    pub format: Option<String>,
    pub output_dir: Option<String>,
    pub display: Option<bool>,
}

impl Default for DefaultsConfig {
    fn default() -> Self {
        Self {
            device: Some("Desktop HD".to_string()),
            format: Some("png".to_string()),
            output_dir: Some("./screenshots".to_string()),
            display: Some(true),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DisplayConfig {
    pub protocol: Option<String>,
    pub width: Option<u32>,
    pub height: Option<u32>,
}

impl Default for DisplayConfig {
    fn default() -> Self {
        Self {
            protocol: Some("auto".to_string()),
            width: Some(80),
            height: Some(24),
        }
    }
}

impl Config {
    /// Get the project directories for config storage
    fn project_dirs() -> Option<ProjectDirs> {
        ProjectDirs::from("com", "allscreenshots", "cli")
    }

    /// Get the config directory path
    pub fn config_dir() -> Option<PathBuf> {
        Self::project_dirs().map(|dirs| dirs.config_dir().to_path_buf())
    }

    /// Get the config file path
    pub fn config_path() -> Option<PathBuf> {
        Self::config_dir().map(|dir| dir.join("config.toml"))
    }

    /// Load config from file, returning default if file doesn't exist
    pub fn load() -> Result<Self, ConfigError> {
        let path = Self::config_path().ok_or(ConfigError::NoConfigDir)?;

        if !path.exists() {
            return Ok(Self::default());
        }

        let contents = fs::read_to_string(&path)?;
        let config: Config = toml::from_str(&contents)?;
        Ok(config)
    }

    /// Save config to file
    pub fn save(&self) -> Result<(), ConfigError> {
        let path = Self::config_path().ok_or(ConfigError::NoConfigDir)?;

        // Ensure directory exists
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }

        let contents = toml::to_string_pretty(self)?;
        fs::write(&path, contents)?;
        Ok(())
    }

    /// Get API key with priority: env var > config file
    pub fn get_api_key(&self) -> Option<String> {
        std::env::var("ALLSCREENSHOTS_API_KEY")
            .ok()
            .or_else(|| self.auth.api_key.clone())
    }

    /// Set the API key and save config
    pub fn set_api_key(&mut self, key: String) -> Result<(), ConfigError> {
        self.auth.api_key = Some(key);
        self.save()
    }

    /// Remove the API key and save config
    pub fn remove_api_key(&mut self) -> Result<(), ConfigError> {
        self.auth.api_key = None;
        self.save()
    }

    /// Mask API key for display (show first 8 and last 4 characters)
    pub fn mask_api_key(key: &str) -> String {
        if key.len() <= 12 {
            return "*".repeat(key.len());
        }
        format!(
            "{}...{}",
            &key[..8],
            &key[key.len() - 4..]
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mask_api_key() {
        assert_eq!(Config::mask_api_key("as_live_abcdefghijklmnop"), "as_live_...mnop");
        assert_eq!(Config::mask_api_key("short"), "*****");
    }

    #[test]
    fn test_default_config() {
        let config = Config::default();
        assert!(config.auth.api_key.is_none());
        assert_eq!(config.defaults.device, Some("Desktop HD".to_string()));
    }
}
