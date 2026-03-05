//! Epkg sandbox implementation
//!
//! Based on sandbox-epkg.sh from https://atomgits.com/openeuler/epkg

use std::collections::HashMap;
use std::path::PathBuf;
use std::process::Command;

use crate::core::errors::{PiError, Result};

const SANDBOX_ROOT: &str = "/tmp/sandbox-pi-rs";

pub struct EpkgSandbox {
    project_path: PathBuf,
    mounts: Vec<PathBuf>,
    env_vars: HashMap<String, String>,
}

impl EpkgSandbox {
    pub fn new(
        project_path: PathBuf,
        mounts: Vec<PathBuf>,
        env_vars: HashMap<String, String>,
    ) -> Self {
        Self {
            project_path,
            mounts,
            env_vars,
        }
    }

    fn check_prerequisites(&self) -> Result<()> {
        let output = Command::new("sudo")
            .arg("--version")
            .output()
            .map_err(|_| PiError::Config("sudo is required for sandbox".to_string()))?;

        if !output.status.success() {
            return Err(PiError::Config("sudo is required for sandbox".to_string()));
        }

        Ok(())
    }

    fn build_script(&self) -> String {
        let mounts_ro = vec![
            "/usr".to_string(),
            "/boot".to_string(),
            "/lib/modules".to_string(),
            "/etc/resolv.conf".to_string(),
            "/etc/nsswitch.conf".to_string(),
            "/etc/localtime".to_string(),
            "/etc/timezone".to_string(),
            "/etc/machine-id".to_string(),
            "/etc/hostname".to_string(),
            "/etc/hosts".to_string(),
            "/etc/passwd".to_string(),
            "/etc/group".to_string(),
            "/etc/subuid".to_string(),
            "/etc/subgid".to_string(),
            "/etc/ld.so.cache".to_string(),
            "/etc/login.defs".to_string(),
            "/etc/default".to_string(),
            "/etc/pam.d".to_string(),
            "/etc/alternatives".to_string(),
            "/etc/fonts".to_string(),
            "/etc/ssl".to_string(),
            "/etc/ca-certificates".to_string(),
            "/etc/pki".to_string(),
        ];

        let mut mounts_rw = vec![
            "/tmp".to_string(),
            "/var/tmp".to_string(),
            "/dev".to_string(),
            "/sys".to_string(),
            "/dev/pts".to_string(),
            "/dev/shm".to_string(),
            "/run/systemd".to_string(),
            "/run/initctl".to_string(),
            "/run/dbus".to_string(),
            "/run/user".to_string(),
            "/tmp/.X11-unix".to_string(),
        ];

        for mount in &self.mounts {
            mounts_rw.push(mount.to_string_lossy().to_string());
        }

        let mounts_rw_project = format!(
            "mount --bind -o rw {} {}{}",
            self.project_path.to_string_lossy(),
            SANDBOX_ROOT,
            self.project_path.to_string_lossy()
        );

        let mounts_ro_section: String = mounts_ro
            .iter()
            .filter(|p| PathBuf::from(p).exists())
            .map(|p| format!("mount_ro {}", p))
            .collect::<Vec<_>>()
            .join("\n");

        let mounts_rw_section: String = mounts_rw
            .iter()
            .filter(|p| PathBuf::from(p).exists())
            .map(|p| format!("mount_rw {}", p))
            .collect::<Vec<_>>()
            .join("\n");

        let env_vars_export: String = self
            .env_vars
            .iter()
            .map(|(k, v)| format!("export {}={}", k, Self::shell_escape(v)))
            .collect::<Vec<_>>()
            .join("\n");

        let pi_binary = std::env::current_exe()
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or_else(|_| "pi".to_string());

        let args: Vec<String> = std::env::args().skip(1).collect();
        let pi_args = args.join(" ");

        format!(
            r#"#!/bin/sh
#
# Sandbox script for pi-rs
# Generated automatically based on sandbox-epkg.sh

set -e

SANDBOX_ROOT={sandbox_root}
PROJECT_DIR="{project_dir}"

# Environment variables to pass into sandbox
{env_vars}

# --- Mount helpers ---
mount_ro() {{
    [ -e "$1" ] || return
    mkdir -p "$SANDBOX_ROOT/$1" 2>/dev/null || true
    mount --bind -o ro "$1" "$SANDBOX_ROOT/$1" 2>/dev/null || true
}}

mount_rw() {{
    [ -e "$1" ] || return
    mkdir -p "$SANDBOX_ROOT/$1" 2>/dev/null || true
    mount --bind -o rw "$1" "$SANDBOX_ROOT/$1" 2>/dev/null || true
}}

mount_tmpfs() {{
    mkdir -p "$SANDBOX_ROOT/$1" 2>/dev/null || true
    mount -t tmpfs tmpfs "$SANDBOX_ROOT/$1"
}}

make_own_dir() {{
    mkdir -p "$SANDBOX_ROOT/$1" 2>/dev/null || true
}}

# --- Setup sandbox layout ---
setup_layout() {{
    mkdir -p "$SANDBOX_ROOT"
    mount_tmpfs "$SANDBOX_ROOT"
    cd "$SANDBOX_ROOT" || exit 1
    
    mkdir -p proc sys dev tmp run var/log var/tmp usr etc root old_root opt bin sbin lib lib64
    make_own_dir "$HOME"
    make_own_dir "$HOME/.config"
    make_own_dir "$HOME/.local/share"
    make_own_dir "$HOME/.local/lib"
    make_own_dir "$HOME/.cache"
    
    ln -sf ../run "$SANDBOX_ROOT/var/run"
    ln -sf usr/lib "$SANDBOX_ROOT/lib"
    ln -sf usr/lib64 "$SANDBOX_ROOT/lib64"
    ln -sf usr/bin "$SANDBOX_ROOT/bin"
    ln -sf usr/sbin "$SANDBOX_ROOT/sbin"
}}

# --- Core VFS ---
setup_core_fs() {{
    mount -t proc proc "$SANDBOX_ROOT/proc"
    mount_rw /dev
    mount_rw /sys
    mount -t devpts -o gid=5,mode=0620,newinstance devpts "$SANDBOX_ROOT/dev/pts"
    [ -e "$SANDBOX_ROOT/dev/ptmx" ] || ln -sf pts/ptmx "$SANDBOX_ROOT/dev/ptmx"
    mount_tmpfs /tmp
    mount_tmpfs /var/tmp
    mount_tmpfs /dev/shm
    chmod 1777 "$SANDBOX_ROOT/tmp" "$SANDBOX_ROOT/var/tmp" "$SANDBOX_ROOT/dev/shm"
}}

# --- System config (read-only) ---
setup_system_config() {{
{ro_mounts}
}}

# --- User mounts (read-write) ---
setup_user_mounts() {{
{rw_mounts}
}}

# --- Project directory ---
setup_project() {{
{project_mount}
}}

# --- Pivot into sandbox ---
enter_sandbox() {{
    cd "$SANDBOX_ROOT" || exit 1
    pivot_root . old_root 2>/dev/null || true
    umount -l /old_root 2>/dev/null || true
}}

# --- Main ---
setup_layout
setup_core_fs
setup_system_config
setup_user_mounts
setup_project
enter_sandbox

# Execute pi-rs inside sandbox
cd "$PROJECT_DIR"
exec {bin} {args}
"#,
            ro_mounts = mounts_ro_section,
            rw_mounts = mounts_rw_section,
            project_mount = mounts_rw_project,
            bin = pi_binary,
            args = pi_args,
            sandbox_root = SANDBOX_ROOT,
            project_dir = self.project_path.to_string_lossy().to_string(),
            env_vars = env_vars_export
        )
    }

