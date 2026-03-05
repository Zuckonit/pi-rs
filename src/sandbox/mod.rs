//! Sandbox module for environment isolation

pub mod config;
pub mod epkg;

pub use config::SandboxConfig;
pub use epkg::EpkgSandbox;

use std::collections::HashMap;
use std::path::PathBuf;

use crate::core::errors::{PiError, Result};

const AUTO_PROPOGATED_ENV_VARS: &[&str] = &[
    "MOONSHOT_API_KEY",
    "OPENAI_API_KEY",
    "ANTHROPIC_API_KEY",
    "GOOGLE_API_KEY",
    "OLLAMA_BASE_URL",
    "AZURE_OPENAI_API_KEY",
    "AZURE_OPENAI_ENDPOINT",
    "MISTRAL_API_KEY",
    "GROQ_API_KEY",
];

pub struct Sandbox {
    pub project_path: PathBuf,
    pub mounts: Vec<PathBuf>,
    pub env_vars: HashMap<String, String>,
    pub sandbox_type: String,
}

impl Sandbox {
    pub fn new(project_path: PathBuf) -> Self {
        Self {
            project_path,
            mounts: Vec::new(),
            env_vars: HashMap::new(),
            sandbox_type: "epkg".to_string(),
        }
    }

    pub fn with_mounts(self, mounts: Vec<PathBuf>) -> Self {
        let mut this = self;
        this.mounts = mounts;
        this
    }

    pub fn with_env_vars(self, env_vars: HashMap<String, String>) -> Self {
        let mut this = self;
        this.env_vars = env_vars;
        this
    }

    pub fn with_sandbox_type(self, sandbox_type: String) -> Self {
        let mut this = self;
        this.sandbox_type = sandbox_type;
        this
    }

    pub fn add_auto_propagated_env_vars(&mut self) {
        for var in AUTO_PROPOGATED_ENV_VARS {
            if let Ok(value) = std::env::var(var) {
                if !value.is_empty() {
                    self.env_vars.insert(var.to_string(), value);
                }
            }
        }
    }

    pub fn validate(&self) -> Result<()> {
        if !self.project_path.exists() {
            return Err(PiError::Config(format!(
                "Project path does not exist: {}",
                self.project_path.display()
            )));
        }

        for mount in &self.mounts {
            if !mount.exists() {
                return Err(PiError::Config(format!(
                    "Mount path does not exist: {}",
                    mount.display()
                )));
            }
        }

        if self.sandbox_type != "epkg" {
            return Err(PiError::Config(format!(
                "Unsupported sandbox type: {}",
                self.sandbox_type
            )));
        }

        Ok(())
    }

    pub fn launch(&self) -> Result<()> {
        self.validate()?;

        match self.sandbox_type.as_str() {
            "epkg" => {
                let sandbox = EpkgSandbox::new(
                    self.project_path.clone(),
                    self.mounts.clone(),
                    self.env_vars.clone(),
                );
                sandbox.launch()
            }
            _ => Err(PiError::Config(format!(
                "Unsupported sandbox type: {}",
                self.sandbox_type
            ))),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sandbox_new() {
        let sandbox = Sandbox::new(PathBuf::from("/tmp/test"));
        assert_eq!(sandbox.project_path, PathBuf::from("/tmp/test"));
        assert!(sandbox.mounts.is_empty());
        assert!(sandbox.env_vars.is_empty());
        assert_eq!(sandbox.sandbox_type, "epkg");
    }

    #[test]
    fn test_sandbox_with_mounts() {
        let sandbox =
            Sandbox::new(PathBuf::from("/tmp/test")).with_mounts(vec![PathBuf::from("/opt/data")]);
        assert_eq!(sandbox.mounts.len(), 1);
        assert_eq!(sandbox.mounts[0], PathBuf::from("/opt/data"));
    }

    #[test]
    fn test_sandbox_with_env_vars() {
        let mut env = HashMap::new();
        env.insert("TEST_KEY".to_string(), "test_value".to_string());
        let sandbox = Sandbox::new(PathBuf::from("/tmp/test")).with_env_vars(env);
        assert_eq!(
            sandbox.env_vars.get("TEST_KEY"),
            Some(&"test_value".to_string())
        );
    }

    #[test]
    fn test_sandbox_validate_nonexistent_path() {
        let sandbox = Sandbox::new(PathBuf::from("/nonexistent/path"));
        let result = sandbox.validate();
        assert!(result.is_err());
    }

    #[test]
    fn test_sandbox_validate_existent_path() {
        let sandbox = Sandbox::new(PathBuf::from("."));
        let result = sandbox.validate();
        assert!(result.is_ok());
    }
}
