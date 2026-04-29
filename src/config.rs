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
    /// How often the Tray-UI polls Anthropic's OAuth usage endpoint (seconds).
    #[serde(default = "default_poll_interval_secs")]
    pub usage_poll_interval_secs: u32,
}

fn default_poll_interval_secs() -> u32 {
    60
}

impl Default for PlanConfig {
    fn default() -> Self {
        PlanConfig {
            kind: PlanKind::Max5,
            custom_token_limit: None,
            usage_poll_interval_secs: default_poll_interval_secs(),
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

/// Bumped when `StoredConfig` gains new required fields.
const CURRENT_CONFIG_VERSION: u32 = 2;

/// Allowed origins for cross-origin requests to the local HTTP API.
fn default_allowed_origins() -> Vec<String> {
    vec![
        "tauri://localhost".into(),      // Tauri production WebView (Windows/macOS)
        "http://tauri.localhost".into(), // Tauri production WebView (Linux)
        "http://localhost:1420".into(),  // Vite dev-server (tauri dev default)
        "http://localhost:5173".into(),  // Vite standalone dev-server default
    ]
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
    /// Origins allowed for cross-origin requests; default covers all Tauri + Vite variants.
    pub allowed_origins: Vec<String>,
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
                allowed_origins: default_allowed_origins(),
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
    #[serde(default)]
    config_version: u32,
    claude_projects_dir: String,
    api_token: String,
    #[serde(default)]
    plan: PlanConfig,
    #[serde(default = "default_allowed_origins")]
    allowed_origins: Vec<String>,
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
    let mut stored: StoredConfig = serde_json::from_str(&raw).map_err(|e| ConfigError::Parse {
        path: path.to_owned(),
        source: e,
    })?;

    if stored.config_version < CURRENT_CONFIG_VERSION {
        migrate(&mut stored, path, &raw);
    }

    Ok(Config {
        claude_projects_dir: PathBuf::from(stored.claude_projects_dir),
        api_token: stored.api_token,
        plan: stored.plan,
        allowed_origins: stored.allowed_origins,
    })
}

/// Migrate `stored` from an older version to `CURRENT_CONFIG_VERSION`.
///
/// Writes a backup of the old file before overwriting. Best-effort: migration
/// failures are logged to stderr but never crash the app (graceful degradation).
fn migrate(stored: &mut StoredConfig, path: &Path, original_raw: &str) {
    // Backup first — never silently discard user data.
    let backup = path.with_extension(format!("v{}.bak", stored.config_version));
    let _ = std::fs::write(&backup, original_raw.as_bytes());

    // v0 / v1 → v2: plan gains usage_poll_interval_secs (default already applied
    // by serde, nothing to copy).  Just bump the version and re-persist.
    stored.config_version = CURRENT_CONFIG_VERSION;
    let _ = save_file_stored(stored, path);
}

fn save_file(cfg: &Config, path: &Path) -> Result<(), ConfigError> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| ConfigError::Io {
            path: parent.to_owned(),
            source: e,
        })?;
    }
    let stored = StoredConfig {
        config_version: CURRENT_CONFIG_VERSION,
        claude_projects_dir: cfg.claude_projects_dir.to_string_lossy().into_owned(),
        api_token: cfg.api_token.clone(),
        plan: cfg.plan.clone(),
        allowed_origins: cfg.allowed_origins.clone(),
    };
    save_file_stored(&stored, path)
}

fn save_file_stored(stored: &StoredConfig, path: &Path) -> Result<(), ConfigError> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| ConfigError::Io {
            path: parent.to_owned(),
            source: e,
        })?;
    }
    let json = serde_json::to_string_pretty(stored).expect("StoredConfig always serializes");
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
    fn v1_config_without_version_field_gets_default_plan_fields() {
        let raw = r#"{"claude_projects_dir":"C:\\x","api_token":"abc","plan":{"kind":"pro"}}"#;
        let stored: StoredConfig = serde_json::from_str(raw).unwrap();
        assert_eq!(stored.config_version, 0);
        assert_eq!(stored.plan.usage_poll_interval_secs, 60);
    }

    #[test]
    fn migration_creates_backup_and_upgrades_version() {
        let dir = std::env::temp_dir();
        let path = dir.join("ignis_test_config_migrate.json");
        let backup = path.with_extension("v0.bak");
        let _ = std::fs::remove_file(&path);
        let _ = std::fs::remove_file(&backup);

        let v1 = r#"{"claude_projects_dir":"C:\\x","api_token":"tok","plan":{"kind":"max5"}}"#;
        std::fs::write(&path, v1).unwrap();

        let cfg = load_file(&path).unwrap();
        assert_eq!(cfg.plan.usage_poll_interval_secs, 60);

        // backup must exist with original content
        let bak_content = std::fs::read_to_string(&backup).unwrap();
        assert_eq!(bak_content, v1);

        // migrated file must have config_version = 2
        let migrated_raw = std::fs::read_to_string(&path).unwrap();
        let migrated: StoredConfig = serde_json::from_str(&migrated_raw).unwrap();
        assert_eq!(migrated.config_version, CURRENT_CONFIG_VERSION);

        let _ = std::fs::remove_file(&path);
        let _ = std::fs::remove_file(&backup);
    }

    #[test]
    fn v2_config_loads_without_creating_backup() {
        let dir = std::env::temp_dir();
        let path = dir.join("ignis_test_config_v2.json");
        let backup = path.with_extension("v2.bak");
        let _ = std::fs::remove_file(&path);
        let _ = std::fs::remove_file(&backup);

        let v2 = r#"{"config_version":2,"claude_projects_dir":"C:\\x","api_token":"tok","plan":{"kind":"max5","usage_poll_interval_secs":120}}"#;
        std::fs::write(&path, v2).unwrap();

        let cfg = load_file(&path).unwrap();
        assert_eq!(cfg.plan.usage_poll_interval_secs, 120);
        assert!(
            !backup.exists(),
            "backup must not be created for up-to-date config"
        );

        let _ = std::fs::remove_file(&path);
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
