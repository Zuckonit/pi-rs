//! Auth Storage

use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::sync::RwLock;

use serde::{Deserialize, Serialize};

/// Credential types
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum Credential {
    ApiKey {
        key: String,
    },
    OAuth {
        access_token: String,
        refresh_token: String,
        expires_at: i64,
    },
}

/// Auth storage
pub struct AuthStorage {
    credentials: RwLock<HashMap<String, Credential>>,
    runtime_overrides: RwLock<HashMap<String, String>>,
    path: PathBuf,
}

impl AuthStorage {
    /// Create a new auth storage
    pub fn new(path: Option<PathBuf>) -> Self {
        let path = path.unwrap_or_else(|| {
            dirs::home_dir()
                .map(|p| p.join(".pi").join("agent").join("auth.json"))
                .unwrap_or_else(|| PathBuf::from(".pi/agent/auth.json"))
        });

        let credentials = if path.exists() {
            let content = fs::read_to_string(&path).unwrap_or_default();
            serde_json::from_str(&content).unwrap_or_default()
        } else {
            HashMap::new()
        };

        Self {
            credentials: RwLock::new(credentials),
            runtime_overrides: RwLock::new(HashMap::new()),
            path,
        }
    }

    /// Create in-memory auth storage
    pub fn in_memory() -> Self {
        Self {
            credentials: RwLock::new(HashMap::new()),
            runtime_overrides: RwLock::new(HashMap::new()),
            path: PathBuf::new(),
        }
    }

    /// Save to file
    pub fn save(&self) -> Result<(), String> {
        let credentials = self.credentials.read().unwrap();
        let content = serde_json::to_string_pretty(&*credentials).map_err(|e| e.to_string())?;

        if let Some(parent) = self.path.parent() {
            fs::create_dir_all(parent).map_err(|e| e.to_string())?;
        }

        fs::write(&self.path, content).map_err(|e| e.to_string())
    }

    /// Get credential for a provider
    pub fn get(&self, provider: &str) -> Option<Credential> {
        // Check runtime overrides first
        if let Ok(overrides) = self.runtime_overrides.read() {
            if let Some(key) = overrides.get(provider) {
                return Some(Credential::ApiKey { key: key.clone() });
            }
        }

        // Check stored credentials
        self.credentials.read().unwrap().get(provider).cloned()
    }

    /// Get API key for a provider
    pub fn get_api_key(&self, provider: &str) -> Option<String> {
        // Check runtime overrides first
        if let Ok(overrides) = self.runtime_overrides.read() {
            if let Some(key) = overrides.get(provider) {
                return Some(key.clone());
            }
        }

        // Check stored credentials
        self.credentials
            .read()
            .unwrap()
            .get(provider)
            .and_then(|c| {
                match c {
                    Credential::ApiKey { key } => Some(key.clone()),
                    Credential::OAuth { .. } => None, // Would need token refresh
                }
            })
            .or_else(|| {
                // Fall back to environment variable
                let env_key = format!("{}_API_KEY", provider.to_uppercase());
                std::env::var(&env_key).ok()
            })
    }

    /// Set API key for a provider
    pub fn set_api_key(&mut self, provider: &str, key: String) {
        self.credentials
            .write()
            .unwrap()
            .insert(provider.to_string(), Credential::ApiKey { key });
    }

    /// Set OAuth credential
    pub fn set_oauth(
        &mut self,
        provider: &str,
        access_token: String,
        refresh_token: String,
        expires_at: i64,
    ) {
        self.credentials.write().unwrap().insert(
            provider.to_string(),
            Credential::OAuth {
                access_token,
                refresh_token,
                expires_at,
            },
        );
    }

    /// Remove credential
    pub fn remove(&mut self, provider: &str) {
        self.credentials.write().unwrap().remove(provider);
    }

    /// List providers
    pub fn list(&self) -> Vec<String> {
        self.credentials.read().unwrap().keys().cloned().collect()
    }

    /// Check if provider has auth
    pub fn has_auth(&self, provider: &str) -> bool {
        self.get_api_key(provider).is_some()
    }

    /// Set runtime API key override
    pub fn set_runtime_api_key(&mut self, provider: &str, api_key: String) {
        self.runtime_overrides
            .write()
            .unwrap()
            .insert(provider.to_string(), api_key);
    }

    /// Remove runtime API key override
    pub fn remove_runtime_api_key(&mut self, provider: &str) {
        self.runtime_overrides.write().unwrap().remove(provider);
    }
}
