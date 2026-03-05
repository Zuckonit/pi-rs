//! Sandbox configuration parsing

use std::collections::HashMap;
use std::fs;
use std::path::Path;

use serde::Deserialize;

use crate::core::errors::{PiError, Result};

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SandboxConfig {
    #[serde(default = "default_enabled")]
    pub enabled: bool,

    #[serde(default = "default_type")]
    pub r#type: String,

    #[serde(default)]
    pub mounts: Vec<String>,

    #[serde(default)]
    pub env: HashMap<String, String>,
}

fn default_enabled() -> bool {
    false
}

fn default_type() -> String {
    "epkg".to_string()
}

impl SandboxConfig {
    pub fn load_from_file(path: &Path) -> Result<Self> {
        if !path.exists() {
            return Ok(SandboxConfig::default());
        }

        let content = fs::read_to_string(path)
            .map_err(|e| PiError::Config(format!("Failed to read config: {}", e)))?;

        let config: SandboxConfig = serde_json::from_str(&content)
            .map_err(|e| PiError::Config(format!("Failed to parse config: {}", e)))?;

        Ok(config)
    }

    pub fn load_from_cwd(cwd: &Path) -> Result<Self> {
        let config_path = cwd.join(".pi").join("sandbox.json");
        Self::load_from_file(&config_path)
    }
}

impl Default for SandboxConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            r#type: "epkg".to_string(),
            mounts: Vec::new(),
            env: HashMap::new(),
        }
    }
}

#[derive(Debug, Clone, Default)]
#[allow(dead_code)]
pub struct SandboxMount {
    pub path: String,
}

impl SandboxMount {
    #[allow(dead_code)]
    pub fn new(path: &str) -> Self {
        Self {
            path: path.to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_default_config() {
        let config = SandboxConfig::default();
        assert!(!config.enabled);
        assert_eq!(config.r#type, "epkg");
        assert!(config.mounts.is_empty());
        assert!(config.env.is_empty());
    }

    #[test]
    fn test_load_from_nonexistent_file() {
        let result = SandboxConfig::load_from_file(Path::new("/nonexistent/path.json"));
        assert!(result.is_ok());
        assert!(!result.unwrap().enabled);
    }

    #[test]
    fn test_load_from_valid_file() {
        let mut file = NamedTempFile::new().unwrap();
        let content = r#"{
            "enabled": true,
            "type": "epkg",
            "mounts": ["/opt/data"],
            "env": {"TEST_VAR": "test_value"}
        }"#;
        file.write_all(content.as_bytes()).unwrap();

        let result = SandboxConfig::load_from_file(file.path());
        assert!(result.is_ok());

        let config = result.unwrap();
        assert!(config.enabled);
        assert_eq!(config.r#type, "epkg");
        assert_eq!(config.mounts.len(), 1);
        assert_eq!(config.mounts[0], "/opt/data");
        assert_eq!(config.env.get("TEST_VAR"), Some(&"test_value".to_string()));
    }

    #[test]
    fn test_load_from_invalid_file() {
        let mut file = NamedTempFile::new().unwrap();
        file.write_all(b"invalid json").unwrap();

        let result = SandboxConfig::load_from_file(file.path());
        assert!(result.is_err());
    }

    #[test]
    fn test_sandbox_mount() {
        let mount = SandboxMount::new("/test/path");
        assert_eq!(mount.path, "/test/path");
    }
}
