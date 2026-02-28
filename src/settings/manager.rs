//! Settings Manager

use std::fs;
use std::path::{Path, PathBuf};
use std::sync::RwLock;

use serde::{Deserialize, Serialize};

use crate::core::ThinkingLevel;

/// Settings
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Settings {
    // Base
    pub default_provider: Option<String>,
    pub default_model: Option<String>,
    pub thinking_level: String,

    // Queue mode
    pub steering_mode: String,
    pub follow_up_mode: String,

    // Compaction
    pub auto_compact: bool,
    pub compact_threshold: f64,
    pub compact_reserve_tokens: u32,
    pub compact_keep_recent_tokens: u32,

    // UI
    pub theme: String,
    pub show_images: bool,
    pub show_thinking: bool,

    // Paths
    pub extensions: Vec<String>,
    pub skills: Vec<String>,
    pub prompts: Vec<String>,

    // Shell
    pub shell_path: Option<String>,

    // Behavior
    pub quiet_startup: bool,
    pub enable_skill_commands: bool,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            default_provider: None,
            default_model: None,
            thinking_level: "medium".to_string(),
            steering_mode: "one-at-a-time".to_string(),
            follow_up_mode: "one-at-a-time".to_string(),
            auto_compact: true,
            compact_threshold: 0.9,
            compact_reserve_tokens: 100000,
            compact_keep_recent_tokens: 10000,
            theme: "dark".to_string(),
            show_images: true,
            show_thinking: true,
            extensions: vec![],
            skills: vec![],
            prompts: vec![],
            shell_path: None,
            quiet_startup: false,
            enable_skill_commands: true,
        }
    }
}

/// Settings Manager
pub struct SettingsManager {
    global: RwLock<Settings>,
    project: RwLock<Option<Settings>>,
    global_path: PathBuf,
    project_path: Option<PathBuf>,
}

impl SettingsManager {
    /// Create a new settings manager
    pub fn new(cwd: &str) -> Self {
        let agent_dir = dirs::home_dir()
            .map(|p| p.join(".pi").join("agent"))
            .unwrap_or_else(|| PathBuf::from(".pi/agent"));

        let global_path = agent_dir.join("settings.json");

        let project_path = PathBuf::from(cwd).join(".pi-settings.json");
        let project_path = if project_path.exists() {
            Some(project_path)
        } else {
            None
        };

        let global = if global_path.exists() {
            Self::load_from_file(&global_path).unwrap_or_default()
        } else {
            Settings::default()
        };

        let project = if let Some(ref pp) = project_path {
            Self::load_from_file(pp).ok()
        } else {
            None
        };

        Self {
            global: RwLock::new(global),
            project: RwLock::new(project),
            global_path,
            project_path,
        }
    }

    /// Load settings from file
    fn load_from_file(path: &Path) -> Result<Settings, String> {
        let content = fs::read_to_string(path).map_err(|e| e.to_string())?;
        serde_json::from_str(&content).map_err(|e| e.to_string())
    }

    /// Save global settings
    pub fn save_global(&self) -> Result<(), String> {
        let settings = self.global.read().unwrap();

        // Ensure directory exists
        if let Some(parent) = self.global_path.parent() {
            fs::create_dir_all(parent).map_err(|e| e.to_string())?;
        }

        let content = serde_json::to_string_pretty(&*settings).map_err(|e| e.to_string())?;
        fs::write(&self.global_path, content).map_err(|e| e.to_string())
    }

    /// Get global setting
    pub fn get(&self, key: &str) -> serde_json::Value {
        // Check project settings first
        if let Ok(project) = self.project.read() {
            if let Some(ref settings) = *project {
                if let Some(value) = Self::get_field(settings, key) {
                    return value;
                }
            }
        }

        // Fall back to global
        if let Ok(global) = self.global.read() {
            if let Some(value) = Self::get_field(&global, key) {
                return value;
            }
        }

        serde_json::Value::Null
    }

