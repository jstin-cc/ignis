//! Provider trait — abstracts a source of AI-editor usage events.
//!
//! Implement `Provider` to add support for additional AI code editors
//! (e.g. Cursor, GitHub Copilot) that emit usage logs in a compatible format.
//! `ClaudeCodeProvider` is the reference implementation for Claude Code.

use crate::parser::ParseError;
use crate::scanner::{scan_all, ScanResult};
use crate::{model::UsageEvent, parser};
use std::path::{Path, PathBuf};

/// Abstracts a source of AI-editor usage events.
///
/// A provider knows:
/// - **where** its log files live (`data_root`)
/// - **how** to parse one raw log line (`parse_line`)
///
/// The default `collect` implementation delegates to `scanner::scan_all`. Override it
/// for custom file-discovery or filtering logic.
pub trait Provider: Send + Sync {
    /// Human-readable provider name, e.g. `"claude-code"` or `"cursor"`.
    fn name(&self) -> &str;

    /// Root directory that contains this provider's JSONL log files.
    fn data_root(&self) -> &Path;

    /// Parse one raw log line into a `UsageEvent`.
    ///
    /// Return `Ok(None)` to silently skip a line (non-billing events, synthetic
    /// entries). Return `Err` only for structurally malformed input.
    fn parse_line(&self, raw: &str) -> Result<Option<UsageEvent>, ParseError>;

    /// Collect all events from this provider via a full directory scan.
    ///
    /// The default implementation calls `scanner::scan_all(self.data_root())`.
    /// Override to apply custom discovery logic (e.g. a different glob pattern).
    fn collect(&self) -> ScanResult {
        scan_all(self.data_root())
    }
}

/// Provider for Claude Code (`~/.claude/projects/`).
pub struct ClaudeCodeProvider {
    root: PathBuf,
}

impl ClaudeCodeProvider {
    pub fn new(root: impl Into<PathBuf>) -> Self {
        Self { root: root.into() }
    }
}

impl Provider for ClaudeCodeProvider {
    fn name(&self) -> &str {
        "claude-code"
    }

    fn data_root(&self) -> &Path {
        &self.root
    }

    fn parse_line(&self, raw: &str) -> Result<Option<UsageEvent>, ParseError> {
        parser::parse_line(raw)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn claude_code_provider_name() {
        let p = ClaudeCodeProvider::new("/tmp/test");
        assert_eq!(p.name(), "claude-code");
    }

    #[test]
    fn claude_code_provider_data_root() {
        let p = ClaudeCodeProvider::new("/tmp/test");
        assert_eq!(p.data_root(), Path::new("/tmp/test"));
    }

    #[test]
    fn parse_line_delegates_to_parser() {
        let p = ClaudeCodeProvider::new("/tmp/test");
        // malformed JSON must return Err
        assert!(p.parse_line("{not json}").is_err());
        // valid JSON without usage field must return Ok(None)
        let ok = p.parse_line(r#"{"type":"user","message":{}}"#);
        assert!(matches!(ok, Ok(None)));
    }
}