    fn shell_escape(s: &str) -> String {
        format!("'{}'", s.replace('\'', "'\\''"))
    }

    pub fn launch(&self) -> Result<()> {
        self.check_prerequisites()?;

        let script = self.build_script();
        let script_path = format!(
            "{}/sandbox-pi-rs.sh",
            std::env::temp_dir().to_string_lossy()
        );

        std::fs::write(&script_path, &script)
            .map_err(|e| PiError::Config(format!("Failed to write script: {}", e)))?;

        let output = Command::new("sudo")
            .arg("sh")
            .arg(&script_path)
            .output()
            .map_err(|e| PiError::Config(format!("Failed to execute sandbox: {}", e)))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(PiError::Config(format!(
                "Sandbox launch failed: {}",
                stderr
            )));
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_epkg_sandbox_new() {
        let sandbox = EpkgSandbox::new(PathBuf::from("/tmp/test"), vec![], HashMap::new());
        assert_eq!(sandbox.project_path, PathBuf::from("/tmp/test"));
        assert!(sandbox.mounts.is_empty());
        assert!(sandbox.env_vars.is_empty());
    }

    #[test]
    fn test_epkg_sandbox_with_mounts() {
        let sandbox = EpkgSandbox::new(
            PathBuf::from("/tmp/test"),
            vec![PathBuf::from("/opt/data")],
            HashMap::new(),
        );
        assert_eq!(sandbox.mounts.len(), 1);
    }

    #[test]
    fn test_epkg_sandbox_with_env_vars() {
        let mut env = HashMap::new();
        env.insert("TEST_KEY".to_string(), "test_value".to_string());
        let sandbox = EpkgSandbox::new(PathBuf::from("/tmp/test"), vec![], env);
        assert_eq!(
            sandbox.env_vars.get("TEST_KEY"),
            Some(&"test_value".to_string())
        );
    }

    #[test]
    fn test_shell_escape() {
        let escaped = EpkgSandbox::shell_escape("test");
        assert_eq!(escaped, "'test'");

        let escaped_with_quote = EpkgSandbox::shell_escape("te'st");
        assert_eq!(escaped_with_quote, "'te'\\''st'");
    }

    #[test]
    fn test_build_script_contains_project_path() {
        let sandbox = EpkgSandbox::new(PathBuf::from("/my/project"), vec![], HashMap::new());
        let script = sandbox.build_script();
        assert!(script.contains("/my/project"));
    }
}