    /// Get field from settings
    fn get_field(settings: &Settings, key: &str) -> Option<serde_json::Value> {
        match key {
            "defaultProvider" | "default_provider" => settings
                .default_provider
                .as_ref()
                .map(|v| serde_json::json!(v)),
            "defaultModel" | "default_model" => settings
                .default_model
                .as_ref()
                .map(|v| serde_json::json!(v)),
            "thinkingLevel" | "thinking_level" => Some(serde_json::json!(&settings.thinking_level)),
            "steeringMode" | "steering_mode" => Some(serde_json::json!(&settings.steering_mode)),
            "followUpMode" | "follow_up_mode" => Some(serde_json::json!(&settings.follow_up_mode)),
            "autoCompact" | "auto_compact" => Some(serde_json::json!(settings.auto_compact)),
            "compactThreshold" | "compact_threshold" => {
                Some(serde_json::json!(settings.compact_threshold))
            }
            "theme" => Some(serde_json::json!(&settings.theme)),
            "showImages" | "show_images" => Some(serde_json::json!(settings.show_images)),
            "showThinking" | "show_thinking" => Some(serde_json::json!(settings.show_thinking)),
            "extensions" => Some(serde_json::json!(&settings.extensions)),
            "skills" => Some(serde_json::json!(&settings.skills)),
            "prompts" => Some(serde_json::json!(&settings.prompts)),
            "shellPath" | "shell_path" => {
                settings.shell_path.as_ref().map(|v| serde_json::json!(v))
            }
            "quietStartup" | "quiet_startup" => Some(serde_json::json!(settings.quiet_startup)),
            "enableSkillCommands" | "enable_skill_commands" => {
                Some(serde_json::json!(settings.enable_skill_commands))
            }
            _ => None,
        }
    }

    /// Set global setting
    pub fn set(&self, key: &str, value: serde_json::Value) -> Result<(), String> {
        let mut settings = self.global.write().unwrap();

        match key {
            "defaultProvider" | "default_provider" => {
                settings.default_provider = value.as_str().map(String::from);
            }
            "defaultModel" | "default_model" => {
                settings.default_model = value.as_str().map(String::from);
            }
            "thinkingLevel" | "thinking_level" => {
                settings.thinking_level = value.as_str().unwrap_or("medium").to_string();
            }
            "steeringMode" | "steering_mode" => {
                settings.steering_mode = value.as_str().unwrap_or("one-at-a-time").to_string();
            }
            "followUpMode" | "follow_up_mode" => {
                settings.follow_up_mode = value.as_str().unwrap_or("one-at-a-time").to_string();
            }
            "autoCompact" | "auto_compact" => {
                settings.auto_compact = value.as_bool().unwrap_or(true);
            }
            "compactThreshold" | "compact_threshold" => {
                settings.compact_threshold = value.as_f64().unwrap_or(0.9);
            }
            "theme" => {
                settings.theme = value.as_str().unwrap_or("dark").to_string();
            }
            "showImages" | "show_images" => {
                settings.show_images = value.as_bool().unwrap_or(true);
            }
            "showThinking" | "show_thinking" => {
                settings.show_thinking = value.as_bool().unwrap_or(true);
            }
            "quietStartup" | "quiet_startup" => {
                settings.quiet_startup = value.as_bool().unwrap_or(false);
            }
            "enableSkillCommands" | "enable_skill_commands" => {
                settings.enable_skill_commands = value.as_bool().unwrap_or(true);
            }
            _ => return Err(format!("Unknown setting: {}", key)),
        }

        Ok(())
    }

    /// Get default provider
    pub fn get_default_provider(&self) -> Option<String> {
        self.get("defaultProvider").as_str().map(String::from)
    }

    /// Get default model
    pub fn get_default_model(&self) -> Option<String> {
        self.get("defaultModel").as_str().map(String::from)
    }

    /// Get thinking level
    pub fn get_thinking_level(&self) -> ThinkingLevel {
        let value = self.get("thinkingLevel");
        let level = value.as_str().unwrap_or("medium");
        match level {
            "off" => ThinkingLevel::Off,
            "minimal" => ThinkingLevel::Minimal,
            "low" => ThinkingLevel::Low,
            "medium" => ThinkingLevel::Medium,
            "high" => ThinkingLevel::High,
            "xhigh" => ThinkingLevel::XHigh,
            _ => ThinkingLevel::Medium,
        }
    }
}
