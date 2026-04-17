use crate::model::UsageEvent;
use serde::Deserialize;
use std::path::PathBuf;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ParseError {
    #[error("invalid JSON: {0}")]
    Json(#[from] serde_json::Error),
}

/// Parse one JSONL line.
///
/// Returns `Ok(None)` for lines that carry no billing information (non-assistant,
/// synthetic messages, API errors). Returns `Ok(Some(_))` for billable events.
/// Returns `Err` only for malformed JSON — callers should log and skip.
pub fn parse_line(line: &str) -> Result<Option<UsageEvent>, ParseError> {
    let raw: RawLine = serde_json::from_str(line)?;

    if raw.line_type.as_deref() != Some("assistant") {
        return Ok(None);
    }
    let msg = match raw.message {
        Some(m) => m,
        None => return Ok(None),
    };
    // Skip synthetic placeholders and API error pseudo-messages.
    if msg.model.as_deref() == Some("<synthetic>") || raw.is_api_error_message {
        return Ok(None);
    }
    let model_str = match msg.model {
        Some(m) => m,
        None => return Ok(None),
    };
    let usage = match msg.usage {
        Some(u) => u,
        None => return Ok(None),
    };

    let (ephemeral_5m, ephemeral_1h) = match &usage.cache_creation {
        Some(cc) => (cc.ephemeral_5m, cc.ephemeral_1h),
        // Fallback: treat the top-level sum as 5m (cheaper, see docs/pricing.md §4).
        None => (usage.cache_creation_input_tokens, 0),
    };

    let (web_search, web_fetch) = match &usage.server_tool_use {
        Some(s) => (s.web_search_requests, s.web_fetch_requests),
        None => (0, 0),
    };

    let cwd = raw.cwd.unwrap_or_default();

    Ok(Some(UsageEvent {
        session_id: raw.session_id.unwrap_or_default(),
        uuid: raw.uuid.unwrap_or_default(),
        timestamp: raw
            .timestamp
            .and_then(|s| s.parse().ok())
            .unwrap_or_else(chrono::Utc::now),
        project_path: PathBuf::from(&cwd),
        git_branch: raw.git_branch,
        model: model_str.into(),
        is_sidechain: raw.is_sidechain,
        input_tokens: usage.input_tokens,
        output_tokens: usage.output_tokens,
        cache_read_tokens: usage.cache_read_input_tokens,
        cache_creation_tokens: usage.cache_creation_input_tokens,
        cache_creation_ephemeral_5m: ephemeral_5m,
        cache_creation_ephemeral_1h: ephemeral_1h,
        web_search_requests: web_search,
        web_fetch_requests: web_fetch,
    }))
}

// ── Private deserialization types ────────────────────────────────────────────

#[derive(Deserialize)]
struct RawLine {
    #[serde(rename = "type")]
    line_type: Option<String>,
    #[serde(rename = "isSidechain", default)]
    is_sidechain: bool,
    #[serde(rename = "isApiErrorMessage", default)]
    is_api_error_message: bool,
    #[serde(rename = "sessionId")]
    session_id: Option<String>,
    uuid: Option<String>,
    timestamp: Option<String>,
    cwd: Option<String>,
    #[serde(rename = "gitBranch")]
    git_branch: Option<String>,
    message: Option<RawMessage>,
}

#[derive(Deserialize)]
struct RawMessage {
    model: Option<String>,
    usage: Option<RawUsage>,
}

#[derive(Deserialize)]
struct RawUsage {
    #[serde(default)]
    input_tokens: u64,
    #[serde(default)]
    output_tokens: u64,
    #[serde(default)]
    cache_read_input_tokens: u64,
    #[serde(default)]
    cache_creation_input_tokens: u64,
    cache_creation: Option<RawCacheCreation>,
    server_tool_use: Option<RawServerToolUse>,
}

#[derive(Deserialize)]
struct RawCacheCreation {
    #[serde(rename = "ephemeral_5m_input_tokens", default)]
    ephemeral_5m: u64,
    #[serde(rename = "ephemeral_1h_input_tokens", default)]
    ephemeral_1h: u64,
}

#[derive(Deserialize)]
struct RawServerToolUse {
    #[serde(default)]
    web_search_requests: u64,
    #[serde(default)]
    web_fetch_requests: u64,
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    fn fixture(name: &str) -> Vec<String> {
        let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("fixtures")
            .join(name);
        std::fs::read_to_string(&path)
            .unwrap_or_else(|e| panic!("failed to read fixture {name}: {e}"))
            .lines()
            .map(str::to_owned)
            .collect()
    }

    #[test]
    fn happy_path_parses_two_assistant_events() {
        let lines = fixture("happy-path.jsonl");
        let events: Vec<_> = lines
            .iter()
            .filter_map(|l| parse_line(l).unwrap())
            .collect();

        assert_eq!(events.len(), 2);

        let first = &events[0];
        assert_eq!(first.model.0, "claude-sonnet-4-6");
        assert_eq!(first.session_id, "aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa");
        assert!(!first.is_sidechain);
        assert_eq!(first.input_tokens, 18);
        assert_eq!(first.output_tokens, 74);
        assert_eq!(first.cache_read_tokens, 0);
        assert_eq!(first.cache_creation_ephemeral_5m, 4200);
        assert_eq!(first.cache_creation_ephemeral_1h, 0);

        let second = &events[1];
        assert_eq!(second.input_tokens, 6);
        assert_eq!(second.cache_read_tokens, 4200);
        assert_eq!(second.cache_creation_ephemeral_5m, 96);
    }

    #[test]
    fn synthetic_error_is_skipped() {
        let lines = fixture("error-synthetic.jsonl");
        let events: Vec<_> = lines
            .iter()
            .filter_map(|l| parse_line(l).unwrap())
            .collect();
        assert_eq!(
            events.len(),
            0,
            "synthetic error must not produce a UsageEvent"
        );
    }

    #[test]
    fn sidechain_event_is_included_with_flag_set() {
        let lines = fixture("sidechain.jsonl");
        let events: Vec<_> = lines
            .iter()
            .filter_map(|l| parse_line(l).unwrap())
            .collect();

        assert_eq!(events.len(), 1);
        let ev = &events[0];
        assert!(ev.is_sidechain);
        assert_eq!(ev.model.0, "claude-haiku-4-5-20251001");
        assert_eq!(ev.input_tokens, 12);
        assert_eq!(ev.cache_read_tokens, 2800);
    }

    #[test]
    fn user_lines_are_skipped() {
        // user and tool_result lines must not produce events.
        let line = r#"{"type":"user","message":{"role":"user","content":"hello"},"uuid":"x","sessionId":"y","timestamp":"2026-04-17T09:00:00Z","cwd":"C:\\test","isSidechain":false}"#;
        assert!(parse_line(line).unwrap().is_none());
    }

    #[test]
    fn malformed_json_returns_error() {
        assert!(parse_line("{not valid json").is_err());
    }

    #[test]
    fn assistant_without_usage_is_skipped() {
        let line = r#"{"type":"assistant","isSidechain":false,"message":{"model":"claude-sonnet-4-6"},"uuid":"z","sessionId":"s","timestamp":"2026-04-17T09:00:00Z","cwd":"C:\\test"}"#;
        assert!(parse_line(line).unwrap().is_none());
    }
}
