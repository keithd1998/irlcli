use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

use crate::error::IrlError;

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Config {
    #[serde(default)]
    pub general: GeneralConfig,
    #[serde(default)]
    pub transport: ServiceConfig,
    #[serde(default)]
    pub cro: ServiceConfig,
    #[serde(default)]
    pub extra: HashMap<String, toml::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeneralConfig {
    #[serde(default = "default_format")]
    pub default_format: String,
    #[serde(default = "default_true")]
    pub colour: bool,
    #[serde(default = "default_cache_ttl")]
    pub cache_ttl_seconds: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ServiceConfig {
    #[serde(default)]
    pub api_key: String,
}

fn default_format() -> String {
    "table".to_string()
}

fn default_true() -> bool {
    true
}

fn default_cache_ttl() -> u64 {
    3600
}

impl Default for GeneralConfig {
    fn default() -> Self {
        Self {
            default_format: default_format(),
            colour: true,
            cache_ttl_seconds: default_cache_ttl(),
        }
    }
}

impl Config {
    pub fn config_dir() -> PathBuf {
        let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
        PathBuf::from(home).join(".irl")
    }

    pub fn config_path() -> PathBuf {
        Self::config_dir().join("config.toml")
    }

    pub fn cache_dir() -> PathBuf {
        Self::config_dir().join("cache")
    }

    pub fn data_dir() -> PathBuf {
        Self::config_dir().join("data")
    }

    pub fn load() -> Result<Self, IrlError> {
        let path = Self::config_path();
        if !path.exists() {
            return Ok(Self::default());
        }
        let content = fs::read_to_string(&path).map_err(|e| {
            IrlError::Config(format!(
                "Failed to read config at {}: {}",
                path.display(),
                e
            ))
        })?;
        let config: Config = toml::from_str(&content)?;
        Ok(config)
    }

    pub fn save(&self) -> Result<(), IrlError> {
        let path = Self::config_path();
        let dir = Self::config_dir();
        if !dir.exists() {
            fs::create_dir_all(&dir)
                .map_err(|e| IrlError::Config(format!("Failed to create config dir: {}", e)))?;
        }
        let content = toml::to_string_pretty(self)
            .map_err(|e| IrlError::Config(format!("Failed to serialize config: {}", e)))?;
        fs::write(&path, content)
            .map_err(|e| IrlError::Config(format!("Failed to write config: {}", e)))?;
        Ok(())
    }

    pub fn set_value(&mut self, key: &str, value: &str) -> Result<(), IrlError> {
        let parts: Vec<&str> = key.splitn(2, '.').collect();
        if parts.len() != 2 {
            return Err(IrlError::Config(format!(
                "Key must be in format 'section.key', got: {}",
                key
            )));
        }
        match (parts[0], parts[1]) {
            ("general", "default_format") => {
                if !["table", "json", "csv"].contains(&value) {
                    return Err(IrlError::Config(format!(
                        "Invalid format '{}'. Must be one of: table, json, csv",
                        value
                    )));
                }
                self.general.default_format = value.to_string();
            }
            ("general", "colour") => {
                self.general.colour = value
                    .parse()
                    .map_err(|_| IrlError::Config("colour must be true or false".to_string()))?;
            }
            ("general", "cache_ttl_seconds") => {
                self.general.cache_ttl_seconds = value.parse().map_err(|_| {
                    IrlError::Config("cache_ttl_seconds must be a number".to_string())
                })?;
            }
            ("transport", "api_key") => {
                self.transport.api_key = value.to_string();
            }
            ("cro", "api_key") => {
                self.cro.api_key = value.to_string();
            }
            _ => {
                return Err(IrlError::Config(format!("Unknown config key: {}", key)));
            }
        }
        Ok(())
    }

    pub fn init_interactive(path: &Path) -> Result<Self, IrlError> {
        let config = Config::default();
        if let Some(parent) = path.parent() {
            if !parent.exists() {
                fs::create_dir_all(parent)
                    .map_err(|e| IrlError::Config(format!("Failed to create config dir: {}", e)))?;
            }
        }
        let content = toml::to_string_pretty(&config)
            .map_err(|e| IrlError::Config(format!("Failed to serialize config: {}", e)))?;
        fs::write(path, content)
            .map_err(|e| IrlError::Config(format!("Failed to write config: {}", e)))?;
        Ok(config)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_default_config() {
        let config = Config::default();
        assert_eq!(config.general.default_format, "table");
        assert!(config.general.colour);
        assert_eq!(config.general.cache_ttl_seconds, 3600);
    }

    #[test]
    fn test_save_and_load() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("config.toml");

        let config = Config::default();
        let content = toml::to_string_pretty(&config).unwrap();
        fs::write(&path, &content).unwrap();

        let loaded: Config = toml::from_str(&fs::read_to_string(&path).unwrap()).unwrap();
        assert_eq!(loaded.general.default_format, "table");
    }

    #[test]
    fn test_set_value() {
        let mut config = Config::default();
        config.set_value("general.default_format", "json").unwrap();
        assert_eq!(config.general.default_format, "json");

        config.set_value("transport.api_key", "test-key").unwrap();
        assert_eq!(config.transport.api_key, "test-key");
    }

    #[test]
    fn test_set_invalid_format() {
        let mut config = Config::default();
        let result = config.set_value("general.default_format", "xml");
        assert!(result.is_err());
    }

    #[test]
    fn test_set_invalid_key() {
        let mut config = Config::default();
        let result = config.set_value("nonexistent.key", "value");
        assert!(result.is_err());
    }

    #[test]
    fn test_set_invalid_key_format() {
        let mut config = Config::default();
        let result = config.set_value("noperiod", "value");
        assert!(result.is_err());
    }
}
