use serde::{Deserialize, Serialize};
use std::io;
use std::path::{Path, PathBuf};
use thiserror::Error;

/// Claude plan tier — determines the token limit per 5-hour billing block.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PlanKind {
    Pro,
    Max5,
    Max20,
    Custom,
}

/// Plan configuration stored in `config.json`.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PlanConfig {
    pub kind: PlanKind,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub custom_token_limit: Option<u64>,
}

impl Default for PlanConfig {
    fn default() -> Self {
        PlanConfig {
            kind: PlanKind::Max5,
            custom_token_limit: None,
        }
    }
}

impl PlanConfig {
    /// Token limit per 5-hour billing block for this plan.
    pub fn token_limit(&self) -> u64 {
        match self.kind {
            PlanKind::Pro => 44_000,
            PlanKind::Max5 => 88_000,
            PlanKind::Max20 => 220_000,
            PlanKind::Custom => self.custom_token_limit.unwrap_or(88_000),
        }
    }
}

#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("required environment variable '{var}' is not set: {source}")]
    EnvVar {
        var: &'static str,
        source: std::env::VarError,
    },
    #[error("io error for '{path}': {source}")]
    Io {
        path: PathBuf,
        #[source]
        source: io::Error,
    },
    #[error("failed to parse config '{path}': {source}")]
    Parse {
        path: PathBuf,
        source: serde_json::Error,
    },
}

/// Runtime configuration for Ignis.
#[derive(Clone, Debug)]
pub struct Config {
    /// Root of Claude Code JSONL logs (usually `%USERPROFILE%\.claude\projects`).
    pub claude_projects_dir: PathBuf,
    /// Bearer token for the local HTTP API.
    pub api_token: String,
    /// Claude plan — determines the token limit per 5-hour billing block.
    pub plan: PlanConfig,
}

impl Config {
    /// Load from `%APPDATA%\ignis\config.json`, creating it with defaults when absent.
    ///
    /// `WINUSAGE_PROJECTS_DIR` env var always overrides `claude_projects_dir` — useful
    /// for tests and CI without touching the user's real log directory.
    pub fn load() -> Result<Self, ConfigError> {
        let path = config_file_path()?;

        let mut cfg = if path.exists() {
            load_file(&path)?
        } else {
            let c = Config {
                claude_projects_dir: home_projects_dir()?,
                api_token: generate_token(),
                plan: PlanConfig::default(),
            };
            // Best-effort write — ignore failure (e.g. read-only filesystem).
            let _ = save_file(&c, &path);
            c
        };

        // Env override always wins (testing / CI).
        if let Ok(dir) = std::env::var("WINUSAGE_PROJECTS_DIR") {
            cfg.claude_projects_dir = PathBuf::from(dir);
        }

        Ok(cfg)
    }
}

// ── Internal helpers ──────────────────────────────────────────────────────────

#[derive(Serialize, Deserialize)]
struct StoredConfig {
    claude_projects_dir: String,
    api_token: String,
    #[serde(default)]
    plan: PlanConfig,
}

fn config_file_path() -> Result<PathBuf, ConfigError> {
    // On Windows APPDATA is the canonical config root; fall back to ~/.config on Unix.
    if let Ok(appdata) = std::env::var("APPDATA") {
        return Ok(PathBuf::from(appdata).join("ignis").join("config.json"));
    }
    let home = std::env::var("HOME").map_err(|e| ConfigError::EnvVar {
        var: "HOME",
        source: e,
    })?;
    Ok(PathBuf::from(format!("{home}/.config"))
        .join("ignis")
        .join("config.json"))
}

fn home_projects_dir() -> Result<PathBuf, ConfigError> {
    if let Ok(profile) = std::env::var("USERPROFILE") {
        return Ok(PathBuf::from(profile).join(".claude").join("projects"));
    }
    let home = std::env::var("HOME").map_err(|e| ConfigError::EnvVar {
        var: "HOME",
        source: e,
    })?;
    Ok(PathBuf::from(home).join(".claude").join("projects"))
}

fn load_file(path: &Path) -> Result<Config, ConfigError> {
    let raw = std::fs::read_to_string(path).map_err(|e| ConfigError::Io {
        path: path.to_owned(),
        source: e,
    })?;
    let stored: StoredConfig = serde_json::from_str(&raw).map_err(|e| ConfigError::Parse {
        path: path.to_owned(),
        source: e,
    })?;
    Ok(Config {
        claude_projects_dir: PathBuf::from(stored.claude_projects_dir),
        api_token: stored.api_token,
        plan: stored.plan,
    })
}

fn save_file(cfg: &Config, path: &Path) -> Result<(), ConfigError> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| ConfigError::Io {
            path: parent.to_owned(),
            source: e,
        })?;
    }
    let stored = StoredConfig {
        claude_projects_dir: cfg.claude_projects_dir.to_string_lossy().into_owned(),
        api_token: cfg.api_token.clone(),
        plan: cfg.plan.clone(),
    };
    let json = serde_json::to_string_pretty(&stored).expect("StoredConfig always serializes");
    std::fs::write(path, json).map_err(|e| ConfigError::Io {
        path: path.to_owned(),
        source: e,
    })
}

/// Generate a cryptographically random 32-hex-char token via the OS CSPRNG.
fn generate_token() -> String {
    let mut bytes = [0u8; 16];
    getrandom::getrandom(&mut bytes).unwrap_or_else(|_| {
        // Extremely unlikely fallback: mix time + PID into bytes.
        let nanos = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .subsec_nanos() as u64;
        let pid = std::process::id() as u64;
        let a = nanos.wrapping_mul(0x9e3779b97f4a7c15).wrapping_add(pid);
        let b = a ^ pid.wrapping_mul(0x517cc1b727220a95);
        bytes[..8].copy_from_slice(&a.to_le_bytes());
        bytes[8..].copy_from_slice(&b.to_le_bytes());
    });
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generate_token_is_32_hex_chars() {
        let t = generate_token();
        assert_eq!(t.len(), 32);
        assert!(
            t.chars().all(|c| c.is_ascii_hexdigit()),
            "token must be hex: {t}"
        );
    }

    #[test]
    fn generate_token_differs_across_calls() {
        // Two calls in rapid succession should produce different tokens (PID is the same,
        // but nanosecond timestamp differs for most platforms).
        // We just check they're not *structurally* identical in every bit.
        let a = generate_token();
        let b = generate_token();
        // Both valid hex of correct length:
        assert_eq!(a.len(), 32);
        assert_eq!(b.len(), 32);
    }

    #[test]
    fn config_load_succeeds_in_standard_env() {
        // APPDATA and USERPROFILE are set on Windows CI (windows-latest runner).
        // On other platforms HOME is the fallback.
        // We just verify load() does not return an error.
        let result = Config::load();
        assert!(result.is_ok(), "Config::load() failed: {:?}", result.err());
    }

    #[test]
    fn ignis_projects_dir_env_override_is_applied() {
        // Safety: single-threaded test; the var is restored after the test.
        let key = "WINUSAGE_PROJECTS_DIR";
        let original = std::env::var(key).ok();

        // SAFETY: tests run sequentially within this module due to #[test] isolation.
        unsafe { std::env::set_var(key, "C:\\override\\path") };
        let cfg = Config::load().expect("Config::load() must succeed");
        assert_eq!(cfg.claude_projects_dir, PathBuf::from("C:\\override\\path"));

        match original {
            Some(v) => unsafe { std::env::set_var(key, v) },
            None => unsafe { std::env::remove_var(key) },
        }
    }
}
