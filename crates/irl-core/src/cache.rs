use std::fs;
use std::path::PathBuf;
use std::time::{Duration, SystemTime};

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::config::Config;
use crate::error::IrlError;

#[derive(Debug, Serialize, Deserialize)]
struct CacheEntry {
    url: String,
    timestamp: u64,
    ttl_seconds: u64,
}

pub struct Cache {
    cache_dir: PathBuf,
    enabled: bool,
}

impl Cache {
    pub fn new(enabled: bool) -> Self {
        Self {
            cache_dir: Config::cache_dir(),
            enabled,
        }
    }

    fn cache_key(url: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(url.as_bytes());
        format!("{:x}", hasher.finalize())
    }

    fn data_path(&self, key: &str) -> PathBuf {
        self.cache_dir.join(format!("{}.data", key))
    }

    fn meta_path(&self, key: &str) -> PathBuf {
        self.cache_dir.join(format!("{}.meta", key))
    }

    pub fn get(&self, url: &str) -> Option<String> {
        if !self.enabled {
            return None;
        }

        let key = Self::cache_key(url);
        let meta_path = self.meta_path(&key);
        let data_path = self.data_path(&key);

        if !meta_path.exists() || !data_path.exists() {
            return None;
        }

        let meta_content = fs::read_to_string(&meta_path).ok()?;
        let entry: CacheEntry = serde_json::from_str(&meta_content).ok()?;

        let now = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        if now - entry.timestamp >= entry.ttl_seconds {
            // Expired — clean up
            let _ = fs::remove_file(&meta_path);
            let _ = fs::remove_file(&data_path);
            return None;
        }

        fs::read_to_string(&data_path).ok()
    }

    pub fn set(&self, url: &str, data: &str, ttl: Duration) -> Result<(), IrlError> {
        if !self.enabled {
            return Ok(());
        }

        if !self.cache_dir.exists() {
            fs::create_dir_all(&self.cache_dir)
                .map_err(|e| IrlError::Cache(format!("Failed to create cache dir: {}", e)))?;
        }

        let key = Self::cache_key(url);

        let now = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        let entry = CacheEntry {
            url: url.to_string(),
            timestamp: now,
            ttl_seconds: ttl.as_secs(),
        };

        let meta_json = serde_json::to_string(&entry)
            .map_err(|e| IrlError::Cache(format!("Failed to serialize cache metadata: {}", e)))?;

        fs::write(self.data_path(&key), data)
            .map_err(|e| IrlError::Cache(format!("Failed to write cache data: {}", e)))?;
        fs::write(self.meta_path(&key), meta_json)
            .map_err(|e| IrlError::Cache(format!("Failed to write cache metadata: {}", e)))?;

        Ok(())
    }

    pub fn clear(&self) -> Result<u64, IrlError> {
        if !self.cache_dir.exists() {
            return Ok(0);
        }
        let mut count = 0u64;
        for entry in fs::read_dir(&self.cache_dir)
            .map_err(|e| IrlError::Cache(format!("Failed to read cache dir: {}", e)))?
            .flatten()
        {
            let _ = fs::remove_file(entry.path());
            count += 1;
        }
        Ok(count / 2) // Each cached item has .data and .meta files
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn test_cache(dir: &TempDir) -> Cache {
        Cache {
            cache_dir: dir.path().to_path_buf(),
            enabled: true,
        }
    }

    #[test]
    fn test_cache_miss() {
        let dir = TempDir::new().unwrap();
        let cache = test_cache(&dir);
        assert!(cache.get("https://example.com/data").is_none());
    }

    #[test]
    fn test_cache_hit() {
        let dir = TempDir::new().unwrap();
        let cache = test_cache(&dir);
        let url = "https://example.com/data";
        let data = r#"{"key": "value"}"#;

        cache.set(url, data, Duration::from_secs(3600)).unwrap();
        let result = cache.get(url);
        assert_eq!(result, Some(data.to_string()));
    }

    #[test]
    fn test_cache_expired() {
        let dir = TempDir::new().unwrap();
        let cache = test_cache(&dir);
        let url = "https://example.com/data";
        let data = "test data";

        // Set with 0 TTL (immediately expired)
        cache.set(url, data, Duration::from_secs(0)).unwrap();
        // Should be expired
        assert!(cache.get(url).is_none());
    }

    #[test]
    fn test_cache_disabled() {
        let dir = TempDir::new().unwrap();
        let cache = Cache {
            cache_dir: dir.path().to_path_buf(),
            enabled: false,
        };
        let url = "https://example.com/data";
        cache.set(url, "data", Duration::from_secs(3600)).unwrap();
        assert!(cache.get(url).is_none());
    }

    #[test]
    fn test_cache_clear() {
        let dir = TempDir::new().unwrap();
        let cache = test_cache(&dir);
        cache
            .set("https://a.com", "a", Duration::from_secs(3600))
            .unwrap();
        cache
            .set("https://b.com", "b", Duration::from_secs(3600))
            .unwrap();

        let count = cache.clear().unwrap();
        assert_eq!(count, 2);
        assert!(cache.get("https://a.com").is_none());
    }

    #[test]
    fn test_cache_key_deterministic() {
        let k1 = Cache::cache_key("https://example.com/api?foo=bar");
        let k2 = Cache::cache_key("https://example.com/api?foo=bar");
        assert_eq!(k1, k2);

        let k3 = Cache::cache_key("https://example.com/api?foo=baz");
        assert_ne!(k1, k3);
    }
}
