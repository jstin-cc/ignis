use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use std::collections::BTreeMap;
use std::path::PathBuf;

/// Stable model identifier. Kept as a String so unknown models never crash the parser.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct ModelId(pub String);

impl std::fmt::Display for ModelId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

impl From<String> for ModelId {
    fn from(s: String) -> Self {
        Self(s)
    }
}

impl From<&str> for ModelId {
    fn from(s: &str) -> Self {
        Self(s.to_owned())
    }
}

/// One billing-relevant API response from a Claude model, derived from a single
/// `"type": "assistant"` JSONL line.
#[derive(Clone, Debug)]
pub struct UsageEvent {
    pub session_id: String,
    pub uuid: String,
    pub timestamp: DateTime<Utc>,
    pub project_path: PathBuf,
    pub git_branch: Option<String>,
    pub model: ModelId,
    pub is_sidechain: bool,
    pub input_tokens: u64,
    pub output_tokens: u64,
    pub cache_read_tokens: u64,
    pub cache_creation_tokens: u64,
    pub cache_creation_ephemeral_5m: u64,
    pub cache_creation_ephemeral_1h: u64,
    pub web_search_requests: u64,
    pub web_fetch_requests: u64,
}

/// Aggregated token and cost metrics for a single model within some time range.
#[derive(Clone, Debug, Default)]
pub struct ModelUsage {
    pub input_tokens: u64,
    pub output_tokens: u64,
    pub cache_read_tokens: u64,
    pub cache_creation_tokens: u64,
    pub cost_usd: Decimal,
    pub event_count: u64,
}

/// Aggregated metrics for a single project (cwd) within some time range.
#[derive(Clone, Debug, Default)]
pub struct ProjectUsage {
    pub total_cost_usd: Decimal,
    pub total_tokens: u64,
    pub session_count: u64,
}

/// All known information about one Claude Code session (one JSONL file).
#[derive(Clone, Debug)]
pub struct SessionState {
    pub session_id: String,
    pub project_path: PathBuf,
    pub git_branch: Option<String>,
    pub first_seen: DateTime<Utc>,
    pub last_seen: DateTime<Utc>,
    pub event_count: u64,
    pub total_cost_usd: Decimal,
    pub by_model: BTreeMap<ModelId, ModelUsage>,
}

/// Aggregated summary for a given time window.
#[derive(Clone, Debug, Default)]
pub struct Summary {
    pub total_cost_usd: Decimal,
    pub total_tokens: u64,
    pub event_count: u64,
    /// Subset of `total_cost_usd` attributable to sidechain (sub-agent) calls.
    pub sidechain_cost_usd: Decimal,
    /// Number of events that are sidechain (sub-agent) calls.
    pub sidechain_event_count: u64,
    pub by_model: BTreeMap<ModelId, ModelUsage>,
    pub by_project: BTreeMap<PathBuf, ProjectUsage>,
}

/// Aggregated cost for a single calendar day — used for the activity heatmap.
#[derive(Clone, Debug)]
pub struct HeatmapDay {
    pub date: chrono::NaiveDate,
    pub cost_usd: Decimal,
}

/// One-minute bucket of token/cost activity, used for the burn-rate sparkline.
#[derive(Clone, Debug)]
pub struct BurnRateBucket {
    pub minute_start: DateTime<Utc>,
    pub tokens: u64,
    pub cost_usd: Decimal,
}

/// One 5-hour billing window inferred from event timestamps.
///
/// Claude Code bills in rolling 5-hour windows. A new block starts with the first
/// event that arrives more than 5 hours after the previous block started, or with
/// the very first event ever seen.
#[derive(Clone, Debug)]
pub struct SessionBlock {
    /// Timestamp of the first event in this block.
    pub start: DateTime<Utc>,
    /// Block closes 5 hours after `start`.
    pub end: DateTime<Utc>,
    pub cost_usd: Decimal,
    pub token_count: u64,
    pub event_count: u64,
}

/// Immutable snapshot handed to consumers (CLI / API / Tray).
///
/// Built by the scanner after every batch of Δ-events. Consumers read this via
/// `Arc<ArcSwap<Snapshot>>` without any locking on the hot path.
#[derive(Clone, Debug)]
pub struct Snapshot {
    pub taken_at: DateTime<Utc>,
    pub today: Summary,
    pub this_week: Summary,
    pub this_month: Summary,
    /// Most recently active session, if any.
    pub active_session: Option<SessionState>,
    /// All known sessions, ordered by `last_seen` descending.
    pub sessions: Vec<SessionState>,
    /// The billing block that contains `taken_at`, if any.
    pub active_block: Option<SessionBlock>,
    /// Model IDs present in events but absent from the pricing table.
    pub pricing_warnings: Vec<ModelId>,
    /// Daily cost aggregates for the last 84 days (activity heatmap).
    pub heatmap: Vec<HeatmapDay>,
    /// Per-minute token/cost buckets for the last 30 minutes (burn-rate sparkline).
    pub burn_rate: Vec<BurnRateBucket>,
}
